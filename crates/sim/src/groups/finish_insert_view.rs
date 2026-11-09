//! **`Group::finish_insert@0070e620`'s dispatch** (item 3372): the loop that
//! pops each order `Group::set_up_insert` copied aside and re-issues it as a
//! group action at `QUEUE_LAST`, as a pure walk over a
//! [`FinishInsertView`]. The sweep's port is a view over the emulated
//! original's windows; the crate's is the view `group.rs` lays over an
//! `Insert` (`docs/GROUPS.md` §6.2, §17).
//!
//! The control flow is the listing's: one virtual call or direct callee per
//! leaf, in the order the listing asks. What it holds, case by `get_type()`:
//!
//! - `1`..`4` (`MOVE_TO`, `ATTACK_TO`, `EXPLORE_TO`, `FLEE_TO`) and `0x13`,
//!   `0x15` (the group moves): `action_move_near` to the record's `+0x44/+0x48`
//!   when both are non-negative, else its `+4/+8`; the grouped two also hand
//!   `+0x14`;
//! - `6` and `0xd` (`BUILD`, `REPAIR`): `action_swarm_around` with the
//!   virtual base's flags byte `& 4` read through the vbtable;
//! - `7 8 0xa 0xb 0xc 0xe 0xf 0x12 0x16 0x17 0x1a`: the other eleven callees,
//!   each handed its record's words.
//!
//! A kind outside that list (`0`, `5`, `9`, `0x10`, `0x11`, `0x14`, `0x18`,
//! `0x19`, above `0x1a`) does nothing, and the order is given back all the
//! same.

/// `QUEUE_LAST`.
pub const QUEUE_LAST: i32 = 1;
/// `OrderIndex`'s `MOVE_TO`.
pub const MOVE_TO: i32 = 1;
/// `ATTACK_TO`.
pub const ATTACK_TO: i32 = 2;
/// `EXPLORE_TO`.
pub const EXPLORE_TO: i32 = 3;
/// `FLEE_TO`.
pub const FLEE_TO: i32 = 4;
/// `REPAIR`.
pub const REPAIR: i32 = 0xd;

/// What `finish_insert` reads and calls. Slots are the order's vtable
/// offsets; `word(slot, off)` is the 32-bit word at `off` of the record that
/// slot answers.
pub trait FinishInsertView {
    /// `[list+0x14] != 0`: the cursor is set to it (`+0xc`, `+4`, `+8`).
    fn reseat(&mut self);
    /// `[list+0x10] != 0`: the count the loop tests.
    fn pending(&mut self) -> bool;
    /// `pUVar1 = [list+4]`, `remove_current` called; whether the order is non-null.
    fn take(&mut self) -> bool;
    /// Vtable slot `+0x10`, `get_type()`.
    fn kind(&mut self) -> i32;
    /// The word at `off` of the record slot `slot` answers.
    fn word(&mut self, slot: u32, off: u32) -> i32;
    /// The record's address itself, handed on (`redo_patrol_order`'s first argument).
    fn record(&mut self, slot: u32) -> i32;
    /// The virtual base's flags byte of the record slot `slot` answers, `& 4`.
    fn flags4(&mut self, slot: u32) -> i32;
    /// What `ecx` holds after the accessor call: the original hands it on as
    /// `redo_patrol_order`'s and `action_form`'s last-but-one argument.
    fn left_ecx(&mut self) -> i32;
    /// `OrdersMemManager::give_obj`, handed the order.
    fn give_obj(&mut self);
    /// `Group::action_move_near`, its eleven arguments.
    fn action_move_near(&mut self, a: [i32; 11]);
    /// `Group::action_swarm_around`.
    fn action_swarm_around(&mut self, a: [i32; 5]);
    /// `Group::action_gather`.
    fn action_gather(&mut self, a: [i32; 2]);
    /// `Group::action_board_ship`.
    fn action_board_ship(&mut self, a: [i32; 2]);
    /// `Group::action_attack`.
    fn action_attack(&mut self, a: [i32; 5]);
    /// `Group::action_follow`.
    fn action_follow(&mut self, a: [i32; 3]);
    /// `Group::action_guard`.
    fn action_guard(&mut self, a: [i32; 4]);
    /// `Group::action_spell`.
    fn action_spell(&mut self, a: [i32; 5]);
    /// `Group::action_trade`.
    fn action_trade(&mut self, a: [i32; 5]);
    /// `Group::action_form`.
    fn action_form(&mut self, a: [i32; 5]);
    /// `Group::redo_patrol_order`.
    fn redo_patrol_order(&mut self, a: [i32; 2]);
    /// `Group::action_attack_ground`.
    fn action_attack_ground(&mut self, a: [i32; 3]);
    /// `Group::action_garrison`.
    fn action_garrison(&mut self, a: [i32; 4]);
}

fn move_near<V: FinishInsertView>(v: &mut V, slot: u32, kind: i32, grouped: bool) {
    let (ox, oy) = (v.word(slot, 0x44), v.word(slot, 0x48));
    let (x, y) = if ox < 0 || oy < 0 {
        (v.word(slot, 4), v.word(slot, 8))
    } else {
        (ox, oy)
    };
    let p3 = if grouped { v.word(slot, 0x14) } else { 0 };
    let angle = v.word(slot, 0xc);
    v.action_move_near([x, y, p3, QUEUE_LAST, 1, angle, kind, 1, -1, -1, 0]);
}

/// The loop at `0070e620`.
pub fn finish_insert<V: FinishInsertView>(v: &mut V) {
    v.reseat();
    while v.pending() {
        if !v.take() {
            continue;
        }
        match v.kind() {
            1 => move_near(v, 0x40, MOVE_TO, false),
            2 => move_near(v, 0x44, ATTACK_TO, false),
            3 => move_near(v, 0x48, EXPLORE_TO, false),
            4 => move_near(v, 0x4c, FLEE_TO, false),
            0x13 => move_near(v, 0x98, MOVE_TO, true),
            0x15 => move_near(v, 0xa0, ATTACK_TO, true),
            6 => {
                let (a, b) = (v.word(0x60, 8), v.word(0x60, 0xc));
                let flags = v.flags4(0x60);
                let kind = v.kind();
                v.action_swarm_around([a, b, QUEUE_LAST, kind, flags]);
            }
            7 => {
                let a = v.word(0x6c, 8);
                v.action_gather([a, QUEUE_LAST]);
            }
            8 => {
                let a = v.word(0x64, 8);
                v.action_board_ship([a, QUEUE_LAST]);
            }
            0xa => {
                let (a, b) = (v.word(0x50, 8), v.word(0x50, 0xc));
                let byte = v.word(0x50, 0x1c) & 0xff;
                v.action_attack([a, b, byte, QUEUE_LAST, 0]);
            }
            0xb => {
                let (a, b) = (v.word(0x78, 8), v.word(0x78, 0xc));
                v.action_follow([a, b, QUEUE_LAST]);
            }
            0xc => {
                let (a, b) = (v.word(0x74, 8), v.word(0x74, 0xc));
                v.action_guard([a, b, QUEUE_LAST, 0]);
            }
            0xd => {
                let (a, b) = (v.word(0x70, 8), v.word(0x70, 0xc));
                let flags = v.flags4(0x70);
                v.action_swarm_around([a, b, QUEUE_LAST, REPAIR, flags]);
            }
            0xe => {
                let t = v.word(0x7c, 0x20);
                let (a, b) = (v.word(0x7c, 8), v.word(0x7c, 0xc));
                let (c, d) = (v.word(0x7c, 0x14), v.word(0x7c, 0x18));
                v.action_spell([t, a, b, c, d]);
            }
            0xf => {
                let (a, b) = (v.word(0x80, 8), v.word(0x80, 0xc));
                let (c, d) = (v.word(0x80, 0x14), v.word(0x80, 0x18));
                v.action_trade([a, b, c, d, QUEUE_LAST]);
            }
            0x12 => {
                let a = v.word(0x90, 0x50);
                let e = v.left_ecx();
                v.action_form([a, 0, QUEUE_LAST, e, 1]);
            }
            0x16 => {
                let r = v.record(0xa4);
                let e = v.left_ecx();
                v.redo_patrol_order([r, e]);
            }
            0x17 => {
                let (a, b) = (v.word(0x54, 4), v.word(0x54, 8));
                v.action_attack_ground([a, b, QUEUE_LAST]);
            }
            0x1a => {
                let (a, b) = (v.word(0xac, 8), v.word(0xac, 0xc));
                v.action_garrison([a, b, QUEUE_LAST, 0]);
            }
            _ => {}
        }
        v.give_obj();
    }
}
