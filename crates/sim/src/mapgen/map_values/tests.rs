use super::*;

/// `rules.xml`'s nine lands as the type indices read them.
fn makes() -> LandMakes {
    vec![
        [3, 0, -1, -1],
        [-1; 4],
        [-1; 4],
        [-1; 4],
        [1, -1, -1, -1],
        [4, -1, -1, -1],
        [-1; 4],
        [5, -1, -1, -1],
        [4, -1, -1, -1],
    ]
}

#[test]
fn open_water_draws_nothing_and_is_zero() {
    let mut g = MapGrid::new(8, 8);
    g.cells[0].land = 2;
    let mut d = Draws::new(1);
    assert_eq!(compute_val(&g, &mut d, &makes(), &[64; 128], 0, 0), (0, 0));
    assert!(d.log.is_empty());
    g.cells[0].flags |= 0x100;
    compute_val(&g, &mut d, &makes(), &[64; 128], 0, 0);
    assert_eq!(d.log.len(), 1, "a HALFLAND cell draws");
}

#[test]
fn the_walk_is_column_major() {
    let mut g = MapGrid::new(12, 10);
    let mut d = Draws::new(7);
    compute_all_vals(&mut g, &mut d, &makes());
    assert_eq!(d.log.len(), 120);
    assert!(d.log.iter().all(|&(s, _)| s == VAL_SITE));
    let vals = |order: &[(i32, i32)]| {
        let mut d = Draws::new(7);
        let mut out = vec![0u8; 120];
        for &(x, y) in order {
            out[at(&g, x, y)] = compute_val(&g, &mut d, &makes(), &[120; 128], x, y).0;
        }
        out
    };
    let columns: Vec<_> = (0..12).flat_map(|x| (0..10).map(move |y| (x, y))).collect();
    let rows: Vec<_> = (0..10).flat_map(|y| (0..12).map(move |x| (x, y))).collect();
    let ours: Vec<u8> = g.cells.iter().map(|c| c.val).collect();
    assert_eq!(ours, vals(&columns));
    assert_ne!(ours, vals(&rows), "the order reaches the values");
}

#[test]
fn a_dead_mountain_spot_becomes_mountain() {
    // A plain cell walled by mountain and forest on all four sides.
    let mut g = MapGrid::new(5, 5);
    for (x, y) in [(2, 1), (1, 2), (3, 2), (2, 3)] {
        let i = at(&g, x, y);
        g.cells[i].flags = if x == 2 && y == 1 {
            flag::MOUNTAIN
        } else {
            flag::FOREST
        };
    }
    eliminate_dead_mount_spots(&mut g);
    assert_eq!(g.cells[at(&g, 2, 2)].flags & 0x3c, flag::MOUNTAIN);
    assert!(
        tiles_of(2, 2).all(|(tx, ty)| g.tiles[tile_at(&g, tx, ty)] & 0x4000 != 0),
        "every tile blocked"
    );
    assert_eq!(g.cells[at(&g, 0, 0)].flags, 0, "an open cell is left");
}
