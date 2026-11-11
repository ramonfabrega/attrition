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
}
