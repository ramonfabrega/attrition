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
    s.tables.tech_tree = t;
    s.frame = 1;
    // Empty buckets, so a pile is legible as itself rather than as a
    // delta on `STARTING_GOODS`.
    for l in &mut s.economy.ledgers {
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

/// **The item's gate falls through to the current sight, and its
/// territory arm sees a leader's own ground** (item 1558, `docs/AI.md`
/// §153). Electronics answers the cell's `was_seen` for every cell, and
/// a box no unit has looked at is still refused by the item's own gate;
/// on the leader's ground with Computerization held (`leader_flags &
/// 0x2000`), `WorldData::is_seen` answers yes and the sweep takes it.
/// An enemy's ground is not lit, and nor is the leader's own without the
/// bonus.
#[test]
fn a_box_on_its_own_ground_is_seen_once_the_leader_holds_computerization() {
    use crate::tech::Kind;
    use crate::world::Owner;
    let (mut s, u) = sweeper();
    assert!(s.world.set_fog(vec![0; 16 * 16]));
    let mut tree = s.tables.tech_tree.clone();
    let electronics = tree.types.len();
    tree.types.push(TypeDef::new("Electronics", Kind::Final));
    let computerization = tree.types.len();
    tree.types
        .push(TypeDef::new("Computerization", Kind::Final));
    tree.roles.explore_map_preq = Some(electronics);
    tree.roles.territory_reveal_preq = Some(computerization);
    s.set_tech_tree(tree);
    for p in &mut s.tech {
        p.tech.resize(computerization + 1, false);
    }
    s.tech[1].tech[electronics] = true;
    assert!(!s.find_goody_box(u), "explored is not seen");
    let c = Cell::new(3, 2);
    s.world.set_owner(c, Owner::Player(1), Owner::None);
    assert!(!s.find_goody_box(u), "its own ground, without the bonus");
    s.tech[1].tech[computerization] = true;
    let mut enemy = s.clone();
    assert!(s.find_goody_box(u), "its own ground, with Computerization");
    assert_eq!(aimed_at(&s, u), Some(c));
    enemy.world.set_owner(c, Owner::Player(0), Owner::None);
    assert!(!enemy.find_goody_box(u), "an enemy's ground is not lit");
    assert!(enemy.units[u].orders.is_empty());
}

/// **The Spanish arm** (item 3318, `docs/GOODY.md` §7.2): a Spanish
/// leader's box on its own ground answers `was_seen` — which its
/// `leader_flags 0x1000` makes every cell — and never reaches the line
/// of sight; on another's ground the item's gate falls through to
/// `is_seen`, which has no `0x1000` arm. Another power on the same
/// ground is refused. On Great Sahara this is the Scout's re-path to
/// the box on 524 (run896b).
#[test]
fn a_spanish_box_on_its_own_ground_answers_was_seen() {
    use crate::world::Owner;
    let (mut s, u) = sweeper();
    assert!(s.world.set_fog(vec![0; 16 * 16]));
    s.tables.setup.starting_town = true;
    s.tech[1].power = Some(crate::cities::nations::power::SPANISH);
    assert!(!s.find_goody_box(u), "nobody's ground: the line of sight");
    let c = Cell::new(3, 2);
    s.world.set_owner(c, Owner::Player(1), Owner::None);
    let mut french = s.clone();
    let mut enemy = s.clone();
    assert!(s.find_goody_box(u), "its own ground: was_seen");
    assert_eq!(aimed_at(&s, u), Some(c));
    french.tech[1].power = Some(crate::cities::nations::power::FRENCH);
    assert!(!french.find_goody_box(u), "another power: unseen");
    enemy.world.set_owner(c, Owner::Player(0), Owner::None);
    assert!(!enemy.find_goody_box(u), "an enemy's ground: unseen");
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
    s.trace.trace_phases = true;
    let before = s.rng.seed;
    step_onto_the_goody(&mut s, u);
    assert_eq!(
        s.trace
            .phase_marks
            .iter()
            .filter(|(l, _)| l == SITE_PICK)
            .count(),
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
        s.economy.ledgers[1].bucket[Resource::Knowledge.index()],
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
        s.economy.ledgers[1].bucket = [500, 500, 500, 500, 500, 0];
        step_onto_the_goody(&mut s, u);
        assert_eq!(
            s.economy.ledgers[1].bucket[Resource::Oil.index()],
            25,
            "seed {seed:#x}: oil is 500 behind and the jitter is 24 wide"
        );
        assert_eq!(s.economy.ledgers[1].goody_box_resources, 25);
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
        s.economy.ledgers[1].goody_box_resources,
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
    assert_eq!(s.economy.ledgers[1].goody_box_resources, 2 * 26 + 30);
}

/// No candidate at all — every good disabled — still pays, and pays
/// wealth: the original starts at `best = -1` and finishes with a
/// `cmovns` against the literal 2.
#[test]
fn a_player_who_can_gather_nothing_is_paid_in_wealth() {
    let (mut s, u) = sim();
    s.trace.trace_phases = true;
    for g in 0..RESOURCES {
        s.tables.tech_tree.types[g].preq[0] = Preq::Disabled;
    }
    step_onto_the_goody(&mut s, u);
    assert_eq!(
        s.trace
            .phase_marks
            .iter()
            .filter(|(l, _)| l == SITE_PICK)
            .count(),
        0,
        "no candidate, no draw"
    );
    assert_eq!(s.economy.ledgers[1].bucket[Resource::Wealth.index()], 25);
}

/// Frame 0 takes the bit and pays nothing — the whole reward half sits
/// behind `game->frame != 0`, so a unit placed on a goody at setup
/// consumes it silently.
#[test]
fn frame_zero_consumes_the_box_without_paying() {
    let (mut s, u) = sim();
    s.frame = 0;
    s.trace.trace_phases = true;
    step_onto_the_goody(&mut s, u);
    assert_eq!(s.world.cell_data(Cell::new(3, 2)).flags & GOODY, 0);
    assert_eq!(s.economy.ledgers[1].bucket, [0; RESOURCES]);
    assert!(s.trace.phase_marks.iter().all(|(l, _)| l != SITE_PICK));
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
    assert_eq!(s.economy.ledgers[1].bucket, [0; RESOURCES]);
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
    s.tables.unit_types[t].combat.domain = crate::territory::attrition::Domain::Sea;
    s.units[u].ty = Some(t);
    step_onto_the_goody(&mut s, u);
    assert_ne!(s.world.cell_data(Cell::new(3, 2)).flags & GOODY, 0);
    assert_eq!(s.economy.ledgers[1].bucket, [0; RESOURCES]);
}
