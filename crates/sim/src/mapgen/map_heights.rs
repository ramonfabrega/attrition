//! **Map generation's heights** (`docs/MAPGEN.md` §10, §11, §15 step 7):
//! the corner grid `TerrainData::master_land_heights` that `Terrain::init`
//! builds between `Terrain.cpp` 699 and 894, in the original's own SSE
//! singles ([`Single`]) and, for the two fractals' bilinear sample, its
//! doubles ([`crate::double::Double`]).
//!
//! Three passes write it on a map without water:
//! - **`TerrainOut::generate_land@0085f8d0`**, one vertex at a time, y
//!   outer and x inner, through
//!   `TerrainOut::determine_land_height_color@0086c1e0` (which returns in
//!   xmm0, the decompile drops it): a river vertex 0, a vertex whose
//!   `CoordInfo` codes carry 0x1000 `(−303 + land_height) · 1`, 0x4 zero,
//!   0x2 100, and any other `(a · 7.5 + land_height) + b · 1.875`, `a` and
//!   `b` the two fractals' samples ([`fractals`]).
//! - **`TerrainOil::create_doobers@008a28e0`**, each oil cell's 4×4 tiles
//!   flattened by `terraform_for_building` ([`create_doobers`]).
//! - **`TerrainOut::adjust_for_mountains@008703c0`**, each placed mountain
//!   adding its template's displacement to the vertices of its cells
//!   ([`adjust_for_mountains`]).
//!
//! On a map with water two more write it between the first two
//! (item 3835): `generate_land`'s **shore blend** and its
//! `TerrainOut::smooth_tcoord@0086c3a0` list ([`find_closest_coordinfo`],
//! [`smooth_tcoord`]), and **`TerrainOut::build_coastal_data@0086c540`**
//! ([`build_coastal_data`]).
//!
//! What no corpus map reaches is refused by name, never guessed
//! ([`Unported`]): `Cliffs::assess_wcoord`'s slope test.

use std::collections::BTreeMap;

use crate::mapgen::{COAST, Fractal, MapGrid};
use crate::single::Single;
use crate::terrain_groups::Template;
use crate::terrain_groups::flag::{OIL, ROCK};
use crate::terrain_init::{MOVE, coord};

/// `TerrainOut+0x4b7c`: vertices per cell each way.
pub const VERTS: i32 = 4;
/// The `OIL` good type (`Objects::init_good`'s type 5).
const OIL_GOOD: i32 = 5;
/// `land_height` (0xcbe54c), which `Terrain::init@00850f70` sets to 30.0f
/// before `generate_land`.
pub const LAND_HEIGHT: Single = Single(0x41f0_0000);
/// `TerrainOut+0x5630`, −303.0f, set beside it.
pub const SEA_FLOOR: Single = Single(0xc397_8000);
/// The first fractal's weight (0xb69610).
const A_WEIGHT: Single = Single(0x40f0_0000);
/// The second fractal's (0xb695a0).
const B_WEIGHT: Single = Single(0x3ff0_0000);
/// A 0x2 vertex's height (0xb696ac).
const BANK: Single = Single(0x42c8_0000);
/// The 0x1000 arm's multiplier (0xc0629c).
const ONE: Single = Single(0x3f80_0000);
/// `MountainRange::init`'s `height · 3.0f` (0xb695c8, at 0x899968).
const THREE: Single = Single(0x4040_0000);
/// And its `/ 255.0f` (0xb696d4, at 0x899dbc).
const BYTE_MAX: Single = Single(0x437f_0000);

/// The `CoordInfo` codes `TerrainOut::get_vert_codes@0086bf70` ORs from a
/// vertex's cells (`param_3 == 0`).
pub mod code {
    /// `OCEAN`: the sea floor's arm.
    pub const OCEAN: u16 = 0x1000;
    /// `NONE`, HALFLAND or 0x400: zero.
    pub const HALF: u16 = 0x4;
    /// A mountain cell, which `adjust_for_mountains` marks: 100.
    pub const MOUNTAIN: u16 = 0x2;
}

/// The arms a map can enter that are not built.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unported {
    /// `Cliffs::assess_wcoord@008a6a40`'s slope test,
    /// `TerrainOut::has_plus_slope@00869790`: a tile of a cell flagged
    /// 0x40 that is not COAST, `land` 1 or 2, nor a mountain tile. The
    /// tiles named. Built in [`crate::cliffs`] (item 3890), which a caller
    /// runs after [`adjust_for_mountains`] instead of [`heights`].
    Slope(Vec<(i32, i32)>),
}

/// The two fractals `TerrainOut::refresh_data@00870050` builds for the
/// heights, on `4·xs + 1` by `4·ys + 1` points with flags 2 (wrapped, and
/// `get_height` scaled by 100/256): smoothing 5 seeded with `World.seed`
/// (+0x7c), and smoothing 3 seeded with twice it. A third fractal it builds
/// beside them, and the `texture_fractal`, feed no height.
///
/// `refresh_data` builds them only when the map's size differs from the
/// process's last (`TerrainData::last_map_size`), and a `semaphore` bit
/// drops both smoothings to 0 and −2; a fresh process with the bit clear
/// is the case modelled (§10's quirks).
pub fn fractals(xs: i32, ys: i32, seed: u32) -> (Fractal, Fractal) {
    let (w, h) = (VERTS * xs + 1, VERTS * ys + 1);
    (
        Fractal::init(w, h, 5, 2, seed),
        Fractal::init(w, h, 3, 2, seed.wrapping_mul(2)),
    )
}

/// `TerrainOut::get_vert_codes@0086bf70(x, y, 0)`: the `CoordInfo` flags
/// of the vertex's cell, OR-ed with the cell to the left when `x` is on a
/// cell edge, the cell above when `y` is, and all four at a corner.
pub fn vert_codes(g: &MapGrid, coord: &[u16], x: i32, y: i32) -> u16 {
    let (cx, cy) = (x / VERTS, y / VERTS);
    let at = |u: i32, v: i32| {
        if u >= 0 && v >= 0 && u < g.xs && v < g.ys {
            coord[(v * g.xs + u) as usize]
        } else {
            0
        }
    };
    let (own, left, up, corner) = (
        at(cx, cy),
        at(cx - 1, cy),
        at(cx, cy - 1),
        at(cx - 1, cy - 1),
    );
    let mut r = own;
    if x % VERTS == 0 {
        if y % VERTS == 0 {
            return own | left | up | corner;
        }
        r = own | left;
    }
    if y % VERTS == 0 {
        return r | own | up;
    }
    r
}

/// `get_vert_codes(x, y, 1) & 0x10`: a river tile (mask 0x800) among the
/// four tiles meeting at the vertex. `generate_land`'s first pass sets the
/// vertex's bit for it, and `determine_land_height_color` answers 0 there.
pub fn river_vertex(g: &MapGrid, x: i32, y: i32) -> bool {
    let (tw, th) = g.tile_dims();
    [(x, y), (x - 1, y), (x, y - 1), (x - 1, y - 1)]
        .iter()
        .any(|&(u, v)| {
            u >= 0 && v >= 0 && u < tw && v < th && g.tiles[(v * tw + u) as usize] & 0x800 != 0
        })
}

/// **`generate_land`'s heights**: `(4·ys + 1)` rows of `(4·xs + 1)`
/// singles, row-major, as the start dump prints them — each vertex through
/// `determine_land_height_color` (y outer, x inner), then the vertices its
/// shore blend listed through [`smooth_tcoord`], in list order: once, and
/// a second time when either coordinate is odd (0x85fec0..0x85ff1a).
///
/// The `Prefs` value that flattens every height to 0 is taken as unset.
pub fn generate_land(g: &MapGrid, coord: &[u16], fractals: &(Fractal, Fractal)) -> Vec<Single> {
    let (w, h) = (VERTS * g.xs + 1, VERTS * g.ys + 1);
    let mut z = Vec::with_capacity((w * h) as usize);
    let mut smooth = Vec::new();
    for y in 0..h {
        for x in 0..w {
            z.push(if river_vertex(g, x, y) {
                Single::ZERO
            } else {
                vertex_height(g, coord, fractals, x, y, &mut smooth)
            });
        }
    }
    for (x, y) in smooth {
        smooth_tcoord(g, &mut z, x, y);
        if (x | y) & 1 != 0 {
            smooth_tcoord(g, &mut z, x, y);
        }
    }
    z
}

/// `oo_WCoordSize` (0xee1864, set on first use to `0x3aaaaaab`): one over
/// a cell's 768 world units.
const OO_WCOORD: Single = Single(0x3aaa_aaab);
/// `MinPolySize` (0xb21764): a vertex's 192 world units.
const POLY: Single = Single(0x4340_0000);
/// `find_closest_coordinfo`'s starting best (0xb6976c), 1e20.
const FAR: Single = Single(0x60ad_78ec);
/// Its answer when no 0x20 cell is in reach (0xb69798).
const NONE_NEAR: Single = Single(0xbf80_0000);
/// The shore blend's constants (0x86c2ff..0x86c35a): 4, 0.05, 0.25.
const FOUR: Single = Single(0x4080_0000);
const BLEND_RAMP: Single = Single(0x3d4c_cccd);
const QUARTER: Single = Single(0x3e80_0000);
/// `radius[]` (0xadd1e0): the end of each ring of `move_x/move_y`.
const RADIUS: [usize; 4] = [1, 9, 25, 49];

/// **`TerrainOut::find_closest_coordinfo@0086a260(x, y, 4)`**: the
/// distance, in cells, from vertex `(x, y)` to the nearest vertex of a
/// `CoordInfo` 0x20 cell, searched ring by ring round the vertex's cell
/// (`move[0]`, then `[1..9]`, `[9..25]`, `[25..49]`) and stopping two
/// rings past the first ring that finds one; −1.0 when none is in reach.
/// Each candidate vertex's squared distance is `(vx·192 − x·192)² + (vy·192
/// − y·192)²` in singles, kept when strictly smaller; the answer is
/// `sqrtf` of the best times `oo_WCoordSize`.
pub fn find_closest_coordinfo(g: &MapGrid, coord: &[u16], x: i32, y: i32) -> Single {
    let (fx, fy) = (
        Single::from_i32(x).mulss(POLY),
        Single::from_i32(y).mulss(POLY),
    );
    let (cx, cy) = (x >> 2, y >> 2);
    let mut best = FAR;
    let mut found = false;
    let mut more = 0;
    let mut start = 0;
    for &end in &RADIUS {
        if found && more == 0 {
            break;
        }
        more = if more == 0 { 0 } else { more - 1 };
        let was = found;
        for &(dx, dy) in &MOVE[start..end] {
            let (u, v) = (cx + dx, cy + dy);
            if u < 0 || v < 0 || u >= g.xs || v >= g.ys {
                continue;
            }
            if coord[(v * g.xs + u) as usize] & coord::COASTAL == 0 {
                continue;
            }
            for j in 0..=VERTS {
                let ey = Single::from_i32(VERTS * v + j).mulss(POLY).subss(fy);
                let ey = ey.mulss(ey);
                for i in 0..=VERTS {
                    let ex = Single::from_i32(VERTS * u + i).mulss(POLY).subss(fx);
                    let d = ex.mulss(ex).addss(ey);
                    if best.gt(d) {
                        best = d;
                        found = true;
                    }
                }
            }
        }
        if found && !was {
            more = 2;
        }
        start = end;
    }
    if found {
        best.sqrtss().mulss(OO_WCOORD)
    } else {
        NONE_NEAR
    }
}

/// `determine_land_height_color` past its river test (0x86c233..0x86c38f):
/// the codes' arms, then the fractals' height, blended toward the shore
/// when [`find_closest_coordinfo`] answers `d ≥ 0` —
/// `0.25·(d·h) + ((d·0.05·7.5·192 + land_height)·(4 − d))·0.25` — which
/// also lists the vertex for smoothing when both its coordinates are under
/// `4·xs` (the listing compares `y` with `4·xs` too).
fn vertex_height(
    g: &MapGrid,
    coord: &[u16],
    (a, b): &(Fractal, Fractal),
    x: i32,
    y: i32,
    smooth: &mut Vec<(i32, i32)>,
) -> Single {
    let codes = vert_codes(g, coord, x, y);
    if codes & code::OCEAN != 0 {
        return SEA_FLOOR.addss(LAND_HEIGHT).mulss(ONE);
    }
    if codes & code::HALF != 0 {
        return Single::ZERO;
    }
    if codes & code::MOUNTAIN != 0 {
        return BANK;
    }
    let h = Single::from_i32(i32::from(a.get_height(x, y)))
        .mulss(A_WEIGHT)
        .addss(LAND_HEIGHT);
    let h = h.addss(Single::from_i32(i32::from(b.get_height(x, y))).mulss(B_WEIGHT));
    let d = find_closest_coordinfo(g, coord, x, y);
    if Single::ZERO.gt(d) {
        return h;
    }
    let rise = d
        .mulss(BLEND_RAMP)
        .mulss(A_WEIGHT)
        .mulss(POLY)
        .addss(LAND_HEIGHT)
        .mulss(FOUR.subss(d))
        .mulss(QUARTER);
    let r = QUARTER.mulss(d.mulss(h)).addss(rise);
    let n = VERTS * g.xs;
    if x < n && y < n {
        smooth.push((x, y));
    }
    r
}

/// **`TerrainOut::smooth_tcoord@0086c3a0(x, y)`**: the 4×4 vertices from
/// `(x − 1, y − 1)` copied (those under the tile grid's bounds; the rest
/// read 0), then each of the 2×2 from `(x, y)` (row outer) whose height is
/// above 0 takes the mean of the copy's non-zero values in its 3×3, summed
/// row by row in singles.
pub fn smooth_tcoord(g: &MapGrid, z: &mut [Single], x: i32, y: i32) {
    let (tw, th) = g.tile_dims();
    let stride = VERTS * g.xs + 1;
    let mut buf = [Single::ZERO; 16];
    for j in 0..4 {
        for i in 0..4 {
            let (u, v) = (x - 1 + i, y - 1 + j);
            if u >= 0 && v >= 0 && u < tw && v < th {
                buf[(i + 4 * j) as usize] = z[(v * stride + u) as usize];
            }
        }
    }
    for r in 0..2 {
        for c in 0..2 {
            let mut sum = Single::ZERO;
            let mut n = 0;
            for j in 0..3 {
                for i in 0..3 {
                    let v = buf[(c + i + 4 * (r + j)) as usize];
                    if !v.eq_value(Single::ZERO) {
                        n += 1;
                        sum = sum.addss(v);
                    }
                }
            }
            let at = ((y + r) * stride + x + c) as usize;
            if n != 0 && z[at].gt(Single::ZERO) {
                z[at] = sum.divss(Single::from_i32(n));
            }
        }
    }
}

/// One cell's `InitCoastInfo` (size 0x28): what
/// `build_coastal_data`'s passes leave on it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CoastInfo {
    /// `+0x0`, `is_valid`: a 0x20 cell `mark_valid_coastal_influencers`
    /// keeps.
    pub valid: bool,
    /// `+0x1`, `has_influence`.
    pub influence: bool,
    /// `+0x2`, `connected_out`: how many of its neighbours it joined, at
    /// most 2. (`+0x3`, `connected_in`, which the join sets on the
    /// neighbour, no height reads and is not carried.)
    pub out: u8,
    /// `+0x4` and `+0x8`, `connected_to` and `connected_to2`.
    pub to: [(i32, i32); 2],
    /// `+0xc`, `influence`: tile indices `x + 4·xs·y`, in append order.
    pub tiles: Vec<i32>,
}

/// `TerrainOut::mark_coast_vert_influence`'s `texture_fractal` scale
/// (0xb69474), its clamp's threshold (0xb6963c) and value (0xb69638).
const TEXTURE_SCALE: Single = Single(0x3e23_d70a);
const TEXTURE_TOP: Single = Single(0x4178_0000);
const TEXTURE_CLAMP: Single = Single(0x4170_0000);
/// `adjust_aquatic_height`'s cap (0xb69744), its 1/9 (0xb6946c), and
/// `beach_steepness` (0xc0629c), a registered tweak whose default is 1.
const DEPTH_CAP: Single = Single(0x461c_4000);
const NINTH: Single = Single(0x3de3_8e39);
const BEACH_STEEPNESS: Single = Single(0x3f80_0000);
/// `relax_aquatic_verts`' weight (0xb69480).
const FIFTH: Single = Single(0x3e4c_cccd);

/// A sub-tile offset `t` in `0..16` of `mark_coast_vert_influence`'s line
/// ends: the column `t % 4` and the row `t / 4`, C's signed rounding.
fn sub_tile(t: i32) -> (i32, i32) {
    (t % 4, t / 4)
}

/// `texture_fractal`'s answer at a cell, scaled by 0.16 and clamped to 15
/// from 15.5 up, truncated (`cvttss2si`).
fn texture_offset(f: &Fractal, x: i32, y: i32) -> i32 {
    let t = Single::from_i32(i32::from(f.get_height(x, y))).mulss(TEXTURE_SCALE);
    if t.gt(TEXTURE_TOP) || t.eq_value(TEXTURE_TOP) {
        TEXTURE_CLAMP.to_i32()
    } else {
        t.to_i32()
    }
}

/// `LineState::init@00a48000` and `next_element@00a48110`: the tiles from
/// `(x0, y0)` to `(x1, y1)`, both ends included, a Bresenham walk along
/// the longer axis (an equal pair steps x).
pub fn line(x0: i32, y0: i32, x1: i32, y1: i32) -> Vec<(i32, i32)> {
    let (dx, dy) = (x1 - x0, y1 - y0);
    let mut out = vec![(x0, y0)];
    if dx == 0 && dy == 0 {
        return out;
    }
    let (sx, sy) = (dx.signum(), dy.signum());
    let (mut x, mut y) = (x0, y0);
    if dy == 0 {
        while x != x1 {
            x += sx;
            out.push((x, y));
        }
        return out;
    }
    if dx == 0 {
        while y != y1 {
            y += sy;
            out.push((x, y));
        }
        return out;
    }
    let (ax, ay) = (dx.abs(), dy.abs());
    if ax < ay {
        let (addx, addy) = (2 * ax, 2 * (ax - ay));
        let mut err = 2 * ax - ay;
        while y != y1 {
            y += sy;
            if err < 0 {
                err += addx;
            } else {
                err += addy;
                x += sx;
            }
            out.push((x, y));
        }
    } else {
        let (addx, addy) = (2 * ay, 2 * (ay - ax));
        let mut err = 2 * ay - ax;
        while x != x1 {
            x += sx;
            if err < 0 {
                err += addx;
            } else {
                err += addy;
                y += sy;
            }
            out.push((x, y));
        }
    }
    out
}

/// **`TerrainOut::mark_valid_coastal_influencers@00869070`**: a `CoordInfo`
/// 0x20 cell is valid, and joins the influencer list (y outer, x inner),
/// unless it is the inner corner of a bend — two orthogonal 0x20
/// neighbours (N or S with E or W) round a diagonal one flagged 0x4
/// (`move[1..9]`, off-map read as 0; 0x869243..0x869299).
pub fn valid_coastal_influencers(
    g: &MapGrid,
    coord: &[u16],
    info: &mut [CoastInfo],
) -> Vec<(i32, i32)> {
    let mut list = Vec::new();
    for y in 0..g.ys {
        for x in 0..g.xs {
            if coord[(y * g.xs + x) as usize] & coord::COASTAL == 0 {
                continue;
            }
            let mut n = [0u16; 8];
            for (k, &(dx, dy)) in MOVE[1..9].iter().enumerate() {
                let (u, v) = (x + dx, y + dy);
                if u >= 0 && v >= 0 && u < g.xs && v < g.ys {
                    n[k] = coord[(v * g.xs + u) as usize];
                }
            }
            // n: NW, N, NE, E, SE, S, SW, W.
            let c = |k: usize| n[k] & coord::COASTAL != 0;
            let h = |k: usize| n[k] & coord::HALF != 0;
            let bend = (c(1) && c(7) && h(0))
                || (c(1) && c(3) && h(2))
                || (c(5) && c(7) && h(6))
                || (c(5) && c(3) && h(4));
            if !bend {
                info[(y * g.xs + x) as usize].valid = true;
                list.push((x, y));
            }
        }
    }
    list
}

/// **`TerrainOut::mark_coast_vert_influence@00868ca0`**, the listing's
/// order: each influencer `c` joins its first two valid neighbours
/// (`move[1..9]`; a neighbour already pointing at itself, which never
/// arises, is skipped), and draws a line of tiles to each — from `c`'s
/// sub-tile at `texture_offset(c)` to the neighbour's at the offset of the
/// **last** neighbour joined. Every tile of a line is appended to the
/// current cell's list; the current cell starts as `c` and becomes the
/// tile's cell, marked `has_influence`, whenever both its coordinates
/// change. The second line reads its end and its count from that current
/// cell, not from `c` (0x868e80 reads `[ebx + 2]`, and `ebx` moved).
pub fn coast_vert_influence(
    g: &MapGrid,
    info: &mut [CoastInfo],
    list: &[(i32, i32)],
    texture: &Fractal,
) {
    let at = |x: i32, y: i32| (y * g.xs + x) as usize;
    for &(cx, cy) in list {
        let ci = at(cx, cy);
        let t0 = texture_offset(texture, cx, cy);
        let mut t1 = t0;
        for &(dx, dy) in &MOVE[1..9] {
            let (u, v) = (cx + dx, cy + dy);
            if u < 0 || v < 0 || u >= g.xs || v >= g.ys {
                continue;
            }
            let ni = at(u, v);
            if !info[ni].valid {
                continue;
            }
            let nb = &info[ni];
            if (nb.out != 0 && nb.to[0] == (u, v)) || (nb.out == 2 && nb.to[1] == (u, v)) {
                continue;
            }
            if info[ci].out == 2 {
                continue;
            }
            info[ci].out += 1;
            let slot = usize::from(info[ci].out != 1);
            info[ci].to[slot] = (u, v);
            t1 = texture_offset(texture, u, v);
        }
        let mut cur = ci;
        for k in 0..2 {
            let out = info[cur].out;
            if out == 1 && k == 1 {
                break;
            }
            let (ex, ey) = if out == 0 { (cx, cy) } else { info[cur].to[k] };
            let (s0, s1) = (sub_tile(t0), sub_tile(t1));
            let tiles = line(s0.0 + 4 * cx, s0.1 + 4 * cy, s1.0 + 4 * ex, s1.1 + 4 * ey);
            let mut prev = (-1, -1);
            for (tx, ty) in tiles {
                info[cur].tiles.push(tx + 4 * g.xs * ty);
                let (wx, wy) = (tx >> 2, ty >> 2);
                if wx != prev.0 && wy != prev.1 {
                    prev = (wx, wy);
                    cur = at(wx, wy);
                    info[cur].influence = true;
                }
            }
        }
    }
}

/// **`TerrainOut::adjust_aquatic_height@00868820`** (y outer, x inner): a
/// cell flagged 0x4 or `OCEAN` (which also gains `CoordInfo` 0x8), `land`
/// 1 or 3, or `BASELAND` with `has_influence`, whose 5×5 (`move[0..25]`)
/// holds a cell with `has_influence`, rewrites its 5×5 vertices (row
/// outer): the nearest tile of those cells' lists by integer squared
/// distance (strictly smaller; the best index carries over from the
/// previous vertex, and starts at 0), and the height
/// `min(|tile centre − vertex| / 768, 10000)² · (1/9) · (−303 + land_height)
/// + land_height`, times `beach_steepness` when negative. Its cells are
/// marked for [`relax_aquatic_verts`].
pub fn adjust_aquatic_height(
    g: &MapGrid,
    coord: &mut [u16],
    info: &[CoastInfo],
    z: &mut [Single],
) -> Vec<bool> {
    let n = VERTS * g.xs;
    let stride = n + 1;
    let at = |x: i32, y: i32| (y * g.xs + x) as usize;
    let mut marked = vec![false; g.cells.len()];
    for cy in 0..g.ys {
        for cx in 0..g.xs {
            let i = at(cx, cy);
            let c = &g.cells[i];
            let enter = if c.flags & 0x4 != 0 || c.land == 2 {
                coord[i] |= 0x8;
                true
            } else {
                c.land != 0 || info[i].influence
            };
            if !enter {
                continue;
            }
            let ring: Vec<usize> = MOVE[0..25]
                .iter()
                .map(|&(dx, dy)| (cx + dx, cy + dy))
                .filter(|&(u, v)| u >= 0 && v >= 0 && u < g.xs && v < g.ys)
                .map(|(u, v)| at(u, v))
                .collect();
            if !ring.iter().any(|&j| info[j].influence) {
                continue;
            }
            let mut best_t = 0;
            for vy in 0..=VERTS {
                for vx in 0..=VERTS {
                    let (px, py) = (VERTS * cx + vx, VERTS * cy + vy);
                    let mut best = i32::MAX;
                    for &j in &ring {
                        for &t in &info[j].tiles {
                            let (ey, ex) = (py - t / n, px - t % n);
                            let d = ey * ey + ex * ex;
                            if d < best {
                                best = d;
                                best_t = t;
                            }
                        }
                    }
                    let fx = Single::from_i32(best_t % n).addss(Single::HALF).mulss(POLY);
                    let fy = Single::from_i32(best_t / n).addss(Single::HALF).mulss(POLY);
                    let dx = fx.subss(Single::from_i32(px).mulss(POLY));
                    let dy = fy.subss(Single::from_i32(py).mulss(POLY));
                    let d = dx.mulss(dx).addss(dy.mulss(dy)).sqrtss().mulss(OO_WCOORD);
                    let d = if d.gt(DEPTH_CAP) || d.eq_value(DEPTH_CAP) {
                        DEPTH_CAP
                    } else {
                        d
                    };
                    let mut h = d
                        .mulss(d)
                        .mulss(NINTH)
                        .mulss(SEA_FLOOR.addss(LAND_HEIGHT))
                        .addss(LAND_HEIGHT);
                    if Single::ZERO.gt(h) {
                        h = h.mulss(BEACH_STEEPNESS);
                    }
                    z[(py * stride + px) as usize] = h;
                    marked[i] = true;
                }
            }
        }
    }
    marked
}

/// **`TerrainOut::relax_aquatic_verts@00868470`**: five passes, each over
/// the marked cells off the map's edge (y outer, x inner), each of their
/// 5×5 vertices (row outer) in place: `(below + above + left + right +
/// itself) · 0.2`, summed in that order.
pub fn relax_aquatic_verts(g: &MapGrid, marked: &[bool], z: &mut [Single]) {
    let stride = (VERTS * g.xs + 1) as usize;
    for _ in 0..5 {
        for cy in 0..g.ys {
            for cx in 0..g.xs {
                if !marked[(cy * g.xs + cx) as usize]
                    || cx == 0
                    || cy == 0
                    || cx == g.xs - 1
                    || cy == g.ys - 1
                {
                    continue;
                }
                for vy in 0..=VERTS {
                    for vx in 0..=VERTS {
                        let v = ((VERTS * cy + vy) as usize) * stride + (VERTS * cx + vx) as usize;
                        z[v] = z[v + stride]
                            .addss(z[v - stride])
                            .addss(z[v - 1])
                            .addss(z[v + 1])
                            .addss(z[v])
                            .mulss(FIFTH);
                    }
                }
            }
        }
    }
}

/// **`TerrainOut::build_coastal_data@0086c540`**'s height half, in its
/// order: `mark_valid_coastal_influencers`, `mark_coast_vert_influence`
/// (on `texture_fractal`: `xs + 1` by `ys + 1`, smoothing 0, flags 2,
/// `World.seed`, built by `refresh_data` beside the height fractals),
/// [`adjust_aquatic_height`] and [`relax_aquatic_verts`]. It adds
/// `CoordInfo` 0x8 to the water cells it enters. A map without a 0x20 cell
/// writes nothing.
pub fn build_coastal_data(g: &MapGrid, coord: &mut [u16], z: &mut [Single], seed: u32) {
    let mut info = vec![CoastInfo::default(); g.cells.len()];
    let list = valid_coastal_influencers(g, coord, &mut info);
    let texture = Fractal::init(g.xs + 1, g.ys + 1, 0, 2, seed);
    coast_vert_influence(g, &mut info, &list, &texture);
    let marked = adjust_aquatic_height(g, coord, &info, z);
    relax_aquatic_verts(g, &marked, z);
}

/// `check_and_set_halfland_wcoord`'s threshold (`__real_c1f00000`), −30.
pub const WATERLINE: Single = Single(0xc1f0_0000);

/// **`Terrain::check_and_set_halfland_wcoord@0084dec0`'s height test**: how
/// many of cell `(x, y)`'s sixteen sub-tiles have all four corner heights
/// strictly below −30 (`*p <= -30 && -30 != *p`, 0x84e0a0…), the count
/// its body keys on — none leaves the cell dry and spreads 0x4, sixteen
/// turns it `OCEAN` through `mark_halfwater`, and the rest leave it
/// HALFLAND. The writes themselves are not built here.
pub fn water_subtiles(g: &MapGrid, z: &[Single], x: i32, y: i32) -> u32 {
    let stride = VERTS * g.xs + 1;
    let below = |u: i32, v: i32| WATERLINE.gt(z[(v * stride + u) as usize]);
    let mut n = 0;
    for j in 0..VERTS {
        for i in 0..VERTS {
            let (u, v) = (VERTS * x + i, VERTS * y + j);
            if below(u, v) && below(u + 1, v) && below(u, v + 1) && below(u + 1, v + 1) {
                n += 1;
            }
        }
    }
    n
}

/// One mountain template's displacement, `MountainRange::init@008998b0`'s
/// `tcoord_verts`: a vertex per tile corner of its `TEMPLATE_TEX` whose
/// alpha is set, as an offset in vertices from the mountain's origin vertex
/// (four times its cell) and the height it adds.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Mesh {
    pub verts: Vec<(i32, i32, Single)>,
}

impl Mesh {
    /// From the template image's sampled pixels — `(x, y, byte)`
    /// (`movzx` of the 32-bit texel's lowest byte, 0x899d90: the file's
    /// red, `docs/MAPGEN.md` §28.3) at every
    /// fourth pixel from `((w/2) % 16, (h/2) % 16)` whose alpha is
    /// not zero, rows outer — and the `<MOUNTAIN>`'s `height`: each vertex
    /// sits `(x − w/2) / 4` vertices across (the listing's `(x − w/2) ·
    /// 192 · 0.25` world units over the 192 of a vertex) and adds
    /// `byte · (height · 3) / 255` in singles.
    pub fn from_pixels(w: i32, h: i32, pixels: &[(i32, i32, u8)], height: Single) -> Mesh {
        let scale = height.mulss(THREE);
        Mesh {
            verts: pixels
                .iter()
                .map(|&(x, y, byte)| {
                    let z = Single::from_i32(i32::from(byte))
                        .mulss(scale)
                        .divss(BYTE_MAX);
                    ((x - w / 2) / 4, (y - h / 2) / 4, z)
                })
                .collect(),
        }
    }
}

/// **`TerrainOut::adjust_for_mountains@008703c0`**: for each placed
/// mountain in place order, its template's cells (`mount_wx`/`_wy`) in
/// order, and each cell's 5×5 vertices, y outer —
/// `TerrainOut::fill_mountain_data@00869380` adds the height of the mesh
/// vertex that stands on the vertex (within 0.01 world units, which on the
/// 192-unit lattice is equality), once per mountain. Its first call, on
/// the fine `fcoord_verts`, writes no height and is not modelled.
pub fn adjust_for_mountains(
    g: &MapGrid,
    z: &mut [Single],
    templates: &[Template],
    meshes: &[Mesh],
) {
    let w = VERTS * g.xs + 1;
    let h = VERTS * g.ys + 1;
    for &(mx, my, t) in &g.mountains {
        let (Some(tpl), Some(mesh)) = (templates.get(t), meshes.get(t)) else {
            continue;
        };
        let at: BTreeMap<(i32, i32), usize> = mesh
            .verts
            .iter()
            .enumerate()
            .map(|(k, &(dx, dy, _))| ((VERTS * mx + dx, VERTS * my + dy), k))
            .collect();
        let mut used = vec![false; mesh.verts.len()];
        for &(dx, dy) in &tpl.cells {
            let (cx, cy) = (mx + dx, my + dy);
            for j in 0..=VERTS {
                for i in 0..=VERTS {
                    let (vx, vy) = (VERTS * cx + i, VERTS * cy + j);
                    let Some(&k) = at.get(&(vx, vy)) else {
                        continue;
                    };
                    if used[k] || vx < 0 || vy < 0 || vx >= w || vy >= h {
                        continue;
                    }
                    used[k] = true;
                    let v = (vy * w + vx) as usize;
                    z[v] = z[v].addss(mesh.verts[k].2);
                }
            }
        }
    }
}

/// **`TerrainOil::create_doobers@008a28e0`**'s terraform, which
/// `Terrain::init` runs between `generate_land` and
/// `adjust_for_mountains`: each cell, y outer, that carries the rock bit
/// (or a good the console's leader cannot see yet, flag 1) and is not
/// ocean reaches `TerrainOil::add_rocky_patch@008a2b30`, which on an
/// `OIL` cell flattens the cell's 4×4 tiles with
/// `TerrainOut::terraform_for_building` at the cell's tile corner
/// (`div_3_table[(12·c + 1.5)]`, `4·c`). Every other cell's patch is
/// render data and writes no height.
///
/// An `OIL` cell carries no good of flag 1 (`World::set_oil_at` sets
/// none), so the flag-1 arm's `LeaderData::type_avail` test never decides
/// a height; its cells are not walked. The ocean arm's clearing of the
/// rock bit is outside the heights and not modelled.
pub fn create_doobers(g: &MapGrid, z: &mut [Single]) {
    for y in 0..g.ys {
        for x in 0..g.xs {
            let c = &g.cells[(y * g.xs + x) as usize];
            if c.flags & ROCK != 0 && c.flags & OIL != 0 && !c.is_ocean() {
                terraform(g, z, VERTS * x, VERTS * y, VERTS, VERTS);
            }
        }
    }
}

/// `TerrainOil::add_rocky_patch@008a2b30`'s terraform of an oil cell
/// `(x, y)`: [`create_doobers`]' box at the cell's tile corner.
pub fn terraform_oil(g: &MapGrid, z: &mut [Single], x: i32, y: i32) {
    terraform(g, z, VERTS * x, VERTS * y, VERTS, VERTS);
}

/// **`TerrainOut::terraform_for_building@00875210`** on the map
/// generator's grid: the same box, mean and refusals as
/// `Sim::terraform_for_building` (`docs/ROADS.md` §7.4) — the box
/// `[cx − 1, cx + w + 2) × [cy − 1, cy + h + 2)` clamped, the mean over it
/// and its closing column and row (abandoned on a corner at or below
/// zero), the interior set to the mean and the border to `(h + mean) / 2`,
/// held back on ocean, on a mountain or cliff tile in the corner's 3×3,
/// on a cell with `land`, and on a tile of a live good that is not oil.
fn terraform(g: &MapGrid, z: &mut [Single], cx: i32, cy: i32, w: i32, h: i32) {
    let (gw, gh) = (VERTS * g.xs, VERTS * g.ys);
    let stride = gw + 1;
    if cx == 0 || cy == 0 || cx == gw - 1 || cy == gw - 1 {
        return;
    }
    let (x0, y0) = ((cx - 1).max(0), (cy - 1).max(0));
    let (x1, y1) = ((cx + w + 2).min(gw - 1), (cy + h + 2).min(gh - 1));
    let at = |x: i32, y: i32| (y * stride + x) as usize;
    let mut sum = Single::ZERO;
    let mut n = 0;
    let column = (y0..y1).map(|y| (x1, y));
    let row = (x0..x1).map(|x| (x, y1));
    let inside = (y0..y1).flat_map(|y| (x0..x1).map(move |x| (x, y)));
    for (x, y) in column.chain(row).chain(inside) {
        let v = z[at(x, y)];
        if !v.gt(Single::ZERO) {
            return;
        }
        sum = sum.addss(v);
        n += 1;
    }
    let mean = sum.divss(Single::from_i32(n));
    let tile = |x: i32, y: i32| g.tiles[(y * gw + x) as usize];
    for y in y0..y1 {
        for x in x0..x1 {
            let c = &g.cells[((y >> 2) * g.xs + (x >> 2)) as usize];
            if c.is_ocean() || c.land != 0 {
                continue;
            }
            let held = (-1..=1).any(|dy| {
                (-1..=1).any(|dx| {
                    let (u, v) = (x + dx, y + dy);
                    u >= 0 && v >= 0 && u < gw && v < gh && matches!(tile(u, v) & 3, 1 | 2)
                })
            });
            if held {
                continue;
            }
            if x < gw && y < gh && tile(x, y) & 0x200 != 0 && good_here(g, x >> 2, y >> 2) {
                continue;
            }
            let border = y == y0 || y == y1 - 1 || x == x0 || x == x1 - 1;
            let v = &mut z[at(x, y)];
            *v = if border {
                v.addss(mean).mulss(Single::HALF)
            } else {
                mean
            };
        }
    }
}

/// A live good that is not oil on the cell (`Sim::good_here`).
fn good_here(g: &MapGrid, x: i32, y: i32) -> bool {
    g.goods
        .iter()
        .flatten()
        .any(|&(t, gx, gy)| gx == x && gy == y && t != OIL_GOOD)
}

/// [`heights`]' refusal of `Cliffs::init@008a8020` (built in
/// [`crate::cliffs`]): every tile `assess_wcoord` would hand
/// to `has_plus_slope` — of a cell flagged 0x40, neither COAST nor `land`
/// 1 or 2, and not a mountain tile (`mask & 3 == 2`).
pub fn slope_tiles(g: &MapGrid) -> Vec<(i32, i32)> {
    let (tw, _) = g.tile_dims();
    let mut out = Vec::new();
    for cy in 0..g.ys {
        for cx in 0..g.xs {
            let c = &g.cells[(cy * g.xs + cx) as usize];
            if c.flags & 0x40 == 0 || c.flags & COAST != 0 || c.land == 1 || c.land == 2 {
                continue;
            }
            for j in 0..VERTS {
                for i in 0..VERTS {
                    let (tx, ty) = (VERTS * cx + i, VERTS * cy + j);
                    if g.tiles[(ty * tw + tx) as usize] & 3 != 2 {
                        out.push((tx, ty));
                    }
                }
            }
        }
    }
    out
}

/// **The phase** on a grid that has stood through `make_rivers`:
/// `generate_land`, the oil cells' terraform, `adjust_for_mountains`, then `Cliffs::init`'s
/// refusal. Between them, `check_and_set_halfland` reads a height only on
/// a gated cell, which `sim::terrain_init` already refuses; the troughs
/// carve only along a river list ([`crate::map_troughs`], not called here:
/// a style without one carves nothing).
pub fn heights(
    g: &MapGrid,
    coord: &mut [u16],
    seed: u32,
    templates: &[Template],
    meshes: &[Mesh],
) -> Result<Vec<Single>, Unported> {
    let f = fractals(g.xs, g.ys, seed);
    let mut z = generate_land(g, coord, &f);
    build_coastal_data(g, coord, &mut z, seed);
    create_doobers(g, &mut z);
    adjust_for_mountains(g, &mut z, templates, meshes);
    let slope = slope_tiles(g);
    if !slope.is_empty() {
        return Err(Unported::Slope(slope));
    }
    Ok(z)
}

#[cfg(test)]
mod tests;
