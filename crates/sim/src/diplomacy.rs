//! `Leader::diplomacy@006bc950` — a computer leader's view of every other
//! leader it has met, once a cadence per pair: how strong and how weak it
//! stands against them (`strong`, `weak`), whether it wants war
//! (`aggression`), and, when it does and the rush rules forbid it, the one
//! message that says so (`agendas |= 0x40`, the receiver's
//! `got_diplo_message`). `docs/AI.md` §220.
//!
//! What is here is the function's head through the rush-rules arm, which
//! is every branch the peace pair reaches before its war: two leaders, at
//! peace, neither allied, `TEAM_STYLE 1`. Each arm the pair does not reach
//! stops at a `SEAM` that names it, so a later lobby that reaches one
//! parts there rather than on a guess.

use crate::build::Ident;
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
        let rr = self.lobby.rush_rules;
        rr == 0
            || self.current_age() >= rr
            || (rr > 8 && self.frame >= i64::from(self.lobby.rush_minutes) * 900)
    }

    /// `Game::current_age@005946d0`: the highest `ages` among the leaders
    /// in play (`leader_flags & 1`), never below 0.
    fn current_age(&self) -> i32 {
        (0..self.players.len())
            .filter(|&j| !self.defeated[j])
            .map(|j| self.tech[j].ages)
            .max()
            .unwrap_or(0)
            .max(0)
    }

    /// `diplos[a][b]`: 0 war, 1 peace, 2 allied; a leader's own slot is 2.
    fn diplos(&self, a: usize, b: usize) -> i32 {
        if a == b || self.allied[a][b] {
            2
        } else if self.at_war[a][b] {
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
        self.build_types
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
        if self.nation[me].human || self.ai_off {
            return;
        }
        let players = self.players.len().min(8);
        // Leaders in play that are not my mutual ally (`LeaderData::is_ally`
        // asked of each about me; I am my own ally).
        let non_allies = (0..players)
            .filter(|&l| !self.defeated[l] && !self.allied_both(l, me))
            .count();
        // SEAM: the score count and the top scorer (`local_64`, `local_70`,
        // `LeaderData::score`) — `compute_score` is not modelled. Their one
        // reader on this path is the top-scorer arm, which waits on frame
        // 45 × 900 (30 × 900 at difficulty 3 and up) and is taken here as
        // "the top scorer is never the target".
        for t in 0..players {
            if t == me || self.defeated[t] {
                continue;
            }
            let treaty = self.treaties[me][t];
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
        let agenda = self.agendas[me][t];
        // SEAM: `dip[t].agree` (`+0x692c`) is never 1 — the Diplomacy
        // records are not carried — and `tribute_demanded`/`counteroffer`
        // (`+0x334`/`+0x314`) are never set.
        let forced = agenda & 4 != 0;
        if !forced {
            let mut period: i64 = 0x800;
            if agenda & 8 == 0 {
                period >>= 2;
            }
            let phase = (t as i64 + me as i64 * 8) * (period >> 6) + frame;
            if (period - 1) & phase != 0 {
                return;
            }
        }
        self.agendas[me][t] = agenda & !4;

        let age_gap = self.tech[me].ages - self.tech[t].ages;
        let flags = self.ai[me].player_flags;
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
            .iter()
            .filter(|c| c.alive && c.owner as usize == me && c.race == Some(t as Player))
            .count();
        let theirs_mine = self
            .cities
            .iter()
            .filter(|c| c.alive && c.owner as usize == t && c.race == Some(me as Player))
            .count();
        // Third parties: their cities I founded, and how many have more
        // than fourteen units on my land.
        let players = self.players.len().min(8);
        let mut founded_by_me = 0;
        let mut invaded_by = 0;
        for k in 0..players {
            if self.defeated[k] || k == me || k == t {
                continue;
            }
            if self.allied_both(t, k) || self.allied_both(me, k) {
                continue;
            }
            founded_by_me += self
                .cities
                .iter()
                .filter(|c| c.alive && c.owner as usize == k && c.founder as usize == me)
                .count();
            if self.ai[me].census.invaders.get(k).copied().unwrap_or(0) > 14 {
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
            let mine = &self.ai[me].census;
            let theirs = &self.ai[t].census;
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
        // `weak /= max(1, dow[who])` — the leader's own `dow` slot, which
        // nothing writes. SEAM: `dow` is not carried.
        let weak_divisor = 1;
        weak /= weak_divisor;
        // Somebody at war with the target whose `weak` against it is 0.
        let someone_unafraid = (0..players).any(|l| {
            !self.defeated[l]
                && l != t
                && (self.diplos(l, t) == 0 || self.diplos(t, l) == 0)
                && self.ai[l].weak[t] == 0
        });
        let my_attack = self.ai[me].census.attack;
        let their_attack = self.ai[t].census.attack;
        if (weak != 0 || their_attack > my_attack * 3) && !someone_unafraid {
            aggression = 0;
        }
        if age_gap > 0 {
            strong = (strong << 2) / 3 + 1;
        } else if age_gap < 0 {
            strong = strong * 3 / 4;
        }
        if agenda & 0x10 != 0 && self.not_at_war(me, t) {
            strong += self.ai[me].census.invaders.get(t).copied().unwrap_or(0);
        }
        let diff = self.ai_difficulty();
        let target_human = self.nation[t].human;
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
                    !self.defeated[l]
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
        let mut grievance = i32::from(self.agendas[me][t] & 2 != 0);
        self.agendas[me][t] &= !2;
        let invaders = self.ai[me].census.invaders.get(t).copied().unwrap_or(0);
        if self.agendas[me][t] & 0x10 != 0
            && self.not_at_war(me, t)
            && (if boomer { 4 } else { 1 }) < invaders
        {
            grievance = 1;
        }
        let at_war = i32::from(!self.not_at_war(me, t));
        let tolerated = if self.ai[me].pers.friendly_human >= 0 {
            at_war
        } else {
            at_war + 1
        };
        let wars = self.ai[me].census.wars;
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
        if self.diplos(me, t) != 0 && self.diplos(t, me) != 0 {
            if target_human {
                // SEAM: `dow` (`+0x270`) and `blacken` (`+0x20c`) are not
                // carried, so the clearing arm reads both as 0.
                let their_wars = self.ai[t].census.wars;
                if their_wars > 0 {
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
                        d if d >= 4 => {
                            if their_wars <= war_now && (!boomer || self.tech[me].epoch[0] > 3) {
                                plan_war = true;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        // `LAB_006bd941`.
        if diff > 1 && non_allies == 1 {
            aggression = 1;
        }
        // SEAM: `get_diff`'s and this arm's `semaphore[0] & 4`
        // (multiplayer) is never set.
        // SEAM: the target's `blacken` (`+0x20c`) is not carried; 0.
        let blacken = 0;
        if theirs_mine != 0 && blacken < 2 && !rusher {
            self.mark(SITE_RACE_COIN);
            let coin = self.rng.roll();
            if coin % (if boomer { 2 } else { 4 }) == 0 {
                aggression = 0;
            }
        }
        // SEAM: `hire_stamp`/`hire_who` (`+0x1b4`, `+0x1d4`) — hiring is
        // not modelled, so neither of their two arms fires.
        if mine_theirs != 0 && theirs_mine == 0 {
            self.mark(SITE_MINE_COIN);
            let coin = self.rng.roll();
            if coin % (if boomer { 2 } else { 3 }) == 0 {
                aggression = 0;
            }
        }
        if self.agendas[me][t] & 0x20 != 0 {
            aggression = 1;
        }
        if plan_war && plan_frame <= frame {
            aggression = 1;
        }
        match self.lobby.team_style {
            0 | 8 | 11 | 7 => {
                // SEAM: the elimination and capital arms of team styles 0,
                // 8 and 11, and team style 7's — every capture is style 1.
                self.store_view(me, t, aggression, strong, weak);
                return;
            }
            _ => {
                aggression = 1;
            }
        }
        self.store_view(me, t, aggression, strong, weak);

        // `LAB_006be428`. SEAM: `dip[t].agree` and the target's
        // `dip[me].agree` are never 1, so `action_clear_all` is not called.
        if !forced && aggression == 0 && self.agendas[me][t] & 8 != 0 {
            self.mark(SITE_QUIET_COIN);
            if self.rng.roll() % 3 != 0 {
                return;
            }
        }
        if aggression == 0 {
            // SEAM: the offers arm (`006bc950:1225–1294`, `LAB_006bf0e0`).
            return;
        }
        if !self.not_at_war(me, t) {
            // SEAM: at war — `LAB_006bf0e0`, the peace and alliance offers.
            return;
        }
        // The rush rules, inline (`war_allowed`): team style 1 goes to
        // `LAB_006bed3f` whether war is allowed or not; the declaring arm
        // past `LAB_006be54c` is team styles 0, 8 and 11 alone.
        let quiet = {
            let stamp = self.ai[t].got_diplo_message;
            stamp != 0 && frame - stamp < 0x1c2
        };
        if self.agendas[me][t] & 0x48 != 0 || quiet {
            return;
        }
        // The message itself (`receive_chat` to the console player) draws
        // only from the interface's own streams.
        self.ai[t].got_diplo_message = frame;
        self.agendas[me][t] |= 0x40;
    }

    /// The three per-target words, as `Leader::diplomacy` leaves them.
    fn store_view(&mut self, me: usize, t: usize, aggression: i32, strong: i32, weak: i32) {
        let l = &mut self.ai[me];
        l.aggression[t] = aggression;
        l.strong[t] = strong;
        l.weak[t] = weak;
    }
}

#[cfg(test)]
mod tests {
    use crate::{Sim, Tuning, World};

    /// The peace pair's shape (`docs/AI.md` §220): who=1 a computer at
    /// difficulty 3, who=0 a human it has met, `TEAM_STYLE 1`, five
    /// minutes' peace, no shared region and no buildings.
    fn peace_pair() -> Sim {
        let mut sim = Sim::new(Tuning::RON, World::new(2, 2), 2);
        sim.nation[0].human = true;
        sim.nation[1].human = false;
        sim.lobby.team_style = 1;
        sim.lobby.difficulty = 3;
        sim.lobby.rush_rules = 10;
        sim.lobby.rush_minutes = 5;
        sim.treaties[1][0] = 1;
        sim
    }

    /// who=1's view of who=0 is taken on 448, 960, 1472, 1984, … — the
    /// period 512 while `agendas & 8` is clear, phased by `(t + who·8)·8`
    /// — and on no other frame; a target not yet met is never visited.
    #[test]
    fn a_pair_is_viewed_once_a_period_on_its_own_phase() {
        let mut sim = peace_pair();
        let mut fired = Vec::new();
        for f in 0..2100 {
            sim.frame = f;
            let before = sim.ai[0].got_diplo_message;
            sim.diplomacy(1);
            if sim.ai[0].got_diplo_message != before || sim.ai[1].strong[0] != 0 {
                fired.push(f);
                sim.ai[0].got_diplo_message = 0;
                sim.ai[1].strong[0] = 0;
                sim.agendas[1][0] = 0;
            }
        }
        assert_eq!(fired, [448, 960, 1472, 1984]);
        sim.treaties[1][0] = 0;
        sim.frame = 2496;
        sim.diplomacy(1);
        assert_eq!(sim.ai[1].strong[0], 0, "an unmet target is skipped");
    }

    /// Run849's block 1985: `aggression` 1 (difficulty > 1, one
    /// non-ally), `strong` `1 << 5` (difficulty 3 against a human, no
    /// building line), and with war not yet allowed the one message:
    /// `agendas |= 0x40`, the receiver stamped with the frame. A second
    /// view inside 450 frames of the stamp, or with `0x40` set, sends none.
    #[test]
    fn the_peace_arm_sends_one_message_and_stamps_the_receiver() {
        let mut sim = peace_pair();
        sim.frame = 1984;
        assert!(!sim.war_allowed(), "1984 is inside the five minutes");
        let seed = sim.rng.seed;
        sim.diplomacy(1);
        assert_ne!(sim.rng.seed, seed, "the peace coin draws");
        assert_eq!(sim.ai[1].aggression[0], 1);
        assert_eq!(sim.ai[1].strong[0], 1 << 5);
        assert_eq!(sim.ai[1].weak[0], 0);
        assert_eq!(sim.agendas[1][0], 0x40);
        assert_eq!(sim.ai[0].got_diplo_message, 1984);
        assert_eq!(sim.ai[1].got_diplo_message, 0, "the stamp is the target's");

        // Inside the quiet window, a cleared bit sends nothing.
        sim.agendas[1][0] = 0;
        sim.frame = 1984 + 512 - 64; // not on the phase: nothing at all
        sim.diplomacy(1);
        assert_eq!(sim.agendas[1][0], 0);
        sim.ai[0].got_diplo_message = 1984 + 100;
        sim.frame = 2496;
        sim.diplomacy(1);
        assert_eq!(
            sim.agendas[1][0], 0,
            "the receiver heard one 412 frames ago"
        );

        // At difficulty 1 nothing raises `aggression` and nothing is sent.
        let mut sim = peace_pair();
        sim.lobby.difficulty = 1;
        sim.frame = 1984;
        sim.diplomacy(1);
        assert_eq!(sim.ai[1].aggression[0], 1, "team style 1 still sets it");
        assert_eq!(sim.ai[1].strong[0], 0, "no human bonus below 3");
    }

    /// The war gate: a timed rule's minutes × 900, or an age at the rule.
    #[test]
    fn war_is_allowed_after_the_rush_minutes() {
        let mut sim = peace_pair();
        sim.frame = 4499;
        assert!(!sim.war_allowed());
        sim.frame = 4500;
        assert!(sim.war_allowed());
        sim.lobby.rush_rules = 0;
        sim.frame = 0;
        assert!(sim.war_allowed());
    }
}
