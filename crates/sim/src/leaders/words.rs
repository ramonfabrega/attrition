//! **The leader record's lobby words and small counters** — `cannon_time`,
//! `handicap`, `multi_diff` (`docs/LEADERS.md` §16), `misery` (§12.3) and
//! `city_name` (§1), item 4162.
//!
//! Each is a leaf of `LeaderData` the leaders line of the sync checksum
//! walks (`docs/CHECKSUM.md` §20), and all but `cannon_time` are printed by
//! `LeaderData::log_data@006e5110`, so `rondata::diff::leader` compares them
//! on every kept dump that prints the record. They decide nothing in a solo
//! game: `get_diff` answers the lobby's byte either way, `handicap` has no
//! reader without the multiplayer bit, `misery` is written 0 by every
//! store, and `city_name` only chooses a city's name.

use crate::{Player, Sim};

impl Sim {
    /// `multi_diff` (`+0x50`): `Game::init_handicaps@0058abf0` writes the
    /// lobby's `difficulty` byte on every seated slot of a solo random map
    /// (no scenario, no multiplayer bit, no bit 26), human or computer, once
    /// per game build; nothing steps it (`docs/LEADERS.md` §16.2). An
    /// unseated slot keeps the clear's 0.
    ///
    /// SEAM: the multiplayer arm (`aidiff[i]`, else 2), `DropControl::
    /// process_drop`'s 3 and the trigger and scenario stores are not
    /// carried; the crate seats a solo game.
    pub fn leader_multi_diff(&self, who: Player) -> i32 {
        if usize::from(who) < self.economy.ledgers.len().min(8) {
            self.ai.lobby.difficulty
        } else {
            0
        }
    }

    /// `handicap` (`+0x4c`): `Game::init_handicaps`' formula over the lobby
    /// `Player` records (`docs/LEADERS.md` §16.2). With one person at the
    /// lobby's handicap 0 every `sum` and `top` is 0, so every slot stores
    /// 0, and every kept dump prints 0 on every leader.
    ///
    /// SEAM: the lobby's per-player `handicap`, `team` and `flags` are not
    /// carried (`ai::Lobby` has none), so the formula's other branches are
    /// not ported; a skirmish set with a handicap would part here.
    pub fn leader_handicap(&self, _who: Player) -> i32 {
        0
    }

    /// `cannon_time` (`+0x48`): `Leader::init@006e3930` seeds every slot
    /// with the `cannontimes` row `CANNON_TIMES` indexes, and only
    /// `TurnControl::start_cannon_time@00956500` on command 0x4d steps it,
    /// which `CommandManager::issue_cannon_time` sends only under the
    /// multiplayer bit (`docs/LEADERS.md` §16.2). `log_data` does not print
    /// it, so the checksum line is its only oracle.
    ///
    /// SEAM: no 0x4d arm; a multiplayer stream would step it.
    pub fn leader_cannon_time(&self, _who: Player) -> i32 {
        self.ai.lobby.cannon_time
    }

    /// `misery` (`+0x7ec`): every store writes 0 — `Leader::calc_pop_cap`'s
    /// first, `gain_tech`'s epoch arm, `Leader::init`, the clear — and
    /// nothing that decides anything reads it (`docs/LEADERS.md` §12.3).
    pub fn leader_misery(&self, _who: Player) -> i32 {
        0
    }

    /// `pop_issues` (`+0x7e8`): this frame's capped visits — the building
    /// arm in [`Sim::advance_slot_with`], the carrier arm in
    /// `carrier_queue_tick` — reset at a live leader's step
    /// (`docs/LEADERS.md` §12.2). The dump prints the whole frame's count.
    ///
    /// SEAM: `Army::find_target`'s ×4 still reads 0 (parked 4162c).
    pub fn leader_pop_issues(&self, who: Player) -> i32 {
        self.leaders
            .pop_issues
            .get(usize::from(who))
            .copied()
            .unwrap_or(0)
    }

    /// `city_name` (`+0x40c`): how many cities this leader's slot has named
    /// for its tribe (`docs/LEADERS.md` §1).
    pub fn leader_city_name(&self, who: Player) -> i32 {
        self.leaders
            .city_name
            .get(usize::from(who))
            .copied()
            .unwrap_or(0)
    }

    /// `City::generate_name@00735c90`, run from `City::init` when
    /// `Build::activate`'s first argument is 0 — the start city and a city
    /// completed by construction, never a capture (`docs/LEADERS.md`
    /// §1.2): the counter's step ([`name_step`]) and the founding name of
    /// city `c` ([`founding_name`]), written where a rename writes
    /// (`crate::cities::rename`).
    ///
    /// Exempt, in the original's order: a Forbidden City (`is(0x213)`; the
    /// crate's `building_is_city` holds for it, §1.3), which takes its
    /// type's name (`type +0x74`); Conquer the World for a non-barbarian
    /// tribe, which takes `ConquestGame::get_city_name` from frame 1. A
    /// tribe file with no `<CITIES>` names the city after the tribe and
    /// steps nothing.
    ///
    /// SEAM: neither exempt name is carried (the type's display name and
    /// the campaign's list are not in the crate), so both cities stay
    /// unnamed; and a tribe whose list was never loaded (a hand-built
    /// tree) steps as if its list were longer than any count and names
    /// nothing.
    pub(crate) fn generate_city_name(&mut self, b: usize, c: usize) {
        if self.building_is(b, crate::cities::build::Ident::ForbiddenCity) {
            return;
        }
        let own = usize::from(self.buildings[b].owner);
        let tribes: Vec<usize> = self.tech.iter().map(|t| t.tribe).collect();
        let tribe = tribes
            .get(own)
            .and_then(|&t| self.tables.tech_tree.tribes.get(t));
        if self.ai.lobby.conquest && !tribe.is_some_and(|t| t.barbarian) {
            return;
        }
        let Some((list, name)) =
            tribe.and_then(|t| t.cities.as_ref().map(|l| (l.clone(), t.name.clone())))
        else {
            name_step(&mut self.leaders.city_name, &tribes, own, None);
            return;
        };
        let written = if list.is_empty() {
            name
        } else {
            match founding_name(
                &mut self.leaders.city_name,
                &tribes,
                own,
                list.len(),
                self.game_seed,
            ) {
                Some(Founding::Listed(i)) => list[i].clone(),
                Some(Founding::Numbered(k)) => format!("{name}{SEPARATOR}{k}"),
                None => return,
            }
        };
        self.cities.names.insert(c, written);
    }
}

/// `loc_str_array_orig +0x15e0`, what `generate_name` puts between the
/// tribe's name and a number: one space, read off ch94p's packet (item
/// 4416).
pub const SEPARATOR: &str = " ";

/// The name [`founding_name`] chose: the list's entry at a 0-based index,
/// or the tribe's name and a number.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Founding {
    Listed(usize),
    Numbered(i32),
}

/// `generate_name`'s choice from a list of `len` names, after the step
/// ([`name_step`]) it runs first (`docs/LEADERS.md` §1.2), with `seed`
/// `world->seed` ([`Sim::game_seed`]). With `n` the winner's new count
/// and `prev` the owner's count before it:
///
/// 1. `n >= len`: the tribe's name and `n + 1`, and no second store.
/// 2. `n <= 1` or `prev == 0`: index `1 +` the slots below the owner of
///    its tribe; the second store.
/// 3. otherwise: `m` the slots of 0..8 of the owner's tribe, `M = len −
///    m + 1`, `x = (tribe + seed) mod M + 1` (signed), then the eight-bit
///    Galois step `x ^= 0x170` on odd, `x >>= 1`, repeated, rejecting `x
///    >= M` and stopping on the `(n − 1)`th value kept; index `x + m`.
/// 4. an index past `len` is the tribe's name and `index + 1`; else the
///    list's `index − 1`th.
///
/// `None` only where the original would not return: `M <= 1` (a divide
/// by zero, or a step that keeps nothing), which needs eight slots of one
/// tribe against a list of eight.
pub(crate) fn founding_name(
    names: &mut [i32],
    tribes: &[usize],
    own: usize,
    len: usize,
    seed: i32,
) -> Option<Founding> {
    let len = i32::try_from(len).ok()?;
    let (n, prev) = name_step(names, tribes, own, Some(len))?;
    if n >= len {
        return Some(Founding::Numbered(n + 1));
    }
    let slots = tribes.len().min(8);
    let index = if n <= 1 || prev == 0 {
        1 + (0..own.min(slots))
            .filter(|&k| tribes[k] == tribes[own])
            .count() as i32
    } else {
        let m = (0..slots).filter(|&k| tribes[k] == tribes[own]).count() as i32;
        let big_m = len - m + 1;
        if big_m <= 1 {
            return None;
        }
        let mut x = (tribes[own] as i32).wrapping_add(seed) % big_m + 1;
        let mut kept = 1;
        // The step is a maximal eight-bit LFSR on a start below 256, so a
        // value in 1..M recurs within 255 steps; the bound only keeps a
        // malformed list from spinning.
        let mut steps = 0u32;
        loop {
            if x & 1 != 0 {
                x ^= 0x170;
            }
            x >>= 1;
            steps += 1;
            if x < big_m {
                kept += 1;
                if kept == n {
                    break;
                }
            } else if steps > 0x10000 {
                return None;
            }
        }
        x + m
    };
    Some(if index > len {
        Founding::Numbered(index + 1)
    } else {
        Founding::Listed(index as usize - 1)
    })
}

/// 256.0, `0x43800000`: `Leader::calc_anti_attrition`'s first store
/// (`docs/LEADERS.md` §6.2).
pub const ANTI_ATT_BASE: u32 = 0x4380_0000;

/// 100.0, `0x42c80000`, the multiplier the chain reads from `.rdata`
/// (`0xb696ac`).
const HUNDRED: u32 = 0x42c8_0000;

impl Sim {
    /// `anti_att` (`+0x7f4`) as its single's bits.
    pub fn leader_anti_att(&self, who: Player) -> u32 {
        self.leaders
            .anti_att
            .get(usize::from(who))
            .copied()
            .unwrap_or(0)
    }

    /// `Leader::calc_anti_attrition@006cdcc0`: `anti_att` recomputed from
    /// 256.0 over the four sources this leader holds (`docs/LEADERS.md`
    /// §6.2). It runs where the original's does — `Leader::init`'s tail
    /// (here [`Sim::set_tribe`] and [`Sim::start_techs`]) and the head of
    /// `calc_unit_stats` on the `0x4000000` fold
    /// ([`Sim::calc_unit_stats`]) — and every source changes only at an
    /// event that raises that flag: a tech's gain, a rare's gather, a
    /// wonder's activation or close.
    ///
    /// The reader, `UnitData::get_attrition`, divides the single itself
    /// ([`crate::territory::attrition::Resistance::Single`], written here
    /// with `has_preq(FORAGING_1)`; item 4159's third residue). The
    /// conquest arms (`rare_conquest`, `conquest_wonders`,
    /// `conquest_racial_powers`) and `leader_flags2 & 0x40` are not carried.
    pub(crate) fn calc_anti_attrition(&mut self, who: Player) {
        let w = usize::from(who);
        if w >= self.leaders.anti_att.len() || w >= self.tech.len() {
            return;
        }
        let disabled = self
            .territory
            .players
            .get(w)
            .is_some_and(|p| p.take_disabled);
        let rows = self.tables.tech_tree.roles.foraging_preq;
        let tier = self.bonus_level(who, &rows);
        let liberty = self.wonders_held(who) & (1 << crate::tech::wonder::STATUE_OF_LIBERTY) != 0;
        let mongol =
            self.tables
                .tech_tree
                .has_tribe_bonus(&self.tables.setup, &self.tech[w], MONGOL_BONUS);
        let titanium = self.has_rare(who, TITANIUM);
        self.leaders.anti_att[w] = anti_attrition(
            &self.tables.tuning,
            disabled,
            tier,
            [liberty, mongol, titanium],
        );
        // The bleed reads the record (item 4159's third residue): the
        // resistance is the single itself, and `has_preq(FORAGING_1)` the
        // first row of the ladder.
        let forage = rows[0].is_some_and(|t| {
            self.tables
                .tech_tree
                .has_tech(&self.tables.setup, &self.tech[w], t)
        });
        if let Some(p) = self.territory.players.get_mut(w) {
            p.resistance =
                crate::territory::attrition::Resistance::Single(self.leaders.anti_att[w]);
            p.foraging_1 = forage;
        }
    }
}

/// `has_tribe_bonus(17)`, the Mongols' (`docs/LEADERS.md` §6.2).
const MONGOL_BONUS: usize = 17;

/// Titanium's `TypeIndex`: `has_rare(36)`, bit 30 of the rare mask.
const TITANIUM: usize = 36;

/// `calc_anti_attrition`'s chain on its inputs: 256.0, then for the tier
/// (`attrition_upgrade[tier − 1]`), the Statue of Liberty, the Mongol bonus
/// and titanium in that order, each held source's `p` either ends the chain
/// at 0 (`p ≥ 100`, signed) or takes `x = x × 100.0 / (100 − p)` in scalar
/// single precision, rounded to nearest even (`docs/LEADERS.md` §6.2, float
/// class 2: every input is an integer the simulation holds). Disabled is 0.
pub fn anti_attrition(
    t: &crate::core::tuning::Tuning,
    disabled: bool,
    tier: usize,
    [liberty, mongol, titanium]: [bool; 3],
) -> u32 {
    use crate::num::single::Single;
    if disabled {
        return 0;
    }
    let mut x = Single::from_bits(ANTI_ATT_BASE);
    let tier_pct =
        (tier > 0).then(|| t.attrition_upgrade[(tier - 1).min(t.attrition_upgrade.len() - 1)]);
    let sources = [
        tier_pct,
        liberty.then_some(t.liberty_attrition),
        mongol.then_some(t.mongol_attrition),
        titanium.then_some(t.titanium_attrition),
    ];
    for p in sources.into_iter().flatten() {
        if p >= 100 {
            return 0;
        }
        x = x
            .mulss(Single::from_bits(HUNDRED))
            .divss(Single::from_i32(100 - p));
    }
    x.bits()
}

/// `generate_name`'s step and second store on the per-slot counts, with
/// `tribes[k]` each slot's tribe (`docs/LEADERS.md` §1.2).
///
/// The winner is the slot of the owner's tribe, among slots 0..8 alone,
/// with the largest count (strict `>` from −1, so a tie keeps the lowest
/// slot) if that count is above 0, else the owner. The winner steps; then,
/// unless `n` reaches the list's length `len` (that arm names by number and
/// leaves, `7361a6`), the owner takes 1 if it has never named a city and
/// the winner's new count `n` is at most 1 or the owner's count was 0
/// before the step. `None` for a list never loaded. Answers `(n, prev)`,
/// `prev` the owner's count before the step.
pub(crate) fn name_step(
    names: &mut [i32],
    tribes: &[usize],
    own: usize,
    len: Option<i32>,
) -> Option<(i32, i32)> {
    if own >= names.len() || own >= tribes.len() {
        return None;
    }
    let mut best = -1;
    let mut at = own;
    for k in 0..names.len().min(tribes.len()).min(8) {
        if tribes[k] == tribes[own] && names[k] > best {
            best = names[k];
            at = k;
        }
    }
    let winner = if best > 0 { at } else { own };
    let prev = names[own];
    names[winner] += 1;
    let n = names[winner];
    if len.is_none_or(|l| n < l) && (n <= 1 || prev == 0) && names[own] == 0 {
        names[own] = 1;
    }
    Some((n, prev))
}

#[cfg(test)]
mod tests;
