use crate::attrition::Domain;
use crate::group::Group;
use crate::orders::{Body, QueuePos, flag};
use crate::world::Pos;
use crate::{Sim, Unit, UnitType};

/// who=0's Airbase (garrison column 0) and a Helicopter beside it.
fn airbase_and_helicopter() -> (Sim, usize, usize) {
    let mut s = Sim::new(
        crate::tuning::Tuning::RON,
        crate::world::World::new(128, 128),
        2,
    );
    s.nation[0].human = true;
    s.frame = 1260;
    let airbase = s.add_build_type(crate::build::BuildType {
        ident: crate::build::Ident::Airbase,
        ..crate::build::BuildType::default()
    });
    let base = s.add_building(0, Pos::new(7680, 19200), 0);
    s.buildings[base].ty = Some(airbase);
    s.buildings[base].started = true;
    s.buildings[base].active = true;
    let mut t = UnitType {
        hits: 100,
        moves: 100,
        mana: 400,
        kind: crate::attrition::UnitKind {
            domain: Domain::Air,
            ..crate::attrition::UnitKind::default()
        },
        ..UnitType::default()
    };
    t.cols.unit_flags |= crate::ai_load::uflags::FLAG_HELICOPTER;
    let ty = s.add_unit_type(t);
    s.tables.unit_types[ty].tree = Some(crate::airbase::HELICOPTER);
    s.tables.unit_types[ty].garrison.trained_at = Some(airbase);
    let index = i16::try_from(s.units.len()).unwrap();
    let mut u = Unit::new(0, index, Pos::new(8568, 18168), 100);
    u.ty = Some(ty);
    u.on_map = true;
    let u = s.add_unit(u);
    s.units[u].kind = s.tables.unit_types[ty].kind;
    s.init_guys(u, Some(ty));
    (s, base, u)
}

/// **A Helicopter ordered into its own Airbase flies in** (item 3990,
/// `Unit::do_garrison@005e6b80:63`–`:83`; run1548's 1262: `0/9` holds
/// a `STRAFEORDER`, `xx/yy −1`, `mandatory 1`, `returning 1`). The
/// garrison order is killed and a strafe with no target, home the
/// base, is put first with the action bit. Made to fail with the arm
/// dropped (the garrison order stays: not adjacent).
#[test]
fn a_helicopter_ordered_into_an_airbase_takes_a_strafe_home_to_it() {
    let (mut s, base, u) = airbase_and_helicopter();
    s.add_garrison_order(u, base, false, QueuePos::Last, false);
    s.do_garrison_order(u);
    assert_eq!(s.units[u].orders.len(), 1, "the garrison order killed");
    let o = s.units[u].orders.front().copied().unwrap();
    assert_eq!(o.flags & flag::ACTION, flag::ACTION, "the action bit");
    let Body::Strafe(sf) = o.body else {
        panic!("a strafe, not {:?}", o.body)
    };
    assert_eq!((sf.target, sf.home), (None, Some(base)));
    assert!(sf.mandatory && sf.returning, "mandatory and returning");
}

/// **An Airbase takes a helicopter's garrison although its column is
/// 0** (item 3990, `Group::action_garrison@007005e0`'s zero-limit arm,
/// `is(0x1bf)`; run1548's `@garrison` on 1260). Made to fail with
/// `data_is_1bf` answering false (no order).
#[test]
fn an_airbase_s_zero_limit_admits_a_helicopter_s_garrison() {
    let (mut s, base, u) = airbase_and_helicopter();
    let g = Group {
        who: 0,
        army: None,
        pushed: None,
        list: vec![u],
    };
    s.group_action_garrison(&g, base, QueuePos::Last);
    assert!(
        matches!(
            s.units[u].orders.front().map(|o| o.body),
            Some(Body::Garrison { building, .. }) if building == base
        ),
        "a garrison order on the Airbase, not {:?}",
        s.units[u].orders.front()
    );
}
