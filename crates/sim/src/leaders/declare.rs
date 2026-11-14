//! **The declaration** — item 4159's step VII-b1 (`docs/LEADERS.md` §7):
//! `Leader::action_declare@006dab50`, its price `Leader::pay_dow@006d2b10`
//! and its purse test `LeaderData::afford_dow@006d5ce0`, and the three
//! leaves the first of them writes: `dow` (`+0x270`, by target), `blacken`
//! (`+0x20c`) and `broke_alliance` (`+0x2b0`, by target).
//!
//! The counting and the stamp live here and never in [`Sim::set_diplo`],
//! whose callers include an accepted treaty, the peace's end and the
//! console verb, none of which counts or stamps (§7.2).

use crate::economy::Resource;
use crate::{Player, Sim};

/// `DECLARE_WAR`, the first of the eight "Declare War" crafts: `TypeIndex`
/// `0x29c + t` is the declaration on slot `t` (`craftrules.xml`'s 40th to
/// 47th `CRAFT`, `COST2 8g` each).
pub const DECLARE_WAR: usize = 0x29c;

/// `CANCEL_ALLIANCE`, the first of the eight "Cancel Alliance" crafts:
/// `0x2a4 + t` (`craftrules.xml`'s 48th to 55th, `COST2 8g` each).
pub const CANCEL_ALLIANCE: usize = 0x2a4;

/// `Constants::ally_to_war_delay`: `Constants::init@00569a90`'s
/// `get_item("ALLY_TO_WAR_DELAY")`, a name no shipped `rules.xml` carries,
/// so the default −1. The refusal compares `frame − stamp < delay`, signed,
/// and cannot fire at −1 for a stamp in the past (`docs/LEADERS.md` §7.2).
pub const ALLY_TO_WAR_DELAY: i64 = -1;

/// `invaders[t] < 5`, signed: block A counts a declaration at peace only
/// while fewer than five of the target's units stand in the declarer's
/// land (`006dad14`).
const INVADERS_JUSTIFY: i32 = 5;

/// What one invading unit takes off a declaration's price, per good:
/// `get_cost`'s `invaders[t] · −10` (its declare-war arm, after `LAB_00666230`), a literal, not
/// `SPELL_COST_FACTOR`.
const INVADER_DISCOUNT: i32 = 10;

/// The leaves [`Sim::leader_declare_leaf`] answers, in the record's order:
/// the declaration's three (step VII-b1) and the accepted peace's
/// `made_peace` (step VII-c).
pub const DECLARE_LEAVES: [&str; 4] = ["blacken", "dow", "broke_alliance", "made_peace"];

/// One leader's relation leaves (`docs/LEADERS.md` §7.1). `Leader::init`
/// zeroes all of them but `hire_who`, which it sets to −1 per slot; nothing
/// decays them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Declared {
    /// `dow` (`+0x270`), by target: the counted declarations.
    pub dow: [i32; 8],
    /// `blacken` (`+0x20c`): the counted declarations on a target already
    /// counted once — read before the block's own `dow` increment.
    pub blacken: i32,
    /// `broke_alliance` (`+0x2b0`), by target: the frame an alliance was
    /// broken through a declaration. The target's mirror slot is not
    /// written.
    pub broke_alliance: [i64; 8],
    /// `good_deeds` (`+0xd4`), by target: the offers arm's running balance
    /// (`Leader::diplomacy@006bc950`), the seed of the next valuation and
    /// charged by each deal it accepts.
    pub good_deeds: [i32; 8],
    /// `made_peace` (`+0x2d0`), by the other party: the frame a peace
    /// treaty was executed (`Leader::action_respond@006d03c0`'s treaty arm,
    /// both records, even when the pair stays allied).
    pub made_peace: [i64; 8],
    /// `hire_stamp` (`+0x1b4`), by the hired-against slot: the frame an
    /// attack deal against it executed (the hire loop, both parties).
    pub hire_stamp: [i64; 8],
    /// `hire_who` (`+0x1d4`), by the hired-against slot: the other party of
    /// that deal, −1 for none.
    pub hire_who: [i32; 8],
}

impl Default for Declared {
    /// `Leader::init@006e3930`'s reset: every leaf 0, `hire_who` −1.
    fn default() -> Self {
        Declared {
            dow: [0; 8],
            blacken: 0,
            broke_alliance: [0; 8],
            good_deeds: [0; 8],
            made_peace: [0; 8],
            hire_stamp: [0; 8],
            hire_who: [-1; 8],
        }
    }
}

impl Sim {
    /// One of [`DECLARE_LEAVES`] as the leaders line and the record print
    /// it: `blacken` one word, `dow`, `broke_alliance` and `made_peace`
    /// eight.
    pub fn leader_declare_leaf(&self, who: Player, name: &str) -> Option<Vec<i64>> {
        let d = self.leaders.declare.get(who as usize)?;
        match name {
            "blacken" => Some(vec![i64::from(d.blacken)]),
            "dow" => Some(d.dow.iter().map(|&x| i64::from(x)).collect()),
            "broke_alliance" => Some(d.broke_alliance.to_vec()),
            "made_peace" => Some(d.made_peace.to_vec()),
            _ => None,
        }
    }

    /// **`Leader::action_declare@006dab50(me, t, level, quiet, force)`**
    /// (`docs/LEADERS.md` §7.2): `me` asks for `level` with `t` — 0 war, 1
    /// peace (an alliance cancelled).
    ///
    /// 1. An enemy already: nothing.
    /// 2. A level-0 ask, unforced, on an ally becomes level 1. On a
    ///    non-ally it is refused inside [`ALLY_TO_WAR_DELAY`] of the later
    ///    of the two `broke_alliance` stamps.
    /// 3. With `quiet` clear, a purse short of the price
    ///    ([`Sim::afford_dow`]) refuses. A level-0 ask under rush rules
    ///    (`Game::war_allowed`) refuses, or, forced, becomes level 1.
    /// 4. With `quiet` clear: [`Sim::pay_dow`], and the counting. Block A:
    ///    a level-0 ask on a non-ally with fewer than five of `t`'s units
    ///    in my land (`invaders[t]`) counts. Block B: still allied counts.
    ///    A count adds one to `blacken` when `dow[t]` was already nonzero,
    ///    then one to `dow[t]`. A forced level 0 on an ally pays twice and
    ///    counts in both blocks; no caller reaches it.
    /// 5. Still allied, `quiet` or not: `broke_alliance[t]` is this frame.
    /// 6. [`Sim::set_diplo`]`(me, t, level)`.
    ///
    /// **Only a human passes `quiet` clear**: the Declare command
    /// (`CommandPackage::process_declare@009472b0`, `action_declare(who,
    /// whom, treaty, 0, 0)`) and a human party's leg of `action_respond`'s
    /// hire loop. So `dow` and `blacken` move for a human declarer alone.
    ///
    /// SEAM: the team tail after step 6 — for each other leader in play, on
    /// the declarer's team or the target's and not both, `ally_diplo` down
    /// to `level` — needs `LeaderData::is_team` at frame 0, which is false
    /// between every two seats at `TEAM_STYLE 0` (no locked teams), so no
    /// style-0 game reaches it; it is not carried. The messages (the
    /// refusal's, the console target's chat and sound) are the interface's.
    pub fn action_declare(&mut self, me: Player, t: Player, level: i32, quiet: bool, force: bool) {
        let (m, x) = (me as usize, t as usize);
        if m >= self.leaders.declare.len() || x >= self.leaders.declare.len() || m == x {
            return;
        }
        let mut level = level;
        loop {
            if self.is_enemy(me, t) {
                return;
            }
            if level == 0 && !force {
                if self.is_ally(me, t) {
                    // The ally's request turned into peace, and asked again.
                    level = 1;
                    continue;
                }
                let stamp = self.leaders.declare[x].broke_alliance[m]
                    .max(self.leaders.declare[m].broke_alliance[x]);
                if stamp != 0 && self.frame - stamp < ALLY_TO_WAR_DELAY {
                    return;
                }
            }
            if !quiet && !self.afford_dow(me, t, level) {
                return;
            }
            if level != 0 || self.war_allowed() {
                break;
            }
            if !force {
                return;
            }
            level = 1;
        }
        if !quiet {
            self.pay_dow(me, t, level);
            let block_a = if !self.is_ally(me, t) {
                true
            } else if level == 0 {
                self.pay_dow(me, t, 0);
                true
            } else {
                false
            };
            let invaders = self.ai.ai[m].census.invaders.get(x).copied().unwrap_or(0);
            if block_a && level == 0 && (self.is_ally(me, t) || invaders < INVADERS_JUSTIFY) {
                self.count_declaration(m, x);
            }
            if self.is_ally(me, t) {
                self.count_declaration(m, x);
            }
        }
        if self.is_ally(me, t) {
            self.leaders.declare[m].broke_alliance[x] = self.frame;
        }
        self.set_diplo(me, t, level);
    }

    /// One counted declaration: `blacken` reads `dow[t]` before the
    /// increment.
    fn count_declaration(&mut self, m: usize, x: usize) {
        let d = &mut self.leaders.declare[m];
        if d.dow[x] != 0 {
            d.blacken += 1;
        }
        d.dow[x] += 1;
    }

    /// The craft a declaration of `level` on `t` is priced as:
    /// [`DECLARE_WAR`]` + t` at level 0, [`CANCEL_ALLIANCE`]` + t` else.
    fn dow_craft(t: usize, level: i32) -> usize {
        if level == 0 {
            DECLARE_WAR + t
        } else {
            CANCEL_ALLIANCE + t
        }
    }

    /// **`LeaderData::afford_dow@006d5ce0(t, level)`**: whether `me`'s
    /// purse meets the declaration's price on every good it can use.
    ///
    /// Free — true before any price — when the pair is at war either way,
    /// and for a level other than 0 when it is not allied both ways. Else
    /// each available good's bucket must hold [`Sim::dow_price`]; the first
    /// short good is what the refusal's warning names.
    pub fn afford_dow(&self, me: Player, t: Player, level: i32) -> bool {
        let (m, x) = (me as usize, t as usize);
        if m != x && (self.diplos(m, x) == 0 || self.diplos(x, m) == 0) {
            return true;
        }
        if level != 0 && m != x && (self.diplos(m, x) != 2 || self.diplos(x, m) != 2) {
            return true;
        }
        let craft = Self::dow_craft(x, level);
        let available = self.economy.holdings[m].available;
        Resource::ALL.iter().all(|&g| {
            !available[g.index()]
                || self.economy.ledgers[m].bucket[g.index()] >= self.dow_price(me, craft, g)
        })
    }

    /// **`Leader::pay_dow@006d2b10(t, level)`**: for each good `me` can
    /// use, the price off the bucket and off the escrow, each floored at
    /// zero. No purse test: [`Sim::afford_dow`] is the caller's.
    pub fn pay_dow(&mut self, me: Player, t: Player, level: i32) {
        let m = me as usize;
        let craft = Self::dow_craft(t as usize, level);
        let available = self.economy.holdings[m].available;
        for g in Resource::ALL {
            let i = g.index();
            if !available[i] {
                continue;
            }
            // The original prices twice, once per store; nothing between
            // the two moves the price.
            let due = self.dow_price(me, craft, g);
            let l = &mut self.economy.ledgers[m];
            l.bucket[i] = (l.bucket[i] - due).max(0);
            l.escrow[i] = (l.escrow[i] - due).max(0);
        }
    }

    /// **`TypeData::get_cost@00664090` on a declaration's craft** — `craft`
    /// one of [`DECLARE_WAR`]`..`[`CANCEL_ALLIANCE`]` + 8`, for `me`, of
    /// good `g`, with no object (`param_3 = −1`) and the redirect on:
    ///
    /// 1. `COST2[g] × SPELL_COST_FACTOR`;
    /// 2. × `LeaderData::cost_factor_dow@006da590`: `((t == me or allied
    ///    both ways) + (a Declare War craft)) × (ages + 1)`;
    /// 3. a Declare War on a non-ally: `− 10 × invaders[t]`, floored at 0;
    /// 4. the rares' tail ([`Sim::rare_goods_discounts`]: Sugar, Coal, Gold
    ///    or Iron on its good, then Gypsum on all);
    /// 5. the redirect: each good `me` cannot use whose table names `g`
    ///    adds its own price × the rate >> 8.
    ///
    /// The tail's other arms — the captured building's doubling, the
    /// missiles' wonder, the Indian elephants — test a unit or an object
    /// and pass a craft by. `has_spell` is assumed: all sixteen rows ship
    /// `PREQ0 None`, `TRIBE_MASK` all ones and no `FLAGS`, so
    /// `type_avail(craft, 1)` answers yes for every leader; its false arm,
    /// the generic price, is not carried.
    pub fn dow_price(&self, me: Player, craft: usize, g: Resource) -> i32 {
        self.dow_price_from(me, craft, g, 0)
    }

    /// [`Sim::dow_price`] with the redirect's goods already on the stack
    /// (`seen`), the guard `cost::redirected_cost` keeps.
    fn dow_price_from(&self, me: Player, craft: usize, g: Resource, seen: u8) -> i32 {
        let m = me as usize;
        let Some(row) = craft
            .checked_sub(crate::tech::SPELL_BASE)
            .and_then(|r| self.orders.spells.get(r))
        else {
            return 0;
        };
        let mut cost = row.cost2[g.index()] * self.tables.tuning.spell_cost_factor;
        let declare_war = (DECLARE_WAR..CANCEL_ALLIANCE).contains(&craft);
        let t = if declare_war {
            craft - DECLARE_WAR
        } else {
            craft - CANCEL_ALLIANCE
        };
        let allied = t == m || (self.diplos(m, t) == 2 && self.diplos(t, m) == 2);
        let ages = self.tech.get(m).map_or(0, |p| p.ages);
        cost *= (i32::from(allied) + i32::from(declare_war)) * (ages + 1);
        if declare_war && !self.is_ally(me, t as Player) {
            let invaders = self.ai.ai[m].census.invaders.get(t).copied().unwrap_or(0);
            cost = (cost - invaders * INVADER_DISCOUNT).max(0);
        }
        let (by, gypsum) = self.rare_goods_discounts(me);
        if by[g.index()] != 0 {
            cost = (100 - by[g.index()]) * cost / 100;
        }
        if gypsum != 0 {
            cost = (100 - gypsum) * cost / 100;
        }
        let h = &self.economy.holdings[m];
        for r in Resource::ALL {
            let i = r.index();
            if h.available[i] || seen & (1 << i) != 0 {
                continue;
            }
            let to = self.economy.redirects.of(r, h.discovered[i]);
            if to.good != g {
                continue;
            }
            let inner = self.dow_price_from(me, craft, r, seen | (1 << i));
            cost += crate::economy::cost::apply_rate(inner, to.rate);
        }
        cost
    }
}

#[cfg(test)]
mod tests;
