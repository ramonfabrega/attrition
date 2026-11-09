use super::*;
use crate::Tuning;
use crate::orders::Body;
use crate::world::{CellData, Terrain, World};

/// A 40 × 40 all-ocean world with one AI player, and a Fisherman —
/// packing, sea domain, in the `FISHERMEN` lineage — standing at cell
/// `(20, 20)`. `land 2` is `OCEAN`, which is what the search accepts.
fn fish_sim() -> (Sim, usize) {
    let mut world = World::new(40, 40);
    world.fill_region(Terrain::Sea, Cell::new(0, 0), Cell::new(39, 39));
    for y in 0..40 {
        for x in 0..40 {
            world.set_cell_data(
                Cell::new(x, y),
                CellData {
                    land: 2,
                    // `WData.down` is `−1` on an empty cell; the
                    // default `0` would read as "object 0 is standing
                    // here" and take every candidate out of the walk.
                    down: -1,
                    ..CellData::default()
                },
            );
            // **The tile grid, which this fixture used to leave
            // empty.** `calc_gather`'s own `sea_sim` has carried it
            // since that walk was written, and `do_cast` now reads it
            // too: its re-seat asks `good_merchant_spot`, whose last
            // test is `calc_gather`, and `gather_tile_ok` refuses a
            // fishing boat any tile that is not ocean. An all-zero
            // mask is plain **land**, so without this the whole
            // fixture was a lake of dry sea.
            for ty in 0..crate::world::TILES_PER_CELL {
                for tx in 0..crate::world::TILES_PER_CELL {
                    world.set_tile_mask(
                        Pos::new(
                            x * crate::world::TILES_PER_CELL + tx,
                            y * crate::world::TILES_PER_CELL + ty,
                        ),
                        crate::world::tile::SURFACE_OCEAN,
                    );
                }
            }
        }
    }
    let mut s = Sim::new(Tuning::RON, world, 2);
    s.nation[0].human = true;
    s.nation[1].human = false;
    // The `TypeIndex` space up to `FISHERMEN`, so the lineage tests in
    // `think` and `add_cast_order` are the real ones. `FISH` is 6 and
    // `OIL` 5, as `TypeIndex` has them.
    {
        let t = &mut s.tables.tech_tree;
        for n in [
            "Food",
            "Timber",
            "Wealth",
            "Knowledge",
            "Metal",
            "Oil",
            "Fish",
        ] {
            t.add(crate::tech::TypeDef::good(n));
        }
        while t.types.len() < FISHERMEN {
            t.add(crate::tech::TypeDef::plain("filler", -1));
        }
        let boat = t.add(crate::tech::TypeDef::unit(
            "Fishing Boat",
            crate::tech::UnitTraits::default(),
        ));
        assert_eq!(boat, FISHERMEN, "the fixture's ids are TypeIndex's");
    }
    s.tech = (0..2)
        .map(|_| crate::tech::PlayerTech::new(&s.tables.tech_tree))
        .collect();
    let t = s.add_unit_type(crate::UnitType {
        hits: 20,
        moves: 40,
        tree: Some(FISHERMEN),
        ..crate::UnitType::default()
    });
    s.tables.unit_types[t].combat.packs = true;
    s.tables.unit_types[t].combat.domain = crate::attrition::Domain::Sea;
    let at = Pos::new(20 * 768 + 384, 20 * 768 + 384);
    let mut u = crate::Unit::new(1, 14, at, 20);
    u.ty = Some(t);
    let i = s.add_unit(u);
    (s, i)
}

/// `TypeIndex::FISH`.
const A_GOOD: usize = 6;

/// `Unit::init@00612100:376` — the bit the whole cadence hangs off.
#[test]
fn a_type_that_packs_is_born_packed() {
    let (s, u) = fish_sim();
    assert!(
        s.units[u].combat.packed,
        "a Fisherman leaves the dock packed, which is what lets it search \
             on its first idle frame instead of once in 1,024"
    );
}

/// The 17 × 17 walk's shape, and the shipped table's typo inside it.
#[test]
fn the_move_table_covers_the_whole_seventeen_by_seventeen() {
    assert_eq!(crate::world::MOVE_289.len(), 0x121);
    assert_eq!(&crate::world::MOVE_289[..49], &crate::world::MOVE_49);
    // Every entry but the last is in the ring its index says, and the
    // ring boundaries are the odd squares.
    assert_eq!(crate::world::MOVE_289[0], (0, 0));
    for (i, &(dx, dy)) in crate::world::MOVE_289.iter().enumerate().take(288).skip(1) {
        let r = dx.abs().max(dy.abs());
        assert!(
            ((2 * r - 1) * (2 * r - 1)..(2 * r + 1) * (2 * r + 1))
                .contains(&i32::try_from(i).unwrap()),
            "entry {i} {:?} is not in ring {r}",
            (dx, dy)
        );
    }
    assert_eq!(
        crate::world::MOVE_289[288],
        (-8, -16),
        "`move_y[288]` is the shipped table's typo; \
             `rondata::pe` checks it against the executable"
    );
}

/// **Nothing worth having anywhere**, which is run54's frame 4462: every
/// numerator is zero, so the score is `rnd(60) + i` and the *last*
/// accepted index all but always wins. The last index is 288, and 288 is
/// the typo — so an empty sea sends a fishing boat sixteen cells north
/// and eight west, out of the square it just searched.
#[test]
fn an_empty_sea_is_decided_by_the_ring_index_and_lands_on_the_typo() {
    let (mut s, u) = fish_sim();
    // Two candidates only — the unit's own cell and the typo's — so the
    // `+ i` term decides it outright rather than on a jitter: index 288
    // beats index 0 by 288 against at most 59. Everything else is
    // `land 0`, `BASELAND`, which the walk refuses.
    for y in 0..40 {
        for x in 0..40 {
            if (x, y) == (20, 20) || (x, y) == (12, 4) {
                continue;
            }
            let c = Cell::new(x, y);
            let mut d = s.world.cell_data(c);
            d.land = 0;
            s.world.set_cell_data(c, d);
        }
    }
    s.units[u].idle = 1;
    assert!(s.think_fish(u, 0), "it takes a move order");
    let Some(Body::Move(m)) = s.units[u].orders.front().map(|o| o.body) else {
        panic!("a move order")
    };
    assert_eq!(
        m.dest.cell(),
        Cell::new(12, 4),
        "(20, 20) + move_289[288] = (20 − 8, 20 − 16)"
    );
}

/// **A good five cells off**, which is run54's frame 4871: a million
/// over the Manhattan distance dwarfs every jitter, so the fish wins
/// however late in the ring the empty water is.
#[test]
fn a_good_in_range_beats_every_empty_cell() {
    let (mut s, u) = fish_sim();
    s.world.add_good(crate::world::Good {
        pos: Pos::new(17 * 768 + 384, 18 * 768 + 384),
        ty: A_GOOD,
        alive: true,
    });
    s.units[u].idle = 1;
    assert!(s.think_fish(u, 0));
    let Some(Body::Move(m)) = s.units[u].orders.front().map(|o| o.body) else {
        panic!("a move order")
    };
    assert_eq!(m.dest.cell(), Cell::new(17, 18), "the fish, at distance 5");
}

/// **Standing on it**, which is run54's frame 4948: the winner is the
/// unit's own cell, so a packed boat unpacks instead of moving — and
/// `add_cast_order` re-aims the generic `0x28c` at the Fisherman's own
/// `0x292`, which is the `spell 658` the dump prints.
#[test]
fn a_boat_on_its_fish_unpacks_with_the_fisherman_s_own_spell() {
    let (mut s, u) = fish_sim();
    let at = s.units[u].pos;
    s.world.add_good(crate::world::Good {
        pos: at,
        ty: A_GOOD,
        alive: true,
    });
    s.units[u].idle = 1;
    assert!(s.think_fish(u, 0));
    let Some(Body::Cast(c)) = s.units[u].orders.front().map(|o| o.body) else {
        panic!("a cast order")
    };
    assert_eq!(c.spell, UNPACK_FISHERMEN, "run58 block 4949: `spell 658`");
}

/// **And the cast survives to run**, which is item 149 end to end
/// (`docs/ORDERS.md` §6.9). run58's `1/14` queues `0x292` on frame
/// 4948, and its dump then prints `spell_time` climbing 1 … 39 on
/// 4950 … 4988 and `unit_masks 786440 → 262152` on 4989 — forty
/// `do_cast` steps, because `craftrules.xml` gives `Deploy`
/// (`Fishermen`) a `JOB_TIME` of 40. Nothing is drawn in any of them,
/// and the boat is still packed until the last.
#[test]
fn the_deploy_waits_out_its_job_time_and_then_clears_the_packed_bit() {
    let (mut s, u) = fish_sim();
    // The craft table as the install has it: 55 rows, and `0x292` is
    // the thirtieth.
    s.orders.spells = vec![crate::orders::SpellType::default(); 55];
    let row = usize::try_from(UNPACK_FISHERMEN - crate::orders::spell::FIRST).unwrap();
    s.orders.spells[row].job_time = 40;
    assert_eq!(s.spell_job_time(UNPACK_FISHERMEN), 40);
    let at = s.units[u].pos;
    s.world.add_good(crate::world::Good {
        pos: at,
        ty: A_GOOD,
        alive: true,
    });
    // **`do_cast`'s re-seat asks `good_merchant_spot` first**, and a
    // Fisherman is a rare collector too (`is_rare_collector`'s
    // fallback is the `FISHERMEN` lineage), so the boat's own tile
    // has to answer `calc_gather` — which reads
    // [`crate::world::tile::AS_BUILDING`] before it looks for a good
    // (`crate::Sim::calc_gather_at`). The shipped ocean carries the
    // bit; this fixture's did not, and without it the deploy died on
    // its first step instead of running its forty.
    let t = at.tile();
    let m = s.world.tile_mask(t);
    s.world
        .set_tile_mask(t, m | crate::world::tile::AS_BUILDING);
    s.units[u].idle = 1;
    assert!(s.think_fish(u, 0));
    let seed = s.rng.seed;
    for step in 1..40 {
        s.work(u, i64::from(step));
        assert_eq!(s.units[u].spell_time, step, "one step a frame");
        assert!(s.units[u].combat.packed, "still packed on step {step}");
        assert!(
            matches!(
                s.units[u].orders.front().map(|o| o.body),
                Some(Body::Cast(_))
            ),
            "and the order is still the cast on step {step}"
        );
    }
    s.work(u, 40);
    assert!(!s.units[u].combat.packed, "the fortieth step deploys it");
    assert_eq!(s.units[u].spell_time, 0, "and the clock is put back");
    assert!(
        s.units[u].orders.is_empty(),
        "`do_cast` kills every craft but the transport once it has cast"
    );
    assert_eq!(s.rng.seed, seed, "and the whole deploy spends no draw");
}

/// **A human's boat deploys through `think`'s rare-collector arm, above
/// `ai off`** (`docs/ORDERS.md` §23, item 543). run127's fisher `0/7`
/// is born under a human leader with the cheat on, so `think_fish` —
/// the tail's, behind the computer block — is never reached; the arm
/// at `think@005f6e40:163` is, and it asks `calc_gather` and then
/// `unpack_merchant(this, 4)`. The two orders are `[MOVE_TO, CAST]`,
/// the walk in front, with the Fisherman's own `0x292`: the dump's
/// block 621 prints them as `CASTORDER spell 658`, then `MOVEORDER`.
#[test]
fn a_human_s_packed_boat_on_its_fish_walks_and_casts_with_ai_off() {
    let (mut s, u) = fish_sim();
    s.nation[1].human = true;
    s.ai_off = true;
    let at = s.units[u].pos;
    s.world.add_good(crate::world::Good {
        pos: at,
        ty: A_GOOD,
        alive: true,
    });
    let t = at.tile();
    let m = s.world.tile_mask(t);
    s.world
        .set_tile_mask(t, m | crate::world::tile::AS_BUILDING);
    s.work(u, 0);
    let bodies: Vec<Body> = s.units[u].orders.iter().map(|o| o.body).collect();
    assert!(
        matches!(bodies.as_slice(), [Body::Move(_), Body::Cast(c)] if c.spell == UNPACK_FISHERMEN),
        "the walk in front of the Fisherman's unpack: {bodies:?}"
    );
    // And without the arm nothing would have: a human's boat under
    // `ai off` loses the tail, `think_fish` with it.
    let (mut s, u) = fish_sim();
    s.nation[1].human = true;
    s.ai_off = true;
    s.units[u].combat.packed = false;
    s.work(u, 0);
    assert!(
        s.units[u].orders.is_empty(),
        "an unpacked human boat takes nothing"
    );
}

/// The arm's other exit: a human's packed boat with nothing to gather
/// where it stands takes no order, and `think:196` bumps `idle` a
/// second time on the way past.
#[test]
fn a_human_s_packed_boat_with_no_fish_bumps_its_idle() {
    let (mut s, u) = fish_sim();
    s.nation[1].human = true;
    s.ai_off = true;
    s.units[u].idle = 0;
    s.work(u, 0);
    assert!(s.units[u].orders.is_empty(), "no deploy without a good");
    let once = {
        let (mut s2, u2) = fish_sim();
        s2.nation[1].human = true;
        s2.ai_off = true;
        s2.units[u2].combat.packed = false;
        s2.units[u2].idle = 0;
        s2.work(u2, 0);
        s2.units[u2].idle
    };
    assert_eq!(
        s.units[u].idle,
        once + 1,
        "the arm's `idle++` on top of `do_idle`'s own"
    );
}

/// The other half of the same frame: a `JOB_TIME` the table does not
/// carry is **0**, and a cast with one lands on its first step. That is
/// the transport craft's own number, and it is why the fixtures that
/// board a barge need no table at all.
#[test]
fn a_craft_with_no_row_casts_on_its_first_step() {
    let (s, _) = fish_sim();
    assert!(s.orders.spells.is_empty());
    assert_eq!(s.spell_job_time(crate::orders::spell::TRANSPORT), 0);
    assert_eq!(s.spell_job_time(UNPACK_FISHERMEN), 0);
}

/// An **oil** patch is in the goods list and in no cell's chain
/// (`Objects::init_good@00653f30` returns before it writes the
/// terminator), and `find_good_at` refuses it a second time by type. So
/// a boat sitting on oil is not sitting on anything.
#[test]
fn oil_is_not_a_good_a_fisherman_can_see() {
    let (mut s, u) = fish_sim();
    let at = s.units[u].pos;
    s.world.add_good(crate::world::Good {
        pos: at,
        ty: OIL_GOOD,
        alive: true,
    });
    assert!(s.find_good_at(at.cell(), 1).is_none());
    s.units[u].idle = 1;
    assert!(s.think_fish(u, 0), "it moves rather than unpacking");
}

/// The head's cadence: an **unpacked** boat looks once in 1,024 frames,
/// phased by `o`, and spends nothing on the other 1,023.
#[test]
fn an_unpacked_boat_re_examines_its_water_once_in_1024_frames() {
    let (mut s, u) = fish_sim();
    s.units[u].combat.packed = false;
    s.units[u].idle = 1;
    let before = s.rng.seed;
    assert!(!s.think_fish(u, 5), "14 + 5 is not a multiple of 1,024");
    assert_eq!(s.rng.seed, before, "and it spends no draw finding out");
    // `o` is 14, so 1,010 is the first frame it looks.
    assert!(s.think_fish(u, 1010));
}

/// A cell whose good another player's object is standing on is skipped
/// **before** the jitter, and a cell holding the unit's own object is
/// not (`005f4e14`: `down >= 0 && down_who < 8 && down != o`).
#[test]
fn a_good_another_unit_is_standing_on_is_not_a_candidate() {
    let (mut s, u) = fish_sim();
    let at = Pos::new(17 * 768 + 384, 18 * 768 + 384);
    s.world.add_good(crate::world::Good {
        pos: at,
        ty: A_GOOD,
        alive: true,
    });
    let mut d = s.world.cell_data(at.cell());
    d.down = 3;
    d.down_who = 0;
    s.world.set_cell_data(at.cell(), d);
    s.units[u].idle = 1;
    assert!(s.think_fish(u, 0));
    let Some(Body::Move(m)) = s.units[u].orders.front().map(|o| o.body) else {
        panic!("a move order")
    };
    assert_ne!(
        m.dest.cell(),
        Cell::new(17, 18),
        "somebody else's claim takes the cell out of the walk"
    );
}

/// And the claim is the **live** chain, not the snapshot a start dump
/// loaded: a boat that has deployed onto its fish is that cell's
/// `WData.down`, and the next boat's search must refuse the cell. That
/// is run54's frame 5285 — 178 accepted cells against the original's
/// 177 — and the second half of item 151.
#[test]
fn a_deployed_boat_is_its_own_cell_s_claim() {
    let (mut s, u) = fish_sim();
    let at = Pos::new(17 * 768 + 384, 18 * 768 + 384);
    s.world.add_good(crate::world::Good {
        pos: at,
        ty: A_GOOD,
        alive: true,
    });
    // Nothing in the snapshot: the fish's cell still reads `down −1`.
    assert_eq!(s.world.cell_data(at.cell()).down, -1);
    let ty = s.units[u].ty;
    let mut other = crate::Unit::new(1, 15, at, 20);
    other.ty = ty;
    let o = s.add_unit(other);
    s.units[o].combat.packed = false;
    assert_eq!(
        s.claim_of(at.cell()),
        Some((15, 1)),
        "`Object::add_to_world` put it at the head of the cell's chain"
    );
    s.units[u].idle = 1;
    assert!(s.think_fish(u, 0));
    let Some(Body::Move(m)) = s.units[u].orders.front().map(|o| o.body) else {
        panic!("a move order")
    };
    assert_ne!(
        m.dest.cell(),
        Cell::new(17, 18),
        "the deployed boat's fish is not a candidate for the second boat"
    );
}

/// **A bird over a deployed boat hides nothing** (item 3709,
/// `docs/COLLISION.md` §3.1): `Object::add_to_world` threads no owner-9
/// object, so the boat stays its cell's head and its fish stays
/// claimed. Threaded, the bird headed the chain with `who 9`, the
/// `who < 8` test let the cell through, and the Mongol Medieval pair's
/// Fishermen `1/85` took the claimed fish on frame 2595.
#[test]
fn a_bird_over_a_deployed_boat_does_not_free_its_fish() {
    let (mut s, u) = fish_sim();
    let at = Pos::new(17 * 768 + 384, 18 * 768 + 384);
    s.world.add_good(crate::world::Good {
        pos: at,
        ty: A_GOOD,
        alive: true,
    });
    let ty = s.units[u].ty;
    let mut other = crate::Unit::new(1, 15, at, 20);
    other.ty = ty;
    let o = s.add_unit(other);
    s.units[o].combat.packed = false;
    let mut bird = crate::Unit::new(9, 19, Pos::new(at.x - 200, at.y - 200), 1);
    bird.ty = ty;
    let b = s.add_unit(bird);
    assert_eq!(
        s.units[b].pos.cell(),
        at.cell(),
        "the bird is over the cell"
    );
    assert_eq!(
        s.cell_chain(at.cell()),
        vec![o],
        "owner 9 is never threaded"
    );
    assert_eq!(s.claim_of(at.cell()), Some((15, 1)));
    s.chain_remove(b);
    assert_eq!(s.cell_chain(at.cell()), vec![o], "nor unthreaded");
    s.units[u].idle = 1;
    assert!(s.think_fish(u, 0));
    let Some(Body::Move(m)) = s.units[u].orders.front().map(|o| o.body) else {
        panic!("a move order")
    };
    assert_ne!(m.dest.cell(), Cell::new(17, 18), "the boat's claim holds");
}
