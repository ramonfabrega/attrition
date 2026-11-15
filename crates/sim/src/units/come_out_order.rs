//! **A garrisoned unit's own order to leave** (item 4280,
//! `docs/CLIENT_INPUT.md` §26): `CommandPackage::process_come_out@009465d0`
//! and `Unit::action_come_out@005e20b0`, the processor of command `0x31`
//! that the building panel's garrisoned-unit icon issues
//! (`Options::exec@007188c0` case `0xaa`, `UnitOut::issue_come_out
//! @00607be0`). The walk out itself is [`Sim::come_out`].

use crate::{Player, Sim};

impl Sim {
    /// **`CommandPackage::process_come_out@009465d0`**: `units[who][o]`,
    /// when it is active (`+8 & 1`) and its `uid` (`+0x30`) is the
    /// command's, takes [`Sim::action_come_out`]. Any other unit, a dead
    /// one or a reused number, is left alone. The command carries no
    /// group. `true` when the action ran.
    pub fn process_come_out(&mut self, who: Player, o: i16, uid: u16) -> bool {
        let Some(u) = self.unit_by_o(who, o) else {
            return false;
        };
        if self.units[u].uid != uid {
            return false;
        }
        self.action_come_out(u);
        true
    }

    /// **`Unit::action_come_out@005e20b0`**: the unit's orders are closed
    /// as `Group::action_return` closes them (`unit_masks &= ~0x4000000`,
    /// the path stack's length `+0xc0` to 0, `close_orders`,
    /// `clear_partial_path`, `update_action`), and then `come_out(this, 0)`
    /// ([`Sim::come_out`]), which does nothing for a unit that is not
    /// inside.
    ///
    /// SEAM: the merchant arm. A unit of type index `0x34` or `0x35`
    /// inside a plain building of its own owner's that `is(0x1a4)` sets
    /// the leader's `0x2000000` bit and hands its trade route (`+0x74`,
    /// `+0x9c` of its order record) down its chain before it leaves. This
    /// crate does not carry it; such a unit leaves without it.
    pub fn action_come_out(&mut self, u: usize) {
        self.units[u].path.clear();
        self.close_orders(u);
        self.clear_partial_path(u);
        self.update_action(u);
        self.come_out(u);
    }
}
