//! **`Unit::action_unqueue@005e1f20`** (item 3391): a unit's own production
//! queue gives one back, as a pure walk over a [`UnitUnqueueView`].
//!
//! The crate has no port of this. `Sim::action_unqueue` (`production.rs`) is
//! `Build::action_unqueue@00620280`, the *building's* cancel; the unit arm of
//! `CommandPackage::process_unqueue` is left open in `rondata::input::unqueue`
//! (the unit arm is not modelled). This view is therefore **standalone**
//! (nothing in `Sim` calls it) and the sweep is its only caller; whether
//! wiring it moves a diff pin is not established.
//!
//! What it does, in this order:
//!
//! 1. a unit whose queue count (short `+0xa0`) is zero returns; otherwise the
//!    count falls by one, and reaching zero clears the word at `+0x64`;
//! 2. the leader's current `FIGHTERBOMBER` upgrade (`LeaderData::
//!    current_upgrade`, argument `0x134`) names a type index; when it is
//!    non-negative the leader's per-type queued tally (short table at leader
//!    `+0x5a22`) falls by one if it is not zero;
//! 3. when that type is a unit type (`0x32 <= type < 0x19e`, either the
//!    inlined test or the type's slot `0xc`) and its unit record's word
//!    `+0x1e8` is set, the AI counters move, each guarded against zero by its
//!    own test: record word `+0x40` of `0x1ab` takes `+0xa10` then `+0xa1c`,
//!    `0x1ac` takes `+0xa14` then `+0xa1c`, `0x1ae` takes `+0xa18`, `0x1b0`
//!    takes `+0xa20`, any other takes `+0xa24` when the record's word `+0x218`
//!    is 2;
//! 4. a non-zero argument finally calls the type's slot `0xd8` with
//!    `(owner, -1, -1)`.
//!
//! **Not established**: step 4 runs even when the upgrade is negative, where
//! the original indexes the type list at `-1` and calls through whatever
//! word precedes it. The sweep cannot say what that does (the emulator faults
//! at that word), so it draws that combination on neither side; the
//! callers' contract is that a negative upgrade comes with a zero argument.

/// `LeaderData::current_upgrade`'s second argument.
pub const FIGHTERBOMBER: i32 = 0x134;
/// Leader offsets of the six AI counters.
pub const BARRACKS_QUEUED: usize = 0xa10;
pub const STABLE_QUEUED: usize = 0xa14;
pub const FACTORY_QUEUED: usize = 0xa18;
pub const DOCK_QUEUED: usize = 0xa20;
pub const AIR_QUEUED: usize = 0xa24;
/// The combat-queued counter both the barracks and the stable arm move.
pub const COMBAT_QUEUED: usize = 0xa1c;

/// The state `Unit::action_unqueue` reads and the calls it makes.
pub trait UnitUnqueueView {
    /// The unit's queue count, short `+0xa0`.
    fn queued(&mut self) -> i16;
    /// Writes the queue count.
    fn set_queued(&mut self, n: i16);
    /// The word at `+0x64`, cleared when the queue empties.
    fn clear_word_64(&mut self);
    /// `LeaderData::current_upgrade(leader(owner), FIGHTERBOMBER)`.
    fn current_upgrade(&mut self) -> i32;
    /// The leader's per-type tally of type `idx`.
    fn tally(&mut self, idx: i32) -> u16;
    /// Writes it.
    fn set_tally(&mut self, idx: i32, v: u16);
    /// Whether type `idx` is a unit type: the type's slot `0xc`, which is
    /// `TypeData::is_unit_type` (the range test) or an override.
    fn type_is_unit(&mut self, idx: i32) -> bool;
    /// The unit record's word `+0x1e8`.
    fn unit_word_1e8(&mut self, idx: i32) -> i32;
    /// The unit record's word `+0x40`.
    fn unit_word_40(&mut self, idx: i32) -> i32;
    /// The unit record's word `+0x218`.
    fn unit_word_218(&mut self, idx: i32) -> i32;
    /// A leader counter word at `off`.
    fn counter(&mut self, off: usize) -> i32;
    /// Writes it.
    fn set_counter(&mut self, off: usize, v: i32);
    /// The type's slot `0xd8`, handed `(owner, -1, -1)`.
    fn type_slot_d8(&mut self, idx: i32);
}

fn dec_counter<V: UnitUnqueueView>(v: &mut V, off: usize) {
    let n = v.counter(off);
    if n != 0 {
        v.set_counter(off, n.wrapping_sub(1));
    }
}

/// Runs `Unit::action_unqueue(this, p)`.
pub fn unit_unqueue_steps<V: UnitUnqueueView>(v: &mut V, p: i32) {
    let n = v.queued();
    if n == 0 {
        return;
    }
    let n = n.wrapping_sub(1);
    v.set_queued(n);
    if n == 0 {
        v.clear_word_64();
    }
    let idx = v.current_upgrade();
    if idx >= 0 {
        let t = v.tally(idx);
        if t != 0 {
            v.set_tally(idx, t - 1);
        }
        if v.type_is_unit(idx) && v.unit_word_1e8(idx) != 0 {
            match v.unit_word_40(idx) {
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
                    if v.unit_word_218(idx) == 2 {
                        dec_counter(v, AIR_QUEUED);
                    }
                }
            }
        }
    }
    if p != 0 {
        v.type_slot_d8(idx);
    }
}
