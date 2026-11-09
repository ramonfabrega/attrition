use crate::Sim;
use crate::world::{Cell, Terrain, World};

/// A two-player sim on a 10×10 map, region 0 a sea (the listing's sum
/// starts at region 1) and region 1 the land, at war.
fn sim() -> Sim {
    let mut w = World::new(10, 10);
    let sea = w.add_region(Terrain::Sea);
    let land = w.add_region(Terrain::Land);
    for y in 0..10 {
        for x in 0..10 {
            w.set_region(Cell::new(x, y), if y == 0 { sea } else { land });
        }
    }
    Sim::new(crate::Tuning::RON, w, 2)
}

/// Gives `who` a territory of `n` cells: `LeaderData::territory`, as
/// `Leader::calc_gather` leaves it.
fn own_cells(s: &mut Sim, who: usize, n: i32) {
    s.economy.holdings[who].territory = n;
}

#[test]
fn a_team_over_the_share_starts_its_timer_on_the_frame() {
    let mut s = sim();
    let land = s.land_size();
    own_cells(&mut s, 1, land * 70 / 100 + 1);
    s.frame = 14_293;
    s.process_victory_territory();
    assert_eq!(
        (s.ai.ai[1].popwin_timer, s.ai.ai[1].popwin_stamp),
        (1, 14_293)
    );
    assert_eq!((s.ai.ai[0].popwin_timer, s.ai.ai[0].popwin_stamp), (0, 0));
    // Running: the stamp stays.
    s.frame = 14_300;
    s.process_victory_territory();
    assert_eq!(s.ai.ai[1].popwin_stamp, 14_293);
}

#[test]
fn a_team_under_the_share_does_not_start() {
    let mut s = sim();
    let land = s.land_size();
    own_cells(&mut s, 1, land * 70 / 100 - 1);
    s.frame = 100;
    s.process_victory_territory();
    assert_eq!(s.ai.ai[1].popwin_timer, 0);
}

#[test]
fn a_stop_leaves_the_time_run_as_a_debt_paid_back_one_a_fifth() {
    let mut s = sim();
    let land = s.land_size();
    own_cells(&mut s, 1, land * 70 / 100 + 1);
    s.frame = 1000;
    s.process_victory_territory();
    own_cells(&mut s, 1, 1);
    s.frame = 1100;
    s.process_victory_territory();
    assert_eq!(
        (s.ai.ai[1].popwin_timer, s.ai.ai[1].popwin_stamp),
        (0, -100)
    );
    s.frame = 1105;
    s.refresh_victory_timers(1);
    assert_eq!(s.ai.ai[1].popwin_stamp, -99);
    s.frame = 1106;
    s.refresh_victory_timers(1);
    assert_eq!(
        s.ai.ai[1].popwin_stamp, -99,
        "only on a multiple of the ratio"
    );
}

/// **The land is counted by the original's region number, not this
/// crate's index** (item 3575, `docs/AI.md` §311). A loader that
/// meets an all-land map's region 1 first declares it as index 0;
/// summed by index, Great Sahara's land was 0 and who=1's timer never
/// started on 8195. Here the land is index 0 numbered 1, and an empty
/// index 1 numbered 0 stands for the coastal `region2` the loader
/// declares beside it.
#[test]
fn a_land_region_declared_first_is_counted_by_its_number() {
    let mut w = World::new(10, 10);
    let land = w.add_region(Terrain::Land);
    let none = w.add_region(Terrain::Land);
    w.set_region_number(land, 1);
    w.set_region_number(none, 0);
    for y in 0..10 {
        for x in 0..10 {
            w.set_region(Cell::new(x, y), land);
        }
    }
    let mut s = Sim::new(crate::Tuning::RON, w, 2);
    assert_eq!(s.land_size(), 100);
    own_cells(&mut s, 1, 71);
    s.frame = 8195;
    s.process_victory_territory();
    assert_eq!(
        (s.ai.ai[1].popwin_timer, s.ai.ai[1].popwin_stamp),
        (1, 8195)
    );
}

#[test]
fn a_victory_other_than_standard_keeps_no_timer() {
    let mut s = sim();
    let land = s.land_size();
    own_cells(&mut s, 1, land * 70 / 100 + 1);
    s.ai.lobby.victory = 3;
    s.process_victory_territory();
    assert_eq!(s.ai.ai[1].popwin_timer, 0);
}
