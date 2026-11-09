use crate::combat::Obj;

/// **A newborn's first figure keeps the dead occupant's aim and turret
/// flags** (twenty-fourth pass, group 16; A3 rows 16 and 17), and a
/// slot nobody died in starts clear.
#[test]
fn a_newborn_s_first_figure_keeps_its_predecessors_aim_and_flags() {
    let mut s = crate::Sim::new(crate::Tuning::RON, crate::world::World::new(8, 8), 2);
    let a = s.add_unit(crate::Unit::new(0, 7, crate::Pos::new(500, 500), 10));
    s.init_guys(a, None);
    s.units[a].guys[0].aim = Some(Obj::Unit(3));
    s.units[a].guys[0].turret.node_flags = 5;
    s.units[a].guys[0].turret.des_flags = 6;
    s.units[a].health = 0;
    let b = s.add_unit(crate::Unit::new(0, 7, crate::Pos::new(500, 500), 10));
    s.init_guys(b, None);
    let g = &s.units[b].guys[0];
    assert_eq!(g.aim, Some(Obj::Unit(3)));
    assert_eq!((g.turret.node_flags, g.turret.des_flags), (5, 6));
    let other = s.add_unit(crate::Unit::new(0, 8, crate::Pos::new(500, 500), 10));
    s.init_guys(other, None);
    assert_eq!(s.units[other].guys[0].aim, None, "a fresh slot is clear");
}
