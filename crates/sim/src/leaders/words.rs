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

    /// `city_name` (`+0x40c`): how many cities this leader's slot has named
    /// for its tribe (`docs/LEADERS.md` §1).
    pub fn leader_city_name(&self, who: Player) -> i32 {
        self.leaders
            .city_name
            .get(usize::from(who))
            .copied()
            .unwrap_or(0)
    }

    /// `City::generate_name@00735c90`'s two stores to `city_name`, run from
    /// `City::init` when `Build::activate`'s first argument is 0 — the start
    /// city and a city completed by construction, never a capture
    /// (`docs/LEADERS.md` §1.2). The step itself is [`name_step`].
    ///
    /// Exempt, in the original's order: a Forbidden City (`is(0x213)`; the
    /// crate's `building_is_city` holds for it, §1.3); Conquer the World for
    /// a non-barbarian tribe; a tribe with no city list.
    ///
    /// SEAM, under the assumption §1.3 records: the tribe's city-name list
    /// is not in the crate, so every list is taken as present and longer
    /// than any count (`n < L`); and the barbarian tribe test under Conquer
    /// the World is not carried, so a conquest game exempts every tribe.
    pub(crate) fn generate_city_name(&mut self, b: usize) {
        if self.building_is(b, crate::cities::build::Ident::ForbiddenCity) {
            return;
        }
        if self.ai.lobby.conquest {
            return;
        }
        let own = usize::from(self.buildings[b].owner);
        let tribes: Vec<usize> = self.tech.iter().map(|t| t.tribe).collect();
        name_step(&mut self.leaders.city_name, &tribes, own);
    }
}

/// `generate_name`'s step and second store on the per-slot counts, with
/// `tribes[k]` each slot's tribe (`docs/LEADERS.md` §1.2).
///
/// The winner is the slot of the owner's tribe, among slots 0..8 alone,
/// with the largest count (strict `>` from −1, so a tie keeps the lowest
/// slot) if that count is above 0, else the owner. The winner steps; then
/// the owner takes 1 if it has never named a city and the winner's new
/// count `n` is at most 1 or the owner's count was 0 before the step.
pub(crate) fn name_step(names: &mut [i32], tribes: &[usize], own: usize) {
    if own >= names.len() || own >= tribes.len() {
        return;
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
    if (n <= 1 || prev == 0) && names[own] == 0 {
        names[own] = 1;
    }
}

#[cfg(test)]
mod tests {
    use super::name_step;

    /// The counter's two stores: two slots of one tribe and one of another
    /// (`docs/LEADERS.md` §1.2).
    #[test]
    fn a_tribe_mate_takes_the_step_and_the_owner_takes_one() {
        let tribes = [4, 4, 11];
        let mut names = [0; 3];
        // Distinct tribes, each naming its first city: the owner steps.
        name_step(&mut names, &tribes, 0);
        name_step(&mut names, &tribes, 2);
        assert_eq!(names, [1, 0, 1]);
        // Slot 1 shares slot 0's tribe: slot 0 wins the step (2), and slot
        // 1, which had named nothing, takes 1.
        name_step(&mut names, &tribes, 1);
        assert_eq!(names, [2, 1, 1]);
        // Slot 0 leads the tribe, so slot 1's next city steps slot 0.
        name_step(&mut names, &tribes, 1);
        assert_eq!(names, [3, 1, 1]);
        // A tie keeps the lowest slot.
        let mut tied = [2, 2, 0];
        name_step(&mut tied, &tribes, 1);
        assert_eq!(tied, [3, 2, 0]);
    }
}
