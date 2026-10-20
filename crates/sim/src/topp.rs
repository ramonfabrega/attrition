//! The head of `Unit::target_opportunity@005fffc0` as steps over a view
//! (item 3348): the diplomacy gate, the group hand-off and the captain walk
//! (`5fffe1`..`600106`), then `valid_target` and the hold-fire return
//! (`600108`). [`target_opportunity_head`] is the original's control flow in
//! its order; [`TargetOppView`] is what it reads and the callees it makes,
//! which [`SimTopp`] answers from the crate's [`Sim`] and
//! `sweep/unit_target_opportunity.rs` from the emulated original's windows.
//! Everything past `600108` (the flee arm, the action tests, the spell casts,
//! the retaliation) stays in [`Sim::target_opportunity_in`], not swept.
//!
//! **The gate** (`5ffff3`..`600017`): return if the attacker's owner is this
//! unit's team (`leaders[owner]+8`), or if `leaders[owner]+0x74[p2]` and
//! `leadersc[p2]+0x74[team]` are both non-zero (the two directions of the
//! treaty tables). The crate keeps one `at_war` matrix and one player per
//! team, so the view answers each table as "not at war" in its direction.

use crate::Sim;
use crate::combat::{Obj, Stance};

/// What the head reads of the unit and the tables, and the callees it makes.
pub trait TargetOppView {
    /// A unit; `T` the attacker (`param_1`, `param_2`).
    type U: Copy;
    type T: Copy;
    /// `((UnitData*)u)->field_0x9`.
    fn owner(&self, u: Self::U) -> i32;
    /// `leaders.list[owner].field_0x8`.
    fn team(&self, owner: i32) -> i32;
    /// `param_2`: the attacker's owner.
    fn target_owner(&self, t: Self::T) -> i32;
    /// `leaders.list[a].+0x74[b] != 0`.
    fn treaty(&self, a: i32, b: i32) -> bool;
    /// `leadersc->list[p2].+0x74[team] != 0`.
    fn treaty_c(&self, p2: i32, team: i32) -> bool;
    /// `group >= 0` and `type +0x2c8 & 0x10000 == 0`: the test before the call.
    fn group_noncombat(&mut self, u: Self::U) -> bool;
    /// `GroupData::member(group, short +0xa, owner, 1)`.
    fn group_member(&mut self, u: Self::U) -> bool;
    /// `Group::target_opportunity(group, param_1, param_2, short +0xa, 0)`.
    fn group_handoff(&mut self, u: Self::U, t: Self::T);
    /// `vtable[0xe8]` (`is_captain`).
    fn is_captain(&mut self, u: Self::U) -> bool;
    /// `objects[owner][o_up]->vtable[0xa8]()`: the next unit up; `None` where
    /// the crate has none (the original assumes one).
    fn up(&mut self, u: Self::U) -> Option<Self::U>;
    /// `Object::valid_target(u, param_1, param_2, 0)`.
    fn valid_target(&mut self, u: Self::U, t: Self::T) -> bool;
    /// `vtable[0xf4]()`: the stance; 5 is hold-fire.
    fn stance(&mut self, u: Self::U) -> i32;
}

/// How the head ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Head<U> {
    /// A `return` before the body (gate, `valid_target`, hold-fire).
    Stop,
    /// `Group::target_opportunity` took the hit.
    Handed,
    /// Past `600108` with this unit as `this`.
    Go(U),
}

/// The control flow from `5fffe1` to `60010b`. `from_group` is `param_3 != 0`.
pub fn target_opportunity_head<V: TargetOppView>(
    v: &mut V,
    unit: V::U,
    t: V::T,
    from_group: bool,
) -> Head<V::U> {
    let p2 = v.target_owner(t);
    let (mut u, mut from_group) = (unit, from_group);
    loop {
        let owner = v.owner(u);
        let team = v.team(owner);
        if p2 == team {
            return Head::Stop;
        }
        if v.treaty(owner, p2) && v.treaty_c(p2, team) {
            return Head::Stop;
        }
        if !from_group && v.group_noncombat(u) && v.group_member(u) {
            v.group_handoff(u, t);
            return Head::Handed;
        }
        if v.is_captain(u) {
            break;
        }
        from_group = false;
        match v.up(u) {
            Some(n) => u = n,
            None => return Head::Go(u),
        }
    }
    if !v.valid_target(u, t) || v.stance(u) == 5 {
        return Head::Stop;
    }
    Head::Go(u)
}

fn player(p: i32) -> crate::Player {
    crate::Player::try_from(p).unwrap_or(crate::Player::MAX)
}

/// [`TargetOppView`] on the crate's state.
pub struct SimTopp<'a> {
    pub(crate) sim: &'a mut Sim,
}

impl TargetOppView for SimTopp<'_> {
    type U = usize;
    type T = Obj;
    fn owner(&self, u: usize) -> i32 {
        i32::from(self.sim.units[u].owner)
    }
    fn team(&self, owner: i32) -> i32 {
        owner
    }
    fn target_owner(&self, t: Obj) -> i32 {
        i32::from(self.sim.owner_of(t))
    }
    fn treaty(&self, a: i32, b: i32) -> bool {
        !self.sim.at_war_with(player(a), player(b))
    }
    fn treaty_c(&self, p2: i32, team: i32) -> bool {
        !self.sim.at_war_with(player(p2), player(team))
    }
    fn group_noncombat(&mut self, u: usize) -> bool {
        !self.sim.profile(Obj::Unit(u)).combat_role
            && self.sim.units[u].on_map
            && self.sim.seat_of(u).is_some()
    }
    fn group_member(&mut self, u: usize) -> bool {
        self.sim
            .seat_of(u)
            .is_some_and(|seat| self.sim.seat_list(seat).contains(&u))
    }
    fn group_handoff(&mut self, u: usize, t: Obj) {
        if let Some(seat) = self.sim.seat_of(u) {
            self.sim.group_target_opportunity(seat, t, u, false);
        }
    }
    fn is_captain(&mut self, u: usize) -> bool {
        self.sim.units[u].captain
    }
    fn up(&mut self, u: usize) -> Option<usize> {
        match self.sim.units[u].o_up {
            Some(c) if c != u && self.sim.units[c].alive() => Some(c),
            _ => None,
        }
    }
    fn valid_target(&mut self, u: usize, t: Obj) -> bool {
        self.sim.valid_target(Obj::Unit(u), t)
    }
    fn stance(&mut self, u: usize) -> i32 {
        if self.sim.units[u].combat.stance == Stance::HoldFire {
            5
        } else {
            0
        }
    }
}
