/// **The spot is searched with the caster's type** (group 11; A6 row
/// 45): a Cannon's block radius 2 beside a General's 1 is not the
/// General's search.
#[test]
fn the_decoy_search_is_asked_with_the_casters_type() {
    let mut s = crate::Sim::new(crate::Tuning::RON, crate::world::World::new(8, 8), 2);
    let general = s.add_unit_type(crate::UnitType::default());
    let cannon = s.add_unit_type(crate::UnitType::default());
    let mut g = crate::Unit::new(0, 0, crate::Pos::new(500, 500), 10);
    g.ty = Some(general);
    let g = s.add_unit(g);
    assert_eq!(s.decoy_search_type(g, cannon), general);
    s.units[g].ty = None;
    assert_eq!(
        s.decoy_search_type(g, cannon),
        cannon,
        "untyped: the source"
    );
}
