//! `Unit::think_spellcaster@005f27a0` — an AI caster deciding whether to
//! cast by itself (`docs/AI.md` §80.5, item 971).
//!
//! Every capture of the first pair was on Easiest, where the computer
//! leader's arm returns before its first test (`get_diff() < 2`), so the
//! function drew nothing on any frame of either long capture and this crate
//! skipped it (`docs/SCOUT.md` §13 item 10). At Toughest the AI's scout
//! reaches it on frame 0 from `Unit::think_scout+0x7c`, and the special
//! unit's arm throws a coin there — the second pair's first word.
//!
//! What is built is the computer leader's arm, in the listing's order:
//!
//! ```text
//! get_diff() < 2                                  -> 0
//! unit_masks & 1 (a decoy)                        -> 0
//! is(SPY):  cloaked? Counterintelligence on a friend in range × 5,
//!           then Bribe on an enemy unit in range × 5            (no draw)
//! not is_special:  the hero arm                                 (seam)
//! is_special:
//!     Sniper (0x281) castable, with the mana, on an enemy in range -> cast
//!     coin = Random::get(game_random, 0, 0xffff)        // +0x413
//!     coin % 3 != 0 and no army                   -> 0
//!     Counterintelligence (0x277) castable, with the mana, on a target
//!     in range                                    -> cast
//!     -> 0
//! ```
//!
//! SEAMS, each drawing nothing where it stands:
//! - **the targets**: every target search goes through
//!   [`Sim::spell_valid_target`], which refuses Bribe, Counterintelligence
//!   and the Sniper outright (`docs/GOLDEN.md` §27's packet has the first
//!   two refuse a staged Barracks); so no craft is cast from here yet, and
//!   what is reproduced is the draw and every return before it;
//! - **the Spy's cloak**: not carried, so a Spy is read as uncloaked and
//!   returns 0 — the original's Spy arm draws nothing either way;
//! - **the hero arm** (a General: Create Decoys, Forced March behind a
//!   `game_random` coin, Ambush): not built, and returns 0. No
//!   General stands in either capture of the second pair's opening;
//! - **the human arm**, which `Unit::think`'s special turn reaches: it
//!   draws nothing and casts only through the same refused targets.

use crate::Sim;

/// The special arm's coin, `Random::get` returning to `005f2bb3`.
pub const SITE_SPECIAL_COIN: &str = "Unit::think_spellcaster+0x413";

/// `TypeIndex` `0x3a`, the Spy.
const SPY: crate::tech::TypeId = 0x3a;

impl Sim {
    /// `Unit::think_spellcaster@005f27a0` for a computer leader's unit.
    /// Returns whether a cast was ordered (the original's 1).
    pub fn think_spellcaster(&mut self, u: usize) -> bool {
        let who = self.units[u].owner;
        if self.nation.get(who as usize).is_some_and(|n| n.human) {
            // SEAM: the human arm (module doc).
            return false;
        }
        if self.ai_difficulty() < 2 {
            return false;
        }
        if self.units[u].decoy {
            return false;
        }
        if self.unit_line_is(u, SPY) {
            // SEAM: the cloak (module doc); an uncloaked Spy returns 0.
            return false;
        }
        if !self.unit_is_special(u) {
            // SEAM: the hero arm (module doc).
            return false;
        }
        // The Sniper (`0x281`): castable, with the mana, on an enemy unit in
        // range -> cast. SEAM: the target search finds none (module doc).
        self.mark(SITE_SPECIAL_COIN);
        let coin = self.rng.roll();
        if coin % 3 != 0 && self.army_of(u).is_none() {
            return false;
        }
        // Counterintelligence (`0x277`, a Scout's own craft): castable, with
        // the mana, on a target in range -> cast. SEAM: as the Sniper's.
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Tuning;
    use crate::ai_load::uflags2;
    use crate::world::{Cell, Terrain, World};

    /// A computer leader's scout on open land, in no army.
    fn ai_scout() -> (Sim, usize) {
        let mut world = World::new(10, 10);
        world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(9, 9));
        let mut s = Sim::new(Tuning::RON, world, 2);
        s.nation[0].human = true;
        s.nation[1].human = false;
        let t = s.add_unit_type(crate::UnitType::default());
        s.unit_types[t].cols.unit_flags2 = uflags2::SCOUT | uflags2::CASTER;
        let mut u = crate::Unit::new(1, 0, crate::Pos::new(5 * 768 + 384, 5 * 768 + 384), 20);
        u.ty = Some(t);
        let u = s.add_unit(u);
        (s, u)
    }

    /// run346's frame 0 (`docs/AI.md` §80.5): at Easiest and Easy the arm
    /// returns before any draw, from Moderate up it throws the coin once,
    /// and with no army and a coin not divisible by three it returns 0.
    #[test]
    fn an_ai_scout_throws_the_special_coin_from_difficulty_two() {
        let (mut sim, scout) = ai_scout();
        for d in 0..=5 {
            sim.lobby.difficulty = d;
            let mut probe = sim.rng;
            let coin = probe.roll();
            let before = sim.rng;
            assert!(!sim.think_spellcaster(scout), "difficulty {d}");
            if d < 2 {
                assert_eq!(sim.rng, before, "difficulty {d}: no draw");
            } else {
                assert_eq!(sim.rng, probe, "difficulty {d}: one draw, {coin}");
            }
        }
    }

    /// A human's scout never reaches the computer's arm, and a decoy
    /// returns before the coin.
    #[test]
    fn a_human_s_scout_and_a_decoy_throw_nothing() {
        let (mut sim, scout) = ai_scout();
        sim.lobby.difficulty = 5;
        sim.units[scout].decoy = true;
        let before = sim.rng;
        assert!(!sim.think_spellcaster(scout));
        assert_eq!(sim.rng, before, "a decoy");
        sim.units[scout].decoy = false;
        sim.nation[1].human = true;
        assert!(!sim.think_spellcaster(scout));
        assert_eq!(sim.rng, before, "a human's");
    }
}
