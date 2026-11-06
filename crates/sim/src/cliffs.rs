//! **The cliffs' heights and blocking** (`docs/MAPGEN.md` §10, §15 step 7,
//! §18.3, §18.4): `Cliffs::init@008a8020`, the last call of
//! `Terrain::init` before `Terrain.cpp` 894, after
//! `TerrainOut::adjust_for_mountains`.
//!
//! In order:
//! 1. each cliff `Cliffs::add_cliff` queued ([`MapGrid::cliffs`]) goes
//!    through `Cliffs::place_cliff@008a7ea0` — its template's heights added
//!    to the vertex box — and its template's cells take its index
//!    (`CoordInfo::cliff_group_index`, +0x6a);
//! 2. `Cliffs::assess_wcoord@008a6a40` on every cell, y outer: on a cell
//!    flagged 0x40, each of its sixteen tiles that `has_plus_slope` passes
//!    is blocked with its eight neighbours, their cells' goods moved
//!    (`Terrain::move_goody@0084a3d0`); a cell no tile of which passes
//!    loses 0x40;
//! 3. the cliff mining data: every cell a blocked neighbour touched,
//!    counted into its cliff's tile list (`Cliff::init@008a5190`'s count);
//! 4. `Cliffs::set_edge_flags@008a85e0`: the gather bit 0x8000 on each such
//!    tile with gather access.
//!
//! What the renderer alone reads (`calculate_norms`, `calculate_tangents`,
//! the `Cliff` meshes, the vertex colours) is not built.

use crate::halfland::{Combo, Pass};
use crate::map_resources::Footprint;
use crate::mapgen::{COAST, MapGrid};
use crate::single::Single;
use crate::terrain_groups::{CliffTemplate, set_blocked_at, tile_at, tile_inb};
use crate::terrain_init::coord;
use crate::world::MOVE_49;

/// `offtable@00af2fb0`: each size index's box, in cells, from the origin.
const OFFTABLE: [i32; 5] = [0, -1, -2, -4, -8];

/// `index_to_size@00af2fd0`: each size index's vertex box side, the
/// texture's width.
const INDEX_TO_SIZE: [i32; 5] = [4, 8, 16, 32, 64];

/// `has_plus_slope`'s clamp, `0x461c4000` (10000) and its negative.
const CLAMP: Single = Single(0x461c_4000);
const CLAMP_NEG: Single = Single(0xc61c_4000);

/// **The slope threshold**, `192 · tanf(25 · 0.017453292)`: `assess_wcoord`
/// hands `has_plus_slope` 25 (`0x41c80000`, `[0xb69658]`) in `xmm3`, which
/// it scales by `0x3c8efa35` before `tanf` and whose result it multiplies
/// by 192 (`0x43400000`): 89.531075 (`docs/MAPGEN.md` §13's float table,
/// pinned; a native-`f32` oracle test re-derives it).
pub const SLOPE: Single = Single(0x42b3_0fe9);

/// What `Cliffs::init` did besides the heights.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CliffsInit {
    /// The tiles `assess_wcoord` handed to `has_plus_slope`.
    pub handed: usize,
    /// The tiles it passed, in call order.
    pub slope: Vec<(i32, i32)>,
    /// The 0x40 cells no tile of which passed, which lost 0x40.
    pub cleared: Vec<(i32, i32)>,
    /// The cells `move_goody` was called on, in call order.
    pub goody_calls: usize,
    /// The oil it moved, from and to.
    pub moved: Vec<((i32, i32), (i32, i32))>,
    /// The goods it moved, from and to.
    pub goods: Vec<((i32, i32), (i32, i32))>,
    /// Each cliff's mining tiles, by queue index.
    pub mining: Vec<Vec<(i32, i32)>>,
    /// The tiles `set_edge_flags` gave 0x8000.
    pub edge: usize,
    /// What the arms that decided nothing on a grid had to decide on.
    pub inert: Inert,
}

/// **What each guard of `Cliffs::init` was handed to refuse** (item 3890):
/// an arm that a mutant leaves unbroken is inert by its grid, and these
/// are the counts that say so. Each counts the guard's own condition,
/// whatever the guards before it decided.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Inert {
    /// `place_cliff`'s on-map box vertices carrying 0x10, dry-skipped,
    /// carrying 0x1000, and on a `CoordInfo` 0x4 cell.
    pub box_skips: [usize; 4],
    /// The 0x40 cells' tiles `assess_wcoord` refused as COAST or `land` 1
    /// or 2, and as a mountain's.
    pub refused: [usize; 2],
    /// The handed tiles' least distance, in ulps, of the span from
    /// [`SLOPE`]; 0 is a tile the strict `>` decides.
    pub nearest_ulps: u32,
    /// The handed tiles with a corner past ±10000, which the clamp
    /// decides.
    pub past_clamp: usize,
    /// The tiles blocked whose mask carried 0x2 at the rewrite.
    pub bit1_rewritten: usize,
    /// The closing `move_goody` calls that moved anything.
    pub closing_moves: usize,
    /// The marked cells under a cliff index whose count was 0 with marked
    /// tiles: what `n != 0` refuses.
    pub zero_count_cells: usize,
    /// The good arm's candidates refused for a forest south of them alone.
    pub south_forest: usize,
}

/// An arm `Cliffs::init` can enter that is not built.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unported {
    /// `Terrain::move_goody` would move a good off these cells.
    Goody(Vec<(i32, i32)>),
}

/// **`init_templates`' heights** for a texture whose pixels are `px` (row 0
/// first, `[byte 0, alpha]` each): `(float)byte · (1/255) · 480`, two
/// `mulss` (0x8a762f..0x8a763a), one a pixel in row order — the array
/// `place_cliff` adds.
pub fn template_heights(px: &[[u8; 2]]) -> Vec<Single> {
    let (inv, scale) = (Single(0x3b80_8081), Single(0x43f0_0000));
    px.iter()
        .map(|p| Single::from_i32(i32::from(p[0])).mulss(inv).mulss(scale))
        .collect()
}

/// `minss a, b`: `a` when it is below `b`, else `b`.
fn minss(a: Single, b: Single) -> Single {
    if b.gt(a) { a } else { b }
}

/// `maxss a, b`: `a` when it is above `b`, else `b`.
fn maxss(a: Single, b: Single) -> Single {
    if a.gt(b) { a } else { b }
}

/// **`TerrainOut::has_plus_slope@00869790`** on tile `(tx, ty)`: its four
/// corners' span, the first clamped to ±10000, strictly above [`SLOPE`]
/// (listing 0x8697ce..0x86984e: `minss`/`maxss` in corner order, `subss`,
/// `comiss`, `jbe`).
pub fn has_plus_slope(z: &[Single], w: i32, tx: i32, ty: i32) -> bool {
    let at = |x: i32, y: i32| z[(y * w + x) as usize];
    let v0 = at(tx, ty);
    let (mut lo, mut hi) = (minss(v0, CLAMP), maxss(v0, CLAMP_NEG));
    for v in [at(tx + 1, ty), at(tx, ty + 1), at(tx + 1, ty + 1)] {
        let (l, h) = (minss(v, lo), maxss(v, hi));
        lo = l;
        hi = h;
    }
    hi.subss(lo).gt(SLOPE)
}

/// One `Cliffs::init` over a grid and its heights.
struct Init<'a> {
    g: &'a mut MapGrid,
    coord: &'a mut [u16],
    z: &'a mut [Single],
    combo: Combo,
    goods: &'a [Footprint],
    /// `CoordInfo::cliff_group_index` (+0x6a), −1 from the constructor.
    group: Vec<i32>,
    /// The three `DynamicBitMask`s `Cliffs::init` sizes: the passed tiles
    /// (+0x5e4), their neighbours (+0x5f0), and the neighbours' cells
    /// (+0x5fc).
    passed: Vec<bool>,
    beside: Vec<bool>,
    cells: Vec<bool>,
    out: CliffsInit,
    goody: Vec<(i32, i32)>,
}

impl Init<'_> {
    fn stride(&self) -> i32 {
        4 * self.g.xs + 1
    }

    fn in_cells(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.g.xs && y < self.g.ys
    }

    /// **`Cliffs::place_cliff@008a7ea0`** of template `heights` (size
    /// index `t`) at cell `(x, y)`: over the `index_to_size[t]`² vertex box
    /// at `offtable[t]` cells, a vertex whose cell is on the map, carries
    /// no 0x10, is not dry-skipped (no 0x100 and `land` 1 or 2), carries no
    /// 0x1000 and whose `CoordInfo` lacks 0x4 takes the template's value
    /// added (`movss` template, `addss` height). The template's index
    /// counts every vertex of the box, skipped or not.
    fn place_cliff(&mut self, x: i32, y: i32, t: usize, heights: &[Single]) {
        let (n, off) = (INDEX_TO_SIZE[t], OFFTABLE[t]);
        let (vx0, vy0) = (4 * (x + off), 4 * (y + off));
        let w = self.stride();
        let mut k = 0;
        for vy in vy0..vy0 + n {
            for vx in vx0..vx0 + n {
                let (cx, cy) = (vx >> 2, vy >> 2);
                if self.in_cells(cx, cy) {
                    let ci = (cy * self.g.xs + cx) as usize;
                    let c = &self.g.cells[ci];
                    let dry = c.flags & 0x100 == 0 && (c.land == 1 || c.land == 2);
                    let skips = [
                        c.flags & 0x10 != 0,
                        dry,
                        c.flags & 0x1000 != 0,
                        self.coord[ci] & coord::HALF != 0,
                    ];
                    for (n, hit) in self.out.inert.box_skips.iter_mut().zip(skips) {
                        *n += usize::from(hit);
                    }
                    if c.flags & 0x10 == 0
                        && !dry
                        && c.flags & 0x1000 == 0
                        && self.coord[ci] & coord::HALF == 0
                    {
                        let v = &mut self.z[(vy * w + vx) as usize];
                        *v = heights[k].addss(*v);
                    }
                }
                k += 1;
            }
        }
    }

    /// **`Terrain::move_goody@0084a3d0`** on cell `(x, y)`
    /// ([`Pass::move_goody`], its good arm under the types' footprints);
    /// oil that moves takes its rocky patch's terraform on its new cell, as
    /// the troughs' does (`crate::map_troughs`).
    fn move_goody(&mut self, x: i32, y: i32) {
        self.out.goody_calls += 1;
        let mut pass =
            Pass::new(&mut *self.g, &mut *self.coord, &*self.z, self.combo).with_goods(self.goods);
        let to = pass.move_goody(x, y);
        self.out.inert.south_forest += pass.south_forest_refusals;
        self.out.goods.extend(pass.moved_goods.iter().copied());
        match to {
            Ok(Some((nx, ny))) => {
                self.out.moved.push(((x, y), (nx, ny)));
                crate::map_heights::terraform_oil(self.g, self.z, nx, ny);
            }
            Ok(None) => {}
            Err(_) => self.goody.push((x, y)),
        }
    }

    /// `assess_wcoord`'s block of a tile: the 0x4000 and its neighbours'
    /// 0x2000 (`World::set_blocked_at(.., 1)`), then the mask's `& 0xfffd |
    /// 1`, read after the call (listing 0x8a6b89..0x8a6ba3).
    fn block(&mut self, tx: i32, ty: i32) {
        set_blocked_at(self.g, tx, ty, true);
        let i = tile_at(self.g, tx, ty);
        self.out.inert.bit1_rewritten += usize::from(self.g.tiles[i] & 2 != 0);
        self.g.tiles[i] = (self.g.tiles[i] & 0xfffd) | 1;
    }

    /// [`Inert`]'s two measures of a handed tile: its span's distance from
    /// [`SLOPE`] and a corner past the clamp.
    fn measure(&mut self, tx: i32, ty: i32) {
        let w = self.stride();
        let at = |x: i32, y: i32| self.z[(y * w + x) as usize];
        let vs = [
            at(tx, ty),
            at(tx + 1, ty),
            at(tx, ty + 1),
            at(tx + 1, ty + 1),
        ];
        let (mut lo, mut hi) = (vs[0], vs[0]);
        for &v in &vs[1..] {
            (lo, hi) = (minss(v, lo), maxss(v, hi));
        }
        let span = hi.subss(lo);
        let d = (i64::from(span.0) - i64::from(SLOPE.0)).unsigned_abs();
        let d = u32::try_from(d).unwrap_or(u32::MAX);
        let i = &mut self.out.inert;
        i.nearest_ulps = if self.out.handed == 1 {
            d
        } else {
            i.nearest_ulps.min(d)
        };
        i.past_clamp += usize::from(vs.iter().any(|&v| v.gt(CLAMP) || CLAMP_NEG.gt(v)));
    }

    /// **`Cliffs::assess_wcoord@008a6a40`** on cell `(x, y)`.
    fn assess(&mut self, x: i32, y: i32) {
        let ci = (y * self.g.xs + x) as usize;
        if self.g.cells[ci].flags & 0x40 == 0 {
            return;
        }
        let tw = 4 * self.g.xs;
        let w = self.stride();
        let mut count = 0;
        for ty in 4 * y..4 * y + 4 {
            for tx in 4 * x..4 * x + 4 {
                let c = &self.g.cells[ci];
                let coastal = c.flags & COAST != 0 || c.land == 2 || c.land == 1;
                let mountain = self.g.tiles[tile_at(self.g, tx, ty)] & 3 == 2;
                self.out.inert.refused[0] += usize::from(coastal);
                self.out.inert.refused[1] += usize::from(mountain);
                if coastal || mountain {
                    continue;
                }
                self.out.handed += 1;
                self.measure(tx, ty);
                if !has_plus_slope(self.z, w, tx, ty) {
                    continue;
                }
                count += 1;
                self.out.slope.push((tx, ty));
                self.passed[(tx + tw * ty) as usize] = true;
                self.block(tx, ty);
                for &(dx, dy) in &MOVE_49[1..9] {
                    let (u, v) = (tx + dx, ty + dy);
                    if !tile_inb(self.g, u, v) {
                        continue;
                    }
                    self.beside[(u + tw * v) as usize] = true;
                    self.block(u, v);
                    let (cx, cy) = (u >> 2, v >> 2);
                    let ni = (cy * self.g.xs + cx) as usize;
                    self.cells[ni] = true;
                    self.move_goody(cx, cy);
                    if (cx, cy) != (x, y) && self.group[ni] == -1 {
                        self.group[ni] = self.group[ci];
                    }
                }
            }
        }
        if count != 0 {
            let before = self.out.moved.len() + self.out.goods.len();
            self.move_goody(x, y);
            let after = self.out.moved.len() + self.out.goods.len();
            self.out.inert.closing_moves += usize::from(after != before);
        } else {
            self.g.cells[ci].flags &= 0xffbf;
            self.out.cleared.push((x, y));
        }
    }

    /// `Cliffs::init`'s cell sweep (y outer): each cell a blocked
    /// neighbour touched builds a `Cliff`, whose `init` answers how many of
    /// its sixteen tiles are not a mountain's and sit in either tile mask;
    /// under a cliff index, a cell that answers any adds those tiles (y
    /// outer) to the cliff's mining list.
    fn mining(&mut self) {
        let tw = 4 * self.g.xs;
        for y in 0..self.g.ys {
            for x in 0..self.g.xs {
                let ci = (y * self.g.xs + x) as usize;
                if !self.cells[ci] {
                    continue;
                }
                let marked = |s: &Self, tx: i32, ty: i32| {
                    let i = (tx + tw * ty) as usize;
                    s.passed[i] || s.beside[i]
                };
                let mut n = 0;
                let mut tiles = Vec::new();
                for ty in 4 * y..4 * y + 4 {
                    for tx in 4 * x..4 * x + 4 {
                        if marked(self, tx, ty) {
                            tiles.push((tx, ty));
                            if self.g.tiles[tile_at(self.g, tx, ty)] & 3 != 2 {
                                n += 1;
                            }
                        }
                    }
                }
                let k = self.group[ci];
                self.out.inert.zero_count_cells +=
                    usize::from(k >= 0 && n == 0 && !tiles.is_empty());
                if k >= 0 && n != 0 {
                    self.out.mining[k as usize].extend(tiles);
                }
            }
        }
    }

    /// **`Cliffs::set_edge_flags@008a85e0`**: each cliff's mining tiles, in
    /// order, take 0x8000 where `WorldData::has_gather_access(.., −1, 0,
    /// 1)` passes.
    fn set_edge_flags(&mut self) {
        for k in 0..self.out.mining.len() {
            for j in 0..self.out.mining[k].len() {
                let (tx, ty) = self.out.mining[k][j];
                if crate::map_values::has_gather_access(self.g, tx, ty) {
                    let i = tile_at(self.g, tx, ty);
                    self.g.tiles[i] |= 0x8000;
                    self.out.edge += 1;
                }
            }
        }
    }
}

/// **`Cliffs::init@008a8020`** on a grid through `adjust_for_mountains`:
/// `templates` are `CliffsOut::init_templates`' in document order, which a
/// queued cliff indexes by size and then by its place among that size's;
/// `goods` the good types' footprints, by type.
pub fn init(
    g: &mut MapGrid,
    coord: &mut [u16],
    z: &mut [Single],
    templates: &[CliffTemplate],
    goods: &[Footprint],
    combo: Combo,
) -> Result<CliffsInit, Unported> {
    let by_size: [Vec<&CliffTemplate>; 5] =
        std::array::from_fn(|k| templates.iter().filter(|t| t.size == k).collect());
    let queued = g.cliffs.clone();
    let (ncells, ntiles) = ((g.xs * g.ys) as usize, (16 * g.xs * g.ys) as usize);
    let mut s = Init {
        g,
        coord,
        z,
        combo,
        goods,
        group: vec![-1; ncells],
        passed: vec![false; ntiles],
        beside: vec![false; ntiles],
        cells: vec![false; ncells],
        out: CliffsInit {
            mining: vec![Vec::new(); queued.len()],
            ..CliffsInit::default()
        },
        goody: Vec::new(),
    };
    for (k, &(x, y, t, i)) in queued.iter().enumerate() {
        let tpl = by_size[t][i];
        s.place_cliff(x, y, t, &tpl.heights);
        let off = OFFTABLE[t];
        for &(cx, cy) in &tpl.cells {
            let (u, v) = (x + off + cx, y + off + cy);
            if s.in_cells(u, v) {
                let ci = (v * s.g.xs + u) as usize;
                s.group[ci] = k as i32;
            }
        }
    }
    for y in 0..s.g.ys {
        for x in 0..s.g.xs {
            s.assess(x, y);
        }
    }
    s.mining();
    s.set_edge_flags();
    if !s.goody.is_empty() {
        return Err(Unported::Goody(s.goody));
    }
    Ok(s.out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`SLOPE`] is the original's arithmetic in native single precision:
    /// `25 · 0x3c8efa35`, `tanf`, `· 192` — the test oracle for the pin.
    #[test]
    fn the_slope_threshold_is_192_tanf_25_degrees() {
        let a = 25.0f32 * f32::from_bits(0x3c8e_fa35);
        let t = a.tan() * 192.0f32;
        assert_eq!(t.to_bits(), SLOPE.0, "{t}");
    }

    /// The span is strict and the first corner is clamped one way each: a
    /// tile whose corners span exactly the threshold fails, one ulp more
    /// passes; a flat tile at 20000 passes, its first corner's low clamp
    /// reading 10000 against the others' 20000, and so does one at −20000.
    #[test]
    fn the_slope_test_is_strict_and_clamps_the_first_corner() {
        let w = 2;
        let flat = |a: Single, b: Single| vec![a, b, Single::ZERO, Single::ZERO];
        assert!(!has_plus_slope(&flat(SLOPE, Single::ZERO), w, 0, 0));
        assert!(has_plus_slope(
            &flat(Single(SLOPE.0 + 1), Single::ZERO),
            w,
            0,
            0
        ));
        assert!(has_plus_slope(
            &flat(Single::ZERO, Single(SLOPE.0 + 1)),
            w,
            0,
            0
        ));
        for n in [20000, -20000] {
            let v = Single::from_i32(n);
            assert!(has_plus_slope(&[v; 4], w, 0, 0), "a flat tile at {n}");
        }
        let v = Single::from_i32(9000);
        assert!(
            !has_plus_slope(&[v; 4], w, 0, 0),
            "a flat tile inside the clamp"
        );
    }
}
