//! **`TypeData`'s index-range predicates** (item 1704). The original asks
//! "what class of thing is this type?" by comparing `TypeData::type` against
//! the bounds of `TypeIndex`'s blocks; the port keeps the answer as a table's
//! column ([`crate::tech::Kind`], the spell rows of [`crate::orders::spell`])
//! and never needed the bounds as a function. These are the bounds, each one
//! backed by `crates/sim/src/sweep/type_*.rs` against the executable, and each
//! names the reader it drives:
//!
//! - [`is_unit_type`], [`is_build_type`], [`is_tech_type`], [`is_final_type`],
//!   [`is_gov_type`]: [`crate::tech::Kind`]'s ranges, the tree loader's classes
//!   (`Kind::is_unit`, `is_building`, `is_tech`, `Final`, `Gov`);
//! - [`is_spell_type`]: `Unit::do_cast`'s head, [`crate::orders::spell`]'s
//!   `FIRST..=LAST`;
//! - [`is_bonus_type`]: `BASE_BONUSTYPES` (`0x2ac`), the bonus rows the
//!   `Roles` bonus tables index off;
//! - [`is_gov_hero_type`]: the six government patriots, `UnitTraits::patriot`;
//! - [`is_specialty_type`]: the two units whose `type` the flamethrower and
//!   mech-infantry rules key on (`role::FLAMETHROWER`);
//! - [`is_item_type`], [`is_wonder_type`], [`is_gaia_type`], [`is_good_type`]
//!   (item 1705): the blocks `TypeIndex` names around the units and
//!   buildings — goods below `0x32`, gaia's `0x192..=0x19d` tail of the unit
//!   block, the wonders `0x20e..=0x21e` and the one item type `0x21f`;
//! - [`is_entrenched`]: `UnitData::is_entrenched`, the reader the combat
//!   step's entrenchment gate and `find_nearby_target`'s `local_24` stand on
//!   (`combat.entrenched`).

/// `TypeIndex::MECHINFANTRY`.
pub const MECHINFANTRY: i32 = 0x68;
/// `TypeIndex::FLAMETHROWER`.
pub const FLAMETHROWER: i32 = 0x83;

/// `TypeData::is_specialty_type@00470570`.
pub const fn is_specialty_type(t: i32) -> bool {
    t == MECHINFANTRY || t == FLAMETHROWER
}
/// `TypeData::is_spell_type@00470590`: `0x275..=0x2ab`.
pub const fn is_spell_type(t: i32) -> bool {
    0x274 < t && t < 0x2ac
}
/// `TypeData::is_bonus_type@004705b0`: `0x2ac..=0x325`.
pub const fn is_bonus_type(t: i32) -> bool {
    0x2ab < t && t < 0x326
}
/// `TypeData::is_final_type@00470660`: `0x243..=0x246`.
pub const fn is_final_type(t: i32) -> bool {
    0x242 < t && t < 0x247
}
/// `TypeData::is_gov_hero_type@00470680`: `0x160..=0x165`.
pub const fn is_gov_hero_type(t: i32) -> bool {
    0x15f < t && t < 0x166
}
/// `TypeData::is_gov_type@004706a0`: `0x26f..=0x274`.
pub const fn is_gov_type(t: i32) -> bool {
    0x26e < t && t < 0x275
}
/// `TypeData::is_tech_type@004706c0`: `0x220..=0x274`.
pub const fn is_tech_type(t: i32) -> bool {
    0x21f < t && t < 0x275
}
/// `TypeData::is_build_type@004707c0`: `0x19e..=0x21e`.
pub const fn is_build_type(t: i32) -> bool {
    0x19d < t && t < 0x21f
}
/// `TypeData::is_unit_type@004707e0`: `0x32..=0x19d`.
pub const fn is_unit_type(t: i32) -> bool {
    0x31 < t && t < 0x19e
}

/// `TypeData::is_item_type@00470760`: only `0x21f`.
pub const fn is_item_type(t: i32) -> bool {
    0x21e < t && t < 0x220
}
/// `TypeData::is_wonder_type@00470780`: `0x20e..=0x21e`.
pub const fn is_wonder_type(t: i32) -> bool {
    0x20d < t && t < 0x21f
}
/// `TypeData::is_gaia_type@004707a0`: `0x192..=0x19d`.
pub const fn is_gaia_type(t: i32) -> bool {
    0x191 < t && t < 0x19e
}
/// `TypeData::is_good_type@00470800`: signed `t < 0x32`, negatives included.
pub const fn is_good_type(t: i32) -> bool {
    t < 0x32
}

/// `UnitData::is_entrenched@0060a810`: with `param` 0 the raw `unit_masks &
/// 0x2000000`; otherwise entrenched only while `unit_masks2 & 0x20000` is clear.
pub const fn is_entrenched(unit_masks: u32, unit_masks2: u32, param: i32) -> bool {
    if param == 0 {
        return unit_masks & 0x200_0000 != 0;
    }
    unit_masks & 0x200_0000 != 0 && unit_masks2 & 0x2_0000 == 0
}

#[cfg(test)]
mod tests {
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
}
