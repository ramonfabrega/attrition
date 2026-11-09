use super::*;
use crate::terrain_init::coord_flags;

const COMBO: Combo = Combo {
    coast_base: 3,
    num_tiles: [Some(4), Some(1), Some(1), None],
};

/// A `xs × ys` grid of heights all at `h` (as `Single` bits).
fn flat(xs: i32, ys: i32, h: Single) -> Vec<Single> {
    vec![h; ((4 * xs + 1) * (4 * ys + 1)) as usize]
}

const DRY: Single = Single(0x4120_0000); // 10
const WET: Single = Single(0xc220_0000); // −40

#[test]
fn the_combo_table_reads_its_lands() {
    let c = COMBO;
    assert_eq!(is_combo_acceptable(&c, 0, 1, 0, 2), Ok(1));
    assert_eq!(is_combo_acceptable(&c, 0, 2, 0, 1), Ok(2));
    assert_eq!(is_combo_acceptable(&c, 0, 0, 0, 2), Ok(0));
    assert_eq!(is_combo_acceptable(&c, 0, 3, 1, 0), Ok(1));
    assert_eq!(is_combo_acceptable(&c, 0, 2, 1, 0), Ok(0));
    assert_eq!(is_combo_acceptable(&c, 1, 0, 0, 3), Ok(2));
    assert_eq!(is_combo_acceptable(&c, 1, 0, 2, 0), Ok(1));
    // A water centre: a land neighbour must sit at num_tiles[0] − 1.
    assert_eq!(is_combo_acceptable(&c, 2, 0, 0, 3), Ok(2));
    assert_eq!(is_combo_acceptable(&c, 2, 0, 0, 2), Ok(0));
    assert_eq!(is_combo_acceptable(&c, 2, 1, 0, 3), Ok(0));
    // Past the special cases, the lower land's last sub-tile decides.
    assert_eq!(is_combo_acceptable(&c, 0, 3, 3, 0), Ok(1));
    assert_eq!(is_combo_acceptable(&c, 3, 0, 1, 0), Ok(2));
    assert_eq!(is_combo_acceptable(&c, 3, 1, 1, 0), Ok(0));
    // A `num_tiles` entry not loaded is named, not guessed.
    assert_eq!(is_combo_acceptable(&c, 3, 0, 0, 1), Ok(0));
    assert_eq!(is_combo_acceptable(&c, 0, 1, 3, 0), Ok(0));
    assert_eq!(is_combo_acceptable(&c, 1, 0, 3, 0), Ok(1));
    let unread = Combo {
        num_tiles: [Some(4), None, None, None],
        ..c
    };
    assert_eq!(is_combo_acceptable(&unread, 1, 0, 3, 0), Err(1));
}

/// **`TerrainOut::check_for_diagonal@008595f0`'s route to
/// `WorldData::is_valid@00471170` needs a P1 pair** (item 3886,
/// `docs/MAPGEN.md` §20): a cell enters the arm only with each corner's
/// flanks F one up from it (C → F 1) and the corner's diagonal D one
/// below (D → C 1), and the arm falls through to `is_valid` only when
/// the corner's two flanks differ. Over every shipped land and sub, at
/// `COAST_BASE` 3 and 0, two distinct one-up flanks always put D beside
/// a flank it is mutually unacceptable with, and a `SANDY` centre has
/// one one-up land only, `OCEAN`.
#[test]
fn the_diagonal_arm_reaches_is_valid_only_past_a_p1_pair() {
    let mut cells: Vec<(i32, i32)> = (0..4).map(|s| (0, s)).collect();
    cells.extend([(1, 0), (2, 0)]);
    for cb in [3, 0] {
        let c = Combo {
            coast_base: cb,
            ..COMBO
        };
        let k = |a: (i32, i32), b: (i32, i32)| is_combo_acceptable(&c, a.0, a.1, b.0, b.1).unwrap();
        let ups = |x: (i32, i32)| cells.iter().copied().filter(move |&f| k(x, f) == 1);
        assert_eq!(
            ups((1, 0)).collect::<Vec<_>>(),
            vec![(2, 0)],
            "SANDY's one-up lands"
        );
        let mut distinct = 0;
        for &centre in &cells {
            for &d in cells.iter().filter(|&&d| k(d, centre) == 1) {
                for f1 in ups(centre) {
                    for f2 in ups(centre).filter(|&f2| f2 != f1) {
                        distinct += 1;
                        let p1 = [f1, f2].iter().any(|&f| k(d, f) == 0 && k(f, d) == 0);
                        assert!(p1, "cb {cb}: C {centre:?} D {d:?} flanks {f1:?} {f2:?}");
                    }
                }
            }
        }
        assert_eq!(distinct, if cb == 3 { 6 } else { 4 });
    }
}

/// A fully wet gated cell turns `OCEAN` (its sixteenth tile), then
/// `spot_update_land(1, 0)` makes it `SANDY` and `verify_land` sets each
/// `BASELAND` neighbour off `COAST_BASE` to it.
#[test]
fn a_wet_cell_turns_sandy_and_its_shore_to_coast_base() {
    let mut g = MapGrid::new(3, 3);
    for c in g.cells.iter_mut() {
        c.land_sub = 1;
        c.region = 1;
    }
    g.cells[4].land = 2;
    g.cells[4].region = 0x41;
    g.cells[3].flags = 0x4;
    let mut coord = coord_flags(&g);
    let mut z = flat(3, 3, DRY);
    for v in 4..=8 {
        for u in 4..=8 {
            z[(v * 13 + u) as usize] = WET;
        }
    }
    let mut p = Pass::new(&mut g, &mut coord, &z, COMBO);
    p.sweeps().expect("nothing unported");
    // The dry shore cell (0, 1) spreads 0x4 east and queues (1, 1),
    // which its sixteenth tile had left not HALFLAND: the second visit
    // is a first touch again and lists it again, as `SANDY` at 0.
    assert_eq!(p.lists.locs, vec![(1, 1), (1, 1)]);
    assert_eq!(
        (p.lists.types.clone(), p.lists.subtypes.clone()),
        (vec![2, 1], vec![1, 0])
    );
    let c = g.cells[4];
    assert_eq!((c.land, c.land_sub, c.flags & 0x100), (1, 0, 0));
    assert_eq!((c.region, c.region2), (0x41, 0));
    for (i, n) in g.cells.iter().enumerate() {
        if i != 4 {
            assert_eq!((n.land, n.land_sub), (0, 3), "cell {i}");
        }
    }
}

/// A gated cell with no water sub-tile: `OCEAN` becomes `BASELAND` at
/// `COAST_BASE`, flags lose 0x3c, and 0x4 goes to the first orthogonal
/// wet neighbour, which is queued and visited.
#[test]
fn a_dry_cell_spreads_0x4_to_its_first_wet_neighbour() {
    let mut g = MapGrid::new(3, 1);
    g.cells[0].flags = 0x4 | 0x8 | 0x20;
    g.cells[1].land = 2;
    g.cells[1].region = 0x41;
    g.cells[2].land = 2;
    g.cells[2].region = 0x41;
    let mut coord = coord_flags(&g);
    let z = flat(3, 1, DRY);
    let mut p = Pass::new(&mut g, &mut coord, &z, COMBO);
    p.sweeps().expect("nothing unported");
    assert_eq!(g.cells[0].flags, 0);
    assert_eq!(coord[0] & coord::GATE_HALF, 0);
    // Both lake cells are shallow, so gated: each dry visit makes it
    // `BASELAND` at 3 and hands 0x4 on. `verify_land` then grades the
    // subs one step at a time from (1, 0)'s 3 against (0, 0)'s 0.
    assert!(g.cells.iter().all(|c| c.land == 0 && c.flags & 0x4 == 0));
    let subs: Vec<u8> = g.cells.iter().map(|c| c.land_sub).collect();
    assert_eq!(subs, vec![1, 2, 3]);
}

/// `clear_halfland_wcoord`: a cell that was not HALFLAND loses its
/// `region2`; one that was keeps it until the scan rewrites it.
#[test]
fn a_cell_not_halfland_loses_its_region2() {
    let mut g = MapGrid::new(1, 1);
    g.cells[0].flags = 0x4;
    g.cells[0].region2 = 7;
    let mut coord = coord_flags(&g);
    let z = flat(1, 1, DRY);
    let mut p = Pass::new(&mut g, &mut coord, &z, COMBO);
    p.clear_halfland_wcoord(0, 0);
    assert_eq!(g.cells[0].region2, 0);
    g.cells[0].flags |= 0x100;
    g.cells[0].region2 = 7;
    let mut p = Pass::new(&mut g, &mut coord, &z, COMBO);
    p.clear_halfland_wcoord(0, 0);
    assert_eq!((g.cells[0].region2, g.cells[0].flags & 0x100), (7, 0));
}

/// `verify_land`'s depth is a static the refusal past 500 never
/// decrements: once it passes, every later call returns at once.
#[test]
fn verify_land_past_500_stays_refused() {
    let mut g = MapGrid::new(2, 1);
    g.cells[1].land_sub = 3;
    let mut coord = vec![0; 2];
    let z = flat(2, 1, DRY);
    let mut p = Pass::new(&mut g, &mut coord, &z, COMBO).with_depth(500);
    let (mut l, mut s) = (0, 0);
    p.verify_land(0, 0, &mut l, &mut s);
    assert_eq!(p.depth, 501);
    assert_eq!(g.cells[1].land_sub, 3, "a refused call writes nothing");
    let mut p = Pass::new(&mut g, &mut coord, &z, COMBO);
    let (mut l, mut s) = (0, 0);
    p.verify_land(0, 0, &mut l, &mut s);
    assert_eq!(p.depth, 0);
    assert_eq!(g.cells[1].land_sub, 2, "one step toward the centre's 0");
}

/// A goody box on a cell turning wet moves to the first clear
/// `BASELAND` cell of `move[9..25]` off the map's edge; the old cell
/// loses 0x8000 and its `down`.
#[test]
fn a_goody_moves_off_a_cell_turning_wet() {
    let mut g = MapGrid::new(7, 7);
    let at = |x: i32, y: i32| (y * 7 + x) as usize;
    let c = &mut g.cells[at(3, 3)];
    c.flags = 0x8000;
    c.down = -3;
    c.down_who = 5;
    let mut coord = vec![0; 49];
    let z = flat(7, 7, DRY);
    let mut p = Pass::new(&mut g, &mut coord, &z, COMBO);
    p.move_ocean_goody(3, 3);
    let old = g.cells[at(3, 3)];
    assert_eq!((old.flags, old.down, old.down_who), (0, -1, -1));
    // move[9] is (−1, −2) from the cell.
    let new = g.cells[at(2, 1)];
    assert_eq!((new.flags, new.down, new.down_who), (0x8000, -3, 5));
}

/// Oil on a cell turning wet: the cell loses 0x800, 0x8 and its good
/// (`down_who` −1); the first water cell of `move[9..49]` clear of
/// 0x8 takes it.
#[test]
fn oil_moves_to_water_and_its_old_cell_takes_down_who_minus_one() {
    let mut g = MapGrid::new(7, 7);
    for c in g.cells.iter_mut() {
        c.land = 2;
        c.down = -1;
    }
    let at = |x: i32, y: i32| (y * 7 + x) as usize;
    g.cells[at(3, 3)].flags = 0x800 | 0x8;
    g.init_good(OIL_GOOD, 3, 3);
    let mut coord = vec![0; 49];
    let z = flat(7, 7, DRY);
    let mut p = Pass::new(&mut g, &mut coord, &z, COMBO);
    p.move_ocean_goody(3, 3);
    let old = g.cells[at(3, 3)];
    assert_eq!((old.flags, old.down_who), (0, -1));
    assert_eq!(g.cells[at(2, 1)].flags, 0x808);
    assert_eq!(g.goods, vec![Some((OIL_GOOD, 2, 1))]);
    create_doobers_flags(&mut g);
    assert_eq!(g.cells[at(2, 1)].flags, 0x800, "738 takes the rock back");
}

/// A 5×5 grid in region 1 whose centre is gated and has one wet
/// sub-tile, its first.
fn centre_wet() -> (MapGrid, Vec<u16>, Vec<Single>) {
    let mut g = MapGrid::new(5, 5);
    for c in g.cells.iter_mut() {
        c.region = 1;
    }
    let mut coord = vec![0; 25];
    coord[12] = coord::GATE_HALF;
    let mut z = flat(5, 5, DRY);
    for v in 8..=9 {
        for u in 8..=9 {
            z[v * 21 + u] = WET;
        }
    }
    (g, coord, z)
}

/// The scan stops at the first neighbour not HALFLAND of another
/// region: `move[1]`'s 5 wins over `move[2]`'s 6.
#[test]
fn the_scan_stops_at_the_first_other_region() {
    let (mut g, mut coord, z) = centre_wet();
    g.cells[6].region = 5; // (1, 1), move[1]
    g.cells[7].region = 6; // (2, 1), move[2]
    Pass::new(&mut g, &mut coord, &z, COMBO).wcoord(2, 2);
    assert_eq!((g.cells[12].region, g.cells[12].region2), (1, 5));
}

/// With no such neighbour, the last HALFLAND neighbour whose `region2`
/// is not the cell's region wins: `move[2]`'s 8 over `move[1]`'s 7.
#[test]
fn the_last_halfland_region2_wins() {
    let (mut g, mut coord, z) = centre_wet();
    for (i, r2) in [(6, 7), (7, 8)] {
        g.cells[i].flags = 0x100;
        g.cells[i].region2 = r2;
    }
    Pass::new(&mut g, &mut coord, &z, COMBO).wcoord(2, 2);
    assert_eq!((g.cells[12].region, g.cells[12].region2), (1, 8));
}

/// The dry arm hands 0x4 to the first orthogonal wet neighbour only:
/// `orthog[2]`, east, before `orthog[4]`, west.
#[test]
fn the_dry_arm_spreads_0x4_once() {
    let mut g = MapGrid::new(3, 1);
    g.cells[0].land = 2;
    g.cells[2].land = 2;
    let mut coord = vec![0, coord::GATE_HALF, 0];
    let z = flat(3, 1, DRY);
    let mut p = Pass::new(&mut g, &mut coord, &z, COMBO);
    p.wcoord(1, 0);
    assert_eq!(p.queue, vec![2]);
    assert_eq!((g.cells[0].flags & 0x4, g.cells[2].flags & 0x4), (0, 0x4));
}

/// Oil searches `move[9..49]`: with water only in the third ring, the
/// third ring's first cell, `move[25]` (−3, −3), takes it.
#[test]
fn oil_reaches_the_third_ring() {
    let mut g = MapGrid::new(9, 9);
    for c in g.cells.iter_mut() {
        c.down = -1;
    }
    let at = |x: i32, y: i32| (y * 9 + x) as usize;
    for &(dx, dy) in &MOVE[25..49] {
        g.cells[at(4 + dx, 4 + dy)].land = 2;
    }
    g.cells[at(4, 4)].land = 2;
    g.cells[at(4, 4)].flags = 0x800 | 0x8;
    g.init_good(OIL_GOOD, 4, 4);
    let mut coord = vec![0; 81];
    let z = flat(9, 9, DRY);
    Pass::new(&mut g, &mut coord, &z, COMBO).move_ocean_goody(4, 4);
    assert_eq!(g.cells[at(1, 1)].flags, 0x808);
    assert_eq!(g.goods, vec![Some((OIL_GOOD, 1, 1))]);
}

/// A good other than oil is refused by name.
#[test]
fn a_good_on_a_cell_turning_wet_is_refused() {
    let mut g = MapGrid::new(3, 3);
    let n = g.init_good(2, 1, 1) as i16;
    g.cells[4].down = -2;
    g.cells[4].down_who = n;
    let mut coord = vec![0; 9];
    let z = flat(3, 3, DRY);
    let mut p = Pass::new(&mut g, &mut coord, &z, COMBO);
    p.move_ocean_goody(1, 1);
    assert_eq!(p.refusals(), Err(Unported::Goody(vec![(1, 1)])));
}

/// The beach: a `BASELAND` cell beside `OCEAN` turns `SANDY` at 0 and
/// clears `CoordInfo` 0x2000; its `BASELAND` neighbours off
/// `COAST_BASE` follow to it.
#[test]
fn a_beach_cell_turns_sandy() {
    let mut g = MapGrid::new(3, 1);
    g.cells[2].land = 2;
    g.cells[0].land_sub = 1;
    g.cells[1].land_sub = 2;
    let mut coord = vec![coord::BEACH; 3];
    let z = flat(3, 1, DRY);
    let entered = Pass::new(&mut g, &mut coord, &z, COMBO)
        .convert_beach_to_sandy()
        .expect("nothing unported");
    assert_eq!(entered, 1);
    assert_eq!((g.cells[1].land, g.cells[1].land_sub), (1, 0));
    assert_eq!(coord[1], 0);
    assert_eq!((g.cells[0].land, g.cells[0].land_sub), (0, 3));
}
