use super::*;

fn facts() -> RoleFacts {
    RoleFacts {
        domain: Domain::Land,
        ..RoleFacts::default()
    }
}

#[test]
fn the_citizen_s_word_is_the_original_s() {
    // Unit type 0x32, `role 262912` in the type dump: land, citizen,
    // civilian — an `ATTACK` of 40 that the `Civilian` category refuses.
    let r = determine_roles(&RoleFacts {
        citizen_id: true,
        cat: cat::CIVILIAN,
        attack: 40,
        ..facts()
    });
    assert_eq!(r, 0x40300);
    assert_eq!(r, role::LAND | role::CITIZEN | role::CIVILIAN);
}

#[test]
fn a_hoplite_takes_the_hoplite_bit_and_not_the_infantry_one() {
    let h = determine_roles(&RoleFacts {
        cat: cat::FOOT,
        attack: 60,
        hoplite: true,
        ..facts()
    });
    assert!(h & role::HOPLITE != 0 && h & role::INFANTRY == 0);
    let s = determine_roles(&RoleFacts {
        cat: cat::FOOT,
        attack: 60,
        ..facts()
    });
    assert!(s & role::INFANTRY != 0 && s & role::HOPLITE == 0);
    assert!(h & role::MELEE != 0 && s & role::MELEE != 0);
}

#[test]
fn command_and_civilian_categories_are_never_military() {
    for c in [cat::COMMAND, cat::CIVILIAN] {
        let r = determine_roles(&RoleFacts {
            cat: c,
            attack: 90,
            ..facts()
        });
        assert!(r & role::MILITARY == 0);
        assert!(r & role::CIVILIAN != 0);
    }
}

#[test]
fn the_sea_and_air_military_bits_come_with_the_domain() {
    let sea = determine_roles(&RoleFacts {
        domain: Domain::Sea,
        attack: 40,
        max_range: 4,
        ..facts()
    });
    assert_eq!(
        sea,
        role::SEA | role::MILITARY | role::SEA_MILITARY | role::RANGED
    );
    let air = determine_roles(&RoleFacts {
        domain: Domain::Air,
        attack: 40,
        ..facts()
    });
    assert_eq!(
        air,
        role::AIR | role::MILITARY | role::AIR_MILITARY | role::MELEE
    );
}

#[test]
fn a_carrier_takes_the_carry_bit_whatever_else_it_is() {
    let r = determine_roles(&RoleFacts {
        domain: Domain::Sea,
        carry: 8,
        ..facts()
    });
    assert!(r & role::CARRY != 0 && r & role::CIVILIAN != 0);
}

#[test]
fn the_caster_bit_walks_down_a_from_chain() {
    // Militia → Minuteman → Partisan: only the root is a craft's owner.
    let mut cols = vec![UnitCols::default(); 3];
    let from = [None, Some(0), Some(1)];
    let graft = [None, None, None];
    spread_casters(&mut cols, &[true, false, false], &graft, &from);
    assert!(cols.iter().all(|c| c.flag2(uflags2::CASTER)));
}

/// A tree with one age, one epoch and one plain tech, a unit that needs
/// the age and a gather building that needs the plain tech.
fn small_tree() -> (TechTree, Vec<UnitType>, Vec<BuildType>) {
    use crate::build::{BuildType, Ident, flags};
    use crate::tech::{Kind, Line, Preq, TypeDef};

    let mut tree = TechTree::new();
    let age = tree.add(TypeDef::new("Classical Age", Kind::Age(0)));
    let epoch = tree.add(TypeDef::new(
        "Science 1",
        Kind::Epoch {
            line: Line::Science,
            level: 0,
        },
    ));
    let plain = tree.add(TypeDef::new("Agriculture", Kind::Plain));
    let mut ud = TypeDef::unit("Hoplite", crate::tech::UnitTraits::default());
    ud.preq[0] = Preq::Of(age);
    let unit_id = tree.add(ud);
    let mut bd = TypeDef::new(
        "Farm",
        Kind::Building {
            auto: false,
            wonder: false,
        },
    );
    bd.preq[0] = Preq::Of(plain);
    let build_id = tree.add(bd);
    // The epoch names the age, so the age gains an epoch dependant.
    tree.types[epoch].preq[0] = Preq::Of(age);

    let unit = UnitType {
        tree: Some(unit_id),
        cols: UnitCols {
            role: role::LAND | role::MILITARY | role::FOOT | role::INFANTRY | role::MELEE,
            ..UnitCols::default()
        },
        ..UnitType::default()
    };
    let build = BuildType {
        ident: Ident::Farm,
        tree: Some(build_id),
        flags: flags::GATHER,
        ..BuildType::default()
    };
    (tree, vec![unit], vec![build])
}

#[test]
fn the_weights_count_what_a_tech_unlocks() {
    let (mut tree, units, builds) = small_tree();
    compute_ai_values(&mut tree, &Setup::STANDARD, &units, &builds, &[], &[]);
    let age = 0;
    let plain = 2;
    // The age is named by a military unit (`ai[0]`, `ai[8]`) and by an
    // epoch tech (`ai[4] + 2`, `ai[0] + 1`).
    assert_eq!(tree.types[age].ai[0], 2);
    assert_eq!(tree.types[age].ai[8], 1);
    assert_eq!(tree.types[age].ai[4], 2);
    // The plain tech is named by a rootless gather building: `2s` with
    // `s = 2`, then the fundamental arm's `s` again.
    assert_eq!(tree.types[plain].ai[1], 6);
    assert_eq!(tree.types[plain].ai[9], 6);
    // `ai[7]` is never written by anything.
    assert!(tree.types.iter().all(|d| d.ai[7] == 0));
}

#[test]
fn a_gaia_type_is_outside_the_unit_loop() {
    let (mut tree, mut units, builds) = small_tree();
    units[0].gaia = true;
    compute_ai_values(&mut tree, &Setup::STANDARD, &units, &builds, &[], &[]);
    // Only the epoch's contribution to the age is left.
    assert_eq!(tree.types[0].ai[0], 1);
    assert_eq!(tree.types[0].ai[8], 0);
}

#[test]
fn a_spell_and_the_first_bonus_score_where_the_original_puts_them() {
    use crate::tech::Preq;
    let (mut tree, units, builds) = small_tree();
    let plain = 2;
    let spell = [Preq::Of(plain), Preq::None, Preq::None];
    let bonus = [Preq::Of(plain), Preq::None, Preq::None];
    compute_ai_values(
        &mut tree,
        &Setup::STANDARD,
        &units,
        &builds,
        &[spell],
        &[bonus, bonus],
    );
    // The building's 6 plus the spell's 2, the first bonus's 2 and the
    // second's 1; `ai[0]` is the spell's 4 alone.
    assert_eq!(tree.types[plain].ai[1], 6 + 2 + 2 + 1);
    assert_eq!(tree.types[plain].ai[0], 4);
    assert_eq!(tree.types[plain].ai[4], 4);
}

#[test]
fn the_caster_bit_does_not_walk_up() {
    let mut cols = vec![UnitCols::default(); 3];
    let from = [None, Some(0), Some(1)];
    let graft = [None, None, None];
    spread_casters(&mut cols, &[false, false, true], &graft, &from);
    assert_eq!(cols.iter().filter(|c| c.flag2(uflags2::CASTER)).count(), 1);
}
