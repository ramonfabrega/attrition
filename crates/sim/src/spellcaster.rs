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
//! not is_special:  the hero arm   (Create Decoys; moving: seam)
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
//! - ~~**the hero arm**~~: built as [`Sim::think_hero`] (item 1305,
//!   `docs/AI.md` §99.13), reached from `Army::use_generals`;
//! - **the human arm**, which `Unit::think`'s special turn reaches: it
//!   draws nothing and casts only through the same refused targets.

use crate::Sim;

/// The special arm's coin, `Random::get` returning to `005f2bb3`.
pub const SITE_SPECIAL_COIN: &str = "Unit::think_spellcaster+0x413";

/// The hero arm's coin, `Random::get` returning to `005f2d29`: Forced
/// March on an odd one, the territory test on an even one.
pub const SITE_HERO_COIN: &str = "Unit::think_spellcaster+0x589";

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
            return self.think_hero(u);
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

impl Sim {
    /// **The hero arm** (`005f2cb3`–`005f2e7d`), a computer leader's
    /// non-special unit, as `Army::use_generals` reaches it on the army's
    /// 128-frame turn (`docs/AI.md` §99.13, item 1305):
    ///
    /// ```text
    /// not is_hero (unit_flags2 & 0x20)                -> 0
    /// is_moving (the head order's is_move):
    ///     guy 0's avg_speed (GuyData +0x84) == 0      -> 0
    ///     leader_flags & 0x8000 clear:
    ///         coin = Random::get(game_random, 0, 0xffff)   // +0x589
    ///         coin odd: FORCED_MARCH (0x27c) with the mana -> cast, else 0
    ///     the cell's owner (WData +0xf) negative or the caster's -> 0
    ///     AMBUSH (0x27b) with the mana                -> cast, else 0
    /// not moving:
    ///     any of the leader's units a live decoy captain -> 0
    ///     CREATE_DECOY (0x27a) with the mana          -> cast, else 0
    /// ```
    ///
    /// "With the mana" is `mana() >= mana_burn + the craft's MANA`; no
    /// arm asks `is_castable`. A cast is `add_cast_order(-1, -1, x, y,
    /// craft, QUEUE_FIRST, 0)` on the unit's own position.
    ///
    /// `leader_flags & 0x8000` is [`Sim::forced_march`]: a leader one of
    /// whose heroes is on the march throws no coin and goes straight to
    /// the territory test.
    ///
    /// SEAM: what Ambush does when cast (`cast_ambush`) and what Forced
    /// March does beyond its flags (`crate::cast`).
    fn think_hero(&mut self, u: usize) -> bool {
        use crate::orders::{QueuePos, spell};
        if !self.is_hero_unit(u) {
            return false;
        }
        let who = self.units[u].owner;
        let craft = if self.is_moving(u) {
            let unit = &self.units[u];
            let avg = unit
                .guys
                .first()
                .and_then(|g| g.follow)
                .map_or(unit.movement.body.avg_speed, |f| f.body.avg_speed);
            if avg == 0 {
                return false;
            }
            let march = !self.forced_march[who as usize] && {
                self.mark(SITE_HERO_COIN);
                self.rng.roll() & 1 != 0
            };
            if march {
                spell::FORCED_MARCH
            } else {
                match self.world.owner_at(self.units[u].pos).player() {
                    Some(p) if p != who => spell::AMBUSH,
                    _ => return false,
                }
            }
        } else {
            let decoy_captain = self
                .units
                .iter()
                .enumerate()
                .any(|(i, x)| x.owner == who && x.alive() && self.is_captain(i) && x.decoy);
            if decoy_captain {
                return false;
            }
            spell::CREATE_DECOY
        };
        let cost = self.spell(craft).map_or(0, |d| d.mana);
        if self.unit_mana(u) < i32::from(self.units[u].mana_burn) + cost {
            return false;
        }
        let at = self.units[u].pos;
        self.add_cast_order_on(u, craft, None, at, QueuePos::First, false);
        true
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

    /// A computer leader's General on the move at (5, 5), its type
    /// `MANA 1000`, and the three crafts of the hero arm priced 500, 500
    /// and 1,000. Returns the sim, the General and a seed per coin parity.
    fn ai_general() -> (Sim, usize, [u32; 2]) {
        use crate::orders::{MoveKind, QueuePos, SpellType, spell};
        let (mut s, _) = ai_scout();
        s.lobby.difficulty = 5;
        let t = s.add_unit_type(crate::UnitType::default());
        s.unit_types[t].cols.unit_flags2 = uflags2::GENERAL;
        s.unit_types[t].mana = 1000;
        // `TypeIndex` `0x36`, the General: the crafts' `FROM`.
        s.unit_types[t].tree = Some(0x36);
        let mut u = crate::Unit::new(1, 1, crate::Pos::new(5 * 768 + 384, 5 * 768 + 384), 20);
        u.ty = Some(t);
        u.captain = true;
        let g = s.add_unit(u);
        s.add_move_order(
            g,
            crate::Pos::new(8 * 768, 8 * 768),
            MoveKind::MoveTo,
            QueuePos::New,
            false,
        );
        s.units[g].movement.body.avg_speed = 5;
        let mut rows = vec![SpellType::default(); 55];
        let row = |x: i32| (x - spell::FIRST) as usize;
        for (craft, mana) in [
            (spell::FORCED_MARCH, 500),
            (spell::AMBUSH, 500),
            (spell::CREATE_DECOY, 1000),
        ] {
            rows[row(craft)] = SpellType {
                mana,
                job_time: 10,
                from: [Some(0x36), None],
                ..Default::default()
            };
        }
        s.spells = rows;
        let seed = |odd: i32| {
            (1..)
                .find(|&k| crate::combat::Rng::new(k).roll() & 1 == odd)
                .unwrap()
        };
        (s, g, [seed(0), seed(1)])
    }

    /// run470's frame 8856 (`docs/AI.md` §99.13): a moving hero throws
    /// the coin at `+0x589`; an odd one lays Forced March first, an even
    /// one reads the cell's owner — its own leader's, or no one's, and
    /// nothing is cast; an enemy's, and Ambush is.
    #[test]
    fn a_moving_general_throws_the_hero_coin() {
        use crate::orders::{Body, spell};
        use crate::world::Owner;
        let cast = |sim: &Sim, g: usize| match sim.units[g].orders.front().map(|o| o.body) {
            Some(Body::Cast(c)) => Some(c.spell),
            _ => None,
        };
        let (mut sim, g, [even, odd]) = ai_general();
        sim.rng = crate::combat::Rng::new(odd);
        let mut probe = sim.rng;
        probe.roll();
        assert!(sim.think_spellcaster(g));
        assert_eq!(sim.rng, probe, "one draw");
        assert_eq!(cast(&sim, g), Some(spell::FORCED_MARCH));

        let (mut sim, g, _) = ai_general();
        sim.rng = crate::combat::Rng::new(even);
        assert!(!sim.think_spellcaster(g), "no one's cell");
        assert_ne!(
            sim.rng,
            crate::combat::Rng::new(even),
            "the coin was thrown"
        );
        sim.world
            .set_owner(Cell::new(5, 5), Owner::Player(1), Owner::None);
        sim.rng = crate::combat::Rng::new(even);
        assert!(!sim.think_spellcaster(g), "its own leader's cell");
        sim.world
            .set_owner(Cell::new(5, 5), Owner::Player(0), Owner::None);
        sim.rng = crate::combat::Rng::new(even);
        assert!(sim.think_spellcaster(g), "an enemy's cell");
        assert_eq!(cast(&sim, g), Some(spell::AMBUSH));

        // Short of the craft's mana, nothing is cast after the coin.
        let (mut sim, g, _) = ai_general();
        sim.units[g].mana_burn = 501;
        sim.rng = crate::combat::Rng::new(odd);
        assert!(!sim.think_spellcaster(g));
        assert_ne!(sim.rng, crate::combat::Rng::new(odd));
    }

    /// run470's 9112 and 9240: Forced March, cast, raises the leader's
    /// `0x8000` and the hero's until `DURATION` runs out — no coin on the
    /// turn between — and a hero's craft of its own draws no animation
    /// roll (`set_anim(CHAR_ATTACK2, 0, 0)`).
    #[test]
    fn a_forced_march_holds_the_coin_until_it_runs_out() {
        use crate::orders::{Body, spell};
        let (mut sim, g, [_, odd]) = ai_general();
        let row = (spell::FORCED_MARCH - spell::FIRST) as usize;
        sim.spells[row].job_time = 0;
        sim.spells[row].duration = 150;
        sim.rng = crate::combat::Rng::new(odd);
        assert!(sim.think_spellcaster(g));
        let Some(Body::Cast(c)) = sim.units[g].orders.front().map(|o| o.body) else {
            panic!("a cast order");
        };
        let before = sim.rng;
        sim.do_cast(g, c);
        assert_eq!(sim.rng, before, "the attack slot rolls nothing");
        assert!(sim.forced_march[1]);
        assert_eq!(sim.units[g].marching, Some(sim.frame + 150));
        // The next turn: moving again, no coin while the march stands.
        let to = crate::Pos::new(8 * 768, 8 * 768);
        sim.add_move_order(
            g,
            to,
            crate::orders::MoveKind::MoveTo,
            crate::orders::QueuePos::New,
            false,
        );
        let before = sim.rng;
        assert!(!sim.think_spellcaster(g));
        assert_eq!(sim.rng, before, "no coin while marching");
        // Past the end frame the spell runs out and the flag with it.
        let end = sim.units[g].marching.unwrap();
        sim.process_spells(g, end);
        assert!(sim.forced_march[1], "held through its end frame");
        sim.process_spells(g, end + 1);
        assert!(!sim.forced_march[1] && sim.units[g].marching.is_none());
    }

    /// Before the coin: a guy standing (`avg_speed` 0) returns 0 with no
    /// draw, and a hero not moving takes the decoy arm — no draw, Create
    /// Decoys with the mana, and nothing while one of its leader's decoy
    /// captains stands.
    #[test]
    fn a_general_standing_draws_nothing_and_a_still_one_makes_decoys() {
        use crate::orders::{Body, spell};
        let (mut sim, g, _) = ai_general();
        sim.units[g].movement.body.avg_speed = 0;
        let before = sim.rng;
        assert!(!sim.think_spellcaster(g));
        assert_eq!(sim.rng, before, "avg_speed 0: no draw");

        let (mut sim, g, _) = ai_general();
        sim.kill_current_order(g);
        let before = sim.rng;
        assert!(sim.think_spellcaster(g));
        assert_eq!(sim.rng, before, "the decoy arm draws nothing");
        assert!(matches!(
            sim.units[g].orders.front().map(|o| o.body),
            Some(Body::Cast(c)) if c.spell == spell::CREATE_DECOY
        ));

        let (mut sim, g, _) = ai_general();
        sim.kill_current_order(g);
        let mut d = crate::Unit::new(1, 2, crate::Pos::new(2 * 768, 2 * 768), 20);
        d.captain = true;
        d.decoy = true;
        sim.add_unit(d);
        assert!(!sim.think_spellcaster(g), "a decoy captain stands");
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
