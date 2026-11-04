//! **Map generation's heights** (`docs/MAPGEN.md` §10, §11, §15 step 7):
//! the corner grid `TerrainData::master_land_heights` that `Terrain::init`
//! builds between `Terrain.cpp` 699 and 894, in the original's own SSE
//! singles ([`Single`]) and, for the two fractals' bilinear sample, its
//! doubles ([`crate::double::Double`]).
//!
//! Two passes write it on a map without water:
//! - **`TerrainOut::generate_land@0085f8d0`**, one vertex at a time, y
//!   outer and x inner, through
//!   `TerrainOut::determine_land_height_color@0086c1e0` (which returns in
//!   xmm0, the decompile drops it): a river vertex 0, a vertex whose
//!   `CoordInfo` codes carry 0x1000 `(−303 + land_height) · 1`, 0x4 zero,
//!   0x2 100, and any other `(a · 7.5 + land_height) + b · 1.875`, `a` and
//!   `b` the two fractals' samples ([`fractals`]).
//! - **`TerrainOut::adjust_for_mountains@008703c0`**, each placed mountain
//!   adding its template's displacement to the vertices of its cells
//!   ([`adjust_for_mountains`]).
//!
//! What a waterless map does not reach is refused by name, never guessed
//! ([`Unported`]): the shore blend and `build_coastal_data`, and
//! `Cliffs::assess_wcoord`'s slope test.

use std::collections::BTreeMap;

use crate::mapgen::{COAST, Fractal, MapGrid};
use crate::single::Single;
use crate::terrain_groups::Template;
use crate::terrain_groups::flag::{OIL, ROCK};
use crate::terrain_init::coord;

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
    /// The shore: `TerrainOut::find_closest_coordinfo@0086a260` finds a
    /// `CoordInfo` 0x20 cell in the 7×7 round a vertex's cell, which blends
    /// the height by the distance and lists the vertex for
    /// `TerrainOut::smooth_tcoord@0086c3a0`; or an `OCEAN` cell, whose
    /// vertices `TerrainOut::build_coastal_data@0086c540` relaxes. The
    /// cells named.
    Coast(Vec<(i32, i32)>),
    /// `Cliffs::assess_wcoord@008a6a40`'s slope test,
    /// `TerrainOut::has_plus_slope@00869790`: a tile of a cell flagged
    /// 0x40 that is not COAST, `land` 1 or 2, nor a mountain tile. The
    /// tiles named.
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

/// The cells `find_closest_coordinfo` or `build_coastal_data` would act
/// on: `CoordInfo` 0x20 (whose 7×7 reaches a vertex) or `OCEAN`.
fn shore(g: &MapGrid, coord: &[u16]) -> Vec<(i32, i32)> {
    (0..g.ys)
        .flat_map(|y| (0..g.xs).map(move |x| (x, y)))
        .filter(|&(x, y)| coord[(y * g.xs + x) as usize] & (coord::COASTAL | coord::OCEAN) != 0)
        .collect()
}

/// **`generate_land`'s heights**: `(4·ys + 1)` rows of `(4·xs + 1)`
/// singles, row-major, as the start dump prints them.
///
/// The `Prefs` value that flattens every height to 0 is taken as unset.
pub fn generate_land(
    g: &MapGrid,
    coord: &[u16],
    fractals: &(Fractal, Fractal),
) -> Result<Vec<Single>, Unported> {
    let shore = shore(g, coord);
    if !shore.is_empty() {
        return Err(Unported::Coast(shore));
    }
    let (w, h) = (VERTS * g.xs + 1, VERTS * g.ys + 1);
    let mut z = Vec::with_capacity((w * h) as usize);
    for y in 0..h {
        for x in 0..w {
            z.push(if river_vertex(g, x, y) {
                Single::ZERO
            } else {
                vertex_height(vert_codes(g, coord, x, y), fractals, x, y)
            });
        }
    }
    Ok(z)
}

/// `determine_land_height_color` past its river test, the shore blend
/// excluded (0x86c233..0x86c37a).
fn vertex_height(codes: u16, (a, b): &(Fractal, Fractal), x: i32, y: i32) -> Single {
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
    h.addss(Single::from_i32(i32::from(b.get_height(x, y))).mulss(B_WEIGHT))
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
    /// From the template image's sampled pixels — `(x, y, first byte)`
    /// (`movzx` of the 32-bit pixel's lowest byte, 0x899d90) at every
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

/// `Cliffs::init@008a8020`'s refusal: every tile `assess_wcoord` would hand
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
/// `generate_land`, then `adjust_for_mountains`, then `Cliffs::init`'s
/// refusal. Between them, `check_and_set_halfland` reads a height only on
/// a gated cell, which `sim::terrain_init` already refuses; the troughs
/// carve only along a river list, which `sim::map_rivers` refuses.
pub fn heights(
    g: &MapGrid,
    coord: &[u16],
    seed: u32,
    templates: &[Template],
    meshes: &[Mesh],
) -> Result<Vec<Single>, Unported> {
    let f = fractals(g.xs, g.ys, seed);
    let mut z = generate_land(g, coord, &f)?;
    create_doobers(g, &mut z);
    adjust_for_mountains(g, &mut z, templates, meshes);
    let slope = slope_tiles(g);
    if !slope.is_empty() {
        return Err(Unported::Slope(slope));
    }
    Ok(z)
}
