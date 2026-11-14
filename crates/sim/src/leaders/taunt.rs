//! **The taunts** — item 4159's step VII-b2 (`docs/LEADERS.md` §9): an
//! allied human's taunt-order chat line reaches a computer through an
//! eight-slot ring (`Leader::receive_taunt@006b8bf0`), fires on its due
//! frame from the drain at the tail of `Leaders::process_all@006ed2a0`'s
//! slot step, and `Leader::process_taunt@006b8cc0` stamps it and acts:
//! for a resource request, a third of the computer's stock as a gift
//! ([`Sim::action_respond`]'s commit and exchange, `docs/LEADERS.md` §8.2).
//!
//! The leaves: `last_taunt` (`+0x354`) and `taunt_frame` (`+0x374`), by
//! sender; the ring `incoming_taunt` (`+0x394`), `incoming_taunt_who`
//! (`+0x3b4`), `incoming_taunt_frame` (`+0x3d4`) and its cursor `incoming`
//! (`+0x3f4`).

use crate::economy::Resource;
use crate::{Player, Sim};

/// `TAUNT_HELP` (`TauntRequest` 16, `rise.pdb`): due on its receiving frame.
pub const TAUNT_HELP: i32 = 16;

/// The delay of every other request: `frame + 0x3c`.
const TAUNT_DELAY: i64 = 60;

/// `process_taunt`'s refusal window after the last gift: `0x1194`.
const GIFT_WINDOW: i64 = 4500;

/// `process_taunt`'s gift floor: a stock over `0x95`.
const GIFT_FLOOR: i32 = 149;

/// The resource requests `TAUNT_FOOD` .. `TAUNT_OIL` (1..5, `rise.pdb`'s
/// `TauntRequest`) and the good each asks for.
const GOODS: [Resource; 5] = [
    Resource::Food,
    Resource::Timber,
    Resource::Wealth,
    Resource::Metal,
    Resource::Oil,
];

/// The ring's length and the stamps' width: one per leader slot.
const RING: usize = 8;

/// The leaves [`Sim::leader_taunt_leaf`] answers and the leaders line emits:
/// `last_taunt` and `taunt_frame`, which move on run1575's block 967, and
/// the ring (`incoming_taunt`, `incoming_taunt_frame`, `incoming`), written
/// on 906 and 966 and witnessed on run1575's packet at 1090, which our walk
/// holds unstaged since step VII-c computes the accept of 1027.
/// `incoming_taunt_who` is held and stays off the line: the only sender any
/// capture holds is who 0, the reset value (`docs/LEADERS.md` §9.5).
pub const TAUNT_LEAVES: [&str; 5] = [
    "last_taunt",
    "taunt_frame",
    "incoming_taunt",
    "incoming_taunt_frame",
    "incoming",
];

/// One leader's taunt leaves (`docs/LEADERS.md` §9.1). All zero at game
/// start; nothing in a match resets them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Taunts {
    /// `last_taunt` (`+0x354`), by sender: the request last processed.
    pub last_taunt: [i32; RING],
    /// `taunt_frame` (`+0x374`), by sender: the frame it was processed.
    pub taunt_frame: [i64; RING],
    /// `incoming_taunt` (`+0x394`): the ring's requests.
    pub incoming_taunt: [i32; RING],
    /// `incoming_taunt_who` (`+0x3b4`): the ring's senders, a leader `who`.
    pub incoming_taunt_who: [i32; RING],
    /// `incoming_taunt_frame` (`+0x3d4`): the ring's due frames.
    pub incoming_taunt_frame: [i64; RING],
    /// `incoming` (`+0x3f4`): the ring's write cursor, 0..7.
    pub incoming: i32,
}

impl Sim {
    /// One of [`TAUNT_LEAVES`] as the leaders line prints it: the arrays
    /// eight words, `incoming` one.
    pub fn leader_taunt_leaf(&self, who: Player, name: &str) -> Option<Vec<i64>> {
        let t = self.leaders.taunts.get(who as usize)?;
        let wide = |a: &[i32; RING]| a.iter().map(|&x| i64::from(x)).collect();
        match name {
            "last_taunt" => Some(wide(&t.last_taunt)),
            "taunt_frame" => Some(t.taunt_frame.to_vec()),
            "incoming_taunt" => Some(wide(&t.incoming_taunt)),
            "incoming_taunt_who" => Some(wide(&t.incoming_taunt_who)),
            "incoming_taunt_frame" => Some(t.incoming_taunt_frame.to_vec()),
            "incoming" => Some(vec![i64::from(t.incoming)]),
            _ => None,
        }
    }

    /// Whether `who` is a slot in play and live (`leader_flags & 2`).
    fn taunt_live(&self, who: usize) -> bool {
        who < self.leaders.taunts.len().min(RING) && !self.leaders.defeated[who]
    }

    /// **`Leader::receive_taunt@006b8bf0(r, req, s)`**: `s`'s request
    /// `req` queued on `r`'s ring. Refused unless `req > 0` (signed), `s`
    /// is live, `r` is a live computer, and `r` is `s`'s mutual ally and
    /// not `s`. A `HELP` is due this frame, every other request 60 frames
    /// on; the cursor steps on every accepted request and nothing tests
    /// occupancy.
    pub fn receive_taunt(&mut self, r: Player, req: i32, s: Player) {
        let (x, y) = (r as usize, s as usize);
        if req <= 0 || !self.taunt_live(y) || !self.taunt_live(x) {
            return;
        }
        if self.cities.nation[x].human || !self.is_ally(r, s) || x == y {
            return;
        }
        let due = if req == TAUNT_HELP {
            self.frame
        } else {
            self.frame + TAUNT_DELAY
        };
        let t = &mut self.leaders.taunts[x];
        let i = (t.incoming & 7) as usize;
        t.incoming_taunt[i] = req;
        t.incoming_taunt_who[i] = i32::from(s);
        t.incoming_taunt_frame[i] = due;
        t.incoming = (t.incoming + 1) & 7;
    }

    /// **The drain**, `Leaders::process_all@006ed2a0`'s last act of a live
    /// slot's step, off frame 0: every ring slot due this frame, in slot
    /// order, to [`Sim::process_taunt`]. Nothing is cleared.
    pub(crate) fn drain_taunts(&mut self, r: Player) {
        let x = r as usize;
        if self.frame == 0 || !self.taunt_live(x) {
            return;
        }
        for k in 0..RING {
            let t = &self.leaders.taunts[x];
            if t.incoming_taunt_frame[k] == self.frame {
                let (req, s) = (t.incoming_taunt[k], t.incoming_taunt_who[k]);
                self.process_taunt(r, req, s as Player);
            }
        }
    }

    /// **`Leader::process_taunt@006b8cc0(r, req, s)`**: on a live computer
    /// `r` still allied both ways with `s` (re-tested now), `last_taunt[s]
    /// = req` and `taunt_frame[s]` this frame, before the request's range
    /// test. Then the goods arm, for `TAUNT_FOOD` .. `TAUNT_OIL`: both
    /// leaders must have the good; under unlocked teams (or `r` neutral) a
    /// gift inside 4500 frames of `gift_stamp[s]` is refused; a stock over
    /// 149 is given a third of — `gift_stamp[s]` stamped first under the
    /// same lock test, then [`Sim::action_clear_all`],
    /// [`Sim::action_offer`] and [`Sim::action_respond`]`(r, s, 1)`: a pure
    /// gift, executed at once. The refusals' chat lines are the
    /// interface's.
    ///
    /// SEAM: `TAUNT_NEED` and the production requests (6..15, the AI's
    /// weights and `TAUNT_DEFEND`'s draw) are not carried; their head
    /// stores are. `LeaderData::is_neutral` is false: a neutral is team 8
    /// at team style 7, and the crate keeps no lobby team.
    pub fn process_taunt(&mut self, r: Player, req: i32, s: Player) {
        let (x, y) = (r as usize, s as usize);
        if !self.taunt_live(x) || self.cities.nation[x].human || y >= RING || x == y {
            return;
        }
        if self.diplos(x, y) != 2 || self.diplos(y, x) != 2 {
            return;
        }
        let frame = self.frame;
        self.leaders.taunts[x].last_taunt[y] = req;
        self.leaders.taunts[x].taunt_frame[y] = frame;
        let Some(&good) = usize::try_from(req - 1).ok().and_then(|i| GOODS.get(i)) else {
            return;
        };
        let g = good.index();
        if !self.economy.holdings[x].available[g] || !self.economy.holdings[y].available[g] {
            return;
        }
        let unlocked = matches!(self.ai.lobby.team_style, 0 | 8 | 11);
        let last = self.leaders.tribute[x].gift_stamp[y];
        if unlocked && last != 0 && frame - last < GIFT_WINDOW {
            return;
        }
        let stock = self.economy.ledgers[x].bucket[g];
        if stock <= GIFT_FLOOR {
            return;
        }
        if unlocked {
            self.leaders.tribute[x].gift_stamp[y] = frame;
        }
        self.action_clear_all(r, s);
        self.action_offer(r, s, good, stock / 3);
        self.action_respond(r, s, 1);
    }
}

#[cfg(test)]
mod tests;
