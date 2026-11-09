use super::{Table, TypeRef};

#[test]
fn two_families_and_growth_keep_every_cell() {
    let t = Table::build(2, 2, |a, b| match (a, b) {
        (TypeRef::Unit(x), TypeRef::Unit(y)) => 10 + (x * 2 + y) as i32,
        (TypeRef::Unit(x), TypeRef::Build(y)) => 20 + (x * 2 + y) as i32,
        (TypeRef::Build(x), TypeRef::Unit(y)) => 30 + (x * 2 + y) as i32,
        (TypeRef::Build(x), TypeRef::Build(y)) => 40 + (x * 2 + y) as i32,
    });
    assert_eq!(t.pct(1, 0), 12);
    assert_eq!(t.pct_of(TypeRef::Unit(0), TypeRef::Build(1)), 21);
    assert_eq!(t.pct_of(TypeRef::Build(1), TypeRef::Unit(1)), 33);
    assert_eq!(t.pct_of(TypeRef::Build(1), TypeRef::Build(1)), 43);
    assert_eq!(t.pct_of(TypeRef::Build(2), TypeRef::Unit(0)), 100);
    assert_eq!(t.pct(5, 0), 100);
    let g = t.grown(3, 3);
    assert_eq!(g.width(), 3);
    assert_eq!(g.builds(), 3);
    assert_eq!(g.pct(1, 0), 12);
    assert_eq!(g.pct_of(TypeRef::Build(1), TypeRef::Build(1)), 43);
    assert_eq!(g.pct_of(TypeRef::Unit(2), TypeRef::Build(2)), 100);
    assert_eq!(g.pct_of(TypeRef::Build(2), TypeRef::Unit(0)), 100);
    // Growing never shrinks.
    assert_eq!(g.grown(1, 1), g);
}
