//! **`Army::process@006f93d0`'s steps**, as a walk over an
//! [`ArmyProcessView`]: [`crate::Sim::army_process`] is this walk over the
//! army's slot, and the sweep's port is this walk over the emulated
//! original's windows (item 3399).
//!
//! The function is the per-frame driver and every arm of it is another
//! method's call, so the view's methods are those calls (answers, and a
//! record of their order, in the sweep) and the reads the driver itself
//! makes of the army, the leader, the city list, the army list and the
//! target object.
//!
//! * `bypass` (the original's argument) skips the cadence: otherwise the
//!   casters' turn runs on `frame - 30 + s` a multiple of 128, and the tick
//!   only on `frame + s` a multiple of 256.
//! * The tick: normalize; an army with no standard line (and a live slot, and
//!   not mustering) sends the survivors to the first live city and closes; a
//!   human's ping holds the AI off; a thin army merges into a stronger one of
//!   its region; an army neither moving nor engaged moves its muster point
//!   toward the target; then the dispatch, each arm independent.
//!
//! Not covered by the sweep: the merge's member copy (the original walks the
//! army's groups table and the units table; the crate's army holds one flat
//! list), which [`ArmyProcessView::merge_into`] stands for.

use crate::world::Pos;

/// `Army::count`'s `COUNT_NON_DECOY_TYPE` type for a General, a Spy and a
/// Scout (the original's constants `0x36`, `0x3a`, `0x45`).
pub const COUNT_GENERAL: i32 = 0x36;
pub const COUNT_SPY: i32 = 0x3a;
pub const COUNT_SCOUT: i32 = 0x45;

/// `Army::status` bits the driver tests.
pub const MUSTERING: i32 = 0x01;
pub const MARCHING: i32 = 0x02;
pub const NO_TARGET: i32 = 0x08;
pub const FORMING: i32 = 0x10;
pub const DEFENDING: i32 = 0x20;
pub const TRANSPORTING: i32 = 0x40;

/// The order `send_here` is given: the original's `1` and `2`.
pub const MOVE_TO: i32 = 1;
pub const ATTACK_TO: i32 = 2;

/// Another army of the same leader, as the merge reads it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Peer {
    pub valid: bool,
    pub navy: bool,
    pub reg: i32,
    pub num_captains: i32,
    pub num_standard: i32,
    pub num_decoys: i32,
}

/// What `Army::process` asks of the army, its leader and its callees.
pub trait ArmyProcessView {
    /// The word at `+0x28`.
    fn human_frame(&self) -> i32;
    fn set_human_frame(&mut self, v: i32);
    /// `GameAccess::game->frame`.
    fn frame(&self) -> i32;
    /// The `s` of the cadence: `(army + 2 * leader) * 2`.
    fn phase(&self) -> i32;
    /// The leader's flag `0x40`.
    fn defeated(&self) -> bool;
    /// The word at `+0x14`, `+0x18`, `+0x1c`.
    fn num_captains(&self) -> i32;
    fn num_standard(&self) -> i32;
    fn num_decoys(&self) -> i32;
    /// The short at `+0` (a live slot).
    fn valid(&self) -> bool;
    /// The word at `+0x4`.
    fn status(&self) -> i32;
    fn set_status(&mut self, s: i32);
    /// The word at `+0x24`.
    fn navy(&self) -> bool;
    /// The word at `+0x8`.
    fn reg(&self) -> i32;
    /// The point at `+0x38`/`+0x3c`.
    fn ping_pos(&self) -> Pos;
    /// The muster cell, `(x, y)`.
    fn muster(&self) -> (i32, i32);
    fn set_muster(&mut self, x: i32, y: i32);
    /// Whether the target (`+0x30`, `+0x34`) is set, and the player half.
    fn has_target(&self) -> bool;
    fn target_owner(&self) -> i32;
    /// The target object's cell, `(x, y)`.
    fn target_cell(&mut self) -> (i32, i32);

    /// The leader's city list: its length, and the `i`th city's liveness and
    /// point. `scan` is the original's cursor at `+0x20`.
    fn city_count(&self) -> i32;
    fn set_scan(&mut self, i: i32);
    fn city_alive(&mut self, i: i32) -> bool;
    fn city_pos(&mut self, i: i32) -> Pos;
    /// The leader's `k`th army, `k < 16`.
    fn peer(&mut self, k: usize) -> Peer;

    // The callees.
    fn normalize(&mut self);
    /// `Army::count(COUNT_NON_DECOY_TYPE, ty)`.
    fn count_type(&mut self, ty: i32) -> i32;
    fn use_generals(&mut self);
    fn use_spies(&mut self);
    fn use_scouts(&mut self);
    fn send_here(&mut self, p: Pos, kind: i32);
    fn close(&mut self);
    /// Every member added, last first, to the peer `k`.
    fn merge_into(&mut self, k: usize);
    fn is_moving(&mut self) -> bool;
    fn is_engaged(&mut self) -> bool;
    fn is_enemy(&mut self, owner: i32) -> bool;
    fn find_muster_spot(&mut self);
    fn set_stance(&mut self, code: i32);
    fn do_mustering(&mut self);
    fn do_defending(&mut self);
    fn do_marching(&mut self);
    /// `ran`: a dispatch arm ran before it (the original pushes its stale
    /// `ecx` as an argument it never reads).
    fn do_forming(&mut self, ran: bool);
    fn engagement(&mut self);
    fn do_transporting(&mut self);
}

/// `Army::process@006f93d0`.
pub fn army_process<V: ArmyProcessView>(v: &mut V, bypass: bool) {
    let h = v.human_frame();
    if h != 0 {
        v.set_human_frame(h - 1);
    }
    if !bypass {
        let s = v.phase();
        if v.frame().wrapping_sub(30).wrapping_add(s) % 128 == 0 {
            if v.defeated() {
                return;
            }
            v.normalize();
            if v.num_captains() == 0 {
                return;
            }
            if v.num_standard() != 0 && v.count_type(COUNT_GENERAL) != 0 {
                v.use_generals();
            }
            if v.count_type(COUNT_SPY) != 0 {
                v.use_spies();
            }
            if v.count_type(COUNT_SCOUT) != 0 {
                v.use_scouts();
            }
        }
        if v.frame().wrapping_add(s) % 256 != 0 {
            return;
        }
    }
    if v.defeated() {
        return;
    }
    tick(v);
}

fn tick<V: ArmyProcessView>(v: &mut V) {
    v.normalize();
    // 1. Disband: the survivors walk home, then the slot is freed.
    if v.num_standard() < 1 && v.valid() && v.status() & MUSTERING == 0 {
        v.set_scan(0);
        let n = v.city_count();
        if n > 0 {
            let mut i = 0;
            while !v.city_alive(i) {
                i += 1;
                v.set_scan(i);
                if v.city_count() <= i {
                    v.close();
                    return;
                }
            }
            let p = v.city_pos(i);
            v.send_here(p, MOVE_TO);
        }
        v.close();
        return;
    }
    // 2. A human's ping: the AI stands aside while the countdown runs.
    if v.human_frame() != 0 {
        let p = v.ping_pos();
        v.send_here(p, ATTACK_TO);
        return;
    }
    // 3. Merge into a stronger army of the same region.
    if v.num_standard() < (v.num_captains() - v.num_decoys()) / 2 {
        for k in 0..16 {
            let b = v.peer(k);
            if b.valid
                && !b.navy
                && b.reg == v.reg()
                && b.num_standard > 4
                && (b.num_captains - b.num_decoys) / 2 <= b.num_standard
            {
                v.merge_into(k);
                v.close();
                return;
            }
        }
    }
    // 4. The muster point walks halfway to the target.
    if !v.is_moving() && !v.is_engaged() {
        let st = v.status() & !0x12;
        v.set_status(st);
        if v.has_target() {
            let owner = v.target_owner();
            if !v.is_enemy(owner) || v.navy() {
                v.find_muster_spot();
            } else {
                let (tx, ty) = v.target_cell();
                let (mx, my) = v.muster();
                v.set_muster((tx + mx) / 2, (ty + my) / 2);
            }
            let st = v.status();
            v.set_status(st | 0x12);
        }
    }
    // 5. Nothing but the forming and no-target bits → marching.
    let st = v.status();
    if st & (FORMING | NO_TARGET) == st {
        v.set_status(MARCHING);
    }
    // 6. Dispatch, each `if` independent.
    let mut ran = false;
    if v.status() & MUSTERING != 0 {
        v.set_stance(1);
        v.do_mustering();
        ran = true;
    }
    if v.status() & DEFENDING != 0 {
        v.set_stance(1);
        v.do_defending();
        ran = true;
    }
    if v.status() & MARCHING != 0 {
        v.set_stance(0);
        v.do_marching();
        ran = true;
    }
    if v.status() & FORMING != 0 {
        v.do_forming(ran);
    } else if v.is_engaged() {
        v.engagement();
    }
    if v.status() & TRANSPORTING != 0 {
        v.do_transporting();
    }
    if v.navy() {
        v.set_stance(3);
    }
}
