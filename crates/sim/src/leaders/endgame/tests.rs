use super::*;
use crate::world::{Cell, Terrain, World};

/// A two-player sim on a 60×60 map, row 0 a sea and the rest one land
/// region (the territory arm's `land_size` counts regions from 1).
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

/// `Leader::init`'s image: `defeated_by` −1, every other leaf 0.
#[test]
fn the_init_image_is_minus_one_and_zeros() {
    let s = sim();
    let v: Vec<i64> = ENDGAME_LEAVES
        .iter()
        .map(|k| s.leader_endgame_leaf(1, k).unwrap())
        .collect();
    assert_eq!(v, [-1, 0, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(s.leader_endgame_leaf(1, "frame_battle"), None);
}

/// The scaling every `Game::*_timer` shares: `(35 + xs·t) / 70`, at least
/// 1, and 0 for a constant below 1. A 60-wide map's wonder clock is 3857
/// frames, its territory clock and its capital countdown 3086.
#[test]
fn the_timers_scale_by_the_map_s_width_over_the_standard_size() {
    assert_eq!(scaled_timer(4500, 60), 3857);
    assert_eq!(scaled_timer(3600, 60), 3086);
    assert_eq!(scaled_timer(3600, 70), 3600);
    assert_eq!(scaled_timer(0, 60), 0);
    assert_eq!(scaled_timer(1, 1), 1, "never below one");
    let s = sim();
    assert_eq!((s.wonder_timer(), s.popwin_timer()), (3857, 3086));
}

/// **§10.2 step 1**: the captor stamp runs ahead of the defeated guard. A
/// resign stamps the frame and type 6 with no captor; a later conquest
/// re-stamps and records the captor while `defeated_by` is −1, but the
/// type stays write-once; once a captor is recorded nothing re-stamps.
#[test]
fn the_captor_stamp_runs_before_the_guard_and_the_type_is_write_once() {
    let mut s = sim();
    s.frame = 5000;
    s.resign(0);
    let e = s.leader_endgame(0);
    assert_eq!(
        (e.defeated_by, e.defeat_stamp, e.defeat_type),
        (-1, 5000, DEFEAT_RESIGN)
    );
    s.frame = 5638;
    s.defeat(0, DEFEAT_CONQUEST, Some(1));
    let e = s.leader_endgame(0);
    assert_eq!(
        (e.defeated_by, e.defeat_stamp, e.defeat_type),
        (1, 5638, DEFEAT_RESIGN),
        "run609's closing record: type 6, stamp and captor of the later call"
    );
    s.frame = 6000;
    s.defeat(0, DEFEAT_CASCADE, None);
    assert_eq!(s.leader_endgame(0).defeat_stamp, 5638, "a captor stops it");
}

/// **The conquest's stamp**: a leader defeated by its captor on the
/// frame its last city falls (run1200: `defeated_by 1`, `defeat_stamp
/// 5639`, `defeat_type 0`).
#[test]
fn a_conquest_records_the_captor_and_the_frame() {
    let mut s = sim();
    s.frame = 5639;
    s.defeat(0, DEFEAT_CONQUEST, Some(1));
    let e = s.leader_endgame(0);
    assert_eq!((e.defeated_by, e.defeat_stamp, e.defeat_type), (1, 5639, 0));
    assert!(s.leaders.defeated[0]);
}

/// **The cascade**: a winner's unallied rival is defeated, type 5, with
/// no captor, on the winner's frame (run600's 17380: slot 1
/// `victory_type` 2, slot 0 `defeat_type` 5, `defeat_stamp` 17380,
/// `defeated_by` −1). A second victory changes nothing.
#[test]
fn a_territory_victory_defeats_the_unallied_rival_by_the_cascade() {
    let mut s = sim();
    s.frame = 17_380;
    s.victory(1, VICTORY_TERRITORY);
    let (w, l) = (s.leader_endgame(1), s.leader_endgame(0));
    assert_eq!((w.won, w.victory_type), (true, 2));
    assert_eq!(
        (l.defeated_by, l.defeat_stamp, l.defeat_type, l.victory_type),
        (-1, 17_380, DEFEAT_CASCADE, 0)
    );
    s.victory(1, VICTORY_WONDER);
    assert_eq!(s.leader_endgame(1).victory_type, 2);
}

/// **An ally wins with the winner**, and is not defeated.
#[test]
fn an_ally_wins_beside_the_winner() {
    let mut s = Sim::new(crate::Tuning::RON, World::new(60, 60), 3);
    s.leaders.allied[1][2] = true;
    s.leaders.allied[2][1] = true;
    s.frame = 900;
    s.victory(1, VICTORY_TERRITORY);
    assert_eq!(s.leader_endgame(2).victory_type, 2);
    assert!(s.leader_endgame(2).won);
    assert!(!s.leaders.defeated[2]);
    assert_eq!(s.leader_endgame(0).defeat_type, DEFEAT_CASCADE);
}

/// **The territory expiry**: a running clock wins once
/// `popwin_timer() − (frame − stamp)` is below one, not before.
#[test]
fn the_territory_clock_wins_when_it_runs_out() {
    let mut s = sim();
    let land = s.land_size();
    s.economy.holdings[1].territory = land * 70 / 100 + 1;
    s.frame = 1000;
    s.process_victory_territory();
    assert_eq!(s.ai.ai[1].popwin_stamp, 1000);
    s.frame = 1000 + 3085;
    s.process_victory_territory();
    assert!(!s.leader_endgame(1).won, "one frame left");
    s.frame = 1000 + 3086;
    s.process_victory_territory();
    assert_eq!(s.leader_endgame(1).victory_type, VICTORY_TERRITORY);
    assert_eq!(s.leader_endgame(0).defeat_stamp, 4086);
}

/// **The wonder arm's clock**: with no wonder leader every clock is off;
/// a stop banks the run as a debt, which the pay-back returns one frame in
/// five while stopped.
#[test]
fn the_wonder_clock_banks_a_stop_and_pays_it_back() {
    let mut s = sim();
    s.leaders.endgame[1].wonderwin_timer = 1;
    s.leaders.endgame[1].wonderwin_stamp = 5037;
    s.frame = 5100;
    s.process_victory_wonder();
    let e = s.leader_endgame(1);
    assert_eq!(
        (e.wonderwin_timer, e.wonderwin_stamp),
        (0, -63),
        "no wonder leader"
    );
    s.frame = 5105;
    s.refresh_wonder_timer(1);
    assert_eq!(s.leader_endgame(1).wonderwin_stamp, -62);
    s.frame = 5106;
    s.refresh_wonder_timer(1);
    assert_eq!(s.leader_endgame(1).wonderwin_stamp, -62);
}
