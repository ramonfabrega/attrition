//! **The score and its eleven parts** — item 4187, step VI of the leaders
//! line (`docs/LEADERS.md` §18): `score` (`+0x18`) and `score_explored`
//! through `score_combat` (`+0x1c..+0x44`).
//!
//! Eleven are recounted by `Leader::compute_score@006ec560` on a leader's
//! recompute frame (frame 0, or `(who + frame) % 10 == 0`), which
//! `Leaders::strategy_all` calls for every leader in play, humans included
//! ([`Sim::strategy_all`]). The units part waits on the dirty bit a unit's
//! birth or close raises (`leader_flags & 0x800000`, [`Score::dirty_units`]),
//! the upgrades and research parts on the one `gain_tech` raises
//! (`0x1000000`, [`crate::tech::PlayerTech::score_dirty`]); the buildings,
//! wonders, economy and territory parts are recounted every time.
//! `score_combat` is the one accumulator: the kill and raze arms of
//! `Object::take_damage` and `Object::do_damage`, sabotage's two arms and
//! counterintelligence move it by a fifth of the victim's kill value
//! ([`Sim::note_score_kill`]).
//!
//! `LeaderData::log_data` prints `score` alone; the eleven parts have the
//! checksum's leaders line as their only oracle.
//!
//! SEAM: `Game::armageddon` (`+0x6e0`) is not carried, so the Armageddon arm
//! (every part but `score_units_2` zeroed once the count reaches
//! `get_armageddon()`) never fires: it needs nukes used past the count. The
//! `Game+0x820 & 0x40` latch (game over, pause, a playback's `read_package`) that forces a
//! recompute every frame is not carried either.

use crate::{Player, Sim};

/// The twelve leaves in the record's order (`+0x18..+0x44`).
pub const SCORE_LEAVES: [&str; 12] = [
    "score",
    "score_explored",
    "score_territory",
    "score_units",
    "score_units_2",
    "score_buildings",
    "score_economy",
    "score_pop",
    "score_unit_upgrades",
    "score_research",
    "score_wonders",
    "score_combat",
];

/// `compute_score`'s period: `(who + frame) % 10 == 0`.
pub const SCORE_PERIOD: i64 = 10;

/// The unit records the units and upgrades parts walk: `0x32..0x192`.
const UNIT_RECORDS: usize = 0x192 - 0x32;
/// The building records the buildings part walks: `0x19e..0x21f`.
const BUILD_RECORDS: usize = 0x21f - 0x19e;
/// The wonders, by building record: `0x20e..0x21f` (`is_wonder_type`'s
/// range).
const WONDER_RECORDS: std::ops::Range<usize> = (0x20e - 0x19e)..(0x21f - 0x19e);
/// The militia family, by unit record (`MILITIA 0x42`, `MINUTEMAN 0x43`,
/// `PARTISAN 0x44`): `get_score_value(t, 0)` prices them as the Citizen
/// (`0x32`, record 0).
const MILITIA_RECORDS: std::ops::RangeInclusive<usize> = (0x42 - 0x32)..=(0x44 - 0x32);
/// The techs the research part walks end at the first spell (`0x275`).
const TECH_END: i64 = 0x275;

/// One leader's score words, and the units part's dirty bit.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Score {
    pub score: i32,
    pub explored: i32,
    pub territory: i32,
    pub units: i32,
    pub units_2: i32,
    pub buildings: i32,
    pub economy: i32,
    pub pop: i32,
    pub upgrades: i32,
    pub research: i32,
    pub wonders: i32,
    pub combat: i32,
    /// `leader_flags & 0x800000`: a unit of this leader's was born
    /// (`Unit::init`) or closed (`Unit::close`) since the last recompute.
    pub dirty_units: bool,
}

impl Score {
    /// One of [`SCORE_LEAVES`].
    pub fn leaf(&self, name: &str) -> Option<i32> {
        Some(match name {
            "score" => self.score,
            "score_explored" => self.explored,
            "score_territory" => self.territory,
            "score_units" => self.units,
            "score_units_2" => self.units_2,
            "score_buildings" => self.buildings,
            "score_economy" => self.economy,
            "score_pop" => self.pop,
            "score_unit_upgrades" => self.upgrades,
            "score_research" => self.research,
            "score_wonders" => self.wonders,
            "score_combat" => self.combat,
            _ => return None,
        })
    }
}

/// `Σ costs[0..6]`, the written `COST` column with no factor.
fn cost_sum(base: &[i32; crate::economy::RESOURCES]) -> i32 {
    base.iter().fold(0i32, |a, &c| a.wrapping_add(c))
}

/// A price's two support amounts, `support_cost[0] + support_cost[1]`.
fn support_sum(price: &crate::economy::cost::Price) -> i32 {
    price
        .support
        .iter()
        .flatten()
        .fold(0i32, |a, &(_, n)| a.wrapping_add(n))
}

impl Sim {
    /// Raise a unit owner's `leader_flags & 0x800000` (`Unit::init`'s and
    /// `Unit::close`'s store, for an owner below 8).
    pub(crate) fn score_units_dirty(&mut self, who: Player) {
        if who < 8
            && let Some(s) = self.leaders.score.get_mut(usize::from(who))
        {
            s.dirty_units = true;
        }
    }

    /// One leader's score words, or the clear's for a slot out of range.
    pub fn leader_score(&self, who: Player) -> Score {
        self.leaders
            .score
            .get(usize::from(who))
            .cloned()
            .unwrap_or_default()
    }

    /// One of [`SCORE_LEAVES`] as the record holds it.
    pub fn leader_score_leaf(&self, who: Player, name: &str) -> Option<i64> {
        self.leaders
            .score
            .get(usize::from(who))
            .and_then(|s| s.leaf(name))
            .map(i64::from)
    }

    /// `TypeData::get_score_value(t, 0)` of a unit record: `UNIT_COST_FACTOR`
    /// times its written costs, the militia family's taken from the Citizen.
    fn unit_score_value(&self, r: usize) -> i32 {
        let src = if MILITIA_RECORDS.contains(&r) { 0 } else { r };
        let base = self
            .tables
            .unit_types
            .get(src)
            .map_or(0, |u| cost_sum(&u.price.base));
        self.tables.tuning.unit_cost_factor.wrapping_mul(base)
    }

    /// `get_score_value(t, 1)`, the research premium arm of a unit record:
    /// `((RESEARCH_PREMIUM × (UCF × costs)) / 256 × research_premium_cost)
    /// / 256`, each `/` truncating toward zero, the type's own costs.
    fn unit_research_value(&self, r: usize) -> i32 {
        let Some(u) = self.tables.unit_types.get(r) else {
            return 0;
        };
        let v = self
            .tables
            .tuning
            .unit_cost_factor
            .wrapping_mul(cost_sum(&u.price.base));
        let v = self.tables.tuning.research_premium.wrapping_mul(v) / 256;
        v.wrapping_mul(u.cols.research_premium_cost) / 256
    }

    /// `TypeData::get_score_value(t, 0)` of a building record:
    /// `BUILD_COST_FACTOR` times its written costs.
    fn build_score_value(&self, r: usize) -> i32 {
        let base = self
            .tables
            .build_types
            .get(r)
            .map_or(0, |b| cost_sum(&b.price.base));
        self.tables.tuning.build_cost_factor.wrapping_mul(base)
    }

    /// `num_buildings[r]` and `num_queued[0x19e + r]` (the placed sites) of
    /// one leader, every building record at once, each a `ushort`.
    fn building_counts(&self, who: Player) -> (Vec<i32>, Vec<i32>) {
        let mut built = vec![0i32; BUILD_RECORDS];
        let mut sites = vec![0i32; BUILD_RECORDS];
        for b in self.buildings.iter().filter(|b| b.alive && b.owner == who) {
            let Some(r) = b.ty.filter(|&r| r < BUILD_RECORDS) else {
                continue;
            };
            if b.active {
                built[r] += 1;
            } else {
                sites[r] += 1;
            }
        }
        let u16_of = |v: Vec<i32>| v.into_iter().map(|x| i32::from(x as u16)).collect();
        (u16_of(built), u16_of(sites))
    }

    /// **`get_kill_value(t, o)`** of a victim (§18.2): `value(t, 0)`, and
    /// for a player owner `o` its two support words times its count of the
    /// type — a unit's `num_units` and `num_queued`, a building's
    /// `num_buildings` and `num_queued` with `BUILD_SUPPORT_FACTOR` — the kill priced as the
    /// owner's next one. An owner past the eight leaders reads no count.
    fn kill_value(&self, victim: Victim, owner: Player) -> i32 {
        let w = usize::from(owner);
        match victim {
            Victim::Unit(r) => {
                let base = self.unit_score_value(r);
                let Some(m) = self.roster.muster.get(w).filter(|_| owner < 8) else {
                    return base;
                };
                let n = i32::from(m.by_type.get(r).copied().unwrap_or(0) as u16)
                    + i32::from(m.queued_by_type.get(r).copied().unwrap_or(0) as u16);
                let sc = self
                    .tables
                    .unit_types
                    .get(r)
                    .map_or(0, |u| support_sum(&u.price));
                sc.wrapping_mul(n).wrapping_add(base)
            }
            Victim::Building(r) => {
                let base = self.build_score_value(r);
                if owner >= 8 || w >= self.economy.ledgers.len() {
                    return base;
                }
                let (built, sites) = self.building_counts(owner);
                let n = built.get(r).copied().unwrap_or(0) + sites.get(r).copied().unwrap_or(0);
                let sc = self
                    .tables
                    .build_types
                    .get(r)
                    .map_or(0, |b| support_sum(&b.price));
                crate::economy::cost::kill_value_by(
                    base,
                    (sc, 0),
                    self.tables.tuning.build_support_factor,
                    Some((n as u16, 0)),
                )
            }
        }
    }

    /// **`score_combat`'s step** (§18.2): `v = kill(t, o) / 5`, `o` the
    /// victim's owner. With `ally_test`, the combat arms' rule: an ally of
    /// the killer (itself included) takes `v` off, floored at 0, anyone
    /// else adds it; without, the spells' plain add.
    pub(crate) fn note_score_kill(
        &mut self,
        killer: Player,
        victim: Victim,
        owner: Player,
        ally_test: bool,
    ) {
        if killer >= 8 || usize::from(killer) >= self.leaders.score.len() {
            return;
        }
        let v = self.kill_value(victim, owner) / 5;
        // SEAM: `LeaderData::is_ally(K, o)` for a gaia owner (8, 9) reads
        // past `diplos[8]` (§18.3's trap); a gaia victim is taken as not
        // allied here.
        let allied = ally_test && owner < 8 && self.is_ally(killer, owner);
        let s = &mut self.leaders.score[usize::from(killer)];
        s.combat = if allied {
            s.combat.wrapping_sub(v).max(0)
        } else {
            s.combat.wrapping_add(v)
        };
    }

    /// **`Leader::compute_score@006ec560`** (§18.2), `forced` its argument:
    /// nothing on a frame that is not the leader's; otherwise the parts in
    /// the original's order and their sum.
    pub(crate) fn compute_score(&mut self, who: Player, forced: bool) {
        let w = usize::from(who);
        if w >= self.leaders.score.len() || w >= self.tech.len() {
            return;
        }
        let frame = self.frame;
        if !(frame == 0 || forced || (i64::from(who) + frame) % SCORE_PERIOD == 0) {
            return;
        }
        let mut s = self.leaders.score[w].clone();
        s.explored = 0;
        let tech_dirty = self.tech[w].score_dirty;
        if s.dirty_units || tech_dirty {
            let (u, u2) = self.score_units_of(who);
            s.units = u;
            s.units_2 = u2;
            s.dirty_units = false;
        }
        let (b, wonders) = self.score_buildings_of(who);
        s.buildings = b;
        s.wonders = wonders;
        s.economy = self.score_economy_of(who);
        s.pop = 0;
        if tech_dirty {
            s.upgrades = self.score_upgrades_of(who);
            s.research = self.score_research_of(who);
            self.tech[w].score_dirty = false;
        }
        let h = &self.economy.holdings[w];
        s.territory = if h.land_size != 0 {
            h.territory.wrapping_mul(1000) / h.land_size
        } else {
            0
        };
        s.score = [
            s.combat,
            s.wonders,
            s.research,
            s.upgrades,
            s.economy,
            s.buildings,
            s.units,
            s.territory,
            s.pop,
            s.explored,
        ]
        .iter()
        .fold(0i32, |a, &x| a.wrapping_add(x));
        self.leaders.score[w] = s;
    }

    /// `Leader::compute_unit_score@006bc500`: every unit record with `n =
    /// num_units + num_queued` non-zero adds `(value·n + ((n−1)·sc·n)/2)/5`
    /// to `score_units`, and to `score_units_2` when the type attacks.
    fn score_units_of(&self, who: Player) -> (i32, i32) {
        let m = &self.roster.muster[usize::from(who)];
        let (mut all, mut armed) = (0i32, 0i32);
        for r in 0..UNIT_RECORDS.min(self.tables.unit_types.len()) {
            let n = i32::from(m.by_type.get(r).copied().unwrap_or(0) as u16)
                + i32::from(m.queued_by_type.get(r).copied().unwrap_or(0) as u16);
            if n == 0 {
                continue;
            }
            let ty = &self.tables.unit_types[r];
            let sc = support_sum(&ty.price);
            let ramp = (n - 1).wrapping_mul(sc).wrapping_mul(n) / 2;
            let u = self.unit_score_value(r).wrapping_mul(n).wrapping_add(ramp) / 5;
            all = all.wrapping_add(u);
            if ty.combat.attack != 0 {
                armed = armed.wrapping_add(u);
            }
        }
        (all, armed)
    }

    /// `Leader::compute_build_score@006bc3f0`: every building record with
    /// `n = num_buildings + num_queued` non-zero is `b = value·n +
    /// ((n−1)·(sc·BSF)·n)/2`, a wonder's `b / 3` to `score_wonders`, any
    /// other's `b / 5` to `score_buildings`.
    fn score_buildings_of(&self, who: Player) -> (i32, i32) {
        let (built, sites) = self.building_counts(who);
        let bsf = self.tables.tuning.build_support_factor;
        let (mut b_sum, mut w_sum) = (0i32, 0i32);
        for r in 0..BUILD_RECORDS.min(self.tables.build_types.len()) {
            let n = built[r] + sites[r];
            if n == 0 {
                continue;
            }
            let sc = support_sum(&self.tables.build_types[r].price).wrapping_mul(bsf);
            let ramp = (n - 1).wrapping_mul(sc).wrapping_mul(n) / 2;
            let b = self.build_score_value(r).wrapping_mul(n).wrapping_add(ramp);
            if WONDER_RECORDS.contains(&r) {
                w_sum = w_sum.wrapping_add(b / 3);
            } else {
                b_sum = b_sum.wrapping_add(b / 5);
            }
        }
        (b_sum, w_sum)
    }

    /// `Leader::compute_economy_score@006bc270`: each available good adds
    /// `clamp((bucket − starting) / 20, 0, 100) + min(income / 80, 200)`
    /// (no floor on the income term), then `25 ×` the rares held.
    fn score_economy_of(&self, who: Player) -> i32 {
        let w = usize::from(who);
        let l = &self.economy.ledgers[w];
        let h = &self.economy.holdings[w];
        let mut e = 0i32;
        for g in 0..crate::economy::RESOURCES {
            if !h.available[g] {
                continue;
            }
            let stock = (l.bucket[g].wrapping_sub(self.starting_good(who, g)) / 20).clamp(0, 100);
            let rate = (l.income[g] / 80).min(200);
            e = e.wrapping_add(stock).wrapping_add(rate);
        }
        e.wrapping_add(25 * l.rare.count_ones() as i32)
    }

    /// `Leader::compute_unit_upgrades_score@006bc1a0`: every unit record
    /// held or queued (`has_tech(t)` or an exact entry in a queue of an
    /// active building), not set at the start and not flag `h` (`unit_flags
    /// & 0x80`), adds `value(t, 1) / 5`.
    fn score_upgrades_of(&self, who: Player) -> i32 {
        let w = usize::from(who);
        let p = &self.tech[w];
        let tree = &self.tables.tech_tree;
        let mut sum = 0i32;
        for r in 0..UNIT_RECORDS.min(self.tables.unit_types.len()) {
            let ty = &self.tables.unit_types[r];
            let Some(t) = ty.tree else { continue };
            if p.tech_at_start.get(t).copied().unwrap_or(false) || ty.cols.unit_flags & 0x80 != 0 {
                continue;
            }
            let held =
                tree.has_tech(&self.tables.setup, p, t) || self.score_queue_holds(who, Some(r), t);
            if held {
                sum = sum.wrapping_add(self.unit_research_value(r) / 5);
            }
        }
        sum
    }

    /// `Leader::compute_research_score@006bc360`: every tech `0x220..0x275`
    /// held or queued, not set at the start, adds `TECH_COST_FACTOR ×
    /// costs / 5`.
    fn score_research_of(&self, who: Player) -> i32 {
        let w = usize::from(who);
        let p = &self.tech[w];
        let tree = &self.tables.tech_tree;
        let tcf = self.tables.tuning.tech_cost_factor;
        let mut sum = 0i32;
        for t in 0..tree.types.len() {
            if !tree.kind(t).is_tech() {
                continue;
            }
            let ti = self.tree_type_index(t);
            if !(0x220..TECH_END).contains(&ti) || p.tech_at_start[t] {
                continue;
            }
            if tree.has_tech(&self.tables.setup, p, t) || p.queued[t] > 0 {
                let v = tcf.wrapping_mul(cost_sum(&tree.types[t].cost));
                sum = sum.wrapping_add(v / 5);
            }
        }
        sum
    }

    /// `LeaderData::researching(t, −1, 0, 1)` for a unit type: an active
    /// building of `who`'s holding an entry of exactly `t` — a research of
    /// tree id `t`, or a train of unit record `rec`.
    fn score_queue_holds(&self, who: Player, rec: Option<usize>, t: usize) -> bool {
        self.buildings
            .iter()
            .filter(|b| b.alive && b.active && b.owner == who)
            .any(|b| {
                b.queue.items.iter().any(|i| {
                    !i.disband && (i.tech == Some(t) || (i.tech.is_none() && Some(i.ty) == rec))
                })
            })
    }
}

/// A victim's type, by its own record (`get_kill_value`'s vslot `+0x80`
/// picks the unit or the building form).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Victim {
    Unit(usize),
    Building(usize),
}

#[cfg(test)]
mod tests;
