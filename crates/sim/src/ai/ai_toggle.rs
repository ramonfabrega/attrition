//! **The AI toggle, as an order** (item 4283, `docs/CLIENT_INPUT.md` §28):
//! `CommandPackage::process_cheat_ai_toggle@00944d20`, the turn pump's
//! handler of command `0x40`.
//!
//! The console's `ai`, `ai on` and `ai off` (`ConsoleWin::run_cmd@007d6a70`
//! case `0xc`) do not act where they are typed: each calls
//! `CommandManager::issue_cheat_ai_toggle@00940fa0`, which adds the
//! one-byte command to the local package. The handler logs and then does
//! what `Game::action_cheat_ai_toggle@005930c0` does, inline: when bit 2
//! of the game's first semaphore byte is clear, `ai_off = !ai_off`. That
//! one flag and its three readers in a game are `docs/INPUT.md` §11.4.
//!
//! SEAM: the semaphore bit is a multiplayer bit (`economy/holdings.rs`,
//! `vision/seen_ally.rs`), and this simulation plays a solo lobby, so the
//! guard is read as clear and the flag always flips.

use crate::Sim;

impl Sim {
    /// `process_cheat_ai_toggle@00944d20`: the AI's global flag, flipped.
    pub fn process_cheat_ai_toggle(&mut self) {
        self.ai.ai_off = !self.ai.ai_off;
    }
}

#[cfg(test)]
mod tests;
