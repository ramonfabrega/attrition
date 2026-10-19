//! `Unit::do_attack@005f1b80` — the attack order's own dispatch, before and
//! around `fight` (item 3304; swept against the original under the emulator
//! by `crates/sim/src/sweep/unit_do_attack*.rs`).
//!
//! The function is four arms and a tail, in the order the listing runs them:
//!
//! 1. **A carrier** (`this.is(AIRCRAFTCARRIER, 1)`, vtable `+0xb8`) never
//!    fights: it kills a dead target's order, waits for a frame divisible
//!    by sixteen, leaves a building target alone unless the order is
//!    mandatory, and then hands the strike to **every unit in its
//!    `inside_down` chain** as an `add_strafe_order` at `QUEUE_NEW`.
//! 2. **A transport barge** (`attack == 0` and `is(TRANSPORTBARGE, 0)`)
//!    lets its first passenger ask where to stand
//!    (`find_attack_pos`, the strict flag clear and then set) and walks
//!    there at `QUEUE_FIRST`; no spot and the target's own point is the
//!    walk.
//! 3. **A unit with no attack in a group** (`attack == 0`, group slot
//!    `>= 0`, so a supply wagon or a citizen the army carries) closes to
//!    `group max_range` + slack of its target, or drops the order when it
//!    already has: the slack is `(max_range + 2) × 0xc0`, a supply type and
//!    a worker each adding another `0xc0` (`(range₁ + 1) × 0xc0` and `0x180`
//!    when the group's own range is nothing), and a building target adds its
//!    footprint (`+0x234 + +0x238`) × 48. Out of reach, it takes the point
//!    `base + 0xc0` from the target along the bearing to itself, asks
//!    `find_nearby_spot` for it **with its group slot cleared**, and walks
//!    there at `QUEUE_FIRST` only when the spot is on the map and in the
//!    target's own `tregion`.
//! 4. **Everything else** — every unit with an attack, and an attack-less
//!    unit outside a group — runs the tail: the mandatory order's gun
//!    tracking (`unit_flags & 0x200000`), the cavalry archer's stance-2
//!    arm (`unit_flags & 0x400`), `fight`, and a cavalry archer's
//!    `cavarch_fight` after a `fight` that answered non-zero, with the
//!    recharge byte held at zero across it.
//!
//! Every arm that does not walk or strafe ends in `kill_current_order(0)`.
//!
//! The body is [`do_attack_steps`] over [`DoAttackView`], which `Sim` and the
//! sweep each implement; this crate's own `fight` half is
//! [`crate::Sim::do_attack_fight`], reached through [`DoAttackView::fight`].

mod live;

use crate::movement::{Angle, find_angle};
use crate::world::vector_dist;

/// What `Unit::do_attack` asks of its unit, the order and the world. Every
/// method is one read or one call the listing makes, in its order; a
/// method with no arguments acts on the order's target (`ox`, `whom`) or on
/// the cargo cursor ([`Self::cargo_start`]).
pub trait DoAttackView {
    /// `this.is(AIRCRAFTCARRIER, 1)` — vtable `+0xb8`, or `ObjectData::is`
    /// inlined to the type record's `+0x60`.
    fn is_carrier(&mut self) -> bool;
    /// The order's target slot has `flags & 1` (`objects[whom][ox]`).
    fn target_alive(&mut self) -> bool;
    /// `GameAccess::game->frame & 0xf != 0`.
    fn frame_off_phase(&mut self) -> bool;
    /// The target's vtable `+0x20` answers non-zero — a building.
    fn target_is_building(&mut self) -> bool;
    /// The order's `+0x1c`.
    fn mandatory(&mut self) -> bool;
    /// The cursor to `this+0x28` / `this+0x3e` (the first unit inside);
    /// whether it names one (`>= 0`).
    fn cargo_start(&mut self) -> bool;
    /// `add_strafe_order` on the cursor's unit: the order's target, this
    /// unit as home, `mandatory` as the order's, `QUEUE_NEW`, action set.
    fn strafe_cargo(&mut self);
    /// The cursor to the next link (`objects[who][idx]`'s `+0x28`/`+0x3e`);
    /// whether it names a unit.
    fn cargo_next(&mut self) -> bool;
    /// `kill_current_order(this, 0)`.
    fn kill_current_order(&mut self);
    /// The type record's `+0x1e8`.
    fn attack(&mut self) -> i32;
    /// `this.is(TRANSPORTBARGE, 0)` — vtable `+0xb8`, or inlined.
    fn is_barge(&mut self) -> bool;
    /// `Object::valid_target(passenger, ox, whom, NULL)` on the unit at the
    /// cursor (the first passenger).
    fn passenger_target_valid(&mut self) -> bool;
    /// `find_attack_pos(passenger, ox, whom, 0, &x, &y, here, strict)`.
    fn passenger_attack_pos(&mut self, strict: bool) -> Option<(i32, i32)>;
    /// The target object's position (`+0x10`/`+0x14` unmasked).
    fn target_pos(&mut self) -> (i32, i32);
    /// `add_move_order(x, y, 1, 0, QUEUE_FIRST, action)`.
    fn move_first(&mut self, x: i32, y: i32, action: bool);
    /// `this+0x80` as a signed short.
    fn group(&mut self) -> i32;
    /// `this+0x80 = group`.
    fn set_group(&mut self, group: i32);
    /// `ObjectData::is_worker@0046fa10`.
    fn is_worker(&mut self) -> bool;
    /// `GroupData::get_max_range(groups[group], worker)`.
    fn group_max_range(&mut self, worker: bool) -> i32;
    /// vtable `+0xcc` (`is_supply`), or the type's `+0x2b8 & 0x40`.
    fn is_supply(&mut self) -> bool;
    /// The order's target names an object (`whom >= 0 && ox >= 0`).
    fn target_set(&mut self) -> bool;
    /// The sum the target building's type holds at `+0x234` and `+0x238`.
    fn target_footprint(&mut self) -> i32;
    /// This unit's position, unmasked.
    fn pos(&mut self) -> (i32, i32);
    /// `UnitType::find_nearby_spot(centre, …, FILTER_NOT_ME, …)`: the spot,
    /// or `None` when the search answers non-zero.
    fn find_nearby_spot(&mut self, x: i32, y: i32) -> Option<(i32, i32)>;
    /// `WorldData::is_valid(&x, &y)`.
    fn spot_valid(&mut self, x: i32, y: i32) -> bool;
    /// `div_3_table[i]`.
    fn div3(&mut self, i: i32) -> i32;
    /// `WorldData::get_tregion(&tx, &ty)`.
    fn tregion(&mut self, tx: i32, ty: i32) -> i32;
    /// The type record's `unit_flags` (`+0x2b4`).
    fn unit_flags(&mut self) -> u32;
    /// Gun tracking for a mandatory order: `this+0xa2` / `+0xa8` take the
    /// target, and `unit_masks |= 0x100` with `+0xa6` the target's `+0x30`
    /// when it is a live object, else the bit cleared.
    fn track_target(&mut self);
    /// vtable `+0xf4` (`get_combat_stance`).
    fn stance(&mut self) -> i32;
    /// `type.+0x1fc = type.+0x2d0` (the cavalry archer's range) around
    /// `fight`, restored to zero after; `fight` as asked.
    fn fight_cavarch_range(&mut self, mandatory: bool) -> bool;
    /// `fight(ox, whom, mandatory, 0, 0)`; whether it answered non-zero.
    fn fight(&mut self, mandatory: bool) -> bool;
    /// `+0xae` (the recharge byte) read.
    fn recharge_byte(&mut self) -> u8;
    /// `+0xae = b`.
    fn set_recharge_byte(&mut self, b: u8);
    /// `Unit::cavarch_fight`.
    fn cavarch_fight(&mut self);
}

/// [`DoAttackView`]'s one body — see the module docs for the arms.
pub fn do_attack_steps<V: DoAttackView>(v: &mut V) {
    if v.is_carrier() {
        if !v.target_alive() {
            v.kill_current_order();
            return;
        }
        if v.frame_off_phase() {
            return;
        }
        if v.target_is_building() && !v.mandatory() {
            return;
        }
        if !v.cargo_start() {
            return;
        }
        loop {
            v.strafe_cargo();
            if !v.cargo_next() {
                break;
            }
        }
        return;
    }
    if v.attack() == 0 {
        if v.is_barge() {
            barge(v);
            return;
        }
        if v.attack() == 0 && v.group() >= 0 {
            grouped(v);
            return;
        }
    }
    tail(v);
}

/// The barge arm (`005f1caf`..`005f1dcc`).
fn barge<V: DoAttackView>(v: &mut V) {
    if !v.passenger_target_valid() {
        v.kill_current_order();
        return;
    }
    let at = v
        .passenger_attack_pos(false)
        .or_else(|| v.passenger_attack_pos(true))
        .unwrap_or_else(|| v.target_pos());
    v.move_first(at.0, at.1, true);
}

/// The grouped arm (`005f1ddf`..`005f2147`).
fn grouped<V: DoAttackView>(v: &mut V) {
    let worker = v.is_worker();
    let range = v.group_max_range(worker);
    let mut base;
    if range == 0 {
        let wide = v.group_max_range(true);
        base = wide.wrapping_add(1).wrapping_mul(0xc0);
        if v.is_supply() {
            base = base.wrapping_add(0x180);
        }
        if v.is_worker() {
            base = base.wrapping_add(0x180);
        }
    } else {
        base = range.wrapping_add(2).wrapping_mul(0xc0);
        if v.is_supply() {
            base = base.wrapping_add(0xc0);
        }
        if v.is_worker() {
            base = base.wrapping_add(0xc0);
        }
    }
    if !v.target_set() || !v.target_alive() {
        v.kill_current_order();
        return;
    }
    if v.target_is_building() {
        base = base.wrapping_add(v.target_footprint().wrapping_mul(0x30));
    }
    let (mx, my) = v.pos();
    let (tx, ty) = v.target_pos();
    if vector_dist(tx.wrapping_sub(mx), ty.wrapping_sub(my)) <= base.wrapping_add(0x240) {
        v.kill_current_order();
        return;
    }
    let toward_me = find_angle(mx.wrapping_sub(tx), my.wrapping_sub(ty));
    let (px, py) = project(tx, ty, toward_me, base.wrapping_add(0xc0));
    let group = v.group();
    v.set_group(-1);
    let spot = v.find_nearby_spot(px, py);
    v.set_group(group);
    if let Some((sx, sy)) = spot
        && v.spot_valid(sx, sy)
    {
        let (stx, sty) = (v.div3(sx >> 6), v.div3(sy >> 6));
        let (ttx, tty) = (v.div3(tx >> 6), v.div3(ty >> 6));
        let theirs = v.tregion(ttx, tty);
        let ours = v.tregion(stx, sty);
        if ours == theirs {
            v.kill_current_order();
            v.move_first(sx, sy, false);
            return;
        }
    }
    v.kill_current_order();
}

/// `project(p, angle, d)` — the crate's polar step.
fn project(x: i32, y: i32, a: Angle, d: i32) -> (i32, i32) {
    let p = crate::army::step_along(crate::Pos::new(x, y), a, d);
    (p.x, p.y)
}

/// The tail (`005f2159`..`005f226e`).
fn tail<V: DoAttackView>(v: &mut V) {
    let mandatory = v.mandatory();
    if mandatory {
        if v.unit_flags() & 0x20_0000 != 0 {
            v.track_target();
        }
    } else if v.unit_flags() & 0x400 != 0 && v.stance() == 2 {
        v.fight_cavarch_range(mandatory);
        return;
    }
    if v.fight(mandatory) && v.unit_flags() & 0x400 != 0 {
        let saved = v.recharge_byte();
        v.set_recharge_byte(0);
        v.cavarch_fight();
        v.set_recharge_byte(saved);
    }
}
