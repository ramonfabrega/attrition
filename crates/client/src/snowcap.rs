//! **The mountain's snowcap** (item 4275; `docs/RENDERER.md` §3.10, "The
//! peak draw" and "The snowcap, drawn").
//!
//! The tileset's `snowcap` underlay (`<UNDERLAYS>`, `t_mountainpeak.tga` on
//! the autumn tilesets), laid by `TerrainOut::add_weather_underlay
//! @00874d00` on the tiles around each range's peak and drawn by
//! `TerrainOut::render_weathering(4, 4)` after the mountains:
//! - `TerrainOut::adjust_for_mountains@008703c0` marks the cell holding a
//!   range's peak (`MountainRange::init@008998b0`'s first template vertex of
//!   greatest height, [`rondata::mountains::peak`]) unless the template's
//!   `mountain_size` is 1 (`area="sm"`), with `snowcap_size` [`SIDE`];
//! - `CoordInfo::add_geometry@0084bef0` lays [`SIDE`] × [`SIDE`] tiles from
//!   the peak less half, clamped to the map, and only on a tileset whose
//!   `snowcaps` flag is set;
//! - `add_weather_underlay` steps an odd corner back one tile and grows that
//!   side to even, gives UV1 the corner's offset over the side times ½ plus
//!   the quadrant's ½ ([`Patch::offset`]) and UV0 that less the offset,
//!   times 2.
//!
//! The quadrant is `random_map_pool` at the corner before the step
//! ([`random_map_pool`]); every operation on it is single precision on the
//! listing (`874e21`–`874e75`), so `f32` here is the original's arithmetic.

use std::sync::Arc;

use render::silhouette::Mips;

/// `snowcap_size` for a template whose `mountain_size` is 2 or 3
/// (`adjust_for_mountains`): the patch's side in tiles before the step.
pub const SIDE: i32 = 6;

/// `internal_random`'s draws before `Terrain::init@00850f70` fills the pool:
/// the 45 under `RiverDoobers::init_class` on run1567's trace
/// (`client::surf`'s [`crate::surf::DRAWS_BEFORE_SURF_1567`] counts them).
/// The GreatLakes map's; another map's count is not read.
pub const DRAWS_BEFORE_POOL: u32 = 45;

/// **`random_map_pool`** (`Terrain::init@00850f70`): `(4W + 1)²` draws of
/// `Random::get(&internal_random, 0, 100)`, row-major, after
/// [`DRAWS_BEFORE_POOL`]. Run1559b's packet holds the same 58,081 bytes
/// (`the_pool_is_the_packet_s`).
pub fn random_map_pool(width_cells: i32) -> Vec<u8> {
    let w = (4 * width_cells + 1).max(0) as usize;
    let mut rng = crate::surf::InternalRandom::after(DRAWS_BEFORE_POOL);
    (0..w * w).map(|_| rng.get(0, 100) as u8).collect()
}

/// One snowcap's tiles and its UV1 offset.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Patch {
    /// The first tile, after an odd corner's step.
    pub corner: (i32, i32),
    /// Tiles on each axis, after the step.
    pub side: (i32, i32),
    /// The quadrant, 0 to 3: ½ on u for 1, on v for 2, on both for 3.
    pub quadrant: i32,
}

impl Patch {
    /// The quadrant's UV1 offset (`874e94`–`874eb4`).
    pub fn offset(&self) -> [f32; 2] {
        match self.quadrant {
            1 => [0.5, 0.0],
            2 => [0.0, 0.5],
            3 => [0.5, 0.5],
            _ => [0.0, 0.0],
        }
    }

    /// The corner `(i, j)` tiles into the patch: UV0, then UV1. UV1 steps
    /// `(1 / side) · ½` from the offset; UV0 is UV1 less the offset, times 2.
    pub fn uvs(&self, (i, j): (i32, i32)) -> ([f32; 2], [f32; 2]) {
        let [ou, ov] = self.offset();
        let du = 1.0 / self.side.0 as f32 * 0.5;
        let dv = 1.0 / self.side.1 as f32 * 0.5;
        let (u1, v1) = (i as f32 * du + ou, j as f32 * dv + ov);
        ([(u1 - ou) * 2.0, (v1 - ov) * 2.0], [u1, v1])
    }

    /// The patch's tiles, row by row.
    pub fn tiles(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        (0..self.side.1).flat_map(move |j| {
            (0..self.side.0).map(move |i| (self.corner.0 + i, self.corner.1 + j))
        })
    }
}

/// **One range's snowcap** (`add_geometry`'s call, `add_weather_underlay`'s
/// rule): `peak` the peak's tile, `tiles_wide` the map's `4W` (the call
/// clamps both axes by it), `pool` the map's [`random_map_pool`].
pub fn patch(peak: (i32, i32), tiles_wide: i32, pool: &[u8]) -> Patch {
    let hi = tiles_wide - SIDE;
    let at = |p: i32| (p - SIDE / 2).max(0).min(hi);
    let (x, y) = (at(peak.0), at(peak.1));
    let stride = (tiles_wide + 1) as usize;
    let quadrant = pool
        .get(x.max(0) as usize * stride + y.max(0) as usize)
        .map_or(0, |&p| {
            let n = 4.0f32;
            let mut q = f32::from(p) * (n / 100.0);
            if n - 0.5 <= q {
                q = n - 1.0;
            }
            q as i32
        });
    let step = |c: i32| {
        if c & 1 != 0 {
            let s = SIDE + 1;
            (c - 1, if s & 1 != 0 { SIDE + 2 } else { s })
        } else {
            (c, SIDE)
        }
    };
    let ((cx, sx), (cy, sy)) = (step(x), step(y));
    Patch {
        corner: (cx, cy),
        side: (sx, sy),
        quadrant,
    }
}

/// The snowcap's art and each template's peak, read once.
pub struct SnowcapArt {
    /// `t_mountainpeak.tga`, tinted (`TileSet::init@0087b370` sets the
    /// tileset's tint on every underlay texture): stage 0 and stage 1 alike.
    pub mips: Arc<Mips>,
    /// Per template: its peak pixel, where its `mountain_size` is not 1.
    pub peaks: Vec<Option<(i32, i32)>>,
}

impl SnowcapArt {
    /// `None` where the tileset names no `snowcap` underlay, its `snowcaps`
    /// flag is clear, or the texture does not read.
    pub fn load(install: &rondata::Install, tileset: &str, tint: [u8; 3]) -> Option<SnowcapArt> {
        let (on, list) = rondata::artdata::underlay_textures(install, tileset).ok()?;
        if !on {
            return None;
        }
        let file = &list
            .iter()
            .find(|(id, _)| id.eq_ignore_ascii_case("snowcap"))?
            .1;
        let bytes = std::fs::read(rondata::artdata::resolve_art(install.root(), file)?).ok()?;
        let mut img = render::tga::decode(&bytes).ok()?;
        img.tint(tint);
        let areas = rondata::mountains::template_areas(install);
        let peaks = rondata::mountains::template_files(install)
            .iter()
            .zip(
                areas
                    .iter()
                    .map(String::as_str)
                    .chain(std::iter::repeat("")),
            )
            .map(|(rel, area)| {
                if area.eq_ignore_ascii_case("sm") {
                    return None;
                }
                let b = std::fs::read(rondata::artdata::resolve_art(install.root(), rel)?).ok()?;
                rondata::mountains::peak(&rondata::mountains::decode_tga(&b)?)
            })
            .collect();
        Some(SnowcapArt {
            mips: Arc::new(Mips::new(img)),
            peaks,
        })
    }

    /// Each placed range's [`Patch`]: `ranges` as
    /// [`crate::ground::MountainArt::ranges`] gives them, the template's
    /// peak pixel at `(px − 128) / 4` tiles from the range's origin cell.
    pub fn patches(
        &self,
        ranges: &[Option<(usize, (i32, i32))>],
        width_cells: i32,
        pool: &[u8],
    ) -> Vec<Patch> {
        use render::terrain::TILES_PER_CELL;
        ranges
            .iter()
            .flatten()
            .filter_map(|&(k, (lx, ly))| {
                let (px, py) = (*self.peaks.get(k)?)?;
                let peak = (
                    (px - 128) / 4 + TILES_PER_CELL * lx,
                    (py - 128) / 4 + TILES_PER_CELL * ly,
                );
                Some(patch(peak, TILES_PER_CELL * width_cells, pool))
            })
            .collect()
    }
}

/// **The snowcaps as one [`crate::world::PatchMesh`]**: each patch tile on
/// the ground's own corners (`positions`, `TerrainMesh::build`'s `[a, b, d]`,
/// `[b, c, d]` per tile of a `tiles`-wide map), UV0 and UV1 by
/// [`Patch::uvs`], and each corner's N·L by `nl` (the land's, item 4217),
/// which `fs_snowcap` takes to `light_vert`'s colour. `None` with no tile.
pub fn mesh(
    art: &SnowcapArt,
    patches: &[Patch],
    positions: &[[f32; 3]],
    tiles: (i32, i32),
    nl: &dyn Fn(&[f32; 3]) -> f32,
) -> Option<crate::world::PatchMesh> {
    let (tw, th) = tiles;
    if positions.len() != (tw * th * 6) as usize {
        return None;
    }
    let mut m = crate::world::PatchMesh {
        piece: crate::world::SNOWCAP_PIECE,
        maps: None,
        positions: Vec::new(),
        uvs: Vec::new(),
        alpha_uvs: Vec::new(),
        mips: art.mips.clone(),
        alpha: crate::world::SNOWCAP_ALPHA,
        alpha_mips: art.mips.clone(),
        mountain: false,
        light: Vec::new(),
    };
    for p in patches {
        for (x, y) in p.tiles() {
            if !(0..tw).contains(&x) || !(0..th).contains(&y) {
                continue;
            }
            let k = ((y * tw + x) * 6) as usize;
            let (i, j) = (x - p.corner.0, y - p.corner.1);
            let (pa, pb, pd, pc) = (
                positions[k],
                positions[k + 1],
                positions[k + 2],
                positions[k + 4],
            );
            let (a, b, c, d) = (
                p.uvs((i, j)),
                p.uvs((i + 1, j)),
                p.uvs((i + 1, j + 1)),
                p.uvs((i, j + 1)),
            );
            for (q, uv) in [(pa, a), (pb, b), (pd, d), (pb, b), (pc, c), (pd, d)] {
                m.positions.push(q);
                m.uvs.push(uv.0);
                m.alpha_uvs.push(uv.1);
                m.light.push(nl(&q));
            }
        }
    }
    (!m.positions.is_empty()).then_some(m)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    /// Our corner: UV0, UV1 and N·L.
    type OurCorner = ([f32; 2], [f32; 2], f32);

    /// A corner's packet fields: its colour (`0xAABBGGRR`), UV0 and UV1.
    type Corner = (u32, [f32; 2], [f32; 2]);

    /// **run1559b's underlays at frame 3294** (`tools/render/snowcap_packet.py`
    /// on the packet): slot 4's corners by tile, its faces as corner tiles,
    /// and the pool. `None`, saying so, where the dump is absent.
    #[allow(clippy::type_complexity)]
    fn packet() -> Option<(BTreeMap<(i32, i32), Corner>, Vec<[(i32, i32); 3]>, Vec<u8>)> {
        let path = std::env::var_os("RON_SNOWCAP_RUN1559B")
            .map(std::path::PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|h| {
                    std::path::PathBuf::from(h).join(
                        "ron-data/lab-experiments/2026-11-15-item-4275/run1559b-underlays.txt",
                    )
                })
            })?;
        let Ok(text) = std::fs::read_to_string(&path) else {
            println!("skipping: no underlay dump at {}", path.display());
            return None;
        };
        let (mut slot, mut verts, mut faces, mut pool) = (-1, Vec::new(), Vec::new(), Vec::new());
        let mut in_pool = false;
        for l in text.lines() {
            let w: Vec<&str> = l.split_whitespace().collect();
            match w.first().copied() {
                Some("slot") => slot = w[1].parse().unwrap(),
                Some("pool") => in_pool = true,
                Some("v") if slot == 4 => {
                    let f = |i: usize| w[i].parse::<f32>().unwrap();
                    let tile = (w[1].parse().unwrap(), w[2].parse().unwrap());
                    let c = u32::from_str_radix(w[3], 16).unwrap();
                    verts.push((tile, (c, [f(4), f(5)], [f(6), f(7)])));
                }
                Some("f") if slot == 4 => {
                    let i = |k: usize| w[k].parse::<usize>().unwrap();
                    faces.push([i(1), i(2), i(3)]);
                }
                Some(h) if in_pool => pool.extend(
                    (0..h.len())
                        .step_by(2)
                        .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap()),
                ),
                _ => {}
            }
        }
        let faces = faces.iter().map(|f| f.map(|i| verts[i].0)).collect();
        Some((verts.into_iter().collect(), faces, pool))
    }

    /// **`random_map_pool` is the packet's** (item 4275): 58,081 of 58,081
    /// bytes on run1559b's map, and 17 at the snowcap's corner (11, 138),
    /// quadrant 0. Made to fail: one draw early, it parts.
    #[test]
    fn the_pool_is_the_packet_s() {
        let Some((_, _, theirs)) = packet() else {
            return;
        };
        let ours = random_map_pool(60);
        assert_eq!(theirs.len(), 241 * 241);
        let agree = |a: &[u8]| a.iter().zip(&theirs).filter(|(x, y)| x == y).count();
        assert_eq!(agree(&ours), 58_081);
        assert_eq!(
            (ours[241 * 11 + 138], patch((14, 141), 240, &ours).quadrant),
            (17, 0)
        );
        let mut rng = crate::surf::InternalRandom::after(DRAWS_BEFORE_POOL - 1);
        let early: Vec<u8> = (0..ours.len()).map(|_| rng.get(0, 100) as u8).collect();
        assert!(agree(&early) < 1_000, "{}", agree(&early));
    }

    /// **The snowcap the client lays on run1559b is the packet's slot 4**
    /// (item 4275; `docs/RENDERER.md` §3.10, "The snowcap, drawn"): every
    /// corner's tile, UV0 and UV1 exactly, every face's corners, and the
    /// vertex colour our N·L gives through `light_vert` against the
    /// packet's, by channel.
    #[test]
    fn the_snowcap_is_the_packet_s_slot_4() {
        use render::silhouette::{LAND_AMBIENT, LAND_DIFFUSE};
        let Some((corners, faces, _)) = packet() else {
            return;
        };
        let Some(mut p) = crate::ground::tests::probe() else {
            return;
        };
        let (_, _, patches) = p.world.drawn_terrain();
        let m = patches
            .iter()
            .find(|m| m.piece == crate::world::SNOWCAP_PIECE)
            .expect("a snowcap laid");
        let per = render::terrain::UNITS_PER_TILE as f32;
        let tile = |q: &[f32; 3]| ((q[0] / per).round() as i32, (q[1] / per).round() as i32);
        // `add_geometry` lays a snowcap only on a cell in view, so the
        // packet's slot holds the one patch on screen; ours lays every
        // range's, and the patch compared is the one over the packet's box.
        let lo = *corners.keys().min().unwrap();
        let hi = *corners.keys().max().unwrap();
        let inside = |t: (i32, i32)| (lo.0..=hi.0).contains(&t.0) && (lo.1..=hi.1).contains(&t.1);
        let mut ours: BTreeMap<(i32, i32), OurCorner> = BTreeMap::new();
        for (i, q) in m.positions.iter().enumerate() {
            if inside(tile(q)) {
                ours.insert(tile(q), (m.uvs[i], m.alpha_uvs[i], m.light[i]));
            }
        }
        println!(
            "snowcap: ours {} tiles laid in all, the packet's box {lo:?}..={hi:?}",
            m.positions.len() / 6
        );
        assert_eq!(
            ours.keys().collect::<Vec<_>>(),
            corners.keys().collect::<Vec<_>>(),
            "the patch's corners"
        );
        let (mut uv, mut exact, mut near, mut worst) = (0, 0, 0, 0i32);
        for (t, (c, uv0, uv1)) in &corners {
            let (o0, o1, nl) = ours[t];
            uv += usize::from(o0 == *uv0 && o1 == *uv1);
            let theirs = [c & 0xff, (c >> 8) & 0xff, (c >> 16) & 0xff].map(|b| b as i32);
            let got = render::terrain::light_vert(nl, LAND_AMBIENT, LAND_DIFFUSE).map(i32::from);
            let d = (0..3).map(|k| (got[k] - theirs[k]).abs()).max().unwrap();
            exact += usize::from(d == 0);
            near += usize::from(d <= 1);
            if d > worst {
                worst = d;
                println!("corner {t:?}: ours {got:?}, theirs {theirs:?}");
            }
        }
        println!(
            "snowcap: {} corners, UVs {uv} exact, colour {exact} exact, {near} within 1, worst {worst}",
            corners.len()
        );
        let tri = |f: [(i32, i32); 3]| {
            let mut f = f;
            f.sort();
            f
        };
        let mut a: Vec<_> = faces.iter().map(|f| tri(*f)).collect();
        let mut b: Vec<_> = m
            .positions
            .chunks(3)
            .map(|c| tri([tile(&c[0]), tile(&c[1]), tile(&c[2])]))
            .filter(|f| f.iter().all(|t| inside(*t)))
            .collect();
        a.sort();
        b.sort();
        let shared = a.iter().filter(|f| b.binary_search(f).is_ok()).count();
        println!(
            "snowcap faces: theirs {}, ours {}, shared {shared}",
            a.len(),
            b.len()
        );
        assert_eq!((corners.len(), uv), (63, 63));
        assert_eq!(SNOWCAP_COLOURS, (exact, near));
        assert_eq!(SNOWCAP_FACES, (a.len(), b.len(), shared));
    }

    /// The snowcap's corners whose `light_vert` colour is the packet's
    /// exactly, and within one step, of 63.
    const SNOWCAP_COLOURS: (usize, usize) = (63, 63);
    /// The snowcap's faces: the packet's, ours, and those both lay.
    const SNOWCAP_FACES: (usize, usize, usize) = (96, 96, 96);

    #[test]
    fn an_odd_corner_steps_back_and_grows_even() {
        let pool = vec![0u8; 241 * 241];
        // m10's peak at (14, 141): corner (11, 138), x odd.
        let p = patch((14, 141), 240, &pool);
        assert_eq!((p.corner, p.side, p.quadrant), ((10, 138), (8, 6), 0));
        assert_eq!(p.tiles().count(), 48);
        // Clamped at the map's near and far edges.
        assert_eq!(patch((1, 239), 240, &pool).corner, (0, 234));
        let (uv0, uv1) = p.uvs((8, 6));
        assert_eq!((uv0, uv1), ([1.0, 1.0], [0.5, 0.5]));
    }

    #[test]
    fn the_quadrant_is_the_pool_s_quarter() {
        let mut pool = vec![0u8; 241 * 241];
        let at = 241 * 11 + 138;
        for (p, q) in [
            (0u8, 0),
            (24, 0),
            (25, 1),
            (50, 2),
            (74, 2),
            (75, 3),
            (99, 3),
        ] {
            pool[at] = p;
            assert_eq!(patch((14, 141), 240, &pool).quadrant, q, "pool {p}");
        }
        pool[at] = 60;
        let p = patch((14, 141), 240, &pool);
        assert_eq!(p.offset(), [0.0, 0.5]);
        assert_eq!(p.uvs((0, 0)), ([0.0, 0.0], [0.0, 0.5]));
    }
}
