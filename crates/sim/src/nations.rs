//! The nation-power layer: a player's nation, and the flags it turns on.
//!
//! Every nation power in the original is one call —
//! `LeaderData::has_tribe_bonus(n)`, where `n` is the nation's index in the
//! roster `rules.xml`'s `TRIBES` block lists. [`TechTree::has_tribe_bonus`]
//! is that call, and it has been here since the tech tree was; what was
//! missing is the wire between it and the number the game actually carries.
//! `LeaderData::tribe` is dumped by `LEADERS=9` on every frame, and until
//! now nothing read it, so every traced game was played with **no nation
//! power on either side** — twenty-four of them inert at once.
//!
//! This module is that wire, and nothing else. [`Sim::set_tribe`] takes the
//! dump's own number; [`Sim::refresh_nation_powers`] recomputes the
//! per-nation booleans on [`city::Nation`] that the mechanics read. Those
//! booleans are a **cache of `has_tribe_bonus`, not a second source**: each
//! is that call's answer for one roster index, so the lobby's "No Nation
//! Powers" flag and the no-city gate apply here exactly as they do there.
//!
//! `docs/TECH.md` ("`has_preq`: are the prerequisites met") is where the
//! roster indices are established; `docs/ECONOMY.md` ("The commerce cap") is
//! the measurement that made the wire worth having.
//!
//! [`TechTree::has_tribe_bonus`]: crate::tech::TechTree::has_tribe_bonus
//! [`city::Nation`]: crate::city::Nation

use crate::{Player, Sim};

/// The nation roster, in the order `rules.xml`'s `TRIBES` block lists it.
///
/// The index is the identity everywhere: it is the `n` of
/// `has_tribe_bonus(n)`, the number `LEADERDATA` prints as `tribe`, and the
/// bit position `TRIBE_MASK` sets (`crate::tech::TypeDef::tribes`). The
/// names are each nation file's own `<TRIBE name="…">`, which is what
/// `ScenarioFuncSet::find_nation` matches — so a name here is a claim about
/// the user's install, and `cargo run -p rondata -- <install>` re-derives
/// the whole list from `rules.xml` and `tribes/` and fails if it has
/// drifted.
pub const ROSTER: [&str; 24] = [
    "Aztecs",
    "Maya",
    "Inca",
    "Bantu",
    "Nubians",
    "Greeks",
    "Romans",
    "Egyptians",
    "Turks",
    "Spanish",
    "French",
    "British",
    "Germans",
    "Russians",
    "Chinese",
    "Japanese",
    "Koreans",
    "Mongols",
    "Iroquois",
    "Lakota",
    "Americans",
    "Indians",
    "Dutch",
    "Persians",
];

/// The roster index of a nation, by the name [`ROSTER`] holds.
pub fn power_of(name: &str) -> Option<usize> {
    ROSTER.iter().position(|n| *n == name)
}

impl Sim {
    /// Sets a player's nation from the dump's own `LeaderData::tribe`, and
    /// refreshes the flags that follow from it.
    ///
    /// The original's `tribe` is a signed index and **−1 is a real value** —
    /// a leader with no nation, which `has_tribe_bonus` answers no to for
    /// every power. That is what `power: None` means here.
    /// [`crate::tech::PlayerTech::tribe`] is a different thing: it indexes
    /// [`crate::tech::TechTree::tribes`] for the unit-graft and unique-unit
    /// tables, so it stays inside the roster and a −1 leaves it at zero,
    /// whose graft table is identity.
    ///
    /// An index past the end of a *loaded* roster still counts as a power —
    /// the tree built by a unit test has one placeholder tribe in it, and a
    /// power is not a table lookup.
    pub fn set_tribe(&mut self, who: Player, tribe: i64) {
        let w = who as usize;
        let power = usize::try_from(tribe).ok();
        self.tech[w].power = power;
        self.tech[w].tribe = power
            .filter(|&i| i < self.tech_tree.tribes.len())
            .unwrap_or(0);
        self.refresh_nation_powers(who);
    }

    /// Recomputes one player's per-nation flags from their power.
    ///
    /// Call it after anything `has_tribe_bonus` reads changes — the power
    /// itself, `has_city`, or the lobby's "No Nation Powers". The seven
    /// nations with no flag on [`crate::city::Nation`] — the Greeks, Spanish,
    /// Japanese, Mongols, Iroquois, Americans and Persians — are not missing:
    /// their powers are read straight off `has_tribe_bonus` at the rules that
    /// use them, and this cache exists only for the mechanics that took a
    /// boolean before the tree did.
    pub fn refresh_nation_powers(&mut self, who: Player) {
        let w = who as usize;
        let on: [bool; ROSTER.len()] = std::array::from_fn(|n| {
            self.tech_tree
                .has_tribe_bonus(&self.setup, &self.tech[w], n)
        });
        let n = &mut self.nation[w];
        n.aztecs = on[0];
        n.maya = on[1];
        n.inca = on[2];
        n.bantu = on[3];
        n.nubians = on[4];
        n.romans = on[6];
        n.egyptians = on[7];
        n.turks = on[8];
        n.french = on[10];
        n.british = on[11];
        n.germans = on[12];
        n.russians = on[13];
        n.chinese = on[14];
        n.koreans = on[16];
        n.lakota = on[19];
        n.indians = on[21];
        n.dutch = on[22];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::economy::{self, Resource};

    fn sim() -> Sim {
        Sim::new(crate::Tuning::RON, crate::World::new(16, 16), 2)
    }

    #[test]
    fn the_roster_index_is_the_power_the_rules_name() {
        // The indices `docs/TECH.md` pins at `has_preq`, each of which is a
        // `has_tribe_bonus(n)` this crate already hardcodes.
        assert_eq!(power_of("Nubians"), Some(4));
        assert_eq!(power_of("Greeks"), Some(5));
        assert_eq!(power_of("Romans"), Some(6));
        assert_eq!(power_of("Egyptians"), Some(7));
        assert_eq!(power_of("French"), Some(10));
        assert_eq!(power_of("British"), Some(11));
        assert_eq!(power_of("Germans"), Some(12));
        assert_eq!(power_of("Russians"), Some(13));
        assert_eq!(power_of("Koreans"), Some(16));
        assert_eq!(power_of("Iroquois"), Some(18));
        assert_eq!(power_of("Lakota"), Some(0x13));
        assert_eq!(power_of("Dutch"), Some(0x16));
        assert_eq!(power_of("Persians"), Some(23));
        assert_eq!(power_of("Prussians"), None);
    }

    #[test]
    fn the_dump_s_tribe_turns_one_flag_on_and_the_rest_off() {
        let mut s = sim();
        s.set_tribe(0, 11);
        assert_eq!(s.tech[0].power, Some(11));
        assert!(s.nation[0].british);
        assert!(!s.nation[0].french);
        assert!(!s.nation[0].nubians);

        // Nubians on the other side, from the same capture.
        s.set_tribe(1, 4);
        assert!(s.nation[1].nubians);
        assert!(!s.nation[1].british);
        // And the first player is untouched by the second's.
        assert!(s.nation[0].british);
    }

    #[test]
    fn a_tribe_of_minus_one_is_a_leader_with_no_nation() {
        let mut s = sim();
        s.set_tribe(0, 11);
        assert!(s.nation[0].british);
        s.set_tribe(0, -1);
        assert_eq!(s.tech[0].power, None);
        assert_eq!(s.tech[0].tribe, 0, "the graft table stays in range");
        assert!(!s.nation[0].british);
        assert!(!s.nation[0].aztecs, "and −1 is not roster index 0");
    }

    #[test]
    fn the_lobby_s_no_nation_powers_puts_every_flag_out() {
        let mut s = sim();
        s.setup.no_nation_powers = true;
        s.set_tribe(0, 11);
        assert_eq!(s.tech[0].power, Some(11), "the nation is still the nation");
        assert!(!s.nation[0].british, "but the power is off");

        // And a leader with no city takes none of it either.
        let mut t = sim();
        t.tech[0].has_city = false;
        t.set_tribe(0, 11);
        assert!(!t.nation[0].british);
    }

    /// The measurement the wire was worth: run40's AI is the British and
    /// carries a commerce cap of 1392 where the human carries 1120.
    #[test]
    fn run40_s_british_cap_follows_from_the_dump_s_tribe() {
        let mut s = sim();
        s.set_tribe(0, 4); // the human, Nubians
        s.set_tribe(1, 11); // the AI, British
        s.assemble_holdings(0);
        s.assemble_holdings(1);
        let t = &s.tuning;
        let cap = |w: usize| economy::commerce_cap(t, &s.holdings[w], Resource::Food);
        assert_eq!(cap(0), 1120, "70 x 16");
        assert_eq!(cap(1), 1392, "70 x 125 / 100 = 87, then x 16");
    }
}
