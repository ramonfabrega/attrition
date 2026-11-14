//! `Leader::diplomacy@006bc950` — a computer leader's view of every other
//! leader it has met, once a cadence per pair: how strong and how weak it
//! stands against them (`strong`, `weak`), whether it wants war
//! (`aggression`), and, when it does and the rush rules forbid it, the one
//! message that says so (`agendas |= 0x40`, the receiver's
//! `got_diplo_message`). `docs/AI.md` §220.
//!
//! What is here is the function's head through the rush-rules arm, which
//! is every branch the peace pair reaches before its war: two leaders, at
//! peace, neither allied, `TEAM_STYLE 1`; and since item 4159's step VII-c
//! ([`offers`]) the arms of team styles 0, 8 and 11 and the offers arm
//! that answers a committed proposal. Each arm no walked lobby reaches
//! stops at a `SEAM` that names it, so a later lobby that reaches one
//! parts there rather than on a guess.

use crate::cities::build::Ident;
use crate::world::Terrain;
use crate::{Player, Sim};

/// `Leader::diplomacy+0x1168`: the peace coin — an odd draw, or a boomer
/// below the fourth Military epoch, sends a non-rusher on without
/// planning a war.
pub const SITE_PEACE_COIN: &str = "Leader::diplomacy+0x1168";
/// `Leader::diplomacy+0x1322`: the coin over the target's cities that
/// were mine, one in four (one in two for a boomer) clearing `aggression`.
pub const SITE_RACE_COIN: &str = "Leader::diplomacy+0x1322";
/// `Leader::diplomacy+0x1388`: the coin over my cities that were the
/// target's, one in three (one in two for a boomer).
pub const SITE_MINE_COIN: &str = "Leader::diplomacy+0x1388";
/// `Leader::diplomacy+0x1b12`: a leader at peace with no `aggression` and
/// `agendas & 8` skips the rest two times in three.
pub const SITE_QUIET_COIN: &str = "Leader::diplomacy+0x1b12";

/// The four building lines `strong` counts one each for when I have one
/// standing and the target has none — `buildtypes[BARRACKS]`, `[STABLE]`,
/// `[SIEGEFACTORY]`, `[AIRBASE]` and each one's `to` chain
/// (`006bc950:403–482`, `num_buildings + get_buildings(to)`).
const LINES: [Ident; 4] = [
    Ident::Barracks,
    Ident::Stable,
    Ident::SiegeFactory,
    Ident::Airbase,
];

impl Sim {
    /// `Game::war_allowed@00594670`: no rush rules; or the leaders'
    /// highest age at or past the rule (`Game::current_age`); or a timed
    /// rule (past 8) whose minutes × 900 have run.
    pub fn war_allowed(&self) -> bool {
        let rr = self.ai.lobby.rush_rules;
        rr == 0
            || self.current_age() >= rr
            || (rr > 8 && self.frame >= i64::from(self.ai.lobby.rush_minutes) * 900)
    }

    /// `Game::current_age@005946d0`: the highest `ages` among the leaders
    /// in play (`leader_flags & 1`), never below 0.
    fn current_age(&self) -> i32 {
        (0..self.territory.players.len())
            .filter(|&j| !self.leaders.defeated[j])
            .map(|j| self.tech[j].ages)
            .max()
            .unwrap_or(0)
            .max(0)
    }

    /// The peace's end (`Game::do_frame@00591ef0:202–261`, between
    /// `Leaders::process_all` and `Leaders::strategy_all`): on the one
    /// frame a timed rule's minutes × 900 is reached, and outside team
    /// styles 0, 8 and 11, every leader in play declares war on every
    /// other in play it is neither allied with nor already at war with —
    /// `Leader::set_diplo(i, j, 0)`, which writes both directions. Its
    /// `Armies::diplo_change(i)` processes a computer declarer's armies;
    /// the outer walk runs from slot 0, so in a pair whose slot 0 is the
    /// human the declarer is the human and no army runs. `docs/AI.md` §224.
    pub fn end_the_peace(&mut self) {
        let rr = self.ai.lobby.rush_rules;
        if rr <= 8 || self.frame != i64::from(self.ai.lobby.rush_minutes) * 900 {
            return;
        }
        if matches!(self.ai.lobby.team_style, 0 | 8 | 11) {
            return;
        }
        // SEAM: team style 7's neutral leaders (`LeaderData::is_neutral`,
        // a player on team 8) neither declare nor are declared on — the
        // crate carries no lobby team, and every capture is style 1.
        // The `+0x1f4` zeroed for each leader in play is **`attrition_stamp`**
        // (the base is the leader slot, `docs/LEADERS.md` §5.2), not the
        // editor's `update_leader_list`.
        self.zero_attrition_stamps();
        let players = self.territory.players.len().min(8);
        for i in 0..players {
            if self.leaders.defeated[i] {
                continue;
            }
            for j in 0..players {
                if j == i || self.leaders.defeated[j] {
                    continue;
                }
                if !self.allied_both(i, j) && self.not_at_war(i, j) {
                    // `Leader::set_diplo(i, j, 0)`: both ways, then
                    // `Armies::diplo_change(i)` (item 4207).
                    self.set_diplo(i as Player, j as Player, 0);
                }
            }
        }
    }

    /// `diplos[a][b]`: 0 war, 1 peace, 2 allied; a leader's own slot is 2.
    pub(crate) fn diplos(&self, a: usize, b: usize) -> i32 {
        if a == b || self.leaders.allied[a][b] {
            2
        } else if self.leaders.at_war[a][b] {
            0
        } else {
            1
        }
    }

    /// `t == me`, or the two at peace or allied both ways — the idiom
    /// `(t == who || (diplos[t] != 0 && theirs[who] != 0))`.
    fn not_at_war(&self, me: usize, t: usize) -> bool {
        me == t || (self.diplos(me, t) != 0 && self.diplos(t, me) != 0)
    }

    /// `t == me`, or allied both ways.
    fn allied_both(&self, me: usize, t: usize) -> bool {
        me == t || (self.diplos(me, t) == 2 && self.diplos(t, me) == 2)
    }

    /// `LeaderData::get_buildings` over one of [`LINES`]: the finished
    /// buildings of the line's root record and of its `to` chain.
    fn line_buildings(&self, who: Player, ident: Ident) -> i32 {
        self.tables
            .build_types
            .iter()
            .position(|b| b.ident == ident)
            .map_or(0, |rec| self.buildings_of_line(who, rec))
    }

    /// `Leader::diplomacy@006bc950` for one leader, every frame from
    /// `Leaders::strategy_all` after `plan_strategy` (`docs/AI.md` §220).
    pub fn diplomacy(&mut self, who: Player) {
        let me = who as usize;
        // The head: a human (`leader_flags & 4`) and `ai off` return.
        // SEAM: `semaphore[1] & 2`, which also returns, is not carried.
        if self.cities.nation[me].human || self.ai.ai_off {
            return;
        }
        let players = self.territory.players.len().min(8);
        // Leaders in play that are not my mutual ally (`LeaderData::is_ally`
        // asked of each about me; I am my own ally).
        let non_allies = (0..players)
            .filter(|&l| !self.leaders.defeated[l] && !self.allied_both(l, me))
            .count();
        // SEAM: the score count and the top scorer (`local_64`, `local_70`,
        // `LeaderData::score`) — `compute_score` is not modelled. Their one
        // reader on this path is the top-scorer arm, which waits on frame
        // 45 × 900 (30 × 900 at difficulty 3 and up) and is taken here as
        // "the top scorer is never the target".
        for t in 0..players {
            if t == me || self.leaders.defeated[t] {
                continue;
            }
            let treaty = self.leaders.treaties[me][t];
            if treaty & 1 == 0 || treaty & 4 != 0 {
                continue;
            }
            self.diplomacy_with(me, t, non_allies);
        }
    }

    /// One target of [`Sim::diplomacy`]: `006bc950:150` to the rush-rules
    /// arm's `agendas |= 0x40` (`:1723`).
    fn diplomacy_with(&mut self, me: usize, t: usize, non_allies: usize) {
        let frame = self.frame;
        let agenda = self.leaders.agendas[me][t];
        // The cadence (`docs/LEADERS.md` §8.2): while I have not committed
        // to `t` (`dip[t].agree`, `+0x692c`), a demand outstanding
        // (`tribute_demanded`, `+0x334`) stretches the period to `0x2000`
        // and a counteroffer (`counteroffer`, `+0x314`) doubles it. The
        // demand arm is not carried; the counteroffer is set by the offers
        // arm on a deal found short (steps VII-c, VII-d): run1575's French
        // on the alliance of 1051, which doubles their period on the human
        // from 1052.
        let forced = agenda & 4 != 0;
        if !forced {
            let mut period: i64 = 0x800;
            let deal = &self.leaders.tribute[me];
            if deal.dip[t].agree != 1 {
                if deal.tribute_demanded[t] != 0 {
                    period = 0x2000;
                }
                if deal.counteroffer[t] != 0 {
                    period *= 2;
                }
            }
            if agenda & 8 == 0 {
                period >>= 2;
            }
            let phase = (t as i64 + me as i64 * 8) * (period >> 6) + frame;
            if (period - 1) & phase != 0 {
                return;
            }
        }
        self.leaders.agendas[me][t] = agenda & !4;

        let age_gap = self.tech[me].ages - self.tech[t].ages;
        let flags = self.ai.ai[me].player_flags;
        let rusher = flags >> 12 & 1 != 0;
        let boomer = flags >> 13 & 1 != 0;
        let seed = self.game_seed as u32;
        let span = if rusher {
            seed % 3 + 4
        } else if boomer {
            seed % 6 + 8
        } else {
            seed % 5 + 6
        };
        // The frame from which a planned war (`local_34`) is wanted.
        let plan_frame = i64::from(((seed.wrapping_add(me as u32) & 7) + span * 3) * 300);

        // My cities of the target's race, and the target's of mine.
        let mine_theirs = self
            .cities
            .cities
            .iter()
            .filter(|c| c.alive && c.owner as usize == me && c.race == Some(t as Player))
            .count();
        let theirs_mine = self
            .cities
            .cities
            .iter()
            .filter(|c| c.alive && c.owner as usize == t && c.race == Some(me as Player))
            .count();
        // Third parties: their cities I founded, and how many have more
        // than fourteen units on my land.
        let players = self.territory.players.len().min(8);
        let mut founded_by_me = 0;
        let mut invaded_by = 0;
        for k in 0..players {
            if self.leaders.defeated[k] || k == me || k == t {
                continue;
            }
            if self.allied_both(t, k) || self.allied_both(me, k) {
                continue;
            }
            founded_by_me += self
                .cities
                .cities
                .iter()
                .filter(|c| c.alive && c.owner as usize == k && c.founder as usize == me)
                .count();
            if self.ai.ai[me].census.invaders.get(k).copied().unwrap_or(0) > 14 {
                invaded_by += 1;
            }
        }

        // The region pass: where we both stand, `aggression`; my weakness
        // and my strength there.
        let mut aggression = 0;
        let mut strong: i32 = 0;
        let mut weak: i32 = 0;
        let ally = self.allied_both(me, t);
        // `(city_num + village_num) / 2`, the share of my settlements a
        // region must hold for the weakness test.
        let halve = (self.city_num(me as Player) + self.village_num(me as Player)) / 2;
        {
            let mine = &self.ai.ai[me].census;
            let theirs = &self.ai.ai[t].census;
            // SEAM: the original walks land slots 1..63 and skips slot 0;
            // this crate's regions carry no original number, so every land
            // region is walked.
            for r in 0..self.world.region_count() {
                if self.world.terrain(r as u16) != Terrain::Land {
                    continue;
                }
                let r16 = r as u16;
                let my_cities = crate::ai::Census::reg_u16(&mine.reg_cities, r16);
                let my_combat = crate::ai::Census::reg_u16(&mine.reg_combat, r16);
                let my_defense = crate::ai::Census::reg_u16(&mine.reg_defense, r16);
                let their_cities = crate::ai::Census::reg_u16(&theirs.reg_cities, r16);
                let their_combat = crate::ai::Census::reg_u16(&theirs.reg_combat, r16);
                if my_cities == 0 && my_combat == 0 {
                    continue;
                }
                if their_cities == 0 && their_combat == 0 {
                    continue;
                }
                aggression = 1;
                if halve <= my_cities {
                    let mut u = my_combat + my_defense * 3;
                    if their_cities != 0 {
                        u += my_combat >> 1;
                    }
                    if age_gap > 0 {
                        u = (u << 2) / 3;
                    } else if age_gap < 0 {
                        u = (u * 3) >> 2;
                    }
                    if u < their_combat {
                        weak += (their_combat << 5) / (u + 1);
                        continue;
                    }
                }
                if their_cities == 0 {
                    if my_cities != 0 {
                        strong += (my_combat >> 1) - their_combat + (my_defense >> 1);
                    }
                } else {
                    let mut u = my_combat;
                    if my_cities != 0 && ally {
                        u -= their_combat;
                    }
                    strong += u;
                }
            }
        }
        // `weak /= max(1, dow[who])` — the leader's own `dow` slot (6bcf0a),
        // which only a self-target would count: no caller sends one.
        let weak_divisor = self.leaders.declare[me].dow[me].max(1);
        weak /= weak_divisor;
        // Somebody at war with the target whose `weak` against it is 0.
        let someone_unafraid = (0..players).any(|l| {
            !self.leaders.defeated[l]
                && l != t
                && (self.diplos(l, t) == 0 || self.diplos(t, l) == 0)
                && self.ai.ai[l].weak[t] == 0
        });
        let my_attack = self.ai.ai[me].census.attack;
        let their_attack = self.ai.ai[t].census.attack;
        if (weak != 0 || their_attack > my_attack * 3) && !someone_unafraid {
            aggression = 0;
        }
        if age_gap > 0 {
            strong = (strong << 2) / 3 + 1;
        } else if age_gap < 0 {
            strong = strong * 3 / 4;
        }
        if agenda & 0x10 != 0 && self.not_at_war(me, t) {
            strong += self.ai.ai[me].census.invaders.get(t).copied().unwrap_or(0);
        }
        let diff = self.ai_difficulty();
        let target_human = self.cities.nation[t].human;
        if target_human {
            if diff >= 4 {
                strong += 2;
            } else if diff == 3 {
                strong += 1;
            }
        }
        for ident in LINES {
            if self.line_buildings(me as Player, ident) != 0
                && self.line_buildings(t as Player, ident) == 0
            {
                strong += 1;
            }
        }
        if diff > 2 {
            strong += (0..players)
                .filter(|&l| {
                    !self.leaders.defeated[l]
                        && l != me
                        && l != t
                        && (self.diplos(l, t) == 0 || self.diplos(t, l) == 0)
                })
                .count() as i32;
        }
        strong <<= 5;
        if strong < 1 {
            aggression = 0;
        }

        // `agendas & 2` (a building of theirs stood in my armed walk) is
        // read and spent here.
        let mut grievance = i32::from(self.leaders.agendas[me][t] & 2 != 0);
        self.leaders.agendas[me][t] &= !2;
        let invaders = self.ai.ai[me].census.invaders.get(t).copied().unwrap_or(0);
        if self.leaders.agendas[me][t] & 0x10 != 0
            && self.not_at_war(me, t)
            && (if boomer { 4 } else { 1 }) < invaders
        {
            grievance = 1;
        }
        let at_war = i32::from(!self.not_at_war(me, t));
        let tolerated = if self.ai.ai[me].pers.friendly_human >= 0 {
            at_war
        } else {
            at_war + 1
        };
        let wars = self.ai.ai[me].census.wars;
        let mut too_many_wars = tolerated < wars - grievance;
        if too_many_wars {
            aggression = 0;
        }
        if wars != 0 && (invaded_by != 0 || founded_by_me != 0) {
            aggression = 0;
            too_many_wars = true;
        }
        // SEAM: the team-score walk (`get_team_score`) and team style 7's
        // arms; the target is neither me nor my ally here, so only team
        // style 7 reads them.
        if ally {
            // The allied arm opens by clearing `aggression`; SEAM: the rest
            // of it (`006bc950:600–700`, `:757`).
            self.store_view(me, t, 0, strong, weak);
            return;
        }

        // `006bc950:762–860`: a target at peace with me.
        let mut plan_war = false;
        if self.diplos(me, t) != 0 && self.diplos(t, me) != 0 && target_human {
            // The clearing arm (6bd871): the human target's `dow` on me
            // under 2 and its `blacken` under 4 (item 4159, step VII-b1's
            // leaves, read since step VII-c).
            let their_wars = self.ai.ai[t].census.wars;
            let (their_dow, their_blacken) = (
                self.leaders.declare[t].dow[me],
                self.leaders.declare[t].blacken,
            );
            if their_dow < 2 && their_blacken < 4 && their_wars > 0 {
                let clear = match diff {
                    d if d < 2 => true,
                    2 => their_wars > 1,
                    3 => their_wars > 2,
                    _ => false,
                };
                if clear {
                    aggression = 0;
                }
            }
            if !too_many_wars {
                let war_now = at_war;
                match diff {
                    3 => {
                        if their_wars <= war_now {
                            self.mark(SITE_PEACE_COIN);
                            let coin = self.rng.roll();
                            let early_boomer = boomer && self.tech[me].epoch[0] < 4;
                            plan_war = !((coin & 1 != 0 || early_boomer) && !rusher);
                        }
                    }
                    2 => {
                        if their_wars <= war_now {
                            plan_war = rusher;
                        }
                    }
                    d if d >= 4
                        && their_wars <= war_now
                        && (!boomer || self.tech[me].epoch[0] > 3) =>
                    {
                        plan_war = true;
                    }
                    _ => {}
                }
            }
        }
        // `LAB_006bd941`.
        if diff > 1 && non_allies == 1 {
            aggression = 1;
        }
        // SEAM: `get_diff`'s and this arm's `semaphore[0] & 4`
        // (multiplayer) is never set.
        // The target's `blacken` (`+0x20c`, 6bdc3b).
        let blacken = self.leaders.declare[t].blacken;
        if theirs_mine != 0 && blacken < 2 && !rusher {
            self.mark(SITE_RACE_COIN);
            let coin = self.rng.roll();
            if coin % (if boomer { 2 } else { 4 }) == 0 {
                aggression = 0;
            }
        }
        // The hires (`hire_stamp`, `+0x1b4`): a deal against `t` inside
        // 13500 frames wants the war (6bdc8a), and the mine coin clears it
        // only with none, or one older than 4500 (6bdce2). The stamps are
        // written by an executed attack deal, which needs a request no
        // crate path sets, so both read 0 in every walked game.
        let hired = self.leaders.declare[me].hire_stamp[t];
        if hired != 0 && frame - hired < 13_500 {
            aggression = 1;
        }
        if mine_theirs != 0 && theirs_mine == 0 {
            self.mark(SITE_MINE_COIN);
            let coin = self.rng.roll();
            if coin % (if boomer { 2 } else { 3 }) == 0 && (hired == 0 || frame - hired > 4500) {
                aggression = 0;
            }
        }
        // SEAM: the third parties' hire arms — 6bdd57 compares
        // `leaders[l].hire_who[me]` with `l` itself, a test no deal can
        // pass; 6bdd87 needs `t` my mutual ally, whose pass this port ends
        // in the allied arm above.
        if self.leaders.agendas[me][t] & 0x20 != 0 {
            aggression = 1;
        }
        if plan_war && plan_frame <= frame {
            aggression = 1;
        }
        let mut rivals = alliance::Rivals::default();
        match self.ai.lobby.team_style {
            // Item 4159, step VII-c: the elimination and victory arms of
            // team styles 0, 8 and 11 (`offers.rs`), then `LAB_006be428`.
            0 | 8 | 11 => {
                self.store_view(me, t, aggression, strong, weak);
                let Some(r) = self.style_zero_arms(me, t) else {
                    return;
                };
                rivals = r;
            }
            7 => {
                // SEAM: team style 7's arms (a neutral team-8 leader); every
                // capture is style 0 or 1.
                self.store_view(me, t, aggression, strong, weak);
                return;
            }
            _ => {
                aggression = 1;
                self.store_view(me, t, aggression, strong, weak);
            }
        }
        // `LAB_006be428` onward: the quiet coin, the clear of a committed
        // deal, the offers arm (item 4159, step VII-c) and the rush rules'
        // message.
        self.diplomacy_tail(me, t, forced, rivals);
    }

    /// The three per-target words, as `Leader::diplomacy` leaves them.
    fn store_view(&mut self, me: usize, t: usize, aggression: i32, strong: i32, weak: i32) {
        let l = &mut self.ai.ai[me];
        l.aggression[t] = aggression;
        l.strong[t] = strong;
        l.weak[t] = weak;
    }
}

mod alliance;
mod offers;

#[cfg(test)]
mod tests;
