//! The walk half of `Unit::add_to_army@005f7740` (`docs/ARMY.md` §4.3):
//! `go_to_unit` marks the squad and walks it to the army's first member, and
//! `go_to` finds the spot and moves the squad there as a pushed group. Moved
//! here from `ai/army.rs` by item 4391, whose listed-joiner arm grew that
//! file past the structure pin.

use crate::Sim;
use crate::movement::Angle;
use crate::orders::{MoveKind, QueuePos};
use crate::world::{Pos, vector_dist};

impl Sim {
    /// `Unit::go_to_unit(o, who)@005f78c0` — mark the squad, then walk it
    /// to another unit if it is far enough away (§4.3).
    ///
    /// The mask walk runs from the **captain** and down the `o_down`
    /// chain, and it runs whatever the distance says; the move is
    /// `vector_dist(target − me) > 0x480` alone. The bit it sets is
    /// `unit_masks & 4` (`5f7933`), which §6.6 step 6 of
    /// `docs/GROUPS.md` reads as "no `GroupMoveOrder`" — a seam here,
    /// because [`Sim::go_to`]'s group has no army and this crate's own
    /// gate refuses one for that reason already.
    pub(super) fn go_to_unit(&mut self, u: usize, target: usize) {
        // The mark is **unconditional** and it is the whole squad's: the
        // loop at `5f78ec` starts at this unit's captain and walks
        // `o_down`, setting `unit_masks |= 4` on each, and it runs above
        // the distance test — so a squad already close enough not to walk
        // is marked all the same. [`sim::Unit::in_danger`].
        for f in self.squad_of(self.captain_of(u)) {
            self.units[f].in_danger = true;
        }
        let to = self.units[target].pos;
        let d = vector_dist(to.x - self.units[u].pos.x, to.y - self.units[u].pos.y);
        if d > 0x480 {
            self.go_to(u, to, MoveKind::AttackTo, 0, 0x300);
        }
    }

    /// `Unit::go_to(x, y, orders, min, max)@005f7a50` — a spot near a
    /// point, and the unit's own squad walked to it as a group (§4.3).
    ///
    /// `find_nearby_spot` is asked with the caller's radii, **step 0** (so
    /// the rings are an eighth of the span apart) and the bias angle
    /// `0x55555555`, a third of a turn; a refusal issues nothing at all.
    /// The group is built on the stack from this unit — `Group::add` takes
    /// the whole squad — pushed with `force = 1`, and moved with
    /// `QUEUE_NEW`, `set_angle 0`, `angle 0`, `action 0`, `form/width −1`.
    pub(super) fn go_to(&mut self, u: usize, to: Pos, kind: MoveKind, min: i32, max: i32) {
        let Some(spot) = self.find_nearby_spot_squad(u, to, min, max, 0, Angle(0x5555_5555)) else {
            return;
        };
        let mut g = crate::groups::group::Group::stack(self.units[u].owner);
        self.group_add(&mut g, u);
        if !self.push_group(&mut g, true) {
            return;
        }
        self.group_action_move_to(&g, spot, QueuePos::New, false, Angle(0), kind, false);
    }
}
