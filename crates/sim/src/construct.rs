//! `Wall::do_construct@006434d0` — one builder's contribution to a site
//! (`docs/CITIES.md` §3.3) — as a function over a host trait, so the sweep
//! (`sweep/wall_do_construct.rs`) runs the same steps against the original
//! under the emulator, and [`Sim::do_construct`] runs them on the world.

use crate::place::Blocked;
use crate::{Sim, build};

/// What `Wall::do_construct@006434d0` asks of the world, one method a call
/// the original makes (the sweep records them in order).
pub(crate) trait ConstructHost {
    /// `*GameAccess::ai_speed`.
    fn ai_speed(&mut self) -> i32;
    /// `is_started` (vslot `0x50`, inline `+8 & 2` for `WallData`).
    fn started(&mut self) -> bool;
    /// `BuildTypeData::blocked_site`'s verdict, as the original's integer.
    fn blocked_site(&mut self) -> i32;
    /// The `BuildData` the objects table holds for this building (vslot
    /// `0xac`): its city, `+0x72`.
    fn wonder_city(&mut self) -> i32;
    /// `LeaderData::has_tribe_bonus(7)`.
    fn tribe_bonus_7(&mut self) -> bool;
    /// `CityData::num_wonders(city, 1)`.
    fn num_wonders(&mut self, city: i32) -> i32;
    /// `Build::start` (vslot `0x1a4`, argument 1).
    fn start(&mut self);
    /// `Object::disband(1)`.
    fn disband(&mut self);
    /// `MiscAccess::console->who == this->who`.
    fn console_owns(&mut self) -> bool;
    /// The "construction blocked" message and sound.
    fn blocked_message(&mut self);
    /// `+8 & 4`.
    fn active(&mut self) -> bool;
    /// `+0x64`, the builders already counted this frame.
    fn helpers(&mut self) -> i32;
    /// `+0x60 & 0x800`.
    fn counted(&mut self) -> bool;
    /// `BuildData::recharging += 1` (vslot `0xac` again) and `+0x60 |= 0x800`.
    fn mark_counted(&mut self);
    /// `helpers += 1`.
    fn bump_helpers(&mut self);
    /// `+0x48`.
    fn job_counter(&mut self) -> i32;
    /// `+0x4c += share; +0x48 += share`.
    fn add_progress(&mut self, share: i32);
    /// `construct_time(0)` (vslot `0x18c`).
    fn construct_time(&mut self) -> i32;
    /// `activate(0, 1, 1)` (vslot `0x1a8`).
    fn activate(&mut self);
    /// `+8 & 0x20`.
    fn is_city(&mut self) -> bool;
    /// Vslot `0x2c`.
    fn slot_2c(&mut self) -> bool;
    /// The finished city's `add_event`.
    fn finish_event(&mut self);
    /// `String::String(EMPTY_STRING)`, ahead of the text.
    fn finish_begin(&mut self);
    /// The text a building's finished message is built from.
    fn finish_text(&mut self);
    /// `MessageWin::add_message` with the two `slot_2c` answers and the close.
    fn finish_message(&mut self, a: bool, b: bool);
    /// `MiscAccess::scene->recalc_builds = 1`.
    fn set_recalc_builds(&mut self);
}

/// `Wall::do_construct@006434d0`, over its host. Answers whether the
/// building finished.
pub(crate) fn do_construct_steps(h: &mut impl ConstructHost, param: i32) -> bool {
    let speed = h.ai_speed();
    let amount = if speed > 1 {
        speed.wrapping_mul(param)
    } else {
        param
    };
    if !h.started() {
        let r = h.blocked_site();
        let ok = if r == 0x2a {
            let city = h.wonder_city();
            city >= 0 && {
                let bonus = i32::from(h.tribe_bonus_7());
                h.num_wonders(city) <= bonus + 1
            }
        } else {
            matches!(r, 0 | 0x27 | 0x28 | 0x29 | 0x2b)
        };
        if !ok {
            h.disband();
            if h.console_owns() {
                h.blocked_message();
            }
            return false;
        }
        h.start();
    }
    if h.active() {
        return false;
    }
    let share = amount.wrapping_div(h.helpers() + 1);
    if !h.counted() {
        h.mark_counted();
    }
    h.bump_helpers();
    let old = h.job_counter();
    h.add_progress(share.max(1));
    let ct = h.construct_time();
    if ct as u32 <= h.job_counter() as u32 {
        h.activate();
        if h.console_owns() {
            if h.is_city() {
                h.finish_event();
            } else {
                h.finish_begin();
                if !h.slot_2c() {
                    h.finish_text();
                }
                let a = h.slot_2c();
                let b = h.slot_2c();
                h.finish_message(a, b);
            }
        }
        return true;
    }
    let ct_old = h.construct_time();
    let new = h.job_counter();
    let ct_new = h.construct_time();
    if build::recalc_crossed(old, new as u32, ct_old as u32, ct_new as u32) {
        h.set_recalc_builds();
    }
    false
}

/// [`ConstructHost`] on a [`Sim`] building.
///
/// SEAM: the console's messages and sounds (`console_owns` answers no: the
/// sim has no console surface), `ai_speed` (1: the cheat's counter is not
/// carried) and `scene->recalc_builds` (a renderer flag) are not modelled.
pub(crate) struct SimConstruct<'a> {
    pub sim: &'a mut Sim,
    pub b: usize,
}

impl ConstructHost for SimConstruct<'_> {
    fn ai_speed(&mut self) -> i32 {
        1
    }
    fn started(&mut self) -> bool {
        self.sim.buildings[self.b].started
    }
    fn blocked_site(&mut self) -> i32 {
        let bd = &self.sim.buildings[self.b];
        let Some(ty) = bd.ty else {
            return Blocked::Clear as i32;
        };
        let (who, pos) = (bd.owner, bd.pos);
        self.sim.blocked_site(Some(who), ty, pos, Some(self.b)) as i32
    }
    fn wonder_city(&mut self) -> i32 {
        self.sim.buildings[self.b].city.map_or(-1, |c| c as i32)
    }
    fn tribe_bonus_7(&mut self) -> bool {
        let who = self.sim.buildings[self.b].owner;
        self.sim.nation[who as usize].egyptians
    }
    fn num_wonders(&mut self, city: i32) -> i32 {
        self.sim.num_wonders(city as usize, true)
    }
    fn start(&mut self) {
        self.sim.start_building(self.b);
    }
    fn disband(&mut self) {
        self.sim.disband_building(self.b, true);
    }
    fn console_owns(&mut self) -> bool {
        false
    }
    fn blocked_message(&mut self) {}
    fn active(&mut self) -> bool {
        self.sim.buildings[self.b].active
    }
    fn helpers(&mut self) -> i32 {
        self.sim.buildings[self.b].helpers
    }
    fn counted(&mut self) -> bool {
        self.sim.buildings[self.b].gather_bumped
    }
    fn mark_counted(&mut self) {
        let bd = &mut self.sim.buildings[self.b];
        bd.recharging = bd.recharging.wrapping_add(1);
        bd.gather_bumped = true;
    }
    fn bump_helpers(&mut self) {
        self.sim.buildings[self.b].helpers += 1;
    }
    fn job_counter(&mut self) -> i32 {
        self.sim.buildings[self.b].job_counter
    }
    fn add_progress(&mut self, share: i32) {
        let bd = &mut self.sim.buildings[self.b];
        bd.job_counter_2 = bd.job_counter_2.wrapping_add(share);
        bd.job_counter = bd.job_counter.wrapping_add(share);
    }
    fn construct_time(&mut self) -> i32 {
        self.sim.construct_time_of(self.b)
    }
    fn activate(&mut self) {
        self.sim.activate(self.b, false, true);
    }
    fn is_city(&mut self) -> bool {
        false
    }
    fn slot_2c(&mut self) -> bool {
        false
    }
    fn finish_event(&mut self) {}
    fn finish_begin(&mut self) {}
    fn finish_text(&mut self) {}
    fn finish_message(&mut self, _a: bool, _b: bool) {}
    fn set_recalc_builds(&mut self) {}
}
