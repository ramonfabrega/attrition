//! **`Group::action_spell@006fe1a0`'s default arm** (item 3386): a group's
//! spell press for a craft that is neither a unit type's nor `TRANSPORT`, as
//! a pure walk over a [`SpellView`]. The swept steps read the world only
//! through the view, so the sweep's port is a view over the emulated
//! original's windows (`docs/ORDERS.md` §37 is the arm's reading).
//!
//! The control flow is the original's, call for call: every leaf is one
//! virtual call, one direct callee or one inline read, asked in the order the
//! listing asks it, which is what the sweep holds. `p` is a player and `o`/`u`
//! an index in that player's `objects`/`units` array (the original indexes
//! both by one index); `ty` is the craft (`param_1`), `(ox, whom)` the target
//! and `(x, y)` the point. The head is the garrison order's
//! ([`crate::garrison_head`]), instruction for instruction.
//!
//! Three things run before the walk: `GroupOut::validate_spell` (a refusal
//! ends the call with the group's formation untouched), the formation reset,
//! and — for a craft carrying `m` (0x1000) — the pick of the one member with
//! the most `mana_left` (the arm [`crate::cast`]'s `group_action_spell`
//! already models as `BEST_MANA`).
//!
//! **Not wired.** The crate's `Sim::group_action_spell` is the same arm over
//! `Sim`'s own state, with `validate_spell`, `can_pay_cost`, the pack and
//! unpack members' tests, a building caster's queue and the feedback left as
//! SEAMs, and `group.rs`'s `FinishInsertView::action_spell` is an empty stub.
//! The view carries what that method leaves out (the mana the order's head
//! cast has pending, the `is_castable == 3` pick, the busy and cost counters,
//! the epilogue's feedback), so wiring it would add refusals and sounds to
//! every capture whose human presses a spell: parked, not done here.
//!
//! **Not swept** (a flag the generator lacks, in the sweep's head): the
//! aircraft arm (a unit type whose `unittypes[ty] + 0x218` is 2) and the
//! `TRANSPORT` arm, which the view refuses with a panic.

pub use crate::garrison_view::{GarrisonHead, garrison_head};

/// `TRANSPORT`'s type index: the arm the view does not model.
pub const TYPE_TRANSPORT: i32 = 0x28a;
/// The craft index a non-spell type falls back to (`0x296`).
pub const TYPE_FALLBACK: i32 = 0x296;
/// The two crafts whose `is_castable` is not asked (`0x29a`, `0x286`).
pub const NO_CASTABLE_CHECK: [i32; 2] = [0x29a, 0x286];
/// `UnitData::order_type`'s answer for a cast (`CAST_SPELL`).
pub const ORDER_CAST: i32 = 0xe;
/// The craft row's `m` flag: the cast goes to the member with the most mana.
pub const FLAG_BEST_MANA: i32 = 0x1000;
/// The craft row's `g` flag: the head casts are killed and the cast goes first.
pub const FLAG_FIRST: i32 = 0x40;
/// The unit's flag the busy test reads (`+0x68`).
pub const UNIT_BUSY_FLAG: i32 = 0x80000;

/// What `Group::action_spell` asks of the group, the world and the objects.
pub trait SpellView: GarrisonHead {
    /// `GroupOut::validate_spell(this, ty, ox, whom, x, y)`: nonzero refuses.
    fn validate_spell(&mut self, a: [i32; 5]) -> bool;
    /// `this + 0x10 = -1`: the group's formation is forgotten.
    fn set_form(&mut self);
    /// `[this + 0xc]`.
    fn count(&self) -> i32;
    /// The short `i` of the group's array at `+0x8cc`.
    fn member(&self, i: i32) -> i32;

    /// `types[ty]`'s `is_spell_type`: slot `+0x48`, or the range test it is
    /// inlined to (`0x275 <= type_index < 0x2ac`) when the slot is the base.
    fn type_is_spell(&mut self, ty: i32) -> bool;
    /// `types[ty]`'s slot `+0xc` (`is_unit_type`).
    fn type_is_unit(&mut self, ty: i32) -> bool;
    /// `unittypes[ty] + 0x218 == 2`: the aircraft arm.
    fn unit_type_is_aircraft(&mut self, ty: i32) -> bool;
    /// `leader_options[p] + 0x1c & 1`: the leader turned the pick off.
    fn leader_no_best(&self, p: i32) -> bool;
    /// `objects[p][o] + 8 & 1`.
    fn obj_flag1(&self, p: i32, o: i32) -> bool;
    /// `objects[p][o]`'s slot `+0x8`.
    fn obj_alive(&mut self, p: i32, o: i32) -> bool;
    /// `objects[p][o]`'s slot `+0xbc`.
    fn obj_bc(&mut self, p: i32, o: i32) -> bool;
    /// `objects[p][o]`'s slot `+0x18`.
    fn obj_s18(&mut self, p: i32, o: i32) -> bool;
    /// `objects[p][o]`'s slot `+0x20`: the building casts in place.
    fn obj_s20(&mut self, p: i32, o: i32) -> bool;
    /// `objects[p][o]`'s slot `+0xc`.
    fn obj_s0c(&mut self, p: i32, o: i32) -> bool;
    /// `objects[p][o]`'s slot `+0xac`, then `Build::queue_up(ty, 1)` on it.
    fn obj_queue_up(&mut self, p: i32, o: i32, ty: i32);
    /// `spelltypes[t] + 0x1c8`.
    fn spell_flags(&self, t: i32) -> i32;
    /// `spelltypes[t] + 0x1d0`: the craft's `MANA`.
    fn spell_mana(&self, t: i32) -> i32;
    /// `UnitData::is_casting(ty)` on `units[p][u]`.
    fn unit_is_casting(&mut self, p: i32, u: i32, ty: i32) -> bool;
    /// `SpellTypeData::is_castable(spelltypes[t], u, p, 0)`.
    fn is_castable(&mut self, t: i32, u: i32, p: i32) -> i32;
    /// `UnitData::mana_left`.
    fn mana_left(&mut self, p: i32, u: i32) -> i32;
    /// `UnitData::mana`.
    fn mana(&mut self, p: i32, u: i32) -> i32;
    /// `units[p][u] + 0x96` (the short, signed): `mana_burn`.
    fn mana_burn(&self, p: i32, u: i32) -> i32;
    /// `units[p][u] + 0x68`.
    fn unit_flags(&self, p: i32, u: i32) -> i32;
    /// The head cast's craft the unit's current action carries, when
    /// `get_action` names an order whose slot `+0x10` is [`ORDER_CAST`]: the
    /// craft's `MANA`, else 0. Asks `get_action`, the order's slot `+0x10`,
    /// `get_action` again and the slot `+0xf4`, in that order.
    fn pending_mana(&mut self, p: i32, u: i32) -> i32;
    /// `types[ty]`'s slot `+0x84(who, -1, u, 1)`.
    fn type_s84(&mut self, ty: i32, who: i32, u: i32) -> bool;
    /// `SpellTypeData::is_valid_target(spelltypes[t], who, ox, whom, 0)`.
    fn is_valid_target(&mut self, t: i32, who: i32, ox: i32, whom: i32) -> bool;
    /// `types[ty]`'s slot `+0x50`.
    fn type_s50(&mut self, ty: i32) -> bool;
    /// `types[ty]`'s slot `+0x54`.
    fn type_s54(&mut self, ty: i32) -> bool;
    /// `SpellTypeData::get_job_time(spelltypes[ty], u, p)`.
    fn get_job_time(&mut self, ty: i32, u: i32, p: i32) -> bool;
    /// `SpellType::pay_cast_costs(spelltypes[ty], u, p)`, then `cast(u, p, 0, 0)`.
    fn pay_and_cast(&mut self, ty: i32, u: i32, p: i32);
    /// `UnitData::is_busy`.
    fn unit_is_busy(&mut self, p: i32, u: i32) -> bool;
    /// `units[p][u]`'s target: `+0x98 = 0`, `+0xa2 = ox`, `+0xa8 = whom`, and
    /// `+0xa6` = the target object's `+0x30` when `ox >= 0`; only when
    /// `(ox, whom)` differs from what the unit holds (`+0xa2` as a signed
    /// short, `+0xa8` as a signed byte).
    fn write_target(&mut self, p: i32, u: i32, ox: i32, whom: i32);
    /// `UnitData::order_type`.
    fn order_type(&mut self, p: i32, u: i32) -> i32;
    /// `Unit::kill_current_order(0)`.
    fn kill_current_order(&mut self, p: i32, u: i32);
    /// `Unit::add_cast_order(ox, whom, x, y, ty, queue, 1)`, queue in the
    /// original's numbering (`QUEUE_FIRST` 0, `QUEUE_NEW` 2).
    fn add_cast_order(&mut self, p: i32, u: i32, args: [i32; 5], queue: i32);

    /// `MiscAccess::console->who`.
    fn console_who(&self) -> i32;
    /// `LeaderOut::warn_resources(leaders[who], ty, last, -1, who)`.
    fn warn_resources(&mut self, who: i32, ty: i32, last: i32);
    /// `SoundGlobal::play(who, sound)`.
    fn play(&mut self, who: i32, sound: i32);
    /// The red feedback line at the string table's offset `msg`
    /// (`String::String`, then `MessageWin::add_feedback`).
    fn feedback(&mut self, msg: i32);
}

/// The string table's offset of the target refusal.
pub const MSG_BAD_TARGET: i32 = 0x85d4;
/// The not-enough-mana refusal.
pub const MSG_NO_MANA: i32 = 0x85e8;
/// The busy refusal of a type whose slot `+0x50` answers.
pub const MSG_BUSY_50: i32 = 0x85fc;
/// The busy refusal of a type whose slot `+0x54` answers.
pub const MSG_BUSY_54: i32 = 0x8610;
/// `S_NOT_ENOUGH_CRAFT`.
pub const SOUND_NOT_ENOUGH: i32 = 0x4d;
/// `S_INVALID_ORDER`.
pub const SOUND_INVALID: i32 = 0x40;

/// `Group::action_spell(ty, ox, whom, x, y)`, the default arm.
pub fn action_spell_steps<V: SpellView>(v: &mut V, ty: i32, ox: i32, whom: i32, x: i32, y: i32) {
    garrison_head(v);
    if v.validate_spell([ty, ox, whom, x, y]) {
        return;
    }
    v.set_form();
    if ty < 0 {
        return;
    }
    let p = v.who() & 0xff;
    // The `m` craft's caster: the most `mana_left` among the members that
    // may cast it (the head's lineage test) and are not already casting it.
    let mut best_idx = -1;
    let mut best_mana = 0;
    if v.type_is_spell(ty) && !v.leader_no_best(p) {
        let mut i = 0;
        while i < v.count() {
            let u = v.member(i);
            if v.obj_alive(p, u)
                && v.obj_bc(p, u)
                && v.spell_flags(ty) & FLAG_BEST_MANA != 0
                && !v.unit_is_casting(p, u, ty)
                && v.is_castable(ty, u, p) == 3
                && best_mana < v.mana_left(p, u)
            {
                best_mana = v.mana_left(p, u);
                best_idx = i;
            }
            i += 1;
        }
    }
    if v.type_is_unit(ty) && v.unit_type_is_aircraft(ty) {
        unreachable!("the aircraft arm is not swept");
    }
    assert!(ty != TYPE_TRANSPORT, "the TRANSPORT arm is not swept");
    let t = if v.type_is_spell(ty) { ty } else { TYPE_FALLBACK };
    // The counters the epilogue reads.
    let mut fail = 0;
    let mut laid = 0;
    let mut busy = 0;
    let mut cost = 0;
    let mut short = 0;
    let mut target = 0;
    let mut last = -1;
    let mut i = 0;
    while i < v.count() {
        let m = v.member(i);
        let this_i = i;
        i += 1;
        if !v.obj_flag1(p, m) || !v.obj_bc(p, m) {
            continue;
        }
        if !NO_CASTABLE_CHECK.contains(&t) && v.is_castable(t, m, p) == 0 {
            continue;
        }
        if best_idx >= 0 && best_idx != this_i {
            continue;
        }
        if v.obj_s18(p, m) {
            let pend = v.pending_mana(p, m);
            let mut have = v.mana(p, m).wrapping_sub(v.mana_burn(p, m));
            if have < 0 {
                have = 0;
            }
            have = have.wrapping_add(pend);
            if have >= v.mana(p, m) {
                have = v.mana(p, m);
            }
            let need = v.spell_mana(ty);
            if need != 0 && need > have {
                fail += 1;
                short += 1;
                last = m;
                continue;
            }
        }
        if !v.type_s84(ty, p, m) {
            fail += 1;
            cost += 1;
            last = m;
            continue;
        }
        if !v.is_valid_target(t, p, ox, whom) {
            fail += 1;
            target += 1;
            continue;
        }
        if v.obj_s18(p, m) {
            let flagged = v.unit_flags(p, m) & UNIT_BUSY_FLAG != 0;
            if v.type_s50(ty) {
                if flagged {
                    fail += 1;
                    busy += 1;
                }
            } else if v.type_s54(ty) && !flagged {
                fail += 1;
                busy += 1;
            }
        }
        if v.unit_is_casting(p, m, ty) {
            fail += 1;
            busy += 1;
        }
        if v.obj_s20(p, m) {
            laid += 1;
            if v.obj_s0c(p, m) && v.get_job_time(ty, m, p) {
                v.obj_queue_up(p, m, ty);
            } else {
                v.pay_and_cast(ty, m, p);
            }
            continue;
        }
        if v.unit_is_busy(p, m) {
            continue;
        }
        laid += 1;
        v.write_target(p, m, ox, whom);
        let queue = if v.spell_flags(t) & FLAG_FIRST != 0 {
            while v.order_type(p, m) == ORDER_CAST {
                v.kill_current_order(p, m);
            }
            0
        } else {
            2
        };
        v.add_cast_order(p, m, [ox, whom, x, y, ty], queue);
    }
    // The epilogue: a refusal is told to the console's player only, and only
    // when no member took the order.
    if fail == 0 || laid != 0 || p != v.console_who() {
        return;
    }
    if target != 0 {
        v.feedback(MSG_BAD_TARGET);
    } else if cost != 0 {
        v.warn_resources(p, ty, last);
    } else if short != 0 {
        v.play(p, SOUND_NOT_ENOUGH);
        v.feedback(MSG_NO_MANA);
    } else if busy != 0 {
        v.play(p, SOUND_INVALID);
        if v.type_s50(ty) {
            v.feedback(MSG_BUSY_50);
        } else if v.type_s54(ty) {
            v.feedback(MSG_BUSY_54);
        }
    }
}
