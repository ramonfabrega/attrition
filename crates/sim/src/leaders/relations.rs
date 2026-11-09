//! Diplomacy on the `Sim`: the players, war, alliance, treaties, and the
//! leader's age and epoch (`docs/STRUCTURE.md` §3, `leaders/`).
//!
//! Moved out of `lib.rs` unchanged by item 4014 (`docs/STRUCTURE.md` §5, move 6).

use crate::*;

/// One lobby seat as `Leader::init`'s diplomacy reads it: `GameInfo`'s
/// `player[k]`, found by its `who` as `LeaderData::get_player` finds it —
/// the `PLAYER` row's `flags` and `team` (a signed byte; 8 is no team).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Seat {
    pub who: Player,
    pub flags: u32,
    pub team: i32,
}

impl Sim {
    /// Whether `who` is at war with `p` — false for an owner outside the
    /// player table (gaia's units path through territory too).
    pub fn at_war_with(&self, who: Player, p: Player) -> bool {
        self.leaders
            .at_war
            .get(who as usize)
            .and_then(|row| row.get(p as usize))
            .copied()
            .unwrap_or(false)
    }

    /// Sets two players as mutual allies — both `diplos` entries at 2.
    pub fn make_allies(&mut self, a: Player, b: Player) {
        self.leaders.allied[a as usize][b as usize] = true;
        self.leaders.allied[b as usize][a as usize] = true;
    }

    /// `Leader::treaty_on@006e1190` — or `param_2` into **both** leaders'
    /// `treaties` slot for the other. The original takes the bits as an
    /// argument and every live caller passes 1, the met bit, so that is
    /// what this takes.
    ///
    /// `Leader::treaty_off@006d0370` is its opposite and clears bit 0
    /// whatever it is passed; its only caller in the whole executable is
    /// the debug console's `run_cmd`, so **nothing in a game ever clears
    /// the met bit** and there is no `treaty_off` here.
    pub fn treaty_on(&mut self, a: Player, b: Player, bits: i32) {
        let (a, b) = (a as usize, b as usize);
        if let Some(x) = self.leaders.treaties.get_mut(a).and_then(|r| r.get_mut(b)) {
            *x |= bits;
        }
        if let Some(x) = self.leaders.treaties.get_mut(b).and_then(|r| r.get_mut(a)) {
            *x |= bits;
        }
    }

    /// `Leader::meet@006e1250` — first contact. `treaty_on(other, 1)`, and
    /// then `LeaderOut::say_meet` for whichever side the console belongs to,
    /// which is the message in the corner of the screen and is not modelled.
    ///
    /// Its callers are `Wall::check_ever_seen@0063ce70` — the live one, and
    /// the reason this hangs off the fog (`docs/VISION.md` §6.2) — and
    /// `Unit::process_attrition@005e11a0`, at **three** sites — its war
    /// arm, its assassin arm, and the generic one under
    /// `get_attrition() != 0`, each after every exemption. That path is
    /// dead in every capture this crate is diffed against — 0 non-exempt
    /// outcomes over run53's 24,000 frames, measured by item 382 — and is
    /// not wired here; the seam is stated in `docs/VISION.md` §6.2.
    ///
    /// `RON_DEBUG_MEET` prints the frame each contact lands on, which is
    /// the only thing about this mechanic no capture on this disk pins:
    /// nothing dumps a leader between 7600 and 8174, so Great Lakes'
    /// original flip is bracketed to (7616, 8174] and this crate's 7944 is
    /// checked against the bracket rather than against a frame.
    pub fn meet(&mut self, a: Player, b: Player) {
        if std::env::var("RON_DEBUG_MEET").is_ok() {
            eprintln!("MEET frame {} {a} {b}", self.frame);
        }
        self.treaty_on(a, b, 1);
    }

    /// Whether `a` has allied `b` — false for an owner outside the player
    /// table, the same reading [`Sim::at_war_with`] gives.
    pub fn allied_with(&self, a: Player, b: Player) -> bool {
        self.leaders
            .allied
            .get(a as usize)
            .and_then(|row| row.get(b as usize))
            .copied()
            .unwrap_or(false)
    }

    /// `LeaderData::is_ally`: the same player, or allied both ways.
    ///
    /// Total in both arguments, like [`Sim::at_war_with`]. The original's
    /// `LeaderData::diplos` is `int[8]`, so `is_ally(8)` reads `treaties[0]`
    /// past the end of it — there is no faithful answer for a leader outside
    /// the table, and no caller asks for one ([`world::PLAYER_SLOTS`]). This
    /// is a guard, not a model: it keeps a gaia unit that reached a scan it
    /// should never have reached from taking the process down.
    pub fn is_ally(&self, a: Player, b: Player) -> bool {
        a == b || (self.allied_with(a, b) && self.allied_with(b, a))
    }

    /// `LeaderData::is_enemy`: different players with war declared either way.
    /// Total in both arguments; see [`Sim::is_ally`].
    pub fn is_enemy(&self, a: Player, b: Player) -> bool {
        a != b && (self.at_war_with(a, b) || self.at_war_with(b, a))
    }

    /// `Leader::set_diplo(whom, level)` as the console's three verbs reach
    /// it — `ally` 2, `peace` 1, `war` 0 (`run_cmd` cases 0x2c–0x2e,
    /// `docs/INPUT.md` §11). The table this crate keeps is two booleans,
    /// so only the ends of that scale are modelled: 0 is
    /// [`Sim::declare_war`], 2 sets the alliance both ways, and 1 clears
    /// both. SEAM: the original's `diplos` byte carries more than three
    /// values (met, tribute state) and none of that is here.
    pub fn set_diplo(&mut self, a: Player, b: Player, level: i32) {
        let (x, y) = (a as usize, b as usize);
        if x >= self.leaders.at_war.len() || y >= self.leaders.at_war.len() || x == y {
            return;
        }
        match level {
            0 => {
                self.leaders.allied[x][y] = false;
                self.leaders.allied[y][x] = false;
                self.declare_war(a, b);
            }
            2 => {
                self.leaders.at_war[x][y] = false;
                self.leaders.at_war[y][x] = false;
                self.leaders.allied[x][y] = true;
                self.leaders.allied[y][x] = true;
            }
            _ => {
                self.leaders.at_war[x][y] = false;
                self.leaders.at_war[y][x] = false;
                self.leaders.allied[x][y] = false;
                self.leaders.allied[y][x] = false;
            }
        }
    }

    /// **`Leader::init@006e3930`'s diplomacy row** for a leader with a
    /// nation (`param_2 >= 0`, the call `Setup::build_game` makes for every
    /// active player; item 4046, `docs/START.md` §2 step 6). For each other
    /// leader `u`, `diplos[u]` (`+0x74`, written as `puVar23[-0x18]` off
    /// `good_deeds` at `+0xd4`, lines 199–227 of the decompile) is
    ///
    /// - left alone in a Conquer-the-World or scenario game
    ///   (`semaphore[2] & 2`, [`crate::ai::Lobby::conquest`]);
    /// - else **2** when `LeaderData::is_team@006ebd30(u, 0)` at frame 0 — the same `team`
    ///   in `0..4`, refused under `team_style 7` to a seat on team 8;
    /// - else **1**, peace, when `RUSH_RULES` is set above the leader's
    ///   `starting_age`;
    /// - else **0**, war, when `Game::teams_locked@00594880` (team style
    ///   neither 0, 8 nor 11), and 1 otherwise.
    ///
    /// Only this leader's row is written; the other's own `Leader::init`
    /// writes the mirror. The diagonal is the crate's untouched `false`.
    /// SEAM: the arm after it, `semaphore[1] & 2` forcing war with every
    /// other leader, is not a lobby field and is not carried; nor are the
    /// loop's other writes (`treaties` = `reveal_map == 3` or'd with the
    /// team bit, `aggression` = the war bit), which the start block does
    /// not compare.
    pub fn init_diplos(&mut self, who: Player, seats: &[Seat]) {
        let lobby = &self.ai.lobby;
        if lobby.conquest {
            return;
        }
        let w = who as usize;
        let seat = |p: usize| seats.iter().find(|s| s.who as usize == p);
        let is_team = |a: usize, b: usize| -> bool {
            if a == b {
                return true;
            }
            let (Some(sa), Some(sb)) = (seat(a), seat(b)) else {
                return false;
            };
            if lobby.team_style == 7 && [sa, sb].iter().any(|s| s.flags & 1 != 0 && s.team == 8) {
                return false;
            }
            (0..4).contains(&sa.team) && sa.team == sb.team
        };
        let starting_age = match self.tech.get(w) {
            Some(p) => self.tables.tech_tree.starting_age(&self.tables.setup, p),
            None => lobby.starting_technology.min(7),
        };
        let teams_locked = !matches!(lobby.team_style, 0 | 8 | 11);
        let n = self.leaders.at_war.len();
        if w >= n {
            return;
        }
        for u in (0..n).filter(|&u| u != w) {
            let d = if is_team(w, u) {
                2
            } else if lobby.rush_rules != 0 && starting_age < lobby.rush_rules {
                1
            } else if teams_locked {
                0
            } else {
                1
            };
            self.leaders.at_war[w][u] = d == 0;
            self.leaders.allied[w][u] = d == 2;
        }
    }

    /// `Leader::set_age(n)` on one leader, which is all the `age` cheat
    /// does — the four epochs stay where they were (`docs/INPUT.md` §11).
    pub fn set_leader_age(&mut self, who: Player, n: i32) -> Vec<tech::Gained> {
        let row = self.tables.tech_tree.ages;
        self.set_leader_levels(who, &row, n)
    }

    /// `Leader::set_epoch(line, level)` — the `military`, `civic`,
    /// `commerce` and `science` cheats, and the four `library` spends.
    pub fn set_leader_epoch(&mut self, who: Player, cat: i32, level: i32) -> Vec<tech::Gained> {
        let Some(line) = tech::Line::of(cat) else {
            return Vec::new();
        };
        let row = self.tables.tech_tree.epochs[line.index()];
        self.set_leader_levels(who, &row, level)
    }

    /// The body `Leader::set_age@006d25a0` and `Leader::set_epoch@006d26f0`
    /// share: lose every level of the row at or above `n`, highest first,
    /// then raise the rest through **a whole `Leader::gain_tech` per step**
    /// (`gain_tech(this, t, 0, 0, ·, 1)`), then drop what no longer
    /// follows.
    ///
    /// The whole call is the point (item 644). Each step carries
    /// `gain_tech`'s tail: the starting grant of a good whose prerequisite
    /// is the step ([`Sim::pay_arriving_goods`] — the Classical age pays
    /// knowledge and metal), the unit upgrades, the Science re-pricing, the
    /// age's snap, and the economy flag `0x2000000`. This crate raised the
    /// levels in the tree alone, so `library who=1 2` left who=1 holding
    /// knowledge 0 and metal 0 where the original held 100 of each; on
    /// run157's 1018 the goody lottery then paid metal, the good at 0, where
    /// the original's paid food, and on 1176 a Woodcutter's Camp at 70 food
    /// was one this crate's AI could not afford. Item 552 had wired the
    /// border half of the tail on the epoch path, and the age path had
    /// none of it.
    pub(crate) fn set_leader_levels(
        &mut self,
        who: Player,
        row: &[Option<tech::TypeId>; tech::LEVELS],
        n: i32,
    ) -> Vec<tech::Gained> {
        let w = who as usize;
        if w >= self.tech.len() {
            return Vec::new();
        }
        let n = n.clamp(0, tech::LEVELS as i32) as usize;
        for &t in row[n..].iter().rev().flatten() {
            if self
                .tables
                .tech_tree
                .has_tech(&self.tables.setup, &self.tech[w], t)
            {
                self.tables
                    .tech_tree
                    .lose_tech(&self.tables.setup, &mut self.tech[w], t);
            }
        }
        let mut events = Vec::new();
        for &t in row[..n].iter().flatten() {
            if !self
                .tables
                .tech_tree
                .has_tech(&self.tables.setup, &self.tech[w], t)
            {
                events.extend(self.gain_tech(who, t));
            }
        }
        self.tables
            .tech_tree
            .drop_unfollowed(&self.tables.setup, &mut self.tech[w]);
        self.apply_gained(who);
        events
    }

    /// Sets two players at war with each other.
    pub fn declare_war(&mut self, a: Player, b: Player) {
        let changed = !self.leaders.at_war[a as usize][b as usize];
        self.leaders.at_war[a as usize][b as usize] = true;
        self.leaders.at_war[b as usize][a as usize] = true;
        // `Leader::set_diplo` → `Armies::diplo_change` on the declarer
        // (`docs/ARMY.md` §15.9).
        if changed && (a as usize) < self.ai.armies.len() {
            self.armies_diplo_change(a);
        }
    }
}

#[cfg(test)]
mod tests;
