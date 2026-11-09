use super::*;
use crate::world::UNITS_PER_CELL;

const T: Tuning = Tuning::RON;

fn plain_player() -> PlayerBorders {
    PlayerBorders::new(
        &T,
        0,
        1,
        1,
        &Wonders::default(),
        &NationBonuses::default(),
        0,
    )
}

fn plain_city(pos: Pos) -> City {
    City {
        pos,
        level: 0,
        capital: false,
        temple: false,
        owner_agrees: true,
    }
}

#[test]
fn the_near_field_contraction_stacks() {
    // The headline case: three tests, each reading the last one's output.
    // 4 -> 2 -> 1 -> 0. The near field around a city costs nothing at all.
    assert_eq!(contract(4), 0);
    // 12 -> 8 -> 5, and the third test does not fire because the second
    // landed exactly on its boundary.
    assert_eq!(contract(12), 5);
    // 8 -> 5 -> 3 -> 1. Eight tiles out is charged as one, which is why a
    // border bulges so hard close in.
    assert_eq!(contract(8), 1);
    // At and above thirteen, nothing happens at all, and the step from
    // twelve to thirteen is therefore a jump from 5 to 13 — the border
    // gradient is discontinuous, by construction.
    assert_eq!(contract(13), 13);
    assert_eq!(contract(44), 44);
}

#[test]
fn contraction_is_monotonic() {
    // Not obvious given three compounding truncating divisions, and
    // load-bearing: a non-monotonic metric would make a border with holes.
    let mut prev = contract(0);
    for d in 1..100 {
        let c = contract(d);
        assert!(
            c >= prev,
            "contract({d}) = {c} < contract({}) = {prev}",
            d - 1
        );
        prev = c;
    }
}

#[test]
fn a_plain_city_reaches_the_distance_the_constants_predict() {
    // Cost cap is TERRITORY_BASE << 8 = 6144. With no bonuses the
    // denominator is TERRITORY_NUM = 11, so the furthest affordable
    // contracted distance solves 5 * d * 256 / 11 <= 6144, giving d <= 52.
    let b = plain_player();
    let s = city_source(&T, 0, &b, &plain_city(Pos::new(0, 0)));
    assert_eq!(s.bonuses, 0);
    assert!(cost(&T, SourceKind::City, 52, 0).unwrap() <= cost_cap(&T));
    assert!(cost(&T, SourceKind::City, 53, 0).unwrap() > cost_cap(&T));
    // But the limit clips it long before that: 44 tiles, which is 11
    // cells. The cost curve is not what bounds a plain city's border.
    assert_eq!(s.limit, T.territory_limit_base);
}

#[test]
fn bonuses_buy_reach() {
    let b = plain_player();
    let plain = city_source(&T, 0, &b, &plain_city(Pos::new(0, 0)));
    let metropolis = city_source(
        &T,
        0,
        &b,
        &City {
            level: 2,
            ..plain_city(Pos::new(0, 0))
        },
    );
    assert!(metropolis.bonuses > plain.bonuses);
    assert!(metropolis.limit > plain.limit);
    // A cheaper cost per tile at the same distance is the whole mechanism.
    let d = 30;
    assert!(
        cost(&T, SourceKind::City, d, metropolis.bonuses)
            < cost(&T, SourceKind::City, d, plain.bonuses)
    );
}

/// **The style's triple is the region's** (item 3963, `docs/AI.md`
/// §407): `compute_reg_territory:114–123` reads the player triple on a
/// region whose flags carry bit 4 and the colonized one elsewhere, so
/// two plain cities on two regions of one world reach 16 and 32 tiles
/// — four and eight cells — where the rules' 44 reached eleven.
#[test]
fn a_player_region_takes_the_style_s_player_limits_and_the_rest_its_colonized() {
    let mut w = World::new(80, 1);
    let a = w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(39, 0));
    w.fill_region(Terrain::Land, Cell::new(40, 0), Cell::new(79, 0));
    let at = |cx: i32| Pos::new(cx * UNITS_PER_CELL + UNITS_PER_CELL / 2, UNITS_PER_CELL / 2);
    let b = plain_player();
    let s = [
        city_source(&T, 0, &b, &plain_city(at(0))),
        city_source(&T, 1, &b, &plain_city(at(79))),
    ];
    let mut rules = w.clone();
    compute_all_territory(&mut rules, &T, &s, 2);
    assert_eq!(rules.owner(Cell::new(11, 0)), Owner::Player(0));
    assert_eq!(rules.owner(Cell::new(12, 0)), Owner::None);
    assert_eq!(rules.owner(Cell::new(68, 0)), Owner::Player(1));

    w.set_territory_limits(crate::world::TerritoryLimits {
        player: [16, 4, 4],
        colonized: [32, 4, 4],
    });
    w.set_region_flags(a, crate::world::REGION_PLAYER);
    compute_all_territory(&mut w, &T, &s, 2);
    assert_eq!(
        w.owner(Cell::new(4, 0)),
        Owner::Player(0),
        "the player triple"
    );
    assert_eq!(w.owner(Cell::new(5, 0)), Owner::None, "the player triple");
    assert_eq!(
        w.owner(Cell::new(71, 0)),
        Owner::Player(1),
        "the colonized triple"
    );
    assert_eq!(
        w.owner(Cell::new(70, 0)),
        Owner::None,
        "the colonized triple"
    );
}

/// A source's steps rebuild its limit on any triple: the rules' gives
/// [`Source::limit`] back, and the city and fort steps each multiply
/// their own term (Civic 2 with a Russian flat step, a town with a
/// level-3 temple, a level-3 fort).
#[test]
fn a_source_s_steps_rebuild_its_limit_on_any_triple() {
    let b = PlayerBorders::new(
        &T,
        2,
        3,
        3,
        &Wonders::default(),
        &NationBonuses {
            russian: true,
            civic: 2,
            ..NationBonuses::default()
        },
        0,
    );
    let city = city_source(
        &T,
        0,
        &b,
        &City {
            level: 1,
            temple: true,
            ..plain_city(Pos::new(0, 0))
        },
    );
    let fort = fort_source(
        &T,
        0,
        &b,
        &Fort {
            pos: Pos::new(0, 0),
            red_fort: false,
        },
    );
    let rules = [
        T.territory_limit_base,
        T.territory_limit_civic,
        T.territory_limit_city,
    ];
    for s in [city, fort] {
        assert_eq!(s.limit_in(rules), s.limit, "{:?}", s.kind);
    }
    // Civic 2 plus the Russians' two (`russian_borders != 0` and the
    // Civic level again): four civic steps; level 1 and temple 3: four
    // city steps; fort level 3: two.
    assert_eq!(city.steps, [4, 4]);
    assert_eq!(fort.steps, [4, 2]);
    assert_eq!(city.limit_in([48, 8, 1]), 48 + 32 + 4);
    assert_eq!(fort.limit_in([48, 8, 1]), 48 + 32 + 2);
}

/// A one-region world with a single city at the centre of cell `(cx, cy)`.
fn one_city_world(n: i32, at: Cell) -> (World, Vec<Source>) {
    let mut w = World::new(n, n);
    w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(n - 1, n - 1));
    let b = plain_player();
    let pos = Pos::new(
        at.x * UNITS_PER_CELL + UNITS_PER_CELL / 2,
        at.y * UNITS_PER_CELL + UNITS_PER_CELL / 2,
    );
    let sources = vec![city_source(&T, 0, &b, &plain_city(pos))];
    (w, sources)
}

#[test]
fn one_city_owns_a_blob_around_itself() {
    let (mut w, s) = one_city_world(24, Cell::new(12, 12));
    compute_all_territory(&mut w, &T, &s, 1);
    assert_eq!(w.owner(Cell::new(12, 12)), Owner::Player(0));
    // 44 tiles of limit is 11 cells, and the limit binds well before the
    // cost cap does, so the border stops exactly there and not a cell on.
    assert_eq!(w.owner(Cell::new(1, 12)), Owner::Player(0));
    assert_eq!(w.owner(Cell::new(0, 12)), Owner::None);
    // The claim is contiguous along the ray out of the city.
    for x in 1..=23 {
        assert!(
            w.owner(Cell::new(x, 12)).is_claimed(),
            "cell {x},12 unclaimed"
        );
    }
    // And it is a blob, not a square: the diagonal falls short of the
    // axis, because the metric overshoots there.
    assert_eq!(w.owner(Cell::new(1, 1)), Owner::None);
}

#[test]
fn the_border_is_not_a_circle() {
    // The metric overshoots on the diagonal, so the corner of the world is
    // further away than the axis at the same Chebyshev distance. Measure
    // it directly rather than trusting the outline by eye.
    let axis = vector_dist(40, 0);
    let diagonal = vector_dist(40, 40);
    assert_eq!(axis, 40);
    assert_eq!(diagonal, 60);
    // A true hypotenuse would be 56, so the original's border is pulled in
    // by four tiles at 45 degrees relative to a circle.
    assert!(diagonal > 56);
}

#[test]
fn sea_is_never_owned() {
    let mut w = World::new(8, 4);
    w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(3, 3));
    w.fill_region(Terrain::Sea, Cell::new(4, 0), Cell::new(7, 3));
    let b = plain_player();
    let pos = Pos::new(UNITS_PER_CELL / 2, UNITS_PER_CELL / 2);
    let s = vec![city_source(&T, 0, &b, &plain_city(pos))];
    compute_all_territory(&mut w, &T, &s, 1);
    assert_eq!(w.owner(Cell::new(3, 3)), Owner::Player(0));
    for y in 0..4 {
        for x in 4..8 {
            assert_eq!(w.owner(Cell::new(x, y)), Owner::None, "sea at {x},{y}");
        }
    }
}

#[test]
fn the_nearer_city_wins_and_the_other_is_recorded_second() {
    let mut w = World::new(20, 1);
    w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(19, 0));
    let b = plain_player();
    let at = |cx: i32| Pos::new(cx * UNITS_PER_CELL + UNITS_PER_CELL / 2, UNITS_PER_CELL / 2);
    let s = vec![
        city_source(&T, 0, &b, &plain_city(at(2))),
        city_source(&T, 1, &b, &plain_city(at(17))),
    ];
    compute_all_territory(&mut w, &T, &s, 2);
    assert_eq!(w.owner(Cell::new(2, 0)), Owner::Player(0));
    assert_eq!(w.owner(Cell::new(17, 0)), Owner::Player(1));
    assert_eq!(w.owner(Cell::new(0, 0)), Owner::Player(0));
    assert_eq!(w.owner(Cell::new(19, 0)), Owner::Player(1));
    // Where both reach, the loser is the runner-up rather than nothing.
    assert_eq!(w.owner(Cell::new(9, 0)), Owner::Player(0));
    assert_eq!(w.second(Cell::new(9, 0)), Owner::Player(1));
    // Where only one reaches, there is no runner-up at all.
    assert_eq!(w.second(Cell::new(2, 0)), Owner::None);
}

#[test]
fn a_fort_winning_first_erases_an_over_cap_runner_up_and_a_city_keeps_it() {
    // The original's runner-up bookkeeping differs between its two source
    // paths. An over-cap city is recorded as runner-up; then the first
    // in-cap claim arrives. If it is a city, the runner-up survives; if it
    // is a fort, it is overwritten with nobody. `who2` only.
    let mut w = World::new(40, 1);
    w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(39, 0));
    let at = |cx: i32| Pos::new(cx * UNITS_PER_CELL + UNITS_PER_CELL / 2, UNITS_PER_CELL / 2);
    // Player 0 is scanned first at x = 0. Its city is 156 tiles away —
    // far over the cost cap, but inside a limit large enough to be
    // considered at all, so it lands as the runner-up and nothing more.
    let far_city = Source {
        owner: 0,
        kind: SourceKind::City,
        pos: at(39),
        bonuses: 0,
        limit: 1000,
        steps: [0, 0],
        owner_agrees: true,
    };
    let near_fort = Source {
        owner: 1,
        kind: SourceKind::Fort,
        pos: at(0),
        bonuses: 0,
        limit: 1000,
        steps: [0, 0],
        owner_agrees: true,
    };
    let near_city = Source {
        kind: SourceKind::City,
        ..near_fort
    };

    compute_all_territory(&mut w, &T, &[far_city, near_fort], 2);
    assert_eq!(w.owner(Cell::new(0, 0)), Owner::Player(1));
    assert_eq!(w.second(Cell::new(0, 0)), Owner::None);

    compute_all_territory(&mut w, &T, &[far_city, near_city], 2);
    assert_eq!(w.owner(Cell::new(0, 0)), Owner::Player(1));
    assert_eq!(w.second(Cell::new(0, 0)), Owner::Player(0));
}

#[test]
fn equal_claims_interleave_along_x() {
    // Two identical cities the same distance from a column of cells. The
    // scan-order rotation is what stops player 0 taking the lot.
    let mut w = World::new(9, 3);
    w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(8, 2));
    let b = plain_player();
    let at = |cx: i32, cy: i32| {
        Pos::new(
            cx * UNITS_PER_CELL + UNITS_PER_CELL / 2,
            cy * UNITS_PER_CELL + UNITS_PER_CELL / 2,
        )
    };
    let s = vec![
        city_source(&T, 0, &b, &plain_city(at(4, 0))),
        city_source(&T, 1, &b, &plain_city(at(4, 2))),
    ];
    compute_all_territory(&mut w, &T, &s, 2);
    // Along the equidistant middle row every claim ties, so ownership
    // alternates with x rather than going wholly to player 0.
    let row: Vec<Owner> = (0..9).map(|x| w.owner(Cell::new(x, 1))).collect();
    assert!(row.contains(&Owner::Player(0)), "{row:?}");
    assert!(row.contains(&Owner::Player(1)), "{row:?}");
    for (x, o) in row.iter().enumerate() {
        // The scan starts at slot `x & 7` of eight: player 1 is reached
        // first only when it starts there (x = 1 and 9); from slots 2..7
        // it wraps to player 0 first.
        assert_eq!(*o, Owner::Player(u8::from(x & 7 == 1)), "cell {x},1");
    }
}

#[test]
fn a_two_player_tie_rotates_over_eight_slots() {
    // `compute_reg_territory:338`: `(i + x) & 7`, not `(i + x) % players`.
    // At x = 13 the scan starts at slot 5, so slots 5, 6 and 7 are empty
    // and player 0 is reached before player 1 — East Indies' (13, 15),
    // which `% 2` gave to player 1 (item 3595).
    let mut w = World::new(16, 3);
    w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(15, 2));
    let b = plain_player();
    let at = |cx: i32, cy: i32| {
        Pos::new(
            cx * UNITS_PER_CELL + UNITS_PER_CELL / 2,
            cy * UNITS_PER_CELL + UNITS_PER_CELL / 2,
        )
    };
    let s = vec![
        city_source(&T, 0, &b, &plain_city(at(13, 0))),
        city_source(&T, 1, &b, &plain_city(at(13, 2))),
    ];
    compute_all_territory(&mut w, &T, &s, 2);
    assert_eq!(w.owner(Cell::new(13, 1)), Owner::Player(0));
    assert_eq!(w.owner(Cell::new(9, 1)), Owner::Player(1));
}

#[test]
fn a_source_beyond_its_limit_does_not_claim_at_all() {
    // Put a city far enough away that the limit clips it, and check the
    // cell is unowned rather than owned at a high cost.
    let mut w = World::new(40, 1);
    w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(39, 0));
    let b = plain_player();
    let s = vec![city_source(
        &T,
        0,
        &b,
        &plain_city(Pos::new(UNITS_PER_CELL / 2, UNITS_PER_CELL / 2)),
    )];
    compute_all_territory(&mut w, &T, &s, 1);
    // The limit is 44 tiles; cell 11 is 44 tiles away and cell 12 is 48.
    assert_eq!(w.owner(Cell::new(11, 0)), Owner::Player(0));
    assert_eq!(w.owner(Cell::new(12, 0)), Owner::None);
    assert_eq!(w.second(Cell::new(12, 0)), Owner::None);
}

#[test]
fn a_disowned_city_claims_ambiguously() {
    let mut w = World::new(4, 1);
    w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(3, 0));
    let b = plain_player();
    let s = vec![city_source(
        &T,
        0,
        &b,
        &City {
            owner_agrees: false,
            ..plain_city(Pos::new(UNITS_PER_CELL / 2, UNITS_PER_CELL / 2))
        },
    )];
    compute_all_territory(&mut w, &T, &s, 1);
    assert_eq!(w.owner(Cell::new(0, 0)), Owner::Ambiguous);
    assert!(w.owner(Cell::new(0, 0)).is_claimed());
    assert_eq!(w.owner(Cell::new(0, 0)).player(), None);
}
