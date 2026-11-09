//! **The `swarm_around` command** — a player's right-click on a friendly
//! building: `CommandPackage::process_swarm_around@00949970`'s one call,
//! `Group::action_swarm_around(ox, whom, queued, orders, 1)@0070fbe0`
//! (`docs/GOLDEN.md` §29).
//!
//! It is the command a player's **repair** travels by. `Console::
//! execute_at_cursor@007c6630` (a right-click on a damaged, finished
//! building of one's own or an ally's, with citizens selected) and
//! `Options::picked_spot@00721c40` (the Repair pick) both pass `REPAIR`
//! through `GroupOut::issue_swarm_around@0070afc0` →
//! `CommandManager::issue_swarm_around@009416b0`, a 17-byte command of
//! type 6. The `repair` command (`0x10`, `process_repair@00948cb0` →
//! `Group::action_repair@007020c0`) is issued by nothing a player does.
//! The same entry carries `BUILD_AT` for a right-click on an unfinished
//! site; the member arm is [`Sim::group_action_swarm_around`]'s either
//! way.

use crate::Sim;
use crate::group::Group;
use crate::orders::{Body, QueuePos, index};

impl Sim {
    /// `process_swarm_around@00949970` → `Group::action_swarm_around(o,
    /// who, queued, orders, 1)`: the group's citizens each take a
    /// `MOVE_TO` approach to the ring round building `b` and, behind it,
    /// the order `orders` names with the action bit. `REPAIR` (13) is a
    /// `RepairOrder` and `BUILD_AT` (6) a `BuildOrder`; the approach is a
    /// `MOVE_TO` for every repair: the swarm's `local_40` is 1 and is
    /// rewritten only on its `BUILD_AT` arm.
    ///
    /// Returns whether the command reached the group. Any other `orders`
    /// gives nothing here; in the original a citizen member reaches
    /// `Error::report("ILLEGAL SWARM AROUND ORDER")` (`docs/ORDERS.md`
    /// §5.4), and no capture issues one.
    pub fn group_swarm_around(
        &mut self,
        g: &Group,
        b: usize,
        queued: QueuePos,
        orders: u8,
    ) -> bool {
        let body = match orders {
            index::REPAIR => Body::Repair(b),
            index::BUILD_AT => Body::Build(b),
            _ => return false,
        };
        self.group_action_swarm_around(g, b, queued, body, true);
        true
    }
}
