//! **The end of the game** — item 4185, step V of the leaders line
//! (`docs/LEADERS.md` §10): `defeated_by`, `defeat_stamp`, `defeat_type`,
//! `victory_type` and the wonder clock, `wonderwin_stamp` and
//! `wonderwin_timer`.
//!
//! `Leader::defeat@006ecb00` opens with `Leader::defeat_by@006d1c80`, whose
//! head is the captor stamp ([`Sim::defeat_by_stamp`]); `Sim::defeat`
//! (`crate::cities::city`) and [`Sim::resign`] call it ahead of their own
//! guard. `Leader::victory@006ec9b0` is [`Sim::victory`], with its cascade;
//! the territory arm of `GameDaemon::process_victory@00730ef0`
//! (`crate::territory::victory`) is its plain-game caller, and
//! [`Sim::process_victory_wonder`] is the wonder arm of the same function.
//!
//! All six are printed by `LeaderData::log_data@006e5110`, so
//! `rondata::diff::leader` compares them on every dump that prints the
//! record. The lost capital's three (§11) are [`crate::leaders::capital`].

use crate::{Player, Sim};

/// `map_sizes` record 3's `+0x144`, the Standard map size every
/// `Game::*_timer` and `Game::retake_capital@00594530` divides by
/// (`docs/MAPGEN.md` §36; `rules.xml`'s `mapsizes`, which
/// `rondata::mapgen::standard_map_size` reads off the install).
pub const STANDARD_MAP_SIZE: i32 = 70;

/// `defeat_type` of a resign (`Player::leave_game@006ee010`, §10.2).
pub const DEFEAT_RESIGN: i32 = 6;
/// `defeat_type` of the victory cascade (§10.2).
pub const DEFEAT_CASCADE: i32 = 5;
/// `defeat_type` of the capital countdown, `Leader::process_elimination`
/// (§10.2), and of a resign under a running capital timer.
pub const DEFEAT_ELIMINATION: i32 = 1;
/// `defeat_type` of `Leader::lost_a_city`'s last-city arm (§10.2).
pub const DEFEAT_CONQUEST: i32 = 0;

/// `victory_type` of the wonder arm (§10.2).
pub const VICTORY_WONDER: i32 = 1;
/// `victory_type` of the territory arm (§10.2).
pub const VICTORY_TERRITORY: i32 = 2;

/// One leader's §10 and §11 words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EndGame {
    /// `defeated_by` (`+0x10`): the captor's slot, −1 when none.
    pub defeated_by: i32,
    /// `defeat_stamp` (`+0x6924`): the frame of the last `defeat_by` entry
    /// while `defeated_by` was −1.
    pub defeat_stamp: i64,
    /// `defeat_type` (`+0x7dc`): the first defeat's type, write-once.
    pub defeat_type: i32,
    /// `victory_type` (`+0x7d8`).
    pub victory_type: i32,
    /// `leader_flags & 0x20`: won. `Leader::victory`'s own guard reads it.
    pub won: bool,
    /// `wonderwin_stamp` (`+0x448`): the start frame while running, minus
    /// the banked time while stopped.
    pub wonderwin_stamp: i64,
    /// `wonderwin_timer` (`+0x44c`): the wonder clock's running flag.
    pub wonderwin_timer: i32,
    /// `lost_capital_stamp` (`+0x414`, §11): the start frame while the
    /// timer runs, a debt (`≤ 0`) while it is stopped.
    pub lost_capital_stamp: i64,
    /// `lost_capital_timer` (`+0x418`, §11): 0 or 1.
    pub lost_capital_timer: i32,
    /// `lost_capital_modifier` (`+0x41c`, §11): 8.8, `0x100` = ×1; kept
    /// stale after a stop.
    pub lost_capital_modifier: i32,
}

impl Default for EndGame {
    /// `Leader::init@006e3930`'s image: `defeated_by` −1, the rest 0.
    fn default() -> Self {
        Self {
            defeated_by: -1,
            defeat_stamp: 0,
            defeat_type: 0,
            victory_type: 0,
            won: false,
            wonderwin_stamp: 0,
            wonderwin_timer: 0,
            lost_capital_stamp: 0,
            lost_capital_timer: 0,
            lost_capital_modifier: 0,
        }
    }
}

/// The leaves [`Sim::leader_endgame_leaf`] answers, in the record's order.
pub const ENDGAME_LEAVES: [&str; 9] = [
    "defeated_by",
    "lost_capital_stamp",
    "lost_capital_timer",
    "lost_capital_modifier",
    "wonderwin_stamp",
    "wonderwin_timer",
    "victory_type",
    "defeat_type",
    "defeat_stamp",
];

/// Of [`ENDGAME_LEAVES`], the two the leaders line emits unheld although
/// the crate holds them: the record diff parts each on a closing block
/// whose human quit, which the window harness does not replay (run609's
/// 5639, run667's 17380; parked 4185a).
pub const ENDGAME_OFF_THE_LINE: [&str; 2] = ["victory_type", "defeat_type"];

/// `max(1, (S/2 + xs·t) / S)` for `t ≥ 1`, else 0: the map-size scaling
/// `Game::popwin_timer@005944b0`, `Game::wonder_timer@005944f0` and
/// `Game::retake_capital@00594530` share, `xs` the map's width in cells
/// (`World+0x0`) and `S` [`STANDARD_MAP_SIZE`]. Every division truncates.
pub fn scaled_timer(t: i32, xs: i32) -> i64 {
    if t < 1 {
        return 0;
    }
    let s = i64::from(STANDARD_MAP_SIZE);
    ((s / 2 + i64::from(xs) * i64::from(t)) / s).max(1)
}

impl Sim {
    /// One leader's §10 and §11 words, or the init image for a slot this
    /// crate does not seat.
    pub fn leader_endgame(&self, who: Player) -> EndGame {
        self.leaders
            .endgame
            .get(usize::from(who))
            .cloned()
            .unwrap_or_default()
    }

    /// One of [`ENDGAME_LEAVES`] as the record holds it.
    pub fn leader_endgame_leaf(&self, who: Player, name: &str) -> Option<i64> {
        let e = self.leader_endgame(who);
        Some(match name {
            "defeated_by" => i64::from(e.defeated_by),
            "defeat_stamp" => e.defeat_stamp,
            "defeat_type" => i64::from(e.defeat_type),
            "victory_type" => i64::from(e.victory_type),
            "wonderwin_stamp" => e.wonderwin_stamp,
            "wonderwin_timer" => i64::from(e.wonderwin_timer),
            "lost_capital_stamp" => e.lost_capital_stamp,
            "lost_capital_timer" => i64::from(e.lost_capital_timer),
            "lost_capital_modifier" => i64::from(e.lost_capital_modifier),
            _ => return None,
        })
    }

    pub(crate) fn endgame_mut(&mut self, who: Player) -> Option<&mut EndGame> {
        self.leaders.endgame.get_mut(usize::from(who))
    }

    /// **`Leader::defeat_by@006d1c80`'s captor stamp**, the first thing
    /// `Leader::defeat@006ecb00` does, ahead of its defeated guard
    /// (§10.2 step 1). While `defeated_by` is −1 the call stamps the frame
    /// and, with a captor, records it; the answer is whether it did, which
    /// is the gate of the captor arm. So a defeated leader is re-stamped by
    /// every later call until a captor is recorded.
    pub(crate) fn defeat_by_stamp(&mut self, who: Player, by: Option<Player>) -> bool {
        let frame = self.frame;
        let Some(e) = self.endgame_mut(who) else {
            return false;
        };
        if e.defeated_by >= 0 {
            return false;
        }
        e.defeat_stamp = frame;
        if let Some(by) = by {
            e.defeated_by = i32::from(by);
        }
        true
    }

    /// `Leader::defeat`'s store past its guards: `defeat_type = type`.
    pub(crate) fn note_defeat_type(&mut self, who: Player, ty: i32) {
        if let Some(e) = self.endgame_mut(who) {
            e.defeat_type = ty;
        }
    }

    /// **`Leader::victory@006ec9b0`** (§10.2): nothing for a leader that has
    /// won or been defeated; else the won flag and `victory_type`, then
    /// every other live seated slot either wins with it (it counts `who` as
    /// an ally) or is defeated by the cascade, `defeat(5, −1, 0)`.
    ///
    /// SEAM: `instant` (`flags2 & 0x100`), the `0x80` mark, the messages
    /// and the achievement are not carried; nothing this crate holds reads
    /// them.
    pub fn victory(&mut self, who: Player, ty: i32) {
        let w = usize::from(who);
        if w >= self.leaders.endgame.len()
            || self.leaders.endgame[w].won
            || self.leaders.defeated.get(w).copied().unwrap_or(true)
        {
            return;
        }
        self.leaders.endgame[w].won = true;
        self.leaders.endgame[w].victory_type = ty;
        for j in 0..self.leaders.endgame.len().min(8) {
            if j == w || self.leaders.defeated[j] {
                continue;
            }
            let j = j as Player;
            if self.is_ally(j, who) {
                self.victory(j, ty);
            } else {
                self.defeat(j, DEFEAT_CASCADE, None);
            }
        }
    }

    /// `Game::wonder_timer@005944f0`: [`scaled_timer`] of `WONDER_TIMER`.
    pub fn wonder_timer(&self) -> i64 {
        scaled_timer(self.tables.tuning.wonder_timer, self.world.width())
    }

    /// `Game::popwin_timer@005944b0`: [`scaled_timer`] of `POPWIN_TIMER`.
    pub fn popwin_timer(&self) -> i64 {
        scaled_timer(self.tables.tuning.popwin_timer, self.world.width())
    }

    /// **The wonder arm of `GameDaemon::process_victory@00730ef0`** (§10.2),
    /// every frame under `VICTORY` 0, 1 or 6, ahead of the territory arm.
    /// `W = Game::wonder_winning@005948a0` is read once. Each live seated
    /// slot in order is off side (`W < 0`, or neither `W` nor mutually
    /// allied with it: a running clock stops and banks, `stamp −= frame`)
    /// or on side: under `VICTORY` 0 with two or more nations the clock
    /// starts (`stamp += frame`), or, already running and no earlier on-side
    /// slot handled this frame, wins when `wonder_timer() − (frame − stamp)`
    /// is **exactly** 0; under any other lobby the first on-side slot wins
    /// at once.
    ///
    /// SEAM: `gg`, a teammate holding preq `GLOBAL_GOVERNMENT_BONUS`
    /// (`0x2b9`), is read as false: the crate keeps no such preq.
    /// Multiplayer (`semaphore[0] & 4`) is read as clear. SEAM scan:
    /// `grep -h '^ *victory_type 1$' gamelog-*.txt | wc -l` — 0 over the
    /// 1,196 kept dumps, so no capture holds the immediate wonder victory
    /// `gg` would give.
    pub(crate) fn process_victory_wonder(&mut self) {
        let victory = self.ai.lobby.victory;
        if !matches!(victory, 0 | 1 | 6) {
            return;
        }
        let win = self.wonder_winning();
        let n = self.leaders.endgame.len().min(8);
        let nations = self.territory.players.len();
        let frame = self.frame;
        let mut handled = false;
        for i in 0..n {
            if self.leaders.defeated.get(i).copied().unwrap_or(true) {
                continue;
            }
            let p = i as Player;
            let on = win >= 0 && (i as i32 == win || self.is_ally(p, win as Player));
            if !on {
                let e = &mut self.leaders.endgame[i];
                if e.wonderwin_timer != 0 {
                    e.wonderwin_timer = 0;
                    e.wonderwin_stamp -= frame;
                }
                continue;
            }
            if victory != 0 || nations <= 1 {
                if !handled {
                    self.victory(p, VICTORY_WONDER);
                }
            } else if self.leaders.endgame[i].wonderwin_timer == 0 {
                let e = &mut self.leaders.endgame[i];
                e.wonderwin_timer = 1;
                e.wonderwin_stamp += frame;
            } else if !handled {
                let remaining =
                    self.wonder_timer() - (frame - self.leaders.endgame[i].wonderwin_stamp);
                if remaining == 0 {
                    self.victory(p, VICTORY_WONDER);
                }
            }
            handled = true;
        }
    }

    /// The wonder clock's pay-back, inlined in `Leaders::process_all@006ed2a0`
    /// inside its live-slot gate (§10.2): every `TIMER_REFRESH_RATIO` frames a
    /// stopped clock's negative stamp gains one.
    pub(crate) fn refresh_wonder_timer(&mut self, who: usize) {
        let r = i64::from(self.tables.tuning.timer_refresh_ratio);
        if r == 0 || self.frame % r != 0 {
            return;
        }
        if let Some(e) = self.leaders.endgame.get_mut(who)
            && e.wonderwin_stamp < 0
            && e.wonderwin_timer == 0
        {
            e.wonderwin_stamp += 1;
        }
    }
}

#[cfg(test)]
mod tests;
