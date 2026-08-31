//! The goody box — `Unit::explore_goody@005f9780`, `docs/GOODY.md`.
//!
//! A map generator scatters a handful of ruins across the world and marks
//! each with one bit on the cell: `WData.flags & 0x8000`, [`GOODY`]. The
//! first land unit that is not gaia to step into such a cell takes it, and
//! what it takes is **a pile of one good, chosen by a lottery over the
//! goods the player can currently gather**. Every candidate good costs one
//! sync draw, which is why a mechanic worth a page of prose is worth a
//! module: on East Indies it is three draws on frame 867 and the whole
//! stream after it.
//!
//! The pick is not "the good you have least of" and not a uniform choice
//! either. Each candidate scores `draw % 25 + bucket[good]` and the lowest
//! score wins — a strict *less-than*, so ties go to the earlier good — so
//! the jitter only decides between goods within 24 of each other and the
//! poorest good wins outright whenever it is more than 24 behind.
//!
//! # What it is not
//!
//! `Unit::find_goody_box@005f2540` (`docs/SCOUT.md` §12) is a scout's
//! *search* for one, and `Unit::get_goody_box@005f7690` — reached from
//! `World::reveal_fog` for a unit carrying `unit_masks & 0x100` — is the
//! auto-walk that orders a unit onto one it has just seen. Neither is
//! here; this module is only what happens on arrival.

use crate::economy::RESOURCES;
use crate::tech::Line;
use crate::world::cell::GOODY;
use crate::{Player, Sim};

/// The lottery's draw site, under the original's own offset. One draw per
/// candidate good, in good order.
pub const SITE_PICK: &str = "Unit::explore_goody+0x27c";

/// `KNOWLEDGE`, the one good the lottery refuses outright — no draw, no
/// candidacy — however available it is.
const KNOWLEDGE: usize = 3;

/// `WEALTH`, the fallback when no good is a candidate. The original starts
/// the search with `best = -1` and finishes with `cmovns`, so a player who
/// can gather nothing at all still gets a pile of gold.
const FALLBACK: usize = 2;

/// The jitter's width: `Random::get(game_random, 0, 0xffff) % 0x19`.
const JITTER: i32 = 25;

impl Sim {
    /// `Unit::explore_goody@005f9780`: the unit `u` has just entered a cell
    /// carrying [`GOODY`]. Clear the bit and pay the finder.
    ///
    /// The caller is [`Sim::set_new_location`], which applies the four
    /// guards (§2); this function assumes them.
    pub(crate) fn explore_goody(&mut self, u: usize) {
        let cell = self.units[u].pos.cell();
        // The bit goes first and unconditionally — before the `frame != 0`
        // gate, so a goody a unit is *placed* on at setup is consumed
        // without paying (§4).
        let mut d = self.world.cell_data(cell);
        d.flags &= !GOODY;
        self.world.set_cell_data(cell, d);
        if self.frame == 0 {
            return;
        }
        let who = self.units[u].owner;
        let good = self.pick_goody_good(who);
        let amount = self.goody_amount(who);
        self.ledgers[who as usize].bucket[good] += amount;
        self.ledgers[who as usize].goody_box_resources += amount;
    }

    /// The lottery — one draw a candidate, the lowest score winning.
    ///
    /// A candidate is a good whose `LeaderData::type_avail(good, 1)` is
    /// `AVAILABLE` and which is not [`KNOWLEDGE`]. The original tests
    /// availability *first*, so an unavailable knowledge costs nothing
    /// either way and the draw count is the number of available non-
    /// knowledge goods.
    fn pick_goody_good(&mut self, who: Player) -> usize {
        let mut best: i64 = 99_999_999;
        let mut pick: Option<usize> = None;
        for g in 0..RESOURCES {
            if !self.type_available(who, g) || g == KNOWLEDGE {
                continue;
            }
            let held = self.ledgers[who as usize].bucket[g];
            self.mark(SITE_PICK);
            let score = i64::from(self.rng.roll() % JITTER) + i64::from(held);
            if score < best {
                best = score;
                pick = Some(g);
            }
        }
        pick.unwrap_or(FALLBACK)
    }

    /// The pile: `epoch * GOODY_BOX_AGE + GOODY_BOX`, and the Spanish pair
    /// in place of both when the finder has `has_tribe_bonus(9)`.
    ///
    /// **The `epoch` is not the age.** The original reads
    /// `LeaderDataEncrypt +0xf4`, which is `epoch[3]` — the Science line's
    /// level, `get_epoch_base(3) == BASE_EPOCHTYPES` — and not `ages` at
    /// `+0xdc`, whatever `GOODY_BOX_AGE`'s name suggests. `docs/GOODY.md`
    /// §3 has the listing that settles it.
    fn goody_amount(&self, who: Player) -> i32 {
        let level = self.tech[who as usize].epoch[Line::Science.index()];
        let t = &self.tuning;
        if self
            .tech_tree
            .has_tribe_bonus(&self.setup, &self.tech[who as usize], 9)
        {
            level * t.spanish_ruins + t.spanish_ruins_base
        } else {
            level * t.goody_box_age + t.goody_box
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::economy::Resource;
    use crate::tech::{Preq, TechTree, TypeDef};
    use crate::world::{Cell, Pos, Terrain, World};
    use crate::{Tuning, Unit};

    /// A flat two-player world whose tree is the six goods and nothing else,
    /// one land unit of player 1, and a goody on the cell east of it.
    ///
    /// The good ids are the engine's own `TypeIndex` 0…5 — `FOOD`, `TIMBER`,
    /// `WEALTH`, `KNOWLEDGE`, `METAL`, `OIL` — which is what lets the
    /// lottery index the tree with a [`Resource`].
    fn sim() -> (crate::Sim, usize) {
        let mut world = World::new(8, 8);
        world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(7, 7));
        let mut s = crate::Sim::new(Tuning::RON, world, 2);
        let mut t = TechTree::new();
        for n in ["Food", "Timber", "Wealth", "Knowledge", "Metal", "Oil"] {
            t.add(TypeDef::good(n));
        }
        s.tech_tree = t;
        s.frame = 1;
        // Empty buckets, so a pile is legible as itself rather than as a
        // delta on `STARTING_GOODS`.
        for l in &mut s.ledgers {
            l.bucket = [0; RESOURCES];
        }
        let at = Pos::new(2 * 0x300 + 0x180, 2 * 0x300 + 0x180);
        let u = s.add_unit(Unit::new(1, 0, at, 20));
        let mut d = s.world.cell_data(Cell::new(3, 2));
        d.flags |= GOODY;
        s.world.set_cell_data(Cell::new(3, 2), d);
        (s, u)
    }

    /// Walking east one cell is one cell change, and the cell change is the
    /// trigger — `Unit::set_new_location`'s `local_10`, not the half-cell
    /// the fog reveal hangs off.
    fn step_onto_the_goody(s: &mut crate::Sim, u: usize) {
        let to = Pos::new(3 * 0x300 + 0x180, 2 * 0x300 + 0x180);
        s.set_new_location(u, to, false);
    }

    /// Five candidates, five draws, and the bit gone: the base case with
    /// every good available. Knowledge is the one the loop refuses whatever
    /// `type_avail` says, so six goods buy five draws.
    #[test]
    fn knowledge_is_never_a_candidate_and_never_costs_a_draw() {
        let (mut s, u) = sim();
        s.trace_phases = true;
        let before = s.rng.seed;
        step_onto_the_goody(&mut s, u);
        assert_eq!(
            s.phase_marks.iter().filter(|(l, _)| l == SITE_PICK).count(),
            5,
            "six goods, knowledge refused"
        );
        assert_ne!(s.rng.seed, before);
        assert_eq!(
            s.world.cell_data(Cell::new(3, 2)).flags & GOODY,
            0,
            "the bit is cleared, so no second unit can take the same ruins"
        );
        assert_eq!(
            s.ledgers[1].bucket[Resource::Knowledge.index()],
            0,
            "and knowledge is never what a box pays"
        );
    }

    /// The pick is `draw % 25 + bucket[good]`, lowest score winning — so a
    /// good more than 24 behind wins outright however the coins fall.
    #[test]
    fn the_poorest_good_by_more_than_the_jitter_always_wins() {
        for seed in [1u32, 0x1234_5678, 0xdead_beef] {
            let (mut s, u) = sim();
            s.rng.seed = seed;
            s.ledgers[1].bucket = [500, 500, 500, 500, 500, 0];
            step_onto_the_goody(&mut s, u);
            assert_eq!(
                s.ledgers[1].bucket[Resource::Oil.index()],
                25,
                "seed {seed:#x}: oil is 500 behind and the jitter is 24 wide"
            );
            assert_eq!(s.ledgers[1].goody_box_resources, 25);
        }
    }

    /// The pile's per-level half is the **Science** library level, not the
    /// age: `LeaderDataEncrypt +0xf4` is `epoch[3]`, whose
    /// `get_epoch_base(3)` is `BASE_EPOCHTYPES`, and `ages` is `+0xdc`.
    #[test]
    fn the_pile_is_the_science_level_and_not_the_age() {
        let (mut s, u) = sim();
        s.tech[1].epoch[Line::Science.index()] = 3;
        s.tech[1].ages = 6;
        step_onto_the_goody(&mut s, u);
        assert_eq!(
            s.ledgers[1].goody_box_resources,
            3 * 25 + 25,
            "the age would pay 175 and the Science level pays 100"
        );
    }

    /// `has_tribe_bonus(9)` swaps both halves for the Spanish pair — 26 and
    /// 30 where the stock game pays 25 and 25.
    #[test]
    fn the_spanish_ruins_replace_both_halves() {
        let (mut s, u) = sim();
        s.tech[1].has_city = true;
        s.set_tribe(1, 9);
        s.tech[1].epoch[Line::Science.index()] = 2;
        step_onto_the_goody(&mut s, u);
        assert_eq!(s.ledgers[1].goody_box_resources, 2 * 26 + 30);
    }

    /// No candidate at all — every good disabled — still pays, and pays
    /// wealth: the original starts at `best = -1` and finishes with a
    /// `cmovns` against the literal 2.
    #[test]
    fn a_player_who_can_gather_nothing_is_paid_in_wealth() {
        let (mut s, u) = sim();
        s.trace_phases = true;
        for g in 0..RESOURCES {
            s.tech_tree.types[g].preq[0] = Preq::Disabled;
        }
        step_onto_the_goody(&mut s, u);
        assert_eq!(
            s.phase_marks.iter().filter(|(l, _)| l == SITE_PICK).count(),
            0,
            "no candidate, no draw"
        );
        assert_eq!(s.ledgers[1].bucket[Resource::Wealth.index()], 25);
    }

    /// Frame 0 takes the bit and pays nothing — the whole reward half sits
    /// behind `game->frame != 0`, so a unit placed on a goody at setup
    /// consumes it silently.
    #[test]
    fn frame_zero_consumes_the_box_without_paying() {
        let (mut s, u) = sim();
        s.frame = 0;
        s.trace_phases = true;
        step_onto_the_goody(&mut s, u);
        assert_eq!(s.world.cell_data(Cell::new(3, 2)).flags & GOODY, 0);
        assert_eq!(s.ledgers[1].bucket, [0; RESOURCES]);
        assert!(s.phase_marks.iter().all(|(l, _)| l != SITE_PICK));
    }

    /// Gaia takes nothing. The guard is `SubObjectData::is_animal`, vftable
    /// slot `+0x30` — the same one `resolve_unit_collision` reads
    /// (`docs/COLLISION.md` §6 step 0) — so a herd wandering over ruins
    /// leaves them where they are. An animal has no ledger to pay into
    /// either, which is the other half of why the guard is not optional.
    #[test]
    fn an_animal_walks_over_a_goody_without_taking_it() {
        let (mut s, _) = sim();
        let at = Pos::new(2 * 0x300 + 0x180, 2 * 0x300 + 0x180);
        let a = s.add_unit(Unit::new(crate::world::PLAYER_SLOTS, 1, at, 20));
        assert!(s.units[a].is_gaia());
        step_onto_the_goody(&mut s, a);
        assert_ne!(s.world.cell_data(Cell::new(3, 2)).flags & GOODY, 0);
        assert_eq!(s.ledgers[1].bucket, [0; RESOURCES]);
    }

    /// A ship crossing a goody cell takes nothing either — `type->domain ==
    /// 0` is one of the four guards, and a coastal ruin is a land unit's.
    #[test]
    fn a_sea_type_leaves_the_goody_where_it_is() {
        let (mut s, u) = sim();
        let t = s.add_unit_type(crate::UnitType {
            hits: 20,
            moves: 40,
            ..crate::UnitType::default()
        });
        s.unit_types[t].combat.domain = crate::attrition::Domain::Sea;
        s.units[u].ty = Some(t);
        step_onto_the_goody(&mut s, u);
        assert_ne!(s.world.cell_data(Cell::new(3, 2)).flags & GOODY, 0);
        assert_eq!(s.ledgers[1].bucket, [0; RESOURCES]);
    }
}
