//! The territory victory's timer — `GameDaemon::process_victory@00730ef0`'s
//! POPWIN arm and `Leader::process@006b88b0`'s pay-back (`docs/AI.md` §126).
//!
//! What is modelled is the **clock and its flag**, `popwin_stamp` and
//! `popwin_timer`, because the AI reads the flag: `Army::find_target`
//! scores an enemy's city ×100 while it runs (`docs/ARMY.md` §12,
//! `about_to_win`). Every frame, under `VICTORY` 0, 1 or 7, the leaders are
//! walked in slot order; one whose team holds less than the lobby's share of
//! the world's land, or one some unallied rival also holds it against, has
//! its timer stopped; the first that holds it alone starts its timer, or
//! keeps it, and the walk ends there.
//!
//! The arm's other outcomes are seams: [`seams`].

use crate::Sim;

/// Where this module knowingly stands in for the original.
///
/// | seam | stands in for | what it costs |
/// | --- | --- | --- |
/// | the expiry | `Leader::victory(2)` once `Game::popwin_timer() − (frame − stamp)` falls below one, and the immediate victory under `VICTORY` 1 or 7 or a teammate's `GLOBAL_GOVERNMENT_BONUS` | the game never ends by territory; the timer runs on |
/// | `semaphore[0] & 4`, `semaphore[1] & 2`, `num_nations` | the arm's gate | read as a game of two or more nations, the arm always on |
///
/// SEAM scan: `grep -h '^  VICTORY' gamelog*.txt | sort | uniq -c` — every
/// capture on file is `VICTORY 0`, so the immediate arm is never reached;
/// and none on file holds a `popwin_timer 1` long enough to expire it.
pub mod seams {}

impl Sim {
    /// `GameDaemon::process_victory`'s POPWIN arm: the territory timers.
    pub(crate) fn process_victory_territory(&mut self) {
        if !matches!(self.lobby.victory, 0 | 1 | 7) {
            return;
        }
        let land = self.land_size();
        if land <= 0 {
            return;
        }
        let share = self.lobby.pop_win_percent;
        let n = self.territory.players.len();
        // `LeaderData::get_team_terr@006d62e0` over `LeaderData::territory`
        // — the leader's own count, which `Leader::calc_gather` rewrites on
        // its reassembly frames ([`crate::holdings`]), not the live map: a
        // live scan starts French East Indies' timer on 14288, five frames
        // before the original's 14293 (item 1487).
        let terr: Vec<i32> = (0..n)
            .map(|i| {
                (0..n)
                    .filter(|&j| j == i || (!self.defeated[j] && self.allied[i][j]))
                    .map(|j| self.holdings[j].territory)
                    .sum()
            })
            .collect();
        let frame = self.frame;
        for i in 0..n {
            // `leader_flags & 3 == 3`: seated and not defeated.
            if self.defeated[i] {
                continue;
            }
            let rivals = (0..n)
                .filter(|&j| {
                    j != i
                        && !self.defeated[j]
                        && !(self.allied[i][j] && self.allied[j][i])
                        && share <= terr[j] * 100 / land
                })
                .count();
            if terr[i] * 100 / land < share || rivals != 0 {
                let l = &mut self.ai[i];
                if l.popwin_timer != 0 {
                    l.popwin_timer = 0;
                    l.popwin_stamp -= frame;
                }
                continue;
            }
            if self.lobby.victory == 0 && self.ai[i].popwin_timer == 0 {
                let l = &mut self.ai[i];
                l.popwin_timer = 1;
                l.popwin_stamp += frame;
            }
            return;
        }
    }

    /// `Leader::process`'s pay-back: every `TIMER_REFRESH_RATIO` frames a
    /// stopped timer's negative stamp gains one, so time lost to a stop is
    /// earned back at a fifth of the rate it ran.
    pub(crate) fn refresh_victory_timers(&mut self, who: usize) {
        let r = i64::from(self.tables.tuning.timer_refresh_ratio);
        if r == 0 || self.frame % r != 0 {
            return;
        }
        let l = &mut self.ai[who];
        if l.popwin_stamp < 0 && l.popwin_timer == 0 {
            l.popwin_stamp += 1;
        }
    }

    /// `WorldData::land_size`: the cells of every land region
    /// (`World::analyze_map@006b58b0` sums regions 1 to 63's sizes — the
    /// `Region` records from `+0x88`, one stride in).
    ///
    /// **The range is the original's numbering, not this crate's index**
    /// (item 3575): the harness declares a loaded map's regions in order of
    /// first appearance, so Great Sahara's one land region, the original's
    /// 1, is index 0 here, and a filter on the index summed nothing — the
    /// Mongols' Classical pair never started the timer theirs started on
    /// 8195, and its army's `about_to_win` ×100 never fired (`docs/AI.md`
    /// §311).
    pub(crate) fn land_size(&self) -> i32 {
        self.world
            .regions()
            .filter(|&(r, t)| {
                (1..0x40).contains(&self.world.region_number(r)) && t == crate::world::Terrain::Land
            })
            .map(|(r, _)| self.world.region_size(r))
            .sum()
    }
}

#[cfg(test)]
mod tests;
