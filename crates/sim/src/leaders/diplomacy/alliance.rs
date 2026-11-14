//! **An alliance asked** — item 4159's step VII-d (`docs/LEADERS.md` §7.2,
//! §8.2): the gates `Leader::diplomacy@006bc950`'s offers arm runs on a
//! committed proposal whose `treaty` is 2 (`6bf143` to `6bf4b6`), with the
//! counts they read (`LeaderData::num_allies@006d6250`,
//! `LeaderData::num_team_members@006ebf90`, `Game::is_victory_timer
//! @00594950`) and `Leader::action_respond@006d03c0`'s team gate on an
//! accepted alliance.
//!
//! Read from the listing (`6bf15a`–`6bf4b2`) as well as the decompile.
//! The gate either answers the proposal itself — a refusal, or at team
//! styles 0, 8 and 11 the automatic accept between two computers — or
//! hands the valuation a **price**, taken off `S` before the goods are
//! counted. The shortfall then runs on the `aggression[t]` arm it would
//! for any treaty; with no peace terms for a `treaty` of 2, the price is
//! what makes the need positive, and a positive need with no give of mine
//! is the counteroffer (`offers.rs`).

use super::Sim;
use crate::Player;

/// The two third parties the team-style arms note before the tail, which
/// only the alliance's price reads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Rivals {
    /// `local_6c`: the elimination arm's holder of my capital, when it is
    /// neither the target nor the target's mutual ally (`6bdfb2` on).
    pub capital: Option<usize>,
    /// `local_58`: a live third party, neither my mutual ally nor the
    /// target's, running a territory or wonder clock (`LAB_006be372`,
    /// `VICTORY` 0): the last computer such, or the first human.
    pub racer: Option<usize>,
}

impl Sim {
    /// **`LeaderData::num_allies@006d6250(me)`**: the live leaders
    /// (`leader_flags & 3 == 3`) other than `me` allied with `me` both
    /// ways.
    pub(crate) fn num_allies(&self, me: usize) -> usize {
        let players = self.territory.players.len().min(8);
        (0..players)
            .filter(|&l| l != me && !self.leaders.defeated[l] && self.allied_both(me, l))
            .count()
    }

    /// **`LeaderData::num_team_members@006ebf90(me, live)`**: `me`, and
    /// each other slot in play (`leader_flags & 1`), live too when `live`
    /// (`& 2`), that `LeaderData::is_team@006ebd30(me, l, live)` calls a
    /// teammate: under team style 7 a seat on team 8 is nobody's; else the
    /// same lobby team in 0..3 — and in play with `live` at team styles 0,
    /// 8 and 11 also allied both ways. In play without `live`, `is_team`
    /// is `is_ally`.
    pub(crate) fn num_team_members(&self, me: usize, live: bool) -> usize {
        let players = self.territory.players.len().min(8);
        let team = |p: usize| self.leaders.team.get(p).copied().unwrap_or(8);
        let style = self.ai.lobby.team_style;
        let is_team = |l: usize| -> bool {
            if style == 7 && (team(me) == 8 || team(l) == 8) {
                return false;
            }
            if self.frame != 0 && !live {
                return self.allied_both(me, l);
            }
            if !(0..4).contains(&team(me)) || team(me) != team(l) {
                return false;
            }
            if self.frame != 0 && matches!(style, 0 | 8 | 11) {
                return self.allied_both(me, l);
            }
            true
        };
        1 + (0..players)
            .filter(|&l| l != me && (!live || !self.leaders.defeated[l]) && is_team(l))
            .count()
    }

    /// **`Game::is_victory_timer@00594950`**: under `VICTORY` 0, a live
    /// leader runs the wonder clock (`wonderwin_timer`) or the territory
    /// clock (`popwin_timer`).
    pub(crate) fn is_victory_timer(&self) -> bool {
        if self.ai.lobby.victory != 0 {
            return false;
        }
        let players = self.territory.players.len().min(8);
        (0..players).any(|l| {
            !self.leaders.defeated[l]
                && (self
                    .leaders
                    .endgame
                    .get(l)
                    .is_some_and(|e| e.wonderwin_timer != 0)
                    || self.ai.ai.get(l).is_some_and(|a| a.popwin_timer != 0))
        })
    }

    /// `action_respond`'s **team gate** on an accepted alliance: each
    /// party's allies must all be its teammates
    /// (`1 − num_team_members(·, 1) + num_allies(·) == 0`), the
    /// responder's asked first.
    pub(crate) fn alliance_open(&self, m: usize, x: usize) -> bool {
        let open = |p: usize| self.num_allies(p) + 1 == self.num_team_members(p, true);
        open(m) && open(x)
    }

    /// `LAB_006be372`'s third party ([`Rivals::racer`]), under `VICTORY` 0.
    pub(super) fn racing_third_party(&self, me: usize, t: usize) -> Option<usize> {
        if self.ai.lobby.victory != 0 {
            return None;
        }
        let players = self.territory.players.len().min(8);
        let mut racer = None;
        for l in 0..players {
            if l == me
                || l == t
                || self.leaders.defeated[l]
                || self.allied_both(me, l)
                || self.allied_both(t, l)
            {
                continue;
            }
            let racing = self.ai.ai.get(l).is_some_and(|a| a.popwin_timer != 0)
                || self
                    .leaders
                    .endgame
                    .get(l)
                    .is_some_and(|e| e.wonderwin_timer != 0);
            if racing {
                racer = Some(l);
                if self.cities.nation[l].human {
                    break;
                }
            }
        }
        racer
    }

    /// **The alliance asked** (`6bf143`, `t.dip[me]` committed with
    /// `treaty` 2): `Some(price)` for the valuation to take off `S`, or
    /// `None` when the gate answers the proposal itself.
    ///
    /// 1. **Refused** — withdrawn if I had committed, else rejected
    ///    (`LAB_006bf3a9`) — when another leader is my ally at team styles
    ///    0, 8 and 11 (`num_allies`), at any other style but 7, or while a
    ///    victory clock runs ([`Sim::is_victory_timer`]).
    /// 2. **Accepted at once** between two computers with `aggression[t]`
    ///    0: `action_respond(me, t, 1)`, then `counteroffer[t] = 0`.
    /// 3. **The price**: `(epoch + 1) · 200` at peace, `· 400` at war, my
    ///    own Military epoch (`data_encrypted + 0xe8`); SEAM: halved under
    ///    `semaphore[0] & 4` (6bf45a, multiplayer), never set; +100 for a rusher,
    ///    else −100 for a boomer below epoch 4; −400 with
    ///    [`Rivals::capital`], −200 with [`Rivals::racer`].
    ///
    /// A racer runs a clock, so the victory gate refuses before its −200
    /// is reached; it is ported as the listing has it.
    ///
    /// SEAM: team style 7's arm (`6bf185`, a neutral seat's alliance under
    /// `1 − num_team_members(me, 1) + num_allies == 0`) is not reached:
    /// [`Sim::diplomacy`] ends a style-7 pass before the tail. Multiplayer
    /// (`semaphore[0] & 4`, which halves the price) is never set. The
    /// refusal's console message is the interface's.
    pub(super) fn alliance_asked(&mut self, me: usize, t: usize, rivals: Rivals) -> Option<i32> {
        let (mp, tp) = (me as Player, t as Player);
        let open = match self.ai.lobby.team_style {
            0 | 8 | 11 => self.num_allies(me) == 0,
            _ => false,
        };
        if !open || self.is_victory_timer() {
            let code = if self.leaders.tribute[me].dip[t].agree == 1 {
                0
            } else {
                2
            };
            self.action_respond(mp, tp, code);
            return None;
        }
        if !self.cities.nation[me].human
            && !self.cities.nation[t].human
            && self.ai.ai[me].aggression[t] == 0
        {
            self.action_respond(mp, tp, 1);
            self.leaders.tribute[me].counteroffer[t] = 0;
            return None;
        }
        let military = crate::tech::Line::Military.index();
        let epoch = self.tech[me].epoch[military];
        let mut price = (epoch + 1) * if self.not_at_war(me, t) { 200 } else { 400 };
        let flags = self.ai.ai[me].player_flags;
        if flags >> 12 & 1 != 0 {
            price += 100;
        } else if flags >> 13 & 1 != 0 && epoch < 4 {
            price -= 100;
        }
        if rivals.capital.is_some() {
            price -= 400;
        }
        if rivals.racer.is_some() {
            price -= 200;
        }
        Some(price)
    }
}

#[cfg(test)]
mod tests;
