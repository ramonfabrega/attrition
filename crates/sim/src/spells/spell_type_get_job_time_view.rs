//! `SpellTypeData::get_job_time@00675800` (item 3395), the whole function over a view.
//!
//! The craft's own `JOB_TIME` (`this+8`) is adjusted by its type (`this+4`, the spell index) under seven
//! arms, each by the caster's leader and its objects and units:
//!
//! - `0x27d` Entrench: the French tribe bonus (`10`) scales by `french_entrench` percent; then, for a
//!   caster named (`unit >= 0`) whose objects row answers its slot `+0x8`, Antipater's rate in 8.8
//!   (`antipater_entrench_rate`), after the units row is asked `is(0x169, 0)` (the answer is dropped).
//! - `0x275` Bribe and `0x27f` Informer: halved under `has_preq(0x304)` (`SPIES_CRAFT_FASTER`).
//! - `0x28b`/`0x28c`, the siege pair: the Turkish bonus (`8`) scales by `turk_pack` percent; one leader
//!   whose short at `+0x59cc` is set, and whose objects row answers slot `+0x18`, and whose units row
//!   `has_general(0, 0x167)` is not negative, and whose units row's type answers slot `+0x10c`, scales by
//!   `napoleon_pack` percent; then the objects row `is(0x10f, 1)` or `is(0x116, 1)` halves, else
//!   `is(0x111, 1)` quarters.
//! - `0x28d`/`0x28e`, the machine gun's: the objects row `is(0x7d, 0)` halves.
//! - `0x280`/`0x281`, Sabotage and Sniper: the tribe bonus `9` halves; a named caster whose units row
//!   `is(0x177, 1)` takes 10.
//!
//! Every arm that names a leader (`who`) is guarded by `who >= 0` *except* the siege pair's leader short
//! and every objects-row read of the siege and machine-gun arms: those index the tables with a negative
//! `who` or `unit` unchecked, and the sweeps keep them non-negative.
//!
//! **Not wired.** The crate's [`crate::Sim::cast_job_time`] holds the siege pair's and the machine gun's
//! type arms (`is`, strict) and nothing else: the crate carries none of `french_entrench`,
//! `antipater_entrench_rate`, `turk_pack` or `napoleon_pack`, nor `SPIES_CRAFT_FASTER`'s preq, nor the
//! leader short at `+0x59cc` (`docs/ORDERS.md` §6.9 names the remaining arms as the SEAM of
//! [`crate::Sim::spell_job_time`]). This is the function whole, so the arms the crate lacks are in one
//! place when they are loaded; the type arms there and here are one rule, and the sweep is the original's
//! check on this one.
//!
//! Arithmetic is the original's: a percentage is `(constant * time) / 100` with the product wrapped to 32
//! bits and the quotient truncated toward zero; the 8.8 rate is `(rate * time) / 256` likewise.

/// What `get_job_time` reads and asks.
pub trait JobTimeView {
    /// `LeaderData::has_tribe_bonus(leaders[who], k)`.
    fn has_tribe_bonus(&mut self, who: i32, k: i32) -> bool;
    /// `LeaderData::has_preq(leaders[who], t)`.
    fn has_preq(&mut self, who: i32, t: i32) -> bool;
    /// `constants->french_entrench` (`+0x6b8`), percent.
    fn french_entrench(&mut self) -> i32;
    /// `constants->antipater_entrench_rate` (`+0xb9c`), 8.8.
    fn antipater_entrench_rate(&mut self) -> i32;
    /// `constants->turk_pack` (`+0x670`), percent.
    fn turk_pack(&mut self) -> i32;
    /// `constants->napoleon_pack` (`+0xbac`), percent.
    fn napoleon_pack(&mut self) -> i32;
    /// The short at `leaders[who] + 0x59cc`.
    fn leader_short_59cc(&mut self, who: i32) -> i16;
    /// `objects[who][unit]`'s vtable slot `+0x8`.
    fn objects_slot_08(&mut self, who: i32, unit: i32) -> bool;
    /// `objects[who][unit]`'s vtable slot `+0x18`.
    fn objects_slot_18(&mut self, who: i32, unit: i32) -> bool;
    /// `objects[who][unit]->is(ty, strict)`.
    fn objects_is(&mut self, who: i32, unit: i32, ty: i32, strict: i32) -> bool;
    /// `units[who][unit]->is(ty, strict)`.
    fn units_is(&mut self, who: i32, unit: i32, ty: i32, strict: i32) -> bool;
    /// `ObjectData::has_general(units[who][unit], 0, 0x167)`.
    fn units_has_general(&mut self, who: i32, unit: i32) -> i32;
    /// `units[who][unit]`'s type's vtable slot `+0x10c`.
    fn units_type_gate(&mut self, who: i32, unit: i32) -> bool;
}

/// A percentage: the product wrapped to 32 bits, the quotient toward zero.
fn percent(rate: i32, time: i32) -> i32 {
    rate.wrapping_mul(time) / 100
}

/// `SpellTypeData::get_job_time(this, unit, who)@00675800` for the spell index `ty` and `JOB_TIME`
/// `time`.
pub fn get_job_time<V: JobTimeView>(v: &mut V, ty: i32, time: i32, unit: i32, who: i32) -> i32 {
    let mut t = time;
    if ty == 0x27d && who >= 0 {
        if v.has_tribe_bonus(who, 10) {
            t = percent(v.french_entrench(), t);
        }
        if unit >= 0 && v.objects_slot_08(who, unit) {
            // The answer is dropped: the call is the original's, not a condition.
            v.units_is(who, unit, 0x169, 0);
            t = v.antipater_entrench_rate().wrapping_mul(t) / 256;
        }
    }
    if (ty == 0x275 || ty == 0x27f) && who >= 0 && v.has_preq(who, 0x304) {
        t /= 2;
    }
    if ty == 0x28b || ty == 0x28c {
        if who >= 0 && v.has_tribe_bonus(who, 8) {
            t = percent(v.turk_pack(), t);
        }
        if v.leader_short_59cc(who) != 0
            && v.objects_slot_18(who, unit)
            && v.units_has_general(who, unit) >= 0
            && v.units_type_gate(who, unit)
        {
            t = percent(v.napoleon_pack(), t);
        }
        if v.objects_is(who, unit, 0x10f, 1) || v.objects_is(who, unit, 0x116, 1) {
            t /= 2;
        } else if v.objects_is(who, unit, 0x111, 1) {
            t /= 4;
        }
    }
    if (ty == 0x28d || ty == 0x28e) && v.objects_is(who, unit, 0x7d, 0) {
        t /= 2;
    }
    if (ty == 0x281 || ty == 0x280) && who >= 0 {
        if v.has_tribe_bonus(who, 9) {
            t /= 2;
        }
        if unit >= 0 && v.units_is(who, unit, 0x177, 1) {
            t = 10;
        }
    }
    t
}
