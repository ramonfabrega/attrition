//! **The map's edge border** (item 4311; `docs/RENDERER.md` §3.15, "Item
//! 4307: the map's edge border draws it" and "Item 4311: the edge border,
//! built").
//!
//! `TerrainBorder::create_borders@008af520` calls `build_edge@008adda0`
//! once for each cell of the map's four edges at load — the top row
//! outward (0, −1), the bottom row outward (0, +1), then the left and right
//! columns outward (∓1, 0) — and each call fills one bucket
//! (`WCoordBorderData`) of `small_border_buckets` and one of
//! `large_border_buckets`. A cell's strip is four 192-unit segments along
//! the edge, two quads each: the inner from the edge's corner heights
//! (`max(0, h)`, the `+0x4a4` table) down to 0 at 1,440 out, the outer flat
//! from 1,440 to 7,200 out. The four corner cells of the left and right
//! columns go into the top and bottom rows' end buckets instead, after a
//! corner patch: one quad into the small bucket and three into the large
//! one, out to 14,400. `TerrainBorder::render@008af600` draws the small
//! list under `border_texture` and the large under `border_texture_outer`,
//! `SELECTARG1(TEXTURE)` through `TL_FAKE_LIGHT`, `ONE`/`ZERO`.
//!
//! The strip is the decompile's own arithmetic. The corner patches'
//! positions and texture coordinates reach `Vert3Array::add` and
//! `TexArray::add` as x87 arguments the decompiler drops, so [`CORNERS`]
//! is the packet's record of them (`tools/render/edge_packet.py`), every
//! value checked against it by `the_buckets_are_the_packet_s`.

use std::sync::Arc;

use render::silhouette::Mips;
use render::terrain::Heightfield;

/// Units a segment runs along the edge (`0x43400000`).
pub const SEGMENT: f32 = 192.0;
/// Segments a cell (`0x40800000`).
pub const SEGMENTS: i32 = 4;
/// Units a cell (`SEGMENT · SEGMENTS`, `0x44400000`).
pub const CELL: f32 = 768.0;
/// Where the inner quad meets the outer, out from the edge
/// (`SEGMENT · 7.5`, `0x40f00000`).
pub const INNER: f32 = 1_440.0;
/// Where the outer quad ends (`INNER · 5`, `0x40a00000`).
pub const OUTER: f32 = 7_200.0;

/// One vertex: its position and its texcoord.
pub type Vertex = ([f32; 3], [f32; 2]);

/// One cell's bucket: the small list and the large, each its vertices and
/// its first face list (`+0x1c`), the one `add_render_bucket@008afdb0`
/// appends unless `zoom_level < 5` or the system grade is above 2.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Bucket {
    pub small: Vec<Vertex>,
    pub small_faces: Vec<[u16; 3]>,
    pub large: Vec<Vertex>,
    pub large_faces: Vec<[u16; 3]>,
}

/// A corner patch's vertex off the map's corner: its offset, whether its z
/// is the corner's height (else 0), and its texcoord.
type CornerVertex = ((f32, f32), bool, [f32; 2]);

/// One corner's patch: the small quad's four vertices and two faces, the
/// large L's eight vertices and six faces.
struct Corner {
    small: [CornerVertex; 4],
    small_faces: [[u16; 3]; 2],
    large: [CornerVertex; 8],
    large_faces: [[u16; 3]; 6],
}

const I: f32 = INNER;
const F: f32 = 14_400.0;

/// **The four corners' patches**, top-left, top-right, bottom-left,
/// bottom-right, off run1559b's packet (frame 3294): see the module's note.
const CORNERS: [Corner; 4] = [
    Corner {
        small: [
            ((0.0, 0.0), true, [0.5, 0.5]),
            ((-I, 0.0), false, [0.0, 0.5]),
            ((0.0, -I), false, [0.5, 0.0]),
            ((-I, -I), true, [0.0, 0.0]),
        ],
        small_faces: [[0, 1, 3], [0, 3, 2]],
        large: [
            ((-I, 0.0), false, [6.0, 7.0]),
            ((-F, 0.0), false, [0.0, 7.0]),
            ((0.0, -I), false, [7.0, 6.0]),
            ((-I, -I), true, [6.0, 6.0]),
            ((-F, -I), false, [0.0, 6.0]),
            ((0.0, -F), false, [7.0, 0.0]),
            ((-I, -F), false, [6.0, 0.0]),
            ((-F, -F), false, [0.0, 0.0]),
        ],
        large_faces: [
            [1, 4, 3],
            [1, 3, 0],
            [4, 7, 6],
            [4, 6, 3],
            [3, 6, 5],
            [3, 5, 2],
        ],
    },
    Corner {
        small: [
            ((0.0, 0.0), true, [0.5, 0.5]),
            ((I, 0.0), false, [0.5, 0.0]),
            ((0.0, -I), false, [0.0, 0.5]),
            ((I, -I), false, [0.0, 0.0]),
        ],
        small_faces: [[0, 2, 3], [0, 3, 1]],
        large: [
            ((I, 0.0), false, [1.0, 7.0]),
            ((F, 0.0), false, [7.0, 7.0]),
            ((0.0, -I), false, [0.0, 6.0]),
            ((I, -I), false, [1.0, 6.0]),
            ((F, -I), false, [7.0, 6.0]),
            ((0.0, -F), false, [0.0, 0.0]),
            ((I, -F), false, [1.0, 0.0]),
            ((F, -F), false, [7.0, 0.0]),
        ],
        large_faces: [
            [3, 6, 7],
            [3, 7, 4],
            [0, 3, 4],
            [0, 4, 1],
            [2, 5, 6],
            [2, 6, 3],
        ],
    },
    Corner {
        small: [
            ((0.0, 0.0), true, [0.5, 0.5]),
            ((-I, 0.0), false, [0.5, 0.0]),
            ((0.0, I), false, [0.0, 0.5]),
            ((-I, I), false, [0.0, 0.0]),
        ],
        small_faces: [[3, 1, 0], [3, 0, 2]],
        large: [
            ((-I, 0.0), false, [6.0, 0.0]),
            ((-F, 0.0), false, [0.0, 0.0]),
            ((0.0, I), false, [7.0, 1.0]),
            ((-I, I), false, [6.0, 1.0]),
            ((-F, I), false, [1.0, 1.0]),
            ((0.0, F), false, [7.0, 7.0]),
            ((-I, F), false, [6.0, 7.0]),
            ((-F, F), false, [0.0, 7.0]),
        ],
        large_faces: [
            [7, 4, 3],
            [7, 3, 6],
            [4, 1, 0],
            [4, 0, 3],
            [6, 3, 2],
            [6, 2, 5],
        ],
    },
    Corner {
        small: [
            ((0.0, 0.0), true, [0.5, 0.5]),
            ((I, 0.0), false, [0.0, 0.5]),
            ((0.0, I), false, [0.5, 0.0]),
            ((I, I), false, [0.0, 0.0]),
        ],
        small_faces: [[2, 0, 1], [2, 1, 3]],
        large: [
            ((I, 0.0), false, [1.0, 0.0]),
            ((F, 0.0), false, [7.0, 0.0]),
            ((0.0, I), false, [0.0, 1.0]),
            ((I, I), false, [1.0, 1.0]),
            ((F, I), false, [7.0, 1.0]),
            ((0.0, F), false, [0.0, 7.0]),
            ((I, F), false, [1.0, 7.0]),
            ((F, F), false, [7.0, 7.0]),
        ],
        large_faces: [
            [3, 0, 1],
            [3, 1, 4],
            [5, 2, 3],
            [5, 3, 6],
            [6, 3, 4],
            [6, 4, 7],
        ],
    },
];

/// A corner of the `+0x4a4` table, `max(0, h)` as `build_edge` reads it.
fn height(field: &Heightfield<'_>, cx: i32, cy: i32) -> f32 {
    field.z(cx, cy).max(0.0)
}

/// **`build_edge`'s strip** for cell `(x, y)` outward `out`, along `along`,
/// appended to `b`'s small list.
fn strip(
    field: &Heightfield<'_>,
    b: &mut Bucket,
    (x, y): (i32, i32),
    out: (i32, i32),
    along: (i32, i32),
) {
    let (ox, oy) = (out.0 as f32, out.1 as f32);
    let (ax, ay) = (along.0 as f32 * SEGMENT, along.1 as f32 * SEGMENT);
    let mut px = x as f32 * CELL + if out.0 == 1 { CELL } else { 0.0 };
    let mut py = y as f32 * CELL + if out.1 == 1 { CELL } else { 0.0 };
    let mut col = x * SEGMENTS + if out.0 == 1 { SEGMENTS } else { 0 };
    let mut row = y * SEGMENTS + if out.1 == 1 { SEGMENTS } else { 0 };
    // `(param_3 == 1) || (param_4 == -1)`: the other winding.
    let flip = out.0 == 1 || out.1 == -1;
    for s in 0..SEGMENTS {
        let base = b.small.len() as u16;
        let (h0, h1) = (
            height(field, col, row),
            height(field, col + along.0, row + along.1),
        );
        let (v0, v1) = (0.5 + s as f32 / 8.0, 0.5 + (s + 1) as f32 / 8.0);
        let at = |d: f32, end: bool| {
            let e = if end { 1.0 } else { 0.0 };
            [px + e * ax + ox * d, py + e * ay + oy * d]
        };
        let p = |d: f32, end: bool, z: f32| {
            let [a, b] = at(d, end);
            [a, b, z]
        };
        b.small.extend([
            (p(0.0, false, h0), [0.0, v0]),
            (p(0.0, true, h1), [0.0, v1]),
            (p(INNER, false, 0.0), [0.5, v0]),
            (p(INNER, true, 0.0), [0.5, v1]),
            (p(INNER, false, 0.0), [v0, 0.0]),
            (p(INNER, true, 0.0), [v1, 0.0]),
            (p(OUTER, false, 0.0), [v0, 6.0]),
            (p(OUTER, true, 0.0), [v1, 6.0]),
        ]);
        let q = |i: u16| base + i;
        if flip {
            b.small_faces.extend([
                [q(0), q(2), q(3)],
                [q(0), q(3), q(1)],
                [q(4), q(6), q(7)],
                [q(4), q(7), q(5)],
            ]);
        } else {
            b.small_faces.extend([
                [q(0), q(1), q(2)],
                [q(2), q(1), q(3)],
                [q(4), q(5), q(6)],
                [q(6), q(5), q(7)],
            ]);
        }
        px += ax;
        py += ay;
        col += along.0;
        row += along.1;
    }
}

/// A corner's patch at the map's corner `at` (world units) of height `h`.
fn corner(b: &mut Bucket, c: &Corner, at: (f32, f32), h: f32) {
    let v = |&((dx, dy), high, uv): &CornerVertex| {
        ([at.0 + dx, at.1 + dy, if high { h } else { 0.0 }], uv)
    };
    let (bs, bl) = (b.small.len() as u16, b.large.len() as u16);
    b.small.extend(c.small.iter().map(v));
    b.small_faces
        .extend(c.small_faces.iter().map(|f| f.map(|i| bs + i)));
    b.large.extend(c.large.iter().map(v));
    b.large_faces
        .extend(c.large_faces.iter().map(|f| f.map(|i| bl + i)));
}

/// **The buckets, in `create_borders`' order**: the top row's cells, the
/// bottom row's, then the left column's and the right column's between
/// the corners; each corner cell of a column goes into its row's end
/// bucket, after the corner's patch. Empty on a field that is not whole.
pub fn buckets(field: &Heightfield<'_>) -> Vec<Bucket> {
    if !field.is_whole() || field.width < 2 || field.height < 2 {
        return Vec::new();
    }
    let (w, h) = (field.width, field.height);
    let mut out: Vec<Bucket> = Vec::new();
    for x in 0..w {
        let mut b = Bucket::default();
        strip(field, &mut b, (x, 0), (0, -1), (1, 0));
        out.push(b);
    }
    for x in 0..w {
        let mut b = Bucket::default();
        strip(field, &mut b, (x, h - 1), (0, 1), (1, 0));
        out.push(b);
    }
    let (cw, ch) = (w as f32 * CELL, h as f32 * CELL);
    let (tw, th) = (w * SEGMENTS, h * SEGMENTS);
    for (side, x, out_x) in [(0, 0, -1), (1, w - 1, 1)] {
        for y in 0..h {
            let corner_at = match (y, side) {
                (0, 0) => Some((0, 0, (0.0, 0.0), (0, 0))),
                (0, _) => Some((1, (w - 1) as usize, (cw, 0.0), (tw, 0))),
                (y, 0) if y == h - 1 => Some((2, w as usize, (0.0, ch), (0, th))),
                (y, _) if y == h - 1 => Some((3, (2 * w - 1) as usize, (cw, ch), (tw, th))),
                _ => None,
            };
            match corner_at {
                Some((k, i, at, (cx, cy))) => {
                    let b = &mut out[i];
                    corner(b, &CORNERS[k], at, height(field, cx, cy));
                    strip(field, b, (x, y), (out_x, 0), (0, 1));
                }
                None => {
                    let mut b = Bucket::default();
                    strip(field, &mut b, (x, y), (out_x, 0), (0, 1));
                    out.push(b);
                }
            }
        }
    }
    out
}

/// The edge border's two textures, tinted at load as `TileSet::init
/// @0087b370` tints them.
pub struct EdgeArt {
    /// `border_texture`, the small list's.
    pub good: Arc<Mips>,
    /// `border_texture_outer`, the large list's.
    pub outer: Arc<Mips>,
}

impl EdgeArt {
    /// `None` where the tileset names no `<BORDERTEX>` or a texture does not
    /// read.
    pub fn load(install: &rondata::Install, tileset: &str, tint: [u8; 3]) -> Option<EdgeArt> {
        let (good, outer) = rondata::artdata::border_textures(install, tileset).ok()?;
        let read = |file: &str| -> Option<Arc<Mips>> {
            let bytes = std::fs::read(rondata::artdata::resolve_art(install.root(), file)?).ok()?;
            let mut img = render::tga::decode(&bytes).ok()?;
            img.tint(tint);
            Some(Arc::new(Mips::new(img)))
        };
        Some(EdgeArt {
            good: read(&good)?,
            outer: read(&outer)?,
        })
    }

    /// **The edge border as two patch meshes**, the small list under
    /// `border_texture` and the large under `border_texture_outer`, each
    /// face unrolled, with a white alpha: the draw is opaque.
    pub fn meshes(&self, buckets: &[Bucket], white: &Arc<Mips>) -> Vec<crate::world::PatchMesh> {
        let mesh = |piece: i32, mips: &Arc<Mips>, large: bool| {
            let mut m = crate::world::PatchMesh {
                piece,
                maps: None,
                positions: Vec::new(),
                uvs: Vec::new(),
                alpha_uvs: Vec::new(),
                mips: mips.clone(),
                alpha: crate::world::EDGE_ALPHA,
                alpha_mips: white.clone(),
                mountain: false,
                light: Vec::new(),
            };
            for b in buckets {
                let (v, f) = if large {
                    (&b.large, &b.large_faces)
                } else {
                    (&b.small, &b.small_faces)
                };
                for &i in f.iter().flatten() {
                    let (p, uv) = v[usize::from(i)];
                    m.positions.push(p);
                    m.uvs.push(uv);
                    m.alpha_uvs.push([0.5, 0.5]);
                }
            }
            m
        };
        [
            mesh(crate::world::EDGE_PIECE, &self.good, false),
            mesh(crate::world::EDGE_OUTER_PIECE, &self.outer, true),
        ]
        .into_iter()
        .filter(|m| !m.positions.is_empty())
        .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One packet bucket: its kind and index, vertices and first face list.
    type PacketBucket = (String, usize, Vec<Vertex>, Vec<[u16; 3]>);

    /// **run1559b's buckets at frame 3294** (`tools/render/edge_packet.py`
    /// on the packet). `None`, saying so, where the dump is absent.
    fn packet() -> Option<Vec<PacketBucket>> {
        let path = std::env::var_os("RON_EDGE_RUN1559B")
            .map(std::path::PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|h| {
                    std::path::PathBuf::from(h)
                        .join("ron-data/lab-experiments/2026-11-16-item-4311/run1559b-edge.txt")
                })
            })?;
        let Ok(text) = std::fs::read_to_string(&path) else {
            println!("skipping: no edge dump at {}", path.display());
            return None;
        };
        let mut out: Vec<PacketBucket> = Vec::new();
        for l in text.lines() {
            let w: Vec<&str> = l.split_whitespace().collect();
            let f = |i: usize| w[i].parse::<f32>().unwrap();
            match w.first().copied() {
                Some("bucket") => out.push((
                    w[1].to_string(),
                    w[2].parse().unwrap(),
                    Vec::new(),
                    Vec::new(),
                )),
                Some("v") => out
                    .last_mut()
                    .unwrap()
                    .2
                    .push(([f(1), f(2), f(3)], [f(4), f(5)])),
                Some("f") => {
                    let i = |k: usize| w[k].parse::<u16>().unwrap();
                    out.last_mut().unwrap().3.push([i(1), i(2), i(3)]);
                }
                _ => {}
            }
        }
        Some(out)
    }

    /// `(vertices equal, vertices, faces equal, faces)` of `ours` against
    /// the packet's buckets, kind by kind and index by index.
    fn agree(ours: &[Bucket], theirs: &[PacketBucket]) -> [usize; 4] {
        let mut n = [0; 4];
        let mut first = true;
        for (kind, i, v, f) in theirs {
            let b = ours.get(*i);
            let (ov, of) = match (kind.as_str(), b) {
                ("small", Some(b)) => (&b.small[..], &b.small_faces[..]),
                ("large", Some(b)) => (&b.large[..], &b.large_faces[..]),
                _ => (&[][..], &[][..]),
            };
            for (k, t) in v.iter().enumerate() {
                n[1] += 1;
                let same = ov.get(k) == Some(t);
                n[0] += usize::from(same);
                if !same && first {
                    first = false;
                    println!(
                        "first parting: {kind} bucket {i} vertex {k}: theirs {t:?}, ours {:?}",
                        ov.get(k)
                    );
                }
            }
            for (k, t) in f.iter().enumerate() {
                n[3] += 1;
                n[2] += usize::from(of.get(k) == Some(t));
            }
        }
        n
    }

    /// **`RUN1559B_EDGE_BUCKETS`** (item 4311): `[vertices, faces]` of the
    /// packet's 472 buckets our [`buckets`] reproduces, value for value:
    /// whole. Made to fail (the heights one corner along), bucket 0's first
    /// vertex parts, theirs z 369.375 against 354.375.
    const RUN1559B_EDGE_BUCKETS: [(usize, usize); 2] = [(7_728, 7_728), (3_872, 3_872)];

    /// **Our buckets are the packet's** (item 4311): every vertex's
    /// position and texcoord and every face of the small and large lists,
    /// on run1559b's heights. Made to fail: the corner heights read one
    /// tile along, which parts the strips' inner rows.
    #[test]
    fn the_buckets_are_the_packet_s() {
        let Some(theirs) = packet() else { return };
        let Some(p) = crate::ground::tests::probe_at(crate::ground::tests::run1559b()) else {
            return;
        };
        let t = p.world.runner.terrain();
        let field = Heightfield {
            width: t.width,
            height: t.height,
            corners: &t.corners,
        };
        let ours = buckets(&field);
        println!(
            "ours: {} buckets, theirs {} small",
            ours.len(),
            theirs.iter().filter(|b| b.0 == "small").count()
        );
        let n = agree(&ours, &theirs);
        println!("run1559b edge buckets: {n:?}");
        // Made to fail: the table shifted one corner along x.
        let mut moved = t.corners.clone();
        moved.rotate_left(1);
        let shifted = Heightfield {
            width: t.width,
            height: t.height,
            corners: &moved,
        };
        let m = agree(&buckets(&shifted), &theirs);
        assert!(m[0] < n[0], "the comparison can fail: {m:?} against {n:?}");
        assert_eq!([(n[0], n[1]), (n[2], n[3])], RUN1559B_EDGE_BUCKETS);
    }
}
