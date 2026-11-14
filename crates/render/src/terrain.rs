//! **The terrain heightfield** (`docs/RENDERER.md` §5.4's binary condition,
//! item 4036): the corner grid the read door reports
//! (`sim::observe::TerrainScene::corners`), as points a camera projects and
//! as the flat-coloured triangles the client draws.
//!
//! A corner `(cx, cy)` stands at world `(cx · 192, cy · 192)`, a tile's
//! width apart (`sim::UNITS_PER_TILE`), and its height is the single the
//! grid holds — `TerrainData::master_land_heights`, the table a `DUMP_ALL`
//! dump prints. The points handed to `world_to_screen` take the height
//! truncated to the integer the projection's third argument is
//! (`MiscAccess::world_to_console`, §3.4), as every other anchor does.
//!
//! **The colours are the land kind's, flat** (`WData.land`, `land_key[]`:
//! `BASELAND`, `SANDY`, `OCEAN`, `NONE`), with a placed mountain's tiles
//! grey. They are a debug palette, not the original's look: the terrain's
//! textures are §6's "After these". Nothing here opens the install.

use crate::camera::{Camera, Projected, Screen, world_to_screen};

/// A tile's width in world units (`sim::UNITS_PER_TILE`): the corner grid's
/// spacing.
pub const UNITS_PER_TILE: i32 = 192;

/// Tiles a cell is wide (`sim::TILES_PER_CELL`).
pub const TILES_PER_CELL: i32 = 4;

/// The corner grid as the door carries it.
#[derive(Clone, Copy, Debug)]
pub struct Heightfield<'a> {
    /// The map's size in cells.
    pub width: i32,
    pub height: i32,
    /// `(4·width + 1) × (4·height + 1)` singles as their bits, row-major.
    pub corners: &'a [u32],
}

impl Heightfield<'_> {
    /// Corners along x and along y.
    pub fn stride(&self) -> (i32, i32) {
        (
            self.width * TILES_PER_CELL + 1,
            self.height * TILES_PER_CELL + 1,
        )
    }

    /// Whether the grid is whole: `false` on a world with no height table.
    pub fn is_whole(&self) -> bool {
        let (sx, sy) = self.stride();
        sx > 1 && sy > 1 && self.corners.len() == (sx * sy) as usize
    }

    /// The corner's height as the single it is.
    pub fn z(&self, cx: i32, cy: i32) -> f32 {
        let (sx, _) = self.stride();
        f32::from_bits(self.corners[(cy * sx + cx) as usize])
    }

    /// The corner as the integer world point the projection takes: the
    /// height truncated toward zero (`cvttss2si`, as a `find_tcoord_z`
    /// answer is).
    pub fn point(&self, cx: i32, cy: i32) -> [i32; 3] {
        [
            cx * UNITS_PER_TILE,
            cy * UNITS_PER_TILE,
            self.z(cx, cy) as i32,
        ]
    }

    /// Every corner, row-major, with its point.
    pub fn points(&self) -> impl Iterator<Item = ((i32, i32), [i32; 3])> + '_ {
        let (sx, sy) = self.stride();
        (0..sy).flat_map(move |cy| (0..sx).map(move |cx| ((cx, cy), self.point(cx, cy))))
    }
}

/// **The binary condition's per-corner half**: every corner's pixel at
/// `camera`, as `world_to_screen` places it. The condition holds when each
/// is within one pixel of the original's projection of the dumped corner.
pub fn corner_anchors(
    field: &Heightfield<'_>,
    camera: &Camera,
    screen: &Screen,
    size: Option<(i32, i32)>,
) -> Vec<((i32, i32), [i32; 3], Projected)> {
    let m = camera.to_screen_matrix();
    field
        .points()
        .map(|(c, [x, y, z])| (c, [x, y, z], world_to_screen(&m, screen, x, y, z, size)))
        .collect()
}

/// `land_key[]`'s four kinds, the index `WData.land` holds.
pub const BASELAND: i8 = 0;
pub const SANDY: i8 = 1;
pub const OCEAN: i8 = 2;
pub const NONE: i8 = 3;

/// The debug palette: one flat colour a land kind, linear RGB.
pub fn land_colour(land: i8) -> [f32; 3] {
    match land {
        BASELAND => [0.36, 0.45, 0.16],
        SANDY => [0.30, 0.55, 0.62],
        OCEAN => [0.08, 0.20, 0.42],
        _ => [0.05, 0.05, 0.05],
    }
}

/// A placed mountain's tiles.
pub const MOUNTAIN: [f32; 3] = [0.50, 0.47, 0.42];

/// Two triangles a tile, each vertex a world point and the tile's colour,
/// shaded by the triangle's slope against a light from the north-west.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TerrainMesh {
    pub positions: Vec<[f32; 3]>,
    pub colours: Vec<[f32; 3]>,
}

impl TerrainMesh {
    /// The heightfield's triangles. `land` is one kind a cell, row-major;
    /// `mountain_tiles` the tiles a placed range covers.
    pub fn build(field: &Heightfield<'_>, land: &[i8], mountain_tiles: &[(i32, i32)]) -> Self {
        Self::build_except(field, land, mountain_tiles, &|_, _| false)
    }

    /// [`TerrainMesh::build`] without the tiles `except` names: those the
    /// client draws textured (item 4127, the ground's base layer).
    pub fn build_except(
        field: &Heightfield<'_>,
        land: &[i8],
        mountain_tiles: &[(i32, i32)],
        except: &dyn Fn(i32, i32) -> bool,
    ) -> Self {
        let mut out = TerrainMesh::default();
        if !field.is_whole() {
            return out;
        }
        let (tw, th) = (field.width * TILES_PER_CELL, field.height * TILES_PER_CELL);
        let mut mountain = vec![false; (tw * th) as usize];
        for &(tx, ty) in mountain_tiles {
            if (0..tw).contains(&tx) && (0..th).contains(&ty) {
                mountain[(ty * tw + tx) as usize] = true;
            }
        }
        let corner = |cx: i32, cy: i32| {
            [
                (cx * UNITS_PER_TILE) as f32,
                (cy * UNITS_PER_TILE) as f32,
                field.z(cx, cy),
            ]
        };
        let light = normalise([-0.4, -0.4, 0.82]);
        for ty in 0..th {
            for tx in 0..tw {
                if except(tx, ty) {
                    continue;
                }
                let base = if mountain[(ty * tw + tx) as usize] {
                    MOUNTAIN
                } else {
                    let cell = (ty / TILES_PER_CELL) * field.width + tx / TILES_PER_CELL;
                    land_colour(land.get(cell as usize).copied().unwrap_or(NONE))
                };
                let (a, b, c, d) = (
                    corner(tx, ty),
                    corner(tx + 1, ty),
                    corner(tx + 1, ty + 1),
                    corner(tx, ty + 1),
                );
                for tri in [[a, b, d], [b, c, d]] {
                    let n = normalise(cross(sub(tri[1], tri[0]), sub(tri[2], tri[0])));
                    // The grid's y runs south, so a counter-clockwise
                    // triangle's normal points down: either way up is lit.
                    let lit = 0.55 + 0.45 * dot(n, light).abs();
                    let colour = base.map(|v| v * lit);
                    for p in tri {
                        out.positions.push(p);
                        out.colours.push(colour);
                    }
                }
            }
        }
        out
    }
}

/// `MinPolySize` (`0x00b21764`, 192.0 in the image): the z the original
/// gives each triangle's unnormalised normal, a tile's width.
pub const MIN_POLY_SIZE: f32 = 192.0;

/// **The land's normals** (item 4217; `docs/RENDERER.md` §3.22), as
/// `TerrainOut::calculate_norms@008710b0` builds them over the whole
/// grid: every corner starts at zero; each tile, rows outer, adds its
/// first triangle's `(−(z[x+1,y] − z[x,y]), −(z[x,y+1] − z[x,y]),
/// MinPolySize)` to `(x, y)`, `(x+1, y)` and `(x, y+1)`
/// (`87140d`–`871486`), then its second's `(−(z[x+1,y+1] − z[x,y+1]),
/// (z[x+1,y] − z[x,y+1]) + (z[x+1,y+1] − z[x,y+1]), MinPolySize)` to
/// `(x, y+1)`, `(x+1, y)` and `(x+1, y+1)` (`87152e`–`871643`; the y is
/// the listing's sum, not a geometric normal's). Each corner is then
/// scaled by `1 / sqrt(x² + y² + z²)` unless that is 0 or 1
/// (`8716cb`–`871749`). Row-major, one a corner; empty on a grid that is
/// not whole.
pub fn land_normals(field: &Heightfield<'_>) -> Vec<[f32; 3]> {
    if !field.is_whole() {
        return Vec::new();
    }
    let (sx, sy) = field.stride();
    let (sx, sy) = (sx as usize, sy as usize);
    let z = |x: usize, y: usize| f32::from_bits(field.corners[y * sx + x]);
    let mut n = vec![[0.0f32; 3]; sx * sy];
    let mut add = |x: usize, y: usize, v: [f32; 3]| {
        let c = &mut n[y * sx + x];
        c[0] += v[0];
        c[1] += v[1];
        c[2] += v[2];
    };
    for y in 0..sy - 1 {
        for x in 0..sx - 1 {
            let first = [
                -(z(x + 1, y) - z(x, y)),
                -(z(x, y + 1) - z(x, y)),
                MIN_POLY_SIZE,
            ];
            add(x, y, first);
            add(x + 1, y, first);
            add(x, y + 1, first);
            let across = z(x + 1, y + 1) - z(x, y + 1);
            let second = [-across, (z(x + 1, y) - z(x, y + 1)) + across, MIN_POLY_SIZE];
            add(x, y + 1, second);
            add(x + 1, y, second);
            add(x + 1, y + 1, second);
        }
    }
    for c in &mut n {
        let l2 = c[0] * c[0] + c[1] * c[1] + c[2] * c[2];
        if l2 != 0.0 && l2 != 1.0 {
            let inv = 1.0 / l2.sqrt();
            *c = c.map(|v| inv * v);
        }
    }
    n
}

/// **N·L at a land corner** (item 4217): `light_vert@00866f90`'s
/// `−(n · d)` for the light's direction `d`, the y term first, then x,
/// then z, clamped to `[0, 1]`. The land's light points along
/// (−1/√2, 0, −1/√2) (`render::silhouette::LAND_N_DOT_L`'s flat case).
pub fn land_n_dot_l(n: [f32; 3]) -> f32 {
    let d = [
        -std::f32::consts::FRAC_1_SQRT_2,
        0.0,
        -std::f32::consts::FRAC_1_SQRT_2,
    ];
    let v = n[1] * d[1] * -1.0 + n[0] * d[0] * -1.0 + n[2] * d[2] * -1.0;
    v.clamp(0.0, 1.0)
}

/// **A land corner's vertex colour** (item 4217; `docs/RENDERER.md`
/// §3.22): `light_vert@00866f90` on [`land_n_dot_l`], each channel
/// `min(1, diffuse · alpha · N·L + ambient)` truncated to a byte, red in
/// the low byte (the light's `+0x140`/`+0x130` pair, its diffuse and
/// ambient red) — the byte d3dgl's input layout reads as red
/// (`DIFFUSE`, `DXGI_FORMAT_R8G8B8A8_UNORM`, `0x100dce34`). `ambient` and
/// `diffuse` are the tileset's `<LIGHTING>` (`early_autumn`: (0.01,
/// 0.03, 0.18) and 1.36). The alpha byte is not computed here.
pub fn light_vert(n_dot_l: f32, ambient: [f32; 3], diffuse: f32) -> [u8; 3] {
    ambient.map(|a| ((1.0 * diffuse * n_dot_l + a).min(1.0) * 255.0) as i32 as u8)
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn normalise(a: [f32; 3]) -> [f32; 3] {
    let l = dot(a, a).sqrt();
    if l == 0.0 { a } else { a.map(|v| v / l) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid(width: i32, height: i32, z: impl Fn(i32, i32) -> f32) -> Vec<u32> {
        let (sx, sy) = (width * 4 + 1, height * 4 + 1);
        (0..sy)
            .flat_map(|cy| (0..sx).map(move |cx| (cx, cy)))
            .map(|(cx, cy)| z(cx, cy).to_bits())
            .collect()
    }

    #[test]
    fn a_corner_is_a_tile_apart_at_its_truncated_height() {
        let corners = grid(
            2,
            1,
            |cx, cy| if (cx, cy) == (3, 2) { 250.75 } else { -4.5 },
        );
        let f = Heightfield {
            width: 2,
            height: 1,
            corners: &corners,
        };
        assert!(f.is_whole());
        assert_eq!(f.stride(), (9, 5));
        assert_eq!(f.point(3, 2), [576, 384, 250]);
        assert_eq!(f.point(0, 0), [0, 0, -4], "toward zero, as cvttss2si");
        assert_eq!(f.points().count(), 45);
    }

    #[test]
    fn a_short_grid_is_not_whole_and_builds_nothing() {
        let f = Heightfield {
            width: 2,
            height: 1,
            corners: &[0; 44],
        };
        assert!(!f.is_whole());
        assert!(TerrainMesh::build(&f, &[0, 0], &[]).positions.is_empty());
    }

    #[test]
    fn the_mesh_is_two_triangles_a_tile_coloured_by_its_cell() {
        let corners = grid(2, 1, |_, _| 0.0);
        let f = Heightfield {
            width: 2,
            height: 1,
            corners: &corners,
        };
        let m = TerrainMesh::build(&f, &[BASELAND, OCEAN], &[(0, 0)]);
        assert_eq!(m.positions.len(), 8 * 4 * 6);
        let flat = 0.55 + 0.45 * normalise([-0.4, -0.4, 0.82])[2];
        let tile = |tx: usize, ty: usize| m.colours[(ty * 8 + tx) * 6];
        assert_eq!(tile(0, 0), MOUNTAIN.map(|v| v * flat));
        assert_eq!(tile(1, 0), land_colour(BASELAND).map(|v| v * flat));
        assert_eq!(tile(4, 3), land_colour(OCEAN).map(|v| v * flat));
        let e = TerrainMesh::build_except(&f, &[BASELAND, OCEAN], &[(0, 0)], &|tx, _| tx < 4);
        assert_eq!(
            e.positions.len(),
            4 * 4 * 6,
            "the excepted tiles are not drawn"
        );
        assert_eq!(e.colours[0], land_colour(OCEAN).map(|v| v * flat));
    }

    /// Item 4217: a flat grid's normals point straight up and its corners
    /// take the flat N·L and `light_vert`'s (247, 252, 255); a grid that
    /// rises along x tips every normal away from the light (`−x`), and
    /// its N·L falls under the flat one.
    #[test]
    fn the_land_s_normals_and_vertex_colour() {
        let flat = grid(1, 1, |_, _| 10.0);
        let f = Heightfield {
            width: 1,
            height: 1,
            corners: &flat,
        };
        let n = land_normals(&f);
        assert_eq!(n.len(), 25);
        assert!(n.iter().all(|v| *v == [0.0, 0.0, 1.0]), "{:?}", n[12]);
        let l = land_n_dot_l(n[12]);
        assert_eq!(l, std::f32::consts::FRAC_1_SQRT_2);
        assert_eq!(light_vert(l, [0.01, 0.03, 0.18], 1.36), [247, 252, 255]);
        let rising = grid(1, 1, |x, _| 40.0 * x as f32);
        let r = Heightfield {
            width: 1,
            height: 1,
            corners: &rising,
        };
        let n = land_normals(&r);
        assert!(n[12][0] < 0.0, "{:?}", n[12]);
        assert!(land_n_dot_l(n[12]) < l);
        assert_eq!(light_vert(0.0, [0.01, 0.03, 0.18], 1.36), [2, 7, 45]);
    }
}
