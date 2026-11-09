use super::*;

#[test]
fn the_circle_is_rings_of_the_octagonal_metric_in_the_original_s_order() {
    let c = circle();
    assert_eq!(c.radius[0], 1, "ring 0 is the centre");
    assert_eq!((c.x[0], c.y[0]), (0, 0));
    // Ring 1: the eight neighbours, x outer then y inner.
    assert_eq!(c.radius[1], 9);
    assert_eq!(
        (1..9).map(|i| (c.x[i], c.y[i])).collect::<Vec<_>>(),
        [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1)
        ]
    );
    // Every entry of ring r is at octagonal distance r, and the rings
    // are contiguous.
    for r in 1..=0x40usize {
        for i in c.radius[r - 1]..c.radius[r] {
            assert_eq!(vector_dist(c.x[i], c.y[i]), r as i32, "entry {i}");
        }
    }
    assert!(c.x.len() <= MAX_POINTS);
}

/// The two tables agree with each other: each `GRID_THREES` row is
/// exactly the tiles of one 3×3 of the 4×4 that are not in the centre
/// 2×2, and the centre 2×2 is `GRID_ORDER[0..4]`.
#[test]
fn grid_threes_are_the_non_centre_tiles_of_each_three_square() {
    let centre: Vec<(i32, i32)> = GRID_ORDER[..4].to_vec();
    assert_eq!(centre, [(1, 1), (2, 1), (1, 2), (2, 2)]);
    let corners = [(1, 1), (0, 1), (0, 0), (1, 0)];
    for (row, (cx, cy)) in GRID_THREES.iter().zip(corners) {
        let mut got: Vec<(i32, i32)> = row.iter().map(|&k| GRID_ORDER[k]).collect();
        got.sort_unstable();
        let mut want: Vec<(i32, i32)> = (0..3)
            .flat_map(|j| (0..3).map(move |i| (cx + i, cy + j)))
            .filter(|t| !centre.contains(t))
            .collect();
        want.sort_unstable();
        assert_eq!(got, want, "3×3 at ({cx},{cy})");
    }
}

/// A 4×4-cell world (16×16 tiles), two players, and a corner well
/// inside it; `block` marks tiles `BLOCKED` by their `(dx, dy)` from
/// the corner.
fn space(block: &[(i32, i32)]) -> i32 {
    let mut w = crate::World::new(4, 4);
    for &(i, j) in block {
        w.set_tile_bits(Pos::new(4 + i, 4 + j), tile::BLOCKED);
    }
    let sim = Sim::new(crate::Tuning::RON, w, 2);
    sim.space_at_corner(1, 4, 4, false)
}

/// The early-out is on the centre 2×2, not the top row: a blocked
/// corner tile leaves three 3×3s free and scores 3 (the harness scored
/// 0 until run20's `CITY` record put `space[0..1]` ten cells short —
/// `docs/AI.md` §15.9), and a blocked centre tile is 0 outright.
#[test]
fn a_blocked_edge_tile_leaves_a_three_square_but_a_blocked_centre_is_nothing() {
    assert_eq!(space(&[]), 4);
    assert_eq!(space(&[(0, 0)]), 3);
    assert_eq!(space(&[(3, 0)]), 3);
    assert_eq!(space(&[(0, 3), (3, 3)]), 3);
    assert_eq!(space(&[(1, 1)]), 0);
    assert_eq!(space(&[(2, 2)]), 0);
    // Both rows out: no 3×3, seven blocked or eight — 2 either way.
    assert_eq!(
        space(&[(0, 0), (1, 0), (2, 0), (3, 0), (0, 3), (1, 3), (2, 3)]),
        2
    );
    assert_eq!(
        space(&[
            (0, 0),
            (1, 0),
            (2, 0),
            (3, 0),
            (0, 3),
            (1, 3),
            (2, 3),
            (3, 3)
        ]),
        2
    );
    // Every non-centre tile blocked: still 2, never 0.
    let ring: Vec<(i32, i32)> = GRID_ORDER[4..].to_vec();
    assert_eq!(space(&ring), 2);
}

/// `check_building_wcoord`'s three gates on the cell itself, before any
/// corner is looked at: another leader's, occupied (`flags & 0x70`),
/// or `blocked == 0x10`.
#[test]
fn check_building_wcoord_gates_on_the_cell_before_the_corners() {
    let mut sim = Sim::new(crate::Tuning::RON, crate::World::new(4, 4), 2);
    let cell = Cell::new(1, 1);
    assert_eq!(sim.check_building_wcoord(1, cell, 0, 0, 1, false), 4);
    let mut d = sim.world.cell_data(cell);
    d.flags |= 0x10;
    sim.world.set_cell_data(cell, d);
    assert_eq!(sim.check_building_wcoord(1, cell, 0, 0, 1, false), 0);
    d.flags &= !0x70;
    d.blocked = 0x10;
    sim.world.set_cell_data(cell, d);
    assert_eq!(sim.check_building_wcoord(1, cell, 0, 0, 1, false), 0);
    d.blocked = 0;
    sim.world.set_cell_data(cell, d);
    assert_eq!(sim.check_building_wcoord(1, cell, 0, 0, 1, false), 4);
    sim.world
        .set_owner(cell, crate::Owner::Player(0), crate::Owner::None);
    assert_eq!(sim.check_building_wcoord(1, cell, 0, 0, 1, false), 0);
}

/// **`find_friends` asks whose footprint covers a cell's centre tile,
/// not whose centre is in the cell** (`docs/AI.md` §26).
///
/// The geometry is East Indies' AI Village, off run58's own dump: 7×7
/// at `(39264, 40032)`, corner tiles `201..207 × 205..211`. Centre
/// tiles are four apart, so a seven-tile footprint covers two of them
/// on each axis — the city is the neighbour of **four** cells, and its
/// own is one of them. Every ordinary building is four tiles or fewer
/// and covers exactly one, which is why the two readings agree
/// everywhere else.
#[test]
fn the_city_centre_is_the_friend_of_four_cells() {
    let mut sim = Sim::new(crate::Tuning::RON, crate::World::new(60, 60), 2);
    let rec = sim.tables.build_types.len();
    sim.tables
        .build_types
        .push(crate::cities::build::BuildType {
            ident: Ident::Village,
            x_size: 7,
            y_size: 7,
            hits: 100,
            ..crate::cities::build::BuildType::default()
        });
    let pos = Pos::new(39264, 40032);
    let b = sim.add_building(1, pos, 8);
    sim.buildings[b].ty = Some(rec);
    assert_eq!(
        (pos.cell().x, pos.cell().y),
        (51, 52),
        "the Village's own cell — what this crate used to answer, alone"
    );
    let corner = sim.tile_corner(rec, pos);
    assert_eq!((corner.x, corner.y), (201, 205), "run58's `1/2000`");
    // The mask is the callee's first gate: a tile no building has
    // masked answers `None` before any footprint is looked at.
    for t in sim.footprint(rec, corner) {
        sim.world
            .set_tile_field(t, tile::OBJECT, tile::OBJECT_BUILDING);
    }
    let friend_of: Vec<(i32, i32)> = (50..53)
        .flat_map(|x| (51..54).map(move |y| (x, y)))
        .filter(|&(x, y)| sim.building_placed_at(1, Cell::new(x, y)) == Some(b))
        .collect();
    assert_eq!(
        friend_of,
        [(50, 51), (50, 52), (51, 51), (51, 52)],
        "the four cells whose centre tile (cell·4 + 2) the 7×7 covers"
    );
    assert_eq!(
        sim.building_placed_at(0, Cell::new(51, 52)),
        None,
        "`param_3` filters by owner"
    );
    // Four tiles or fewer: exactly one cell, and the two readings agree.
    sim.tables.build_types[rec].x_size = 4;
    sim.tables.build_types[rec].y_size = 4;
    sim.buildings[b].pos = Pos::new(51 * 768 + 384, 52 * 768 + 384);
    let small: Vec<(i32, i32)> = (50..53)
        .flat_map(|x| (51..54).map(move |y| (x, y)))
        .filter(|&(x, y)| sim.building_placed_at(1, Cell::new(x, y)) == Some(b))
        .collect();
    assert_eq!(small, [(51, 52)]);
}

/// **A placed site holds no centre until it starts** (item 3860,
/// `docs/GOLDEN.md` §153.5): the spiral's skip is the cell's `0x4000`,
/// which `BuildType::mask_me` sets from `Wall::start` and clears from
/// `Wall::close`. The fourth melee's French Shipyard `1/2033`, placed
/// on 1782 and unstarted on 1985, left its cell (19, 48) to the next
/// dock in theirs.
#[test]
fn a_placed_site_holds_no_centre_until_it_starts() {
    let mut sim = Sim::new(crate::Tuning::RON, crate::World::new(60, 60), 2);
    let rec = sim.tables.build_types.len();
    sim.tables
        .build_types
        .push(crate::cities::build::BuildType {
            x_size: 4,
            y_size: 4,
            hits: 100,
            ..crate::cities::build::BuildType::default()
        });
    let pos = Pos::new(14592, 36864);
    let b = sim.init_build(1, rec, pos, false);
    let cell = Cell::new(19, 48);
    assert_eq!(pos.cell(), cell, "`1/2033`'s own cell");
    assert!(!sim.buildings[b].started, "placed, not begun");
    assert!(!sim.cell_has_centre(cell), "no `Wall::start`, no `0x4000`");
    sim.buildings[b].started = true;
    assert!(sim.cell_has_centre(cell), "`Wall::start` masks the centre");
    sim.buildings[b].alive = false;
    assert!(!sim.cell_has_centre(cell), "`Wall::close` clears it");
}

/// **An enhancer's friends are the gather buildings of its own good**
/// (`find_friends`' first arm, `0x639380`–`0x6393b0`): a Granary counts
/// a farm beside it, `+2` on a cardinal and `+1` on a diagonal, and
/// nothing else; a Lumber Mill counts the woodcutter's camp and not the
/// farm. Great Sahara at Toughest's Granary `1/2023` on frame 5376 is
/// this arm: without it every cell scores friendless and the Granary
/// goes three cells south (`docs/AI.md` §99.7).
#[test]
fn an_enhancer_counts_the_gather_buildings_of_its_good() {
    let mut sim = Sim::new(crate::Tuning::RON, crate::World::new(60, 60), 2);
    let mut ty = |ident: Ident| {
        let rec = sim.tables.build_types.len();
        sim.tables
            .build_types
            .push(crate::cities::build::BuildType {
                ident,
                x_size: 4,
                y_size: 4,
                hits: 100,
                ..crate::cities::build::BuildType::default()
            });
        rec
    };
    let (farm, camp, barracks) = (ty(Ident::Farm), ty(Ident::Woodcutter), ty(Ident::Barracks));
    let (granary, mill) = (ty(Ident::Granary), ty(Ident::Lumbermill));
    let mut put = |rec: usize, x: i32, y: i32| {
        let pos = Pos::new(x * 768 + 384, y * 768 + 384);
        let b = sim.add_building(1, pos, 8);
        sim.buildings[b].ty = Some(rec);
        let corner = sim.tile_corner(rec, pos);
        for t in sim.footprint(rec, corner) {
            sim.world
                .set_tile_field(t, tile::OBJECT, tile::OBJECT_BUILDING);
        }
    };
    // Around the candidate (20, 21): a farm on the cardinal above, a
    // farm on the diagonal above-right, a camp on the cardinal below and
    // a barracks on the cardinal left.
    put(farm, 20, 20);
    put(farm, 21, 20);
    put(camp, 20, 22);
    put(barracks, 19, 21);
    let at = Cell::new(20, 21);
    assert_eq!(
        sim.find_friends(granary, at, None, 1),
        3,
        "the two farms, cardinal and diagonal"
    );
    assert_eq!(sim.find_friends(mill, at, None, 1), 2, "the camp alone");
    assert_eq!(
        sim.find_friends(granary, at, None, 0),
        0,
        "another player's farms are nobody's friends"
    );
}

/// **A trainer's friends are the trainers the derived flag names**
/// (`find_friends`' second arm, `is_military_trainer` on both sides):
/// the flag on the root of the `FROM` chain, which holds the Dock, the
/// Airbase and the Missile Silo beside the land trainers. A Silo counts
/// an Airbase beside it, `+2` on a cardinal and `+1` on a diagonal, and
/// not a Library. The coverage pair's second Silo, `1/2021` on frame
/// 182, is this arm: read off a five-ident list it fell to the
/// "anything else" arm and went to (190, 214) where the original puts it
/// at (214, 194) (`docs/AI.md` §143).
#[test]
fn a_missile_silo_counts_the_trainers_the_derived_flag_names() {
    let mut sim = Sim::new(crate::Tuning::RON, crate::World::new(60, 60), 2);
    let mut ty = |ident: Ident, trainer: bool| {
        let rec = sim.tables.build_types.len();
        sim.tables
            .build_types
            .push(crate::cities::build::BuildType {
                ident,
                x_size: 4,
                y_size: 4,
                hits: 100,
                flags: if trainer { flags::MILITARY_TRAINER } else { 0 },
                ..crate::cities::build::BuildType::default()
            });
        rec
    };
    let silo = ty(Ident::MissileSilo, true);
    let (airbase, library) = (ty(Ident::Airbase, true), ty(Ident::Other, false));
    let mut put = |rec: usize, x: i32, y: i32| {
        let pos = Pos::new(x * 768 + 384, y * 768 + 384);
        let b = sim.add_building(1, pos, 8);
        sim.buildings[b].ty = Some(rec);
        let corner = sim.tile_corner(rec, pos);
        for t in sim.footprint(rec, corner) {
            sim.world
                .set_tile_field(t, tile::OBJECT, tile::OBJECT_BUILDING);
        }
    };
    // Around the candidate (20, 21): an Airbase on the cardinal above,
    // one on the diagonal above-right, a Library on the cardinal left.
    put(airbase, 20, 20);
    put(airbase, 21, 20);
    put(library, 19, 21);
    let at = Cell::new(20, 21);
    assert_eq!(
        sim.find_friends(silo, at, None, 1),
        3,
        "the two Airbases, cardinal and diagonal; the Library is no trainer"
    );
    // Off the flag, the same Silo falls to the "anything else" arm: the
    // Library counts and a non-city trainer is nothing.
    sim.tables.build_types[silo].flags = 0;
    assert_eq!(sim.find_friends(silo, at, None, 1), 2, "the Library alone");
}

/// **A Keep's friends are a Tower's** (item 1377): `find_friends`'
/// tower arm is `is(0x1b7, 0)`, the line, so a Keep (`FROM` Tower)
/// counts each gather neighbour `+2` as a Tower does, and its arrows do
/// not send it to the early zero. East Indies' Keep `1/2047` on frame
/// 15985 is this arm: two farms beside (51, 28) make it `(4 + 2)² ×
/// 1000` where every friendless cell scores 1255.
#[test]
fn a_keep_counts_its_gather_neighbours_as_a_tower_does() {
    let mut sim = Sim::new(crate::Tuning::RON, crate::World::new(60, 60), 2);
    let mut ty = |ident: Ident, from: Option<usize>, letters: &str, attack: i32| {
        let rec = sim.tables.build_types.len();
        sim.tables
            .build_types
            .push(crate::cities::build::BuildType {
                ident,
                from,
                flags: flags::parse(letters),
                attack,
                x_size: 2,
                y_size: 2,
                hits: 100,
                ..crate::cities::build::BuildType::default()
            });
        rec
    };
    let farm = ty(Ident::Farm, None, "gda", 0);
    let tower = ty(Ident::Tower, None, "ean", 12);
    let keep = ty(Ident::Other, Some(tower), "ecan", 16);
    let mut put = |rec: usize, x: i32, y: i32| {
        let pos = Pos::new(x * 768 + 384, y * 768 + 384);
        let b = sim.add_building(1, pos, 8);
        sim.buildings[b].ty = Some(rec);
        let corner = sim.tile_corner(rec, pos);
        for t in sim.footprint(rec, corner) {
            sim.world
                .set_tile_field(t, tile::OBJECT, tile::OBJECT_BUILDING);
        }
    };
    put(farm, 50, 28);
    put(farm, 50, 29);
    // And the placement's own `local_84`: the line, inside the `e` arm.
    assert_eq!(sim.placement_lines(keep, true), (true, false));
    assert_eq!(sim.placement_lines(keep, false), (false, false), "frame 0");
    let at = Cell::new(51, 28);
    assert_eq!(
        sim.find_friends(tower, at, None, 1),
        4,
        "two farms, +2 each"
    );
    assert_eq!(
        sim.find_friends(keep, at, None, 1),
        4,
        "the Keep is of the Tower's line"
    );
}
/// A world of 20×20 cells with an oil patch (a good of type `OIL`, in
/// the goods list and in no cell's chain) at each of `cells`, every one
/// owned by player 1; the patches are in leader 1's list in order.
fn oil_world(cells: &[(i32, i32)]) -> (Sim, Vec<usize>) {
    let mut sim = Sim::new(crate::Tuning::RON, crate::World::new(20, 20), 2);
    let mut gis = Vec::new();
    for &(x, y) in cells {
        let c = Cell::new(x, y);
        sim.world
            .set_owner(c, crate::Owner::Player(1), crate::Owner::None);
        let gi = sim.world.add_good(crate::world::Good {
            pos: Pos::new(x * UNITS_PER_CELL + 384, y * UNITS_PER_CELL + 384),
            ty: crate::world::OIL_GOOD,
            alive: true,
        });
        gis.push(gi);
        sim.ai.ai[1].oil_patches.push(gi);
    }
    (sim, gis)
}

fn well() -> crate::cities::build::BuildType {
    crate::cities::build::BuildType {
        ident: Ident::OilWell,
        x_size: 3,
        y_size: 3,
        ..crate::cities::build::BuildType::default()
    }
}

/// **Arm 4.2 walks the patches last to first, takes the best score, and
/// removes a land patch the leader does not own** (`006e1730`–`006e18da`):
/// the score is `2 · width − vector_dist(patch − anchor)` in cells, the
/// candidate the cell's corner plus the well's half footprint
/// (`x · 0x300 + x_size · 0x60`), and the list loses the unowned one —
/// by value, the tail shifting down.
#[test]
fn an_oil_well_takes_the_nearest_owned_patch_and_prunes_the_unowned() {
    let (mut sim, gis) = oil_world(&[(5, 5), (10, 10), (12, 3)]);
    sim.world.set_owner(
        Cell::new(12, 3),
        crate::Owner::Player(0),
        crate::Owner::None,
    );
    let (best, sp, cand) = sim.pick_oil_patch(1, Cell::new(6, 6), &well());
    assert_eq!(best, 40 - vector_dist(1, 1), "20·2 − the octagonal 1");
    assert_eq!(sp, 3, "the well's x_size");
    assert_eq!(
        cand,
        Some(Pos::new(
            5 * UNITS_PER_CELL + 3 * 96,
            5 * UNITS_PER_CELL + 3 * 96
        ))
    );
    assert_eq!(
        sim.ai.ai[1].oil_patches,
        [gis[0], gis[1]],
        "the third is another leader's land, and goes"
    );
}

/// **A tie goes to the earlier patch**: the walk is last to first and
/// the comparison `>=` (`006e19c8`, `jl` skips only a lower score).
#[test]
fn an_equal_score_goes_to_the_earlier_patch() {
    let (mut sim, _) = oil_world(&[(5, 7), (7, 5)]);
    let (_, _, cand) = sim.pick_oil_patch(1, Cell::new(6, 6), &well());
    assert_eq!(cand.map(|p| p.cell()), Some(Cell::new(5, 7)));
}

/// An enemy object within `0x600` of the patch's cell centre refuses it
/// (and only an enemy: `SEARCH_ENEMY`, so a leader not at war does not).
#[test]
fn an_enemy_within_three_halves_of_a_cell_refuses_a_patch() {
    let (mut sim, _) = oil_world(&[(5, 5), (10, 10)]);
    let centre = Pos::new(5 * UNITS_PER_CELL + 0x180, 5 * UNITS_PER_CELL + 0x180);
    let b = sim.add_building(0, Pos::new(centre.x + 0x500, centre.y), 8);
    let _ = b;
    let (_, _, cand) = sim.pick_oil_patch(1, Cell::new(6, 6), &well());
    assert_eq!(
        cand.map(|p| p.cell()),
        Some(Cell::new(5, 5)),
        "no war, so no enemy"
    );
    sim.declare_war(0, 1);
    let (_, _, cand) = sim.pick_oil_patch(1, Cell::new(6, 6), &well());
    assert_eq!(cand.map(|p| p.cell()), Some(Cell::new(10, 10)));
}

/// **A patch a building closed on is passed over once** (`Build::close
/// @00628980:183-187` sets [`cell::CLOSED`]; `006e1730`–`006e195f`
/// reads it, item 1602, `docs/AI.md` §168): with the bit set the arm
/// takes nothing from the patch, and clears the bit only with no enemy
/// object within `0xf00` of the cell's centre — so the next pass takes
/// it. A capture's close, reason 5, sets nothing. In the coverage pair
/// the disbanded Oil Platform site `1/2048` left (36, 40) flagged on
/// 2048, and leader 1's next platform went to (58, 31) on 2182.
#[test]
fn a_patch_a_building_closed_on_is_passed_over_once() {
    let (mut sim, _) = oil_world(&[(5, 5), (10, 10)]);
    let (near, far) = (Cell::new(5, 5), Cell::new(10, 10));
    let anchor = Cell::new(6, 6);
    let centre = Pos::new(5 * UNITS_PER_CELL + 0x180, 5 * UNITS_PER_CELL + 0x180);
    let pick = |sim: &mut Sim| sim.pick_oil_patch(1, anchor, &well()).2.map(|p| p.cell());
    let flagged = |sim: &Sim| sim.world.cell_data(near).flags & cell::CLOSED != 0;
    assert_eq!(pick(&mut sim), Some(near), "unflagged, the nearer");
    let site = sim.add_building(1, centre, 8);
    sim.close_building(site, true);
    assert!(!flagged(&sim), "reason 5 flags nothing");
    let site = sim.add_building(1, centre, 8);
    sim.close_building(site, false);
    assert!(flagged(&sim), "any other close flags the cell");
    // An enemy within `0xf00`: passed over, and the bit stays.
    let enemy = sim.add_building(0, Pos::new(centre.x + 0xe00, centre.y), 8);
    sim.declare_war(0, 1);
    assert_eq!(pick(&mut sim), Some(far));
    assert_eq!(pick(&mut sim), Some(far));
    assert!(flagged(&sim));
    // None: passed over once more, and the bit clears for the next.
    sim.close_building(enemy, false);
    assert_eq!(pick(&mut sim), Some(far), "the pass that clears it");
    assert!(!flagged(&sim));
    assert_eq!(pick(&mut sim), Some(near));
}

/// A patch on the sea is kept, not pruned, and not taken, by a leader
/// with no transport level (`leader_flags & 0x700`).
#[test]
fn a_sea_patch_waits_for_a_transport_level_and_is_not_pruned() {
    let (mut sim, gis) = oil_world(&[(5, 5)]);
    let c = Cell::new(5, 5);
    let mut d = sim.world.cell_data(c);
    d.land = 2;
    sim.world.set_cell_data(c, d);
    sim.world
        .set_owner(c, crate::Owner::None, crate::Owner::None);
    assert!(sim.world.is_ocean(c));
    let (best, _, cand) = sim.pick_oil_patch(1, Cell::new(6, 6), &well());
    assert_eq!((best, cand), (0, None));
    assert_eq!(sim.ai.ai[1].oil_patches, [gis[0]], "kept");
}

/// **`reveal_fog`'s oil arm** adds the first live good on a cell whose
/// own flag is `0x800`, once; and **`compute_reg_territory`'s per-cell
/// scan** adds an oil patch on a cell it gives a leader.
#[test]
fn a_reveal_and_a_claim_each_add_the_patch_once() {
    let (mut sim, gis) = oil_world(&[(5, 5), (8, 8)]);
    sim.ai.ai[1].oil_patches.clear();
    // No `0x800` on the cell: the reveal adds nothing.
    sim.reveal_fog(10, 10, 1);
    assert!(sim.ai.ai[1].oil_patches.is_empty());
    let mut d = sim.world.cell_data(Cell::new(5, 5));
    d.flags |= crate::world::cell::OIL;
    sim.world.set_cell_data(Cell::new(5, 5), d);
    sim.reveal_fog(10, 10, 1);
    sim.reveal_fog(11, 10, 1);
    assert_eq!(
        sim.ai.ai[1].oil_patches,
        [gis[0]],
        "once, for two fog cells"
    );
    // The claim needs no flag, only the patch standing on the cell.
    sim.claim_cell_goods(Cell::new(8, 8), 1);
    sim.claim_cell_goods(Cell::new(8, 8), 1);
    assert_eq!(sim.ai.ai[1].oil_patches, [gis[0], gis[1]]);
    sim.claim_cell_goods(Cell::new(9, 9), 1);
    assert_eq!(sim.ai.ai[1].oil_patches.len(), 2, "no good on (9, 9)");
    // The wholesale form walks the goods list, for whoever owns the cell.
    sim.ai.ai[1].oil_patches.clear();
    sim.claim_oil_from_owners();
    assert_eq!(sim.ai.ai[1].oil_patches, [gis[0], gis[1]]);
    assert!(sim.ai.ai[0].oil_patches.is_empty());
}

/// **A land patch joins the list in the per-cell scan's order, not the
/// goods list's** (item 1538, `docs/AI.md` §147): the wholesale claim
/// walks a land region row-major, so of two patches tied on score
/// `pick_oil_patch` — last to first, `>=` — reaches the later row's
/// first and ends on the earlier row's, however the goods list orders
/// them. A patch outside any land region keeps the goods list's order,
/// after the land ones.
#[test]
fn a_wholesale_claim_adds_land_patches_in_cell_scan_order() {
    // Goods in the order (10, 3), (4, 9), (6, 3), (2, 15); the last is
    // water. Row-major over the land: (6, 3), (10, 3), (4, 9).
    let (mut sim, gis) = oil_world(&[(10, 3), (4, 9), (6, 3), (2, 15)]);
    sim.world.fill_region(
        crate::world::Terrain::Land,
        Cell::new(0, 0),
        Cell::new(19, 9),
    );
    sim.ai.ai[1].oil_patches.clear();
    sim.claim_oil_from_owners();
    assert_eq!(sim.ai.ai[1].oil_patches, [gis[2], gis[0], gis[1], gis[3]]);
    // From (8, 8) the patches at (6, 3) and (10, 3) tie at 2 · 20 − 6;
    // the walk ends on the list's first, the earlier row's.
    sim.ai.ai[1]
        .oil_patches
        .retain(|g| *g == gis[0] || *g == gis[2]);
    let (best, _, cand) = sim.pick_oil_patch(1, Cell::new(8, 8), &well());
    assert!(best > 0);
    assert_eq!(cand.map(|p| p.cell()), Some(Cell::new(6, 3)));
}
