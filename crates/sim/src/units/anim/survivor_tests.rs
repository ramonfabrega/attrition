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

/// **A plane born on a reused number takes its dead occupant's stored
/// `z`, held height included** (item 4730, `docs/RECGAME.md` §13.3):
/// row 93's Citizen `2/112` died with its figure held at 528 over ground
/// that reads 539, and the Fighter born on its number reads 528.
#[test]
fn a_plane_on_a_reused_number_inherits_its_occupant_s_held_z() {
    use crate::territory::attrition::Domain;
    let mut s = crate::Sim::new(crate::Tuning::RON, crate::world::World::new(8, 8), 2);
    let at = crate::Pos::new(500, 500);
    let a = s.add_unit(crate::Unit::new(2, 112, at, 10));
    s.init_guys(a, None);
    let ground = s.units[a].guys.first().map(|_| s.guy_stored_z(a, 0).0);
    assert_eq!(ground, Some(0), "the flat ground reads 0");
    s.units[a].guys[0].z_held = Some((s.units[a].movement.body.pos, 528));
    s.units[a].health = 0;
    let t = s.add_unit_type(crate::UnitType {
        kind: crate::territory::attrition::UnitKind {
            domain: Domain::Air,
            ..crate::territory::attrition::UnitKind::default()
        },
        combat: crate::combat::Profile {
            domain: Domain::Air,
            ..crate::combat::Profile::default()
        },
        ..crate::UnitType::default()
    });
    let mut f = crate::Unit::new(2, 112, at, 10);
    f.ty = Some(t);
    let b = s.add_unit(f);
    s.init_guys(b, Some(t));
    let af = s.units[b].airframe;
    assert_eq!((af.z, af.last_z), (528, 528), "the occupant's held z, not the ground");
}
