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
//!   `RIVER` element ([`make_riverlist`]): the only `game_random` draws —
//!   the river count's, and `PathFinder::astar_river@00686690`'s one per
//!   dry cell it prices ([`astar_river`]) — and the only 0x1000 (path) and
//!   0x2 (bank) writes;
//! - 5076; every cell loses 0x2000 and 0x2;
//! - 5116; `Regions::find_all`;
//! - 5118.
//!
//! No height is read, and no float: the river's spline
//! (`RiverOut::add_spline_point`, `RiversOut::start_river`) is the
//! renderer's, takes integer cell coordinates, and writes nothing a
//! checksum part reads; it is not built.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};

use crate::ai_place::circle;
use crate::mapgen::{MapGrid, Regions, WCell};
use crate::terrain_groups::Draws;
use crate::terrain_init::{MOVE, ORTHOG};

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

/// The phase's two draw sites, as return addresses.
pub mod site {
    /// `make_riverlist`'s `r % (spread + 1)`, the river count.
    pub const SPREAD: u32 = 0x0069_3820 + 0x76;
    /// `astar_river`'s `r % 20 + 10`, a dry cell's price.
    pub const PRICE: u32 = 0x0068_6690 + 0xa28;
}

/// One `RIVER` element of a `RIVERLIST`, each attribute as
/// `XMLElement::get_attrib_num` reads it: the leading integer, −1 when
/// absent. `make_rivers` scales `river_length`, `river_min_length` and
/// `river_valley` by the map's width; the rest are used as written.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct River {
    pub start_range: i32,
    pub end_range: i32,
    pub river_length: i32,
    pub river_min_length: i32,
    pub river_valley: i32,
    pub river_wide: i32,
    pub river_med: i32,
    pub river_twist: i32,
    pub river_flags: i32,
}

/// What `make_rivers` reads besides the grid.
#[derive(Clone, Copy, Debug)]
pub struct Phase<'a> {
    /// `Map::river_thresh`.
    pub thresh: i32,
    /// `CITY_CENTER_RADIUS`.
    pub city_radius: i32,
    /// The style's `RIVERLIST`, or `default.xml`'s when the style has none.
    pub rivers: &'a [River],
    /// `rules.xml`'s Standard map size, what `SCALE` measures against.
    pub standard: i32,
    /// `World.seed` (+0x7c), the source walk's seed.
    pub seed: u32,
    /// `Map::stash_nations_here`, the lobby's nations.
    pub nations: i32,
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

/// `make_rivers`' and `make_riverlist`'s `SCALE`: `max(1, (standard/2 +
/// xs·v) / standard)`, 0 for a value under 1 (`Map::scale_number@006a0320`'s
/// arithmetic, inlined).
fn scale(v: i32, xs: i32, standard: i32) -> i32 {
    if v < 1 {
        0
    } else {
        ((standard / 2 + xs * v) / standard).max(1)
    }
}

/// `make_riverlist`'s and `astar_river`'s water: not HALFLAND (0x100), and
/// `land` `SANDY` (1) or `OCEAN` (2).
fn wet(c: &WCell) -> bool {
    c.flags & 0x100 == 0 && (c.land == 1 || c.land == 2)
}

fn border(g: &MapGrid, x: i32, y: i32) -> bool {
    x == 0 || y == 0 || x == g.xs - 1 || y == g.ys - 1
}

/// The start city nearest `(x, y)` by [`octagonal`], the first of equals;
/// `None` when no distance is under 99999.
fn nearest_start(g: &MapGrid, x: i32, y: i32, nations: i32) -> Option<usize> {
    let (xs, ys) = (&g.starts[2].list, &g.starts[3].list);
    let mut best = (99999, None);
    for (i, (&cx, &cy)) in xs.iter().zip(ys).enumerate().take(nations.max(0) as usize) {
        let d = octagonal((cx - x).abs(), (cy - y).abs());
        if d < best.0 {
            best = (d, Some(i));
        }
    }
    best.1
}

/// A node of `astar_river`'s search, in cells.
#[derive(Clone, Copy)]
struct Node {
    x: i32,
    y: i32,
    length: i32,
    value: i32,
    parent: Option<usize>,
}

/// **`PathFinder::astar_river@00686690`**: the river's path from the
/// source `(sx, sy)`, source first, or empty for the original's 0. A
/// greedy best-first search over the four orthogonal neighbours
/// (`orthog[1..5]`), the open list ordered on `value` with equals newest
/// first (`docs/PATHFINDER.md` §2.1); a cell enters the open list once.
///
/// - The source is refused outright when an orthogonal neighbour is off
///   the map, a path (0x1000), water ([`wet`]) or on the map's border.
/// - A popped node that is water, a path or on the border is the end: the
///   path is returned when `max_len < 1` or its `length` exceeds `max_len`;
///   otherwise the node is dropped, not closed, and the search goes on.
/// - Each neighbour of the node expanded, `k` = 1..4, is refused when off
///   the map, open, closed, one of the node's ancestors but the source, a
///   bank (0x2), next to such an ancestor, or — when it is itself dry,
///   inland and no path — when two of its other neighbours are paths,
///   water or border cells.
/// - A neighbour that is water, a path or on the border ends the
///   expansion: when it lies at least `max_len` from the source
///   ([`octagonal`]) and no other neighbour of the node is a path, it
///   alone enters the open list, at value 0; otherwise nothing does.
/// - A dry neighbour inside a start city's radius, `blocked` or rock (0x8)
///   is refused; any other is priced with one draw, `r % 20 + 10` (none,
///   price 0, when its land class is negative), times `twist + 1` on a
///   straight step or `101 − twist` on a turn (ten times more when `twist`
///   is 0), plus a pull toward `max_len` on the node's `length`.
pub fn astar_river(
    g: &MapGrid,
    (sx, sy): (i32, i32),
    twist: i32,
    max_len: i32,
    city_radius: i32,
    draws: &mut Draws,
) -> Vec<(i32, i32)> {
    let cell = |x: i32, y: i32| &g.cells[at(g, x, y)];
    let end = |x: i32, y: i32| {
        let c = cell(x, y);
        wet(c) || c.flags & bit::PATH != 0 || border(g, x, y)
    };
    for &(dx, dy) in &ORTHOG[1..5] {
        let (u, v) = (sx + dx, sy + dy);
        if !inb(g, u, v) || end(u, v) {
            return Vec::new();
        }
    }
    let mut nodes = vec![Node {
        x: sx,
        y: sy,
        length: 0,
        value: 0,
        parent: None,
    }];
    let mut seq = 0u64;
    let mut open: BTreeMap<(i32, Reverse<u64>), usize> = BTreeMap::new();
    let mut live: BTreeSet<usize> = BTreeSet::new();
    let mut closed: BTreeSet<usize> = BTreeSet::new();
    let mut push = |open: &mut BTreeMap<(i32, Reverse<u64>), usize>,
                    live: &mut BTreeSet<usize>,
                    nodes: &[Node],
                    id: usize| {
        let n = nodes[id];
        open.insert((n.value, Reverse(seq)), id);
        live.insert(at(g, n.x, n.y));
        seq += 1;
    };
    push(&mut open, &mut live, &nodes, 0);
    // The node's ancestors that `astar_river` compares: itself and up the
    // chain, all but the source.
    let in_chain = |nodes: &[Node], from: usize, x: i32, y: i32| {
        let mut p = from;
        while let Some(up) = nodes[p].parent {
            if (nodes[p].x, nodes[p].y) == (x, y) {
                return true;
            }
            p = up;
        }
        false
    };
    while let Some((_, id)) = open.pop_first() {
        let n = nodes[id];
        live.remove(&at(g, n.x, n.y));
        if end(n.x, n.y) {
            if max_len < 1 || max_len < n.length {
                let mut path = vec![(0, 0); n.length as usize + 1];
                let mut p = Some(id);
                for slot in path.iter_mut().rev() {
                    let q = nodes[p.expect("the chain is length + 1 long")];
                    *slot = (q.x, q.y);
                    p = q.parent;
                }
                return path;
            }
            continue;
        }
        let mut pending: Vec<usize> = Vec::new();
        for &(dx, dy) in &ORTHOG[1..5] {
            let (cx, cy) = (n.x + dx, n.y + dy);
            if !inb(g, cx, cy) {
                continue;
            }
            let m = at(g, cx, cy);
            if live.contains(&m) || closed.contains(&m) || in_chain(&nodes, id, cx, cy) {
                continue;
            }
            let c = *cell(cx, cy);
            if c.flags & bit::BANK != 0 {
                continue;
            }
            let terminal = end(cx, cy);
            let mut count = 0;
            let mut touches = false;
            for &(ex, ey) in &ORTHOG[1..5] {
                let (u, v) = (cx + ex, cy + ey);
                if (u, v) == (n.x, n.y) || !inb(g, u, v) {
                    continue;
                }
                if !terminal && end(u, v) {
                    count += 1;
                }
                if in_chain(&nodes, id, u, v) {
                    touches = true;
                    break;
                }
            }
            if touches || count > 1 {
                continue;
            }
            if terminal {
                pending.clear();
                let far = max_len <= octagonal((cx - sx).abs(), (cy - sy).abs());
                let joins = ORTHOG[1..5].iter().any(|&(qx, qy)| {
                    let (u, v) = (n.x + qx, n.y + qy);
                    inb(g, u, v) && cell(u, v).flags & bit::PATH != 0 && (u, v) != (cx, cy)
                });
                if far && !joins {
                    nodes.push(Node {
                        x: cx,
                        y: cy,
                        length: n.length + 1,
                        value: 0,
                        parent: Some(id),
                    });
                    pending.push(nodes.len() - 1);
                }
                break;
            }
            if near_start_city(g, cx, cy, city_radius) || c.blocked != 0 || c.flags & 0x8 != 0 {
                continue;
            }
            let mut price = if land_class(&c) < 0 {
                0
            } else {
                draws.get(site::PRICE) % 20 + 10
            };
            if n.length + 1 > 1 {
                let p = nodes[n.parent.expect("a node past the source has a parent")];
                let mult = if cx == p.x || cy == p.y {
                    twist + 1
                } else {
                    if twist == 0 {
                        price *= 10;
                    }
                    101 - twist
                };
                price *= mult;
            }
            let l = n.length;
            let pull = if max_len < l {
                1 + l - max_len
            } else if l < max_len / 2 {
                let b = max_len - l;
                (b / 2 + 2 * b) * 10 + 1
            } else {
                let d = max_len - l;
                (d / 4 + 2 * (d / 2)) * 10 + 1
            };
            let value = pull.wrapping_add(price);
            if value < 0 {
                continue;
            }
            nodes.push(Node {
                x: cx,
                y: cy,
                length: n.length + 1,
                value,
                parent: Some(id),
            });
            pending.push(nodes.len() - 1);
        }
        for &p in &pending {
            push(&mut open, &mut live, &nodes, p);
        }
        closed.insert(at(g, n.x, n.y));
    }
    Vec::new()
}

/// One path cell of an accepted river: 0x1000, and 0x2 on every cell of the
/// `circle_radius[valley]` disc round it (`circle_x`/`circle_y`, centre
/// first) that is on the map and in the path cell's `region`.
fn mark(g: &mut MapGrid, (x, y): (i32, i32), valley: i32) {
    let i = at(g, x, y);
    g.cells[i].flags |= bit::PATH;
    if valley > 0 {
        let region = g.cells[i].region;
        let c = circle();
        for k in 0..c.radius[valley.min(0x40) as usize] {
            let (u, v) = (x + c.x[k], y + c.y[k]);
            if inb(g, u, v) && g.cells[at(g, u, v)].region == region {
                let j = at(g, u, v);
                g.cells[j].flags |= bit::BANK;
            }
        }
    }
}

/// **`Map::make_riverlist@00693820`**, one `RIVER` element, its lengths
/// already scaled by [`make_rivers`] (`length` capped at 399).
///
/// The river count is `SCALE(start + r)`, at least 1, `r` one draw
/// `% (spread + 1)` when the spread is positive. The sources are visited in
/// the order of a Galois LFSR over `1..=beds.len()` — 8 bits and taps
/// 0xb8 under 251 sources, 12 and 0xca0 under 4001, else 16 and 0xb400 —
/// from `World.seed`'s low bits `% n + 1`; the walk stops at its third
/// return to that start. With nations, a source whose nearest start city
/// already ends more rivers than the least is passed over. Each other
/// source runs [`astar_river`] (`max_len` the length under flag 8, else
/// 0); a path of 1..=399 cells is kept when its end is water (flag 1), on
/// the border (flag 2) or inland (flag 4), and it is at least `min_length`
/// and `length / (wraps + 1)` long. A kept river counts against its end's
/// nearest start city and marks its cells ([`mark`]) in the three spline
/// stretches `wide`/`med` cut it into, whose union is the path; a border
/// end marks its last cell again. The walk ends when the count is spent or
/// after its second wrap.
pub fn make_riverlist(
    g: &mut MapGrid,
    beds: &[(i32, i32)],
    r: &River,
    (length, min_length, valley): (i32, i32, i32),
    p: &Phase,
    draws: &mut Draws,
) {
    let n = beds.len() as i32;
    if n == 0 {
        return;
    }
    let spread = (r.end_range - r.start_range).max(0);
    let pick = if spread < 1 {
        0
    } else {
        draws.get(site::SPREAD) % (spread + 1)
    };
    let mut left = scale(pick + r.start_range, g.xs, p.standard).max(1);
    let (bits, taps) = if n < 0xfb {
        (p.seed & 0xff, 0xb8)
    } else if n < 0xfa1 {
        (p.seed & 0xfff, 0xca0)
    } else {
        (p.seed & 0xffff, 0xb400)
    };
    let first = (bits % n as u32) as i32 + 1;
    let (mut s, mut wraps) = (first, 0);
    let mut counts = [0i32; 8];
    let mut least = 0;
    let max_len = if r.river_flags & 8 != 0 { length } else { 0 };
    loop {
        loop {
            let lsb = s & 1;
            s >>= 1;
            if lsb != 0 {
                s ^= taps;
            }
            if s == first {
                wraps += 1;
                if wraps > 2 {
                    return;
                }
            }
            if s <= n {
                break;
            }
        }
        let (x, y) = beds[(s - 1) as usize];
        if p.nations >= 1 && nearest_start(g, x, y, p.nations).is_some_and(|k| counts[k] > least) {
            continue;
        }
        let path = astar_river(g, (x, y), r.river_twist, max_len, p.city_radius, draws);
        let len = path.len() as i32;
        if !(1..=399).contains(&len) {
            continue;
        }
        let (ex, ey) = path[len as usize - 1];
        let end = g.cells[at(g, ex, ey)];
        let water = wet(&end);
        let edge = border(g, ex, ey);
        let allowed = if water {
            r.river_flags & 1
        } else if edge {
            r.river_flags & 2
        } else {
            r.river_flags & 4
        };
        if allowed == 0 || len < min_length || len < length / (wraps + 1) {
            continue;
        }
        if left != 0 {
            left -= 1;
        }
        let more = left != 0 && wraps <= 1;
        // The three stretches' ends. A `wide + med` over 100 would start the
        // second before the path; no stock style has one, and it is clamped.
        let (a, b, c) = if len < 7 {
            (len, len, len)
        } else if water || edge || end.flags & bit::PATH != 0 {
            let a = (100 - r.river_wide - r.river_med) * len / 100;
            (a, len * r.river_med / 100 + a, len)
        } else {
            (0, 0, 0)
        };
        if p.nations > 0
            && let Some(k) = nearest_start(g, ex, ey, p.nations)
        {
            counts[k] += 1;
            least = counts[..p.nations.min(8) as usize]
                .iter()
                .copied()
                .min()
                .unwrap_or(0);
        }
        let stretch = |g: &mut MapGrid, from: i32, to: i32| {
            for i in from.max(0)..to.min(len) {
                mark(g, path[i as usize], valley);
            }
        };
        if a != 0 {
            stretch(g, 0, a);
        }
        if b != a {
            stretch(g, a.max(1) - 1, b);
        }
        if c != b {
            stretch(g, b.max(1) - 1, len);
        }
        if edge {
            let i = at(g, ex, ey);
            g.cells[i].flags |= bit::PATH;
        }
        if !more {
            return;
        }
    }
}

/// **`Map::make_rivers@00692fb0`** on `g`: `checkpoint` is called with the
/// grid and the stream at each `map.cpp` line the original prints (5012, 5014, and, when
/// a source was found, 5076, 5116 and 5118). Each `RIVER` is
/// [`make_riverlist`] with `river_length` (capped at 399),
/// `river_min_length` and `river_valley` scaled by the map's width.
/// Returns `river_beds`.
pub fn make_rivers(
    g: &mut MapGrid,
    regions: &mut Regions,
    p: &Phase,
    draws: &mut Draws,
    mut checkpoint: impl FnMut(i64, &MapGrid, &Draws),
) -> Vec<(i32, i32)> {
    checkpoint(5012, g, draws);
    let beds = make_riverbeds(g, p.thresh, p.city_radius);
    checkpoint(5014, g, draws);
    clear(g, !bit::PATH);
    if beds.is_empty() {
        return beds;
    }
    for r in p.rivers {
        let sc = |v: i32| scale(v, g.xs, p.standard);
        let lengths = (
            sc(r.river_length).min(399),
            sc(r.river_min_length),
            sc(r.river_valley),
        );
        make_riverlist(g, &beds, r, lengths, p, draws);
    }
    checkpoint(5076, g, draws);
    clear(g, !(bit::SOURCE | bit::BANK));
    checkpoint(5116, g, draws);
    regions.find_all(g);
    checkpoint(5118, g, draws);
    beds
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
            (2, 0x20, false),
            (2, 0x900, true),
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

    /// A mountain tile is `mask & 3 == 2`: a mask of 3 is not one.
    #[test]
    fn a_mask_of_3_is_no_mountain() {
        let mut g = forest(7, 7);
        g.tiles[8 * 28 + 9] = 3;
        let t = (4 * 3 * 4 * 7 + 4 * 3) as usize;
        g.tiles[t] = 0x7;
        assert_eq!(make_riverbeds(&mut g, 8, 20), vec![(3, 3)]);
    }

    /// `find_all` runs after 5116: an unlabelled cell is labelled by 5118.
    #[test]
    fn find_all_runs_at_5118() {
        let mut g = forest(7, 7);
        g.cells[0].region = 0;
        let mut regions = Regions::default();
        regions.count[1] = 48;
        let mut at_5116 = 0;
        make_rivers(
            &mut g,
            &mut regions,
            &phase(&[]),
            &mut Draws::new(1),
            |l, g, _| {
                if l == 5116 {
                    at_5116 = g.cells[0].region;
                }
            },
        );
        assert_eq!((at_5116, g.cells[0].region), (0, 1));
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
        // `− 1`: at radius 21, distance 5 (4·5 = 20) is outside.
        let mut g = forest(15, 7);
        g.starts[2].list.push(8);
        g.starts[3].list.push(3);
        assert_eq!(make_riverbeds(&mut g, 8, 21).first(), Some(&(3, 3)));
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

    fn phase(rivers: &[River]) -> Phase<'_> {
        Phase {
            thresh: 8,
            city_radius: 20,
            rivers,
            standard: 70,
            seed: 0,
            nations: 0,
        }
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
        let mut d = Draws::new(1);
        let beds = make_rivers(
            &mut g,
            &mut Regions::default(),
            &phase(&[]),
            &mut d,
            |l, g, _| seen.push((l, g.cell_part())),
        );
        assert_eq!(beds, vec![(3, 3)]);
        let lines: Vec<i64> = seen.iter().map(|s| s.0).collect();
        assert_eq!(lines, [5012, 5014, 5076, 5116, 5118]);
        assert_eq!(seen[0].1, before);
        assert_ne!(seen[1].1, before);
        assert!(g.cells.iter().all(|c| c.flags == 0x20));
        assert_eq!(seen[3].1, seen[4].1);
        assert!(d.log.is_empty());
    }

    /// No source: the paths are still cleared, and nothing past 5014 is
    /// printed or drawn.
    #[test]
    fn no_source_ends_at_5014() {
        let mut g = forest(5, 5);
        *flags(&mut g, 2, 2) |= bit::PATH;
        let mut lines = Vec::new();
        let mut d = Draws::new(1);
        let beds = make_rivers(
            &mut g,
            &mut Regions::default(),
            &phase(&[GREAT_LAKES]),
            &mut d,
            |l, _, _| lines.push(l),
        );
        assert_eq!(beds, vec![]);
        assert_eq!(lines, [5012, 5014]);
        assert_eq!(*flags(&mut g, 2, 2), 0x20);
        assert!(d.log.is_empty());
    }

    /// `GREATLAKES.xml`'s one `RIVER`.
    const GREAT_LAKES: River = River {
        start_range: 1,
        end_range: 2,
        river_length: 12,
        river_min_length: 6,
        river_valley: 8,
        river_wide: 0,
        river_med: 100,
        river_twist: 60,
        river_flags: 11,
    };

    /// A land strip with water along its south edge: `(x, y)` is land for
    /// `y < water`, ocean from there.
    fn shore(xs: i32, ys: i32, water: i32) -> MapGrid {
        let mut g = MapGrid::new(xs, ys);
        for y in 0..ys {
            for x in 0..xs {
                let i = at(&g, x, y);
                let c = &mut g.cells[i];
                c.flags = 0;
                c.land = if y < water { 0 } else { 2 };
                c.region = if y < water { 1 } else { 0x40 };
            }
        }
        g
    }

    /// The search walks south to the nearer water, one draw a dry cell it prices,
    /// and returns the path source first.
    #[test]
    fn a_river_runs_to_the_water() {
        let g = shore(21, 20, 12);
        let mut d = Draws::new(7);
        let path = astar_river(&g, (10, 8), 60, 0, 0, &mut d);
        assert_eq!(path.first(), Some(&(10, 8)));
        let last = *path.last().expect("a path");
        assert!(wet(&g.cells[at(&g, last.0, last.1)]), "{path:?}");
        for w in path.windows(2) {
            let (a, b) = (w[0], w[1]);
            assert_eq!((a.0 - b.0).abs() + (a.1 - b.1).abs(), 1, "{path:?}");
        }
        assert!(!d.log.is_empty());
        assert!(d.log.iter().all(|&(s, _)| s == site::PRICE));
    }

    /// A source next to the water is refused before the search, with no
    /// draw.
    #[test]
    fn a_source_on_the_shore_is_refused() {
        let g = shore(9, 12, 8);
        let mut d = Draws::new(7);
        assert!(astar_river(&g, (4, 7), 60, 0, 0, &mut d).is_empty());
        assert!(d.log.is_empty());
    }

    /// Under flag 8 a path no longer than `max_len` is not returned.
    #[test]
    fn a_short_river_is_not_returned() {
        let g = shore(9, 12, 8);
        let mut d = Draws::new(7);
        assert!(astar_river(&g, (4, 4), 60, 10, 0, &mut d).is_empty());
    }

    /// A kept river marks its path 0x1000, and its valley's disc 0x2 only in
    /// the region of the path cell it is drawn round.
    #[test]
    fn a_river_marks_its_path_and_banks() {
        let mut g = shore(21, 20, 12);
        let beds = [(10, 8)];
        let r = River {
            river_flags: 1,
            ..GREAT_LAKES
        };
        let mut d = Draws::new(3);
        make_riverlist(&mut g, &beds, &r, (0, 0, 2), &phase(&[]), &mut d);
        assert_eq!(d.log[0].0, site::SPREAD);
        let path: Vec<usize> = (0..g.cells.len())
            .filter(|&i| g.cells[i].flags & bit::PATH != 0)
            .collect();
        assert!(!path.is_empty());
        assert!(g.cells[at(&g, 10, 8)].flags & bit::PATH != 0);
        let regions: BTreeSet<i16> = path.iter().map(|&i| g.cells[i].region).collect();
        assert!(
            g.cells
                .iter()
                .all(|c| c.flags & bit::BANK == 0 || regions.contains(&c.region))
        );
        assert!(g.cells.iter().any(|c| c.flags & bit::BANK != 0));
    }
}
