//! **The river troughs** (`docs/MAPGEN.md` §10, §15 step 8):
//! `RiversOut::calculate_all@00888820(0, 0)`, inside `Terrain::init`
//! between `make_rivers` (`map.cpp` 5118) and `adjust_for_mountains`.
//!
//! Each river [`crate::map_rivers::make_riverlist`] started
//! ([`MapGrid::rivers`]) is, in order:
//! 1. its control points (`RiverOut::add_spline_point@00885a40`, the cell
//!    centre in world units, 768 a cell) and their length in cells;
//! 2. `RiverOut::ensure_ocean_end@00885400`: up to two more points toward
//!    the nearest water or off-map cell ahead of the last one;
//! 3. `Spline::calc_spline@00912f00`: a clamped cubic B-spline of
//!    `length · 1.7` steps, and its normals;
//! 4. `RiverOut::create_sections@00884e00`: the spline walked in steps of
//!    `768 / 1.7`, each step a `RiverSection` whose three trough edges
//!    (`RiverSection::create_trough_edge@00889660`) lower the heights
//!    (`RiverSectionOut::terraform_from_river_vert@0088b360`) and mark the
//!    wet tiles beside them (`World::set_river_at`); then
//!    `TerrainOut::ensure_45_deg_slope@00869890` until it settles, and two
//!    `smooth_tcoord` passes over what the terraform touched;
//! 5. `Terrain::spot_update_land_batch@0084d540`: every terraformed cell
//!    becomes `BASELAND` of the tileset's `riverbase`
//!    ([`crate::halfland::Pass::spot_update_land_batch`]).
//!
//! Every operation is the original's SSE single precision, through
//! [`Single`]; the spline's `z` is zero throughout and is carried so the
//! signed zeros fall where the original's do. What the renderer alone reads
//! (the water planes, texture coordinates, the sections' own normals and
//! tangents) is not built.

use crate::halfland::{Combo, Pass, Unported as HalfUnported};
use crate::mapgen::{Fractal, MapGrid};
use crate::single::Single;
use crate::world::{MOVE_49, MOVE_289};

/// `0x44400000`: 768.0, a cell in world units.
const CELL: Single = Single(0x4440_0000);
const ONE: Single = Single(0x3f80_0000);
const TWO: Single = Single(0x4000_0000);
const FOUR: Single = Single(0x4080_0000);
const QUARTER: Single = Single(0x3e80_0000);
const EIGHTH: Single = Single(0x3e00_0000);
const ONE_HALF: Single = Single(0x3fc0_0000);
const MINUS_ONE: Single = Single(0xbf80_0000);
/// `__real_41f00000`: 30, the wet-tile deepening.
const THIRTY: Single = Single(0x41f0_0000);
/// `RiversOut::one_time_init@00888510`'s values, written after its XML.
/// `max_width` (+0x5c), each river's `max_width` and first `last_width`.
pub const MAX_WIDTH: Single = Single(0x3f40_0000);
/// `max_depth` (+0xfc): −60.
pub const MAX_DEPTH: Single = Single(0xc270_0000);
/// `sections_per_wcoord` (+0x100): 1.7.
pub const SECTIONS_PER_WCOORD: Single = Single(0x3fd9_999a);
/// `next_at_dist`'s minimum-step ratio, `0x3ea8f5c3` (0.33).
const STEP_RATIO: Single = Single(0x3ea8_f5c3);
/// The width fractal's scale, `__real_3ba3d70a` (0.005).
const WIDTH_SCALE: Single = Single(0x3ba3_d70a);
/// `ensure_ocean_end`'s first best distance, `0x49742400` (1e6).
const FAR: Single = Single(0x4974_2400);
/// `half_TCoordSize@00b21768` (79.68), which nothing writes.
pub const HALF_TCOORD: Single = Single(0x429f_5c29);
/// `generate_bspline`'s tail test is `(double)f > 0.001`; the largest
/// single below 0.001 is `0x3a83126e`, so it is `f > that`.
const TAIL: Single = Single(0x3a83_126e);
/// `create_trough_edge`'s depth profile across its nine vertices
/// (`0xb69870`, `0xb69ac0`, and a last 1.0).
const PROFILE: [Single; 9] = [
    Single(0x3f80_0000),
    Single(0x3f40_0000),
    Single(0x3e85_1eb8),
    Single(0),
    Single(0),
    Single(0),
    Single(0x3e85_1eb8),
    Single(0x3f40_0000),
    Single(0x3f80_0000),
];

/// A `Vector<float>` / `Point<float>`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V3 {
    pub x: Single,
    pub y: Single,
    pub z: Single,
}

impl V3 {
    fn new(x: Single, y: Single, z: Single) -> V3 {
        V3 { x, y, z }
    }

    fn sub(self, o: V3) -> V3 {
        V3::new(self.x.subss(o.x), self.y.subss(o.y), self.z.subss(o.z))
    }

    fn add(self, o: V3) -> V3 {
        V3::new(self.x.addss(o.x), self.y.addss(o.y), self.z.addss(o.z))
    }

    fn scale(self, s: Single) -> V3 {
        V3::new(self.x.mulss(s), self.y.mulss(s), self.z.mulss(s))
    }

    /// `(y² + x²) + z²`, the listings' order.
    fn len2(self) -> Single {
        self.y
            .mulss(self.y)
            .addss(self.x.mulss(self.x))
            .addss(self.z.mulss(self.z))
    }

    /// The length, its square root taken in double and stored as a
    /// single, which is `sqrtss`.
    fn len(self) -> Single {
        self.len2().sqrtss()
    }

    /// `Vector::norm@00420870`: unless the squared length is 0 or 1, each
    /// component times `1 / sqrt`.
    fn norm(self) -> V3 {
        let l = self.len2();
        if l.eq_value(Single::ZERO) || l.eq_value(ONE) {
            return self;
        }
        self.scale(ONE.divss(l.sqrtss()))
    }
}

/// `div_3_table[i]`: floor division by 3, negatives included
/// (`init_coord_lookup_array@00681db0`).
fn div3(i: i32) -> i32 {
    i.div_euclid(3)
}

/// A world coordinate's tile: `div_3_table[(int)v >> 6]`.
fn tile_of(v: Single) -> i32 {
    div3(v.to_i32() >> 6)
}

/// A world coordinate's cell: `div_3_table[(int)v >> 8]`.
fn cell_of(v: Single) -> i32 {
    div3(v.to_i32() >> 8)
}

/// A cell's centre in world units, `(float)(768·c + 384)`.
fn centre(c: i32) -> Single {
    Single::from_i32(768 * c + 384)
}

/// A tile mask's water bits: `mask & 0x30 == 0x20`.
fn wet(mask: u16) -> bool {
    mask & 0x30 == 0x20
}

/// `Spline` (type 3, a B-spline; flags 0x20, so `up` is +z): the control
/// points, the generated vertices, their normals and their total length.
#[derive(Clone, Debug, Default)]
pub struct Spline {
    pub control: Vec<V3>,
    pub verts: Vec<V3>,
    pub normals: Vec<V3>,
    pub total: Single,
    curr: Single,
    target: Single,
    scan: i32,
}

impl Spline {
    /// **`Spline::clamp_knots@00910be0`** (no weights): `n + degree + 1`
    /// knots, `degree + 1` zeros, then 1, 2, … to `n − degree`, held.
    fn knots(n: i32, degree: i32) -> Vec<Single> {
        (0..n + degree + 1)
            .map(|i| Single::from_i32((i - degree).clamp(0, n - degree)))
            .collect()
    }

    /// **`Spline::calc_bspline_basis@009116e0`**, by the listing.
    fn basis(k: &[Single], t: Single, i: usize, d: usize) -> Single {
        if i + d >= k.len() {
            return ONE;
        }
        if d == 0 {
            return if !k[i].gt(t) && k[i + 1].gt(t) {
                ONE
            } else {
                Single::ZERO
            };
        }
        let mut a = Single::ZERO;
        let (n1, d1) = (t.subss(k[i]), k[i + d].subss(k[i]));
        if n1.gt(Single::ZERO) && d1.gt(Single::ZERO) {
            a = Spline::basis(k, t, i, d - 1).mulss(n1.divss(d1));
        }
        let (n2, d2) = (k[i + d + 1].subss(t), k[i + d + 1].subss(k[i + 1]));
        if n2.gt(Single::ZERO) && d2.gt(Single::ZERO) {
            a = a.addss(Spline::basis(k, t, i + 1, d - 1).mulss(n2.divss(d2)));
        }
        a
    }

    /// **`Spline::calc_spline@00912f00`** for a B-spline of `degree` 3:
    /// the degree drops to `n − 1` on fewer points, the depth rises to
    /// the degree, then **`generate_bspline@00911820`** and, when it made
    /// any vertex, **`build_normals@00911f60`**.
    pub fn calc(&mut self, depth: i32) {
        let n = self.control.len() as i32;
        if n == 0 {
            return;
        }
        let degree = if n <= 3 { n - 1 } else { 3 };
        let depth = depth.max(degree);
        let k = Spline::knots(n, degree);
        let last = k.len() - 1;
        let mut delta = k[last].subss(k[0]).divss(Single::from_i32(depth));
        if delta.eq_value(Single::ZERO) {
            delta = ONE;
        }
        let mut t = k[0].addss(delta);
        self.verts.push(self.control[0]);
        let mut i = 0usize;
        while k[last].gt(t) {
            while i + 1 < k.len() && (t.gt(k[i + 1]) || !k[i + 1].gt(k[i])) {
                i += 1;
            }
            if i > last {
                break;
            }
            let mut s = V3::default();
            for m in i.saturating_sub(degree as usize)..=i {
                let b = Spline::basis(&k, t, m, degree as usize);
                s = s.add(self.control[m].scale(b));
            }
            self.verts.push(s);
            t = t.addss(delta);
        }
        if k[last].subss(t.subss(delta)).gt(TAIL) {
            self.verts.push(self.control[n as usize - 1]);
        }
        self.build_normals();
    }

    /// **`Spline::build_normals@00911f60`** with `up = (0, 0, 1)`: each
    /// segment's length summed into `total`, its normal `up × d`
    /// normalised, and at each vertex but the last the normalised sum of
    /// the segment's normal and the previous one's (none before the first);
    /// the last takes the last segment's own.
    fn build_normals(&mut self) {
        let up = V3::new(Single::ZERO, Single::ZERO, ONE);
        let mut prev = V3::default();
        for i in 1..self.verts.len() {
            let d = self.verts[i].sub(self.verts[i - 1]);
            self.total = self.total.addss(d.len());
            let n = V3::new(
                up.z.mulss(d.y).subss(up.y.mulss(d.z)),
                up.x.mulss(d.z).subss(up.z.mulss(d.x)),
                up.y.mulss(d.x).subss(up.x.mulss(d.y)),
            )
            .norm();
            self.normals.push(prev.add(n).norm());
            prev = n;
        }
        self.normals.push(prev);
    }

    /// `create_sections`' reset of the walk: at the start, or −1 with no
    /// length.
    fn rewind(&mut self) {
        let v = if self.verts.is_empty() || self.total.eq_value(Single::ZERO) {
            (MINUS_ONE, -1)
        } else {
            (Single::ZERO, 0)
        };
        (self.curr, self.target, self.scan) = (v.0, v.0, v.1);
    }

    fn seg(&self, s: usize) -> Single {
        self.verts[s].sub(self.verts[s - 1]).len()
    }

    /// **`Spline::next_at_dist@00912b70`**: the point and normal `step`
    /// further along, interpolated within a segment; 0 past the end.
    fn next_at_dist(&mut self, step: Single, ratio: Single) -> (V3, V3, bool) {
        if Single::ZERO.gt(self.curr) || self.scan < 0 {
            return (V3::default(), V3::default(), false);
        }
        let n = self.verts.len() as i32;
        self.scan += 1;
        self.target = self.target.addss(step);
        let min = step.mulss(ratio);
        let mut seg = MINUS_ONE;
        while self.scan < n && self.target.gt(self.curr) {
            seg = self.seg(self.scan as usize);
            self.scan += 1;
            self.curr = self.curr.addss(seg);
        }
        self.scan -= 1;
        let s = self.scan as usize;
        if self.curr.gt(self.target) && self.scan < n && !min.gt(self.total.subss(self.target)) {
            if Single::ZERO.gt(seg) {
                seg = self.seg(s);
            }
            let f = if seg.eq_value(Single::ZERO) {
                Single::ZERO
            } else {
                self.curr.subss(self.target).divss(seg)
            };
            let pos = self.verts[s].add(self.verts[s - 1].sub(self.verts[s]).scale(f));
            let nrm = self.normals[s]
                .add(self.normals[s - 1].sub(self.normals[s]).scale(f))
                .norm();
            return (pos, nrm, true);
        }
        let (pos, nrm) = (self.verts[s], self.normals[s]);
        if self.scan + 1 >= n || !self.total.gt(self.curr) {
            self.curr = self.total;
            self.scan = -1;
            return (pos, nrm, false);
        }
        (pos, nrm, true)
    }
}

/// What the troughs leave besides the heights.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Troughs {
    /// The rivers calculated, and the sections each made.
    pub sections: Vec<usize>,
    /// The cells `World::set_river_at` flagged 0x1000 that had not been.
    pub river_cells: usize,
    /// `verify_land`'s static depth after the last batch.
    pub depth: i32,
    /// The oil `move_goody` moved off a trough, from and to.
    pub moved: Vec<((i32, i32), (i32, i32))>,
}

/// The arms the troughs can enter that are not built.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unported {
    /// `Terrain::move_goody@0084a3d0` would move a good off these cells.
    Goody(Vec<(i32, i32)>),
    /// The land batch's `verify_land` refused (`crate::halfland`).
    Land(HalfUnported),
}

/// One river's state as `create_sections` and its callees keep it.
struct River<'a> {
    g: &'a mut MapGrid,
    coord: &'a mut [u16],
    combo: Combo,
    /// The oil `move_goody` moved, from and to.
    moved: Vec<((i32, i32), (i32, i32))>,
    z: &'a mut [Single],
    /// `spot_update_wcoords` (+0x284): every terraformed cell, repeats
    /// kept.
    spot: Vec<(i32, i32)>,
    /// `tcoords_to_smooth` (+0xb4) and its bitmask (+0x2a0), `x + 4·xs·y`.
    smooth: Vec<i32>,
    added: Vec<bool>,
    goody: Vec<(i32, i32)>,
}

impl River<'_> {
    fn stride(&self) -> i32 {
        4 * self.g.xs + 1
    }

    fn tile(&self, tx: i32, ty: i32) -> u16 {
        self.g.tiles[(ty * 4 * self.g.xs + tx) as usize]
    }

    fn in_tiles(&self, tx: i32, ty: i32) -> bool {
        let (tw, th) = self.g.tile_dims();
        tx >= 0 && ty >= 0 && tx < tw && ty < th
    }

    fn in_cells(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.g.xs && y < self.g.ys
    }

    /// **`RiverSectionOut::terraform_from_river_vert@0088b360`** at
    /// world `(x, y)`, its tile on the map: unless the tile is wet, its cell
    /// is `SANDY`, a neighbour cell is `SANDY`, `OCEAN` or HALFLAND, or the
    /// tile is a mountain's (`& 3 == 2`), the cell is listed for the land
    /// batch, the tile's four corners take `(z + depth) / 2`, and the
    /// vertices of the 11 × 11 round it (`move[0..121]`) are listed to
    /// smooth.
    fn terraform(&mut self, x: Single, y: Single, depth: Single) {
        let (tx, ty) = (tile_of(x), tile_of(y));
        let (cx, cy) = (tx >> 2, ty >> 2);
        let mask = self.tile(tx, ty);
        let xs = self.g.xs;
        let held = wet(mask) || self.g.cells[(cy * xs + cx) as usize].land == 1;
        let beside = MOVE_49[1..9].iter().any(|&(dx, dy)| {
            let (u, v) = (cx + dx, cy + dy);
            if !self.in_cells(u, v) {
                return false;
            }
            let c = &self.g.cells[(v * xs + u) as usize];
            c.land == 2 || c.land == 1 || c.flags & 0x100 != 0
        });
        if beside || held || mask & 3 == 2 {
            return;
        }
        self.spot.push((cx, cy));
        let w = self.stride();
        for (u, v) in [(tx, ty), (tx + 1, ty), (tx, ty + 1), (tx + 1, ty + 1)] {
            let h = &mut self.z[(v * w + u) as usize];
            *h = h.addss(depth).mulss(Single::HALF);
        }
        for &(dx, dy) in &MOVE_289[..121] {
            let (u, v) = (tx + dx, ty + dy);
            if self.in_tiles(u, v) {
                let i = u + 4 * xs * v;
                if !self.added[i as usize] {
                    self.added[i as usize] = true;
                    self.smooth.push(i);
                }
            }
        }
    }

    /// `create_trough_edge`'s 3 × 3 round tile `(tx, ty)`: a tile with a
    /// corner strictly below zero, of a cell free of 0x10, 0x28, 0x800 and
    /// 0x40, HALFLAND or neither `SANDY` nor `OCEAN`, not blocked and with
    /// `CoordInfo` free of 0x4, goes through `World::set_river_at`: the
    /// tile 0x800, the cell 0x1000.
    fn mark_wet(&mut self, tx: i32, ty: i32, marked: &mut usize) {
        let w = self.stride();
        let xs = self.g.xs;
        for &(dx, dy) in &MOVE_49[..9] {
            let (u, v) = (tx + dx, ty + dy);
            if !self.in_tiles(u, v) {
                continue;
            }
            let below = [(u, v), (u + 1, v), (u, v + 1), (u + 1, v + 1)]
                .iter()
                .any(|&(a, b)| Single::ZERO.gt(self.z[(b * w + a) as usize]));
            if !below {
                continue;
            }
            let ci = ((v >> 2) * xs + (u >> 2)) as usize;
            let c = self.g.cells[ci];
            if c.flags & (0x10 | 0x28 | 0x800) != 0 {
                continue;
            }
            if c.flags & 0x100 == 0 && (c.land == 2 || c.land == 1) {
                continue;
            }
            if c.flags & 0x40 != 0 || c.blocked != 0 || self.coord[ci] & 0x4 != 0 {
                continue;
            }
            self.g.tiles[(v * 4 * xs + u) as usize] |= 0x800;
            if self.g.cells[ci].flags & 0x1000 == 0 {
                *marked += 1;
            }
            self.g.cells[ci].flags |= 0x1000;
        }
    }

    /// **`Terrain::move_goody@0084a3d0`** on cell `(x, y)`
    /// ([`Pass::move_goody`]); oil that moves takes its rocky patch's
    /// terraform on its new cell. A good refuses by name.
    fn move_goody(&mut self, x: i32, y: i32) {
        let to = Pass::new(&mut *self.g, &mut *self.coord, &*self.z, self.combo).move_goody(x, y);
        match to {
            Ok(Some((nx, ny))) => {
                self.moved.push(((x, y), (nx, ny)));
                crate::map_heights::terraform_oil(self.g, self.z, nx, ny);
            }
            Ok(None) => {}
            Err(_) => self.goody.push((x, y)),
        }
    }

    /// **`RiverSection::create_trough_edge@00889660`** across `o` at `c`,
    /// `width` wide: nine vertices a step apart, centred on `c`, each
    /// lowered to the profile's depth.
    fn edge(&mut self, c: V3, o: V3, width: Single, marked: &mut usize) {
        let span = |v: Single| width.mulss(v).mulss(TWO).mulss(CELL).mulss(EIGHTH);
        let (sx, sy) = (span(o.x), span(o.y));
        let mut x = c.x.subss(sx.mulss(FOUR));
        let mut y = c.y.subss(sy.mulss(FOUR));
        let dip = MAX_DEPTH.mulss(QUARTER).mulss(MINUS_ONE);
        let edge = Single::from_i32(768 * self.g.xs);
        for (i, p) in PROFILE.iter().enumerate() {
            let mut depth = MAX_DEPTH.subss(p.mulss(MAX_DEPTH)).addss(dip);
            let (tx, ty) = (tile_of(x), tile_of(y));
            if self.in_tiles(tx, ty) {
                if wet(self.tile(tx, ty)) && Single::ZERO.gt(depth) && (i as i32 - 4).abs() < 2 {
                    depth = depth.subss(THIRTY).addss(depth).mulss(Single::HALF);
                }
                self.terraform(x, y, depth);
                self.mark_wet(tx, ty, marked);
                self.move_goody(cell_of(x), cell_of(y));
            } else {
                if Single::ZERO.gt(y) {
                    y = Single::ZERO;
                }
                if Single::ZERO.gt(x) {
                    x = Single::ZERO;
                }
                if y.gt(edge) {
                    y = edge;
                }
                if x.gt(edge) {
                    x = edge;
                }
            }
            y = y.addss(sy);
            x = x.addss(sx);
        }
    }

    /// **`RiverSection::init@0088c360`** →
    /// **`create_trough_and_water@00889f20`**: the edges at the first
    /// end, the midpoint (the orthogonals' mean, not normalised) and the
    /// second end.
    fn section(&mut self, a: (V3, V3, Single), b: (V3, V3, Single), marked: &mut usize) {
        self.edge(a.0, a.1, a.2, marked);
        let mid = b.0.add(a.0).scale(Single::HALF);
        let o = b.1.add(a.1).scale(Single::HALF);
        self.edge(mid, o, a.2.addss(b.2).mulss(Single::HALF), marked);
        self.edge(b.0, b.1, b.2, marked);
    }

    /// **`TerrainOut::ensure_45_deg_slope@00869890`**: each listed index's
    /// 2 × 2 vertices; a neighbour (3 × 3, its tile neither a mountain's,
    /// `& 3 == 3`, nor wet) more than 79.68 above one comes down to 4 under
    /// that, when that is above zero, and is listed for the next call.
    fn ensure_45(&mut self, list: &mut Vec<i32>) {
        let todo = std::mem::take(list);
        let tw = 4 * self.g.xs;
        let w = self.stride();
        let mut added = vec![false; self.added.len()];
        for idx in todo {
            let (x0, y0) = (idx % tw, idx / tw);
            for y in y0..y0 + 2 {
                for x in x0..x0 + 2 {
                    if !self.in_tiles(x, y) {
                        continue;
                    }
                    let h = self.z[(y * w + x) as usize];
                    for ny in y - 1..y + 2 {
                        for nx in x - 1..x + 2 {
                            if !self.in_tiles(nx, ny) {
                                continue;
                            }
                            let m = self.tile(nx, ny);
                            if m & 3 == 3 || wet(m) || m & 3 == 2 {
                                continue;
                            }
                            let zi = (ny * w + nx) as usize;
                            let to = HALF_TCOORD.addss(h).subss(FOUR);
                            if self.z[zi].subss(h).gt(HALF_TCOORD) && to.gt(Single::ZERO) {
                                self.z[zi] = to;
                                let n = nx + tw * ny;
                                if !added[n as usize] {
                                    added[n as usize] = true;
                                    list.push(n);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// **`RiverOut::ensure_ocean_end@00885400`** on a river's control points,
/// three or more: the direction in from the second-to-last point
/// (normalised; from the third-to-last, raw, when that is zero in x and
/// y), then the nearest of the end cell's 7 × 7 (`move[0..49]`) that is
/// off the map or `OCEAN` and not behind it becomes a point, and the cell a
/// step on from it, along the last candidate's offset less the direction,
/// another when it too is on the map and `OCEAN`.
fn ensure_ocean_end(g: &MapGrid, pts: &mut Vec<(i32, i32)>, cd: &mut Single) {
    let n = pts.len();
    if n < 3 {
        return;
    }
    let p = |k: usize| V3::new(centre(pts[k].0), centre(pts[k].1), Single::ZERO);
    let last = p(n - 1);
    let mut dir = last.sub(p(n - 2)).norm();
    if dir.x.eq_value(Single::ZERO) {
        if dir.y.eq_value(Single::ZERO) {
            dir = last.sub(p(n - 3));
        }
        if dir.x.eq_value(Single::ZERO) && dir.y.eq_value(Single::ZERO) {
            return;
        }
    }
    let (ex, ey) = (cell_of(last.x), cell_of(last.y));
    let mut best = FAR;
    let mut found = None;
    let mut v = V3::default();
    for &(dx, dy) in &MOVE_49 {
        let (cx, cy) = (ex + dx, ey + dy);
        let off = cx < 0 || cy < 0 || cx >= g.xs || cy >= g.ys;
        if !off && g.cells[(cy * g.xs + cx) as usize].land != 2 {
            continue;
        }
        v = V3::new(
            centre(cx).subss(last.x),
            centre(cy).subss(last.y),
            Single::ZERO,
        );
        let dot =
            v.y.mulss(dir.y)
                .addss(v.x.mulss(dir.x))
                .addss(dir.z.mulss(Single::ZERO));
        if Single::ZERO.gt(dot) {
            continue;
        }
        let dist =
            v.y.mulss(v.y)
                .addss(v.x.mulss(v.x))
                .addss(Single::ZERO)
                .sqrtss();
        if best.gt(dist) {
            best = dist;
            found = Some((cx, cy));
        }
    }
    let Some((bx, by)) = found else {
        return;
    };
    add_point(pts, cd, (bx, by));
    let mut w = V3::new(
        v.x.subss(dir.x),
        v.y.subss(dir.y),
        Single::ZERO.subss(dir.z),
    );
    let l = w.len2();
    if !l.eq_value(Single::ZERO) && !l.eq_value(ONE) {
        let inv = ONE.divss(l.sqrtss());
        w = V3::new(w.x.mulss(inv), w.y.mulss(inv), w.z);
    }
    let sx = cell_of(centre(bx).addss(w.x.mulss(CELL)));
    let sy = cell_of(centre(by).addss(w.y.mulss(CELL)));
    if sx >= 0 && sy >= 0 && sx < g.xs && sy < g.ys && g.cells[(sy * g.xs + sx) as usize].land == 2
    {
        add_point(pts, cd, (sx, sy));
    }
}

/// **`RiverOut::add_spline_point@00885a40`**'s length: each point after
/// the first adds its distance from the one before, in cells.
fn add_point(pts: &mut Vec<(i32, i32)>, cd: &mut Single, c: (i32, i32)) {
    if let Some(&(px, py)) = pts.last() {
        let dx = centre(px).subss(centre(c.0));
        let dy = centre(py).subss(centre(c.1));
        let d = dy.mulss(dy).addss(dx.mulss(dx)).sqrtss();
        *cd = d.divss(CELL).addss(*cd);
    }
    pts.push(c);
}

/// **`RiversOut::calculate_all@00888820(0, 0)`** on `g`'s rivers, the
/// heights `z` and the `CoordInfo` flags `coord`; `riverbase` is the drawn
/// tileset's (`TileSet+0x229`), `combo` and `depth` the land pass's.
pub fn calculate_all(
    g: &mut MapGrid,
    coord: &mut [u16],
    z: &mut [Single],
    riverbase: i32,
    combo: Combo,
    depth: i32,
) -> Result<Troughs, Unported> {
    let mut out = Troughs {
        depth,
        ..Troughs::default()
    };
    let width = Fractal::init(50, 50, 2, 2, 0);
    let step = CELL.divss(SECTIONS_PER_WCOORD);
    let rivers = std::mem::take(&mut g.rivers);
    let mut goody = Vec::new();
    for cells in &rivers {
        // The points as `make_riverlist` added them, then the ocean end.
        let mut pts = Vec::new();
        let mut cd = Single::ZERO;
        for &c in cells {
            add_point(&mut pts, &mut cd, c);
        }
        ensure_ocean_end(g, &mut pts, &mut cd);
        let mut spline = Spline {
            control: pts
                .iter()
                .map(|&(x, y)| V3::new(centre(x), centre(y), Single::ZERO))
                .collect(),
            ..Spline::default()
        };
        // `depth` is a `u16` field; no river is 38,000 cells long.
        let steps = cd.mulss(SECTIONS_PER_WCOORD).to_i32() as u16;
        spline.calc(i32::from(steps));
        let mut r = River {
            coord: &mut *coord,
            combo,
            moved: Vec::new(),
            z: &mut *z,
            spot: Vec::new(),
            smooth: Vec::new(),
            added: vec![false; (16 * g.xs * g.ys) as usize],
            goody: Vec::new(),
            g: &mut *g,
        };
        let mut made = 0;
        if spline.verts.len() > 1 {
            spline.rewind();
            let (mut pos, mut nrm, _) = spline.next_at_dist(Single::ZERO, STEP_RATIO);
            let (mut fx, mut fy) = (0, 0);
            let mut last_width = MAX_WIDTH;
            let mut delta = false;
            loop {
                fx += 1;
                if fx > 49 {
                    fy += 1;
                    fx = 0;
                    if fy > 49 {
                        fy = 0;
                    }
                }
                let last = last_width;
                let mut w = if delta {
                    let wide = MAX_WIDTH.addss(MAX_WIDTH);
                    let grow = last.mulss(ONE_HALF);
                    if !grow.gt(wide) { grow } else { wide }
                } else {
                    let b = Single::from_i32(i32::from(width.get_height(fx, fy)));
                    b.mulss(WIDTH_SCALE).addss(Single::HALF).mulss(MAX_WIDTH)
                };
                let prev = (pos, nrm);
                last_width = w;
                let (p2, n2, more) = spline.next_at_dist(step, STEP_RATIO);
                (pos, nrm) = (p2, n2);
                if !delta {
                    let tx = tile_of(pos.x.addss(prev.0.x).mulss(Single::HALF));
                    let ty = tile_of(pos.y.addss(prev.0.y).mulss(Single::HALF));
                    if r.in_tiles(tx, ty) && wet(r.tile(tx, ty)) {
                        delta = true;
                        w = w.mulss(ONE_HALF);
                        last_width = w;
                    }
                }
                r.section((prev.0, prev.1, last), (pos, nrm, w), &mut out.river_cells);
                made += 1;
                if !more {
                    break;
                }
            }
            let mut list = r.smooth.clone();
            let smooth = list.clone();
            while !list.is_empty() {
                r.ensure_45(&mut list);
            }
            let tw = 4 * r.g.xs;
            for _ in 0..2 {
                for &i in &smooth {
                    crate::map_heights::smooth_tcoord(r.g, r.z, i % tw, i / tw);
                }
            }
        }
        out.sections.push(made);
        goody.append(&mut r.goody);
        out.moved.append(&mut r.moved);
        let spot = std::mem::take(&mut r.spot);
        let mut p = Pass::new(g, coord, &*z, combo).with_depth(out.depth);
        p.spot_update_land_batch(&spot, 0, riverbase)
            .map_err(Unported::Land)?;
        out.depth = p.depth;
    }
    g.rivers = rivers;
    if !goody.is_empty() {
        return Err(Unported::Goody(goody));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The knots of a clamped cubic on five points: four zeros, 1, 2,
    /// then 2 held to the ninth.
    #[test]
    fn the_knots_are_clamped() {
        let k: Vec<i32> = Spline::knots(5, 3).iter().map(|s| s.to_i32()).collect();
        assert_eq!(k, vec![0, 0, 0, 0, 1, 2, 2, 2, 2]);
    }

    /// A straight spline stays on its line, starts and ends on its end
    /// points, and its normals are the line's left-hand perpendicular.
    #[test]
    #[allow(clippy::float_arithmetic)]
    fn a_straight_spline_stays_straight() {
        let mut s = Spline {
            control: (0..5)
                .map(|i| V3::new(centre(i), centre(2), Single::ZERO))
                .collect(),
            ..Spline::default()
        };
        s.calc(8);
        assert_eq!(s.verts.first(), s.control.first());
        assert_eq!(s.verts.last(), s.control.last());
        assert!(s.verts.iter().all(|v| v.y == centre(2)));
        assert!(s.verts.windows(2).all(|w| w[1].x.gt(w[0].x)));
        assert_eq!(s.normals.len(), s.verts.len());
        for n in &s.normals {
            assert_eq!((n.x.eq_value(Single::ZERO), n.y), (true, MINUS_ONE));
        }
        assert_eq!(s.total, Single::from_i32(4 * 768));
    }

    /// The walk visits the start, then steps of `step` until the end.
    #[test]
    fn next_at_dist_steps_to_the_end() {
        let mut s = Spline {
            control: (0..4)
                .map(|i| V3::new(centre(i), centre(0), Single::ZERO))
                .collect(),
            ..Spline::default()
        };
        s.calc(6);
        s.rewind();
        let (p, _, more) = s.next_at_dist(Single::ZERO, STEP_RATIO);
        assert_eq!((p, more), (s.verts[0], true));
        let mut n = 0;
        loop {
            let (_, _, more) = s.next_at_dist(Single::from_i32(500), STEP_RATIO);
            n += 1;
            if !more {
                break;
            }
            assert!(n < 100);
        }
        // 2,304 world units in steps of 500: four whole, the last short.
        assert_eq!(n, 5);
    }
}
