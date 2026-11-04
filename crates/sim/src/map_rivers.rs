//! **Map generation's eighth phase** (`docs/MAPGEN.md` §10, §15 step 8):
//! `Map::make_rivers@00692fb0`, `map.cpp` 5012 through 5118, inside
//! `Terrain::init`.
//!
//! In order, with the checkpoints the original prints between them:
//! - 5012; `Map::make_riverbeds@00694530`: every cell loses 0x2000 and 0x2,
//!   then the river sources are picked and flagged 0x2000 ([`make_riverbeds`]);
//! - 5014; every cell loses 0x1000; with no source the function returns
//!   here and prints nothing more;
//! - the style's `RIVERLIST`, one `Map::make_riverlist@00693820` per
//!   `RIVER` element: the only `game_random` draws and the only 0x1000
//!   (path) and 0x2 (bank) writes. A list that is not empty is refused
//!   ([`Unported::RiverList`]): `PathFinder::astar_river@00686690` is not
//!   ported;
//! - 5076; every cell loses 0x2000 and 0x2;
//! - 5116; `Regions::find_all`;
//! - 5118.
//!
//! No height is read, and no float: on Great Sahara (`SAHARA.xml`'s
//! `<RIVERLIST/>` is empty) the sources are the phase's only write, and the
//! clear at 5076 takes them back, so part 5 moves at 5014 and returns to
//! 5012's sum at 5116 on every file.

use crate::mapgen::{MapGrid, Regions, WCell};
use crate::terrain_init::MOVE;

/// `Map::river_thresh` before any file sets it (`Map::Map@006a06e0`): 30.
/// `Map::init_map_data@0069f250` takes `MAP/RIVER_THRESHOLD`'s value from
/// `default.xml` (8) and then from the style, when either has one.
pub const RIVER_THRESH_UNSET: i32 = 30;

/// The `WData.flags` bits the phase writes.
pub mod bit {
    /// A river's path, `make_riverlist`'s.
    pub const PATH: u16 = 0x1000;
    /// A river source, `make_riverbeds`'.
    pub const SOURCE: u16 = 0x2000;
    /// A river bank, `make_riverlist`'s circle around a path cell.
    pub const BANK: u16 = 0x2;
}

/// An arm this port does not build.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unported {
    /// The style's `RIVERLIST` holds this many `RIVER` elements: each is a
    /// `make_riverlist`, whose sources walk an LFSR seeded from
    /// `World.seed` and whose paths are `astar_river`'s, with draws.
    RiverList(usize),
}

fn at(g: &MapGrid, x: i32, y: i32) -> usize {
    (y * g.xs + x) as usize
}

fn inb(g: &MapGrid, x: i32, y: i32) -> bool {
    x >= 0 && y >= 0 && x < g.xs && y < g.ys
}

/// `WorldData::has_mountain_tcoords@006b3050`: any of the cell's sixteen
/// tiles is mountain (`mask & 3 == 2`).
fn mountain_tiles(g: &MapGrid, x: i32, y: i32) -> bool {
    (0..16).any(|j| {
        let (tx, ty) = (4 * x + (j & 3), 4 * y + (j >> 2));
        g.tiles[(ty * 4 * g.xs + tx) as usize] & 3 == 2
    })
}

/// `WorldData::start_city_rad_wcoord@006b3850`: some start city
/// (`start_city_x`/`start_city_y`, part 2's last two arrays) whose
/// octagonal distance from the cell, times 4, is under
/// `CITY_CENTER_RADIUS − 1`.
fn near_start_city(g: &MapGrid, x: i32, y: i32, city_radius: i32) -> bool {
    let (xs, ys) = (&g.starts[2].list, &g.starts[3].list);
    xs.iter()
        .zip(ys)
        .any(|(&cx, &cy)| 4 * octagonal((cx - x).abs(), (cy - y).abs()) < city_radius - 1)
}

/// The distance `start_city_rad_wcoord` computes inline: the longer leg
/// plus the shorter's square over twice the longer, in unsigned integer
/// arithmetic, and half the sum when the shorter leg reaches 60000.
fn octagonal(a: i32, b: i32) -> i32 {
    let (a, b) = (a as u32, b as u32);
    let (long, short) = if (b as i32) < (a as i32) {
        (a, b)
    } else {
        (b, a)
    };
    let d = if long == 0 {
        0
    } else if short < 60000 {
        short * short / (long * 2) + long
    } else {
        (short + long * 2) >> 1
    };
    d as i32
}

/// `WorldData::get_land@006b4730` with its third argument 1: coast 3,
/// forest 4, mountain (0x10 or 0x40) 5, rock 6 or 7 with oil, oil at sea 7,
/// else the stored `land`.
fn land_class(c: &WCell) -> i32 {
    let f = c.flags;
    if f & 0x4 != 0 {
        3
    } else if f & 0x20 != 0 {
        4
    } else if f & 0x50 != 0 {
        5
    } else if f & 0x8 != 0 {
        6 + i32::from(f & 0x800 != 0)
    } else if c.is_ocean() && f & 0x800 != 0 {
        7
    } else {
        i32::from(c.land)
    }
}

/// `& mask` on every cell's flags: the three whole-grid clears.
fn clear(g: &mut MapGrid, mask: u16) {
    for c in &mut g.cells {
        c.flags &= mask;
    }
}

/// **`Map::make_riverbeds@00694530`**: every cell loses 0x2000 and 0x2;
/// then, y outer and x inner, a forest cell (`flags & 0x21 == 0x20`) with
/// no mountain tile, outside every start city's radius, becomes a source
/// (0x2000, appended to `river_beds`) when its 7×7 square (`move[1..49]`)
/// is on the map, holds no source yet and no cell of land class 2, and at
/// least `thresh` of its eight neighbours have no mountain tile and no
/// rock (0x8). No draw.
///
/// The listing (0x6945a5..0x6945ff) also inlines a land class of the cell
/// and refuses a negative one; it is reached only when `flags & 0x74` is
/// clear, which the forest bit already rules out.
pub fn make_riverbeds(g: &mut MapGrid, thresh: i32, city_radius: i32) -> Vec<(i32, i32)> {
    clear(g, !(bit::SOURCE | bit::BANK));
    let mut beds = Vec::new();
    for y in 0..g.ys {
        for x in 0..g.xs {
            if g.cells[at(g, x, y)].flags & 0x21 != 0x20
                || mountain_tiles(g, x, y)
                || near_start_city(g, x, y, city_radius)
            {
                continue;
            }
            let mut open = 0;
            let mut fit = true;
            for (k, &(dx, dy)) in MOVE.iter().enumerate().skip(1) {
                let (u, v) = (x + dx, y + dy);
                if !inb(g, u, v) {
                    fit = false;
                    break;
                }
                let n = &g.cells[at(g, u, v)];
                if n.flags & bit::SOURCE != 0 || land_class(n) == 2 {
                    fit = false;
                    break;
                }
                if k < 9 && !mountain_tiles(g, u, v) && n.flags & 0x8 == 0 {
                    open += 1;
                }
            }
            if thresh <= if fit { open } else { 0 } {
                beds.push((x, y));
                let i = at(g, x, y);
                g.cells[i].flags |= bit::SOURCE;
            }
        }
    }
    beds
}

/// **`Map::make_rivers@00692fb0`** on `g`: `checkpoint` is called with the
/// grid at each `map.cpp` line the original prints (5012, 5014, and, when
/// a source was found, 5076, 5116 and 5118). `rivers` is the number of
/// `RIVER` elements in the style's `RIVERLIST` (`default.xml`'s when the
/// style has none). Returns `river_beds`, or the arm it refused.
pub fn make_rivers(
    g: &mut MapGrid,
    regions: &mut Regions,
    thresh: i32,
    city_radius: i32,
    rivers: usize,
    mut checkpoint: impl FnMut(i64, &MapGrid),
) -> Result<Vec<(i32, i32)>, Unported> {
    checkpoint(5012, g);
    let beds = make_riverbeds(g, thresh, city_radius);
    checkpoint(5014, g);
    clear(g, !bit::PATH);
    if beds.is_empty() {
        return Ok(beds);
    }
    if rivers > 0 {
        return Err(Unported::RiverList(rivers));
    }
    checkpoint(5076, g);
    clear(g, !(bit::SOURCE | bit::BANK));
    checkpoint(5116, g);
    regions.find_all(g);
    checkpoint(5118, g);
    Ok(beds)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn forest(xs: i32, ys: i32) -> MapGrid {
        let mut g = MapGrid::new(xs, ys);
        for c in &mut g.cells {
            c.flags = 0x20;
            c.region = 1;
        }
        g
    }

    fn flags(g: &mut MapGrid, x: i32, y: i32) -> &mut u16 {
        let i = at(g, x, y);
        &mut g.cells[i].flags
    }

    /// A 7×7 forest has one cell whose square is on the map, its centre.
    #[test]
    fn a_source_s_square_must_be_on_the_map() {
        let mut g = forest(7, 7);
        assert_eq!(make_riverbeds(&mut g, 8, 20), vec![(3, 3)]);
        assert_eq!(
            g.cells
                .iter()
                .filter(|c| c.flags & bit::SOURCE != 0)
                .count(),
            1
        );
    }

    /// A source blocks every later cell whose 7×7 holds it, so sources
    /// stand four apart along a row and none share a square.
    #[test]
    fn a_source_blocks_its_square() {
        let mut g = forest(14, 7);
        assert_eq!(make_riverbeds(&mut g, 8, 20), vec![(3, 3), (7, 3)]);
    }

    /// The threshold counts the eight neighbours without a mountain tile
    /// or rock: one rock neighbour takes 8 to 7.
    #[test]
    fn the_threshold_counts_the_eight() {
        let mut g = forest(7, 7);
        *flags(&mut g, 4, 4) |= 0x8;
        assert_eq!(make_riverbeds(&mut g.clone(), 8, 20), vec![]);
        assert_eq!(make_riverbeds(&mut g, 7, 20), vec![(3, 3)]);
        let mut g = forest(7, 7);
        // A tile of cell (2, 2), a neighbour: row 8, column 9.
        g.tiles[8 * 28 + 9] = 2;
        assert_eq!(make_riverbeds(&mut g, 8, 20), vec![]);
    }

    /// The ring past the eight only refuses: a rock there is no matter,
    /// an ocean cell there is.
    #[test]
    fn the_outer_rings_only_refuse() {
        let mut g = forest(7, 7);
        *flags(&mut g, 0, 0) = 0x8;
        assert_eq!(make_riverbeds(&mut g.clone(), 8, 20), vec![(3, 3)]);
        *flags(&mut g, 0, 0) = 0;
        g.cells[0].land = 2;
        assert_eq!(make_riverbeds(&mut g, 8, 20), vec![]);
    }

    /// Land class 2 is plain water: an oil-flagged sea cell is 7, a
    /// coastal one 3, a `SANDY` cell 1, so none of them refuse.
    #[test]
    fn only_class_2_refuses() {
        for (land, f, refuses) in [
            (2, 0, true),
            (2, 0x800, false),
            (2, 0x4, false),
            (1, 0, false),
        ] {
            let mut g = forest(7, 7);
            g.cells[0].land = land;
            g.cells[0].flags = f;
            assert_eq!(
                make_riverbeds(&mut g, 8, 20).is_empty(),
                refuses,
                "{land} {f:#x}"
            );
        }
    }

    /// Bit 0x1 refuses the cell itself, and the cell needs the forest bit.
    #[test]
    fn a_source_is_forest_without_0x1() {
        let mut g = forest(7, 7);
        *flags(&mut g, 3, 3) |= 0x1;
        assert_eq!(make_riverbeds(&mut g.clone(), 8, 20), vec![]);
        *flags(&mut g, 3, 3) = 0;
        assert_eq!(make_riverbeds(&mut g, 8, 20), vec![]);
    }

    /// A refusal in the square zeroes the count rather than skipping the
    /// cell, so a threshold of 0 makes every eligible forest cell a source,
    /// edge or not.
    #[test]
    fn a_zero_threshold_takes_every_forest_cell() {
        let mut g = forest(3, 3);
        assert_eq!(make_riverbeds(&mut g, 0, 20).len(), 9);
    }

    /// A cell with a mountain tile of its own is never a source.
    #[test]
    fn a_mountain_tile_refuses_the_cell() {
        let mut g = forest(7, 7);
        let t = (4 * 3 * 4 * 7 + 4 * 3 + 3) as usize;
        g.tiles[t] = 0x12;
        assert_eq!(make_riverbeds(&mut g, 8, 20), vec![]);
    }

    /// A start city at distance d refuses the cell when 4·d < radius − 1:
    /// at radius 20, distance 4 refuses and 5 does not.
    #[test]
    fn a_start_city_s_radius_refuses() {
        for (cx, refused) in [(7, true), (8, false)] {
            let mut g = forest(15, 7);
            g.starts[2].list.push(cx);
            g.starts[3].list.push(3);
            let beds = make_riverbeds(&mut g, 8, 20);
            assert_eq!(beds.first() == Some(&(3, 3)), !refused, "city at {cx}");
        }
        assert_eq!(octagonal(4, 0), 4);
        assert_eq!(octagonal(3, 4), 3 * 3 / 8 + 4);
        assert_eq!(octagonal(4, 3), 3 * 3 / 8 + 4);
    }

    /// The first clear takes 0x2000 and 0x2 and nothing else, before the
    /// scan, so an old source blocks nothing.
    #[test]
    fn the_first_clear_runs_before_the_scan() {
        let mut g = forest(7, 7);
        *flags(&mut g, 0, 0) |= bit::SOURCE | bit::BANK | bit::PATH;
        assert_eq!(make_riverbeds(&mut g, 8, 20), vec![(3, 3)]);
        assert_eq!(g.cells[0].flags, 0x20 | bit::PATH);
    }

    /// The whole phase on a forest with sources and no `RIVER`: the five
    /// checkpoints, the paths cleared at 5014's tail, and the sources gone
    /// again by 5116.
    #[test]
    fn the_phase_takes_its_sources_back() {
        let mut g = forest(7, 7);
        *flags(&mut g, 0, 0) |= bit::PATH;
        let before = g.cell_part();
        let mut seen = Vec::new();
        let beds = make_rivers(&mut g, &mut Regions::default(), 8, 20, 0, |l, g| {
            seen.push((l, g.cell_part()))
        })
        .expect("no river list");
        assert_eq!(beds, vec![(3, 3)]);
        let lines: Vec<i64> = seen.iter().map(|s| s.0).collect();
        assert_eq!(lines, [5012, 5014, 5076, 5116, 5118]);
        assert_eq!(seen[0].1, before);
        assert_ne!(seen[1].1, before);
        assert!(g.cells.iter().all(|c| c.flags == 0x20));
        assert_eq!(seen[3].1, seen[4].1);
    }

    /// No source: the paths are still cleared, and nothing past 5014 is
    /// printed or refused.
    #[test]
    fn no_source_ends_at_5014() {
        let mut g = forest(5, 5);
        *flags(&mut g, 2, 2) |= bit::PATH;
        let mut lines = Vec::new();
        let beds = make_rivers(&mut g, &mut Regions::default(), 8, 20, 1, |l, _| {
            lines.push(l)
        });
        assert_eq!(beds, Ok(vec![]));
        assert_eq!(lines, [5012, 5014]);
        assert_eq!(*flags(&mut g, 2, 2), 0x20);
    }

    /// A river list with a source to start from is refused by name.
    #[test]
    fn a_river_list_is_refused() {
        let mut g = forest(7, 7);
        assert_eq!(
            make_rivers(&mut g, &mut Regions::default(), 8, 20, 1, |_, _| {}),
            Err(Unported::RiverList(1))
        );
    }
}
