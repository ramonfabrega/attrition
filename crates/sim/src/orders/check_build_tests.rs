use crate::orders::{Body, MoveKind, QueuePos, flag};
use crate::world::Pos;
use crate::{Sim, Unit, UnitType};

/// who=0's finished site, its unfinished one beside it, and a builder
/// standing between them.
fn two_sites_and_a_builder() -> (Sim, usize, usize, usize) {
    let mut s = Sim::new(
        crate::core::tuning::Tuning::RON,
        crate::world::World::new(128, 128),
        2,
    );
    s.cities.nation[0].human = true;
    let ty = s.add_build_type(crate::cities::build::BuildType::default());
    let done = s.add_building(0, Pos::new(7296, 12480), 0);
    s.buildings[done].ty = Some(ty);
    s.buildings[done].started = true;
    s.buildings[done].active = true;
    let site = s.add_building(0, Pos::new(8256, 12480), 0);
    s.buildings[site].ty = Some(ty);
    s.buildings[site].started = true;
    s.buildings[site].active = false;
    let t = s.add_unit_type(UnitType {
        hits: 40,
        moves: 100,
        ..UnitType::default()
    });
    let index = i16::try_from(s.units.len()).unwrap();
    let mut u = Unit::new(0, index, Pos::new(7752, 12696), 40);
    u.ty = Some(t);
    u.on_map = true;
    let u = s.add_unit(u);
    s.units[u].kind = s.tables.unit_types[t].kind;
    s.init_guys(u, Some(t));
    (s, done, site, u)
}

/// **The scan's kill is the front's** (item 4555, the 2017 sample's
/// `3/55` on 6669; `Unit::check_build_order@00603470`,
/// `Unit::kill_current_order@005e2cb0`). A builder holds `[BUILD_AT done,
/// MOVE_TO, BUILD_AT site]`; `do_build` finds `done` finished, kills it
/// and runs the scan. The scan steps over the move and collects `site`;
/// `repath` strips the move off the front; `kill_current_order`
/// re-points the cursor at the front and kills `site`'s order; the list
/// is empty and `site` is re-added once. The crate killed at the scan's
/// stale index, which the move's removal had left past the end, so the
/// scan collected `site` a second time and the builder ended with two
/// orders on it (run1786, units 6669: `orders.count` ours 2, theirs 1).
///
/// Made to fail with the kill at the scan's cursor index.
#[test]
fn a_build_queued_behind_a_move_is_collected_once() {
    let (mut s, done, site, u) = two_sites_and_a_builder();
    s.add_build_order(u, done, QueuePos::New, true);
    s.add_move_order(
        u,
        Pos::new(7800, 12696),
        MoveKind::MoveTo,
        QueuePos::Last,
        false,
    );
    s.add_build_order(u, site, QueuePos::Last, true);
    let kinds: Vec<Body> = s.units[u].orders.iter().map(|o| o.body).collect();
    assert!(
        matches!(
            kinds.as_slice(),
            [Body::Build(a), Body::Move(_), Body::Build(b)] if *a == done && *b == site
        ),
        "the list as the sample's: {kinds:?}"
    );
    s.do_build(u, 0);
    let got: Vec<(Body, bool)> = s.units[u]
        .orders
        .iter()
        .map(|o| (o.body, o.has(flag::ACTION)))
        .collect();
    assert_eq!(
        got,
        vec![(Body::Build(site), true)],
        "one order on the site, the move gone"
    );
}

/// The same list without the move: the scan's cursor never leaves the
/// front, and the site is re-added once on either reading.
#[test]
fn a_build_queued_behind_a_finished_one_is_collected_once() {
    let (mut s, done, site, u) = two_sites_and_a_builder();
    s.add_build_order(u, done, QueuePos::New, true);
    s.add_build_order(u, site, QueuePos::Last, true);
    s.do_build(u, 0);
    let got: Vec<Body> = s.units[u].orders.iter().map(|o| o.body).collect();
    assert_eq!(got, vec![Body::Build(site)]);
}
