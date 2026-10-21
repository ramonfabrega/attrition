//! **`SpellTypeData::is_valid_target@006763c0`** (item 3395), as a pure walk
//! over a [`ValidTargetView`]: may player `who` cast the craft on the object
//! `objects[whom][ox]`, and if not, which refusal code does the out-pointer
//! carry.
//!
//! Every leaf is one virtual call on the object, one direct callee or one
//! inline read of the listing, asked in the listing's order, which is what the
//! sweep holds (`sweep/spell_type_is_valid_target.rs`). The sweep's port is a
//! view over the emulated original's windows. **No `Sim` view yet**: the
//! crate's own reading of this function is [`crate::Sim::spell_valid_target`]
//! (`cast.rs`), written arm by arm over `Obj`; this walk is the function's
//! whole shape, for it to be rewired onto once its leaves (the object's
//! slots `0x18`..`0x148`, the tile owner) have crate names.
//!
//! The leaves, by the slot the original asks:
//!
//! ```text
//! 0x18 is_unit    0x1c is_building   0x148 has_objmask(0x200000)
//! 0x20 (a land unit, the Bribe arm's tile test)       0xb8 is(type, 0)
//! 0xc4 / 0x120 / 0xcc  (the Spy-Sniper's "fights" disjunction)
//! 0xc8, 0xd0 (refusals)     0xdc(p)  (infiltrated by p; -1 is anyone)
//! 0x48(p, 0)  (a target p sees)
//! ```
//!
//! `has_objmask` and `is` are devirtualised in the listing: when the slot
//! holds the base implementation (`0x46cf10`, `0x653790`) the call goes to
//! the object's type record's slot (`0xf0`, `0x60`) directly. That choice is
//! the view's, not the walk's: both arms ask the same question.

/// `SpellType` rows this function names.
pub mod craft {
    /// Bribe.
    pub const BRIBE: i32 = 0x275;
    /// Pilfer.
    pub const PILFER: i32 = 0x276;
    /// Counterintelligence.
    pub const COUNTERINTEL: i32 = 0x277;
    /// Informer.
    pub const INFORMER: i32 = 0x27f;
    /// Sabotage.
    pub const SABOTAGE: i32 = 0x280;
    /// Sniper.
    pub const SNIPER: i32 = 0x281;
}

/// `TypeIndex` rows the `is(type, 0)` tests ask.
pub const SPY: i32 = 0x3a;
/// The Sniper's "hero of the field" test row.
pub const SNIPER_LINE: i32 = 0x45;
/// The CEO whose reach `has_general` asks for.
pub const CEO: i32 = 0x165;
/// `has_objmask`'s mask: vehicles.
pub const VEHICLE_MASK: i32 = 0x20_0000;

/// Which table of object pointers is asked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Table {
    /// `objects` (`0x00c0ab84` + `0x1c` a player).
    Objects,
    /// `units` (`0x00c0aec0`): the same objects as units.
    Units,
}

/// What the walk asks of the craft's row, the players and the object.
pub trait ValidTargetView {
    /// The craft row's `spell_flags` (`+0x1c8`).
    fn flags(&self) -> i32;
    /// The craft row's index (`+0x4`).
    fn spell(&self) -> i32;
    /// `*param_4 = code`, when the pointer is non-null.
    fn set_out(&mut self, code: i32);
    /// Slot `0x18` of `objects[whom][ox]`.
    fn is_unit(&mut self, whom: i32, ox: i32) -> bool;
    /// Slot `0x1c`.
    fn is_building(&mut self, whom: i32, ox: i32) -> bool;
    /// Slot `0x148(0x200000)` of the table's object.
    fn has_objmask(&mut self, table: Table, whom: i32, ox: i32) -> bool;
    /// `LeaderData::is_enemy(leaders[who], whom)`.
    fn is_enemy(&mut self, who: i32, whom: i32) -> bool;
    /// `LeaderData::is_ally`.
    fn is_ally(&mut self, who: i32, whom: i32) -> bool;
    /// `LeaderData::is_peace`.
    fn is_peace(&mut self, who: i32, whom: i32) -> bool;
    /// `ObjectData::is_active_build`.
    fn is_active_build(&mut self, whom: i32, ox: i32) -> bool;
    /// The object's flag byte `+8`, bit `0x20` (a city).
    fn object_is_city(&self, whom: i32, ox: i32) -> bool;
    /// Slot `0xdc(arg)`.
    fn slot_dc(&mut self, whom: i32, ox: i32, arg: i32) -> bool;
    /// `ObjectData::has_general(obj, 0, 0x165)`: signed.
    fn has_general(&mut self, whom: i32, ox: i32) -> i32;
    /// Slot `0xc8`.
    fn slot_c8(&mut self, whom: i32, ox: i32) -> bool;
    /// Slot `0x20`.
    fn slot_20(&mut self, whom: i32, ox: i32) -> bool;
    /// `ObjectData::is(ty, 0)`, slot `0xb8`.
    fn is_type(&mut self, whom: i32, ox: i32, ty: i32) -> bool;
    /// Slot `0x120`.
    fn slot_120(&mut self, whom: i32, ox: i32) -> bool;
    /// Slot `0xc4`.
    fn slot_c4(&mut self, whom: i32, ox: i32) -> bool;
    /// Slot `0xd0`.
    fn slot_d0(&mut self, whom: i32, ox: i32) -> bool;
    /// Slot `0xcc`.
    fn slot_cc(&mut self, whom: i32, ox: i32) -> bool;
    /// `ObjectData::is_worker`.
    fn is_worker(&mut self, whom: i32, ox: i32) -> bool;
    /// `UnitData::is_merchant`.
    fn is_merchant(&mut self, whom: i32, ox: i32) -> bool;
    /// The word at `+0x218` of the object's type record (its domain).
    fn type_domain(&self, whom: i32, ox: i32) -> i32;
    /// `Game::teams_locked`.
    fn teams_locked(&mut self, who: i32) -> bool;
    /// `leaders[whom]`'s first byte, bit `2`.
    fn leader_bit2(&self, whom: i32) -> bool;
    /// The owner byte of the tile under the object (`world` tile `+0xf`).
    fn tile_owner(&mut self, whom: i32, ox: i32) -> i32;
    /// Slot `0x48(who, 0)`.
    fn slot_48(&mut self, who: i32, whom: i32, ox: i32) -> bool;
}

/// `006763c0`..`676a5b`.
pub fn is_valid_target_steps(v: &mut impl ValidTargetView, who: i32, ox: i32, whom: i32) -> bool {
    v.set_out(0);
    let f = v.flags();
    if f & 0xe == 0 || f & 8 != 0 {
        return true;
    }
    let spell = v.spell();
    if v.is_unit(whom, ox)
        && f & 2 == 0
        && spell == craft::SABOTAGE
        && !v.has_objmask(Table::Objects, whom, ox)
    {
        v.set_out(1);
        return false;
    }
    if v.is_building(whom, ox) && f & 4 == 0 {
        v.set_out(2);
        return false;
    }
    if who == whom {
        if f & 0x10 == 0 {
            v.set_out(3);
            return false;
        }
    } else if v.is_enemy(who, whom) {
        if f & 0x20 == 0 {
            v.set_out(6);
            return false;
        }
    } else if v.is_ally(who, whom) {
        if f & 0x80 == 0 {
            v.set_out(4);
            return false;
        }
    } else if v.is_peace(who, whom) && f & 0x400 == 0 {
        v.set_out(5);
        return false;
    }
    v.set_out(8);
    if v.is_unit(whom, ox) {
        if f & 0x100 != 0 && !v.has_objmask(Table::Units, whom, ox) {
            return false;
        }
        if f & 0x200 != 0 && v.has_objmask(Table::Units, whom, ox) {
            return false;
        }
    }
    if spell == craft::PILFER && (!v.is_active_build(whom, ox) || !v.object_is_city(whom, ox)) {
        return false;
    }
    if spell == craft::INFORMER && v.slot_dc(whom, ox, who) {
        return false;
    }
    if spell == craft::BRIBE && !bribe_arm(v, who, ox, whom) {
        return false;
    }
    if spell == craft::SNIPER && !sniper_arm(v, ox, whom) {
        return false;
    }
    if spell == craft::COUNTERINTEL && !counterintel_arm(v, who, ox, whom) {
        return false;
    }
    v.set_out(0);
    true
}

/// `00676674`..`676859`.
fn bribe_arm(v: &mut impl ValidTargetView, who: i32, ox: i32, whom: i32) -> bool {
    if v.has_general(whom, ox) >= 0 || v.slot_c8(whom, ox) || whom == who {
        return false;
    }
    if v.is_ally(who, whom) {
        // `Game::teams_locked`, and the target's leader not flagged `2`.
        return v.teams_locked(who) && !v.leader_bit2(whom);
    }
    if !v.is_enemy(who, whom) {
        return false;
    }
    if v.slot_20(whom, ox) {
        return v.tile_owner(whom, ox) == who;
    }
    if v.is_type(whom, ox, SPY) {
        return false;
    }
    if !v.slot_120(whom, ox) && !v.slot_c4(whom, ox) {
        return false;
    }
    !(v.slot_d0(whom, ox)
        || v.is_worker(whom, ox)
        || v.slot_cc(whom, ox)
        || v.is_merchant(whom, ox)
        || v.type_domain(whom, ox) != 0)
}

/// `00676859`..`676962`.
fn sniper_arm(v: &mut impl ValidTargetView, ox: i32, whom: i32) -> bool {
    if v.has_general(whom, ox) >= 0 || v.slot_c8(whom, ox) || v.type_domain(whom, ox) != 0 {
        return false;
    }
    if !v.slot_120(whom, ox)
        && !v.slot_c4(whom, ox)
        && !v.slot_cc(whom, ox)
        && !v.is_type(whom, ox, SNIPER_LINE)
    {
        return false;
    }
    !(v.slot_d0(whom, ox) || v.is_worker(whom, ox) || v.is_merchant(whom, ox))
}

/// `00676962`..`676a46`.
fn counterintel_arm(v: &mut impl ValidTargetView, who: i32, ox: i32, whom: i32) -> bool {
    if !v.slot_48(who, whom, ox) {
        return false;
    }
    if whom == who {
        return v.slot_dc(whom, ox, -1);
    }
    if v.is_ally(who, whom) {
        return v.slot_dc(whom, ox, -1) && !v.slot_dc(whom, ox, who);
    }
    v.is_enemy(who, whom) && v.is_type(whom, ox, SPY)
}
