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

/// `ObjectData` type test the building arm asks of the attacker.
pub const IS_ATTACKER_WALL: i32 = 0x42;
/// The city's flags `+4`: the alarm bell is rung.
pub const CITY_BELL: i32 = 0x40;
/// What a hit marks on the city it strikes: under attack, attacking, ever
/// attacked.
pub const CITY_STRUCK: i32 = 0xe;
/// What a hit marks on the victim's data when it comes from beyond the
/// walls: the data's `+0x60` flags.
pub const DATA_STRUCK: i32 = 0x30;
/// `Object` flags `+8`: a city building.
pub const OBJ_CITY_BUILDING: i32 = 0x20;
/// `Object` flags `+8`: set where a city building is held at its share.
pub const OBJ_HELD: i32 = 0x10;
/// A site's progress lost per hit point.
pub const PROGRESS_PER_HIT: u32 = 0x32;
/// `LeaderData::get_diff` at or above which a computer player raises its
/// alarms.
pub const ALARM_DIFFICULTY: i32 = 2;

/// What the accumulate, the building's walls and the peasants' alarm ask of
/// the world, after the head: everything up to the call of slot `+0x18`
/// that opens the kill arm.
pub trait TakeDamageArms: TakeDamageHead {
    /// `this`'s slot `+0x11c(0)`: what the figure can absorb.
    fn share(&mut self) -> i32;
    /// Sets `[this + 0x24]`.
    fn set_damage(&mut self, v: i32);
    /// Sets `[this + 8]`.
    fn set_flags8(&mut self, v: i32);
    /// `this`'s slot `+0x4c`: finished.
    fn this_finished(&mut self) -> bool;
    /// The attacker `objects[p8][p7]` is live and its data's domain
    /// (`+0x218`) is the air.
    fn attacker_air(&self, p8: i32, p7: i32) -> bool;
    /// `objects[who][idx]`'s slot `+0xb0`, then its `+0x48`: a site's
    /// progress.
    fn progress(&mut self) -> u32;
    /// The same slot again, then the store.
    fn set_progress(&mut self, v: u32);
    /// `this`'s slot `+0x20`: a building of a city.
    fn this_city_building(&mut self) -> bool;
    /// `ObjectData::is_peasant` of the attacker `objects[p8][p7]`.
    fn attacker_peasant(&mut self, p8: i32, p7: i32) -> bool;
    /// `objects[p8][p7]->is(ty, 0)`.
    fn attacker_obj_is(&mut self, p8: i32, p7: i32, ty: i32) -> bool;
    /// `ObjectData::is_peasant` of `this`.
    fn this_peasant(&mut self) -> bool;
    /// `objects[who][idx]`'s slot `+0xac`, then its data's `+0x60` flags
    /// `|= 0x30`.
    fn mark_struck(&mut self);
    /// Slot `+0xac`, then the data's short `+0x72`: its city.
    fn data_city(&mut self) -> i32;
    /// Slot `+0xac`, then slot `+0x90` of its type.
    fn data_type_slot90(&mut self) -> bool;
    /// Slot `+0xac`, then the data's flags `+8`.
    fn data_flags8(&mut self) -> i32;
    /// `cities[who][c]`'s flags `+4`.
    fn city_flags(&self, c: i32) -> i32;
    /// Sets them.
    fn set_city_flags(&mut self, c: i32, v: i32);
    /// `cities[who][c]`'s short `+8`: its building.
    fn city_building(&self, c: i32) -> i32;
    /// Its signed byte `+0x5e`: the building's player.
    fn city_player(&self, c: i32) -> i32;
    /// The leader's word at `+4`.
    fn leader_word4(&self, who: i32) -> i32;
    /// `LeaderData::get_diff`.
    fn leader_diff(&mut self, who: i32) -> i32;
    /// `ObjectsData::find_city_at(x, y, who, -1, 0)` over `this`'s position.
    fn find_city_at(&mut self, who: i32) -> i32;
    /// The city's building `objects[player][idx]`, finished: its data is a
    /// `Build` and flag `4` is set, else the data's slot `+0x4c`.
    fn city_building_finished(&mut self, player: i32, idx: i32) -> bool;
    /// The stack group's `Group::clear(-1)`.
    fn group_clear(&mut self);
    /// `Group::add(idx, player, 0, 0)`.
    fn group_add(&mut self, idx: i32, player: i32);
    /// `Groups::push_group(who, group, 1)`.
    fn push_group(&mut self, who: i32) -> i32;
    /// `Group::action_alarm` of the group `g` pushed.
    fn group_alarm(&mut self, g: i32);
    /// `ObjectData::can_carry(this, AIR)`.
    fn can_carry_air(&mut self) -> bool;
    /// `Object::eject_contents(1, -1, 0, 1)`.
    fn eject(&mut self);
}

/// Where the arms leave the function.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArmsOut {
    /// The function returned.
    Return(i32),
    /// On to the kill arm; the figure's share, read once more at the join.
    Join {
        /// `local_c`: the share before the squad's division.
        share: i32,
    },
}

/// The accumulate and what follows it up to the kill arm: the damage taken,
/// the site's lost progress, the walls' flags and the alarm a city raises.
pub fn take_damage_arms<V: TakeDamageArms>(v: &mut V, a: &TdArgs, whole: i32) -> ArmsOut {
    let [_, _, _, _, p5, _, p7, p8, _] = a.p;
    // The share is asked and the answer dropped, as the original does.
    v.share();
    let damage = v.damage().wrapping_add(whole);
    v.set_damage(damage);
    if v.is_build() && p5 == 0 {
        let air = p7 >= 0 && p8 >= 0 && v.attacker_air(p8, p7);
        if !v.this_finished() && whole > 0 && !air {
            let cost = (whole as u32).wrapping_mul(PROGRESS_PER_HIT);
            let cur = v.progress();
            v.set_progress(if cost < cur { cur - cost } else { 0 });
        }
    }
    let who = v.who();
    if v.this_city_building() {
        if p5 == 0 && who != p8 {
            wall_notice(v, who, p7, p8);
        }
        let share = v.share();
        if v.damage() >= share && !v.can_carry_air() {
            v.eject();
        }
        if v.flags8() & OBJ_CITY_BUILDING != 0 && v.this_finished() {
            let share = v.share();
            if v.damage() < share {
                return ArmsOut::Return(0);
            }
            let share = v.share();
            let flags = v.flags8() | OBJ_HELD;
            v.set_flags8(flags);
            v.set_damage(share);
            return ArmsOut::Return(0);
        }
    } else {
        peasant_alarm(v, who);
    }
    ArmsOut::Join { share: v.share() }
}

/// A blow from outside on a city building: the victim's data is marked, the
/// city is marked struck and, for a computer player, its alarm is rung.
fn wall_notice<V: TakeDamageArms>(v: &mut V, who: i32, p7: i32, p8: i32) {
    let marked = if p7 < 0 || p8 < 0 {
        true
    } else if !v.attacker_peasant(p8, p7) && !v.attacker_obj_is(p8, p7, IS_ATTACKER_WALL) {
        true
    } else {
        v.cell_owner() == p8
    };
    if marked {
        v.mark_struck();
    }
    if v.data_city() < 0 {
        return;
    }
    let c = v.data_city();
    let flags = v.city_flags(c) | CITY_STRUCK;
    v.set_city_flags(c, flags);
    if v.leader_flags(who) & LEADER_OUT != 0
        || v.leader_word4(who) & 8 != 0
        || v.leader_diff(who) < ALARM_DIFFICULTY
    {
        return;
    }
    if !v.data_type_slot90() && v.data_flags8() & OBJ_CITY_BUILDING == 0 {
        return;
    }
    if v.city_flags(c) & CITY_BELL != 0 {
        return;
    }
    let (idx, player) = (v.city_building(c), v.city_player(c));
    push_alarm(v, who, idx, player);
}

/// A peasant struck in the open: the city it stands in rings its alarm.
fn peasant_alarm<V: TakeDamageArms>(v: &mut V, who: i32) {
    if v.leader_flags(who) & LEADER_OUT != 0
        || v.leader_word4(who) & 8 != 0
        || v.leader_diff(who) < ALARM_DIFFICULTY
        || !v.this_peasant()
    {
        return;
    }
    let c = v.find_city_at(who);
    if c < 0 {
        return;
    }
    let (idx, player) = (v.city_building(c), v.city_player(c));
    if !v.city_building_finished(player, idx) || v.city_flags(c) & CITY_BELL != 0 {
        return;
    }
    push_alarm(v, who, idx, player);
}

/// The stack group of one building, pushed on the owner's groups and
/// alarmed.
fn push_alarm<V: TakeDamageArms>(v: &mut V, who: i32, idx: i32, player: i32) {
    v.group_clear();
    v.group_clear();
    v.group_add(idx, player);
    let g = v.push_group(who);
    if g >= 0 {
        v.group_alarm(g);
    }
}
