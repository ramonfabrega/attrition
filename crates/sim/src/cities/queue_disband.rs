//! **A Disband queued at a building** (item 3918, `docs/GOLDEN.md` §165;
//! `docs/AI.md` §392.3): `DISBAND` (`TypeIndex` 0x29a) is a craft, and a
//! building takes it as a queue entry rather than a press. The chain, in the
//! original's order:
//!
//! ```text
//! CommandPackage::process_queue_up → Group::action_queue_up(0x29a, num)
//!   research arm: LeaderData::researching(0x29a, -1, 0, 0) refuses the command whole
//!   two passes over the members → Build::queue_up(0x29a, 1)
//!     no library forward (`param_1 != DISBAND`)
//!     BuildData::could_queue → can_make: active; queue_here; type_avail;
//!       has_spell(0x29a) → SpellTypeData::is_castable(o, who, 0) == 3
//!     the craft's price (none) and the room
//! Build::do_queue, the entry's clock: ObjectData::train_time@006508c0:51 —
//!   the building's own type's time, × DISBAND_CITY_RATE for a city,
//!   × DISBAND_SENATE_RATE for a Senate, × LAKOTA_RAZE_SPEED, then / 2
//! Build::finished@00628490's spell arm: has_spell(0x29a) →
//!   SpellType::cast@00676ce0(o, who, 0, 0): is_castable(o, who, 1) == 3 →
//!   cast_disband@00671ed0: Build::plunder(o, who) for an active plain
//!   building, then Object::disband(o, 0)
//! ```
//!
//! run1512's Barracks `0/2007` (city −1) holds the entry from 622 at 100 a
//! frame, reaches 21000 on 831 and is gone on 832; its twin run1512b enters
//! `SpellType::cast`, `cast_disband`, `Build::plunder` and `Object::disband`
//! on trace frame 831. The second Barracks' two presses (641, 661) are
//! refused whole by the research arm while `0/2007`'s entry stands.

use crate::Sim;
use crate::build::Ident;

/// `DISBAND`, the craft's `TypeIndex` (`craftrules.xml`'s 38th `CRAFT`,
/// `0x275 + 37`).
pub const DISBAND: i32 = 0x29a;

impl Sim {
    /// **`Group::action_queue_up@006fdbb0`** for `DISBAND`: the research
    /// arm ([`crate::group_queue_up::action_queue_up_steps`]) — refused whole
    /// while any of the player's buildings holds one
    /// ([`Sim::researching_disband`]), else the first member of two passes
    /// (the first only members holding nothing) whose
    /// [`Sim::queue_disband`] answers. Returns the entries laid, 0 or 1.
    pub fn action_queue_disband(&mut self, buildings: &[usize]) -> usize {
        self.queue_group(buildings, crate::group_queue_up::Queued::Disband, 0, 0)
    }

    /// `LeaderData::researching@006db510(0x29a, o, only, 0)`: an active
    /// building of `who`'s holding a `DISBAND` entry — any of them with
    /// `only` `None` (the group's `(-1, 0)`), the building `only` alone
    /// otherwise (`is_castable`'s `(o, 1)`).
    pub(crate) fn researching_disband(
        &self,
        who: crate::world::Player,
        only: Option<usize>,
    ) -> bool {
        self.buildings.iter().enumerate().any(|(b, bd)| {
            bd.owner == who
                && bd.alive
                && bd.active
                && only.is_none_or(|o| o == b)
                && bd.queue.count_disband() != 0
        })
    }

    /// `CityData::is_disbanding@007380a0`: the `DISBAND` entries the city's
    /// active buildings hold, skipping a type whose `+0x2c0 & 0x10` is set
    /// (`NO_CITY`, flag `e`: the Barracks, run1517's `0/2008`; item 3930,
    /// `docs/GOLDEN.md` §168).
    ///
    /// The twin run1517b's `is_castable` skips the city arm for its one city
    /// building, a Barracks; run1519b's Temple (`jam`) in city 0 enters it on
    /// 840, the press, and its entry closes the Temple on 1051 (item 3934,
    /// `docs/GOLDEN.md` §169).
    fn city_disbanding(&self, c: usize) -> i32 {
        self.buildings
            .iter()
            .filter(|bd| bd.alive && bd.active && bd.city == Some(c) && !self.no_city(bd.ty))
            .map(|bd| bd.queue.count_disband())
            .sum()
    }

    /// A building type's `+0x2c0 & 0x10`, `NO_CITY` (flag `e`).
    fn no_city(&self, ty: Option<usize>) -> bool {
        ty.is_some_and(|t| self.build_types[t].has(crate::build::flags::NO_CITY))
    }

    /// **`SpellTypeData::is_castable@00675bc0`'s 0x29a case** on building
    /// `b`, `finishing` its third argument (0 from `can_make`, 1 from
    /// `SpellType::cast`): the head exempts the craft, then
    ///
    /// - at the queue, a building already holding a `DISBAND` refuses;
    /// - the city arm is asked only of a type without `NO_CITY` (`+0x2c0 &
    ///   0x10` clear) or a city (`+8 & 0x20`): a Barracks (`e`) skips it,
    ///   and run1517b's press at the Barracks `0/2008` in city 0 enters no
    ///   `is_disbanding` (item 3930, `docs/GOLDEN.md` §168), while run1519b's
    ///   at a Temple does (item 3934, §169);
    /// - a building that is not a city in a city refuses while the city's
    ///   `is_disbanding` exceeds `finishing` — so at the finish its own
    ///   entry, still standing (`do_queue` unqueues after `finished`), is
    ///   allowed and a second is not.
    ///
    /// SEAM: the vslot `0x20` test (a wall answers 0 — this crate's
    /// buildings are not walls), and a city's own arm (`cities[c] + 4 &
    /// 0x10`, and the `+0x68` byte of other players' claims), which no
    /// capture reaches (scan: `grep -lF 'queue[scan].type 666'
    /// gamelog-run*.txt` over the 1,143 kept dumps on 2026-11-06: run1512b
    /// alone, a Barracks outside any city); it answers castable.
    pub(crate) fn disband_castable(&self, b: usize, finishing: bool) -> bool {
        let who = self.buildings[b].owner;
        if !finishing && self.researching_disband(who, Some(b)) {
            return false;
        }
        if let Some(c) = self.buildings[b].city
            && !self.no_city(self.buildings[b].ty)
            && !self.building_is_city(b)
            && self.city_disbanding(c) > i32::from(finishing)
        {
            return false;
        }
        true
    }

    /// **`Build::queue_up@00620f40(DISBAND, 1)`**: no library forward;
    /// `BuildData::could_queue` → `can_make` (the building active; the
    /// unassimilated arm exempts `DISBAND`; `has_spell` → [`Sim::
    /// disband_castable`] at the queue); no price; the room. True when it
    /// was laid.
    ///
    /// SEAM: `BuildTypeData::queue_here(0x29a)`, `type_avail(0x29a, 1)` and
    /// `LeaderData::has_spell(0x29a)` (`craftrules.xml`'s `PREQ0 None`,
    /// `TRIBE_MASK` all ones) answer yes; this crate's tree holds no craft.
    pub(crate) fn queue_disband(&mut self, at: usize) -> bool {
        if !self.buildings[at].alive || !self.buildings[at].active {
            return false;
        }
        if !self.disband_castable(at, false) || !self.buildings[at].queue.has_room() {
            return false;
        }
        self.buildings[at].queue.push_disband();
        true
    }

    /// **`ObjectData::train_time@006508c0:51`'s 0x29a block**: the time of
    /// the building's own type (`TypeData::time`, `JOB_TIME × 100`), a city
    /// (`is(0x19e)`) × `DISBAND_CITY_RATE` / 100, a Senate (`is(0x1b6)`) ×
    /// `DISBAND_SENATE_RATE` / 100, a Lakota owner × `LAKOTA_RAZE_SPEED` /
    /// 100, and the whole halved. run1512's Barracks: 42000 / 2 = 21000.
    ///
    /// SEAM: the block's recursive `train_time(type)` runs the whole
    /// function for the building type; for a building type its time is
    /// `TypeData::time`'s alone in every arm this crate carries.
    pub(crate) fn disband_time(&self, at: usize) -> i32 {
        let b = &self.buildings[at];
        let Some(ty) = b.ty else { return 0 };
        let mut t = self.build_types[ty].job_time * crate::production::TIME_SCALE;
        if self.building_is(at, Ident::Village) {
            t = self.tuning.disband_city_rate * t / 100;
        }
        if self.building_is(at, Ident::Senate) {
            t = self.tuning.disband_senate_rate * t / 100;
        }
        if self.nation[b.owner as usize].lakota {
            t = self.tuning.lakota_raze_speed * t / 100;
        }
        t / 2
    }

    /// **`Build::finished@00628490`'s spell arm** for a `DISBAND` entry:
    /// `has_spell(0x29a)` (yes, [`Sim::queue_disband`]'s SEAM), then
    /// `SpellType::cast(o, who, 0, 0)` — [`Sim::disband_castable`] at the
    /// finish, then `SpellType::cast_disband@00671ed0`: an active building
    /// (`+8 & 1`, vslots `0x20` and `0x4c`) is plundered by its own owner
    /// (`Build::plunder`, [`Sim::plunder_kill`]), and `Object::disband(o,
    /// 0)` ([`Sim::disband_building`]) closes it. The console's
    /// `S_ALERT_BUILDING_RAZED` is the interface's. `finished` answers 1
    /// either way, so a refused cast still spends the entry.
    pub(crate) fn finish_disband(&mut self, at: usize) {
        if !self.disband_castable(at, true) {
            return;
        }
        let owner = self.buildings[at].owner;
        if self.buildings[at].alive && self.buildings[at].active {
            self.plunder_kill(at, owner);
        }
        self.disband_building(at, false);
    }
}

#[cfg(test)]
mod tests;
