use crate::build::{BuildType, Ident};

/// **The recruiter's wonder test is the range flag** (twenty-fourth
/// pass, group 5; A8 row 29): a Forbidden City site recruits like any
/// other wonder, though its `Ident` is `ForbiddenCity`.
#[test]
fn a_forbidden_city_site_is_a_wonder_to_the_recruiter() {
    let mut s = crate::Sim::new(crate::Tuning::RON, crate::world::World::new(8, 8), 2);
    let t = s.add_build_type(BuildType {
        ident: Ident::ForbiddenCity,
        wonder: true,
        ..BuildType::default()
    });
    let b = s.add_building(1, crate::Pos::new(0x600, 0x600), 1);
    s.buildings[b].ty = Some(t);
    assert!(s.is_wonder_site(b));
    s.build_types[t].wonder = false;
    assert!(!s.is_wonder_site(b));
}
