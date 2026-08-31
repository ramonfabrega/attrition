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
//! # The other half: the search
//!
//! `Unit::find_goody_box@005f2540` is the *approach* — a land unit's
//! 49-cell sweep for a box it has seen and not yet taken, and the
//! `EXPLORE_TO` order `Unit::get_goody_box@005f7690` gives it. It runs from
//! two places: the head of `Unit::think_scout` and, every fifteenth frame,
//! the tail of `Unit::do_explore_to`. Neither spends a draw. `docs/GOODY.md`
//! §7 is the whole of it.
//!
//! `World::reveal_fog@006b3d30` reaches `get_goody_box` directly for a unit
//! carrying `unit_masks & 0x100`; nothing here sets that bit, so that third
//! caller is still a seam.

use crate::economy::RESOURCES;
use crate::orders::{MoveKind, QueuePos};
use crate::tech::Line;
use crate::world::cell::GOODY;
use crate::world::{Cell, MOVE_49, Pos, UNITS_PER_CELL};
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

    /// `Unit::find_goody_box@005f2540` (`docs/GOODY.md` §7): the 49-cell
    /// sweep for a box worth walking to. Returns whether an order was
    /// issued; spends no draw either way.
    ///
    /// A candidate cell must be **in the sweeper's own region**, carry
    /// [`GOODY`], and have been seen — `WorldData::was_seen` at any one of
    /// the cell's four half-cells, so a cell glimpsed at one corner
    /// counts. The first candidate in `move_x`/`move_y` order wins, and a
    /// candidate the unit is *already* walking to (the cell of
    /// `orders_x`/`orders_y`) ends the sweep with no order rather than
    /// re-issuing the one it has.
    pub(crate) fn find_goody_box(&mut self, u: usize) -> bool {
        let Some(t) = self.units[u].ty else {
            return false;
        };
        if self.unit_types[t].kind.domain != crate::attrition::Domain::Land {
            return false;
        }
        let who = self.units[u].owner;
        let here = self.units[u].pos.cell();
        // The original reads `WData.region` of its own cell whatever it is
        // — including the `-1` of a cell in no region — and compares the
        // raw short, so an off-region unit looks for off-region boxes.
        let region = self.world.region_of(here);
        for (dx, dy) in MOVE_49 {
            let c = Cell::new(here.x + dx, here.y + dy);
            if !self.world.contains(c) {
                continue;
            }
            if self.world.region_of(c) != region {
                continue;
            }
            let d = self.world.cell_data(c);
            if d.flags & GOODY == 0 {
                continue;
            }
            if !self.goody_was_seen(c, who) {
                continue;
            }
            // …and then the **item** on the cell: `find_goody_at`'s chain
            // walk, and `ItemData::is_seen` on what it finds. That is a
            // second, *stricter* fog read — `ever_seen`, the bare
            // accumulation of line of sight, with none of `was_seen`'s
            // ally-territory shortcut — and on East Indies it is the gate
            // that matters, because the box sits inside its finder's own
            // borders and the shortcut above answers yes from frame 0.
            // `docs/GOODY.md` §7.2.
            if !self.goody_item_is_seen(c, who) {
                continue;
            }
            if c == self.units[u].orders_pos.cell() {
                return false;
            }
            self.get_goody_box(u, c);
            return true;
        }
        false
    }

    /// The sweep's fog read: `WorldData::was_seen` at all four half-cells
    /// of `c`, in the original's own order — `(2x+1, 2y+1)`, `(2x, 2y+1)`,
    /// `(2x+1, 2y)`, `(2x, 2y)` — and true if **any** of them answers yes.
    /// The listing tests them in sequence and only falls through to the
    /// next cell when all four are clear.
    fn goody_was_seen(&self, c: Cell, who: Player) -> bool {
        let (x, y) = (2 * c.x, 2 * c.y);
        self.was_seen_fog(x + 1, y + 1, who)
            || self.was_seen_fog(x, y + 1, who)
            || self.was_seen_fog(x + 1, y, who)
            || self.was_seen_fog(x, y, who)
    }

    /// `ItemData::is_seen@00677850` (vtable `+0x48`) on the box's own
    /// item, as far as a simulation with no item chain can answer it.
    ///
    /// The original's first arm is `ever_seen & ally_mask` — a per-item
    /// byte that `check_ever_seen` ORs the **current** line-of-sight grid
    /// into every frame the object is processed, which makes it the same
    /// monotone accumulation `World::seen2` already is. So this reads
    /// `was_really_seen` — the fog with none of `was_seen`'s
    /// ally-territory shortcut — over the cell's four half-cells, because
    /// the item's own point inside the cell is not something this crate
    /// carries.
    ///
    /// SEAM: the two arms below it — the Spanish `has_tribe_bonus(9)`
    /// exemption and the fall-through to `WorldData::is_seen`, the
    /// *current* grid rather than the accumulated one — and the whole
    /// chain walk that finds the item in the first place
    /// (`docs/QUEUE.md` item 48). A box on a cell nothing has ever seen
    /// answers no here either way.
    fn goody_item_is_seen(&self, c: Cell, who: Player) -> bool {
        let (x, y) = (2 * c.x, 2 * c.y);
        self.was_really_seen_fog(x + 1, y + 1, who)
            || self.was_really_seen_fog(x, y + 1, who)
            || self.was_really_seen_fog(x + 1, y, who)
            || self.was_really_seen_fog(x, y, who)
    }

    /// `Unit::get_goody_box@005f7690`: a one-member group, and an
    /// `EXPLORE_TO` to the box's **cell centre** at `QUEUE_FIRST`.
    ///
    /// The queue position is the group's, not the unit's, and the two are
    /// different things (`docs/GROUPS.md` §17): the group halts its
    /// members and re-issues the move as `QUEUE_NEW`, so the walk the unit
    /// was on is dropped rather than stacked behind.
    fn get_goody_box(&mut self, u: usize, c: Cell) {
        let who = self.units[u].owner;
        let mut g = crate::group::Group::stack(who);
        self.group_add(&mut g, u);
        if !self.push_group(&g, true) {
            return;
        }
        let half = UNITS_PER_CELL / 2;
        let to = Pos::new(c.x * UNITS_PER_CELL + half, c.y * UNITS_PER_CELL + half);
        self.group_action_move_to(
            &g,
            to,
            QueuePos::First,
            false,
            crate::movement::Angle(0),
            MoveKind::ExploreTo,
            false,
        );
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

    // ------------------------------------------------------------------
    // §7 — the search
    // ------------------------------------------------------------------

    /// [`sim`]'s world with a **typed** land unit, so the sweep's own
    /// `domain == 0` head lets it in, and no fog grid — which answers every
    /// `was_seen` yes, the flat world this harness has always had.
    fn sweeper() -> (crate::Sim, usize) {
        let (mut s, u) = sim();
        let t = s.add_unit_type(crate::UnitType {
            hits: 20,
            moves: 40,
            ..crate::UnitType::default()
        });
        s.units[u].ty = Some(t);
        (s, u)
    }

    /// The box the unit is aimed at, if it has been given one — the cell
    /// centre `get_goody_box` orders, read back off the order.
    fn aimed_at(s: &crate::Sim, u: usize) -> Option<Cell> {
        match s.units[u].orders.front().map(|o| o.body) {
            Some(crate::orders::Body::Move(m)) => Some(m.dest.cell()),
            _ => None,
        }
    }

    /// The base case: a box two cells away, in range of the 49-cell sweep,
    /// and the walk the sweep issues aims at its **cell centre** —
    /// `c × 0x300 + 0x180`, which is not the `4c + 2` tile centre
    /// `think_scout` aims at.
    #[test]
    fn the_sweep_orders_a_walk_to_the_box_s_cell_centre() {
        let (mut s, u) = sweeper();
        let before = s.rng.seed;
        assert!(s.find_goody_box(u), "the box at (3, 2) is one cell east");
        assert_eq!(aimed_at(&s, u), Some(Cell::new(3, 2)));
        let m = match s.units[u].orders.front().map(|o| o.body) {
            Some(crate::orders::Body::Move(m)) => m,
            other => panic!("an EXPLORE_TO, not {other:?}"),
        };
        assert_eq!(m.kind, crate::orders::MoveKind::ExploreTo);
        // The cell centre, then `add_move_facing_order`'s 48-unit snap —
        // `p / 0x30 * 0x30 + 0x18`, which lands a cell centre 24 units on.
        // run39's `FRAME 826` shows the same pair: `orders_x 34968` for a
        // box whose cell centre is 34944.
        assert_eq!(
            m.dest,
            Pos::new(3 * 0x300 + 0x180 + 0x18, 2 * 0x300 + 0x180 + 0x18)
        );
        assert_eq!(s.rng.seed, before, "the whole sweep is free");
    }

    /// The sweep walks `move_x`/`move_y`, so of two boxes the winner is the
    /// one **earlier in that table**, not the nearer one by any metric of
    /// its own: `(2, 1)` is the compass entry 1 and `(3, 2)` is entry 4.
    #[test]
    fn the_first_box_in_the_table_s_order_wins() {
        let (mut s, u) = sweeper();
        let mut d = s.world.cell_data(Cell::new(1, 1));
        d.flags |= GOODY;
        s.world.set_cell_data(Cell::new(1, 1), d);
        assert!(s.find_goody_box(u));
        assert_eq!(
            aimed_at(&s, u),
            Some(Cell::new(1, 1)),
            "`move_49[1]` is `(-1, -1)` and `(1, 0)` is entry 4"
        );
    }

    /// A box the sweeper has never seen is not a candidate. With a fog grid
    /// installed nothing is seen, and lighting **one** of the box cell's
    /// four half-cells for the sweeper is enough — the listing falls
    /// through to the next cell only when all four are clear.
    #[test]
    fn a_box_no_line_of_sight_has_reached_is_not_a_candidate() {
        let (mut s, u) = sweeper();
        assert!(s.world.set_fog(vec![0; 16 * 16]));
        assert!(!s.find_goody_box(u), "the whole map is dark");
        assert!(s.units[u].orders.is_empty());
        // `(2 × 3, 2 × 2)`, the box cell's first half-cell each way, lit
        // for player 1 alone.
        s.world.set_seen(6, 4, 1 << 1);
        assert!(s.find_goody_box(u), "one half-cell is enough");
        assert_eq!(aimed_at(&s, u), Some(Cell::new(3, 2)));
    }

    /// A box in another region is not a candidate however close it is: the
    /// sweep compares each cell's `WData.region` against the sweeper's own.
    #[test]
    fn a_box_across_a_region_boundary_is_not_a_candidate() {
        let (mut s, u) = sweeper();
        let other = s
            .world
            .fill_region(Terrain::Land, Cell::new(3, 0), Cell::new(3, 7));
        assert_eq!(s.world.region_of(Cell::new(3, 2)), Some(other));
        assert!(!s.find_goody_box(u), "the box is on the far side of a seam");
        assert!(s.units[u].orders.is_empty());
    }

    /// The box the unit is **already** walking to ends the sweep with no
    /// order at all — the original's `return 0` at `005f2761`, which is
    /// what keeps the fifteen-frame look from re-issuing the same walk over
    /// and over.
    #[test]
    fn the_box_already_aimed_at_ends_the_sweep_with_nothing() {
        let (mut s, u) = sweeper();
        assert!(s.find_goody_box(u));
        let orders = s.units[u].orders.clone();
        let before = s.rng.seed;
        assert!(!s.find_goody_box(u), "it is already going there");
        assert_eq!(s.units[u].orders, orders, "and the walk is untouched");
        assert_eq!(s.rng.seed, before);
    }

    /// The walk **replaces** what the unit was doing. `get_goody_box` asks
    /// for `QUEUE_FIRST`, but it asks a *group*, and a group's
    /// `QUEUE_FIRST` halts its members and re-issues as `QUEUE_NEW`
    /// (`docs/GROUPS.md` §17) — so a plain transit move is dropped rather
    /// than stacked behind, and run39's `FRAME 826` holds one order where a
    /// unit-level `QUEUE_FIRST` would leave two.
    #[test]
    fn the_walk_replaces_the_transit_it_interrupts() {
        let (mut s, u) = sweeper();
        s.add_move_order(
            u,
            Pos::new(6 * 0x300 + 0x180, 6 * 0x300 + 0x180),
            crate::orders::MoveKind::ExploreTo,
            crate::orders::QueuePos::New,
            false,
        );
        assert_eq!(s.units[u].orders.len(), 1);
        assert!(s.find_goody_box(u));
        assert_eq!(s.units[u].orders.len(), 1, "the old walk is gone");
        assert_eq!(aimed_at(&s, u), Some(Cell::new(3, 2)));
    }

    /// The look is one frame in fifteen, phased by `o`, and it runs from
    /// `do_explore_to` — so a unit walking an `EXPLORE_TO` re-aims on a
    /// multiple of fifteen and on no other frame.
    #[test]
    fn the_look_runs_one_frame_in_fifteen() {
        let (mut s, u) = sweeper();
        s.add_move_order(
            u,
            Pos::new(6 * 0x300 + 0x180, 6 * 0x300 + 0x180),
            crate::orders::MoveKind::ExploreTo,
            crate::orders::QueuePos::New,
            false,
        );
        // `o` is 0 here, so 14 is not a look and 15 is.
        for f in 1..15 {
            s.frame = f;
            s.work(u, f);
            assert_ne!(
                aimed_at(&s, u),
                Some(Cell::new(3, 2)),
                "frame {f} is not a fifteenth"
            );
        }
        s.frame = 15;
        s.work(u, 15);
        assert_eq!(aimed_at(&s, u), Some(Cell::new(3, 2)), "frame 15 is");
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
