//! **`Map::place_resources@0068f4f0`** — map generation's fourth phase
//! (`docs/MAPGEN.md` §8, §15 step 4): the bonus ladder over the map file's
//! `BONUSES`, `FISH ` and `GOODIES` rows, the divvy pool's rare goods, and
//! the two placers — `Map::place_player_resource@00691f70` around each
//! start and `Map::place_region_resource@00690480` over a region's cells in
//! its 16-bit LFSR order. Checked draw for draw against the original's
//! draw-site trace (every draw tagged with its return address), and at
//! `map.cpp` 7927 by parts 5 and 6.
//!
//! Integers throughout. A placed good writes its footprint's tile masks
//! (0x200, and `World::set_blocked_at` where the good blocks), its cell's
//! flag 1 and `down`/`down_who`; a goody box its cell's 0x8000 and
//! `down`/`down_who`. The objects themselves (their checksum lines, the
//! herds) are not held: what later phases read of them is in the cell.

use crate::ai::ai_place::{MOVE_X, MOVE_Y};
use crate::mapgen::MapGrid;
use crate::mapgen::terrain_groups::{Draws, set_blocked_at};
use crate::units::think::scout::circle;
use crate::world::vector_dist;

/// The draw sites, as return addresses.
pub mod site {
    pub const LADDER: u32 = 0x0068_f4f0 + 0x879;
    pub const GET_EARLY: u32 = 0x0068_a640 + 0x31;
    pub const GET_LATE: u32 = 0x0068_a570 + 0x32;
    /// `get_water`'s draw; no divvy row reaches it (a divvy type is −1, so
    /// never water).
    pub const GET_WATER: u32 = 0x0068_a4a0 + 0x32;
    pub const FIND_AVAIL: u32 = 0x0068_f380 + 0x11d;
    pub const PLAYER_RING: u32 = 0x0069_1f70 + 0x1a9;
    pub const REGION_START: u32 = 0x0069_0480 + 0x29f;
}

/// `TYPE_NONE`: a divvy row's type before the pool names it.
pub const NO_TYPE: i32 = -1;
/// The goody box's item type (`BASE_ITEMTYPES`).
pub const GOODY_TYPE: i32 = 0x21f;
/// `OIL`, which a spacing test never counts.
const OIL: i32 = 5;
/// The two sea goods (`Fish`, `Whales`), which only space against each other.
const SEA: [i32; 2] = [6, 0x1f];

/// One `BONUS` row, resolved as `place_resources` reads it: the counts
/// already through `Map::scale_number`, `player_keep_away` raw.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Row {
    /// A good index, [`GOODY`], or [`NO_TYPE`] for a divvy row.
    pub ty: i32,
    /// 0 a named type, 1 `Random`, 2 `Early`, 3 `Late`.
    pub kind: i32,
    pub chance: i32,
    pub group: i32,
    /// 0 player, 1 continent, 2 world, 3 nonplayer, 4 corner.
    pub pattern: i32,
    pub saturate: i32,
    pub numrare: i32,
    pub player_keep_away: i32,
    pub player_stay_near: i32,
    pub spacing: i32,
    pub group_spacing: i32,
    pub center_keep_away: i32,
    pub center_stay_near: i32,
    pub edge_keep_away: i32,
    pub edge_stay_near: i32,
    pub corner_keep_away: i32,
    pub corner_stay_near: i32,
}

/// A good type's footprint, as `Good::init@0066da20` reads it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Footprint {
    pub x_size: i32,
    pub y_size: i32,
    pub block: bool,
}

/// What `place_resources` reads that is not the grid.
#[derive(Clone, Debug, Default)]
pub struct Inputs {
    /// The three sections in order, each its rows in file order; a row
    /// whose type resolves to nothing is left out (it draws nothing).
    pub sections: Vec<Vec<Row>>,
    /// The divvy pool's lists, in `add_good` order.
    pub early: Vec<i32>,
    pub late: Vec<i32>,
    pub water: Vec<i32>,
    /// Per good index.
    pub goods: Vec<Footprint>,
    /// `lands[c].rare`: the goods each land class allows.
    pub lands: Vec<Vec<i32>>,
}

/// `ResourceDivvyPool`'s three lists and their taken bits.
struct Pool<'a> {
    lists: [&'a [i32]; 3],
    taken: [Vec<bool>; 3],
}

impl Pool<'_> {
    /// **`get_early@0068a640` / `get_late@0068a570` / `get_water@0068a4a0`**:
    /// draw an index (`r % count`, only when the list holds two or more)
    /// until one not yet taken comes up; once every index is taken the bits
    /// clear, with no draw.
    fn get(&mut self, k: usize, d: &mut Draws) -> i32 {
        let site = [site::GET_EARLY, site::GET_LATE, site::GET_WATER][k];
        let list = self.lists[k];
        let c = list.len() as i32;
        let got = loop {
            let i = if c - 1 > 0 { d.get(site) % c } else { 0 } as usize;
            if !self.taken[k][i] {
                self.taken[k][i] = true;
                break list[i];
            }
        };
        if self.taken[k].iter().all(|&t| t) {
            self.taken[k].iter_mut().for_each(|t| *t = false);
        }
        got
    }
}

/// The goody boxes placed so far, by cell, in creation order — the
/// `items` list, whose index is the cell's `down_who`. The goods list is
/// the grid's ([`MapGrid::goods`]), oil from `place_all` included.
#[derive(Default)]
struct Placed {
    items: Vec<(i32, i32)>,
}

struct Pass<'a> {
    g: &'a mut MapGrid,
    d: &'a mut Draws,
    inp: &'a Inputs,
    pool: Pool<'a>,
    placed: Placed,
    nations: i32,
}

/// **`Map::place_resources@0068f4f0`** on a grid at `map.cpp` 7880, for
/// `nations` players. Returns the count placed.
pub fn place_resources(g: &mut MapGrid, d: &mut Draws, inp: &Inputs, nations: i32) -> i32 {
    let mut pass = Pass {
        g,
        d,
        inp,
        pool: Pool {
            lists: [&inp.early, &inp.late, &inp.water],
            taken: [
                vec![false; inp.early.len()],
                vec![false; inp.late.len()],
                vec![false; inp.water.len()],
            ],
        },
        placed: Placed::default(),
        nations,
    };
    // `roll` and the last group carry across sections; the selected flag
    // does not.
    let (mut roll, mut last_group, mut total) = (0, -1, 0);
    for rows in &inp.sections {
        let mut selected = false;
        for row in rows {
            if row.group != last_group || row.group == 0 {
                last_group = row.group;
                roll = pass.d.get(site::LADDER) % 100;
                selected = false;
            }
            if roll < 0 && (row.chance != 0 || !selected) {
                selected = false;
                continue;
            }
            roll -= row.chance;
            if roll >= 0 {
                selected = false;
                continue;
            }
            selected = true;
            if row.numrare == 0 {
                continue;
            }
            total += if row.pattern == 0 {
                pass.player(row)
            } else {
                pass.region(row)
            };
        }
    }
    total
}

impl Pass<'_> {
    fn at(&self, x: i32, y: i32) -> usize {
        (y * self.g.xs + x) as usize
    }

    fn inb(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.g.xs && y < self.g.ys
    }

    fn flags(&self, x: i32, y: i32) -> u16 {
        self.g.cells[self.at(x, y)].flags
    }

    fn starts(&self) -> Vec<(i32, i32)> {
        let [sx, sy, ..] = &self.g.starts;
        sx.list
            .iter()
            .copied()
            .zip(sy.list.iter().copied())
            .collect()
    }

    /// The pool's type for a divvy row: `Early` from the early list, `Random`
    /// and `Late` from the late one, a water good from the water list.
    fn divvy(&mut self, kind: i32, water: bool) -> i32 {
        let k = if water {
            2
        } else if kind == 2 {
            0
        } else {
            1
        };
        self.pool.get(k, self.d)
    }

    /// `min(x, y, W − x, H − y)`, as the edge tests take it.
    fn edge(&self, x: i32, y: i32) -> i32 {
        x.min(y).min(self.g.xs - x).min(self.g.ys - y)
    }

    /// The distance to the corners `corner_box_x/y` name, (0,0), (0,H),
    /// (W,0), (W,H), capped at 9999: all four, or one.
    fn corner(&self, x: i32, y: i32, only: Option<usize>) -> i32 {
        const BOX: [(i32, i32); 4] = [(0, 0), (0, 1), (1, 0), (1, 1)];
        (0..4)
            .filter(|&c| only.is_none_or(|o| o == c))
            .map(|c| vector_dist(x - BOX[c].0 * self.g.xs, y - BOX[c].1 * self.g.ys))
            .fold(9999, i32::min)
    }

    fn centre(&self, x: i32, y: i32) -> i32 {
        vector_dist(x - self.g.xs / 2, y - self.g.ys / 2)
    }

    /// The centre, edge and corner bounds; a zero bound is no test
    /// (a negative centre keep-away still runs).
    fn bounds_ok(&self, r: &Row, x: i32, y: i32, corner: Option<usize>) -> bool {
        let c = self.centre(x, y);
        if r.center_keep_away != 0 && c < r.center_keep_away {
            return false;
        }
        if r.center_stay_near > 0 && c > r.center_stay_near {
            return false;
        }
        let e = self.edge(x, y);
        if (r.edge_keep_away > 0 && e < r.edge_keep_away)
            || (r.edge_stay_near > 0 && e > r.edge_stay_near)
        {
            return false;
        }
        if r.corner_keep_away > 0 || r.corner_stay_near > 0 {
            let m = self.corner(x, y, corner);
            if (r.corner_keep_away > 0 && m < r.corner_keep_away)
                || (r.corner_stay_near > 0 && m > r.corner_stay_near)
            {
                return false;
            }
        }
        true
    }

    /// `Terrain::has_south_forest@00850060`.
    fn south_forest(&self, x: i32, y: i32) -> bool {
        [(x + 1, y + 1), (x, y + 1), (x + 1, y)]
            .into_iter()
            .any(|(u, v)| self.inb(u, v) && self.flags(u, v) & 0x20 != 0)
    }

    /// `WorldData::has_mountain_tcoords@006b3050`.
    fn mountain_tiles(&self, x: i32, y: i32) -> bool {
        (0..16).any(|j| {
            let (tx, ty) = (4 * x + (j & 3), 4 * y + (j >> 2));
            self.g.tiles[(ty * 4 * self.g.xs + tx) as usize] & 3 == 2
        })
    }

    /// The map's border, with y compared against `W − 1` as the listing
    /// does (harmless on a square map).
    fn border(&self, x: i32, y: i32) -> bool {
        x == 0 || y == 0 || x == self.g.xs - 1 || y == self.g.xs - 1
    }

    /// Spacing against the goods (oil never; the sea goods only against
    /// each other) or, for a goody box, against every box.
    fn spaced(&self, ty: i32, sea: bool, x: i32, y: i32, spacing: i32) -> bool {
        if ty == GOODY_TYPE {
            self.placed
                .items
                .iter()
                .all(|&(u, v)| vector_dist(u - x, v - y) >= spacing)
        } else {
            self.g
                .goods
                .iter()
                .flatten()
                .filter(|&&(t, _, _)| t != OIL && SEA.contains(&t) == sea)
                .all(|&(_, u, v)| vector_dist(u - x, v - y) >= spacing)
        }
    }

    /// `WorldData::get_land(…, 1)@006b4730`'s class of a cell.
    fn land_class(&self, x: i32, y: i32) -> usize {
        let c = self.g.cells[self.at(x, y)];
        let f = c.flags;
        (if f & 0x4 != 0 {
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
        }) as usize
    }

    fn allows(&self, class: usize, ty: i32) -> bool {
        self.inp.lands.get(class).is_some_and(|l| l.contains(&ty))
    }

    /// The occupancy chain's end: a cell already holding a good (−2) or a
    /// goody box (−3) is refused. No unit exists before the first frame,
    /// so the chain is the cell's own `down`.
    fn occupied(&self, x: i32, y: i32) -> bool {
        self.g.cells[self.at(x, y)].down < -1
    }

    /// **`Objects::init_good@00653f30`** / **`init_item@00653e00`** at a
    /// cell: the good's footprint (tile mask 0x200, then `set_blocked_at`
    /// where it blocks, tile by tile, x outer) around its centre — the cell
    /// centre for an even X_SIZE, a tile up-left of it for an odd — then
    /// the cell's flag and its `down`/`down_who`.
    fn create(&mut self, ty: i32, x: i32, y: i32) {
        let i = self.at(x, y);
        if ty == GOODY_TYPE {
            let n = self.placed.items.len() as i16;
            let c = &mut self.g.cells[i];
            c.flags |= 0x8000;
            c.down = -3;
            c.down_who = n;
            self.placed.items.push((x, y));
            return;
        }
        let fp = self.inp.goods[ty as usize];
        if ty == OIL {
            self.g.init_good(ty, x, y);
        } else {
            place_good(self.g, fp, ty, x, y);
        }
    }

    /// **`Map::place_player_resource@00691f70`**: per player, per count,
    /// the pool's type (a divvy row), then one draw for where to start in
    /// the ring of offsets between the keep-away and stay-near radii, and a
    /// scan of the ring from there; the first cell that passes takes the
    /// good. No retry.
    fn player(&mut self, r: &Row) -> i32 {
        let t = circle::table();
        let starts = self.starts();
        let sea = SEA.contains(&r.ty);
        let (mut ty, mut keep, mut near) = (r.ty, r.player_keep_away, r.player_stay_near.max(0));
        let spacing = r.spacing.max(1);
        let gspacing = r.group_spacing.max(1);
        let mut group: Vec<(i32, i32)> = Vec::new();
        for p in 0..self.nations as usize {
            let Some(&(sx, sy)) = starts.get(p) else {
                continue;
            };
            if !self.inb(sx, sy) {
                continue;
            }
            for _ in 0..r.numrare {
                if r.kind != 0 {
                    ty = self.divvy(r.kind, false);
                }
                if near > 0x40 || near == 0 {
                    near = 0x40;
                }
                keep = keep.min(near);
                let lo = t.radius[keep as usize];
                let len = t.radius[near as usize] - lo;
                let start = if len < 1 {
                    0
                } else {
                    self.d.get(site::PLAYER_RING) % (len + 1)
                };
                for j in 0..len {
                    let k = ((start + j) % len + lo) as usize;
                    let (x, y) = (sx + i32::from(t.x[k]), sy + i32::from(t.y[k]));
                    if self.player_ok(
                        r,
                        ty,
                        sea,
                        p,
                        &starts,
                        &group,
                        (x, y),
                        (keep, near, spacing, gspacing),
                    ) {
                        self.create(ty, x, y);
                        group.push((x, y));
                        break;
                    }
                }
            }
        }
        group.len() as i32
    }

    /// `place_player_resource`'s candidate test, in the original's order.
    #[allow(clippy::too_many_arguments)]
    fn player_ok(
        &self,
        r: &Row,
        ty: i32,
        sea: bool,
        p: usize,
        starts: &[(i32, i32)],
        group: &[(i32, i32)],
        (x, y): (i32, i32),
        (keep, near, spacing, gspacing): (i32, i32, i32, i32),
    ) -> bool {
        if !self.inb(x, y) || self.g.city[self.at(x, y)] {
            return false;
        }
        let f = self.flags(x, y);
        if f & 0x1000 != 0 {
            return false;
        }
        let nations = (self.nations as usize).min(starts.len());
        let dist = |q: usize| vector_dist(starts[q].0 - x, starts[q].1 - y);
        if keep > 0 && (0..nations).any(|q| dist(q) < keep) {
            return false;
        }
        if f & 0x1878 != 0 || self.g.cells[self.at(x, y)].blocked != 0 {
            return false;
        }
        let own = dist(p);
        if own > near {
            return false;
        }
        if (0..nations).any(|q| q != p && dist(q) <= own) {
            return false;
        }
        if !self.bounds_ok(r, x, y, None) || self.south_forest(x, y) {
            return false;
        }
        if group
            .iter()
            .any(|&(u, v)| vector_dist(u - x, v - y) < gspacing)
        {
            return false;
        }
        if self.border(x, y) || !self.spaced(ty, sea, x, y, spacing) {
            return false;
        }
        if ty != GOODY_TYPE
            && [2, 4, 6, 8].into_iter().any(|k| {
                let (u, v) = (x + MOVE_X[k], y + MOVE_Y[k]);
                self.inb(u, v) && self.flags(u, v) & 0x4 != 0
            })
        {
            return false;
        }
        if self.mountain_tiles(x, y) {
            return false;
        }
        let fits = if ty == GOODY_TYPE {
            !self.g.cells[self.at(x, y)].is_ocean() && f & 0x70 == 0
        } else {
            self.allows(self.land_class(x, y), ty)
        };
        fits && !self.occupied(x, y)
    }

    /// **`Map::find_avail_regions@0068f380`**: the land (1..0x40) or water
    /// (0x40..0x7f) regions of more than five cells — for `nonplayer`, none
    /// holding a start — in descending id, the ring's cursor seated at
    /// `r % len` when there are two or more.
    fn avail_regions(&mut self, water: bool, pattern: i32) -> (Vec<i32>, usize) {
        let ids = if water { 0x40..0x7f } else { 1..0x40 };
        let starts = self.starts();
        let mut out = Vec::new();
        for reg in ids.rev() {
            let size = self
                .g
                .cells
                .iter()
                .filter(|c| i32::from(c.region) == reg)
                .count();
            if size <= 5 {
                continue;
            }
            if pattern == 3
                && starts.iter().any(|&(x, y)| {
                    self.inb(x, y) && i32::from(self.g.cells[self.at(x, y)].region) == reg
                })
            {
                continue;
            }
            out.push(reg);
        }
        let k = if out.len() <= 1 {
            0
        } else {
            (self.d.get(site::FIND_AVAIL) % out.len() as i32) as usize
        };
        (out, k)
    }

    /// A region's cells (`Region.coords`), in `find_all`'s scan order.
    fn region_cells(&self, reg: i32) -> Vec<(i32, i32)> {
        let mut out = Vec::new();
        for y in 0..self.g.ys {
            for x in 0..self.g.xs {
                if i32::from(self.g.cells[self.at(x, y)].region) == reg {
                    out.push((x, y));
                }
            }
        }
        out
    }

    /// **`Map::place_region_resource@00690480`**: over the available
    /// regions, per placement the pool's type, one draw for the walk's start
    /// state, and the region's cells in the order of the 16-bit Galois LFSR
    /// `place_all` walks (0x16800 before the shift); the first cell that
    /// passes takes the good. `world` and `nonplayer` move to the next
    /// region after each try, `continent` after each pass, `corner` never —
    /// it runs one pass per corner.
    fn region(&mut self, r: &Row) -> i32 {
        let water = r.ty >= 0 && r.ty != GOODY_TYPE && self.allows(2, r.ty);
        let (regs, mut cur) = self.avail_regions(water, r.pattern);
        if regs.is_empty() {
            return 0;
        }
        let thresh =
            if r.numrare > 2 && r.saturate == 0 && (!matches!(r.pattern, 1 | 3) || regs.len() == 1)
            {
                r.numrare >> 1
            } else {
                i32::MAX
            };
        let outer = match r.pattern {
            1 => regs.len(),
            4 => 4,
            _ => 1,
        };
        let starts = self.starts();
        let spacing = r.spacing.max(1);
        let gspacing = r.group_spacing.max(1);
        let mut ty = r.ty;
        let mut acc = vec![0; starts.len()];
        let mut starved = 0;
        let mut group: Vec<(i32, i32)> = Vec::new();
        for o in 0..outer {
            let mut last_ok = regs[cur];
            let mut i = 0;
            while i < r.numrare {
                if r.kind != 0 {
                    ty = self.divvy(r.kind, water);
                }
                let cells = self.region_cells(regs[cur]);
                let n = cells.len() as i32;
                let s0 = if n - 1 > 0 {
                    self.d.get(site::REGION_START) % n + 1
                } else {
                    1
                };
                let corner = (r.pattern == 4).then_some(o);
                let mut s = s0;
                let mut done = false;
                loop {
                    let (x, y) = cells[(s - 1) as usize];
                    let balance = (i >= thresh).then_some(starved);
                    if self.region_ok(
                        r,
                        ty,
                        water,
                        &starts,
                        &group,
                        (x, y),
                        corner,
                        balance,
                        (spacing, gspacing),
                    ) {
                        self.create(ty, x, y);
                        for (a, &(sx, sy)) in acc.iter_mut().zip(&starts) {
                            *a += vector_dist(sx - x, sy - y);
                        }
                        let mut max = 0;
                        for (p, &a) in acc.iter().enumerate() {
                            if a > max {
                                max = a;
                                starved = p;
                            }
                        }
                        group.push((x, y));
                        done = true;
                        break;
                    }
                    loop {
                        if s & 1 != 0 {
                            s ^= 0x16800;
                        }
                        s >>= 1;
                        if s <= n {
                            break;
                        }
                    }
                    if s == s0 {
                        break;
                    }
                }
                if matches!(r.pattern, 2 | 3) {
                    let old = regs[cur];
                    cur = (cur + 1) % regs.len();
                    if !done {
                        if regs[cur] != last_ok {
                            i -= 1;
                        }
                    } else {
                        last_ok = old;
                    }
                }
                i += 1;
            }
            if r.pattern == 1 {
                cur = (cur + 1) % regs.len();
            }
        }
        group.len() as i32
    }

    /// `place_region_resource`'s cell test, in the original's order.
    #[allow(clippy::too_many_arguments)]
    fn region_ok(
        &self,
        r: &Row,
        ty: i32,
        water: bool,
        starts: &[(i32, i32)],
        group: &[(i32, i32)],
        (x, y): (i32, i32),
        corner: Option<usize>,
        balance: Option<usize>,
        (spacing, gspacing): (i32, i32),
    ) -> bool {
        let f = self.flags(x, y);
        let dist = |&(sx, sy): &(i32, i32)| vector_dist(sx - x, sy - y);
        if f & 0x1000 == 0
            && r.player_keep_away > 0
            && starts.iter().any(|s| dist(s) < r.player_keep_away)
        {
            return false;
        }
        if f & 0x1878 != 0 || self.g.cells[self.at(x, y)].blocked != 0 {
            return false;
        }
        if r.player_stay_near > 0 && starts.iter().any(|s| dist(s) > r.player_stay_near) {
            return false;
        }
        if !self.bounds_ok(r, x, y, corner) {
            return false;
        }
        if group
            .iter()
            .any(|&(u, v)| vector_dist(u - x, v - y) < gspacing)
        {
            return false;
        }
        if !self.spaced(ty, water, x, y, spacing) {
            return false;
        }
        let touches = if water {
            (-2..=2).any(|dy| {
                (-2..=2).any(|dx| {
                    let (u, v) = (x + dx, y + dy);
                    (dx, dy) != (0, 0) && self.inb(u, v) && self.g.cells[self.at(u, v)].land == 0
                })
            })
        } else {
            (1..=8).any(|k| {
                let (u, v) = (x + MOVE_X[k], y + MOVE_Y[k]);
                self.inb(u, v) && self.g.cells[self.at(u, v)].is_ocean()
            })
        };
        if touches || self.mountain_tiles(x, y) || self.border(x, y) || self.south_forest(x, y) {
            return false;
        }
        let fits = if ty == GOODY_TYPE {
            f & 0x70 == 0
        } else {
            self.allows(self.land_class(x, y), ty) && (!water || f & 0x4 == 0)
        };
        if !fits || self.occupied(x, y) {
            return false;
        }
        if let Some(starved) = balance {
            let mut nearest = 0;
            let mut best = i32::MAX;
            for (p, s) in starts.iter().enumerate() {
                let d = dist(s);
                if d < best {
                    best = d;
                    nearest = p;
                }
            }
            if nearest != starved {
                return false;
            }
        }
        true
    }
}

/// A good's footprint's first tile: its centre (`GoodTypeData::snap_center@0066fcb0`,
/// the cell centre for an even `X_SIZE`, a tile up-left of it for an odd)
/// over 192, less half the size each way.
fn footprint_corner(fp: Footprint, x: i32, y: i32) -> (i32, i32) {
    let off = if fp.x_size & 1 != 0 { 0x120 } else { 0x180 };
    let (gx, gy) = (x * 0x300 + off, y * 0x300 + off);
    (gx / 0xc0 - fp.x_size / 2, gy / 0xc0 - fp.y_size / 2)
} // twin of GoodTypeData::snap_center

/// **`Objects::init_good@00653f30`** of a good other than oil at cell
/// `(x, y)`: the footprint's tile mask 0x200, then `set_blocked_at` where
/// the type blocks, tile by tile, x outer; then the cell's flag 1 and its
/// `down`/`down_who`. Returns the slot.
pub(crate) fn place_good(g: &mut MapGrid, fp: Footprint, ty: i32, x: i32, y: i32) -> usize {
    let i = (y * g.xs + x) as usize;
    let n = g.init_good(ty, x, y);
    let (tx0, ty0) = footprint_corner(fp, x, y);
    let w = g.xs * 4;
    for tx in tx0..tx0 + fp.x_size {
        for tyy in ty0..ty0 + fp.y_size {
            g.tiles[(tyy * w + tx) as usize] |= 0x200;
            if fp.block {
                set_blocked_at(g, tx, tyy, true);
            }
        }
    }
    let c = &mut g.cells[i];
    c.flags |= 1;
    c.down = -2;
    c.down_who = n as i16;
    n
}

/// **`Good::close@0066d860`** of the good in `slot`: the footprint's tile
/// 0x200 cleared and, where the type blocks, `set_blocked_at(.., 0)`, x
/// outer; flag 1 off the cell of the footprint's first tile; then
/// `World::clear_down@006b3ad0` on the centre's cell, whose `down` and
/// `down_who` go to −1 when no object chain hangs there. A herd type's
/// `Herd::close` is not reached: no herd exists inside `Terrain::init`.
pub(crate) fn close_good(g: &mut MapGrid, fp: Footprint, slot: usize) {
    let Some((_, x, y)) = g.goods[slot] else {
        return;
    };
    let (tx0, ty0) = footprint_corner(fp, x, y);
    let w = g.xs * 4;
    for tx in tx0..tx0 + fp.x_size {
        for tyy in ty0..ty0 + fp.y_size {
            g.tiles[(tyy * w + tx) as usize] &= 0xfdff;
            if fp.block {
                set_blocked_at(g, tx, tyy, false);
            }
        }
    }
    let corner = ((ty0 >> 2) * g.xs + (tx0 >> 2)) as usize;
    g.cells[corner].flags &= 0xfffe;
    let c = &mut g.cells[(y * g.xs + x) as usize];
    if c.down < 0 {
        c.down = -1;
        c.down_who = -1;
    }
    g.goods[slot] = None;
}
