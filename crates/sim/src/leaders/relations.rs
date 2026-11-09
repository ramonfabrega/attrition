//! Diplomacy on the `Sim`: the players, war, alliance, treaties, and the
//! leader's age and epoch (`docs/STRUCTURE.md` §3, `leaders/`).
//!
//! Moved out of `lib.rs` unchanged by item 4014 (`docs/STRUCTURE.md` §5.6).

use crate::*;

impl Sim {
    /// Whether `who` is at war with `p` — false for an owner outside the
    /// player table (gaia's units path through territory too).
    pub fn at_war_with(&self, who: Player, p: Player) -> bool {
        self.at_war
            .get(who as usize)
            .and_then(|row| row.get(p as usize))
            .copied()
            .unwrap_or(false)
    }

    /// Sets two players as mutual allies — both `diplos` entries at 2.
    pub fn make_allies(&mut self, a: Player, b: Player) {
        self.allied[a as usize][b as usize] = true;
        self.allied[b as usize][a as usize] = true;
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
        if let Some(x) = self.treaties.get_mut(a).and_then(|r| r.get_mut(b)) {
            *x |= bits;
        }
        if let Some(x) = self.treaties.get_mut(b).and_then(|r| r.get_mut(a)) {
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
        self.allied
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
        if x >= self.at_war.len() || y >= self.at_war.len() || x == y {
            return;
        }
        match level {
            0 => {
                self.allied[x][y] = false;
                self.allied[y][x] = false;
                self.declare_war(a, b);
            }
            2 => {
                self.at_war[x][y] = false;
                self.at_war[y][x] = false;
                self.allied[x][y] = true;
                self.allied[y][x] = true;
            }
            _ => {
                self.at_war[x][y] = false;
                self.at_war[y][x] = false;
                self.allied[x][y] = false;
                self.allied[y][x] = false;
            }
        }
    }

    /// `Leader::set_age(n)` on one leader, which is all the `age` cheat
    /// does — the four epochs stay where they were (`docs/INPUT.md` §11).
    pub fn set_leader_age(&mut self, who: Player, n: i32) -> Vec<tech::Gained> {
        let row = self.tech_tree.ages;
        self.set_leader_levels(who, &row, n)
    }

    /// `Leader::set_epoch(line, level)` — the `military`, `civic`,
    /// `commerce` and `science` cheats, and the four `library` spends.
    pub fn set_leader_epoch(&mut self, who: Player, cat: i32, level: i32) -> Vec<tech::Gained> {
        let Some(line) = tech::Line::of(cat) else {
            return Vec::new();
        };
        let row = self.tech_tree.epochs[line.index()];
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
            if self.tech_tree.has_tech(&self.setup, &self.tech[w], t) {
                self.tech_tree.lose_tech(&self.setup, &mut self.tech[w], t);
            }
        }
        let mut events = Vec::new();
        for &t in row[..n].iter().flatten() {
            if !self.tech_tree.has_tech(&self.setup, &self.tech[w], t) {
                events.extend(self.gain_tech(who, t));
            }
        }
        self.tech_tree
            .drop_unfollowed(&self.setup, &mut self.tech[w]);
        self.apply_gained(who);
        events
    }

    /// Sets two players at war with each other.
    pub fn declare_war(&mut self, a: Player, b: Player) {
        let changed = !self.at_war[a as usize][b as usize];
        self.at_war[a as usize][b as usize] = true;
        self.at_war[b as usize][a as usize] = true;
        // `Leader::set_diplo` → `Armies::diplo_change` on the declarer
        // (`docs/ARMY.md` §15.9).
        if changed && (a as usize) < self.armies.len() {
            self.armies_diplo_change(a);
        }
    }
}
