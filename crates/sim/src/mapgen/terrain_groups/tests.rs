use super::*;

/// `init_templates`' height arm in native single precision, the test
/// oracle for [`cliff_cells`]: `byte · (1/255) · 480`, each cell's 6 × 6
/// window from pixel `4c − 1`, 0 off the texture, max from 0 and min
/// from 480, covered when the span exceeds 192.
#[allow(clippy::float_arithmetic)] // the oracle: the host's own `f32`
fn heights_oracle(width: i32, px: &[[u8; 2]]) -> Vec<(i32, i32)> {
    let n = width >> 2;
    let h = |r: i32, c: i32| -> f32 {
        if r < 0 || c < 0 || r >= width || c >= width {
            return 0.0;
        }
        f32::from(px[(r * width + c) as usize][0]) * f32::from_bits(0x3b80_8081) * 480.0
    };
    let mut out = Vec::new();
    for cy in 0..n {
        for cx in 0..n {
            let (mut hi, mut lo) = (0.0f32, 480.0f32);
            for r in 4 * cy - 1..=4 * cy + 4 {
                for c in 4 * cx - 1..=4 * cx + 4 {
                    hi = hi.max(h(r, c));
                    lo = lo.min(h(r, c));
                }
            }
            if hi - lo > 192.0 {
                out.push((cx, cy));
            }
        }
    }
    out
}

/// **The cliff mask's height arm** (`docs/MAPGEN.md` §18.3, item 3875):
/// on textures with no zero alpha — the arm only six of
/// `Data/Cliffs.xml`'s 32-pixel templates take, and none run1493's
/// stream decides — [`cliff_cells`] is the oracle's set, over a step
/// either side of the real-number boundary (Δ = 102 is 192 exactly) at
/// every offset, a step one pixel past the window, and seeded noise.
#[test]
fn the_cliff_mask_s_height_arm_is_single_precision() {
    let flat = |w: i32, v: u8| vec![[v, 0xff]; (w * w) as usize];
    let mut cases = Vec::new();
    for lo in 0..=152u8 {
        for d in [101u8, 102, 103] {
            let mut px = flat(16, lo);
            px[5 * 16 + 6] = [lo + d, 0xff];
            cases.push(px);
        }
    }
    // A spike at pixel (4, 3) is inside the windows of cells 0 and 1
    // both ways (pixels −1..=4 and 3..=8), one at (8, 8) inside cells
    // 1..=2's.
    for at in [(4, 3), (8, 8), (0, 0), (15, 15)] {
        let mut px = flat(16, 0);
        px[at.1 * 16 + at.0] = [200, 0xff];
        cases.push(px);
    }
    let mut rng = Rng::new(3875);
    for _ in 0..200 {
        cases.push(
            (0..256)
                .map(|_| [(rng.roll() & 0xff) as u8, 0xff])
                .collect(),
        );
    }
    for (k, px) in cases.iter().enumerate() {
        assert_eq!(cliff_cells(16, px), heights_oracle(16, px), "case {k}");
    }
    assert_eq!(
        cliff_cells(16, &cases[cases.len() - 204]),
        vec![(0, 0), (1, 0), (0, 1), (1, 1)],
        "a spike at (4, 3) reaches four windows"
    );
}

/// `randomize_mountains` on run1000's two words (`docs/MAPGEN.md` §7):
/// the small ring holds one range and draws nothing; 1131 % 8 seats the
/// medium ring on range 11 and 13530 % 7 the large on range 0, and
/// `get_range` then walks each ring from its head, wrapping.
#[test]
fn the_rings_seat_on_the_traced_words() {
    let mut r = Rings::stock();
    let mut d = Draws::new(0x8c60_9701);
    r.randomize(&mut d);
    assert_eq!(
        d.log,
        [(site::MTN_MED, 0x8c60_9701), (site::MTN_LARGE, 0x989a_046c)]
    );
    assert_eq!((r.get(1), r.get(1)), (15, 15));
    assert_eq!((r.get(2), r.get(2), r.get(2), r.get(2)), (11, 10, 9, 8));
    assert_eq!(r.get(2), 7);
    assert_eq!(r.get(2), 14, "the medium ring wraps to its head");
    assert_eq!((r.get(3), r.get(3)), (0, 6), "the large ring wraps at once");
    assert_eq!(r.get(4), 0);
}

/// The sea-oil disc's reach is the group's `rock_space` past 3: a wet
/// oiled cell on ring 4 refuses at `rock_space` 5 and not at 3, and a
/// dry cell refuses only inside `circle_radius[2]`. Great Lakes'
/// corpus never decides a start by ring 4 or 5 (`docs/MAPGEN.md` §7),
/// so this pins the listing's reading, not a diff.
#[test]
fn the_sea_oil_disc_reaches_rock_space() {
    let mut g = MapGrid::new(20, 20);
    for c in &mut g.cells {
        c.land = 2;
    }
    let mut d = Draws::new(1);
    let t = circle::table();
    let ring = |k: usize| {
        let i = t.radius[k - 1] as usize;
        (10 + i32::from(t.x[i]), 10 + i32::from(t.y[i]))
    };
    let (ox, oy) = ring(4);
    g.cells[(oy * 20 + ox) as usize].flags |= flag::OIL;
    let pass = |g: &mut MapGrid, d: &mut Draws, rock: i32| {
        Pass {
            g,
            d,
            templates: &[],
            cliffs: Default::default(),
            rings: Rings::stock(),
            help: Help::default(),
            marks: Vec::new(),
            tiles: Vec::new(),
            base_tiles: 4,
        }
        .sea_oil_room(rock, 10, 10)
    };
    assert!(pass(&mut g, &mut d, 3), "ring 4 is past circle_radius[3]");
    assert!(
        !pass(&mut g, &mut d, 5),
        "ring 4 is inside circle_radius[5]"
    );
    g.cells[(oy * 20 + ox) as usize].flags &= !flag::OIL;
    let (lx, ly) = ring(3);
    g.cells[(ly * 20 + lx) as usize].land = 0;
    assert!(pass(&mut g, &mut d, 5), "a dry cell past circle_radius[2]");
    let (lx, ly) = ring(2);
    g.cells[(ly * 20 + lx) as usize].land = 0;
    assert!(
        !pass(&mut g, &mut d, 5),
        "a dry cell inside circle_radius[2]"
    );
}

/// `check_player_forest`'s three patch sizes, as the listing reads
/// (`docs/MAPGEN.md` §7): on dry, treeless land the first cell from
/// `circle_radius[2]` takes its first two growing neighbours, north
/// then east; on a lone dry cell (an islet in the disc) only the patch
/// of one lands, and on a dry pair the patch of two; a start-city cell
/// never grows; a disc already
/// holding three forest cells is left alone, two are not enough. The
/// East Indies fuzz lobby's 7861 is the diff; this pins the arms it
/// does not reach.
#[test]
fn check_player_forest_patches_three_then_two_then_one() {
    let t = circle::table();
    let first = |o: i32| {
        let i = t.radius[2] as usize;
        (o + i32::from(t.x[i]), o + i32::from(t.y[i]))
    };
    let grid = |land: i8| {
        let mut g = MapGrid::new(20, 20);
        for c in &mut g.cells {
            c.land = land;
        }
        g.starts[0].list = vec![10];
        g.starts[1].list = vec![10];
        g
    };
    let forest = |g: &MapGrid| -> Vec<(i32, i32)> {
        (0..g.cells.len() as i32)
            .filter(|&i| g.cells[i as usize].flags & flag::FOREST != 0)
            .map(|i| (i % 20, i / 20))
            .collect()
    };
    let (x, y) = first(10);
    let mut g = grid(0);
    check_player_forest(&mut g, 20);
    let mut want = vec![(x, y), (x, y - 1), (x + 1, y)];
    want.sort_by_key(|&(x, y)| (y, x));
    assert_eq!(
        forest(&g),
        want,
        "the patch of three: the cell, north, east"
    );
    let mut g = grid(0);
    g.city[(y * 20 + x) as usize] = true;
    check_player_forest(&mut g, 20);
    assert!(
        !forest(&g).contains(&(x, y)) && forest(&g).len() == 3,
        "a start-city cell never grows"
    );
    let mut g = grid(2);
    g.cells[(y * 20 + x) as usize].land = 0;
    check_player_forest(&mut g, 20);
    assert_eq!(
        forest(&g),
        vec![(x, y)],
        "a lone dry cell: the patch of one"
    );
    let mut g = grid(2);
    g.cells[(y * 20 + x) as usize].land = 0;
    g.cells[((y - 1) * 20 + x) as usize].land = 0;
    check_player_forest(&mut g, 20);
    assert_eq!(
        forest(&g),
        vec![(x, y - 1), (x, y)],
        "a dry pair: the patch of two"
    );
    let near = |k: usize| {
        let i = t.radius[0] as usize + k;
        (10 + i32::from(t.x[i]), 10 + i32::from(t.y[i]))
    };
    let mut g = grid(0);
    for k in 0..2 {
        let (u, v) = near(k);
        g.cells[(v * 20 + u) as usize].flags |= flag::FOREST;
    }
    check_player_forest(&mut g, 20);
    assert_eq!(
        forest(&g).len(),
        5,
        "two forest cells in the disc are not enough"
    );
    let mut g = grid(0);
    for k in 0..3 {
        let (u, v) = near(k);
        g.cells[(v * 20 + u) as usize].flags |= flag::FOREST;
    }
    check_player_forest(&mut g, 20);
    assert_eq!(
        forest(&g).len(),
        3,
        "three forest cells in the disc: no patch"
    );
}

/// `place_region_group`'s walk is a maximal 16-bit Galois LFSR — the
/// listing's 0x16800 is applied before the shift, so the taps are
/// [`REGION_TAPS`]: from any nonzero state it visits every state below 2^16 once
/// before returning.
#[test]
fn the_region_walk_visits_every_state() {
    let (s0, mut s, mut n) = (1i32, 1i32, 0i32);
    loop {
        s = if s & 1 != 0 {
            (s >> 1) ^ REGION_TAPS
        } else {
            s >> 1
        };
        n += 1;
        if s == s0 {
            break;
        }
        assert!(s != 0 && s < 1 << 16 && n < 1 << 16);
    }
    assert_eq!(n, (1 << 16) - 1);
}
