//! **A player who quits** — `Player::resign@006edcb0` → `Player::leave_game(6)`
//! → `Leader::defeat@006ecb00(6, −1, 0)`, the resign with no captor (item
//! 3841, `docs/GOLDEN.md` §152.15).
//!
//! A capture ends by the human quitting through the menu, and the trace
//! dates it: `CommandManager::issue_quit` is the first function the quit
//! frame enters, before any unit acts, and the closing dump is written
//! later in that frame (run1468b, frame 2600: `issue_quit`, `Player::quit`,
//! `resign`, `leave_game`, `Leader::defeat`, `defeat_by`, `Armies::
//! leader_defeated`, then the units' `do_idle` draws).
//!
//! What the defeat does to the loser's units (`Leader::defeat`, the walk
//! over `objects[who]` after the building loop): every live unit asks its
//! vslot `+0xc0`, **`UnitData::is_plane`** — a plane `die`s through `+0x158`
//! with `(0, −1, 0)`, `dtype` 0, and every other unit has its orders
//! cleared (`Unit::clear_orders`). Both then lose `unit_masks & 0x40000`,
//! which this crate reads off the leader ([`Sim::unit_ai_bit`]).
//!
//! `defeat_by(−1, 6)` takes neither of its arms here: the captor arm wants
//! `param_1 >= 0`, and the blow-up arm a second human alive (`local_1c`), so
//! the loser's buildings and land units stand — run1468's 2601 holds who=0's
//! `0/0`..`0/13` and its buildings `2000`..`2006`, and not its V2 Rocket.
//!
//! **The other units' `clear_orders`** (item 4204). Carried with the quit
//! at every closing frame's head it moved 54 closing states the wrong way
//! (item 3841): a human unit walking on the quit frame takes its step in
//! theirs and stopped at once in ours. The step is the frame's body: 110
//! of the 124 closings walked whole quit past it, from a game already over
//! (each capture's trace dates the resign's sound, `docs/LEADERS.md`
//! §10.2), so their walkers move before the clear in theirs too. The 13
//! that quit at the head (the melees' 2600, the 24,000s) stop at once on
//! both sides: their closing blocks lose the held orders and the walker's
//! step (2601's `0/1`..`0/7`, run80's 24001, run82's shutdown tear).
//!
//! SEAM: the building loop's `Build::clean_queue` on each active building
//! (a refund; no closing dump on disk holds a quitter's queue), the
//! achievement event, the messages and the tribute arm for allies
//! (`semaphore & 0x44 == 4`). `Game::check_victory` is carried since item
//! 4189 ([`Sim::check_victory`]).

use crate::combat::Obj;
use crate::{Player, Sim};

impl Sim {
    /// `Leader::defeat(6, −1, 0)` for `who`, the human who quits: the leader
    /// is defeated, its armies let go and its planes die.
    ///
    /// `Player::leave_game@006ee010` first: a leader whose capital clock runs
    /// is `defeat(1, holder)` instead under elimination 1 (the holder
    /// `find_capital(−1, −1)` names, itself when none), and `defeat(1, self)`
    /// under any other; else `defeat(6, −1)` (`docs/LEADERS.md` §10.2). The
    /// captor stamp runs ahead of the defeated guard either way.
    ///
    /// SEAM: under a running clock the captor arm of `defeat_by` (the
    /// cities handed to the holder) is not carried here. SEAM scan:
    /// `grep -h '^ *defeat_type 1$' gamelog-*.txt | wc -l` — 0 over the
    /// 1,196 kept dumps: no capture resigns under a running clock.
    pub fn resign(&mut self, who: Player) {
        if (who as usize) >= self.leaders.defeated.len() {
            return;
        }
        let (ty, by) = if self.leader_endgame(who).lost_capital_timer != 0 {
            let by = if self.ai.lobby.elimination == 1 {
                self.capital_holder(who)
            } else {
                who
            };
            (crate::leaders::endgame::DEFEAT_ELIMINATION, Some(by))
        } else {
            (crate::leaders::endgame::DEFEAT_RESIGN, None)
        };
        self.defeat_by_stamp(who, by);
        if self.leaders.defeated[who as usize] {
            return;
        }
        self.leaders.defeated[who as usize] = true;
        self.note_defeat_type(who, ty);
        self.armies_leader_defeated(who);
        for u in 0..self.units.len() {
            if self.units[u].owner != who || !self.units[u].alive() {
                continue;
            }
            if self.is_plane(u) {
                self.die_quietly(u);
            } else {
                // Item 4204: every other unit's orders go
                // (`Unit::clear_orders@005e3860`).
                self.clear_orders(u);
            }
        }
        // `Leader::defeat@006ecb00`'s tail (item 4189): with the human gone
        // and one computer left, the game is decided for it at once.
        self.check_victory();
    }

    /// `Unit::die(0, −1, 0)` → `Object::die`: the close through vslot
    /// `+0x150` with `dtype` 0 — no death animation, no draw and no death
    /// object (`close_unit`'s first test) — and the slot held.
    pub(crate) fn die_quietly(&mut self, v: usize) {
        let frame = self.frame;
        self.units[v].health = 0;
        self.close_count(v);
        self.relink_squad(v);
        self.close_unit(v, 0, frame);
        self.kill_contents(v);
        self.kremlin_spy_closed(v);
        self.close_dead_orders(v);
        self.hold_dead_slot(v);
        self.close_supply_in(v, false);
        self.forget(Obj::Unit(v));
        self.clear_decoy_at_close(v);
    }
}
