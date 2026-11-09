use super::*;
use crate::build::{BuildType, flags};
use crate::orders::{GatherOrder, Order, flag};
use crate::tech::{TechTree, TypeDef, UnitTraits};
use crate::world::World;
use crate::{Tuning, Unit, UnitType};

/// A citizen of `owner` at `pos`.
fn spawn(sim: &mut Sim, owner: Player, ty: usize, pos: Pos) -> usize {
    let index = i16::try_from(sim.units.len()).unwrap();
    let mut u = Unit::new(owner, index, pos, 100);
    u.ty = Some(ty);
    sim.add_unit(u)
}

/// Owns the cells around the corner of the map, so a site at cell
/// `(1, 1)` stands in friendly territory.
fn own_around(sim: &mut Sim, who: Player) {
    for x in 0..4 {
        for y in 0..4 {
            sim.world
                .set_owner(Cell::new(x, y), Owner::Player(who), Owner::Player(who));
        }
    }
}

/// Puts a `GATHER` order on a unit, so it counts as a gatherer.
fn set_gathering(sim: &mut Sim, u: usize) {
    sim.units[u].orders.push_back(Order {
        flags: flag::ACTION,
        body: Body::Gather(GatherOrder {
            building: 0,
            tile: None,
            wait: 0,
            goto_build: false,
            dist_mod: 0,
            been_there: false,
        }),
    });
    sim.update_action(u);
}

// ------------------------------------------------------------------
// The static tables
// ------------------------------------------------------------------

#[test]
fn the_25_offsets_are_the_5_by_5_each_cell_once() {
    let mut seen: Vec<(i32, i32)> = (0..25).map(|k| (MOVE_X[k], MOVE_Y[k])).collect();
    seen.sort_unstable();
    seen.dedup();
    assert_eq!(seen.len(), 25, "no offset repeats");
    for (x, y) in &seen {
        assert!(
            x.abs() <= 2 && y.abs() <= 2,
            "({x}, {y}) is outside the 5×5"
        );
    }
    assert_eq!((MOVE_X[0], MOVE_Y[0]), (0, 0), "entry 0 is the centre");
    // Entries 1..8 are the compass ring, in `ai_place`'s order rotated
    // to start north-west.
    assert_eq!(
        (1..9).map(|k| (MOVE_X[k], MOVE_Y[k])).collect::<Vec<_>>(),
        [
            (-1, -1),
            (0, -1),
            (1, -1),
            (1, 0),
            (1, 1),
            (0, 1),
            (-1, 1),
            (-1, 0)
        ]
    );
}

#[test]
fn the_outer_ring_is_the_forty_cells_at_chebyshev_five() {
    let mut seen: Vec<(i32, i32)> = (0..40).map(|k| (RING5_X[k], RING5_Y[k])).collect();
    seen.sort_unstable();
    seen.dedup();
    assert_eq!(seen.len(), 40);
    for k in 0..40 {
        assert_eq!(
            RING5_X[k].abs().max(RING5_Y[k].abs()),
            5,
            "entry {k} is off the ring"
        );
    }
    // 11 × 11 minus 9 × 9 = 40.
    assert_eq!(11 * 11 - 9 * 9, 40);
}

// ------------------------------------------------------------------
// A world with the types these functions need
// ------------------------------------------------------------------

struct Kit {
    village: usize,
    citizen_unit: usize,
}

/// A `w × h`-cell land world, two players, with a tree of six goods, a
/// citizen and the three city levels, and the matching records.
fn kit(w: i32, h: i32, players: usize) -> (Sim, Kit) {
    let mut world = World::new(w, h);
    world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(w - 1, h - 1));
    let mut sim = Sim::new(Tuning::RON, world, players);

    let mut tree = TechTree::new();
    for n in ["Food", "Timber", "Wealth", "Knowledge", "Metal", "Oil"] {
        tree.types.push(TypeDef::good(n));
    }
    let citizen = tree.types.len();
    tree.types
        .push(TypeDef::unit("Citizen", UnitTraits::default()));
    let village_t = tree.types.len();
    tree.types.push(TypeDef::new(
        "Small City",
        Kind::Building {
            auto: false,
            wonder: false,
        },
    ));
    let town_t = tree.types.len();
    tree.types.push(TypeDef::new(
        "Large City",
        Kind::Building {
            auto: false,
            wonder: false,
        },
    ));
    tree.types[town_t].from = Some(village_t);
    tree.finalize();
    sim.set_tech_tree(tree);

    let village = sim.add_build_type(BuildType {
        ident: Ident::Village,
        tree: Some(village_t),
        x_size: 7,
        y_size: 7,
        flags: flags::parse("ean"),
        job_time: 100,
        hits: 100,
        price: cost::Price {
            kind: cost::Kind::Building,
            ..cost::Price::free().with_base(crate::economy::Resource::Timber, 10)
        },
        ..BuildType::default()
    });
    sim.add_build_type(BuildType {
        ident: Ident::Town,
        from: Some(village),
        tree: Some(town_t),
        x_size: 7,
        y_size: 7,
        flags: flags::parse("ean"),
        job_time: 100,
        hits: 100,
        price: cost::Price {
            kind: cost::Kind::Building,
            ..cost::Price::free().with_base(crate::economy::Resource::Timber, 10)
        },
        ..BuildType::default()
    });
    let citizen_unit = sim.add_unit_type(UnitType {
        tree: Some(citizen),
        ..UnitType::default()
    });
    for l in &mut sim.ledgers {
        l.bucket = [10_000; RESOURCES];
    }
    for a in &mut sim.ai {
        a.census.resize(64, players);
    }
    (
        sim,
        Kit {
            village,
            citizen_unit,
        },
    )
}

/// **`was_seen`'s leader arm** (`docs/AI.md` §141): on a fog grid with
/// nothing lit and an unowned cell, a leader sees nothing — until it
/// holds `EXPLORE_MAP_BONUS`'s prerequisite (Electronics, `leader_flags
/// & 0x1000`), or a Fouché stands (`num_units[0x141]`, `TypeIndex` 371).
/// The other leader is not lent the sight.
#[test]
fn electronics_or_a_fouche_sees_every_cell() {
    let (mut sim, kit) = kit(8, 8, 2);
    assert!(sim.world.set_fog(vec![0; 8 * 8 * 4]));
    let (fx, fy) = (2 * 5 + 1, 2 * 5 + 1);
    assert!(!sim.was_seen_fog(fx, fy, 1), "the fog alone: unseen");

    let mut tree = sim.tables.tech_tree.clone();
    let electronics = tree.types.len();
    tree.types.push(TypeDef::new("Electronics", Kind::Final));
    tree.roles.explore_map_preq = Some(electronics);
    sim.set_tech_tree(tree);
    for p in &mut sim.tech {
        p.tech.resize(electronics + 1, false);
    }
    let mut fouche = sim.clone();
    sim.tech[1].tech[electronics] = true;
    assert!(sim.was_seen_fog(fx, fy, 1), "Electronics: every cell");
    assert!(!sim.was_seen_fog(fx, fy, 0), "and only its holder's");

    let u = spawn(&mut fouche, 1, kit.citizen_unit, Pos::new(0x180, 0x180));
    assert!(!fouche.was_seen_fog(fx, fy, 1), "a citizen is no Fouché");
    fouche.units[u].type_index = 0x32 + 0x141;
    assert!(
        fouche.was_seen_fog(fx, fy, 1),
        "a Fouché standing: every cell"
    );
    // `TRANSPORTGALLEON` is `TypeIndex` 0x141 itself, which the slot is
    // not: `num_units` starts at `BASE_UNITTYPES`.
    fouche.units[u].type_index = 0x141;
    assert!(
        !fouche.was_seen_fog(fx, fy, 1),
        "a transport galleon sees nothing"
    );
}

/// `Setup::build_empire@005abb80`'s `leader_flags |= 0x1000` for a
/// leader with `has_tribe_bonus(9)`: the Spanish count every cell as
/// seen once they have a city or a starting town, and nobody else does
/// (item 3318, run958's who=1 `leader_flags` 4099).
#[test]
fn the_spanish_see_every_cell_once_they_hold_the_bonus() {
    let (mut sim, _) = kit(8, 8, 2);
    assert!(sim.world.set_fog(vec![0; 8 * 8 * 4]));
    let (fx, fy) = (2 * 5 + 1, 2 * 5 + 1);
    sim.tech[1].power = Some(crate::nations::power::SPANISH);
    sim.tables.setup.starting_town = false;
    sim.tech[1].has_city = false;
    assert!(!sim.was_seen_fog(fx, fy, 1), "no city, no bonus: unseen");
    sim.tech[1].has_city = true;
    assert!(sim.was_seen_fog(fx, fy, 1), "the Spanish: every cell");
    assert!(!sim.was_seen_fog(fx, fy, 0), "and only theirs");
    sim.tech[1].power = Some(crate::nations::power::FRENCH);
    assert!(!sim.was_seen_fog(fx, fy, 1), "another power: unseen");
    sim.tech[1].power = Some(crate::nations::power::SPANISH);
    sim.tables.setup.no_nation_powers = true;
    assert!(!sim.was_seen_fog(fx, fy, 1), "powers off: unseen");
}

#[test]
fn seen_shortcut_matches_reference_after_live_mutations() {
    let (mut sim, kit) = kit(8, 8, 2);
    let u = spawn(&mut sim, 1, kit.citizen_unit, Pos::new(0x180, 0x180));
    for fog in [0, 1, 2, 3, 255] {
        assert!(sim.world.set_fog(vec![fog; 8 * 8 * 4]));
        for reveal in 0..=3 {
            sim.lobby.reveal_map = reveal;
            for ally in [0, 1, 2, 3] {
                sim.ai[0].census.ally_mask = ally;
                sim.ai[1].census.ally_mask = ally;
                for kind in [0x141, FOUCHE] {
                    sim.units[u].type_index = kind;
                    for who in [0, 1, 8, 255] {
                        for fy in -2..18 {
                            for fx in -2..18 {
                                assert_eq!(
                                    sim.was_seen_fog(fx, fy, who),
                                    sim.was_seen_fog_reference(fx, fy, who),
                                    "fog {fog}, reveal {reveal}, ally {ally}, kind {kind}, who {who}, {fx}/{fy}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

// ------------------------------------------------------------------
// The ranking and the insertion
// ------------------------------------------------------------------

/// Run8's block: one site scores and the other nine are empty, so the
/// scorer ranks 10 and the empties rank 9 — `rank` counts every site
/// that scores no more, itself included.
#[test]
fn the_rank_counts_the_sites_that_score_no_more_including_itself() {
    let (mut sim, _) = kit(8, 8, 2);
    sim.frame = 1;
    sim.nation[1].human = false;
    // The nomad's wood test would otherwise zero every site.
    sim.lobby.starting_resources = 8;
    sim.ai[1].census.reg_peasants[0] = 1;
    // One valuable cell, on an even row-major index so the stride-2
    // sampler visits it.
    let c = Cell::new(4, 4);
    assert_eq!((4 * 8 + 4) % 2, 0);
    let mut d = sim.world.cell_data(c);
    d.val = 100;
    sim.world.set_cell_data(c, d);

    sim.compute_sites(1, true);
    let scored: Vec<&Site> = sim.ai[1].sites.iter().filter(|s| s.val > 0).collect();
    assert_eq!(scored.len(), 1, "exactly one cell scores");
    assert_eq!((scored[0].wx, scored[0].wy), (4, 4));
    assert_eq!(
        scored[0].dist, 9,
        "no reference object: dist is the constant"
    );
    assert_eq!(scored[0].rank, 10, "the best of ten ranks ten");
    for s in &sim.ai[1].sites {
        if s.val == 0 {
            assert_eq!(s.rank, 9, "a zero site ranks nine");
        }
    }
}

/// The running-minimum quirk, transliterated from `006cce8c`: with an
/// empty slot 0 and a scored slot 1 the new site beats, the *scored*
/// slot is the one displaced.
#[test]
fn the_insertion_takes_a_beaten_positive_slot_over_an_earlier_empty_one() {
    let pick = |sites: [i32; SITE_SLOTS], val: i32| -> i32 {
        let mut min = 0i32;
        let mut idx = -1i32;
        for (j, s) in sites.iter().enumerate() {
            if !(min < *s && (min > 0 || val <= *s)) {
                min = *s;
                idx = j as i32;
            }
        }
        idx
    };
    // All empty: the last slot wins (every 0 is "taken").
    assert_eq!(pick([0; SITE_SLOTS], 50), 9);
    // Slot 1 scores 5 and we score 100: slot 1 is taken, then slots
    // 2..9 are empty and the last empty wins.
    let mut s = [0; SITE_SLOTS];
    s[1] = 5;
    assert_eq!(pick(s, 100), 9);
    // With every slot filled, the smallest wins, ties to the later.
    let full = [90, 30, 70, 30, 80, 60, 50, 95, 99, 40];
    assert_eq!(pick(full, 100), 3, "the later of the two 30s");
    // A site that beats no slot and finds no empty one is dropped
    // outright: the running minimum never leaves `(0, -1)`.
    assert_eq!(pick(full, 1), -1);
}

// ------------------------------------------------------------------
// The sampler
// ------------------------------------------------------------------

/// A region of 200+ cells takes two draws and samples with the strided
/// offset; a small one takes none and strides by two.
#[test]
fn a_large_region_strides_by_the_two_draws_and_a_small_one_by_two() {
    // 20 × 20 = 400 cells, one region.
    let (mut sim, _) = kit(20, 20, 2);
    sim.frame = 1;
    sim.nation[1].human = false;
    sim.ai[1].census.reg_peasants[0] = 1;
    sim.ai[1].site_mark = 16;
    let before = sim.rng.seed;
    sim.compute_sites(1, true);
    assert_ne!(before, sim.rng.seed, "a 400-cell region draws");

    // Reproduce the arithmetic the sampler ran.
    let mut rng = crate::combat::Rng::new(before);
    let world_w = 20;
    let r0 = rng.roll() % world_w;
    let stride = (400 / (world_w - r0 / 2)).max(10);
    let start = 16 % stride;
    let q = stride / 4;
    let r1 = if q <= 1 { 0 } else { rng.roll() % q };
    assert_eq!(rng.seed, sim.rng.seed, "exactly two draws, in that order");
    assert_eq!(
        sim.ai[1].site_mark,
        16 + (r1 + 3) as u32,
        "site_mark advances by the draw plus three"
    );
    assert!(stride >= 10 && start < stride);

    // A small region draws nothing.
    let (mut small, _) = kit(8, 8, 2);
    small.frame = 1;
    small.nation[1].human = false;
    small.ai[1].census.reg_peasants[0] = 1;
    small.ai[1].site_mark = 16;
    let seed = small.rng.seed;
    small.compute_sites(1, true);
    assert_eq!(seed, small.rng.seed, "64 cells is under the 200 threshold");
    assert_eq!(small.ai[1].site_mark, 16, "and site_mark does not move");
}

/// A human leader takes neither the draws nor the ranking unless the
/// call is forced.
#[test]
fn a_human_leader_is_only_sampled_when_forced() {
    let (mut sim, _) = kit(20, 20, 2);
    sim.frame = 1;
    sim.nation[1].human = true;
    sim.ai[1].census.reg_peasants[0] = 1;
    let seed = sim.rng.seed;
    sim.compute_sites(1, false);
    assert_eq!(seed, sim.rng.seed, "no sweep for a human");
    sim.compute_sites(1, true);
    assert_ne!(seed, sim.rng.seed, "forced, it sweeps");
}

/// Frame 0 zeroes all ten before sampling.
#[test]
fn frame_zero_clears_every_site_first() {
    let (mut sim, _) = kit(8, 8, 2);
    sim.frame = 0;
    sim.nation[1].human = false;
    sim.ai[1].sites[3] = Site {
        wx: 4,
        wy: 4,
        val: 900,
        reg: 0,
        dist: 8,
        rank: 9,
    };
    sim.compute_sites(1, true);
    assert_eq!(sim.ai[1].sites[3].val, 0);
    assert_eq!(sim.ai[1].sites[3].wx, 0);
}

/// Off frame 0, a site is kept and re-scored only with `rank ≥ 6` and
/// `val ≥ 1`.
#[test]
fn a_kept_site_needs_rank_six_and_a_positive_score() {
    let (mut sim, _) = kit(8, 8, 2);
    sim.frame = 1;
    sim.nation[1].human = true;
    sim.ai[1].sites[0] = Site {
        wx: 4,
        wy: 4,
        val: 900,
        reg: 0,
        dist: 8,
        rank: 5,
    };
    sim.ai[1].sites[1] = Site {
        wx: 5,
        wy: 5,
        val: 0,
        reg: 0,
        dist: 8,
        rank: 9,
    };
    sim.compute_sites(1, false);
    assert_eq!(sim.ai[1].sites[0], Site::default(), "rank 5 is cleared");
    assert_eq!(sim.ai[1].sites[1], Site::default(), "val 0 is cleared");
}

// ------------------------------------------------------------------
// compute_site_stats
// ------------------------------------------------------------------

/// §13's settled item: a city of mine outside the site's region ends
/// the score at zero, and so does a dead slot — the loop `return`s.
#[test]
fn a_city_in_another_region_scores_the_site_zero() {
    let (mut sim, kit) = kit(16, 16, 2);
    // Two regions: the left half is region 0, the right half region 1.
    let right = sim
        .world
        .fill_region(Terrain::Land, Cell::new(8, 0), Cell::new(15, 15));
    for c in [Cell::new(2, 2), Cell::new(3, 3)] {
        sim.world.set_owner(c, Owner::Player(1), Owner::Player(1));
    }
    let mut d = sim.world.cell_data(Cell::new(2, 2));
    d.val = 200;
    sim.world.set_cell_data(Cell::new(2, 2), d);

    // One city of mine, in the right-hand region.
    let b = sim.init_build(
        1,
        kit.village,
        Pos::new(10 * UNITS_PER_CELL, 2 * UNITS_PER_CELL),
        false,
    );
    sim.activate(b, false, true);
    assert_eq!(sim.world.region_of(Cell::new(10, 2)), Some(right));
    assert!(sim.city_num(1) > 0, "the city stands");

    let st = sim.compute_site_stats(1, 2, 2, true, true, 0, false);
    assert_eq!(st.val, 0, "the city is in another region: the loop returns");
    assert_eq!(st.dist, 0);
}

/// With no reference object at all the distance term is the constant
/// `k = 3`, `dist = 9` — the loop is skipped.
#[test]
fn no_reference_object_gives_k_three_and_dist_nine() {
    let (mut sim, _) = kit(16, 16, 2);
    // Skip the nomad's wood test (`city_num == 0` here).
    sim.lobby.starting_resources = 8;
    let c = Cell::new(8, 8);
    sim.world.set_owner(c, Owner::Player(1), Owner::Player(1));
    let mut d = sim.world.cell_data(c);
    d.val = 40;
    sim.world.set_cell_data(c, d);
    let st = sim.compute_site_stats(1, 8, 8, false, false, 0, false);
    assert_eq!(st.dist, 9, "dist is the constant, not 2k²");
    assert!(st.val > 0);
}

/// The base is zero — and so is the score — where the **Town's**
/// footprint is blocked, even though a Village would fit.
#[test]
fn a_zero_value_cell_scores_nothing() {
    let (sim, _) = kit(16, 16, 2);
    // Every cell's `val` byte is 0 on a flat world.
    let st = sim.compute_site_stats(1, 8, 8, false, false, 0, false);
    assert_eq!(
        st,
        SiteStats {
            val: 0,
            dist: 0,
            wx: 8,
            wy: 8
        }
    );
}

/// `keep` slides the site onto the best-valued open cell of its 5×5;
/// without it the site stays put.
#[test]
fn keep_slides_the_site_to_the_best_cell_of_its_five_by_five() {
    let (mut sim, _) = kit(16, 16, 2);
    sim.lobby.starting_resources = 8;
    for x in 6..12 {
        for y in 6..12 {
            sim.world
                .set_owner(Cell::new(x, y), Owner::Player(1), Owner::Player(1));
        }
    }
    let mut d = sim.world.cell_data(Cell::new(8, 8));
    d.val = 10;
    sim.world.set_cell_data(Cell::new(8, 8), d);
    // A four-cell block of 200s two cells away: its 2×2 mean is 200.
    for c in [
        Cell::new(10, 9),
        Cell::new(11, 9),
        Cell::new(10, 10),
        Cell::new(11, 10),
    ] {
        let mut d = sim.world.cell_data(c);
        d.val = 200;
        sim.world.set_cell_data(c, d);
    }
    let stay = sim.compute_site_stats(1, 8, 8, false, false, 0, false);
    assert_eq!((stay.wx, stay.wy), (8, 8), "keep = 0 never moves");
    let slid = sim.compute_site_stats(1, 8, 8, false, false, 0, true);
    assert_eq!((slid.wx, slid.wy), (10, 9), "keep = 1 slides to the 200s");
    assert!(slid.val > stay.val);
}

/// **A placed fort is not on the forts list** (item 3708, `docs/AI.md`
/// §358.3): `find_fort` answers only for a fort `Build::activate` has
/// put there, so the sampler's `forts` term — the `×3/2` a leader of
/// three cities gives a site with a fort in its 5×5 — waits for the
/// fort to finish. Ours counted the Persians' unbuilt `1/2034` at
/// (44, 53) from 2375 and scored their home region's samples half
/// again too high.
#[test]
fn a_fort_is_found_once_it_is_built_not_once_it_is_placed() {
    let (mut sim, _) = kit(16, 16, 2);
    let fort = sim.add_build_type(BuildType {
        ident: Ident::Fort,
        x_size: 3,
        y_size: 3,
        flags: flags::parse("ean"),
        job_time: 100,
        hits: 100,
        ..BuildType::default()
    });
    let at = Cell::new(6, 6);
    let b = sim.init_build(
        1,
        fort,
        Pos::new(at.x * UNITS_PER_CELL, at.y * UNITS_PER_CELL),
        false,
    );
    assert_eq!(sim.buildings[b].pos.cell(), at);
    assert!(!sim.fort_at(1, at), "placed, not built: an UnbuiltFort");
    sim.activate(b, false, true);
    assert!(sim.fort_at(1, at), "activated: on forts.lists[1]");
    assert!(!sim.fort_at(0, at), "another leader's list");
}

// ------------------------------------------------------------------
// found_cities
// ------------------------------------------------------------------

/// A leader with a peasant in the region, under its city limit, on an
/// empty region — the multipliers, on a hand-built census.
fn city_ai_fixture() -> (Sim, usize, TypeId) {
    let (mut sim, k) = kit(16, 16, 2);
    sim.frame = 1;
    sim.nation[0].human = false;
    sim.nation[1].human = false;
    sim.ai[1].census.free_peasants = 1;
    sim.ai[1].census.reg_free_peasants[0] = 1;
    sim.ai[1].census.home_reg = 0;
    // Own the cell (else the score is divided by three) and lift the
    // population cap out of the way.
    sim.world
        .set_owner(Cell::new(8, 8), Owner::Player(1), Owner::Player(1));
    sim.roster.muster[1].cap = 200;
    sim.ai[1].sites[0] = Site {
        wx: 8,
        wy: 8,
        val: 1_000,
        reg: 0,
        dist: 8,
        rank: 9,
    };
    let village_t = sim.tables.build_types[k.village].tree.unwrap();
    (sim, k.village, village_t)
}

#[test]
fn the_city_ai_scores_a_site_by_its_census() {
    let (mut sim, village, village_t) = city_ai_fixture();
    assert!(
        sim.offer_site(1, village, village_t, 0),
        "under the city limit, so `want` is set"
    );
    let head = sim.ai[1].make_list.list[0];
    assert_eq!(head.t, village_t as i32);
    assert_eq!((head.wx, head.wy), (8, 8));
    assert_eq!(head.cat, 9, "a city site is category 9");
    assert_eq!(head.escrow, 1);
    assert_eq!(head.num, 1);
    // x4 (no city in the region), x2 (reg_pop <= reg_peasants),
    // x2 (reg_cities <= reg_free_peasants), x30 (under the city limit),
    // then x10, then `/256 * 0x100 / 256` and `/(queued + 1)`.
    let v = 1_000 * 4 * 2 * 2 * 30;
    assert_eq!(v, 480_000);
    assert_eq!(
        head.val,
        (v * 10 / 256) * 0x100 / 256,
        "the value as scored"
    );
    assert_eq!(head.val, 18_750);
}

/// Not owning the cell costs a third, and an unaffordable site is
/// quartered — the last two terms before `make_me`.
#[test]
fn an_unowned_or_unaffordable_site_is_thirded_and_quartered() {
    let (mut sim, village, village_t) = city_ai_fixture();
    sim.world
        .set_owner(Cell::new(8, 8), Owner::None, Owner::None);
    sim.offer_site(1, village, village_t, 0);
    assert_eq!(sim.ai[1].make_list.list[0].val, 18_750 / 3);

    let (mut sim, village, village_t) = city_ai_fixture();
    for l in &mut sim.ledgers {
        l.bucket = [0; RESOURCES];
    }
    assert_eq!(sim.type_affordable(1, village_t, true), 0);
    sim.offer_site(1, village, village_t, 0);
    assert_eq!(sim.ai[1].make_list.list[0].val, 18_750 / 4);
}

/// A live city within five cells drops the site outright.
#[test]
fn a_city_within_five_cells_drops_the_site() {
    let (mut sim, village, village_t) = city_ai_fixture();
    let b = sim.init_build(
        1,
        village,
        Pos::new(11 * UNITS_PER_CELL, 8 * UNITS_PER_CELL),
        false,
    );
    sim.activate(b, false, true);
    assert!(vector_dist(8 - 11, 0) < 5);
    assert!(!sim.offer_site(1, village, village_t, 0));
    assert_eq!(sim.ai[1].make_list.list[0].t, -1, "nothing was offered");
}

/// The human cap: on difficulty 0 the AI stops at `max(the best
/// human's total, 2)`, and `found_cities` then offers nothing at all —
/// the sentinel in the make list survives untouched.
#[test]
fn the_human_cap_stops_the_ai_at_the_best_human_s_count() {
    let (mut sim, k) = kit(16, 16, 2);
    sim.frame = 1;
    sim.nation[0].human = true;
    sim.nation[1].human = false;
    sim.lobby.difficulty = 0;
    sim.ai[1].census.free_peasants = 1;
    sim.ai[1].census.reg_free_peasants[0] = 1;
    sim.ai[1].sites[0] = Site {
        wx: 3,
        wy: 3,
        val: 1_000,
        reg: 0,
        dist: 8,
        rank: 9,
    };
    // The human holds one city; the cap is max(1, 2) = 2.
    let h = sim.init_build(
        0,
        k.village,
        Pos::new(UNITS_PER_CELL, UNITS_PER_CELL),
        false,
    );
    sim.activate(h, false, true);
    assert_eq!(sim.max_human_cities(), 1);
    assert!(!sim.human_capped(1), "with no cities the AI is not capped");

    // Give the AI two: now it is.
    for (x, y) in [(12, 12), (12, 4)] {
        let b = sim.init_build(
            1,
            k.village,
            Pos::new(x * UNITS_PER_CELL, y * UNITS_PER_CELL),
            false,
        );
        sim.activate(b, false, true);
    }
    assert_eq!(sim.total_cities(1), 2);
    assert!(sim.human_capped(1));

    sim.ai[1].make_list.list[0].t = 4242;
    sim.found_cities(1);
    assert_eq!(
        sim.ai[1].make_list.list[0].t, 4242,
        "the capped leader offered nothing and bought nothing"
    );
}
// produce_city
// ------------------------------------------------------------------

/// The nearest citizen is sent — and a gatherer is handicapped
/// twenty-four cells, so a gatherer at ten loses to an idler at thirty.
#[test]
fn the_nearest_citizen_is_sent_with_the_gatherer_s_penalty() {
    let (mut sim, k) = kit(64, 8, 2);
    sim.frame = 1;
    let village_t = sim.tables.build_types[k.village].tree.unwrap();
    // A city of mine, so this is not the nomad path (which sends every
    // citizen at once) — far enough away not to crowd the site, with
    // the site's own cells owned so the placement is in my territory.
    let home = sim.init_build(
        1,
        k.village,
        Pos::new(50 * UNITS_PER_CELL, 4 * UNITS_PER_CELL),
        false,
    );
    sim.activate(home, false, true);
    assert_eq!(sim.city_num(1), 1);
    own_around(&mut sim, 1);

    // A gatherer ten cells away and an idler thirty away.
    let gatherer = spawn(
        &mut sim,
        1,
        k.citizen_unit,
        Pos::new(11 * UNITS_PER_CELL, UNITS_PER_CELL),
    );
    let idler = spawn(
        &mut sim,
        1,
        k.citizen_unit,
        Pos::new(31 * UNITS_PER_CELL, UNITS_PER_CELL),
    );
    set_gathering(&mut sim, gatherer);
    assert_eq!(sim.order_type(gatherer), index::GATHER);

    assert_eq!(
        sim.blocked_site(
            Some(1),
            k.village,
            Pos::new(UNITS_PER_CELL, UNITS_PER_CELL),
            None
        ),
        Blocked::Clear
    );
    assert!(sim.produce_city(1, village_t, 1, 1, 0));
    // 10 + 24 = 34 > 30: the idler goes.
    assert!(
        sim.units[idler]
            .orders
            .iter()
            .any(|o| matches!(o.body, Body::Build(_))),
        "the idler was ordered onto the site"
    );
    assert!(
        !sim.units[gatherer]
            .orders
            .iter()
            .any(|o| matches!(o.body, Body::Build(_))),
        "the gatherer's twenty-four kept it out"
    );
    assert!(
        sim.buildings.iter().any(|b| b.owner == 1 && !b.active),
        "a site was placed"
    );
}

/// A nomad (`city_num == 0`) sends every citizen it owns, not one.
#[test]
fn a_nomad_sends_every_citizen() {
    let (mut sim, k) = kit(64, 8, 2);
    sim.frame = 1;
    let village_t = sim.tables.build_types[k.village].tree.unwrap();
    let a = spawn(
        &mut sim,
        1,
        k.citizen_unit,
        Pos::new(11 * UNITS_PER_CELL, UNITS_PER_CELL),
    );
    let b = spawn(
        &mut sim,
        1,
        k.citizen_unit,
        Pos::new(31 * UNITS_PER_CELL, UNITS_PER_CELL),
    );
    assert_eq!(sim.city_num(1), 0);
    assert!(sim.produce_city(1, village_t, 1, 1, 0));
    for u in [a, b] {
        assert!(
            sim.units[u]
                .orders
                .iter()
                .any(|o| matches!(o.body, Body::Build(_))),
            "citizen {u} was sent"
        );
    }
}

/// A citizen in another region is not a candidate, and with no
/// candidate at all the producer refuses without placing or paying.
#[test]
fn no_citizen_in_the_region_means_no_city() {
    let (mut sim, k) = kit(16, 16, 2);
    sim.frame = 1;
    let village_t = sim.tables.build_types[k.village].tree.unwrap();
    sim.world
        .fill_region(Terrain::Land, Cell::new(8, 0), Cell::new(15, 15));
    spawn(
        &mut sim,
        1,
        k.citizen_unit,
        Pos::new(12 * UNITS_PER_CELL, UNITS_PER_CELL),
    );
    let before = sim.ledgers[1].bucket;
    assert!(!sim.produce_city(1, village_t, 2, 2, 0));
    assert_eq!(sim.ledgers[1].bucket, before, "nothing was charged");
    assert!(!sim.buildings.iter().any(|b| b.owner == 1));
}

/// The census decrement follows the *last examined* citizen's action,
/// not the chosen one's — the original reuses one stack slot
/// (`-0xc(%ebp)`), and the listing confirms it.
#[test]
fn the_census_decrement_uses_the_last_examined_citizen_s_action() {
    let (mut sim, k) = kit(64, 8, 2);
    sim.frame = 1;
    let village_t = sim.tables.build_types[k.village].tree.unwrap();
    let home = sim.init_build(
        1,
        k.village,
        Pos::new(50 * UNITS_PER_CELL, 4 * UNITS_PER_CELL),
        false,
    );
    sim.activate(home, false, true);
    own_around(&mut sim, 1);
    sim.ai[1].census.free_peasants = 5;
    sim.ai[1].census.gatherers = 5;
    sim.ai[1].census.reg_free_peasants[0] = 5;
    sim.ai[1].census.reg_gatherers[0] = 5;

    // The idler comes first in the list and wins; the gatherer is
    // examined after and leaves its action behind.
    spawn(
        &mut sim,
        1,
        k.citizen_unit,
        Pos::new(2 * UNITS_PER_CELL, UNITS_PER_CELL),
    );
    let gatherer = spawn(
        &mut sim,
        1,
        k.citizen_unit,
        Pos::new(40 * UNITS_PER_CELL, UNITS_PER_CELL),
    );
    set_gathering(&mut sim, gatherer);

    assert!(sim.produce_city(1, village_t, 1, 1, 0));
    assert_eq!(sim.ai[1].census.gatherers, 4, "the gatherer's count fell");
    assert_eq!(
        sim.ai[1].census.free_peasants, 5,
        "though the idler is the one that went"
    );
}

/// **"Last" is by object number, not by slot** (item 3710): the loop
/// walks `units[who][o]`, so a gatherer holding a reused, lower number
/// at the end of `self.units` is examined *before* an idler numbered
/// above it. run1387's 4777: ours ended on `1/59` (a gatherer) and
/// took the builder from the gatherers; theirs ended on `1/179`.
#[test]
fn the_last_examined_citizen_is_the_highest_numbered_not_the_last_slot() {
    let (mut sim, k) = kit(64, 8, 2);
    sim.frame = 1;
    let village_t = sim.tables.build_types[k.village].tree.unwrap();
    let home = sim.init_build(
        1,
        k.village,
        Pos::new(50 * UNITS_PER_CELL, 4 * UNITS_PER_CELL),
        false,
    );
    sim.activate(home, false, true);
    own_around(&mut sim, 1);
    sim.ai[1].census.free_peasants = 5;
    sim.ai[1].census.gatherers = 5;
    sim.ai[1].census.reg_free_peasants[0] = 5;
    sim.ai[1].census.reg_gatherers[0] = 5;

    let idler = spawn(
        &mut sim,
        1,
        k.citizen_unit,
        Pos::new(2 * UNITS_PER_CELL, UNITS_PER_CELL),
    );
    let gatherer = spawn(
        &mut sim,
        1,
        k.citizen_unit,
        Pos::new(40 * UNITS_PER_CELL, UNITS_PER_CELL),
    );
    set_gathering(&mut sim, gatherer);
    // The gatherer took a number below the idler's — a reused slot —
    // while it sits after it in this crate's list.
    let (a, b) = (sim.units[idler].index, sim.units[gatherer].index);
    sim.units[idler].index = b;
    sim.units[gatherer].index = a;
    assert!(gatherer > idler && sim.units[gatherer].index < sim.units[idler].index);

    assert!(sim.produce_city(1, village_t, 1, 1, 0));
    assert_eq!(
        sim.ai[1].census.free_peasants, 4,
        "the idler, numbered last, is the last examined"
    );
    assert_eq!(sim.ai[1].census.gatherers, 5, "the gatherers keep theirs");
}
