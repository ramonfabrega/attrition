//! **`UnitData::attack@006103c0`** (item 3399): a unit's attack, as a pure walk
//! over a [`UnitAttackView`].
//!
//! `ObjectData::attack@006469f0` is the base ([`crate::object_attack_view`];
//! `Sim::attack_of` in the crate); a base of zero returns zero. Past it the
//! override adds, in this order, ten times a constant for each term that holds:
//!
//! 1. `terra_cotta_attack`: the leader has the wonder `0x211`;
//! 2. `thecitizen_attack_bonus`: the unit has a general `0x164` (`>= 0`);
//! 3. `obsidian_archers_attack`, twice: once for each of the masks `0x4000`
//!    and `0x400` the unit answers (`has_objmask`; the type's slot `0xf0` when
//!    the unit's slot `0x148` is `ObjectData::has_objmask`) while either of the
//!    leader's bytes `+0x6da7`, `+0x6dcf` has bit 4;
//! 4. when the type's word `+0x218` is 2 and the leader has the wonder
//!    `0x21e`, the sum so far becomes `(space_air_attack + 100) * sum / 100`
//!    (not a bonus of ten times: a scale);
//! 5. the generals' terms, each gated by a short of the leader's record
//!    (`+0x59ca` Alexander, `+0x59cc` Napoleon, `+0x59ce` Parmenio, `+0x59d2`
//!    Ptolemy, `+0x59d8` Spitamenes, `+0x59e8` Blucher, `+0x59e2` Djezzar,
//!    `+0x59d6` Memnon, `+0x59c2` the Monarch) and a general of the unit's
//!    (`has_general(0, type)`, `>= 0`), most with a type predicate of the unit
//!    (`is(ty, 0)`: `ObjectData::is`'s own — the type's slot `0x60` — or the
//!    override), Napoleon's with the type's slot `0x10c`, Blucher's and the
//!    Monarch's with the type's word `+0x40 == 0x1ac`.
//!
//! The crate carries none of this: `Sim::attack_of` has no unit override and
//! none of the wonder, general or leader flag words, so the view is
//! **standalone** (nothing in `Sim` calls it) and the sweep is its only
//! caller. Whether wiring it moves a diff pin is not established.

/// Offsets into the constants record (`GameAccessConst::constantsc`).
pub mod constant {
    pub const TERRA_COTTA_ATTACK: u32 = 0x458;
    pub const SPACE_AIR_ATTACK: u32 = 0x560;
    pub const OBSIDIAN_ARCHERS_ATTACK: u32 = 0x944;
    pub const ALEXANDER_ATTACK: u32 = 0xb44;
    pub const ALEXANDER_HEAVY_INF_ATTACK: u32 = 0xb48;
    pub const PARMENIO_HEAVY_CAV_ATTACK: u32 = 0xb50;
    pub const PTOLEMY_SIEGE_ATTACK: u32 = 0xb68;
    pub const SPITAMENES_HORSE_ARCHER_ATTACK: u32 = 0xb74;
    pub const MEMNON_MERC_ATTACK: u32 = 0xb88;
    pub const NAPOLEON_SIEGE_ATTACK: u32 = 0xba4;
    pub const DJEZZAR_ATTACK: u32 = 0xbc8;
    pub const BLUCHER_CAV_ATTACK: u32 = 0xbcc;
    pub const THEMONARCH_CAV_ATTACK: u32 = 0xc80;
    pub const THECITIZEN_ATTACK_BONUS: u32 = 0xc94;
}

/// Offsets of the leader's record shorts that gate a general's term.
pub mod leader {
    pub const THEMONARCH: u32 = 0x59c2;
    pub const ALEXANDER: u32 = 0x59ca;
    pub const NAPOLEON: u32 = 0x59cc;
    pub const PARMENIO: u32 = 0x59ce;
    pub const PTOLEMY: u32 = 0x59d2;
    pub const MEMNON: u32 = 0x59d6;
    pub const SPITAMENES: u32 = 0x59d8;
    pub const DJEZZAR: u32 = 0x59e2;
    pub const BLUCHER: u32 = 0x59e8;
}

/// The type word `+0x40` the Blucher and Monarch terms compare.
pub const TYPE_CAVALRY: i32 = 0x1ac;

/// The state `UnitData::attack` reads and the calls it makes.
pub trait UnitAttackView {
    /// `ObjectData::attack@006469f0`.
    fn base_attack(&mut self) -> i32;
    /// `LeaderData::has_wonder(leader, ty)`, non-zero.
    fn has_wonder(&mut self, ty: i32) -> bool;
    /// `ObjectData::has_general(this, 0, ty)`, the raw answer.
    fn has_general(&mut self, ty: i32) -> i32;
    /// A word of the constants record at `off` ([`constant`]).
    fn constant(&mut self, off: u32) -> i32;
    /// A short of the leader's record at `off` ([`leader`]).
    fn leader_short(&mut self, off: u32) -> i16;
    /// The leader's bytes `+0x6da7`, `+0x6dcf` have bit 4.
    fn obsidian(&mut self) -> bool;
    /// Slot `0x148` of `this` is `ObjectData::has_objmask`.
    fn slot_148_is_base(&mut self) -> bool;
    /// The type's slot `0xf0`, handed the mask.
    fn type_slot_f0(&mut self, mask: i32) -> i32;
    /// Slot `0x148` of `this`, overridden, handed the mask.
    fn slot_148(&mut self, mask: i32) -> i32;
    /// Slot `0xb8` of `this` is `ObjectData::is`.
    fn slot_b8_is_base(&mut self) -> bool;
    /// The type's slot `0x60`, handed `(ty, 0)`.
    fn type_slot_60(&mut self, ty: i32) -> i32;
    /// Slot `0xb8` of `this`, overridden, handed `(ty, 0)`.
    fn slot_b8(&mut self, ty: i32) -> i32;
    /// The type's slot `0x10c`.
    fn type_slot_10c(&mut self) -> i32;
    /// The type's word `+0x218`.
    fn type_word_218(&mut self) -> i32;
    /// The type's word `+0x40`.
    fn type_word_40(&mut self) -> i32;
}

/// `this->has_objmask(mask)` as the function calls it.
fn has_objmask<V: UnitAttackView>(v: &mut V, mask: i32) -> bool {
    if v.slot_148_is_base() {
        v.type_slot_f0(mask) != 0
    } else {
        v.slot_148(mask) != 0
    }
}

/// `this->is(ty, 0)` as the function calls it.
fn is<V: UnitAttackView>(v: &mut V, ty: i32) -> bool {
    if v.slot_b8_is_base() {
        v.type_slot_60(ty) != 0
    } else {
        v.slot_b8(ty) != 0
    }
}

fn add10<V: UnitAttackView>(v: &mut V, attack: i32, off: u32) -> i32 {
    attack.wrapping_add(v.constant(off).wrapping_mul(10))
}

/// `UnitData::attack@006103c0`.
pub fn unit_attack_steps<V: UnitAttackView>(v: &mut V) -> i32 {
    let mut attack = v.base_attack();
    if attack == 0 {
        return 0;
    }
    if v.has_wonder(0x211) {
        attack = add10(v, attack, constant::TERRA_COTTA_ATTACK);
    }
    if v.has_general(0x164) >= 0 {
        attack = add10(v, attack, constant::THECITIZEN_ATTACK_BONUS);
    }
    if has_objmask(v, 0x4000) && v.obsidian() {
        attack = add10(v, attack, constant::OBSIDIAN_ARCHERS_ATTACK);
    }
    if has_objmask(v, 0x400) && v.obsidian() {
        attack = add10(v, attack, constant::OBSIDIAN_ARCHERS_ATTACK);
    }
    if v.type_word_218() == 2 && v.has_wonder(0x21e) {
        let scale = v.constant(constant::SPACE_AIR_ATTACK).wrapping_add(100);
        attack = scale.wrapping_mul(attack) / 100;
    }
    if v.leader_short(leader::ALEXANDER) != 0 && v.has_general(0x166) >= 0 {
        attack = add10(v, attack, constant::ALEXANDER_ATTACK);
        if is(v, 0x84) {
            attack = add10(v, attack, constant::ALEXANDER_HEAVY_INF_ATTACK);
        }
    }
    if v.leader_short(leader::NAPOLEON) != 0 && v.has_general(0x167) >= 0 && v.type_slot_10c() != 0
    {
        attack = add10(v, attack, constant::NAPOLEON_SIEGE_ATTACK);
    }
    if v.leader_short(leader::PARMENIO) != 0 && is(v, 0xe3) && v.has_general(0x168) >= 0 {
        attack = add10(v, attack, constant::PARMENIO_HEAVY_CAV_ATTACK);
    }
    if v.leader_short(leader::PTOLEMY) != 0 && is(v, 0x109) && v.has_general(0x16a) >= 0 {
        attack = add10(v, attack, constant::PTOLEMY_SIEGE_ATTACK);
    }
    if v.leader_short(leader::SPITAMENES) != 0 && is(v, 0xbb) && v.has_general(0x16d) >= 0 {
        attack = add10(v, attack, constant::SPITAMENES_HORSE_ARCHER_ATTACK);
    }
    if v.leader_short(leader::BLUCHER) != 0
        && v.type_word_40() == TYPE_CAVALRY
        && v.has_general(0x175) >= 0
    {
        attack = add10(v, attack, constant::BLUCHER_CAV_ATTACK);
    }
    if v.leader_short(leader::DJEZZAR) != 0 && v.has_general(0x172) >= 0 {
        attack = add10(v, attack, constant::DJEZZAR_ATTACK);
    }
    if v.leader_short(leader::MEMNON) != 0 && v.has_general(0x16c) >= 0 && is(v, 0x18a) {
        attack = add10(v, attack, constant::MEMNON_MERC_ATTACK);
    }
    if v.leader_short(leader::THEMONARCH) != 0
        && v.type_word_40() == TYPE_CAVALRY
        && v.has_general(0x162) >= 0
    {
        attack = add10(v, attack, constant::THEMONARCH_CAV_ATTACK);
    }
    attack
}
