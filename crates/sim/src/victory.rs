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
        let n = self.players.len();
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
        let r = i64::from(self.tuning.timer_refresh_ratio);
        if r == 0 || self.frame % r != 0 {
            return;
        }
        let l = &mut self.ai[who];
        if l.popwin_stamp < 0 && l.popwin_timer == 0 {
            l.popwin_stamp += 1;
        }
    }

    /// `WorldData::land_size`: the cells of every land region
    /// (`World::analyze_map@006b58b0` sums regions 1 to 63's sizes).
    pub(crate) fn land_size(&self) -> i32 {
        self.world
            .regions()
            .filter(|&(r, t)| (1..0x40).contains(&r) && t == crate::world::Terrain::Land)
            .map(|(r, _)| self.world.cells_in(r).count() as i32)
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use crate::Sim;
    use crate::world::{Cell, Terrain, World};

    /// A two-player sim on a 10×10 map, region 0 a sea (the listing's sum
    /// starts at region 1) and region 1 the land, at war.
    fn sim() -> Sim {
        let mut w = World::new(10, 10);
        let sea = w.add_region(Terrain::Sea);
        let land = w.add_region(Terrain::Land);
        for y in 0..10 {
            for x in 0..10 {
                w.set_region(Cell::new(x, y), if y == 0 { sea } else { land });
            }
        }
        Sim::new(crate::Tuning::RON, w, 2)
    }

    /// Gives `who` a territory of `n` cells: `LeaderData::territory`, as
    /// `Leader::calc_gather` leaves it.
    fn own_cells(s: &mut Sim, who: usize, n: i32) {
        s.holdings[who].territory = n;
    }

    #[test]
    fn a_team_over_the_share_starts_its_timer_on_the_frame() {
        let mut s = sim();
        let land = s.land_size();
        own_cells(&mut s, 1, land * 70 / 100 + 1);
        s.frame = 14_293;
        s.process_victory_territory();
        assert_eq!((s.ai[1].popwin_timer, s.ai[1].popwin_stamp), (1, 14_293));
        assert_eq!((s.ai[0].popwin_timer, s.ai[0].popwin_stamp), (0, 0));
        // Running: the stamp stays.
        s.frame = 14_300;
        s.process_victory_territory();
        assert_eq!(s.ai[1].popwin_stamp, 14_293);
    }

    #[test]
    fn a_team_under_the_share_does_not_start() {
        let mut s = sim();
        let land = s.land_size();
        own_cells(&mut s, 1, land * 70 / 100 - 1);
        s.frame = 100;
        s.process_victory_territory();
        assert_eq!(s.ai[1].popwin_timer, 0);
    }

    #[test]
    fn a_stop_leaves_the_time_run_as_a_debt_paid_back_one_a_fifth() {
        let mut s = sim();
        let land = s.land_size();
        own_cells(&mut s, 1, land * 70 / 100 + 1);
        s.frame = 1000;
        s.process_victory_territory();
        own_cells(&mut s, 1, 1);
        s.frame = 1100;
        s.process_victory_territory();
        assert_eq!((s.ai[1].popwin_timer, s.ai[1].popwin_stamp), (0, -100));
        s.frame = 1105;
        s.refresh_victory_timers(1);
        assert_eq!(s.ai[1].popwin_stamp, -99);
        s.frame = 1106;
        s.refresh_victory_timers(1);
        assert_eq!(s.ai[1].popwin_stamp, -99, "only on a multiple of the ratio");
    }

    #[test]
    fn a_victory_other_than_standard_keeps_no_timer() {
        let mut s = sim();
        let land = s.land_size();
        own_cells(&mut s, 1, land * 70 / 100 + 1);
        s.lobby.victory = 3;
        s.process_victory_territory();
        assert_eq!(s.ai[1].popwin_timer, 0);
    }
}
