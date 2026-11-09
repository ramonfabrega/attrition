//! **`Build::unqueue@006207c0`** (item 3399): a building takes one entry off
//! its production queue, as a pure walk over a [`BuildUnqueueView`].
//!
//! **Port or namesake.** `Queue::unqueue` (`production.rs`) is a *namesake*:
//! it removes an `Item` from the crate's own queue vector and refunds into a
//! `Ledger`, which is this function's queue and refund steps and nothing of
//! its library redirect, its per-type tallies, its AI counters or its
//! age/epoch counters (the callers in `lib.rs`/`city.rs` do those beside it,
//! on their own state). This view is therefore **standalone**: nothing in
//! `Sim` calls it, the sweep is its only caller, and whether wiring it would
//! move a diff pin is not established.
//!
//! In order:
//!
//! 1. `bVar4`: the count byte (`+0x82`) is not zero, the argument is under
//!    it and under the queue's length (`+0x88`), and entry `p1`'s type short
//!    (`+4` of the `0x14`-byte entry behind `+0x8c`) is `0x29a`;
//! 2. the library gate (`*this == Build::vftable` ? the data's slot `0x60` :
//!    this's slot `0xb8(0x1b3, 0)`) nonzero and not `bVar4`: the owner's
//!    `get_first_library` names a library; when it is not negative and not the
//!    building's own (short `+0xa`) the call is handed to that library's
//!    `unqueue` (vtable slot `0x1c8`) with `(p1 - count, p2)` and this returns;
//! 3. an empty queue, or `p1` past the last entry, returns;
//! 4. `options->rebuild` (`+0x90`) is set; with `p2` the argument walks
//!    forward while the next entry has the same type (never past the queue's
//!    length);
//! 5. the entry's word `+0` is cleared and its type read (`-1` past the
//!    length); a non-negative type takes the owner's per-type tally down one
//!    unless zero, and a unit type with record word `+0x1e8` set takes the
//!    AI counters exactly as `Unit::action_unqueue` does;
//! 6. an age type takes the owner's byte `+0x67f4` down, an epoch type the
//!    byte `+0x67f5`, both unguarded;
//! 7. `p2` calls `unpay_cost(p1)`; `p1` under `count - 1` calls
//!    `BuildQueue::un_queue(p1, count)`; the count falls, and reaching zero
//!    clears bit `0x40` of the word at `+0x60`.
//!
//! **Not established**: a final entry index past the queue's length reads
//! the type list at `-1` in the original, which the emulator faults on; the
//! sweep draws the queue length above every count so it is reached on
//! neither side, and the port panics on a negative type. A negative `p1` is
//! likewise undrawn.

use crate::unit_unqueue_view::{
    AIR_QUEUED, BARRACKS_QUEUED, COMBAT_QUEUED, DOCK_QUEUED, FACTORY_QUEUED, STABLE_QUEUED,
};

/// The entry type that skips the library redirect.
pub const NO_REDIRECT_TYPE: i16 = 0x29a;
/// Leader offset of the age counter (a byte).
pub const AGE_COUNT: usize = 0x67f4;
/// Leader offset of the epoch counter (a byte).
pub const EPOCH_COUNT: usize = 0x67f5;

/// The state `Build::unqueue` reads and the calls it makes.
pub trait BuildUnqueueView {
    /// The count byte, `+0x82`.
    fn count(&mut self) -> u8;
    /// Writes it.
    fn set_count(&mut self, n: u8);
    /// The queue's length, the int at `+0x88`.
    fn queue_len(&mut self) -> i32;
    /// The type short of queue entry `i` (`+4` of the entry).
    fn entry_type(&mut self, i: i32) -> i16;
    /// Zeroes the word at the start of entry `i`.
    fn clear_entry(&mut self, i: i32);
    /// The library gate: the vtable compare and whichever slot it picks.
    fn library_gate(&mut self) -> bool;
    /// `LeaderData::get_first_library` of the owner.
    fn first_library(&mut self) -> i32;
    /// The building's own library short, `+0xa`.
    fn own_library(&mut self) -> i16;
    /// The library's `unqueue` (slot `0x1c8`).
    fn redirect(&mut self, lib: i32, a: i32, b: i32);
    /// `options->rebuild = 1`.
    fn set_rebuild(&mut self);
    /// The owner's per-type tally.
    fn tally(&mut self, idx: i32) -> u16;
    /// Writes it.
    fn set_tally(&mut self, idx: i32, v: u16);
    /// Whether type `idx` is a unit type (slot `0xc`).
    fn type_is_unit(&mut self, idx: i32) -> bool;
    /// Whether type `idx` is an age type (slot `0x34`).
    fn type_is_age(&mut self, idx: i32) -> bool;
    /// Whether type `idx` is an epoch type (slot `0x38`).
    fn type_is_epoch(&mut self, idx: i32) -> bool;
    /// The unit record's word `+0x1e8`.
    fn unit_word_1e8(&mut self, idx: i32) -> i32;
    /// The unit record's word `+0x40`.
    fn unit_word_40(&mut self, idx: i32) -> i32;
    /// The unit record's word `+0x218`.
    fn unit_word_218(&mut self, idx: i32) -> i32;
    /// An owner counter word at `off`.
    fn counter(&mut self, off: usize) -> i32;
    /// Writes it.
    fn set_counter(&mut self, off: usize, v: i32);
    /// The owner's byte counter at `off`.
    fn byte_counter(&mut self, off: usize) -> u8;
    /// Writes it.
    fn set_byte_counter(&mut self, off: usize, v: u8);
    /// `Build::unpay_cost(this, p1)`.
    fn unpay_cost(&mut self, p1: i32);
    /// `BuildQueue::un_queue(&queue, p1, count)`.
    fn un_queue(&mut self, p1: i32, count: i32);
    /// Clears bit `0x40` of the word at `+0x60`.
    fn clear_mask_bit(&mut self);
}

fn dec_counter<V: BuildUnqueueView>(v: &mut V, off: usize) {
    let n = v.counter(off);
    if n != 0 {
        v.set_counter(off, n.wrapping_sub(1));
    }
}

/// Runs `Build::unqueue(this, p1, p2)`.
pub fn build_unqueue_steps<V: BuildUnqueueView>(v: &mut V, p1: i32, p2: i32) {
    let count = v.count();
    let skip_redirect = count != 0
        && p1 < i32::from(count)
        && p1 < v.queue_len()
        && v.entry_type(p1) == NO_REDIRECT_TYPE;
    if v.library_gate() && !skip_redirect {
        let lib = v.first_library();
        if lib != i32::from(v.own_library()) && lib >= 0 {
            v.redirect(lib, p1 - i32::from(count), p2);
            return;
        }
    }
    if count == 0 || i32::from(count) - 1 < p1 {
        return;
    }
    v.set_rebuild();
    let mut p1 = p1;
    if p2 != 0 {
        let mut next = p1 + 1;
        while next < i32::from(count) {
            let here = if p1 < v.queue_len() {
                v.entry_type(p1)
            } else {
                -1
            };
            let there = if next < v.queue_len() {
                v.entry_type(next)
            } else {
                -1
            };
            if here != there {
                break;
            }
            next += 1;
            p1 += 1;
        }
    }
    let ty = if p1 < v.queue_len() {
        v.clear_entry(p1);
        i32::from(v.entry_type(p1))
    } else {
        -1
    };
    assert!(ty >= 0, "a negative type faults in the original");
    let t = v.tally(ty);
    if t != 0 {
        v.set_tally(ty, t - 1);
    }
    if v.type_is_unit(ty) && v.unit_word_1e8(ty) != 0 {
        match v.unit_word_40(ty) {
            0x1ab => {
                dec_counter(v, BARRACKS_QUEUED);
                dec_counter(v, COMBAT_QUEUED);
            }
            0x1ac => {
                dec_counter(v, STABLE_QUEUED);
                dec_counter(v, COMBAT_QUEUED);
            }
            0x1ae => dec_counter(v, FACTORY_QUEUED),
            0x1b0 => dec_counter(v, DOCK_QUEUED),
            _ => {
                if v.unit_word_218(ty) == 2 {
                    dec_counter(v, AIR_QUEUED);
                }
            }
        }
    }
    if v.type_is_age(ty) {
        let b = v.byte_counter(AGE_COUNT);
        v.set_byte_counter(AGE_COUNT, b.wrapping_sub(1));
    }
    if v.type_is_epoch(ty) {
        let b = v.byte_counter(EPOCH_COUNT);
        v.set_byte_counter(EPOCH_COUNT, b.wrapping_sub(1));
    }
    if p2 != 0 {
        v.unpay_cost(p1);
    }
    if p1 < i32::from(count) - 1 {
        v.un_queue(p1, i32::from(count));
    }
    let left = count.wrapping_sub(1);
    v.set_count(left);
    if left == 0 {
        v.clear_mask_bit();
    }
}
