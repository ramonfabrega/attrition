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
/// `BuildTypeData::num_preq@00472a60`: always 3 (a building or a tech).
pub const BUILD_NUM_PREQ: i64 = 3;
/// `Game::is_raining@00594480`: always 0 — the original keeps no rain.
pub const GAME_IS_RAINING: i32 = 0;
/// `Player::is_over@0043e220`: the 16-bit flags word at `+0x30` has `0x100`
/// or any of `0xd0` set.
pub const fn player_is_over(flags: u16) -> bool {
    flags & 0x100 != 0 || flags & 0xd0 != 0
}
/// `UnitTypeData::get_relative_value@0061d740`: the signed short at
/// `+0x2b0 + i*2`, sign-extended.
pub const fn relative_value(slot: i16) -> i32 {
    slot as i32
}
/// `WallData::hits@00642bb0`: the int at `+0x20` for a non-zero argument,
/// the one at `+0x54` (the construction hits) for zero.
pub const fn wall_data_hits(arg: i32, at_0x20: i32, at_0x54: i32) -> i32 {
    if arg != 0 { at_0x20 } else { at_0x54 }
}
/// `TargetOrder::get_target_order@0047ff40`: `this - 0x18`.
pub const TARGET_ORDER_BASE: i32 = 0x18;
/// `AirOrder::get_air_order@0047ff70`: `this - 0x20`.
pub const AIR_ORDER_BASE: i32 = 0x20;
/// `StrafeOrder::get_air_order@0047ff60`: `this - 0x28`.
pub const STRAFE_AIR_ORDER_BASE: i32 = 0x28;
/// `StrafeOrder::get_attack_order@0047ff50`: `this - 0x4c`.
pub const STRAFE_ATTACK_ORDER_BASE: i32 = 0x4c;
/// `FormOrder::get_form_order@00482dc0`: `this - 0x5c`.
pub const FORM_ORDER_BASE: i32 = 0x5c;
/// `GuardOrder::get_guard_order@00486920`: `this - 0x30`.
pub const GUARD_ORDER_BASE: i32 = 0x30;
/// `FollowOrder::get_follow_order@00486c60`: `this - 0x24`.
pub const FOLLOW_ORDER_BASE: i32 = 0x24;
/// `PatrolOrder::get_patrol_order@004843e0`: `this - 0x44`.
pub const PATROL_ORDER_BASE: i32 = 0x44;
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
/// `GameAccessConst::get_scary_fog@005833b0`: the address of the global `fog`.
pub const FOG_ADDR: u32 = 0x00e8_a790;
/// `GameAccessConst::get_scary_cliffs@00583370`: the address of the global `cliffs`.
pub const CLIFFS_ADDR: u32 = 0x00e8_8eb0;
/// `GameAccessConst::get_scary_doobers@00583420`: the address of the global `doobers`.
pub const DOOBERS_ADDR: u32 = 0x00ea_fda0;
/// The getters that return a global cell's word as it stands:
/// `GameAccessConst::get_scary_camera@005833a0`, `get_scary_console@005833c0`,
/// `get_scary_terrain@00583400`, `get_scary_scene@00583470`,
/// `Signposts::get_num@00897bc0`, `cur_x@00897a60`, `cur_y@00897a50`.
pub const fn global_cell(word: i32) -> i32 {
    word
}

/// The getters that return a global cell's word are [`global_cell`] too:
/// `GameAccessConst::get_scary_iface_main@00583390` (`0x00e37b9c`),
/// `get_scary_message_win@00583480` (`0x00c06208`), `get_scary_iface_current@005833d0`
/// (`0x00e39964`), `get_scary_graphic_pieces@00583490` (`0x00c06214`) and
/// `get_scary_scenario_editor@00583380` (`0x00e7f8dc`) (item 1766).
///
/// `GameAccessConst::get_scary_console_who@005834a0`: the console player's
/// record is the pointer in the cell `0x00c06210`; `who` is its word at `+0x298`.
/// `rec` is that record as words.
pub fn console_who(rec: &[i32]) -> i32 {
    rec[0x298 / 4]
}
/// `GameAccessConst::get_scary_console_leader@005833e0`: `0x00e3a390 + who *
/// 0x6eec` (a 32-bit multiply), the address of that player's leader record.
pub fn console_leader(rec: &[i32]) -> i32 {
    console_who(rec)
        .wrapping_mul(0x6eec)
        .wrapping_add(0x00e3_a390)
}
/// `Game::is_solo@0043ec40`: bit 2 of the game's flag byte at `+0x820` clear.
pub const fn game_is_solo(flags: u8) -> bool {
    flags & 4 == 0
}
/// `Game::is_team_game@00593f70`: bit 7 of the same byte.
pub const fn game_is_team_game(flags: u8) -> bool {
    flags >> 7 != 0
}
/// `MoveOrder::is_pathed@00488980`: bit 0 of the order's flag byte (a signed
/// byte, widened, then masked).
pub const fn move_order_is_pathed(flags: i8) -> bool {
    flags & 1 != 0
}
