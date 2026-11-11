//! **The caster slots a unit is born into** (`docs/SCOUT.md` §16, item
//! 4113): `Specials::init_special@007401c0` and `Heroes::init_hero@0073a330`,
//! the two per-leader lists whose index `Unit::init@00612100:430`/`:460`
//! writes into the union at `UnitData +0x86`, and the two closes
//! `Unit::close@0060ee50` gives them back through.
//!
//! Both lists have the one shape [`crate::economy::caravan::Caravans`] has:
//! a record per slot with an in-use bit (`+0x28 & 1`), and a mark on the
//! leader one past the highest slot in use (`LeaderData +0x43c` for the
//! specials, `+0x420` for the heroes). A birth takes the first slot below
//! the mark whose bit is clear, else the mark itself — growing the array
//! when the mark has reached its length — and leaves the mark at
//! `max(mark, slot + 1)`. A close clears the bit and walks the mark down
//! past every free slot at the top. The record's other fields (the
//! caster's spell state) are not this module's; only the slot index is.
//!
//! **The hold doober** is the union's third writer and has its own table,
//! [`HoldDoobers`].

use crate::Sim;

/// One leader's specials or heroes list: the in-use bits, and the mark.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SlotList {
    /// One per record the array holds: `+0x28 & 1`, in use.
    pub in_use: Vec<bool>,
    /// The leader's mark (`+0x43c` specials, `+0x420` heroes).
    pub mark: usize,
}

impl SlotList {
    /// `init_special` / `init_hero`: the first free slot below the mark,
    /// else the mark (the array grown by one when the mark is its length).
    pub fn take(&mut self) -> usize {
        let slot = (0..self.mark)
            .find(|&i| !self.in_use[i])
            .unwrap_or(self.mark);
        if slot == self.in_use.len() {
            self.in_use.push(false);
        }
        self.in_use[slot] = true;
        self.mark = self.mark.max(slot + 1);
        slot
    }

    /// `close_special` / `close_hero`: the bit cleared, then the mark walked
    /// down while the record under it is free.
    pub fn give_back(&mut self, slot: usize) {
        if let Some(b) = self.in_use.get_mut(slot) {
            *b = false;
        }
        while self.mark > 0 && !self.in_use[self.mark - 1] {
            self.mark -= 1;
        }
    }
}

/// **The hold doobers' table** (`Doober::doober_hold_type`, `_locs`,
/// `_fade`; the global `doobers`): the prop a woodcutter or miner holds at
/// its tile. Only the fade matters to the simulation, because a slot is
/// free exactly when its fade is 0: `Doober::add_hold_doober@00846dd0`
/// takes the first slot whose fade is 0, else appends one, and writes 1;
/// `Doober::remove_hold_doobers@00846cd0` writes −255 (`0xffffff01`); and
/// `Doober::inc_time@00846770`, the last call but one of
/// `Objects::inc_time@0065db70`, steps every fade by 10 toward 255 or
/// toward 0, freeing a slot that reaches 0 and trimming free slots off the
/// end (`docs/ORDERS.md` §6.12). The type and the point are the
/// renderer's.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HoldDoobers {
    /// `doober_hold_fade`, one per slot: 0 free, 1..=255 fading in or
    /// held, negative fading out.
    pub fade: Vec<i32>,
}

/// `remove_hold_doobers`' fade: `0xffffff01`.
const FADE_OUT: i32 = -255;
/// `inc_time`'s step, both ways.
const FADE_STEP: i32 = 10;
/// `inc_time`'s ceiling.
const FADE_FULL: i32 = 0xff;

impl HoldDoobers {
    /// `Doober::add_hold_doober`: the first slot whose fade is 0, else a
    /// new one at the end; its fade 1.
    pub fn add(&mut self) -> usize {
        let slot = self
            .fade
            .iter()
            .position(|&f| f == 0)
            .unwrap_or(self.fade.len());
        if slot == self.fade.len() {
            self.fade.push(0);
        }
        self.fade[slot] = 1;
        slot
    }

    /// `Doober::remove_hold_doobers(slot)`: the fade set to −255. An index
    /// past the table is the original's `Invalid Doober Index` report and
    /// writes nothing.
    pub fn remove(&mut self, slot: usize) {
        if let Some(f) = self.fade.get_mut(slot) {
            *f = FADE_OUT;
        }
    }

    /// `Doober::inc_time`: every slot in order. A negative fade climbs by
    /// 10, and one that would reach 0 or above is 0 — and when it is the
    /// last slot, every free slot at the end is dropped. A positive fade
    /// climbs by 10 to 255.
    pub fn inc_time(&mut self) {
        let mut i = 0;
        while i < self.fade.len() {
            let f = self.fade[i];
            if f < 0 {
                if f + FADE_STEP < 0 {
                    self.fade[i] = f + FADE_STEP;
                } else {
                    self.fade[i] = 0;
                    if i + 1 == self.fade.len() {
                        while self.fade.last() == Some(&0) {
                            self.fade.pop();
                        }
                    }
                }
            } else if f > 0 {
                self.fade[i] = (f + FADE_STEP).min(FADE_FULL);
            }
            i += 1;
        }
    }
}

/// `ObjectData::is_worker` as the hold-doober sites test it, inline or by
/// call: the four citizen types, `TypeIndex` `0x32..=0x35`.
pub const DOOBER_HOLDERS: std::ops::RangeInclusive<i32> = 0x32..=0x35;

impl Sim {
    /// `Unit::init@00612100:405`–`:460`'s union arm for unit `u`, after the
    /// supply slot: a **hero** (`unit_flags2 & 0x20`) takes a heroes slot;
    /// else a land **caravan** has taken its route slot already
    /// ([`Sim::init_caravan`], `caravan_arm`); else a citizen type
    /// (`0x32..=0x35`) keeps −1, and a **special** (`unit_flags2 & 0x10`)
    /// takes a specials slot.
    pub(crate) fn init_caster_slot(&mut self, u: usize, caravan_arm: bool) {
        let Some(t) = self.units[u].ty else {
            return;
        };
        let cols = &self.tables.unit_types[t].cols;
        let hero = cols.flag2(crate::ai::ai_load::uflags2::FLAG2_GENERAL);
        let special = cols.flag2(crate::ai::ai_load::uflags2::FLAG2_SCOUT);
        let who = self.units[u].owner as usize;
        if hero {
            if let Some(list) = self.roster.heroes.get_mut(who) {
                self.units[u].hero = Some(list.take());
            }
            return;
        }
        if caravan_arm || DOOBER_HOLDERS.contains(&self.units[u].type_index) {
            return;
        }
        if special && let Some(list) = self.roster.specials.get_mut(who) {
            self.units[u].special = Some(list.take());
        }
    }

    /// `Unit::close@0060ee50:360`–`:404`'s two caster closes, and its
    /// hold-doober removal at `:133`–`:139` for a citizen type.
    pub(crate) fn close_caster_slot(&mut self, u: usize) {
        let who = self.units[u].owner as usize;
        if let Some(slot) = self.units[u].hero.take()
            && let Some(list) = self.roster.heroes.get_mut(who)
        {
            list.give_back(slot);
        }
        if let Some(slot) = self.units[u].special.take()
            && let Some(list) = self.roster.specials.get_mut(who)
        {
            list.give_back(slot);
        }
        if DOOBER_HOLDERS.contains(&self.units[u].type_index) {
            self.remove_hold_doober(u);
        }
    }

    /// `Unit::do_non_flat_gather@005f0170:212`–`:256`: a gatherer at its
    /// tile with no hold doober takes one.
    pub(crate) fn add_hold_doober(&mut self, u: usize) {
        if self.units[u].doober.is_none() {
            self.units[u].doober = Some(self.roster.hold_doobers.add());
        }
    }

    /// `Doober::remove_hold_doobers(unit +0x86)` and `+0x86 = −1`, for a
    /// unit holding one; nothing otherwise. Each caller tests its own
    /// predicate first (`docs/ORDERS.md` §6.12).
    pub(crate) fn remove_hold_doober(&mut self, u: usize) {
        if let Some(slot) = self.units[u].doober.take() {
            self.roster.hold_doobers.remove(slot);
        }
    }

    /// `Doober::inc_time@00846770` from `Objects::inc_time`.
    pub(crate) fn hold_doobers_inc_time(&mut self) {
        self.roster.hold_doobers.inc_time();
    }

    /// The union's value for a hero, a special or a hold doober, for the
    /// units line (`docs/CHECKSUM.md` §18).
    pub(crate) fn caster_or_doober_slot(&self, u: usize) -> Option<usize> {
        let un = &self.units[u];
        un.hero.or(un.special).or(un.doober)
    }
}
