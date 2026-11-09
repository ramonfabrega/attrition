use super::*;
use crate::world::{Cell, World};

/// A world of `n` land regions, each one row of `w` cells.
fn rows(w: i32, n: i32) -> World {
    let mut world = World::new(w, n);
    for y in 0..n {
        world.fill_region(Terrain::Land, Cell::new(0, y), Cell::new(w - 1, y));
    }
    world
}

/// **The pass writes 256 cells a frame, regions in order, each from its
/// resume index**, and a cell it has not reached keeps its old owner:
/// the shape run407's packet reads on East Indies' 5518..5521.
#[test]
fn a_fix_reaches_the_map_at_256_cells_a_frame_in_region_order() {
    let mut sim = Sim::new(crate::Tuning::RON, rows(200, 3), 2);
    sim.in_play = true;
    let before = sim.world.owners();
    let mut target = before.clone();
    target.fill((Owner::Player(1), Owner::None));
    sim.world.set_owners(&target);
    sim.start_border_pass(before);
    assert_eq!(
        sim.world.owner(Cell::new(0, 0)),
        Owner::None,
        "a fix writes no cell"
    );
    sim.advance_border_pass();
    // Frame one: region 0 whole (200) and region 1's first 56.
    assert_eq!(sim.world.owner(Cell::new(199, 0)), Owner::Player(1));
    assert_eq!(sim.world.owner(Cell::new(55, 1)), Owner::Player(1));
    assert_eq!(
        sim.world.owner(Cell::new(56, 1)),
        Owner::None,
        "the budget ends it"
    );
    assert_eq!(
        sim.territory.border_pass.as_ref().unwrap().index,
        vec![200, 56, 0]
    );
    sim.advance_border_pass();
    // Frame two: region 1's other 144, region 2's first 112.
    assert_eq!(
        sim.territory.border_pass.as_ref().unwrap().index,
        vec![200, 200, 112]
    );
    assert_eq!(sim.world.owner(Cell::new(111, 2)), Owner::Player(1));
    assert_eq!(sim.world.owner(Cell::new(112, 2)), Owner::None);
    sim.advance_border_pass();
    assert!(
        sim.territory.border_pass.is_none(),
        "the third frame finishes it"
    );
    assert_eq!(sim.world.owner(Cell::new(199, 2)), Owner::Player(1));
}

/// **A region the pass has not reached is zeroed again every frame**
/// (`compute_reg_territory@006b0bb0:78`–`101`, item 3594): a census
/// between two frames of the pass keeps its count only in the regions
/// the pass has started — run1241's who=1, whose tick-5575 census was
/// undone on regions 7, 9 and 11 (`docs/AI.md` §318.1).
#[test]
fn a_census_inside_the_pass_is_undone_for_the_regions_still_waiting() {
    let mut sim = Sim::new(crate::Tuning::RON, rows(200, 3), 2);
    sim.in_play = true;
    let before = sim.world.owners();
    let mut target = before.clone();
    target.fill((Owner::Player(1), Owner::None));
    sim.world.set_owners(&target);
    sim.start_border_pass(before);
    sim.advance_border_pass();
    // The census refills every region between frames one and two.
    let n = sim.world.region_count();
    sim.ai_state.ai[1].census.reg_known_rares = vec![1; n.max(3)];
    sim.advance_border_pass();
    let r = |c: Cell| usize::from(sim.world.region_of(c).unwrap());
    let rares = &sim.ai_state.ai[1].census.reg_known_rares;
    assert_eq!(
        rares[r(Cell::new(0, 0))],
        1,
        "region 0 finished on frame one"
    );
    assert_eq!(
        rares[r(Cell::new(0, 1))],
        1,
        "region 1 was started on frame one"
    );
    assert_eq!(
        rares[r(Cell::new(0, 2))],
        0,
        "region 2 still stood at index 0 when frame two reached it"
    );
}
/// A city of `who`'s, as `init_city` records it.
fn city_of(who: crate::Player) -> crate::city::City {
    crate::city::City {
        alive: true,
        owner: who,
        race: None,
        founder: who,
        building: 0,
        members: Vec::new(),
        reg: None,
        pos: crate::Pos::new(0, 0),
        capital: false,
        founding_capital: false,
        was_founding_capital: false,
        unassimilated: false,
        no_heal: false,
        attacking: false,
        ever_attacked: false,
        alarm: false,
        no_muster: false,
        was_capital: 0,
        capture_stamp: 0,
        assimilation_timer: 0,
        attack_stamp: 0,
        reduce_stamp: 0,
        raid_stamp: 0,
        capture_strength: 0,
        pop: 1,
        has_citizen: false,
        source: None,
        trade_val: 0,
        traded_with: [0; 8],
    }
}

/// **A rival's runner-up cell marks the winner's city `bordering`**
/// (`compute_reg_territory:595`–`607`, `docs/AI.md` §167): both
/// players' bits, only when a side is at war, only for a city's win
/// (a fort's, `−1`, marks nothing), and cleared when the region's pass
/// starts. Great Sahara's Persian city `1/2018` reads 3 from 1980.
///
/// Made to fail once with the war test removed and once with the clear
/// removed.
#[test]
fn a_rival_s_runner_up_marks_the_winning_city_s_bordering() {
    let mut sim = Sim::new(crate::Tuning::RON, rows(8, 1), 2);
    sim.cities_state.cities.push(city_of(1));
    sim.in_play = true;
    let run = |sim: &mut Sim, won: i32| {
        let before = sim.world.owners();
        let mut target = before.clone();
        target.fill((Owner::Player(1), Owner::Player(0)));
        sim.world.set_owners(&target);
        sim.territory.territory_won = vec![won; 8];
        sim.start_border_pass(before);
        sim.advance_border_pass();
    };
    let bordering = |sim: &Sim| {
        sim.ai_state.ai[1]
            .city_ai
            .first()
            .map_or(0, |c| c.bordering)
    };
    run(&mut sim, 0);
    assert_eq!(bordering(&sim), 0, "at peace, nothing is marked");
    sim.declare_war(0, 1);
    run(&mut sim, -1);
    assert_eq!(bordering(&sim), 0, "a fort's win marks no city");
    run(&mut sim, 0);
    assert_eq!(bordering(&sim), 0b11, "both players' bits");
    // A second fix clears it before it marks again: with the runner-up
    // gone the city reads 0.
    let before = sim.world.owners();
    let mut target = before.clone();
    target.fill((Owner::Player(1), Owner::None));
    sim.world.set_owners(&target);
    sim.territory.territory_won = vec![0; 8];
    sim.start_border_pass(before);
    sim.advance_border_pass();
    assert_eq!(bordering(&sim), 0, "the region's first call clears it");
}
