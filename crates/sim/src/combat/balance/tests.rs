use super::*;

const T: Tuning = Tuning::RON;

fn unit(masks: u32, age: i32) -> Kind {
    Kind {
        masks,
        age,
        unit: true,
        domain: Domain::Land,
        unit_index: Some(0),
        ..Kind::default()
    }
}

fn building(masks: u32) -> Kind {
    Kind {
        masks,
        build: true,
        domain: Domain::Land,
        ..Kind::default()
    }
}

#[test]
fn the_age_bonus_and_a_few_flag_pairs() {
    let hop = unit(mask::FOOT | mask::HEAVY_INF | mask::MELEE, 0);
    let arch = unit(mask::FOOT | mask::FOOT_ARCHER | mask::ARCHERY, 0);
    // Heavy infantry without guns against foot archers: ×86.
    assert_eq!(type_damage(&T, &hop, &arch), 86);
    // Foot archers against heavy infantry: ×253.
    assert_eq!(type_damage(&T, &arch, &hop), 253);
    // An age ahead: 115 first, then ×253 → 290.
    let arch2 = unit(arch.masks, 1);
    assert_eq!(type_damage(&T, &arch2, &hop), (115 * 253) / 100);
    // Five ages ahead caps at five.
    let arch7 = unit(arch.masks, 7);
    assert_eq!(type_damage(&T, &arch7, &hop), (170 * 253) / 100);
    // Behind: nothing.
    assert_eq!(type_damage(&T, &hop, &arch2), 86);
}

#[test]
fn siege_against_buildings_and_the_early_return() {
    let cat = unit(mask::SIEGE | mask::BOMBARD | mask::WAR_MACHINE, 0);
    let mut cat = cat;
    cat.siege = true;
    let b = building(0);
    // SIEGE ×430, BOMBARD ×250: 430 → 1075.
    assert_eq!(type_damage(&T, &cat, &b), 1075);
    // A mortar line adds ×110 after the early-return gate only because
    // it has SIEGE.
    let mut mortar = cat;
    mortar.lines = line::LINE_MORTAR;
    assert_eq!(type_damage(&T, &mortar, &b), (1075 * 110) / 100);
    let mut not_siege = mortar;
    not_siege.masks &= !mask::SIEGE;
    assert_eq!(type_damage(&T, &not_siege, &b), 250);
}

#[test]
fn naval_rules() {
    let ship = unit(mask::NAVAL | mask::LARGE, 0);
    let mut ship = ship;
    ship.domain = Domain::Sea;
    let land = unit(mask::FOOT, 0);
    // A ship against a land unit: ×33.
    assert_eq!(type_damage(&T, &ship, &land), 33);
    // A land unit against a ship: ×33; a siege land unit: ×350.
    assert_eq!(type_damage(&T, &land, &ship), 33);
    let mut siege = unit(mask::SIEGE, 0);
    siege.siege = true;
    assert_eq!(type_damage(&T, &siege, &ship), 350);
}

#[test]
fn categories_follow_return_pack() {
    let mut k = unit(mask::FOOT | mask::HEAVY_INF, 2);
    k.unit_index = Some(40);
    assert_eq!(
        categories(&k),
        vec![40, 0x166, 0x167 + 2, 0x16f + 5, 0x16f + 7]
    );
    let mut s = k;
    s.siege = true;
    assert_eq!(categories(&s)[1], 0x160);
    let mut fort = building(mask::ARCHERY);
    fort.lines = line::LINE_FORT;
    fort.age = 1;
    assert_eq!(categories(&fort), vec![0x161, 0x165, 0x168, 0x16f + 17]);
    let mut tower = building(0);
    tower.lines = line::LINE_TOWER | line::LINE_CITY;
    // CITIES outranks TOWERS in the ladder.
    assert_eq!(categories(&tower)[0], 0x163);
}

#[test]
fn the_xml_product_truncates_step_by_step_and_gaia_is_flat() {
    let xml = |r: usize, c: usize| {
        if r == 1 && c == 2 {
            66
        } else if r == 3 && c == 2 {
            150
        } else {
            100
        }
    };
    // 100 → ×66 → 66 → ×150 → 99.
    assert_eq!(xml_product(&[1, 3], &[2], xml), 99);
    // Order matters only through truncation: 100 → 150 → ×66 = 99 too
    // here, but 33 then 150: 33 → 49 vs 150 → 49.
    assert_eq!(xml_product(&[3, 1], &[2], xml), 99);
    let a = unit(mask::FOOT, 0);
    let mut g = a;
    g.gaia = true;
    assert_eq!(compute_modifier(&T, &a, &g, |_, _| 50), 100);
    assert_eq!(compute_modifier(&T, &a, &a, |_, _| 100), 100);
    // Every pair of categories contributes: a unit with one flag has
    // four categories (its row, UNITS, its age, the flag), so 4 × 4 = 16
    // multiplications of 90 %.
    let p = xml_product(&categories(&a), &categories(&a), |_, _| 90);
    let mut q = 100;
    for _ in 0..16 {
        q = (90 * q) / 100;
    }
    assert_eq!(p, q);
}
