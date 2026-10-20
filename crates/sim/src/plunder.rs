//! `Unit::plunder@00605b00` as steps over a view (item 3356): the good a
//! kill, or a refund, puts in a leader's bucket, and the message that tells
//! the console player. [`plunder_steps`] is the original's control flow in
//! its order; [`PlunderView`] is what it reads and the callees it makes,
//! which [`SimPlunder`] answers from the crate's [`Sim`] and
//! `sweep/unit_plunder.rs` from the emulated original's windows.
//!
//! The function has three arms, told apart by its arguments:
//!
//! * **the refund** (`p1` the unit's own index and `p2` its owner): a unit
//!   that "plunders" itself pays back `INCA_REFUND` percent of its type's
//!   price in every good the owner has, and says so in a message;
//! * **a kill** in the patch the Extended Edition ships (`get_patch_version`
//!   above 8): a General, a caravan or a supply wagon pays its own constant
//!   (`GENERAL_PLUNDER`, `CARAVAN_PLUNDER`); an Aztec killer (`p3`) or a
//!   hero of The Despot near the victim (`p4`) names the least-stocked good
//!   by one `Random::get` per good, and the larger of the two amounts is
//!   paid in it, the first in `WEALTH`;
//! * **a kill** in the pre-Extended patches, the same without the larger-of.
//!
//! Every method is **one reach into the original**: a field, a virtual slot,
//! a callee. The sweep's stubs answer them in the order the original asks.
//! A leader's bucket is read and written as the plain count: the original's
//! `^ 0x8221` is the emulated view's to undo.
//!
//! **What the crate carries.** [`SimPlunder`] is the crate's one caller,
//! `Object::take_damage`'s kill of a squad's last figure: the Extended
//! Edition's arm for a victim that is no General, caravan or supply wagon,
//! with the killer's leader fixed by the caller. The constants the other
//! arms read (`GENERAL_PLUNDER`, `CARAVAN_PLUNDER`, `AZTEC_SLAY_*`,
//! `INCA_REFUND`) and the console's messages are not carried, so it
//! answers them as zero and as "not the console player".

use crate::Sim;
use crate::single::Single;

/// The goods the loops walk: `TypeIndex` `BASE_GOODTYPES` up to six.
const GOODS: i32 = 6;
/// `KNOWLEDGE`, never plundered.
const KNOWLEDGE: i32 = 3;
/// `WEALTH`, the good a fixed constant is paid in.
const WEALTH: i32 = 2;
/// The Despot and Spitamenes, the heroes that add `THEDESPOT_UNIT_PLUNDER`.
const DESPOT: i32 = 0x160;
const SPITAMENES: i32 = 0x16d;
/// The sound a plunder plays (`S_CARAVAN_PLUNDERED`).
const SOUND: i32 = 0x91;

/// The two local `String`s the original builds a message in: `A` is
/// `local_48`, the amount and the good; `B` is `local_5c`, the whole line.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Str {
    A,
    B,
}

/// What `Unit::plunder` reads of the unit, the leaders, the game and the
/// tables, and the callees it makes. A unit is named by its owner and its
/// index in the owner's unit table; `Ty` is a type record, `Bub` a
/// `TextBubble` taken from the recycler.
pub trait PlunderView {
    type Ty: Copy;
    type Bub: Copy;
    /// `this->flags` (the word at `+0x68`).
    fn flags68(&mut self) -> i32;
    /// `this->who` (the byte at `+9`).
    fn this_who(&mut self) -> i32;
    /// `this->index` (the short at `+0xa`).
    fn this_index(&mut self) -> i32;
    /// `this`'s position, the words at `+0x10` and `+0x14` unscrambled.
    fn this_coord(&mut self) -> (i32, i32);
    /// `this->type->flags & 0x10` (`+0x2b4`): the unit stands for another.
    fn this_redirects(&mut self) -> bool;
    /// The unit it stands for: the owner (the signed byte at `+0x3e`) and
    /// its index (the short at `+0x28`).
    fn this_redirect(&mut self) -> (i32, i32);
    /// `THEDESPOT_UNIT_PLUNDER`.
    fn thedespot_unit_plunder(&mut self) -> i32;
    /// `GENERAL_PLUNDER`.
    fn general_plunder(&mut self) -> i32;
    /// `CARAVAN_PLUNDER`.
    fn caravan_plunder(&mut self) -> i32;
    /// `AZTEC_SLAY_BONUS`.
    fn aztec_slay_bonus(&mut self) -> i32;
    /// `AZTEC_SLAY_CAP`.
    fn aztec_slay_cap(&mut self) -> i32;
    /// `AZTEC_PLUNDER`.
    fn aztec_plunder(&mut self) -> i32;
    /// `INCA_REFUND`.
    fn inca_refund(&mut self) -> i32;
    /// The short at leader `+0x59be` (`ty` The Despot) or `+0x59d8`
    /// (Spitamenes): how many heroes of the type the leader has.
    fn hero_count(&mut self, who: i32, ty: i32) -> i32;
    /// `HeroesData::find_hero@0073a1b0`: the hero of type `ty` within reach
    /// of `(x, y)`, or −1.
    fn find_hero(&mut self, x: i32, y: i32, who: i32, ty: i32) -> i32;
    /// `LeaderData::type_avail(good, 1)`.
    fn type_avail(&mut self, who: i32, good: i32) -> bool;
    /// `LeaderData::has_tribe_bonus(bonus)`.
    fn has_tribe_bonus(&mut self, who: i32, bonus: i32) -> bool;
    /// The leader's bucket of `good`.
    fn bucket(&mut self, who: i32, good: i32) -> i32;
    /// The word at bucket `+0xdc` (the Aztec slay count) unscrambled.
    fn slay_count(&mut self, who: i32) -> i32;
    /// The bucket of `good` gains `n`.
    fn add_good(&mut self, who: i32, good: i32, n: i32);
    /// The float at leader `+0x914`, its bits: the scenario's plunder scale.
    fn plunder_scale(&mut self, who: i32) -> u32;
    /// `units[who][index]->type` (the pointer at `+0x18`).
    fn unit_type(&mut self, who: i32, index: i32) -> Self::Ty;
    /// `units[who][index]->is(0x42, 0)`, inlined to its type's slot `0x60`
    /// when the unit's slot is `ObjectData::is`.
    fn unit_is_42(&mut self, who: i32, index: i32, ty: Self::Ty) -> bool;
    /// The unit's cached type index (`+0x54`).
    fn unit_cached_type(&mut self, who: i32, index: i32) -> i32;
    /// ... and its store.
    fn set_unit_cached_type(&mut self, who: i32, index: i32, t: i32);
    /// The type index a tribe's unit takes when it has none: `0x32` when
    /// `LeaderData::tribe_can_type` takes the record at `types[0x32]`, else
    /// that record's slot `0x68` handed the leader's `+8`.
    fn tribe_default_type(&mut self, who: i32) -> i32;
    /// `unittypes[index]`.
    fn unittype_record(&mut self, index: i32) -> Self::Ty;
    /// The type record's slot `0x78`: the price of `good` for `who`.
    fn price(&mut self, ty: Self::Ty, good: i32, who: i32) -> i32;
    /// `UnitData::is_hero` of the unit (inlined to a test of its type's
    /// `+0x2b8 & 0x20` when its slot is that function's).
    fn unit_is_hero(&mut self, who: i32, index: i32) -> bool;
    /// `UnitData::is_supply` (`+0x2b8 & 0x40`).
    fn unit_is_supply(&mut self, who: i32, index: i32) -> bool;
    /// `UnitData::is_caravan` (inlined to the type's slot `0x130`).
    fn unit_is_caravan(&mut self, who: i32, index: i32) -> bool;
    /// `Game::get_patch_version`.
    fn patch_version(&mut self) -> i32;
    /// The two game bytes the plunder reads (`+0x821`, `+0x822`).
    fn game_bytes(&mut self) -> (i32, i32);
    /// `ScenarioData::plunder`.
    fn scenario_plunder(&mut self) -> bool;
    /// `Random::get(game_random, lo, hi)`.
    fn rand(&mut self, lo: i32, hi: i32) -> i32;
    /// `console->who`.
    fn console_who(&mut self) -> i32;
    /// `s = loc_str_array[off]` (a localised string).
    fn str_assign_loc(&mut self, s: Str, off: i32);
    /// `s += loc_str_array[off]`.
    fn str_cat_loc(&mut self, s: Str, off: i32);
    /// `s += n`.
    fn str_cat_int(&mut self, s: Str, n: i32);
    /// `s += goodtypes[good]->name`.
    fn str_cat_good(&mut self, s: Str, good: i32);
    /// `dst += src`.
    fn str_cat_str(&mut self, dst: Str, src: Str);
    /// `_itow_s(n, String::buff, 0x400, 10)`.
    fn itow(&mut self, n: i32);
    /// The length of `String::buff` in wide characters.
    fn buff_len(&mut self) -> i32;
    /// `String::reinit(len, -1)`: zero when it made room.
    fn str_reinit(&mut self, s: Str, len: i32) -> i32;
    /// `wcscpy_s` of `String::buff` into the string.
    fn str_copy_buff(&mut self, s: Str);
    /// `String::close`.
    fn str_close(&mut self, s: Str);
    /// `Recycler<TextBubble>::pop`.
    fn pop_bubble(&mut self) -> Self::Bub;
    /// `bubble->string = s`.
    fn bubble_text(&mut self, b: Self::Bub, s: Str);
    /// `bubble->who = who`.
    fn bubble_who(&mut self, b: Self::Bub, who: i32);
    /// `TeamColor::get_neon` of the leader's team colour: the colour.
    fn neon(&mut self, who: i32) -> i32;
    /// `MessageWin::add_message(s, x, y, 0, color, 1, tc, 0, bubble, tc)`.
    fn add_message(&mut self, s: Str, b: Self::Bub, color: i32, who: i32);
    /// `SoundGlobal::play(sound)`.
    fn play(&mut self, sound: i32);
}

/// `Unit::plunder@00605b00(this, p1, p2, p3, p4)`.
pub fn plunder_steps<V: PlunderView>(v: &mut V, p1: i32, p2: i32, p3: i32, p4: i32) {
    if v.flags68() & 1 != 0 {
        return;
    }
    let who0 = v.this_who();
    let mut despot = p4;
    if p2 != who0 && v.thedespot_unit_plunder() != 0 && p4 == 0 {
        let (x, y) = v.this_coord();
        let first = if v.hero_count(p2, DESPOT) == 0 {
            -1
        } else {
            v.find_hero(x, y, p2, DESPOT)
        };
        let second = if v.hero_count(p2, SPITAMENES) == 0 {
            -1
        } else {
            v.find_hero(x, y, p2, SPITAMENES)
        };
        despot = i32::from(first >= 0 || second >= 0);
    }
    let id0 = v.this_index();
    let (mut who, mut idx) = (who0, id0);
    if v.this_redirects() {
        (who, idx) = v.this_redirect();
    }
    if idx < 0 {
        (who, idx) = (who0, id0);
    }
    if p1 == id0 && p2 == who0 {
        refund(v, p2, who0, who, idx);
        return;
    }
    let mut amount;
    let good;
    if v.patch_version() < 9 {
        if p3 != 0 || despot != 0 {
            let (best, slay) = pick(v, p2, p3, despot);
            (good, amount) = (best, slay);
        } else {
            good = WEALTH;
            amount = if v.unit_is_hero(who, idx) {
                v.general_plunder()
            } else {
                // The answer is not read: a supply wagon pays what a
                // caravan does, and so does anything else.
                v.unit_is_supply(who, idx);
                v.caravan_plunder()
            };
        }
    } else {
        let mut base = 0;
        if v.unit_is_hero(who, idx) {
            base = v.general_plunder();
        } else if v.unit_is_supply(who, idx) || v.unit_is_caravan(who, idx) {
            base = v.caravan_plunder();
        }
        amount = base;
        if p3 == 0 && despot == 0 {
            good = WEALTH;
        } else {
            let (best, slay) = pick(v, p2, p3, despot);
            if base < slay {
                (good, amount) = (best, slay);
            } else {
                (good, amount) = (WEALTH, base);
            }
        }
    }
    if p3 == 0 {
        let aztec = v.aztec_plunder();
        if aztec != 0 && v.has_tribe_bonus(p2, 0) {
            amount = aztec.wrapping_mul(amount) / 100;
        }
    }
    let (sem1, sem2) = v.game_bytes();
    if (sem1 & 0x10 != 0 || sem2 & 2 != 0) && !v.scenario_plunder() {
        return;
    }
    let scale = Single::from_bits(v.plunder_scale(p2));
    if scale.gt(Single::ZERO) || scale.eq_value(Single::ZERO) {
        amount = Single::from_i32(amount).mulss(scale).to_i32();
    }
    v.add_good(p2, good, amount);
    if p2 != v.console_who() {
        return;
    }
    let hero = v.unit_is_hero(who, idx);
    v.str_assign_loc(Str::B, if hero { 0x11e90 } else { 0x11ea4 });
    v.itow(amount);
    let len = v.buff_len();
    if v.str_reinit(Str::A, len) == 0 {
        v.str_copy_buff(Str::A);
    }
    v.str_cat_loc(Str::A, 0x11e40);
    v.str_cat_good(Str::A, good);
    v.str_cat_str(Str::B, Str::A);
    post(v, Str::B, Str::A, p2);
    v.str_close(Str::A);
    v.str_close(Str::B);
}

/// The good the leader holds least of (one draw each, `KNOWLEDGE` skipped,
/// the first on a tie) and what is paid in it: the Aztec slay count times
/// `AZTEC_SLAY_BONUS` up to `AZTEC_SLAY_CAP` when `p3`, plus The Despot's
/// cut when `despot`.
fn pick<V: PlunderView>(v: &mut V, who: i32, p3: i32, despot: i32) -> (i32, i32) {
    let (mut best, mut least) = (2, 99_999_999);
    for g in 0..GOODS {
        if v.type_avail(who, g) && g != KNOWLEDGE {
            let b = v.bucket(who, g);
            let r = v.rand(0, 0xffff);
            let score = b.wrapping_add(r % 25);
            if score < least {
                (best, least) = (g, score);
            }
        }
    }
    let mut slay = 0;
    if p3 != 0 {
        let cap = v.aztec_slay_cap();
        slay = v
            .slay_count(who)
            .wrapping_add(1)
            .wrapping_mul(v.aztec_slay_bonus());
        if cap < slay {
            slay = cap;
        }
    }
    if despot != 0 {
        slay = slay.wrapping_add(v.thedespot_unit_plunder());
    }
    (best, slay)
}

/// The refund arm: the unit `units[who][idx]` pays `p2` back in every good.
fn refund<V: PlunderView>(v: &mut V, p2: i32, who0: i32, who: i32, idx: i32) {
    let mut ty = v.unit_type(who, idx);
    if v.unit_is_42(who, idx, ty) {
        if v.unit_cached_type(who, idx) == 0 {
            let t = v.tribe_default_type(who0);
            v.set_unit_cached_type(who, idx, t);
        }
        let t = v.unit_cached_type(who, idx);
        ty = v.unittype_record(t);
    }
    let mut any = false;
    for g in 0..GOODS {
        if v.type_avail(p2, g) {
            let price = v.price(ty, g, who0);
            let n = v.inca_refund().wrapping_mul(price) / 100;
            if n != 0 {
                if any {
                    v.str_cat_loc(Str::A, 0x11e7c);
                }
                v.str_cat_int(Str::A, n);
                v.str_cat_loc(Str::A, 0x11e40);
                v.str_cat_good(Str::A, g);
                any = true;
                v.add_good(p2, g, n);
            }
        }
    }
    if p2 == v.console_who() && any {
        post(v, Str::A, Str::A, p2);
    }
    v.str_close(Str::A);
}

/// The message and the sound: a bubble that says `text`, the line `msg`
/// in the leader's neon.
fn post<V: PlunderView>(v: &mut V, msg: Str, text: Str, who: i32) {
    let b = v.pop_bubble();
    v.bubble_text(b, text);
    v.bubble_who(b, who);
    let color = v.neon(who);
    v.add_message(msg, b, color, who);
    v.play(SOUND);
}

/// [`PlunderView`] for the crate's one caller: `Object::take_damage`'s
/// kill of the squad captain `v` by a unit of `who`, handed `(k, who, 0,
/// 1)`. The crate's tribe flags, buckets and draws answer; what it does not
/// carry is answered as the sentence on the module says.
pub struct SimPlunder<'a> {
    pub sim: &'a mut Sim,
    pub v: usize,
}

impl PlunderView for SimPlunder<'_> {
    type Ty = ();
    type Bub = ();
    fn flags68(&mut self) -> i32 {
        0
    }
    fn this_who(&mut self) -> i32 {
        i32::from(self.sim.units[self.v].owner)
    }
    fn this_index(&mut self) -> i32 {
        self.v as i32
    }
    fn this_coord(&mut self) -> (i32, i32) {
        let p = self.sim.units[self.v].pos;
        (p.x, p.y)
    }
    fn this_redirects(&mut self) -> bool {
        false
    }
    fn this_redirect(&mut self) -> (i32, i32) {
        (-1, -1)
    }
    fn thedespot_unit_plunder(&mut self) -> i32 {
        self.sim.tuning.thedespot_unit_plunder
    }
    fn general_plunder(&mut self) -> i32 {
        0
    }
    fn caravan_plunder(&mut self) -> i32 {
        0
    }
    fn aztec_slay_bonus(&mut self) -> i32 {
        0
    }
    fn aztec_slay_cap(&mut self) -> i32 {
        0
    }
    fn aztec_plunder(&mut self) -> i32 {
        self.sim.tuning.aztec_plunder
    }
    fn inca_refund(&mut self) -> i32 {
        0
    }
    fn hero_count(&mut self, _who: i32, _ty: i32) -> i32 {
        // `find_hero` counts them itself.
        1
    }
    fn find_hero(&mut self, x: i32, y: i32, who: i32, ty: i32) -> i32 {
        let at = crate::world::Pos { x, y };
        i32::from(!self.sim.hero_near(who as crate::Player, ty, at, 0)) - 1
    }
    fn type_avail(&mut self, who: i32, good: i32) -> bool {
        self.sim.holdings[who as usize].available[good as usize]
    }
    fn has_tribe_bonus(&mut self, who: i32, bonus: i32) -> bool {
        bonus == 0 && self.sim.nation[who as usize].aztecs
    }
    fn bucket(&mut self, who: i32, good: i32) -> i32 {
        self.sim.ledgers[who as usize].bucket[good as usize]
    }
    fn slay_count(&mut self, _who: i32) -> i32 {
        0
    }
    fn add_good(&mut self, who: i32, good: i32, n: i32) {
        let b = &mut self.sim.ledgers[who as usize].bucket[good as usize];
        *b = b.wrapping_add(n);
    }
    fn plunder_scale(&mut self, _who: i32) -> u32 {
        // `Leader::init` sets -1.0 and only a scenario script writes it.
        Single::from_i32(-1).bits()
    }
    fn unit_type(&mut self, _who: i32, _index: i32) {}
    fn unit_is_42(&mut self, _who: i32, _index: i32, _ty: ()) -> bool {
        false
    }
    fn unit_cached_type(&mut self, _who: i32, _index: i32) -> i32 {
        0
    }
    fn set_unit_cached_type(&mut self, _who: i32, _index: i32, _t: i32) {}
    fn tribe_default_type(&mut self, _who: i32) -> i32 {
        0
    }
    fn unittype_record(&mut self, _index: i32) {}
    fn price(&mut self, _ty: (), _good: i32, _who: i32) -> i32 {
        0
    }
    fn unit_is_hero(&mut self, _who: i32, _index: i32) -> bool {
        false
    }
    fn unit_is_supply(&mut self, _who: i32, _index: i32) -> bool {
        false
    }
    fn unit_is_caravan(&mut self, _who: i32, _index: i32) -> bool {
        false
    }
    fn patch_version(&mut self) -> i32 {
        // The Extended Edition's.
        9
    }
    fn game_bytes(&mut self) -> (i32, i32) {
        (0, 0)
    }
    fn scenario_plunder(&mut self) -> bool {
        false
    }
    fn rand(&mut self, lo: i32, hi: i32) -> i32 {
        debug_assert_eq!((lo, hi), (0, 0xffff));
        self.sim.mark(crate::fight::SITE_UNIT_PLUNDER);
        self.sim.rng.roll()
    }
    fn console_who(&mut self) -> i32 {
        -1
    }
    fn str_assign_loc(&mut self, _s: Str, _off: i32) {}
    fn str_cat_loc(&mut self, _s: Str, _off: i32) {}
    fn str_cat_int(&mut self, _s: Str, _n: i32) {}
    fn str_cat_good(&mut self, _s: Str, _good: i32) {}
    fn str_cat_str(&mut self, _dst: Str, _src: Str) {}
    fn itow(&mut self, _n: i32) {}
    fn buff_len(&mut self) -> i32 {
        0
    }
    fn str_reinit(&mut self, _s: Str, _len: i32) -> i32 {
        1
    }
    fn str_copy_buff(&mut self, _s: Str) {}
    fn str_close(&mut self, _s: Str) {}
    fn pop_bubble(&mut self) {}
    fn bubble_text(&mut self, _b: (), _s: Str) {}
    fn bubble_who(&mut self, _b: (), _who: i32) {}
    fn neon(&mut self, _who: i32) -> i32 {
        0
    }
    fn add_message(&mut self, _s: Str, _b: (), _color: i32, _who: i32) {}
    fn play(&mut self, _sound: i32) {}
}
