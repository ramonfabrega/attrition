//! **`Object::take_damage@00652020`'s steps** (item 3376, parked 3358a): a
//! blow on an object, as a pure walk over a view. The slices of the function
//! are separate entry points because the function is far too long for one
//! emulated call; each runs the head of the one before it.
//!
//! The control flow is the original's, leaf for leaf: every read of a
//! record and every call is one method of the view, asked in the order the
//! listing asks it, which is the order the sweep holds. **The `Random::get`
//! draws are the original's own, in the original's order**: the head spends
//! at most two (the first-wound roll, then the flock's size), and a view
//! that answers them from its own generator spends them where the original
//! does. `this` is the struck object, `who` its owner byte, and the attacker
//! is `objects[p8][p7]` (`p7` negative: none).
//!
//! The nine arguments are the original's own: `p1` the whole hit points,
//! `p2` the sixteenths (a signed byte), `p3` the death class, `p4` the
//! attacker's projectile slot, `p5` the attrition flag, `p6` the angle,
//! `p7` and `p8` the attacker's index and player, `p9` the splash flag.

/// `Object` flags `+8`: the record is live.
pub const OBJ_LIVE: i32 = 0x01;
/// A hit lands on a first wound once in twenty: `roll % 100 < 5`.
pub const FIRST_WOUND_PERCENT: i32 = 5;
/// Type tests the first-wound arm asks of `this`: a fort-like building
/// (the type indices the original pushes at `+0x1b5` and `+0x19f`).
pub const IS_FORTLIKE_A: i32 = 0x1b5;
/// See [`IS_FORTLIKE_A`].
pub const IS_FORTLIKE_B: i32 = 0x19f;
/// The attacker type that never provokes a declaration of war.
pub const IS_NO_WAR: i32 = 0x13b;
/// `ObjectData` flags `+0x1e4`: the type provokes war when it strikes.
pub const DATA_PROVOKES: i32 = 0x0800_0000;
/// The leader's `+0` flags: the player is out (bit 2).
pub const LEADER_OUT: i32 = 0x4;

/// The nine arguments of `Object::take_damage`, `p[0]` the first.
#[derive(Clone, Copy, Debug)]
pub struct TdArgs {
    /// `param_1` ... `param_9`.
    pub p: [i32; 9],
}

impl TdArgs {
    /// `param_2`, the sixteenths: the low byte, signed.
    pub fn sixteenths(&self) -> i32 {
        i32::from(self.p[1] as i8)
    }
    /// `param_3`, the death class: the low byte, signed.
    pub fn dtype(&self) -> i32 {
        i32::from(self.p[2] as i8)
    }
}

/// What the head of `take_damage` asks of the world: everything up to the
/// accumulate, whose first virtual call (slot `+0x11c`, the figure's share)
/// ends the slice.
pub trait TakeDamageHead {
    /// `[this + 8]`'s low byte.
    fn flags8(&self) -> i32;
    /// The two `GameLog::say` lines the function opens with: `which` is 1
    /// for the line of `p1..p4` and 2 for the line of `p5..p8`.
    fn say(&mut self, which: i32, a: i32, b: i32, c: i32, d: i32);
    /// `Game`'s semaphore byte, bit 2 (`+0x820 & 4`).
    fn sem4(&self) -> bool;
    /// `Game`'s difficulty byte (`+0x2b`), unsigned.
    fn difficulty(&self) -> i32;
    /// `Game::frame` (`+0x550`).
    fn game_frame(&self) -> i32;
    /// `[this + 9]`: the owner.
    fn who(&self) -> i32;
    /// The leader's `frame_attacked` (`+0xa40`).
    fn set_frame_attacked(&mut self, who: i32, frame: i32);
    /// `[this + 0x24]`: the damage taken so far.
    fn damage(&self) -> i32;
    /// `this`'s slot `+0x1c`: a building.
    fn is_build(&mut self) -> bool;
    /// `Random::get(game_random, 0, 0xffff)`.
    fn roll(&mut self) -> i32;
    /// `this`'s data's slot `+0xfc`.
    fn data_fc(&mut self) -> bool;
    /// `this->is(ty, 0)`: `this`'s own slot `+0xb8`, or on the base arm the
    /// data's slot `+0x60`.
    fn this_is(&mut self, ty: i32) -> bool;
    /// The data of `objects[p8][p7]`, slot `+0x10c`.
    fn attacker_siege(&mut self, p8: i32, p7: i32) -> bool;
    /// `Objects::add_flock(x, y, -1, n)` over the tile `this` stands on.
    fn flock(&mut self, n: i32);
    /// `objects[p8][p7]`'s data `+0x1e4 & 0x8000000`.
    fn attacker_provokes(&self, p8: i32, p7: i32) -> bool;
    /// `units[p8][p7]->is(ty, 0)`.
    fn attacker_is(&mut self, p8: i32, p7: i32, ty: i32) -> bool;
    /// `Game::war_allowed`.
    fn war_allowed(&mut self) -> bool;
    /// The leader `who`'s word at `+0`.
    fn leader_flags(&self, who: i32) -> i32;
    /// The world cell under `this` (`div_3_table[pos >> 8]`): its owner
    /// byte `+0xf`, signed.
    fn cell_owner(&self) -> i32;
    /// `LeaderData::is_enemy(leader who, p8)`.
    fn is_enemy(&mut self, who: i32, p8: i32) -> bool;
    /// `Leader::action_declare(who, p8, 0, flag, 0)`.
    fn declare(&mut self, who: i32, p8: i32, flag: i32);
    /// `[this + 0x3b]`: the sixteenths carried.
    fn carry(&self) -> i32;
    /// Sets it.
    fn set_carry(&mut self, v: i32);
}

/// What the head leaves for the accumulate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeadOut {
    /// A dead record: the function returned 0 at once.
    Return(i32),
    /// On to slot `+0x11c`; the whole hits (`p1` plus the sixteenths' carry)
    /// and the sixteenths' byte after the floor.
    Accumulate {
        /// `param_1` after the carry.
        whole: i32,
        /// `param_2` after the floor at one sixteenth.
        sixteenths: i32,
    },
}

/// The head of `Object::take_damage`: the notice before the accumulate and
/// the accumulate's own arithmetic, up to the figure's share.
pub fn take_damage_head<V: TakeDamageHead>(v: &mut V, a: &TdArgs) -> HeadOut {
    let [p1, _, _, _, p5, _, p7, p8, _] = a.p;
    if v.flags8() & OBJ_LIVE == 0 {
        return HeadOut::Return(0);
    }
    v.say(1, p1, a.sixteenths(), a.dtype(), a.p[3]);
    v.say(2, p5, a.p[5], p7, p8);
    let mut p2 = a.sixteenths();
    // `param_1 < 1 && param_2 < 1`: a hit of nothing is one sixteenth.
    if p1 < 1 && p2 < 1 {
        p2 = 1;
    }
    if p5 == 0 {
        notice(v, p7, p8);
    }
    // The accumulate: the carried sixteenths plus this blow's, a signed
    // byte, divided by sixteen toward zero for the whole points and kept
    // modulo sixteen (a negative remainder borrows).
    let t = i32::from((v.carry() as i8).wrapping_add(p2 as i8));
    let whole = p1.wrapping_add((t + ((t >> 31) & 0xf)) >> 4);
    let mut rem = t & i32::MIN.wrapping_add(0xf);
    if rem < 0 {
        rem = (rem.wrapping_sub(1) | -16).wrapping_add(1);
    }
    v.set_carry(rem);
    HeadOut::Accumulate {
        whole,
        sixteenths: p2,
    }
}

/// The `p5 == 0` arm: the owner's `frame_attacked` stamp, the first-wound
/// flock and the declaration of war on a provoker. The two rolls are the
/// original's only draws here, the first-wound roll then the flock's size.
fn notice<V: TakeDamageHead>(v: &mut V, p7: i32, p8: i32) {
    if !v.sem4() && v.difficulty() < 2 {
        let who = v.who();
        let frame = v.game_frame();
        v.set_frame_attacked(who, frame);
    }
    // `goto accumulate` when there is no attacker to provoke anything.
    let mut war = true;
    if v.damage() == 0 && v.is_build() && v.roll() % 100 < FIRST_WOUND_PERCENT {
        let fort = v.data_fc() || v.this_is(IS_FORTLIKE_A) || v.this_is(IS_FORTLIKE_B);
        if fort {
            if p7 < 0 {
                war = false;
            } else if v.attacker_siege(p8, p7) {
                let r = v.roll();
                // `r % 2` as the original's signed remainder leaves it.
                let n = (r % 2) + 3;
                v.flock(n);
            }
        }
    }
    if !war || p7 < 0 || !v.attacker_provokes(p8, p7) || v.attacker_is(p8, p7, IS_NO_WAR) {
        return;
    }
    if !v.war_allowed() {
        return;
    }
    let who = v.who();
    if who == p8 {
        return;
    }
    let flags = v.leader_flags(who);
    if flags & LEADER_OUT != 0 || v.cell_owner() != who || v.is_enemy(who, p8) {
        return;
    }
    v.declare(who, p8, ((flags as u32 >> 2) ^ 1) as i32 & 1);
}
