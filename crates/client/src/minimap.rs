//! **The minimap** (item 4581; `docs/RENDERER.md` §3.32,
//! `docs/CLIENT_INPUT.md` §9).
//!
//! The original draws it in two layers, both read off run1564's packet at
//! frame 3294 (`MiscAccess::worldmap`, `0x00e399ac`):
//!
//! - **the background**: `WorldMapBackground::render@008cfc40` draws the
//!   terrain (`TerrainOut::render`, `render_mountains`, `CliffsOut::render`)
//!   once, from a camera straight down over the whole map, into a
//!   `back_xs` × `back_ys` (256 × 256) texture;
//! - **the quad**: `WorldMap::render_background@008cb6e0` draws that
//!   texture on two pre-transformed triangles, `translated_verts`
//!   (`+0xbb8`), whose texcoords `texs` (`+0xbdc`) put `(0, 0)` on the top
//!   corner, `(1, 0)` on the right, `(1, 1)` on the bottom and `(0, 1)` on
//!   the left.
//!
//! The texture's `(u, v)` is the world's `(x, y)` over the map's extent:
//! the minimap's own camera (`WorldMap +0x808`, parallel, `dir_angle` 45,
//! `tilt_angle` 0, its look-at the map's centre) sends world `(0, 0)` to the
//! top corner and `(extent, 0)` to the right, and [`MINIMAP_CAMERA`] is that
//! camera, its `to_screen_matrix` the packet's to the printed digit.
//!
//! Ours: the background is our own terrain drawn straight down
//! ([`background_camera`]), and the HUD draws it one cell a quad
//! ([`layer`]), each cell in the background's colour at its centre. The
//! borders and the view's rectangle are drawn over it ([`overlays`]); the
//! fog, the blips and the pings are not.

/// **The quad's corners** (item 4581): `translated_verts`' screen x, y on
/// run1564's packet at frame 3294, top, right, bottom, left, on the
/// photograph's 1920 × 1080 frame (the window's 32-row title bar cut). The
/// triangles are top-right-bottom and top-bottom-left.
pub const QUAD: [(i32, i32); 4] = [(485, 910), (643, 990), (484, 1069), (326, 990)];

/// The frame [`QUAD`] was read on.
pub const QUAD_FRAME: (i32, i32) = (1920, 1080);

/// `WorldMap::back_xs`, `back_ys` (`+0x10c8`, `+0x10cc`): 256 each, set by
/// `WorldMap::init@008d2160` and read on the packet.
pub const BACK: u32 = 256;

/// **The minimap's own camera** (item 4581): `WorldMap::cam` (`+0x808`) on
/// run1564's packet, a map of 60 × 60 tiles. `init_camera@008cbb10` sets
/// `angle` 45 (`+0x934`) and the look-at to the map's centre; the rest is
/// the packet's. Only [`tests`] reads it: it pins the texture's
/// orientation, which [`background_camera`] keeps.
#[cfg(test)]
pub fn minimap_camera() -> render::Camera {
    let f = f32::from_bits;
    render::Camera {
        zoom_distance: f(0x4799_2e8d),
        camera_scale: 1.0,
        tilt_angle: 0.0,
        dir_angle: f(0x4234_0000),
        loc_x: f(0x46b4_0000),
        loc_y: f(0x46b4_0000),
        loc_x_adjust: 0.0,
        loc_y_adjust: 0.0,
        loc_z_adjust: 0.0,
        angle: f(0x4234_0000),
        near_plane: f(0x4791_ae8d),
        far_plane: f(0x47a5_d2f0),
        view_ratio: 1.0,
        parallel: true,
        parallel_distance: f(0x4799_2e8d),
    }
}

/// The quad's corners on a `size` frame: [`QUAD`] kept at its distance
/// from the frame's left and bottom edges, where the console's bar sits.
/// Not established: the original's layout at another size.
pub fn quad(size: (i32, i32)) -> [(i32, i32); 4] {
    let dy = size.1 - QUAD_FRAME.1;
    QUAD.map(|(x, y)| (x, y + dy))
}

/// The two triangles' corners as `(screen, uv)`, each texcoord a corner of
/// the unit square: top-right-bottom, then top-bottom-left.
type Corner = ((f32, f32), (f32, f32));

fn triangles(size: (i32, i32)) -> [[Corner; 3]; 2] {
    let q = quad(size).map(|(x, y)| (x as f32, y as f32));
    let (t, r, b, l) = (q[0], q[1], q[2], q[3]);
    [
        [(t, (0.0, 0.0)), (r, (1.0, 0.0)), (b, (1.0, 1.0))],
        [(t, (0.0, 0.0)), (b, (1.0, 1.0)), (l, (0.0, 1.0))],
    ]
}

/// **World to minimap**: the screen point `(x, y)` names, through the
/// triangle its texcoord falls in (`v ≤ u` the first).
pub fn to_screen(size: (i32, i32), extent: (i32, i32), x: i32, y: i32) -> (f32, f32) {
    let (u, v) = (x as f32 / extent.0 as f32, y as f32 / extent.1 as f32);
    let tri = &triangles(size)[usize::from(v > u)];
    // Barycentric weights of (u, v) in the triangle's texcoords.
    let [(p0, a), (p1, b), (p2, c)] = *tri;
    let det = (b.0 - a.0) * (c.1 - a.1) - (c.0 - a.0) * (b.1 - a.1);
    let w1 = ((u - a.0) * (c.1 - a.1) - (c.0 - a.0) * (v - a.1)) / det;
    let w2 = ((b.0 - a.0) * (v - a.1) - (u - a.0) * (b.1 - a.1)) / det;
    let w0 = 1.0 - w1 - w2;
    (
        w0 * p0.0 + w1 * p1.0 + w2 * p2.0,
        w0 * p0.1 + w1 * p1.1 + w2 * p2.1,
    )
}

/// **Minimap to world**: the world point at the screen point `(px, py)`,
/// or `None` outside the quad: the right press's, held inside the map as
/// `WorldMap::on_right_up@008c7050` holds its own, and the window's test
/// of whether a press is on the minimap at all.
pub fn to_world(size: (i32, i32), extent: (i32, i32), px: f32, py: f32) -> Option<(i32, i32)> {
    for tri in triangles(size) {
        let [(a, ta), (b, tb), (c, tc)] = tri;
        let det = (b.0 - a.0) * (c.1 - a.1) - (c.0 - a.0) * (b.1 - a.1);
        let w1 = ((px - a.0) * (c.1 - a.1) - (c.0 - a.0) * (py - a.1)) / det;
        let w2 = ((b.0 - a.0) * (py - a.1) - (px - a.0) * (b.1 - a.1)) / det;
        let w0 = 1.0 - w1 - w2;
        let eps = -1e-4;
        if w0 >= eps && w1 >= eps && w2 >= eps {
            let u = w0 * ta.0 + w1 * tb.0 + w2 * tc.0;
            let v = w0 * ta.1 + w1 * tb.1 + w2 * tc.1;
            let x = ((u * extent.0 as f32) as i32).clamp(0, extent.0 - 1);
            let y = ((v * extent.1 as f32) as i32).clamp(0, extent.1 - 1);
            return Some((x, y));
        }
    }
    None
}

/// **The left press's point** (item 4581): the texcoord under `(px, py)`
/// on the quad's plane, unbounded, as `RenderWindow::mouse_to_xy_plane`
/// reads it off the minimap's camera, then held to `[0, extent]` each way
/// as `WorldMap::update_world_map@008d2ba0` holds the camera's look-at.
/// [`QUAD`] is a parallelogram to a pixel (its bottom 1069, not 1070), so
/// the plane is the triangle's the texcoord falls in.
pub fn to_world_held(size: (i32, i32), extent: (i32, i32), px: f32, py: f32) -> (i32, i32) {
    let on = |k: usize| {
        let [(a, ta), (b, tb), (c, tc)] = triangles(size)[k];
        let det = (b.0 - a.0) * (c.1 - a.1) - (c.0 - a.0) * (b.1 - a.1);
        let w1 = ((px - a.0) * (c.1 - a.1) - (c.0 - a.0) * (py - a.1)) / det;
        let w2 = ((b.0 - a.0) * (py - a.1) - (px - a.0) * (b.1 - a.1)) / det;
        let w0 = 1.0 - w1 - w2;
        (
            w0 * ta.0 + w1 * tb.0 + w2 * tc.0,
            w0 * ta.1 + w1 * tb.1 + w2 * tc.1,
        )
    };
    // The triangle the point's texcoord falls in, `v ≤ u` the first.
    let (u, v) = on(0);
    let (u, v) = if v > u { on(1) } else { (u, v) };
    (
        ((u * extent.0 as f32) as i32).clamp(0, extent.0),
        ((v * extent.1 as f32) as i32).clamp(0, extent.1),
    )
}

/// **The background's camera** (item 4581): straight down over the whole
/// map, parallel, so that the [`BACK`]-square still's pixel `(i, j)` is the
/// world's `(x, y)` at `((i + ½), (j + ½)) · extent / BACK`.
/// `WorldMapBackground::setup_camera@008d0b70` puts the look-at at the
/// map's centre too; its `dir` is the register the decompile loses, so the
/// orientation is [`tests`]' corners, against [`minimap_camera`]'s.
pub fn background_camera(extent: (i32, i32)) -> render::Camera {
    let half = extent.0.max(extent.1) as f32 * 0.5;
    let angle = 45.0_f32;
    let t = ((angle.to_radians() * 0.5) as f64).tan() as f32;
    let height = 4.0 * half;
    render::Camera {
        zoom_distance: height,
        camera_scale: 1.0,
        tilt_angle: 0.0,
        dir_angle: BACK_DIR,
        loc_x: half,
        loc_y: half,
        loc_x_adjust: 0.0,
        loc_y_adjust: 0.0,
        loc_z_adjust: 0.0,
        angle,
        near_plane: height * 0.5,
        far_plane: height * 1.5,
        view_ratio: 1.0,
        parallel: true,
        parallel_distance: half / t,
    }
}

/// The background camera's `dir_angle`: the one of 0, 90, 180 and 270
/// that sends world `(0, 0)` to the still's top-left and `(extent, 0)` to
/// its top-right ([`tests`]; 270, 180 and 90 each fail it).
const BACK_DIR: f32 = 0.0;

/// **The background** (item 4581): a [`BACK`]-square still of the terrain
/// alone, as rows of RGBA bytes.
pub struct Background {
    pub rgba: Vec<u8>,
    pub extent: (i32, i32),
}

impl Background {
    /// The texel under texcoord `(u, v)`, bilinear, as the original's
    /// sampler minifies it.
    pub fn sample(&self, u: f32, v: f32) -> [u8; 3] {
        let n = BACK as i32;
        let (fx, fy) = (u * n as f32 - 0.5, v * n as f32 - 0.5);
        let (x0, y0) = (fx.floor() as i32, fy.floor() as i32);
        let (ax, ay) = (fx - x0 as f32, fy - y0 as f32);
        let at = |x: i32, y: i32| {
            let (x, y) = (x.clamp(0, n - 1), y.clamp(0, n - 1));
            let i = ((y * n + x) * 4) as usize;
            [self.rgba[i], self.rgba[i + 1], self.rgba[i + 2]].map(f32::from)
        };
        let (a, b, c, d) = (
            at(x0, y0),
            at(x0 + 1, y0),
            at(x0, y0 + 1),
            at(x0 + 1, y0 + 1),
        );
        let mut out = [0u8; 3];
        for k in 0..3 {
            let top = a[k] + (b[k] - a[k]) * ax;
            let bot = c[k] + (d[k] - c[k]) * ax;
            out[k] = (top + (bot - top) * ay).round().clamp(0.0, 255.0) as u8;
        }
        out
    }

    /// Cell `(i, j)`'s colour: the background at its centre.
    pub fn cell(&self, tiles: (i32, i32), i: i32, j: i32) -> [u8; 3] {
        let u = (i as f32 + 0.5) / tiles.0 as f32;
        let v = (j as f32 + 0.5) / tiles.1 as f32;
        self.sample(u, v)
    }
}

/// A map cell (`terrain.width` counts them): 768 world units, four 192-unit tiles a side.
pub const CELL: i32 = 768;

/// **The HUD's minimap** (item 4581): one quad a cell, its corners through
/// [`to_screen`], in the background's colour at the cell's centre, as an
/// opaque screen layer over the frame.
pub fn layer(back: &Background, size: (i32, i32)) -> crate::gpu::ScreenLayer {
    let extent = back.extent;
    let tiles = (extent.0 / CELL, extent.1 / CELL);
    let (sw, sh) = (size.0 as f32, size.1 as f32);
    let ndc = |(x, y): (f32, f32)| [x / sw * 2.0 - 1.0, 1.0 - y / sh * 2.0, 0.0];
    let corner = |i: i32, j: i32| ndc(to_screen(size, extent, i * CELL, j * CELL));
    let mut pos = Vec::with_capacity((tiles.0 * tiles.1 * 6) as usize);
    let mut col = Vec::with_capacity(pos.capacity());
    for j in 0..tiles.1 {
        for i in 0..tiles.0 {
            let c = back.cell(tiles, i, j).map(|v| f32::from(v) / 255.0);
            let (a, b, d, e) = (
                corner(i, j),
                corner(i + 1, j),
                corner(i + 1, j + 1),
                corner(i, j + 1),
            );
            for p in [a, b, d, a, d, e] {
                pos.push(p);
                col.push(c);
            }
        }
    }
    crate::gpu::ScreenLayer {
        alpha: 255,
        pos,
        col,
    }
}

/// **`WorldMapFog::fill_colors_who@008d1070`'s alpha on a territory's
/// inside**, `0x50`, and **on its edge**, 200 (item 4588).
pub const WHO_INSIDE: u8 = 0x50;
pub const WHO_EDGE: u8 = 200;

/// **`who_render_state.alpha_ref`** as `WorldMap::render_borders@008cc490`
/// sets it when its argument is 0: 100, against `CMP_GREATER`, so the
/// inside's `0x50` is dropped and the edge alone is drawn (item 4588). The
/// argument is 1, and the reference 0, under `stat_mode` 0 or without
/// the profile's `prefs & 4`; run1564's photograph shows no inside, and
/// the client draws this one.
pub const WHO_ALPHA_REF: f32 = 100.0;

/// The eight neighbours `fill_colors_who` walks, `move_x[1..9]` and
/// `move_y[1..9]` (`.rdata`, read off the executable): `(-1, -1)` first,
/// then clockwise.
const MOVE: [(i32, i32); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
];

/// **The who texture** (item 4588; `docs/RENDERER.md` §3.32, "The
/// overlays"): `WorldMapFog::fill_colors_who@008d1070`, one texel a fog
/// cell, `2 · cells` each way, row-major, `[r, g, b, a]`. A texel whose
/// cell a player owns takes that player's `TeamColor::get_border` at
/// [`WHO_INSIDE`], or at [`WHO_EDGE`] when one of its eight neighbours in
/// the fog grid lies on a cell owned by anyone else or by no one. A
/// neighbour off the grid, or on a cell of open water (`flags & 0x100`
/// clear and the land class 1 or 2), is passed over. Every other texel is
/// zero. Assumed here: the console sees every player's borders
/// (`LeaderData::can_see_borders`), and no two owners are mutual allies
/// (diplomacy 2 both ways, which would spare their shared edge).
pub fn who_texels(
    cells: (i32, i32),
    owner: &[sim::Owner],
    sea: impl Fn(usize) -> bool,
    colour: impl Fn(sim::Player) -> [u8; 3],
) -> Vec<[u8; 4]> {
    let (fw, fh) = (cells.0 * 2, cells.1 * 2);
    let cell = |fx: i32, fy: i32| ((fy >> 1) * cells.0 + (fx >> 1)) as usize;
    let mut out = vec![[0u8; 4]; (fw * fh) as usize];
    for fy in 0..fh {
        for fx in 0..fw {
            let sim::Owner::Player(p) = owner[cell(fx, fy)] else {
                continue;
            };
            let edge = MOVE.iter().any(|&(dx, dy)| {
                let (nx, ny) = (fx + dx, fy + dy);
                if nx < 0 || ny < 0 || nx >= fw || ny >= fh || sea(cell(nx, ny)) {
                    return false;
                }
                owner[cell(nx, ny)] != sim::Owner::Player(p)
            });
            let [r, g, b] = colour(p);
            out[(fy * fw + fx) as usize] = [r, g, b, if edge { WHO_EDGE } else { WHO_INSIDE }];
        }
    }
    out
}

/// The texcoord under the screen point `(px, py)` on the quad, or `None`
/// outside it.
fn to_uv(size: (i32, i32), px: f32, py: f32) -> Option<(f32, f32)> {
    triangles(size)
        .into_iter()
        .find_map(|[(a, ta), (b, tb), (c, tc)]| {
            let det = (b.0 - a.0) * (c.1 - a.1) - (c.0 - a.0) * (b.1 - a.1);
            let w1 = ((px - a.0) * (c.1 - a.1) - (c.0 - a.0) * (py - a.1)) / det;
            let w2 = ((b.0 - a.0) * (py - a.1) - (px - a.0) * (b.1 - a.1)) / det;
            let w0 = 1.0 - w1 - w2;
            let eps = -1e-4;
            (w0 >= eps && w1 >= eps && w2 >= eps).then(|| {
                (
                    w0 * ta.0 + w1 * tb.0 + w2 * tc.0,
                    w0 * ta.1 + w1 * tb.1 + w2 * tc.1,
                )
            })
        })
}

/// One pixel's square, `(x, y)` to `(x + 1, y + 1)`, as two triangles in
/// the frame's device coordinates.
fn pixel(size: (i32, i32), x: i32, y: i32) -> [[f32; 3]; 6] {
    let (sw, sh) = (size.0 as f32, size.1 as f32);
    let ndc = |x: i32, y: i32| [x as f32 / sw * 2.0 - 1.0, 1.0 - y as f32 / sh * 2.0, 0.0];
    let (a, b, c, d) = (ndc(x, y), ndc(x + 1, y), ndc(x + 1, y + 1), ndc(x, y + 1));
    [a, b, c, a, c, d]
}

/// **The borders** (item 4588): `WorldMap::render_borders@008cc490` draws
/// the who texture ([`who_texels`]) on the background's own two triangles,
/// its texcoords the fog grid over the texture, `MIP_LINEAR` both ways,
/// alpha-tested above [`WHO_ALPHA_REF`] and blended `SRCALPHA`,
/// `INVSRCALPHA` with the texel's colour and alpha selected. Each pixel
/// of the quad samples the texture bilinearly at its centre (a texel past
/// the grid is zero), and a pixel that passes is one square in its
/// filtered colour, gathered into one layer per alpha.
pub fn borders(
    size: (i32, i32),
    cells: (i32, i32),
    who: &[[u8; 4]],
) -> Vec<crate::gpu::ScreenLayer> {
    let (fw, fh) = (cells.0 * 2, cells.1 * 2);
    let at = |x: i32, y: i32| {
        if x < 0 || y < 0 || x >= fw || y >= fh {
            [0.0f32; 4]
        } else {
            who[(y * fw + x) as usize].map(f32::from)
        }
    };
    let q = quad(size);
    let (x0, x1) = (q.iter().map(|p| p.0).min(), q.iter().map(|p| p.0).max());
    let (y0, y1) = (q.iter().map(|p| p.1).min(), q.iter().map(|p| p.1).max());
    let (Some(x0), Some(x1), Some(y0), Some(y1)) = (x0, x1, y0, y1) else {
        return Vec::new();
    };
    let mut by: std::collections::BTreeMap<u8, crate::gpu::ScreenLayer> = Default::default();
    for y in y0..=y1 {
        for x in x0..=x1 {
            let Some((u, v)) = to_uv(size, x as f32 + 0.5, y as f32 + 0.5) else {
                continue;
            };
            let (tx, ty) = (u * fw as f32 - 0.5, v * fh as f32 - 0.5);
            let (ix, iy) = (tx.floor() as i32, ty.floor() as i32);
            let (ax, ay) = (tx - ix as f32, ty - iy as f32);
            let (a, b, c, d) = (
                at(ix, iy),
                at(ix + 1, iy),
                at(ix, iy + 1),
                at(ix + 1, iy + 1),
            );
            let f: [f32; 4] = std::array::from_fn(|k| {
                let top = a[k] + (b[k] - a[k]) * ax;
                let bot = c[k] + (d[k] - c[k]) * ax;
                top + (bot - top) * ay
            });
            if f[3] <= WHO_ALPHA_REF {
                continue;
            }
            let l = by.entry(f[3].round().min(255.0) as u8).or_insert_with(|| {
                crate::gpu::ScreenLayer {
                    alpha: f[3].round().min(255.0) as u8,
                    pos: Vec::new(),
                    col: Vec::new(),
                }
            });
            let c = [f[0] / 255.0, f[1] / 255.0, f[2] / 255.0];
            for p in pixel(size, x, y) {
                l.pos.push(p);
                l.col.push(c);
            }
        }
    }
    by.into_values().collect()
}

/// **The view's rectangle** (item 4588): `WorldMap::draw_view_rect@008c7f70`,
/// last of `on_redraw`'s draws. `get_view_rect@008c8120` takes the plane
/// points under the main view's top-left and bottom-right pixels
/// (`RenderWindow::mouse_to_xy_plane`); each goes through the minimap's
/// camera to the window's pixels (`MiscAccess::world_to_console@00583840`,
/// rounded half up), held to `[4, size - 4]`, and `Buffer::box` draws a one-pixel
/// `WHITE` outline from the first to the second less one. `view` is the
/// main camera and `screen` its frame; the box is held to the quad's
/// bounds (an assumption: the window's rectangle is not read).
pub fn view_rect(
    size: (i32, i32),
    extent: (i32, i32),
    view: &render::Camera,
    screen: (u32, u32),
) -> crate::gpu::ScreenLayer {
    let m = view.to_screen_matrix().0;
    let (w, h) = (screen.0 as f32, screen.1 as f32);
    let ground = |px: f32, py: f32| {
        let sx = 2.0 * px / w - 1.0;
        let sy = 1.0 - 2.0 * py / h;
        let (a, b, e) = (m[0] - sx * m[3], m[4] - sx * m[7], sx * m[15] - m[12]);
        let (c, d, f) = (m[1] - sy * m[3], m[5] - sy * m[7], sy * m[15] - m[13]);
        let det = a * d - b * c;
        ((e * d - b * f) / det, (a * f - e * c) / det)
    };
    let console = |px: f32, py: f32| {
        let (x, y) = ground(px, py);
        let (sx, sy) = to_screen(size, extent, x as i32, y as i32);
        // `world_to_console@00583840`: truncated, then one more when the
        // remainder is at least `__real_3f000000`, 0.5.
        let round = |v: f32| v as i32 + i32::from(v - (v as i32) as f32 >= 0.5);
        (round(sx), round(sy))
    };
    let q = quad(size);
    let lo = (q[3].0 + 4, q[0].1 + 4);
    let hi = (q[1].0 - 4, q[2].1 - 4);
    let (a, b) = (console(0.0, 0.0), console(w, h));
    let (x0, y0) = (a.0.max(lo.0), a.1.max(lo.1));
    let (x1, y1) = (b.0.min(hi.0) - 1, b.1.min(hi.1) - 1);
    let mut l = crate::gpu::ScreenLayer {
        alpha: 255,
        pos: Vec::new(),
        col: Vec::new(),
    };
    let mut put = |x: i32, y: i32| {
        for p in pixel(size, x, y) {
            l.pos.push(p);
            l.col.push([1.0; 3]);
        }
    };
    for x in x0..=x1 {
        put(x, y0);
        put(x, y1);
    }
    for y in y0 + 1..y1 {
        put(x0, y);
        put(x1, y);
    }
    l
}

/// **The overlays** (item 4588; `docs/RENDERER.md` §3.32): what
/// `WorldMap::on_redraw@008c81f0` draws over the background, in its order
/// (`render_borders`, `render_fog`, `render_blips`, `draw_view_rect`,
/// `render_ping`): the borders ([`borders`]) and the view's rectangle
/// ([`view_rect`]). The fog draws nothing on run1564's photograph and is
/// not built; nor are the blips and the pings.
pub fn overlays(
    world: &crate::world::World,
    view: &render::Camera,
    screen: (u32, u32),
) -> Vec<crate::gpu::ScreenLayer> {
    let size = (screen.0 as i32, screen.1 as i32);
    let t = world.runner.terrain();
    let terr = world.runner.territory();
    let cells = (t.width, t.height);
    let mut out = Vec::new();
    if terr.width == t.width && terr.height == t.height {
        let sea =
            |i: usize| t.flags[i] & sim::world::cell::HALFLAND == 0 && matches!(t.land[i], 1 | 2);
        let colour = |p: sim::Player| {
            crate::borders::BORDER_COLOURS
                .get(usize::from(p))
                .copied()
                .unwrap_or([255; 3])
        };
        let who = who_texels(cells, &terr.owner, sea, colour);
        out.extend(borders(size, cells, &who));
    }
    out.push(view_rect(
        size,
        (cells.0 * CELL, cells.1 * CELL),
        view,
        screen,
    ));
    out
}

/// **The background drawn** (item 4581): the terrain alone, no figure, no
/// tree, no sprite, through [`background_camera`], on `gpu`. The world's
/// screen ramp is laid for this camera and left for the caller to lay
/// again.
pub fn background(world: &mut crate::world::World, gpu: &crate::gpu::Gpu) -> Background {
    let t = world.runner.terrain();
    let extent = (
        t.width * crate::input::TILES_PER_CELL * crate::input::TILE,
        t.height * crate::input::TILES_PER_CELL * crate::input::TILE,
    );
    let camera = background_camera(extent);
    let size = (BACK, BACK);
    // Item 4584 (§3.32): the coral on the faded sand's light and deep grey,
    // as draw 131 carries the sand layer's vertex colours (§3.18), for this
    // draw alone; the window's own stays as it stands.
    let coral_lit = world.coral_lit();
    world.set_coral_lit(true);
    let version = world.runner.terrain().version;
    world.set_screen(&camera, size);
    world.drawn_terrain();
    let (patch_version, surf_version) = (world.patch_version(), world.surf_version());
    // Item 4584 (§3.32): under `NO_HDR` there is no HDR target, so every
    // layer is blended straight onto the background's own.
    let (land_hdr, surf_hdr, border_hdr) = (false, false, false);
    let objects_before_water = false;
    let gradient = world.minimap_colors.map(|c| [c.ocean, c.ocean_bottom]);
    let (t, ground, patch) = world.drawn_terrain();
    let frame = crate::gpu::Frame {
        terrain_version: version,
        terrain: (&t.positions, &t.colours),
        ground,
        patch,
        patch_version,
        surf_version,
        land_hdr,
        surf_hdr,
        border_hdr,
        objects_before_water,
        figures: (&[], &[]),
        textured: &[],
        trees: &[],
        overlay: (&[], &[]),
        underlay: &[],
        screen: &[],
        sprites: &[],
        shadows: &[],
        particles: &[],
        minimap: gradient,
    };
    let mut rgba = gpu.still(size, &camera, &frame);
    // The window's surface is BGRA, and the still reads it back as is.
    if matches!(
        gpu.format,
        wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb
    ) {
        rgba.as_chunks_mut::<4>()
            .0
            .iter_mut()
            .for_each(|p| p.swap(0, 2));
    }
    if let Some(c) = world.minimap_colors {
        let t = world.runner.terrain();
        paint_cells(
            &mut rgba,
            (t.width, t.height),
            |i| t.flags[i] & sim::world::cell::FOREST != 0,
            c.tree,
        );
        add_mountains(
            &mut rgba,
            (t.width, t.height),
            &t.flags,
            &t.tiles,
            c.mountain,
        );
    }
    world.set_coral_lit(coral_lit);
    Background { rgba, extent }
}

/// **The forests** (item 4582; `docs/RENDERER.md` §3.32):
/// `WorldMapBackground::render_feature@008cf710` under `TREE_FEATURE`. Every
/// cell whose flags carry `FOREST` is laid again as its terrain mesh, its
/// 25 vertices in the tileset's `<TREES minimap_color>` at alpha `0xff`,
/// drawn `SRCALPHA`, `INVSRCALPHA` with the vertex colour selected: an
/// opaque square of that colour over the cell. The decompile's test that
/// leaves a corner tile white compares the map's tile coordinates, not the
/// cell's, so it leaves only the map's own corners, and those are not
/// kept here. A pixel takes the cell its centre falls in, as the
/// rasterizer does; the still is [`BACK`] square over the whole map.
pub fn paint_cells(
    rgba: &mut [u8],
    cells: (i32, i32),
    on: impl Fn(usize) -> bool,
    colour: [u8; 3],
) {
    let n = BACK as i32;
    for y in 0..n {
        let cy = ((2 * y + 1) * cells.1 / (2 * n)).min(cells.1 - 1);
        for x in 0..n {
            let cx = ((2 * x + 1) * cells.0 / (2 * n)).min(cells.0 - 1);
            if on((cy * cells.0 + cx) as usize) {
                let i = ((y * n + x) * 4) as usize;
                rgba[i..i + 3].copy_from_slice(&colour);
            }
        }
    }
}

/// The mountain feature's vertex alpha, `0x77` of `0xff`.
pub const MOUNTAIN_ALPHA: u8 = 0x77;

/// **The mountains** (item 4586; `docs/RENDERER.md` §3.32):
/// `WorldMapBackground::render_feature@008cf710` under `MOUNTAIN_FEATURE`,
/// drawn last. Every cell whose flags carry [`sim::world::cell::MOUNTAIN`]
/// or [`sim::world::cell::CLIFF`] (`& 0x50`) is laid again as its terrain
/// mesh, its 25 vertices white at alpha 0 (`0x00ffffff`), save the four
/// corners of each of its sixteen tiles whose object field is a cliff or a
/// mountain (`mask & 3` of 1 or 2), which take the tileset's `<MOUNTAINS
/// minimap_color>` at alpha [`MOUNTAIN_ALPHA`]. The mesh's vertices are its
/// own, so a corner on the cell's edge is coloured by the cell's own tiles
/// alone. `mountain_feature_rs` selects the vertex colour and alpha and
/// blends `SRCALPHA`, `ONE`: the colour times its alpha, both interpolated,
/// added to the background and clamped. Each tile is interpolated
/// bilinearly over its corners (an assumption: the mesh's two triangles
/// a tile interpolate linearly, along a diagonal not read).
pub fn add_mountains(
    rgba: &mut [u8],
    cells: (i32, i32),
    flags: &[u16],
    tiles: &[u16],
    colour: [u8; 3],
) {
    use sim::world::{cell, tile};
    let n = BACK as i32;
    let tw = cells.0 * 4;
    let on_cell = |cx: i32, cy: i32| {
        flags[(cy * cells.0 + cx) as usize] & (cell::MOUNTAIN | cell::CLIFF) != 0
    };
    let on_tile = |x: i32, y: i32| {
        matches!(
            tiles[(y * tw + x) as usize] & tile::OBJECT,
            tile::OBJECT_MOUNTAIN | tile::OBJECT_CLIFF
        )
    };
    // A vertex of cell `(cx, cy)` at its local corner `(vx, vy)` in 0..=4:
    // coloured when one of the cell's own tiles it bounds is.
    let vert = |cx: i32, cy: i32, vx: i32, vy: i32| {
        let mut hit = false;
        for ty in (vy - 1).max(0)..=vy.min(3) {
            for tx in (vx - 1).max(0)..=vx.min(3) {
                hit |= on_tile(cx * 4 + tx, cy * 4 + ty);
            }
        }
        hit
    };
    let a = f32::from(MOUNTAIN_ALPHA) / 255.0;
    let c = colour.map(|v| f32::from(v) / 255.0);
    for y in 0..n {
        let fy = (y as f32 + 0.5) / n as f32 * (cells.1 * 4) as f32;
        let cy = ((fy / 4.0) as i32).min(cells.1 - 1);
        let ly = fy - (cy * 4) as f32;
        let vy = (ly as i32).min(3);
        let ay = ly - vy as f32;
        for x in 0..n {
            let fx = (x as f32 + 0.5) / n as f32 * tw as f32;
            let cx = ((fx / 4.0) as i32).min(cells.0 - 1);
            if !on_cell(cx, cy) {
                continue;
            }
            let lx = fx - (cx * 4) as f32;
            let vx = (lx as i32).min(3);
            let ax = lx - vx as f32;
            let w = [
                ((1.0 - ax) * (1.0 - ay), vert(cx, cy, vx, vy)),
                (ax * (1.0 - ay), vert(cx, cy, vx + 1, vy)),
                ((1.0 - ax) * ay, vert(cx, cy, vx, vy + 1)),
                (ax * ay, vert(cx, cy, vx + 1, vy + 1)),
            ];
            let (mut alpha, mut col) = (0.0f32, [0.0f32; 3]);
            for (wt, hit) in w {
                if hit {
                    alpha += wt * a;
                }
                for (v, ck) in col.iter_mut().zip(c) {
                    *v += wt * if hit { ck } else { 1.0 };
                }
            }
            if alpha <= 0.0 {
                continue;
            }
            let i = ((y * n + x) * 4) as usize;
            for k in 0..3 {
                let add = (col[k] * alpha * 255.0).round();
                rgba[i + k] = (f32::from(rgba[i + k]) + add).min(255.0) as u8;
            }
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// Our camera code builds the packet's `to_screen_matrix` from
    /// [`minimap_camera`]'s fields: `WorldMap +0x808 +0x240`, printed to
    /// six decimals on run1564's packet.
    #[test]
    fn the_minimap_camera_is_the_packet_s() {
        let m = minimap_camera().to_screen_matrix().0;
        let packet: [f32; 16] = [
            1.707106, -1.707107, 0.0, 0.0, -1.707107, -1.707106, 0.0, 0.0, 0.0, 0.0, -7.605044,
            0.0, 0.007073, 78663.47, 29203.375, 78429.1,
        ];
        for k in 0..16 {
            let tol = 1e-5_f32.max(packet[k].abs() * 1e-6);
            assert!(
                (m[k] - packet[k]).abs() <= tol.max(0.01),
                "to_screen[{k}]: ours {} packet {}",
                m[k],
                packet[k]
            );
        }
        // Seen red: dir 44 parts on [0].
    }

    /// The minimap camera sends world (0, 0) to the top corner and
    /// (extent, 0) to the right one: the texcoords [`QUAD`] carries.
    #[test]
    fn the_texture_s_u_is_the_world_s_x() {
        let m = minimap_camera().to_screen_matrix();
        let s = crate::pinned_screen();
        let at = |x, y| render::camera::world_to_console(&m, &s, x, y, 0, Some((317, 159)));
        let (top, right, bottom, left) = (at(0, 0), at(46080, 0), at(46080, 46080), at(0, 46080));
        assert!(top.y < left.y && top.y < right.y, "{top:?}");
        assert!(right.x > top.x && right.x > bottom.x, "{right:?}");
        assert!(bottom.y > left.y && bottom.y > right.y, "{bottom:?}");
        assert!(left.x < top.x && left.x < bottom.x, "{left:?}");
    }

    /// [`background_camera`]'s still puts world (0, 0) top-left and
    /// (extent, 0) top-right, so its pixel rows are the texture's `v`.
    #[test]
    fn the_background_camera_lays_the_texture_s_axes() {
        let extent = (46080, 46080);
        let n = BACK as i32;
        let m = background_camera(extent).to_screen_matrix();
        let s = crate::pinned_screen();
        let at = |x, y| {
            let p = render::camera::world_to_console(&m, &s, x, y, 0, Some((n, n)));
            (p.x, p.y)
        };
        let ok = |p: (i32, i32), q: (i32, i32)| (p.0 - q.0).abs() <= 1 && (p.1 - q.1).abs() <= 1;
        let corners = [at(0, 0), at(46080, 0), at(46080, 46080), at(0, 46080)];
        println!("background corners {corners:?}");
        assert!(ok(corners[0], (0, 0)), "{corners:?}");
        assert!(ok(corners[1], (n, 0)), "{corners:?}");
        assert!(ok(corners[2], (n, n)), "{corners:?}");
        assert!(ok(corners[3], (0, n)), "{corners:?}");
    }

    /// **A cell's water** (item 4581): blue over red by more than 40 and
    /// not under green by more than 40. The photograph's lakes read
    /// (21, 130, 160) at the centre to (90, 170, 180) at the shore; its
    /// grass, sand, forest and rock all read blue under red.
    pub fn water(c: [u8; 3]) -> bool {
        let [r, g, b] = c.map(i32::from);
        b > r + 40 && b + 40 >= g
    }

    /// A cell equal on both sides: every channel within this many steps.
    pub const NEAR: i32 = 32;

    /// **`RUN1564_3279_MINIMAP`** (item 4581; `docs/RENDERER.md` §3.32): of
    /// run1564's 60 × 60 cells, each read at its centre's pixel through
    /// [`to_screen`] on the photograph and on the client's still,
    /// `[water on both, water on theirs, water on ours, the same side of
    /// water, every channel within NEAR, cells]`. Minted at the first act,
    /// the minimap drawn: 3,354 of 3,600 on the same side of water, 2,288
    /// within [`NEAR`]. Seen red: the background a quarter turn off
    /// (`dir_angle` 90) gives `[186, 596, 480, 2896, 1444, 3600]`. Item
    /// 4582, the forests ([`paint_cells`]): within `NEAR` 2,288 → 2,419
    /// (+131), the water counts unmoved; without them the row reads 2,288
    /// again. Item 4584, the background `NO_HDR` over the ocean gradient
    /// with no floor, water or surf, and the coral on the sand's light:
    /// `[425, 596, 500, 3354, 2419, 3600]` → `[416, 596, 475, 3361, 2629,
    /// 3600]`, within `NEAR` +210. Seen red: with the coral at its white
    /// vertex, `[424, 596, 492, 3360, 2398, 3600]`. Item 4586, the
    /// mountain feature ([`add_mountains`]): within `NEAR` 2,629 → 2,769
    /// (+140; 151 cells in, 11 out), the water counts unmoved; without it
    /// the row reads 2,629 again. Item 4588, the overlays ([`overlays`]):
    /// `[416, 596, 475, 3361, 2769, 3600]` → `[465, 596, 534, 3400, 2808,
    /// 3600]`, within `NEAR` +39 (50 cells in, 11 out): the borders 40 in
    /// (21 of the blue team's, 19 of the red's) and 11 out, the view's
    /// rectangle 10 in; the border's blue reads as water on both sides,
    /// +49 cells. Seen red: the view's rectangle truncated rather than
    /// rounded half up sits a pixel left of theirs, `[…, 2795, 3600]`.
    pub const RUN1564_3279_MINIMAP: [usize; 6] = [465, 596, 534, 3400, 2808, 3600];

    /// **The who texture's edge** (item 4588): a 2 × 1 map, player 0 on the
    /// left cell and player 1 on the right; then the right cell open water.
    #[test]
    fn the_who_texture_marks_the_edge_alone() {
        let o = [sim::Owner::Player(0), sim::Owner::Player(1)];
        let colour = |p: sim::Player| [p * 100, 0, 0];
        let who = who_texels((2, 1), &o, |_| false, colour);
        // Fog grid 4 × 2: texels 0 and 3 are a fog cell from the other
        // owner's cell; 1 and 2 touch it.
        let a: Vec<u8> = who.iter().map(|t| t[3]).collect();
        assert_eq!(a, [WHO_INSIDE, WHO_EDGE, WHO_EDGE, WHO_INSIDE].repeat(2));
        assert_eq!(who[2][0], 100, "player 1's colour");
        // Open water is passed over as a neighbour: player 0's edge texel
        // no longer sees a stranger, while the water cell's own texel,
        // still player 1's, sees player 0's land.
        let who = who_texels((2, 1), &o, |i| i == 1, colour);
        assert_eq!(who[1][3], WHO_INSIDE);
        assert_eq!(who[2][3], WHO_EDGE);
        // An unowned cell is a stranger too, and is itself left zero.
        let o = [sim::Owner::Player(0), sim::Owner::None];
        let who = who_texels((2, 1), &o, |_| false, colour);
        assert_eq!(who[1][3], WHO_EDGE);
        assert_eq!(who[2], [0; 4]);
    }

    /// **The view's rectangle on run1564** (item 4588): the photograph's
    /// white box is x 458 to 499 and y 1028 to 1051, inclusive, read off
    /// its pixels; [`view_rect`] from run1564's camera draws the same box.
    /// Truncated instead of rounded, it would start at 457 and end at 498.
    #[test]
    fn the_view_rect_is_the_photograph_s() {
        let camera = crate::ground::tests::run1564().camera;
        let l = view_rect(QUAD_FRAME, (46080, 46080), &camera, (1920, 1080));
        let px = |p: [f32; 3]| {
            (
                ((p[0] + 1.0) * 0.5 * 1920.0).round() as i32,
                ((1.0 - p[1]) * 0.5 * 1080.0).round() as i32,
            )
        };
        let pts: Vec<(i32, i32)> = l.pos.iter().map(|&p| px(p)).collect();
        let xs = (pts.iter().map(|p| p.0).min(), pts.iter().map(|p| p.0).max());
        let ys = (pts.iter().map(|p| p.1).min(), pts.iter().map(|p| p.1).max());
        // A pixel's square runs to its far corner, one past.
        assert_eq!((xs, ys), ((Some(458), Some(500)), (Some(1028), Some(1052))));
    }

    #[test]
    fn the_minimap_s_cells_on_run1564() {
        use crate::ground::tests::{probe_at, run1564};
        let Some(mut p) = probe_at(run1564()) else {
            return;
        };
        let size = (p.photo.width as u32, p.photo.height as u32);
        let s = (size.0 as i32, size.1 as i32);
        assert_eq!(s, QUAD_FRAME, "the photograph's frame");
        let camera = p.camera;
        let ours = crate::still_minimap(
            &mut p.world,
            &camera,
            size,
            crate::gpu::SilhouetteOrder::Before,
            wgpu::TextureFormat::Rgba8Unorm,
            &[],
            &|_| None,
            true,
        );
        let t = p.world.runner.terrain();
        let cells = (t.width, t.height);
        let extent = (cells.0 * CELL, cells.1 * CELL);
        let w = p.photo.width;
        let px = |rgba: &[u8], x: i32, y: i32| {
            let i = (y as usize * w + x as usize) * 4;
            [rgba[i], rgba[i + 1], rgba[i + 2]]
        };
        if let Some(dir) = std::env::var_os("RON_STILL_DIR") {
            let out = std::path::PathBuf::from(dir).join("still-run1564-minimap.bmp");
            std::fs::write(&out, crate::gpu::bmp(&ours, size)).expect("the still writes");
        }
        let mut got = [0usize; 6];
        for j in 0..cells.1 {
            for i in 0..cells.0 {
                let (x, y) = to_screen(s, extent, i * CELL + CELL / 2, j * CELL + CELL / 2);
                let (x, y) = (x as i32, y as i32);
                let (a, b) = (px(&p.photo.rgba, x, y), px(&ours, x, y));
                let (wa, wb) = (water(a), water(b));
                got[0] += usize::from(wa && wb);
                got[1] += usize::from(wa);
                got[2] += usize::from(wb);
                got[3] += usize::from(wa == wb);
                got[4] +=
                    usize::from((0..3).all(|k| (i32::from(a[k]) - i32::from(b[k])).abs() <= NEAR));
                got[5] += 1;
                if std::env::var_os("RON_MINIMAP_CELLS").is_some() {
                    println!("cell ({i}, {j}) pixel ({x}, {y}) theirs {a:?} ours {b:?}");
                }
            }
        }
        println!("RUN1564_3279_MINIMAP: {got:?}");
        assert_eq!(got, RUN1564_3279_MINIMAP);
    }

    /// [`to_screen`] and [`to_world`] invert one another and put the
    /// map's corners on [`QUAD`].
    #[test]
    fn the_quad_s_mapping_inverts() {
        let (size, extent) = (QUAD_FRAME, (46080, 46080));
        let r = |p: (f32, f32)| (p.0.round() as i32, p.1.round() as i32);
        assert_eq!(r(to_screen(size, extent, 0, 0)), QUAD[0]);
        assert_eq!(r(to_screen(size, extent, 46080, 0)), QUAD[1]);
        assert_eq!(r(to_screen(size, extent, 46080, 46080)), QUAD[2]);
        assert_eq!(r(to_screen(size, extent, 0, 46080)), QUAD[3]);
        for &(x, y) in &[(36960, 38496), (100, 45000), (23040, 23040), (45000, 300)] {
            let (px, py) = to_screen(size, extent, x, y);
            let (wx, wy) = to_world(size, extent, px, py).expect("inside");
            assert!(
                (wx - x).abs() <= 2 && (wy - y).abs() <= 2,
                "({x}, {y}) → ({wx}, {wy})"
            );
        }
        assert_eq!(
            to_world(size, extent, 330.0, 912.0),
            None,
            "outside the quad"
        );
        // The held press: inside, the same point; outside, held to the map.
        let (px, py) = to_screen(size, extent, 36960, 38496);
        let (hx, hy) = to_world_held(size, extent, px, py);
        assert!(
            (hx - 36960).abs() <= 2 && (hy - 38496).abs() <= 2,
            "({hx}, {hy})"
        );
        assert_eq!(
            to_world_held(size, extent, 485.0, 880.0),
            (0, 0),
            "above the top corner"
        );
        assert_eq!(
            to_world_held(size, extent, 700.0, 990.0).0,
            46080,
            "right of the quad"
        );
        // run1564's camera centre falls on the photograph's view rectangle,
        // (459..499, 1028..1051).
        let (px, py) = to_screen(size, extent, 36960, 38496);
        assert!(
            (459.0..499.0).contains(&px) && (1028.0..1051.0).contains(&py),
            "{px} {py}"
        );
    }
}
