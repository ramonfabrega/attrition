//! **`BuildData::attack@0062e610`** (item 3387): a building's attack, as a pure
//! walk over a [`BuildAttackView`].
//!
//! `ObjectData::attack@006469f0` is the base (the crate's [`crate::Sim::attack_of`]
//! and `profile().attack`); this override adds, in this order, three
//! garrison terms of ten times a constant each:
//!
//! 1. **the general's** (`general_building_attack`), when the building is a
//!    *garrison building* — slot `0xb8` (`ObjectData::is`'s own, then the type's
//!    slot `0x60`, else the override called with `(0x1b7, 0)`) or the type's
//!    slot `0xfc` answers — or it is `flags & 0x20` with a city (`>= 0`); and
//!    the building holds one of the leader's general type (`0x36` when the
//!    tribe can type the type at `types +0xd8`, else the type's slot `0x68` of
//!    the leader's `who`) counted by `count_inside(COUNT_TYPE = 0x11)`;
//! 2. **the obsidian archers'** (`obsidian_archers_attack`), when either of the
//!    leader's bytes `+0x6da7`, `+0x6dcf` has bit 4;
//! 3. **Antipater's** (`antipater_garrison_attack_bonus`), when the leader's
//!    short `+0x59d0` is set and `count_inside(COUNT_STRICT_TYPE = 0x10, 0x169)`
//!    is non-zero.
//!
//! A building not in the garrison class returns the base alone. Terms 2 and 3
//! are read only past that gate, and term 3 follows term 2 whatever the latter
//! did.
//!
//! The crate carries none of this: `Sim::attack_of` has no building override,
//! no `general_building_attack` constant and no leader flag words, so the view
//! is **standalone** (nothing in `Sim` calls it), and the sweep is its only
//! caller. Whether wiring it moves a diff pin is not established.

/// `CountIndex` `COUNT_TYPE`.
pub const COUNT_TYPE: i32 = 0x11;
/// `CountIndex` `COUNT_STRICT_TYPE`.
pub const COUNT_STRICT_TYPE: i32 = 0x10;
/// The type word counted for Antipater's bonus.
pub const TYPE_ANTIPATER: i32 = 0x169;
/// The type the leader's tribe-can-type arm names.
pub const TYPE_TRIBE: i32 = 0x36;

/// The state `BuildData::attack` reads and the calls it makes.
pub trait BuildAttackView {
    /// `ObjectData::attack@006469f0`.
    fn base_attack(&mut self) -> i32;
    /// Slot `0xb8` of `this` is `ObjectData::is`.
    fn is_base_is(&mut self) -> bool;
    /// The type's slot `0x60`, when slot `0xb8` is `ObjectData::is`.
    fn type_slot_60(&mut self) -> i32;
    /// Slot `0xb8` called with `(0x1b7, 0)`, when it is overridden.
    fn is_override(&mut self) -> i32;
    /// The type's slot `0xfc`.
    fn type_slot_fc(&mut self) -> i32;
    /// `this +0x8 & 0x20`.
    fn flag_20(&mut self) -> bool;
    /// `this +0x72`, the city (a short).
    fn city(&mut self) -> i16;
    /// `LeaderData::tribe_can_type(leader, types +0xd8)`.
    fn tribe_can_type(&mut self) -> bool;
    /// The type record's slot `0x68`, handed the leader's `who`.
    fn type_slot_68_who(&mut self) -> i32;
    /// `ObjectData::count_inside(this, kind, ty)`.
    fn count_inside(&mut self, kind: i32, ty: i32) -> i32;
    /// The leader's bytes `+0x6da7` and `+0x6dcf` have bit 4.
    fn obsidian(&mut self) -> bool;
    /// The leader's short `+0x59d0` is non-zero.
    fn antipater(&mut self) -> bool;
    /// `constants +0xc60`, `general_building_attack`.
    fn general_building_attack(&mut self) -> i32;
    /// `constants +0x944`, `obsidian_archers_attack`.
    fn obsidian_archers_attack(&mut self) -> i32;
    /// `constants +0xb94`, `antipater_garrison_attack_bonus`.
    fn antipater_garrison_attack_bonus(&mut self) -> i32;
}

/// `BuildData::attack@0062e610`.
pub fn build_attack_steps<V: BuildAttackView>(v: &mut V) -> i32 {
    let mut attack = v.base_attack();
    let mut garrison = if v.is_base_is() {
        v.type_slot_60()
    } else {
        v.is_override()
    };
    if garrison == 0 {
        garrison = v.type_slot_fc();
        if garrison == 0 && !(v.flag_20() && v.city() >= 0) {
            return attack;
        }
    }
    let ty = if v.tribe_can_type() {
        TYPE_TRIBE
    } else {
        v.type_slot_68_who()
    };
    if v.count_inside(COUNT_TYPE, ty) != 0 {
        attack = attack.wrapping_add(v.general_building_attack().wrapping_mul(10));
    }
    if v.obsidian() {
        attack = attack.wrapping_add(v.obsidian_archers_attack().wrapping_mul(10));
    }
    if v.antipater() && v.count_inside(COUNT_STRICT_TYPE, TYPE_ANTIPATER) != 0 {
        attack = attack.wrapping_add(v.antipater_garrison_attack_bonus().wrapping_mul(10));
    }
    attack
}
