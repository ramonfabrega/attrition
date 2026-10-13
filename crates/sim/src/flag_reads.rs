//! **Flag-word and constant readers** (item 1705). The original answers a
//! handful of "is it X?" questions by masking one word of a record; the port
//! keeps each answer as a field of its own and never needed the mask. These
//! are the masks, each backed by `crates/sim/src/sweep/` against the
//! executable and each naming its reader:
//!
//! - [`sub_object_is_active`], [`wall_is_active`], [`wall_is_started`]: the
//!   object flags byte at `+0x8` (`SubObjectData` / `WallData`) — `& 1`
//!   (placed), `& 4` (active: `Wall::process`'s idle-site gate, AI §162) and
//!   `& 2` (started: `find_any_building`'s cell-circle arm, AI §79);
//! - [`unit_has_repeat_air`]: `unit_masks & 0x200000`, vslot `0xf0`, the
//!   unit's twin of [`crate::Building::repeat_air`];
//! - [`unit_is_blown`]: `unit_masks & 0x1000`;
//! - [`NUM_PREQ`]: `UnitTypeData::num_preq`, a constant.
//!
//! Item 1708 adds [`object_get_captain`] (the signed short at `+0xa`, a
//! building's `o`), [`tcoord_less`] (`TCoord::operator<`, a signed compare of
//! the two words) and [`SUPPLIES_LEFT`] (`UnitData::supplies_left`, a
//! constant).

/// `SubObjectData::is_active@0046cda0`: flags byte `& 1`.
pub const fn sub_object_is_active(flags: u8) -> bool {
    flags & 1 != 0
}
/// `WallData::is_active@00472350`: flags byte `& 4`.
pub const fn wall_is_active(flags: u8) -> bool {
    flags & 4 != 0
}
/// `WallData::is_started@00472360`: flags byte `& 2`.
pub const fn wall_is_started(flags: u8) -> bool {
    flags & 2 != 0
}
/// `UnitData::has_repeat_air@0046cec0`: `unit_masks & 0x200000`.
pub const fn unit_has_repeat_air(unit_masks: u32) -> bool {
    unit_masks & 0x20_0000 != 0
}
/// `UnitData::is_blown@0060a6e0`: `unit_masks & 0x1000`.
pub const fn unit_is_blown(unit_masks: u32) -> bool {
    unit_masks & 0x1000 != 0
}
/// `UnitTypeData::num_preq@00470810`: always 2.
pub const NUM_PREQ: i64 = 2;
/// `ObjectData::get_captain@00472400`: the signed short at `+0xa` (`o`),
/// sign-extended. For a building it is the building's own number, which
/// `fight`'s attacker side reads.
pub const fn object_get_captain(o: i16) -> i32 {
    o as i32
}
/// `TCoord::operator<@00472b30`: a signed compare of the two words.
pub const fn tcoord_less(a: i32, b: i32) -> bool {
    a < b
}
/// `UnitData::supplies_left@00609a20`: always 1.
pub const SUPPLIES_LEFT: i64 = 1;
