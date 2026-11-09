use super::*;

const T: Tuning = Tuning::RON;

fn wagon(x: i32, y: i32) -> Wagon {
    Wagon {
        pos: Pos::new(x, y),
        active: true,
        on_map: true,
    }
}

#[test]
fn a_plain_wagon_reaches_fourteen_tiles_and_a_researched_one_twenty() {
    assert_eq!(supply_radius(&T, 0, false), 14);
    assert_eq!(supply_radius(&T, 3, false), 20);
    // The wonder is wired in and ships contributing nothing.
    assert_eq!(supply_radius(&T, 0, true), 14);
}

/// Item 3641, run1299's frame 5324: who=1's Bombard `1/98` at (7608,
/// 10680) and its Supply Wagon `1/102` at (8040, 7977), 2737 apart.
/// The network's answer reads the research it is handed: out at the
/// plain 14 tiles (2688), in at the 16 one `SUPPLY_WAGONS_*` step
/// gives (3072; who=1's Forage) — and so the Bombard reloads 60
/// rather than 120.
#[test]
fn the_network_s_radius_is_the_research_it_is_asked_with() {
    let mut net = Network::default();
    net.list.register(0);
    let bombard = Pos::new(7608, 10680);
    let at = |upgrades| net.supplies(&T, upgrades, bombard, |_| Some(wagon(8040, 7977)));
    assert_eq!(crate::world::vector_dist(7608 - 8040, 10680 - 7977), 2737);
    assert!(!at(0));
    assert!(at(1));
}

#[test]
fn the_radius_is_in_tiles_and_the_query_is_in_position_units() {
    let mut list = SupplyList::new();
    list.register(0);
    let at_tiles = |n: i32| Pos::new(n * UNITS_PER_TILE, 0);
    let found = |n: i32| list.find(at_tiles(n), 14, |_| Some(wagon(0, 0)));
    // Fourteen tiles out is inside; the comparison is `<=`.
    assert_eq!(found(14), Some(0));
    assert_eq!(found(15), None);
    // And a fraction of a tile past it is outside, which is the whole
    // reason the conversion has to be exact.
    assert_eq!(
        list.find(Pos::new(14 * UNITS_PER_TILE + 1, 0), 14, |_| Some(wagon(
            0, 0
        ))),
        None
    );
}

#[test]
fn the_supply_radius_is_octagonal_like_a_border() {
    let mut list = SupplyList::new();
    list.register(0);
    let diag = |n: i32| Pos::new(n * UNITS_PER_TILE, n * UNITS_PER_TILE);
    // The metric charges 1.5x the leg on the diagonal against a true
    // 1.414x, so the reachable set is an octagon pulled in at the corners.
    // 9,9 is 12.73 tiles away and is charged 13.5, which still fits inside
    // 14. 10,10 is 14.14 away — barely outside a true circle — and is
    // charged 15, so it misses by a whole tile rather than by a tenth.
    assert_eq!(list.find(diag(9), 14, |_| Some(wagon(0, 0))), Some(0));
    assert_eq!(list.find(diag(10), 14, |_| Some(wagon(0, 0))), None);
}

#[test]
fn a_wagon_off_the_map_supplies_nothing() {
    let mut list = SupplyList::new();
    list.register(7);
    let at = Pos::new(0, 0);
    assert_eq!(list.find(at, 14, |_| Some(wagon(0, 0))), Some(0));
    assert_eq!(
        list.find(at, 14, |_| Some(Wagon {
            on_map: false,
            ..wagon(0, 0)
        })),
        None
    );
    assert_eq!(
        list.find(at, 14, |_| Some(Wagon {
            active: false,
            ..wagon(0, 0)
        })),
        None
    );
}

#[test]
fn slots_are_reused_from_the_front_and_trimmed_from_the_back() {
    let mut list = SupplyList::new();
    assert_eq!(list.register(10), 0);
    assert_eq!(list.register(11), 1);
    assert_eq!(list.register(12), 2);
    // A hole in the middle is refilled rather than appended to.
    list.close(1);
    assert_eq!(list.len(), 3);
    assert_eq!(list.register(13), 1);
    // Holes at the end are trimmed, and the trim walks down past every
    // free slot it meets rather than stopping at the first.
    list.close(2);
    list.close(1);
    assert_eq!(list.len(), 1);
    assert_eq!(list.unit(0), Some(10));
}

#[test]
fn a_general_reaches_nine_tiles_and_kutosov_twenty_seven() {
    let plain = General::default();
    assert_eq!(general_radius(&T, &plain), 9);
    assert_eq!(
        general_radius(
            &T,
            &General {
                kutosov: true,
                ..plain
            }
        ),
        27
    );
    // Parmenio's 3/2 is stored as 8.8 fixed point: 9 * 384 / 256 = 13.
    assert_eq!(
        general_radius(
            &T,
            &General {
                parmenio: true,
                ..plain
            }
        ),
        13
    );
    assert_eq!(
        general_radius(
            &T,
            &General {
                wellington: true,
                ..plain
            }
        ),
        18
    );
    // The upgrade chain: 6 * (n + 3) / 2.
    assert_eq!(
        general_radius(
            &T,
            &General {
                upgrades: 3,
                ..plain
            }
        ),
        18
    );
    assert_eq!(
        general_radius(
            &T,
            &General {
                patriot: Some(Patriot::Military),
                ..plain
            }
        ),
        12
    );
    assert_eq!(
        general_radius(
            &T,
            &General {
                patriot: Some(Patriot::Economic),
                ..plain
            }
        ),
        10
    );
}

#[test]
fn generals_are_tried_in_the_originals_order() {
    let aura = |kind, x| GeneralAura {
        kind,
        pos: Pos::new(x, 0),
        radius_tiles: 9,
        active: true,
        on_map: true,
    };
    let both = [
        aura(SupplyGeneral::Chandragupta, 0),
        aura(SupplyGeneral::Darius, 0),
    ];
    assert_eq!(
        find_general(&both, Pos::new(0, 0)),
        Some(SupplyGeneral::Darius)
    );
    // Out of range of everything.
    assert_eq!(find_general(&both, Pos::new(20 * UNITS_PER_TILE, 0)), None);
}

#[test]
fn supply_heals_nobody_in_the_shipped_game() {
    assert_eq!(heal_period(&T, false, false), 0);
    assert_eq!(heal_period(&T, true, false), 20);
    assert_eq!(heal_period(&T, false, true), 20);
    // The fold, and the fastest supply healing there is.
    assert_eq!(heal_period(&T, true, true), 10);
}

#[test]
fn out_of_supply_costs_siege_a_half_and_artillery_a_whole_reload() {
    let f = |class, under_attack, supplied| reload_frames(&T, 60, class, under_attack, supplied);
    assert_eq!(f(SiegeClass::None, false, false), 60);
    assert_eq!(f(SiegeClass::Siege, false, true), 60);
    assert_eq!(f(SiegeClass::Siege, false, false), 90);
    assert_eq!(f(SiegeClass::Artillery, false, false), 120);
    // Under attack, a siege unit fires as if out of supply wherever it is.
    assert_eq!(f(SiegeClass::Siege, true, true), 90);
    assert_eq!(f(SiegeClass::None, true, true), 60);
    // The 3/2 truncates.
    assert_eq!(reload_frames(&T, 61, SiegeClass::Siege, false, false), 91);
}

#[test]
fn your_own_ground_supplies_you_and_your_allys_does_not() {
    let land = Domain::Land;
    assert!(in_supply(land, Owner::Player(2), 2, false));
    assert!(!in_supply(land, Owner::Player(3), 2, false));
    assert!(!in_supply(land, Owner::None, 2, false));
    assert!(in_supply(land, Owner::None, 2, true));
    // Ships and aircraft never ask: the domain test comes first.
    assert!(in_supply(Domain::Sea, Owner::Player(3), 2, false));
    assert!(in_supply(Domain::Air, Owner::Player(3), 2, false));
}
