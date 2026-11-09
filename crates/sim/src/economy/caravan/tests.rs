use super::*;

fn bare() -> Sim {
    Sim::new(crate::Tuning::RON, crate::world::World::new(8, 8), 2)
}

#[test]
fn a_slot_is_reused_and_the_mark_walks_back_down() {
    let mut sim = bare();
    let a = sim.init_caravan(1, 7).expect("a free slot");
    let b = sim.init_caravan(1, 8).expect("a free slot");
    assert_eq!((a, b), (0, 1));
    assert_eq!(sim.economy.caravans[1].mark, 2);
    sim.close_caravan(1, 0);
    // The mark stays where the higher slot holds it.
    assert_eq!(sim.economy.caravans[1].mark, 2);
    assert_eq!(sim.init_caravan(1, 9), Some(0), "the hole is reused first");
    sim.close_caravan(1, 1);
    sim.close_caravan(1, 0);
    assert_eq!(
        sim.economy.caravans[1].mark, 0,
        "and then it walks back down"
    );
}

#[test]
fn twenty_slots_and_no_more() {
    let mut sim = bare();
    for i in 0..CARAVAN_SLOTS {
        assert_eq!(sim.init_caravan(0, i), Some(i));
    }
    assert_eq!(sim.init_caravan(0, 99), None, "the list is twenty long");
}

fn node(x: i32, y: i32) -> PathData {
    PathData {
        to: Pos::new(x, y),
        tolerance: 0x60,
        flags: 0,
    }
}

/// §7.1: the displacement is the step turned a quarter turn, and the
/// step is measured from where the *search* left the node before.
///
/// Written from run64's own road, whose last three nodes are the
/// straight north run `(38496, 39264)`, `(38496, 39456)`,
/// `(38496, 39648)`: a step of `(0, 192)` puts `+64` on x and nothing
/// on y, twice over, and the second `+64` is off `38496` and not off
/// the `38560` the first one wrote.
#[test]
fn the_offset_is_perpendicular_and_does_not_compound() {
    let mut p = vec![node(38496, 39264), node(38496, 39456), node(38496, 39648)];
    offset_road(&mut p);
    assert_eq!(p[0].to, Pos::new(38496, 39264), "the first node is fixed");
    assert_eq!(p[1].to, Pos::new(38560, 39456));
    assert_eq!(p[2].to, Pos::new(38560, 39648));

    // The other axis, and the sign: a step due **east** displaces
    // south by `−dx/3`, which is negative y.
    let mut q = vec![node(0, 0), node(192, 0)];
    offset_road(&mut q);
    assert_eq!(q[1].to, Pos::new(192, -64));

    // Truncation is toward zero, as the original's two divisions are.
    let mut r = vec![node(0, 0), node(-100, -100)];
    offset_road(&mut r);
    assert_eq!(r[1].to, Pos::new(-100 - 33, -100 + 33));
}

/// A step wider than one tile on either axis is left where it is —
/// which is the arm a sampled water run takes, since the sampling
/// leaves gaps of five tiles.
#[test]
fn a_step_past_one_tile_is_not_displaced() {
    let mut p = vec![node(0, 0), node(0, 0xc0), node(0, 0xc0 + 0xc1)];
    offset_road(&mut p);
    assert_eq!(p[1].to, Pos::new(0x40, 0xc0), "0xc0 exactly is inside");
    assert_eq!(p[2].to, Pos::new(0, 0xc0 + 0xc1), "0xc1 is not");
}

/// §7.3: the one-off is paid once per (city, leader, partner), and it
/// is `(epoch[2] + 1) · 10` — **Commerce**, `LeaderDataEncrypt +0xf0`
/// — twenty when the payee is not the city's own owner. The Civic
/// level is set to something else on purpose: until item 573 this
/// crate read it, and this test set only it.
#[test]
fn the_arrival_bonus_is_paid_once_and_doubles_for_a_foreigner() {
    let mut sim = bare();
    sim.cities.cities.push(crate::cities::city::City {
        alive: true,
        owner: 0,
        race: Some(0),
        founder: 0,
        building: 0,
        members: Vec::new(),
        reg: None,
        pos: Pos::new(0, 0),
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
        pop: 0,
        has_citizen: false,
        source: None,
        trade_val: 0,
        traded_with: [0; 8],
    });
    let wealth = crate::economy::Resource::Wealth as usize;
    for l in &mut sim.economy.ledgers {
        l.bucket[wealth] = 0;
    }
    for p in 0..2 {
        sim.tech[p].epoch[crate::tech::Line::Commerce.index()] = 2;
        sim.tech[p].epoch[crate::tech::Line::Civic.index()] = 5;
    }

    sim.city_new_caravan(0, 0, 5);
    assert_eq!(
        sim.economy.ledgers[0].bucket[wealth], 30,
        "(2 + 1) × 10, the owner's"
    );
    sim.city_new_caravan(0, 0, 5);
    assert_eq!(
        sim.economy.ledgers[0].bucket[wealth], 30,
        "the bit is already set"
    );
    sim.city_new_caravan(0, 0, 6);
    assert_eq!(
        sim.economy.ledgers[0].bucket[wealth], 60,
        "a different partner pays"
    );
    sim.city_new_caravan(0, 1, 5);
    assert_eq!(
        sim.economy.ledgers[1].bucket[wealth], 60,
        "(2 + 1) × 20 for a leader who does not own the city"
    );
}

/// **`do_move`'s road check** (§10): a waypoint the road was laid on
/// asks only when its tile is no longer road *and* is built on or
/// blocked. Then the route is verified — a built-on tile of the road
/// stack starts `build_road` over, which with no cities empties the
/// stack — and every waypoint of the walk loses the flag. Great Lakes'
/// `1/23` on 14529: (220, 101), placed on by the Barracks and eroded.
///
/// Made to fail once with the surface test inverted: the road tile
/// verified too.
#[test]
fn a_road_waypoint_built_over_verifies_the_route_and_drops_the_flag() {
    use crate::world::tile;
    let mut sim = bare();
    let u = sim.add_unit(crate::Unit::new(1, 0, Pos::new(960, 960), 10));
    let v = sim.init_caravan(1, u).expect("a slot");
    sim.units[u].caravan = Some(v);
    let (road, built, open) = (
        Pos::new(1056, 864),
        Pos::new(1056, 1056),
        Pos::new(1248, 1056),
    );
    let flagged = |p: Pos| PathData {
        to: p,
        tolerance: 0x60,
        flags: path_flag::ROAD_LAID,
    };
    sim.units[u].path = vec![flagged(road), flagged(built)];
    sim.economy.caravans[1].slots[v].road = vec![flagged(road), flagged(built)];
    let m = sim.world.tile_mask(road.tile());
    sim.world.set_tile_mask(road.tile(), m | tile::SURFACE_ROAD);
    let m = sim.world.tile_mask(built.tile());
    sim.world.set_tile_mask(built.tile(), m | tile::PLACED);
    // Still a road: nothing asked.
    sim.caravan_road_step(u, road);
    assert!(
        sim.units[u]
            .path
            .iter()
            .all(|p| p.flags & path_flag::ROAD_LAID != 0)
    );
    // Plain ground, nothing on it: nothing asked either.
    sim.caravan_road_step(u, open);
    assert_eq!(sim.economy.caravans[1].slots[v].road.len(), 2);
    // Built over and not road: verified, and the flag is gone.
    sim.caravan_road_step(u, built);
    assert!(
        sim.units[u]
            .path
            .iter()
            .all(|p| p.flags & path_flag::ROAD_LAID == 0),
        "every waypoint loses the flag"
    );
    assert!(
        sim.economy.caravans[1].slots[v].road.is_empty(),
        "the route is planned again"
    );
}
/// **A restart consumes the reset flag** (item 1581, `docs/AI.md`
/// §157): `LAB_0073dbf5` writes `+0x24 = 0` as it throws the parked
/// search away, so the search the restart parks is resumed next frame —
/// Great Sahara's caravan slot 7 from 1817, 3,201 nodes where the
/// original's second frame was its 3,204.
///
/// Made to fail once with the flag left set.
#[test]
fn a_restart_consumes_the_reset_flag() {
    let mut van = Caravan {
        making_road: true,
        reset_road: true,
        ..Caravan::default()
    };
    assert!(!Sim::road_gate(&mut van), "a flagged route starts over");
    assert!(
        !van.reset_road && !van.making_road && van.search.is_none(),
        "and the flag goes with the search it threw away"
    );
    // A route with nothing parked starts over too, flag or none.
    let mut bare = Caravan {
        making_road: true,
        ..Caravan::default()
    };
    assert!(!Sim::road_gate(&mut bare));
    assert!(!bare.making_road);
}
