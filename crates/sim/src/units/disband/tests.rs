use crate::world::{Cell, Terrain, World};
use crate::{Pos, Sim, Tuning, Unit, UnitType};

/// Three of who=1's units, each of its own type, on an 8×8 land map.
fn three() -> (Sim, [usize; 3]) {
    let mut world = World::new(8, 8);
    world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(7, 7));
    let mut s = Sim::new(Tuning::RON, world, 2);
    let units = [0, 1, 2].map(|k| {
        let ty = s.add_unit_type(UnitType::default());
        let mut u = Unit::new(1, k, Pos::new((2 + i32::from(k)) * 768, 3 * 768), 20);
        u.ty = Some(ty);
        s.add_unit(u)
    });
    (s, units)
}

/// **`Group::action_disband`'s walk** (item 3914, `docs/GOLDEN.md`
/// §161): from the last member, and with `all` 0 only the first it
/// reaches; each disband spends `Object::disband`'s draw before the
/// death's own. Made to fail with the `all` break dropped and with
/// the walk run forwards.
#[test]
fn a_disband_takes_the_last_member_and_all_takes_every_one() {
    let (mut s, [a, b, c]) = three();
    let mut g = crate::groups::group::Group::stack(1);
    for u in [a, b, c] {
        s.group_add(&mut g, u);
    }
    let last = *g.list.last().unwrap();
    assert_eq!(s.group_action_disband(&g, false), 1);
    assert!(!s.units[last].alive(), "the last member is disbanded");
    let left: Vec<usize> = g
        .list
        .iter()
        .copied()
        .filter(|&u| s.units[u].alive())
        .collect();
    assert_eq!(left.len(), 2, "with all 0, only one goes");
    assert_eq!(s.group_action_disband(&g, true), 2);
    assert!([a, b, c].iter().all(|&u| !s.units[u].alive()));
}

/// **A disbanded unit disbands what it holds first** (`Object::disband`'s
/// `garrison_count` loop): the rider dies with it, and spends its own
/// draw ahead of the holder's. Made to fail with the loop dropped.
#[test]
fn a_disbanded_holder_disbands_its_rider_first() {
    let (mut s, [holder, rider, other]) = three();
    s.units[rider].inside_unit = Some(holder);
    s.disband_unit(holder);
    assert!(!s.units[holder].alive());
    assert!(!s.units[rider].alive(), "the rider is disbanded with it");
    assert!(s.units[other].alive());
}
