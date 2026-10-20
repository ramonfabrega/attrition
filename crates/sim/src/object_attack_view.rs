//! `ObjectData::attack@006469f0`: the base attack of an object (`docs/COMBAT.md` §4.1), swept against the
//! original under `tools/emu` (item 3387, `sweep::object_data_attack`).
//!
//! Start from the type record's attack (`+0x1e8`). An object whose slot `+0xd0` (`is_caravan`) answers
//! non-zero and whose owner `has_preq(CARAVAN_ATTACK_BONUS)` has it **replaced** by
//! `constants.caravan_attack_bonus` (`+0xcd4`, loaded x10). Then the Dutch arm: the owner has the tribe bonus
//! `0x16`, `constants.dutch_attack_bonus` (`+0x8b8`) is non-zero, the object is a unit (slot `+0x18`), it is a
//! merchant kind (type `0x3d`, `0x3e` or `400`, or slot `+0xd0`, or slot `+0xcc` `is_supply`) and it is not a
//! government hero (slot `+0xc8`): `+ ages * dutch_attack_bonus * 10`, `ages` the owner's age count (the
//! original keeps it xor `0x62766`, a storage detail the view hides). Unlike `ObjectData::armor@00647db0`, no
//! patch-version gate stands on the hero test.
//!
//! `Sim::attack_of` (`fight.rs`) runs these steps through `SimObjectAttack`; the caravan arm was missing before
//! item 3387.

/// What `ObjectData::attack` reads, in the order it reads it.
pub trait ObjectAttackView {
    /// The type record's attack, `+0x1e8`.
    fn type_attack(&mut self) -> i32;
    /// Slot `+0xd0`: `UnitData::is_caravan`.
    fn is_caravan(&mut self) -> bool;
    /// `LeaderData::has_preq(owner, CARAVAN_ATTACK_BONUS)`.
    fn has_caravan_preq(&mut self) -> bool;
    /// `constants.caravan_attack_bonus`.
    fn caravan_attack_bonus(&mut self) -> i32;
    /// `LeaderData::has_tribe_bonus(owner, 0x16)`.
    fn has_dutch_bonus(&mut self) -> bool;
    /// `constants.dutch_attack_bonus`.
    fn dutch_attack_bonus(&mut self) -> i32;
    /// Slot `+0x18`: true on a unit, false on a building.
    fn is_unit(&mut self) -> bool;
    /// The type record's `TypeIndex` (`+4`).
    fn type_index(&mut self) -> i32;
    /// Slot `+0xcc`: `UnitData::is_supply`.
    fn is_supply(&mut self) -> bool;
    /// Slot `+0xc8`: `UnitData::is_gov_hero`.
    fn is_gov_hero(&mut self) -> bool;
    /// The owner's age count.
    fn ages(&mut self) -> i32;
}

/// `ObjectData::attack`, call for call.
pub fn object_attack_steps<V: ObjectAttackView>(v: &mut V) -> i32 {
    let mut attack = v.type_attack();
    if v.is_caravan() && v.has_caravan_preq() {
        attack = v.caravan_attack_bonus();
    }
    if !v.has_dutch_bonus() || v.dutch_attack_bonus() == 0 || !v.is_unit() {
        return attack;
    }
    let id = v.type_index();
    let kind = id == 0x3d || id == 0x3e || id == 400 || v.is_caravan() || v.is_supply();
    if !kind || v.is_gov_hero() {
        return attack;
    }
    attack.wrapping_add(
        v.ages()
            .wrapping_mul(v.dutch_attack_bonus())
            .wrapping_mul(10),
    )
}
