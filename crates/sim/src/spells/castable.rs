//! `SpellTypeData::is_castable@00675bc0` as steps over a view (item 3364):
//! whether the unit or building `(who, idx)` may be handed this craft, and
//! in which of the four answers — `0` no, `1` not now (the depopulation
//! queue is full), `2` no, here, `3` yes. [`is_castable_steps`] is the
//! original's control flow in its order; [`CastableView`] is what it reads
//! and the callees it makes, which `sweep/spell_type_data_is_castable.rs`
//! answers from the emulated original's windows.
//!
//! **The head** asks whether the object `is` the craft's `FROM` or `FROM2`
//! line — by the raw type, then through the leader's tribe substitution
//! (`tribe_can_type`, else the type's own slot `0x68` of the leader), then
//! through the leader's graft — and a craft that is neither Disband-like
//! (`0x29a`, `0x28a`) is refused outright. A decoy-like unit (`unit +0x68 &
//! 1`, with the craft's own two slots `0x50`/`0x54` both false) is refused
//! next. Then the craft's own case: Rally (`0x27e`) wants the unit on a
//! tile of the leader's own borders that holds one of the leader's cities;
//! `0x284` a tile of an ally; `0x287` (a research project) a building that
//! is not unassimilated, not already researching, and not a city under
//! someone else's capture; `0x28a` the object's slot `0x110`; the paired
//! locks and unlocks `0x28b..0x292` the unit on the map, with the flag
//! `0x80000` clear for the odd ones and set for the even; To Arms (`0x294`)
//! the leader holding MILITIA (`0x42`), MINUTEMAN (`0x43`) or PARTISAN
//! (`0x44`) — each through its `tribe_can_type` substitution — and the tile
//! no one's, the leader's or an ally's; `0x297` and `0x298` the unit on the
//! map; `0x29a` the disband rules of the object's city. `0x286` (depopulate)
//! is the one case that does not answer: it either answers `1` or goes round
//! again with the craft record the global spell table holds at `+0xa68`.
//!
//! **What the crate does not carry.** [`crate::Sim::spell_castable`] is a
//! partial reading of this on the crate's own data (the head's lineage test
//! and To Arms; its SEAM names the rest). It is not this function and is
//! not changed here; there is no `Sim` view yet, so this module has no
//! caller in the simulation.

/// What `SpellTypeData::is_castable` reads of the world and the callees it
/// makes. The craft is `this` (and, after a depopulation round, the record at
/// `spelltypes + 0xa68`, [`CastableView::next_spell`]); the object is
/// `objects[who][idx]` and its unit record `units[who][idx]`.
pub trait CastableView {
    /// The craft's own number (`this + 4`).
    fn id(&mut self) -> i32;
    /// The craft's `FROM` type (`this + 0x3c`).
    fn from(&mut self) -> i32;
    /// The craft's `FROM2` type (`this + 0x1dc`).
    fn from2(&mut self) -> i32;
    /// `this := spelltypes[0xa68]`.
    fn next_spell(&mut self);
    /// The craft's vslot `0x50`.
    fn spell_slot50(&mut self) -> i32;
    /// The craft's vslot `0x54`.
    fn spell_slot54(&mut self) -> i32;
    /// The object's `is(ty, 0)`: vslot `0xb8`, or, when that is
    /// `ObjectData::is`, the data record's vslot `0x60`.
    fn obj_is(&mut self, ty: i32) -> i32;
    /// The object's vslot `0x18`.
    fn obj_slot18(&mut self) -> i32;
    /// The object's vslot `0x20`.
    fn obj_slot20(&mut self) -> i32;
    /// The object's vslot `0x110`.
    fn obj_slot110(&mut self) -> i32;
    /// `ObjectData::is_map_unit(obj)`.
    fn is_map_unit(&mut self) -> i32;
    /// `units[who][idx] + 0x68`, whole.
    fn unit_flags(&mut self) -> u32;
    /// `leader.tribe_can_type(types[ty])`.
    fn tribe_can_type(&mut self, ty: i32) -> i32;
    /// `types[ty]`'s vslot `0x68`, handed the leader's `who` (`+8`).
    fn type_slot68(&mut self, ty: i32) -> i32;
    /// `leader.get_graft(ty)`.
    fn get_graft(&mut self, ty: i32) -> i32;
    /// `leader.has_tech(ty)`.
    fn has_tech(&mut self, ty: i32) -> i32;
    /// `leader.researching(id, idx, 1, 0)`.
    fn researching(&mut self, id: i32) -> i32;
    /// `leader.is_ally(whom)`.
    fn is_ally(&mut self, whom: i32) -> i32;
    /// Whether leader `i`'s byte `+0` has bit 2 (`leaders[i]`, any index).
    fn leader_flag2(&mut self, i: i32) -> bool;
    /// The word at `tile_flags[div3[y >> 6] * w18 + div3[x >> 6]]` has bit
    /// `0x100`, `(x, y)` the object's decoded position.
    fn tile_flag100(&mut self) -> bool;
    /// The signed owner byte of the `>> 8` tile under the object (`+0xf` of
    /// a `0x1c`-byte record).
    fn tile_owner(&mut self) -> i32;
    /// `objects.find_city_at(x, y, who, -1, 0)`.
    fn find_city_at(&mut self) -> i32;
    /// The object's vslot `0x3c`: its data record, which the `rec_*` reads
    /// below are of (each call is one call of the slot).
    fn rec(&mut self);
    /// `BuildData::is_unassimilated(rec)`.
    fn rec_unassimilated(&mut self) -> i32;
    /// `(short) rec + 0x72`: the object's city.
    fn rec_city(&mut self) -> i32;
    /// `rec + 8`.
    fn rec_flag8(&mut self) -> u32;
    /// `(byte) (*(rec + 0x18) + 0x2c0)`.
    fn rec_cfg(&mut self) -> u32;
    /// `(char) cities[who][c] + 0x5f`.
    fn city_owner(&mut self, c: i32) -> i32;
    /// `(char) cities[who][c] + 0x60`.
    fn city_b60(&mut self, c: i32) -> i32;
    /// `(byte) cities[who][c] + 0x68`.
    fn city_flags68(&mut self, c: i32) -> u32;
    /// `(byte) cities[who][c] + 4`.
    fn city_flags4(&mut self, c: i32) -> u32;
    /// `CityData::depop_can_queue(cities[who][c])`.
    fn depop_can_queue(&mut self, c: i32) -> i32;
    /// `CityData::is_disbanding(cities[who][c])`.
    fn is_disbanding(&mut self, c: i32) -> i32;
}

/// MILITIA, MINUTEMAN, PARTISAN: the types To Arms asks the leader for, by
/// their slot in the type table (`0x108/4`, `0x10c/4`, `0x110/4`).
const TO_ARMS_TYPES: [i32; 3] = [0x42, 0x43, 0x44];

/// `SpellTypeData::is_castable@00675bc0(this, idx, who, param_3)`.
pub fn is_castable_steps<V: CastableView>(v: &mut V, who: i32, param_3: i32) -> i32 {
    loop {
        // The head: `FROM`, `FROM2`, the tribe's substitute, the graft.
        let from = v.from();
        let mut pass = v.obj_is(from) != 0;
        if !pass {
            let from2 = v.from2();
            pass = v.obj_is(from2) != 0;
        }
        if !pass {
            let from = v.from();
            let arg = if from >= 0 {
                if v.tribe_can_type(from) != 0 {
                    from
                } else {
                    v.type_slot68(from)
                }
            } else {
                from
            };
            pass = v.obj_is(arg) != 0;
            if !pass {
                let from2 = v.from2();
                let g = v.get_graft(from2);
                pass = v.obj_is(g) != 0;
                if !pass {
                    let id = v.id();
                    if id != 0x29a && id != 0x28a {
                        return 0;
                    }
                }
            }
        }
        if v.obj_slot18() != 0
            && v.unit_flags() & 1 != 0
            && v.spell_slot50() == 0
            && v.spell_slot54() == 0
        {
            return 0;
        }
        let id = v.id();
        match id {
            0x27e => {
                if v.is_map_unit() == 0 {
                    return 0;
                }
                if !v.tile_flag100() || v.tile_owner() != who {
                    return 2;
                }
                let c = v.find_city_at();
                if c < 0 || v.city_owner(c) != who {
                    return 2;
                }
            }
            0x284 => {
                if v.is_map_unit() == 0 {
                    return 0;
                }
                let owner = v.tile_owner();
                if owner < 0 || v.is_ally(owner) == 0 {
                    return 2;
                }
            }
            0x286 => {
                v.rec();
                let c = v.rec_city();
                if c >= 0 && param_3 == 0 && v.depop_can_queue(c) == 0 {
                    return 1;
                }
                v.next_spell();
                continue;
            }
            0x287 => {
                v.rec();
                if v.rec_unassimilated() != 0 {
                    return 0;
                }
                if param_3 == 0 && v.researching(id) != 0 {
                    return 0;
                }
                v.rec();
                let c = v.rec_city();
                let b = v.city_b60(c);
                // `shl cl` takes the count mod 32 and the test is a byte's.
                let bit = (1u32 << (b as u32 & 31)) as u8;
                if v.city_flags68(c) as u8 & bit != 0 && b != who && v.leader_flag2(b) {
                    return 0;
                }
            }
            0x288 | 0x289 => return 0,
            0x28a => {
                if v.obj_slot110() == 0 {
                    return 0;
                }
            }
            0x28b | 0x28d | 0x28f | 0x291 => {
                if v.is_map_unit() == 0 || v.unit_flags() & 0x80000 != 0 {
                    return 0;
                }
            }
            0x28c | 0x28e | 0x290 | 0x292 => {
                if v.is_map_unit() == 0 || v.unit_flags() & 0x80000 == 0 {
                    return 0;
                }
            }
            0x294 => {
                if v.is_map_unit() == 0 {
                    return 0;
                }
                let mut held = false;
                for t in TO_ARMS_TYPES {
                    let arg = if v.tribe_can_type(t) != 0 {
                        t
                    } else {
                        v.type_slot68(t)
                    };
                    if v.has_tech(arg) != 0 {
                        held = true;
                        break;
                    }
                }
                if !held {
                    return 0;
                }
                let owner = v.tile_owner();
                if owner >= 0 && owner != who && v.is_ally(owner) == 0 {
                    return 0;
                }
            }
            0x297 | 0x298 => {
                if v.is_map_unit() == 0 {
                    return 0;
                }
            }
            0x29a => {
                if param_3 == 0 && v.researching(id) != 0 {
                    return 0;
                }
                if v.obj_slot20() == 0 {
                    return 0;
                }
                v.rec();
                let mut go = v.rec_cfg() & 0x10 == 0;
                if !go {
                    v.rec();
                    go = v.rec_flag8() & 0x20 != 0;
                }
                if go {
                    v.rec();
                    let c = v.rec_city();
                    v.rec();
                    if c >= 0 {
                        if v.rec_flag8() & 0x20 == 0 {
                            if v.is_disbanding(c) > i32::from(param_3 != 0) {
                                return 0;
                            }
                        } else {
                            if v.city_flags4(c) & 0x10 != 0 {
                                return 0;
                            }
                            let held = v.city_flags68(c) as u8;
                            if held != 0 {
                                for i in 0..8 {
                                    if who != i && held & (1 << i) != 0 && v.leader_flag2(i) {
                                        return 0;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        return 3;
    }
}
