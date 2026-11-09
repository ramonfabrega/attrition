use super::*;
use crate::orders::spell;

#[test]
fn the_bounds_meet_the_ranges_the_port_already_names() {
    assert!(is_spell_type(spell::FIRST) && is_spell_type(spell::LAST));
    assert!(!is_spell_type(spell::FIRST - 1) && !is_spell_type(spell::LAST + 1));
    // The classes tile the index space between goods and bonuses.
    assert!(is_unit_type(0x32) && is_build_type(0x19e) && is_tech_type(0x220));
    assert!(is_final_type(0x243) && is_tech_type(0x243) && is_gov_type(0x26f));
    assert!(is_tech_type(0x274) && is_spell_type(0x275) && is_bonus_type(0x2ac));
    assert!(is_specialty_type(0x68) && !is_specialty_type(0x69));
    assert!(is_good_type(0x31) && !is_good_type(0x32) && is_gaia_type(0x19d));
    assert!(is_wonder_type(0x21e) && is_item_type(0x21f) && !is_item_type(0x220));
    assert!(is_entrenched(0x200_0000, 0x2_0000, 0) && !is_entrenched(0x200_0000, 0x2_0000, 1));
}
