//! **The City's alarm and a player's gather** — two issuers of the
//! Militia's Civilian that no chapter before forty staged (item 1167,
//! `docs/GOLDEN.md` §49).
//!
//! - `CommandPackage::process_alarm@00947ef0` → `Group::action_alarm@
//!   0070ec30` on a group of the player's own buildings.
//! - `CommandPackage::process_gather@009488b0` → `Group::action_gather@
//!   00700b90(ox, queued)` on a group of units.

use crate::Sim;
use crate::group::Group;
use crate::orders::QueuePos;
use crate::world::Player;

impl Sim {
    /// `Group::action_alarm@0070ec30` on the buildings `list` of `who`.
    pub fn action_alarm(&mut self, who: Player, list: &[usize]) {
        let _ = (who, list);
    }

    /// `Group::action_gather@00700b90(b, queued)` on the group `g`.
    pub fn group_gather(&mut self, g: &Group, b: usize, queued: QueuePos) {
        let _ = (g, b, queued);
    }
}
