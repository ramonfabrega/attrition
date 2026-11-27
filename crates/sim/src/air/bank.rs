//! **The bank of a plane** (`Unit::bank_aircraft@005e9520` for a plane and
//! a flock's bird, and `do_air_physics@005e86d0`'s `local_20`, the turn sign
//! it hands in; `docs/ORDERS.md` §33.3, §34.6). It moved here whole from
//! `air.rs` with item 4740, which added the sign.

use super::*;

impl Sim {
    /// **`Unit::bank_aircraft(des, &speed, 0)` for a plane** —
    /// [`Sim::bank_aircraft`]'s shape, with the three arms the order's
    /// `returning` flag opens (`docs/ORDERS.md` §33.3): the bank it wants
    /// is **doubled** before the clamp, a **second turn** toward `des`
    /// follows the first (and takes it outright inside 3°), and a banked
    /// plane flies at **three quarters** of its speed.
    ///
    /// **Not returning** (`docs/ORDERS.md` §34.6), a player's plane wants
    /// **no bank at all within 199 of the ground** under it (`0x5e9700`:
    /// `|guy.z − find_tcoord_z| ≤ 199`), and none of the three arms: it
    /// levels its wings off the runway before it turns.
    ///
    /// **An animal** — a flock's bird (`crate::gaia::flock`) — takes two arms a
    /// plane never does (`005e9520`): with its bank settled and its
    /// `spell_time` not negative it skips the whole call every eighth
    /// frame (`frame & 7`, the game's own), and the ground test is an
    /// owner-under-eight's (`(byte)+0x9 > 7` goes straight to the clamp).
    pub(crate) fn bank_plane(&mut self, u: usize, des: Angle, speed: &mut i32, returning: bool) {
        let animal = self.units[u].is_gaia();
        self.bank_plane_as(u, des, speed, returning, animal, false, 0);
    }

    /// **`do_air_physics`'s close-target arm** (`0x5e8a0c`–`0x5e8b6a`,
    /// item 4740, `docs/ORDERS.md` §34.6): `local_20`, the turn sign
    /// `bank_aircraft` is handed as its `param_3`. It is 1 for a flight
    /// that is not returning (`AirOrder +0x18`), whose front order is a
    /// strike (`is_attack`, vslot `+0x18`), flown by a type that is not a
    /// helicopter, on a target whose type flies (`+0x218 == 2`, a
    /// helicopter included), that `valid_target` takes, nearer than
    /// `min_range · 0xc0` (`vector_dist(|dx|, |dy|)`, strict) and within
    /// 90° of the nose (`fold(find_angle(dx, dy) − heading) < 0x4000_0000`).
    /// Otherwise 0, and `bank_aircraft` takes the sign of `des − heading`.
    /// So a plane on top of a flying target banks the `+` way whichever
    /// side the target is on. run1797's Jet Fighter `2/116` on 3789, 106 off
    /// the Attack Helicopter `1/34` under its nose, banks −10 and turns
    /// right, away from it.
    pub(super) fn close_air_target(&mut self, u: usize, heli: bool, returning: bool) -> i32 {
        if returning || heli {
            return 0;
        }
        let Some(t) = self.current_strafe(u).and_then(|sf| sf.target) else {
            return 0;
        };
        if !matches!(
            self.profile(t).domain,
            crate::territory::attrition::Domain::Air
        ) {
            return 0;
        }
        let me = crate::combat::Obj::Unit(u);
        if !self.valid_target(me, t) {
            return 0;
        }
        let (at, to) = (self.units[u].pos, self.pos_of(t));
        let (dx, dy) = (to.x - at.x, to.y - at.y);
        if vector_dist(dx.abs(), dy.abs()) >= self.profile(me).min_range * 0xc0 {
            return 0;
        }
        let heading = self.units[u].movement.heading;
        let off = fold((find_angle(dx, dy).0 as u32).wrapping_sub(heading.0 as u32));
        i32::from(off < 0x4000_0000)
    }

    /// [`Sim::bank_plane`] with the two answers it reads off the unit made
    /// arguments (item 3272, the sweep of `Unit::bank_aircraft@005e9520`):
    /// `animal` is the `is_animal` slot (`+0x30`), the owner byte's own
    /// `< 8` test staying the unit's; `heli` the type's helicopter bit,
    /// whose arms the plane's caller does not yet ask for (parked 3272a).
    ///
    /// **A helicopter** reads its bank five times over (`roll × 5` before the
    /// work, `÷ 5` before the store) when it is not an animal, takes no
    /// floor on the divisor, never doubles the wanted bank on a return (it
    /// goes the ground-clearance way instead), and [`air_turn_speed_as`]
    /// reads its bank times five too.
    pub fn bank_plane_as(
        &mut self,
        u: usize,
        des: Angle,
        speed: &mut i32,
        returning: bool,
        animal: bool,
        heli: bool,
        turn_sign: i32,
    ) {
        let stored = self.units[u].airframe.bank;
        let mut roll = stored.neg();
        if animal {
            if roll.is_zero() && self.units[u].spell_time >= 0 && self.frame & 7 == 0 {
                return;
            }
        } else if heli {
            roll = roll.mulss(F5);
        }
        let heading = self.units[u].movement.heading;
        let d = (des.0 as u32).wrapping_sub(heading.0 as u32);
        let adelta = fold(d);
        // `param_3`: the caller's sign when it hands one (`do_air_physics`'s
        // close-target arm, [`Sim::close_air_target`]), else the way to
        // `des` (`005e9573`).
        let sign_in = if turn_sign != 0 {
            turn_sign
        } else if d > 0x8000_0000 {
            -1i32
        } else {
            1
        };
        let raw = ((self.units[u].movement.turning.type_turn_speed as u32) >> 8)
            .wrapping_mul(self.tables.tuning.unit_turn_speed as u32);
        let divisor = if raw < MIN_TURN && !heli {
            MIN_TURN
        } else {
            raw
        };
        let mut want = Single::from_u32(adelta)
            .divss(Single::from_i32(divisor as i32))
            .mulss(F55)
            .mulss(F_33);
        if F5.gt(want.abs()) {
            want = want.mulss(F_HALF);
        }
        if returning && !heli {
            // The returning arm: doubled, then the upper clamp.
            want = want.addss(want);
            if !F55.gt(want) {
                want = F55;
            }
        } else {
            let at = self.units[u].pos;
            let low = self.units[u].owner < 8
                && (self.units[u].airframe.z - self.world.tile_z(at.tile())).abs() <= 199;
            if low {
                want = Single::ZERO;
            } else if !F55.gt(want) {
                want = F55;
            }
        }
        let step = Single::from_i32(sign_in).mulss(want).subss(roll);
        let mag = step.abs();
        let mut settled = false;
        if F2.gt(mag) {
            if F2.gt(roll.abs()) {
                roll = Single::ZERO;
                settled = true;
            }
        } else if step.gt(Single::ZERO) {
            if F55.gt(roll) {
                roll = roll.addss(if F10.gt(mag) { mag } else { F10 });
            }
        } else if roll.gt(F55_NEG) {
            roll = roll.subss(if F10.gt(mag) { mag } else { F10 });
        }
        let mut sign_out = 0;
        if !settled {
            if roll.gt(Single::ZERO) {
                sign_out = 1;
            } else if Single::ZERO.gt(roll) {
                sign_out = -1;
            } else {
                settled = true;
            }
        }
        let mut snapped = false;
        if settled {
            sign_out = 0;
            if adelta < SNAP_WITHIN {
                self.units[u].movement.heading = des;
                snapped = true;
            }
        }
        if !snapped {
            let rate = air_turn_speed_as(raw, stored, sign_out, heli) as i32;
            let h = self.units[u].movement.heading;
            self.units[u].movement.heading = Angle(h.0.wrapping_add(rate.wrapping_mul(sign_out)));
        }
        // `LAB_005e97b9`: the second turn, which only a returning order
        // takes.
        if returning {
            let h = self.units[u].movement.heading;
            let d2 = fold((h.0 as u32).wrapping_sub(des.0 as u32));
            if d2 < RETURN_SNAP {
                self.units[u].movement.heading = des;
            } else {
                let rate = air_turn_speed_as(raw, stored, sign_out, heli) as i32;
                let h2 = Angle(h.0.wrapping_add(rate.wrapping_mul(sign_out)));
                if fold((h2.0 as u32).wrapping_sub(des.0 as u32)) < d2 {
                    self.units[u].movement.heading = h2;
                }
            }
        }
        // `(s·3 + (s·3 >> 31 & 3)) >> 2`: three quarters, toward zero —
        // returning only.
        if returning && !roll.eq_value(Single::ZERO) {
            *speed = speed.wrapping_mul(3) / 4;
        }
        if heli {
            roll = roll.divss(F5);
        }
        let af = &mut self.units[u].airframe;
        af.last_bank = af.bank;
        af.bank = roll.neg();
    }
}
