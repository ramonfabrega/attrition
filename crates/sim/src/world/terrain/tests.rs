use crate::num::single::Single;
use crate::world::{Cell, Pos, TILES_PER_CELL, Terrain, World};
use crate::{Sim, Tuning};

/// **The terraform is the original's SSE, bit for bit** (item 3390):
/// a grid of singles, twenty building boxes flattened one after
/// another, against the host's own `f32` doing the listing's
/// operations in the listing's order — an oracle in the arithmetic
/// the sim refuses, which is the only way to know it agrees.
#[test]
fn a_terraform_lands_on_the_original_s_singles() {
    let cells = 8;
    let mut world = World::new(cells, cells);
    world.fill_region(
        Terrain::Land,
        Cell::new(0, 0),
        Cell::new(cells - 1, cells - 1),
    );
    let mut sim = Sim::new(Tuning::RON, world, 2);
    let side = (cells * TILES_PER_CELL + 1) as usize;
    let mut seed: u32 = 0x1234_5678;
    let mut next = || {
        seed = seed.wrapping_mul(0x0019_660d).wrapping_add(0x3c6e_f35f);
        seed >> 8
    };
    // Heights from 16 up, where a six-decimal print names one single.
    let mut host: Vec<f32> = (0..side * side)
        .map(|_| 16.0 + (next() % 200_000) as f32 / 997.0)
        .collect();
    let grid = host
        .iter()
        .map(|&v| (f64::from(v) * 1e6).round() as i64)
        .collect();
    assert!(sim.world.set_corner_grid(grid));
    for _ in 0..20 {
        let (cx, cy) = (2 + (next() % 22) as i32, 2 + (next() % 22) as i32);
        let (w, h) = (1 + (next() % 4) as i32, 1 + (next() % 4) as i32);
        sim.terraform_for_building(Pos::new(cx, cy), w, h);
        // The host's own singles, in `00875210`'s order.
        let s = side as i32;
        let (x0, y0) = (cx - 1, cy - 1);
        let (x1, y1) = ((cx + w + 2).min(s - 2), (cy + h + 2).min(s - 2));
        let at = |x: i32, y: i32| (y * s + x) as usize;
        let mut sum = 0.0f32;
        let mut n = 0;
        for y in y0..y1 {
            sum += host[at(x1, y)];
            n += 1;
        }
        for x in x0..x1 {
            sum += host[at(x, y1)];
            n += 1;
        }
        for y in y0..y1 {
            for x in x0..x1 {
                sum += host[at(x, y)];
                n += 1;
            }
        }
        let mean = sum / n as f32;
        for y in y0..y1 {
            for x in x0..x1 {
                let border = y == y0 || y == y1 - 1 || x == x0 || x == x1 - 1;
                host[at(x, y)] = if border {
                    (host[at(x, y)] + mean) * 0.5
                } else {
                    mean
                };
            }
        }
    }
    for y in 0..side as i32 {
        for x in 0..side as i32 {
            let (z, exact) = sim.world.corner_z(x, y).expect("on the grid");
            let want = host[y as usize * side + x as usize];
            assert_eq!(z, Single::from_bits(want.to_bits()), "corner ({x}, {y})");
            assert!(exact, "corner ({x}, {y}) is known");
        }
    }
    for ty in 0..side as i32 - 1 {
        for tx in 0..side as i32 - 1 {
            let a = host[(ty as usize + 1) * side + tx as usize];
            let b = host[ty as usize * side + tx as usize + 1];
            assert_eq!(
                sim.world.tile_z(Pos::new(tx, ty)),
                ((a + b) * 0.5) as i32,
                "tile ({tx}, {ty})"
            );
        }
    }
}

/// A 16-cell headless world of plain `BASELAND` at sub 0, the tileset's
/// numbers set, and two building types: a 7×7 city that hides two tiles
/// back and a 5×5 that hides three.
fn placed_world() -> (Sim, usize, usize) {
    use crate::cities::build::{BuildType, Ident, flags};
    use crate::mapgen::halfland::Combo;
    let mut world = World::new(16, 16);
    world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(15, 15));
    world.set_land_set(Some(super::TileSetLand {
        weatherbase: 2,
        combo: Combo {
            coast_base: 3,
            num_tiles: [Some(4), Some(4), Some(4), None],
        },
    }));
    let mut sim = Sim::new(Tuning::RON, world, 2);
    let price = crate::economy::cost::Price {
        kind: crate::economy::cost::Kind::Building,
        ..crate::economy::cost::Price::free()
    };
    let city = sim.add_build_type(BuildType {
        ident: Ident::Village,
        x_size: 7,
        y_size: 7,
        behind_height: 2,
        flags: flags::parse("ean"),
        job_time: 150,
        hits: 400,
        price: price.clone(),
        ..BuildType::default()
    });
    let library = sim.add_build_type(BuildType {
        ident: Ident::Library,
        x_size: 5,
        y_size: 5,
        behind_height: 3,
        job_time: 150,
        hits: 400,
        price,
        ..BuildType::default()
    });
    (sim, city, library)
}

/// The tiles carrying [`crate::world::tile::BEHIND`].
fn behind(sim: &Sim) -> Vec<(i32, i32)> {
    let n = 16 * TILES_PER_CELL;
    (0..n)
        .flat_map(|y| (0..n).map(move |x| (x, y)))
        .filter(|&(x, y)| sim.world.tile_mask(Pos::new(x, y)) & crate::world::tile::BEHIND != 0)
        .collect()
}

/// **A started building writes its ground and what it hides** (item
/// 4136, `docs/MAPGEN.md` §35): `Wall::start`'s `object_placed` gives
/// every `BASELAND` cell from the corner's cell to the cell of corner +
/// size the tileset's `weatherbase`, and `verify_land` steps the sub-0
/// ring beside them to 1; its `mark_behind_tiles` sets the footprint's
/// top row and left column slid back one and two tiles, 26 tiles for a
/// 7×7 at height 2 — the count each Small City holds on run 1562's frame
/// 91. A close clears them, and the neighbour's come back.
#[test]
fn a_started_building_writes_its_ground_and_what_it_hides() {
    let (mut sim, city, library) = placed_world();
    // Tile (32, 32): the corner is (29, 29), cells 7..=9 each way.
    let b = sim.init_build(0, city, Pos::new(32 * 192, 32 * 192), false);
    sim.start_building(b);
    for y in 6..=10 {
        for x in 6..=10 {
            let sub = sim.world.cell_data(Cell::new(x, y)).land_sub;
            let inner = (7..=9).contains(&x) && (7..=9).contains(&y);
            assert_eq!(sub, if inner { 2 } else { 1 }, "cell ({x}, {y})");
        }
    }
    assert_eq!(sim.world.cell_data(Cell::new(5, 5)).land_sub, 0);
    let mut want: Vec<(i32, i32)> = Vec::new();
    for k in 1..=2 {
        want.extend((29 - k..29 - k + 7).map(|x| (x, 29 - k)));
        want.extend((29 - k + 1..29 - k + 7).map(|y| (29 - k, y)));
    }
    want.sort_by_key(|&(x, y)| (y, x));
    want.dedup();
    assert_eq!(want.len(), 26);
    assert_eq!(behind(&sim), want, "the city's hidden tiles");

    // A library to its right, corner (36, 30), whose third ring's top row
    // (y 27, x 33..=37) crosses the city's second ring at (33, 27): the
    // city's close clears that tile with its own, and
    // `refresh_nearby_tiles` (the library's cell (9, 8) is in the band
    // beside the city's (8, 8)) gives the active library it back.
    let l = sim.init_build(0, library, Pos::new(38 * 192 + 96, 32 * 192 + 96), false);
    sim.start_building(l);
    sim.buildings[l].active = true;
    let mut lib: Vec<(i32, i32)> = Vec::new();
    for k in 1..=3 {
        lib.extend((36 - k..36 - k + 5).map(|x| (x, 30 - k)));
        lib.extend((30 - k + 1..30 - k + 5).map(|y| (36 - k, y)));
    }
    lib.sort_by_key(|&(x, y)| (y, x));
    lib.dedup();
    assert!(lib.contains(&(33, 27)) && want.contains(&(33, 27)));
    sim.close_building(b, false);
    assert_eq!(behind(&sim), lib, "the library's own, (33, 27) given back");
    // Without the refresh the shared tile stays cleared.
    sim.mark_behind_tiles(l, false);
    sim.mark_behind_tiles(l, true);
    sim.world
        .clear_tile_bits(Pos::new(33, 27), crate::world::tile::BEHIND);
    assert_ne!(behind(&sim), lib, "the comparison can part");
    // And the ground stays what the city made it.
    assert_eq!(sim.world.cell_data(Cell::new(8, 8)).land_sub, 2);
}

/// Without a tileset ([`World::land_set`]) a building writes no ground.
#[test]
fn no_tileset_no_ground() {
    let (mut sim, city, _) = placed_world();
    sim.world.set_land_set(None);
    let b = sim.init_build(0, city, Pos::new(32 * 192, 32 * 192), false);
    sim.start_building(b);
    assert_eq!(sim.world.cell_data(Cell::new(8, 8)).land_sub, 0);
    assert_eq!(behind(&sim).len(), 26, "the hidden tiles are the type's");
}
