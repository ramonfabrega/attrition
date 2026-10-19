//! `Group::action_queue_up@006fdbb0` as steps over a view (item 3330,
//! `docs/PRODUCTION.md` "The player's order"): the group a `queue_up`
//! command names is sorted by what each member already holds, then the type
//! picks the arm. [`action_queue_up_steps`] is the original's control flow,
//! call for call and in its order; [`QueueUpView`] is the state it reads and
//! the callees it makes, which [`SimQueueUp`] answers from the crate's
//! [`Sim`] and `sweep/group_action_queue_up.rs` from the emulated original's
//! windows.
//!
//! ```text
//! scenario ignore-orders on, who < 8:  for each object it lists, >= 0: group slot +0x10(o, who, 0, 0)
//! group flag +0x49 clear:              return
//! list = the group's shorts (+0x8cc), count at +0xc
//! selection sort by queued, least first, from each i whose object is alive and active
//! group slot +0x14()
//! train    = is_unit_type(ty) and the player's bit for ty set
//!          | is_build_type(ty) and the building type's +0x2c0 & 4 clear
//! train:   num times over, each member alive, active and targetable:
//!              a silo (is(0x208)) and is_unit_type(ty): can_carry(ty) else skip
//!              Build::queue_up(ty, 1)                       (the answer is not read)
//! research: LeaderData::researching(ty, -1, 0, 0) set:  the console hears a sound and a feedback line, nothing else
//!          else two passes over the list: a member alive, active, targetable,
//!          and in the first pass holding nothing, is offered Build::queue_up(ty, 1);
//!          the first that answers 0 ends the command
//! ```

use crate::Sim;

/// The only object `is` is asked about here: `MISSILE_SILO`, `TypeIndex` 0x208.
pub const SILO: i32 = 0x208;

/// What `action_queue_up` reads and does. An object `o` is the short the
/// group's list holds: its place in the player's object array.
pub trait QueueUpView {
    /// `ScenarioData::ignore_orders != 0`.
    fn ignore_orders_on(&self) -> bool;
    /// `objects_ignoring_orders[who]`'s count, read again at every step.
    fn ignored_count(&self, who: u8) -> i32;
    /// Its `i`th entry.
    fn ignored_nth(&self, who: u8, i: i32) -> i32;
    /// The group's own slot `+0x10(o, who, 0, 0)`.
    fn ignore_call(&mut self, o: i32, who: u8);
    /// The group's byte `+0x49`.
    fn group_on(&self) -> bool;
    /// The group's player, the byte `+0x4a`.
    fn who(&self) -> u8;
    /// The group's count (`+0xc`).
    fn count(&self) -> i32;
    /// The short at `+0x8cc + 2k`.
    fn member(&self, k: usize) -> i32;
    /// The object's slot `+0xc`.
    fn alive(&mut self, o: i32) -> bool;
    /// The object's slot `+0x4c`.
    fn active(&mut self, o: i32) -> bool;
    /// The object's slot `+0x20`: a folded `return 1` on a building.
    fn targetable(&mut self, o: i32) -> bool;
    /// The byte at `+0x82` of the object's `Build` (its slot `+0xac`).
    fn queued(&mut self, o: i32) -> u8;
    /// The group's slot `+0x14`.
    fn refresh(&mut self);
    /// The type record's slot `+0xc`: `TypeData::is_unit_type`.
    fn type_is_unit(&mut self, ty: i32) -> bool;
    /// The player's availability bit for `ty` (`0xe40fa8 + who * 0x6eec`).
    fn type_avail(&mut self, ty: i32) -> bool;
    /// The type record's slot `+0x14`: `TypeData::is_build_type`.
    fn type_is_build(&mut self, ty: i32) -> bool;
    /// The building type's `+0x2c0 & 4`.
    fn build_flag4(&mut self, ty: i32) -> bool;
    /// `ObjectData::is(ty, 0)` of the object.
    fn is(&mut self, o: i32, ty: i32) -> bool;
    /// `ObjectData::can_carry(ty)` of the object.
    fn can_carry(&mut self, o: i32, ty: i32) -> bool;
    /// `Build::queue_up(ty, 1)` of the object's `Build`: true when it answers 0.
    fn queue_up(&mut self, o: i32, ty: i32) -> bool;
    /// `LeaderData::researching(ty, -1, 0, 0)` of the player's leader.
    fn researching(&mut self, ty: i32) -> bool;
    /// The player is `MiscAccess::console->who`.
    fn is_console(&mut self) -> bool;
    /// The refusal the console's player hears: a sound and `add_feedback`.
    fn refused(&mut self);
}

/// The original's body over a view; `ty` and `num` are its two arguments.
/// Returns the entries laid (the crate's own count: the original returns
/// nothing).
pub fn action_queue_up_steps<V: QueueUpView>(v: &mut V, ty: i32, num: i32) -> usize {
    let who = v.who();
    if v.ignore_orders_on() && who < 8 {
        let mut i = 0;
        while i < v.ignored_count(who) {
            let o = v.ignored_nth(who, i);
            if o >= 0 {
                v.ignore_call(o, who);
            }
            i += 1;
        }
    }
    if !v.group_on() {
        return 0;
    }
    let count = v.count();
    let n = usize::try_from(count).unwrap_or(0);
    let mut list: Vec<i32> = (0..n).map(|k| v.member(k)).collect();
    for i in 0..n.saturating_sub(1) {
        if !(v.alive(list[i]) && v.active(list[i])) {
            continue;
        }
        for j in i + 1..n {
            let qj = v.queued(list[j]);
            let qi = v.queued(list[i]);
            if qi > qj {
                list.swap(i, j);
            }
        }
    }
    v.refresh();
    let train =
        (v.type_is_unit(ty) && v.type_avail(ty)) || (v.type_is_build(ty) && !v.build_flag4(ty));
    let mut laid = 0;
    if train {
        for _ in 0..num.max(0) {
            for &o in &list {
                if v.alive(o) && v.active(o) && v.targetable(o) {
                    if v.is(o, SILO) && v.type_is_unit(ty) && !v.can_carry(o, ty) {
                        continue;
                    }
                    if v.queue_up(o, ty) {
                        laid += 1;
                    }
                }
            }
        }
        return laid;
    }
    if v.researching(ty) {
        if v.is_console() {
            v.refused();
        }
        return 0;
    }
    for pass in 0..2 {
        for &o in &list {
            if v.alive(o)
                && v.active(o)
                && v.targetable(o)
                && (pass != 0 || v.queued(o) == 0)
                && v.queue_up(o, ty)
            {
                return 1;
            }
        }
    }
    0
}

/// What the command asks the building to queue.
#[derive(Clone, Copy)]
pub(crate) enum Queued {
    /// A unit type, by the crate's unit-type index.
    Unit(usize),
    /// A technology.
    Tech(crate::tech::TypeId),
}

/// The crate's [`Sim`] as the view: `o` is a building's index, the group's
/// list the buildings the command names. The scenario's ignore-orders filter,
/// the console's refusal sound and `targetable` (a folded `return 1` on every
/// building) are the seams: nothing in the crate carries them.
pub(crate) struct SimQueueUp<'a> {
    pub(crate) sim: &'a mut Sim,
    pub(crate) who: crate::world::Player,
    pub(crate) list: &'a [usize],
    pub(crate) what: Queued,
}

impl QueueUpView for SimQueueUp<'_> {
    fn ignore_orders_on(&self) -> bool {
        false
    }
    fn ignored_count(&self, _who: u8) -> i32 {
        0
    }
    fn ignored_nth(&self, _who: u8, _i: i32) -> i32 {
        -1
    }
    fn ignore_call(&mut self, _o: i32, _who: u8) {}
    fn group_on(&self) -> bool {
        true
    }
    fn who(&self) -> u8 {
        self.who as u8
    }
    fn count(&self) -> i32 {
        i32::try_from(self.list.len()).unwrap_or(i32::MAX)
    }
    fn member(&self, k: usize) -> i32 {
        self.list[k] as i32
    }
    fn alive(&mut self, o: i32) -> bool {
        self.sim.buildings[o as usize].alive
    }
    fn active(&mut self, o: i32) -> bool {
        self.sim.buildings[o as usize].active
    }
    fn targetable(&mut self, _o: i32) -> bool {
        true
    }
    fn queued(&mut self, o: i32) -> u8 {
        u8::try_from(self.sim.buildings[o as usize].queue.items.len()).unwrap_or(u8::MAX)
    }
    fn refresh(&mut self) {}
    fn type_is_unit(&mut self, _ty: i32) -> bool {
        matches!(self.what, Queued::Unit(_))
    }
    fn type_avail(&mut self, _ty: i32) -> bool {
        match self.what {
            Queued::Unit(t) => self.sim.muster[self.who as usize].researched[t],
            Queued::Tech(_) => false,
        }
    }
    fn type_is_build(&mut self, _ty: i32) -> bool {
        false
    }
    fn build_flag4(&mut self, _ty: i32) -> bool {
        false
    }
    fn is(&mut self, o: i32, ty: i32) -> bool {
        ty == SILO && self.sim.building_ident(o as usize) == crate::build::Ident::MissileSilo
    }
    fn can_carry(&mut self, o: i32, _ty: i32) -> bool {
        match self.what {
            Queued::Unit(t) => self.sim.silo_takes(o as usize, t),
            Queued::Tech(_) => false,
        }
    }
    fn queue_up(&mut self, o: i32, _ty: i32) -> bool {
        match self.what {
            Queued::Unit(t) => self.sim.queue_up(o as usize, t).is_ok(),
            Queued::Tech(t) => self.sim.queue_tech(o as usize, t).is_ok(),
        }
    }
    fn researching(&mut self, _ty: i32) -> bool {
        match self.what {
            Queued::Unit(t) => self.sim.researching_unit(self.who, t),
            Queued::Tech(t) => self.sim.researching(self.who, t),
        }
    }
    fn is_console(&mut self) -> bool {
        false
    }
    fn refused(&mut self) {}
}
