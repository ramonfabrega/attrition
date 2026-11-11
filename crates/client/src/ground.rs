//! **The ground's base layer** (item 4127; `docs/RENDERER.md` §3.6, "The
//! ground's base layer"): what a `BASELAND` tile draws, read from the
//! terrain's loader and its draw.
//!
//! - `TerrainOut::generate_land_lists@0085f6a0` hands each cell's `land`
//!   (`WData +2`) and `land_sub` (`+3`) to `TerrainList::add_new_coord_info
//!   @0084ba10`, which buckets the cell by `TileSet::convert_land_tex_to_linear
//!   @0087ace0`: on `BASELAND` (land 0) the `land_sub`th `<BASE>`'s first
//!   texture, plus `(cx mod 3) + 3 · (cy mod 3)` over the cell's own
//!   coordinates.
//! - A `supertile="1"` base's file is cut 3 × 3 by
//!   `TextureEffect::load_as_supertile@00542640`: sub-tile `k` is the
//!   `w/3`-square at column `k mod 3`, row `k / 3` of the loaded image
//!   (top row first, as `render::tga` gives it and as the figures' textures
//!   already sample).
//! - `TerrainOut::generate_land@0085f8d0` gives corner `(x, y)` the
//!   texcoord `((x & 3)/4 + (x >> 2), (y & 3)/4 + (y >> 2))` (`85fb9a`–
//!   `85fc7e`): one sub-tile a cell, u along world x, v along world y.
//! - The vertex colour `determine_land_height_color@0086c1e0` writes is
//!   `0xffffffff` on every path: white. The light is not drawn here.

use render::silhouette::Mips;
use std::path::Path;

/// The sub-tile a `BASELAND` cell samples, `(cx mod 3) + 3 · (cy mod 3)`
/// (`convert_land_tex_to_linear`).
pub fn subtile(cx: i32, cy: i32) -> usize {
    (cx.rem_euclid(3) + 3 * cy.rem_euclid(3)) as usize
}

/// Sub-tile `k` of a supertile, `load_as_supertile`'s cut: the `w/3` by
/// `h/3` block at column `k mod 3`, row `k / 3`.
pub fn cut(image: &render::tga::Image, k: usize) -> render::tga::Image {
    let (w, h) = (image.width / 3, image.height / 3);
    let (x0, y0) = ((k % 3) * w, (k / 3) * h);
    let mut rgba = Vec::with_capacity(w * h * 4);
    for y in y0..y0 + h {
        let at = (y * image.width + x0) * 4;
        rgba.extend_from_slice(&image.rgba[at..at + w * 4]);
    }
    render::tga::Image {
        width: w,
        height: h,
        rgba,
        ..image.clone()
    }
}

/// A tile's four corners' texcoords inside its cell's sub-tile, in the
/// order `(tx, ty)`, `(tx + 1, ty)`, `(tx + 1, ty + 1)`, `(tx, ty + 1)`:
/// `generate_land`'s `corner / 4` less the cell, so `[0, 1]` across a cell.
pub fn tile_uvs(tx: i32, ty: i32) -> [[f32; 2]; 4] {
    let (u0, v0) = (tx.rem_euclid(4) as f32 / 4.0, ty.rem_euclid(4) as f32 / 4.0);
    let (u1, v1) = (u0 + 0.25, v0 + 0.25);
    [[u0, v0], [u1, v0], [u1, v1], [u0, v1]]
}

/// The lobby tileset's `BASELAND` textures, each base's sub-tiles as mips.
pub struct GroundTextures {
    /// The tileset's bases as read, which the probe prints.
    #[cfg_attr(not(test), allow(dead_code))]
    pub art: rondata::artdata::GroundArt,
    /// `bases[land_sub][subtile]`; a base that is not a supertile holds one.
    pub bases: Vec<Vec<Mips>>,
}

impl GroundTextures {
    pub fn load(root: &Path, tileset: &str) -> Result<GroundTextures, String> {
        let install = rondata::Install::new(root);
        let art = rondata::artdata::ground_art(&install, tileset).map_err(|e| format!("{e:?}"))?;
        let mut bases = Vec::new();
        for b in &art.baseland {
            let path = rondata::artdata::resolve_art(root, &b.texture)
                .ok_or_else(|| format!("no file for {}", b.texture))?;
            let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let image = render::tga::decode(&bytes).map_err(|e| format!("{}: {e:?}", b.texture))?;
            bases.push(if b.supertile {
                (0..9).map(|k| Mips::new(cut(&image, k))).collect()
            } else {
                vec![Mips::new(image)]
            });
        }
        Ok(GroundTextures { art, bases })
    }

    /// The texture a `BASELAND` cell draws: `(base, sub-tile)` as an index
    /// into [`GroundTextures::flat`], `None` past the tileset's bases.
    pub fn index(&self, land_sub: u8, cx: i32, cy: i32) -> Option<usize> {
        let b = usize::from(land_sub);
        let n = self.bases.get(b)?.len();
        let k = if n == 9 { subtile(cx, cy) } else { 0 };
        Some(self.bases[..b].iter().map(Vec::len).sum::<usize>() + k)
    }

    /// Every sub-tile, base by base.
    pub fn flat(&self) -> Vec<Mips> {
        self.bases.iter().flatten().cloned().collect()
    }
}

/// **The base layer's triangles** (item 4127): every `BASELAND` tile not
/// under a placed mountain, its two triangles as [`TerrainMesh`] holds
/// them, each with its corners' texcoords, its texture's index into
/// [`GroundTextures::flat`] and its tile.
///
/// [`TerrainMesh`]: render::terrain::TerrainMesh
#[derive(Clone, Debug, Default)]
pub struct GroundBase {
    pub tris: Vec<[[f32; 3]; 3]>,
    pub uvs: Vec<[[f32; 2]; 3]>,
    pub of: Vec<usize>,
    pub tile: Vec<(i32, i32)>,
}

pub fn ground_base(
    t: &sim::observe::TerrainScene,
    mountains: &std::collections::BTreeSet<(i32, i32)>,
    tex: &GroundTextures,
    mesh: &render::terrain::TerrainMesh,
) -> GroundBase {
    use render::terrain::{BASELAND, TILES_PER_CELL};
    let mut out = GroundBase::default();
    let (tw, th) = (t.width * TILES_PER_CELL, t.height * TILES_PER_CELL);
    if mesh.positions.len() != (tw * th * 6) as usize {
        return out;
    }
    for ty in 0..th {
        for tx in 0..tw {
            let (cx, cy) = (tx / TILES_PER_CELL, ty / TILES_PER_CELL);
            let c = (cy * t.width + cx) as usize;
            if t.land.get(c) != Some(&BASELAND) || mountains.contains(&(tx, ty)) {
                continue;
            }
            let Some(k) = tex.index(t.land_sub[c], cx, cy) else {
                continue;
            };
            let i = ((ty * tw + tx) * 6) as usize;
            let [a, b, cc, d] = tile_uvs(tx, ty);
            // `TerrainMesh::build`'s order: [a, b, d], then [b, c, d].
            for (j, uv) in [[a, b, d], [b, cc, d]].into_iter().enumerate() {
                let v = &mesh.positions[i + j * 3..i + j * 3 + 3];
                out.tris.push([v[0], v[1], v[2]]);
                out.uvs.push(uv);
                out.of.push(k);
                out.tile.push((tx, ty));
            }
        }
    }
    out
}

/// **`RUN1559B_3279_GROUND`** (item 4127; `docs/RENDERER.md` §3.6, "The
/// ground's base layer"): of the scored tiles on run1559b's frame 3279 at
/// its pinned camera, those whose drawn mean colour is the photograph's
/// within the colour reader's 0.05 in (r, g)/(r + g + b). Item 4129: it
/// backs the tileset and the mapping, not the choice of base — every tile
/// drawn in base 2 scores 1,053 and in base 0 1,052; the choice's row is
/// [`RUN1559B_3279_GROUND_BASE`].
#[cfg(test)]
pub const RUN1559B_3279_GROUND: (usize, usize) = (949, 1109);

/// The same row on the debug colour, the floor (item 4127).
#[cfg(test)]
pub const RUN1559B_3279_GROUND_FLOOR: (usize, usize) = (92, 1109);

/// **`RUN1559B_3279_GROUND_BASE`** (item 4129; `docs/RENDERER.md` §3.6,
/// "Which base a tile draws"): of the 1,003 scored plain tiles, those where
/// the base our draw lays is the best of the four by the per-tile Pearson r
/// of the photograph's luminance with each base's own sub-tile at the read
/// mapping, and stands above its own control, the same base with the
/// sub-tile's cell index one on. New, 758 of 1,003; its controls, every
/// tile one base, are [`RUN1559B_3279_GROUND_BASE_ONE`].
#[cfg(test)]
pub const RUN1559B_3279_GROUND_BASE: (usize, usize) = (758, 1003);

/// The same row with every plain tile drawn in base 0, 1, 2, 3 (item 4129).
#[cfg(test)]
pub const RUN1559B_3279_GROUND_BASE_ONE: [usize; 4] = [2, 274, 697, 26];

/// Run1559b's packet against our bucket on its 2,810 land-0 cells (item
/// 4129; `tools/render/land_buckets.py`): the `pCoords` list a cell sits
/// in equals our bucket on the first, and the rule on the packet's own
/// `land_sub` on the second; the 31 between are cells whose `land_sub` the
/// original's `spot_update_land` moved and our terrain has not.
#[cfg(test)]
pub const RUN1559B_3279_PACKET_LIST: (u32, u32, u32) = (2779, 2810, 2810);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::silhouette::{INTERFACE, PHOTO_FRAME, TRACED, frame_scene, photo_path};
    use crate::world::{TreePixels, World};
    use render::silhouette::{HUE, Object, chromaticity, inside, rasterize_cut};
    use render::terrain::{TILES_PER_CELL, land_colour};
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    #[test]
    fn a_cell_takes_its_sub_tile_by_its_own_coordinates() {
        assert_eq!(subtile(0, 0), 0);
        assert_eq!(subtile(4, 0), 1);
        assert_eq!(subtile(2, 5), 8);
        assert_eq!(subtile(-1, 0), 2);
        assert_eq!(
            tile_uvs(5, 7),
            [[0.25, 0.75], [0.5, 0.75], [0.5, 1.0], [0.25, 1.0]]
        );
    }

    #[test]
    fn a_supertile_is_cut_three_by_three_from_the_top_row() {
        let image = render::tga::Image {
            width: 6,
            height: 6,
            kind: 2,
            bits: 32,
            footer: false,
            rgba: (0..36u8).flat_map(|i| [i, 0, 0, 255]).collect(),
        };
        let c = cut(&image, 5);
        assert_eq!((c.width, c.height), (2, 2));
        // Column 2, row 1: pixel (4, 2) of the top-row-first image.
        assert_eq!(c.rgba[0], 2 * 6 + 4);
        assert_eq!(c.rgba[4 * 3], 3 * 6 + 5);
    }

    /// The probe's state at the photograph's frame.
    pub(crate) struct Probe {
        pub world: World,
        pub photo: render::tga::Image,
        pub camera: render::Camera,
        pub root: PathBuf,
    }

    pub(crate) fn probe() -> Option<Probe> {
        let root = std::env::var_os("RON_INSTALL")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("../../game"));
        let path = photo_path();
        let Ok(bytes) = std::fs::read(&path) else {
            println!("skipping: no photograph at {}", path.display());
            return None;
        };
        let photo = render::tga::decode(&bytes).expect("the photograph decodes");
        let mut world = match World::open(&root, &crate::world::gamelog_dir()) {
            Ok(w) => w,
            Err(e) => {
                println!("skipping: {e}");
                return None;
            }
        };
        world.run_to(PHOTO_FRAME);
        world.blend = crate::world::Blend::Last;
        world.run_to(PHOTO_FRAME + 1);
        Some(Probe {
            world,
            photo,
            camera: crate::pinned_camera(),
            root,
        })
    }

    /// **The client's draw** (item 4127, step 4): every tile of the
    /// heightfield is drawn once — the `BASELAND` tiles the base layer
    /// textures, each in the mesh of its own texture, and the rest in the
    /// debug colours — and `debug_ground` draws the whole heightfield as the
    /// floor did. The readers' [`World::terrain`] stays whole either way.
    #[test]
    fn the_client_draws_every_tile_once() {
        let Some(mut p) = probe() else { return };
        let full = p.world.terrain().clone();
        let t = p.world.runner.terrain();
        let mountains: std::collections::BTreeSet<(i32, i32)> =
            t.mountains.iter().flatten().map(|q| (q.x, q.y)).collect();
        let tileset = crate::world::lobby_tileset(
            &p.root,
            &crate::world::gamelog_dir().join(crate::world::START_DUMP),
        )
        .expect("the lobby's tileset");
        let tex = GroundTextures::load(&p.root, &tileset).expect("the ground's textures load");
        let base = ground_base(&t, &mountains, &tex, &full);
        let (rest, ground) = p.world.drawn_terrain();
        let drawn: usize = ground.iter().map(|m| m.positions.len()).sum();
        assert!(drawn > 0, "the ground is textured");
        assert_eq!(drawn, base.tris.len() * 3);
        assert_eq!(rest.positions.len() + drawn, full.positions.len());
        for m in ground {
            assert_eq!(m.positions.len(), m.uvs.len());
            let k = (-1 - m.piece) as usize;
            let tris = base.of.iter().filter(|&&o| o == k).count();
            assert_eq!(m.positions.len(), tris * 3, "texture {k}");
        }
        assert_eq!(p.world.terrain().positions, full.positions);
        p.world.debug_ground = true;
        let (all, none) = p.world.drawn_terrain();
        assert_eq!((all.positions.len(), none.len()), (full.positions.len(), 0));
    }

    /// **The ground's population** (item 4127): per pixel, the tile whose
    /// terrain is the nearest drawn surface where it is a ground pixel —
    /// none of our objects draws there (figures, sprites, trees), outside
    /// every object's window (its pixels plus 6), every trace and every
    /// `INTERFACE` rectangle — else `None`.
    pub(crate) struct Ground {
        pub tile: Vec<Option<(i32, i32)>>,
        pub tw: i32,
        pub th: i32,
    }

    pub(crate) fn ground(p: &mut Probe) -> Ground {
        let size = (p.photo.width, p.photo.height);
        let (w, h) = size;
        p.world.tree_pixels = TreePixels::Scene;
        let scene = frame_scene(&mut p.world, &p.camera, size);
        let mut out = vec![true; w * h];
        let mut boxes: BTreeMap<u32, (i32, i32, i32, i32)> = BTreeMap::new();
        for y in 0..h {
            for x in 0..w {
                let k = scene.keys.at(x, y);
                if k == 0 {
                    continue;
                }
                let (xi, yi) = (x as i32, y as i32);
                let b = boxes.entry(k).or_insert((xi, yi, xi, yi));
                *b = (b.0.min(xi), b.1.min(yi), b.2.max(xi), b.3.max(yi));
            }
        }
        let m = 6;
        for &(x0, y0, x1, y1) in boxes.values() {
            for y in (y0 - m).max(0)..=(y1 + m).min(h as i32 - 1) {
                for x in (x0 - m).max(0)..=(x1 + m).min(w as i32 - 1) {
                    out[y as usize * w + x as usize] = false;
                }
            }
        }
        for y in 0..h {
            for x in 0..w {
                let (xi, yi) = (x as i32, y as i32);
                if INTERFACE
                    .iter()
                    .any(|r| (r.x0..r.x1).contains(&xi) && (r.y0..r.y1).contains(&yi))
                    || TRACED.iter().any(|(_, poly)| inside(poly, xi, yi))
                {
                    out[y * w + x] = false;
                }
            }
        }
        let t = p.world.runner.terrain();
        let (tw, th) = (t.width * TILES_PER_CELL, t.height * TILES_PER_CELL);
        let mesh = p.world.terrain().clone();
        let objects: Vec<Object> = (0..(tw * th) as usize)
            .map(|i| Object {
                key: i as u32 + 1,
                tris: (0..2)
                    .map(|j| {
                        let v = &mesh.positions[i * 6 + j * 3..i * 6 + j * 3 + 3];
                        [v[0], v[1], v[2]]
                    })
                    .collect(),
            })
            .collect();
        let tiles = rasterize_cut(&objects, &[], &[], &p.camera, size).keys;
        let tile = (0..w * h)
            .map(|i| {
                let k = tiles.keys[i];
                (out[i] && k != 0).then(|| {
                    let n = k as i32 - 1;
                    (n % tw, n / tw)
                })
            })
            .collect();
        Ground { tile, tw, th }
    }

    fn mean(px: &[[f32; 3]]) -> [f32; 3] {
        let n = px.len().max(1) as f32;
        [0, 1, 2].map(|c| px.iter().map(|p| p[c]).sum::<f32>() / n)
    }

    fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
        ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
    }

    /// **Step 1's tables** (item 4127): the selection, the floor, the
    /// photograph's colour by tile and the install's files, before any rule.
    #[test]
    fn the_ground_s_values_on_the_photograph() {
        let Some(mut p) = probe() else { return };
        let g = ground(&mut p);
        let t = p.world.runner.terrain();
        let mountains: std::collections::BTreeSet<(i32, i32)> =
            t.mountains.iter().flatten().map(|q| (q.x, q.y)).collect();
        let (w, _) = (p.photo.width, p.photo.height);
        let mut by_tile: BTreeMap<(i32, i32), Vec<[f32; 3]>> = BTreeMap::new();
        for (i, tile) in g.tile.iter().enumerate() {
            if let Some(tt) = tile {
                let at = i * 4;
                by_tile
                    .entry(*tt)
                    .or_default()
                    .push([0, 1, 2].map(|c| f32::from(p.photo.rgba[at + c])));
            }
        }
        let _ = w;
        let cell = |(tx, ty): (i32, i32)| (tx / TILES_PER_CELL, ty / TILES_PER_CELL);
        let land = |tt| {
            let (cx, cy) = cell(tt);
            let i = (cy * t.width + cx) as usize;
            (t.land[i], t.land_sub[i])
        };
        let px_by_kind = g.tile.iter().flatten().fold(BTreeMap::new(), |mut m, &tt| {
            *m.entry((land(tt).0, mountains.contains(&tt)))
                .or_insert(0u32) += 1;
            m
        });
        println!(
            "map {}×{} cells, {}×{} tiles",
            t.width, t.height, g.tw, g.th
        );
        println!(
            "ground pixels {} by (land_key, mountain): {px_by_kind:?}",
            g.tile.iter().flatten().count()
        );
        type Px = Vec<[f32; 3]>;
        let scored: Vec<(&(i32, i32), &Px)> = by_tile
            .iter()
            .filter(|(_, v)| v.len() as u32 >= HUE.min_pixels)
            .collect();
        let mut kinds: BTreeMap<(i8, bool), u32> = BTreeMap::new();
        for (tt, _) in &scored {
            *kinds
                .entry((land(**tt).0, mountains.contains(tt)))
                .or_default() += 1;
        }
        println!(
            "tiles with any ground pixel {}; scored (≥ {}) {}",
            by_tile.len(),
            HUE.min_pixels,
            scored.len()
        );
        println!("scored by (land_key, mountain): {kinds:?}");
        // The floor: the debug colour (its share is the land kind's alone;
        // the slope's shade multiplies every channel alike).
        let mut floor = 0;
        let mut floor_by: BTreeMap<i8, (u32, u32)> = BTreeMap::new();
        for (tt, px) in &scored {
            let (l, _) = land(**tt);
            let ours = if mountains.contains(tt) {
                render::terrain::MOUNTAIN
            } else {
                land_colour(l)
            };
            let d = dist(chromaticity(ours), chromaticity(mean(px)));
            let e = floor_by.entry(l).or_default();
            e.1 += 1;
            if d <= HUE.distance {
                floor += 1;
                e.0 += 1;
            }
        }
        println!(
            "FLOOR (debug colour): {floor} of {} by land: {floor_by:?}",
            scored.len()
        );
        // Step 1: per scored tile.
        let (sx, _) = (g.tw + 1, g.th + 1);
        let z = |cx: i32, cy: i32| f32::from_bits(t.corners[(cy * sx + cx) as usize]);
        println!("tile\tland\tsub\tk\tmtn\tn\tr\tg\tb\tshare_r\tshare_g\tlum\tz\tslope\tedge");
        let mut by_sub: BTreeMap<(i8, u8, bool), Vec<[f32; 2]>> = BTreeMap::new();
        for (tt, px) in &scored {
            let (l, s) = land(**tt);
            let (cx, cy) = cell(**tt);
            let m = mean(px);
            let c = chromaticity(m);
            let (tx, ty) = **tt;
            let zs = [z(tx, ty), z(tx + 1, ty), z(tx + 1, ty + 1), z(tx, ty + 1)];
            let zmax = zs.iter().cloned().fold(f32::MIN, f32::max);
            let zmin = zs.iter().cloned().fold(f32::MAX, f32::min);
            let edge = tx.min(ty).min(g.tw - 1 - tx).min(g.th - 1 - ty);
            let mtn = mountains.contains(tt);
            by_sub.entry((l, s, mtn)).or_default().push(c);
            println!(
                "{tx},{ty}\t{l}\t{s}\t{}\t{}\t{}\t{:.0}\t{:.0}\t{:.0}\t{:.3}\t{:.3}\t{:.0}\t{:.0}\t{:.0}\t{edge}",
                subtile(cx, cy),
                u8::from(mtn),
                px.len(),
                m[0],
                m[1],
                m[2],
                c[0],
                c[1],
                0.3 * m[0] + 0.59 * m[1] + 0.11 * m[2],
                zs.iter().sum::<f32>() / 4.0,
                zmax - zmin,
            );
        }
        for ((l, s, mtn), cs) in &by_sub {
            let mr = cs.iter().map(|c| c[0]).sum::<f32>() / cs.len() as f32;
            let mg = cs.iter().map(|c| c[1]).sum::<f32>() / cs.len() as f32;
            let sd = (cs.iter().map(|c| dist(*c, [mr, mg]).powi(2)).sum::<f32>() / cs.len() as f32)
                .sqrt();
            println!(
                "by (land {l}, sub {s}, mountain {mtn}): {} tiles, share ({mr:.3}, {mg:.3}) sd {sd:.3}",
                cs.len()
            );
        }
        // Step 2: the install's files.
        let tileset = crate::world::lobby_tileset(
            &p.root,
            &crate::world::gamelog_dir().join(crate::world::START_DUMP),
        )
        .unwrap_or_else(|| "early_autumn".to_string());
        let tex = GroundTextures::load(&p.root, &tileset).expect("the ground's textures load");
        println!("tileset {tileset}: {:?}", tex.art);
        for (b, subs) in tex.bases.iter().enumerate() {
            for (k, m) in subs.iter().enumerate() {
                let img = &m.levels[0];
                let n = (img.width * img.height) as f32;
                let sum =
                    [0, 1, 2].map(|c| img.rgba.chunks(4).map(|q| f32::from(q[c])).sum::<f32>() / n);
                let c = chromaticity(sum);
                println!(
                    "base {b} sub {k} {}×{}: mean ({:.0}, {:.0}, {:.0}) share ({:.3}, {:.3})",
                    img.width, img.height, sum[0], sum[1], sum[2], c[0], c[1]
                );
            }
        }
    }

    /// **The count** (item 4127, the rule committed before it in
    /// `docs/RENDERER.md` §3.6): [`RUN1559B_3279_GROUND`] on the base
    /// layer as read, beside the floor, the per-pixel share and the
    /// mapping's check against its half-period control.
    #[test]
    fn the_ground_s_base_layer_against_the_photograph() {
        use render::silhouette::{Paint, Rect, paint};
        let Some(mut p) = probe() else { return };
        let g = ground(&mut p);
        let size = (p.photo.width, p.photo.height);
        let (w, _) = size;
        let tileset = crate::world::lobby_tileset(
            &p.root,
            &crate::world::gamelog_dir().join(crate::world::START_DUMP),
        )
        .expect("the lobby's tileset");
        let tex = GroundTextures::load(&p.root, &tileset).expect("the ground's textures load");
        let mesh = p.world.terrain().clone();
        let t = p.world.runner.terrain();
        let mountains: std::collections::BTreeSet<(i32, i32)> =
            t.mountains.iter().flatten().map(|q| (q.x, q.y)).collect();
        let base = ground_base(&t, &mountains, &tex, &mesh);
        let flat = tex.flat();
        let mut window = Rect::new(i32::MAX, i32::MAX, 0, 0);
        for (i, tt) in g.tile.iter().enumerate() {
            if tt.is_some() {
                let (x, y) = ((i % w) as i32, (i / w) as i32);
                window = Rect::new(
                    window.x0.min(x),
                    window.y0.min(y),
                    window.x1.max(x + 1),
                    window.y1.max(y + 1),
                );
            }
        }
        let object = Object {
            key: 1,
            tris: base.tris.clone(),
        };
        let shifted: Vec<[[f32; 2]; 3]> = base
            .uvs
            .iter()
            .map(|t| t.map(|[u, v]| [u + 0.5, v + 0.5]))
            .collect();
        let draw = |uvs: &[[[f32; 2]; 3]]| {
            paint(
                &object,
                &Paint::Textures {
                    uvs,
                    mips: &flat,
                    of: &base.of,
                },
                &p.camera,
                size,
                window,
            )
        };
        let ours = draw(&base.uvs);
        let control = draw(&shifted);
        let ww = (window.x1 - window.x0) as usize;
        let cell = |(tx, ty): (i32, i32)| (tx / TILES_PER_CELL, ty / TILES_PER_CELL);
        let sub = |(cx, cy): (i32, i32)| t.land_sub[(cy * t.width + cx) as usize];
        let photo_at = |i: usize| [0, 1, 2].map(|c| f32::from(p.photo.rgba[i * 4 + c]));
        type Px = Vec<[f32; 3]>;
        let mut tiles: BTreeMap<(i32, i32), (Px, Px)> = BTreeMap::new();
        let mut pixel: BTreeMap<u8, (u32, u32)> = BTreeMap::new();
        let mut lum: Vec<(f32, f32, f32)> = Vec::new();
        let l = |c: [f32; 3]| 0.3 * c[0] + 0.59 * c[1] + 0.11 * c[2];
        for (i, tt) in g.tile.iter().enumerate() {
            let Some(tt) = *tt else { continue };
            let (x, y) = ((i % w) as i32, (i / w) as i32);
            let j = (y - window.y0) as usize * ww + (x - window.x0) as usize;
            let them = photo_at(i);
            let e = tiles.entry(tt).or_default();
            e.0.push(them);
            let drawn = if mountains.contains(&tt) {
                Some(render::terrain::MOUNTAIN.map(|v| v * 255.0))
            } else {
                ours[j]
                    .filter(|q| base.tile[q.tri as usize] == tt)
                    .map(|q| q.rgb)
            };
            let Some(c) = drawn else { continue };
            e.1.push(c);
            if !mountains.contains(&tt) {
                if let Some(k) = control[j].filter(|q| base.tile[q.tri as usize] == tt) {
                    lum.push((l(them), l(c), l(k.rgb)));
                }
                let s = sub(cell(tt));
                let sum = |q: [f32; 3]| q[0] + q[1] + q[2];
                if sum(c) >= HUE.dark && sum(them) >= HUE.dark {
                    let pe = pixel.entry(s).or_default();
                    pe.1 += 1;
                    if dist(chromaticity(c), chromaticity(them)) <= HUE.distance {
                        pe.0 += 1;
                    }
                }
            }
        }
        let scored: Vec<_> = tiles
            .iter()
            .filter(|(_, (th, _))| th.len() as u32 >= HUE.min_pixels)
            .collect();
        let mut row = 0;
        let mut by: BTreeMap<(u8, bool), (u32, u32)> = BTreeMap::new();
        let mut parts: BTreeMap<&str, u32> = BTreeMap::new();
        for (tt, (th, us)) in &scored {
            let m = mountains.contains(tt);
            let c = cell(**tt);
            let e = by.entry((sub(c), m)).or_default();
            e.1 += 1;
            let ok = !us.is_empty()
                && dist(chromaticity(mean(us)), chromaticity(mean(th))) <= HUE.distance;
            if ok {
                row += 1;
                e.0 += 1;
                continue;
            }
            let (tx, ty) = **tt;
            let blend = (-1..=1).any(|dy| {
                (-1..=1).any(|dx| {
                    let (nx, ny) = (c.0 + dx, c.1 + dy);
                    (0..t.width).contains(&nx)
                        && (0..t.height).contains(&ny)
                        && sub((nx, ny)) != sub(c)
                })
            });
            let why = if m {
                "a mountain's own draw"
            } else if tx.min(ty).min(g.tw - 1 - tx).min(g.th - 1 - ty) < 4 {
                "the edge's darkening"
            } else if blend {
                "a blend between bases"
            } else {
                "inside one base"
            };
            *parts.entry(why).or_default() += 1;
            println!(
                "parts {tx},{ty} sub {} k {} mountain {m}: ours {:?} theirs {:?} ({why})",
                sub(c),
                subtile(c.0, c.1),
                chromaticity(mean(us)),
                chromaticity(mean(th)),
            );
        }
        type Pair = dyn Fn(&(f32, f32, f32)) -> (f32, f32);
        let pearson = |f: &Pair| {
            let n = lum.len() as f32;
            let (sa, sb) = lum
                .iter()
                .map(f)
                .fold((0.0, 0.0), |a, (x, y)| (a.0 + x, a.1 + y));
            let (ma, mb) = (sa / n, sb / n);
            let (mut sab, mut saa, mut sbb) = (0.0, 0.0, 0.0);
            for q in &lum {
                let (x, y) = f(q);
                sab += (x - ma) * (y - mb);
                saa += (x - ma).powi(2);
                sbb += (y - mb).powi(2);
            }
            sab / (saa * sbb).sqrt()
        };
        let r = pearson(&|q| (q.0, q.1));
        let rc = pearson(&|q| (q.0, q.2));
        println!(
            "RUN1559B_3279_GROUND: {row} of {} (floor {}), by (land_sub, mountain): {by:?}",
            scored.len(),
            RUN1559B_3279_GROUND_FLOOR.0
        );
        println!("parting by what it would take: {parts:?}");
        println!("per pixel, by land_sub (agree, lit): {pixel:?}");
        println!(
            "mapping's check over {} pixels: r {r:.3} against the half-period control's {rc:.3}",
            lum.len()
        );
        assert_eq!((row, scored.len()), RUN1559B_3279_GROUND);
    }

    /// The land pass's own bucket per cell on run1559b's packet,
    /// `tools/render/land_buckets.py`'s `cell` rows: `(cx, cy)` →
    /// `(land, WData land_sub, the pCoords list it sits in, its patch's
    /// landindex)` (item 4129).
    #[allow(clippy::type_complexity)]
    fn packet_buckets() -> Option<BTreeMap<(i32, i32), (i8, u8, i32, Option<i32>)>> {
        let path = std::env::var_os("RON_LAND_BUCKETS")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|h| {
                    PathBuf::from(h)
                        .join("ron-data/lab-experiments/2026-11-11-item-4129/run1559b-buckets.txt")
                })
            })?;
        let Ok(text) = std::fs::read_to_string(&path) else {
            println!("skipping: no packet buckets at {}", path.display());
            return None;
        };
        let mut out = BTreeMap::new();
        for line in text.lines() {
            let f: Vec<&str> = line.split_whitespace().collect();
            if f.first() != Some(&"cell") || f.len() < 17 {
                continue;
            }
            let n = |i: usize| f[i].parse::<i32>().expect("a number");
            let patch = f[14]
                .split_once('/')
                .map(|(_, li)| li.parse::<i32>().expect("a landindex"));
            out.insert((n(1), n(2)), (n(4) as i8, n(6) as u8, n(16), patch));
        }
        Some(out)
    }

    /// **Which base a tile draws, told apart** (item 4129; `docs/RENDERER.md`
    /// §3.6): the row under controls, a per-tile measure that can tell the
    /// bases apart, and the packet's own bucket against ours.
    #[test]
    fn which_base_a_tile_draws() {
        use render::silhouette::{Paint, Rect, paint};
        let Some(mut p) = probe() else { return };
        let Some(packet) = packet_buckets() else {
            return;
        };
        let g = ground(&mut p);
        let size = (p.photo.width, p.photo.height);
        let (w, _) = size;
        let tileset = crate::world::lobby_tileset(
            &p.root,
            &crate::world::gamelog_dir().join(crate::world::START_DUMP),
        )
        .expect("the lobby's tileset");
        let tex = GroundTextures::load(&p.root, &tileset).expect("the ground's textures load");
        assert!(tex.bases.iter().all(|b| b.len() == 9), "four supertiles");
        let nb = tex.bases.len();
        let mesh = p.world.terrain().clone();
        let t = p.world.runner.terrain();
        let mountains: std::collections::BTreeSet<(i32, i32)> =
            t.mountains.iter().flatten().map(|q| (q.x, q.y)).collect();
        let base = ground_base(&t, &mountains, &tex, &mesh);
        let flat = tex.flat();
        let cell = |(tx, ty): (i32, i32)| (tx / TILES_PER_CELL, ty / TILES_PER_CELL);
        let sub = |(cx, cy): (i32, i32)| t.land_sub[(cy * t.width + cx) as usize];

        // Our land_sub and our bucket against the packet's, every cell.
        let mut cells: BTreeMap<(bool, bool, bool), u32> = BTreeMap::new();
        let mut parts: BTreeMap<(u8, u8, i32), u32> = BTreeMap::new();
        for cy in 0..t.height {
            for cx in 0..t.width {
                let c = (cy * t.width + cx) as usize;
                let Some(&(land, psub, bucket, _)) = packet.get(&(cx, cy)) else {
                    continue;
                };
                if land != 0 || t.land[c] != 0 {
                    continue;
                }
                let ours = 9 * i32::from(t.land_sub[c]) + subtile(cx, cy) as i32;
                *cells
                    .entry((
                        t.land_sub[c] == psub,
                        ours == bucket,
                        9 * psub as i32 + subtile(cx, cy) as i32 == bucket,
                    ))
                    .or_default() += 1;
                if ours != bucket {
                    *parts.entry((t.land_sub[c], psub, bucket / 9)).or_default() += 1;
                }
            }
        }
        println!(
            "cells on land 0 by (our sub = packet's WData sub, our bucket = the packet's list, the packet's list = the rule on its own sub): {cells:?}"
        );
        println!("parting cells by (our sub, packet's WData sub, packet list's base): {parts:?}");
        let agree = cells
            .iter()
            .filter(|(k, _)| k.1)
            .map(|(_, n)| n)
            .sum::<u32>();
        let on_own = cells
            .iter()
            .filter(|(k, _)| k.2)
            .map(|(_, n)| n)
            .sum::<u32>();
        let all = cells.values().sum::<u32>();

        let mut window = Rect::new(i32::MAX, i32::MAX, 0, 0);
        for (i, tt) in g.tile.iter().enumerate() {
            if tt.is_some() {
                let (x, y) = ((i % w) as i32, (i / w) as i32);
                window = Rect::new(
                    window.x0.min(x),
                    window.y0.min(y),
                    window.x1.max(x + 1),
                    window.y1.max(y + 1),
                );
            }
        }
        let object = Object {
            key: 1,
            tris: base.tris.clone(),
        };
        let draw = |of: &[usize]| {
            paint(
                &object,
                &Paint::Textures {
                    uvs: &base.uvs,
                    mips: &flat,
                    of,
                },
                &p.camera,
                size,
                window,
            )
        };
        // `of` under a choice of base and sub-tile per cell.
        let choose = |f: &dyn Fn((i32, i32)) -> (usize, usize)| -> Vec<usize> {
            base.tile
                .iter()
                .map(|&tt| {
                    let (b, k) = f(cell(tt));
                    9 * b + k
                })
                .collect()
        };
        let rule = |c: (i32, i32)| (usize::from(sub(c)), subtile(c.0, c.1));
        let pk = |c: (i32, i32)| {
            let b = packet[&c].2;
            ((b / 9) as usize, (b % 9) as usize)
        };
        assert_eq!(choose(&rule), base.of, "the rule is the draw");
        let ww = (window.x1 - window.x0) as usize;
        let photo_at = |i: usize| [0, 1, 2].map(|c| f32::from(p.photo.rgba[i * 4 + c]));
        let l = |c: [f32; 3]| 0.3 * c[0] + 0.59 * c[1] + 0.11 * c[2];
        // Per scored tile, its photograph pixels and, per paint, the pairs.
        let mut photo: BTreeMap<(i32, i32), Vec<[f32; 3]>> = BTreeMap::new();
        for (i, tt) in g.tile.iter().enumerate() {
            if let Some(tt) = *tt {
                photo.entry(tt).or_default().push(photo_at(i));
            }
        }
        let scored: Vec<(i32, i32)> = photo
            .iter()
            .filter(|(_, v)| v.len() as u32 >= HUE.min_pixels)
            .map(|(&k, _)| k)
            .collect();
        let plain: Vec<(i32, i32)> = scored
            .iter()
            .copied()
            .filter(|tt| !mountains.contains(tt))
            .collect();
        type Pairs = BTreeMap<(i32, i32), Vec<([f32; 3], [f32; 3])>>;
        let pairs = |of: &[usize]| -> Pairs {
            let ours = draw(of);
            let mut out: Pairs = BTreeMap::new();
            for (i, tt) in g.tile.iter().enumerate() {
                let Some(tt) = *tt else { continue };
                let (x, y) = ((i % w) as i32, (i / w) as i32);
                let j = (y - window.y0) as usize * ww + (x - window.x0) as usize;
                if let Some(q) = ours[j].filter(|q| base.tile[q.tri as usize] == tt) {
                    out.entry(tt).or_default().push((photo_at(i), q.rgb));
                }
            }
            out
        };
        // The row as 4127 pinned it, under a paint: mountains keep the grey.
        let row = |pr: &Pairs| {
            let mut n = 0;
            let mut by: BTreeMap<(u8, bool), (u32, u32)> = BTreeMap::new();
            for tt in &scored {
                let m = mountains.contains(tt);
                let e = by.entry((sub(cell(*tt)), m)).or_default();
                e.1 += 1;
                let us: Vec<[f32; 3]> = if m {
                    vec![render::terrain::MOUNTAIN.map(|v| v * 255.0)]
                } else {
                    pr.get(tt)
                        .map(|v| v.iter().map(|q| q.1).collect())
                        .unwrap_or_default()
                };
                if !us.is_empty()
                    && dist(chromaticity(mean(&us)), chromaticity(mean(&photo[tt]))) <= HUE.distance
                {
                    n += 1;
                    e.0 += 1;
                }
            }
            (n, by)
        };
        let pearson = |v: &[(f32, f32)]| {
            let n = v.len() as f32;
            let (ma, mb) = v
                .iter()
                .fold((0.0, 0.0), |a, q| (a.0 + q.0 / n, a.1 + q.1 / n));
            let (mut sab, mut saa, mut sbb) = (0.0, 0.0, 0.0);
            for &(x, y) in v {
                sab += (x - ma) * (y - mb);
                saa += (x - ma).powi(2);
                sbb += (y - mb).powi(2);
            }
            if saa * sbb > 0.0 {
                sab / (saa * sbb).sqrt()
            } else {
                0.0
            }
        };
        let r_all = |pr: &Pairs| {
            let v: Vec<(f32, f32)> = pr.values().flatten().map(|q| (l(q.0), l(q.1))).collect();
            pearson(&v)
        };
        let r_tile = |pr: &Pairs, tt: &(i32, i32)| {
            pr.get(tt)
                .map(|v| pearson(&v.iter().map(|q| (l(q.0), l(q.1))).collect::<Vec<_>>()))
                .unwrap_or(f32::NAN)
        };

        // Step 1: the controls.
        let ruled = pairs(&base.of);
        let (rn, rby) = row(&ruled);
        println!("control rule: row {rn} by {rby:?} r {:.3}", r_all(&ruled));
        let mut one: Vec<Pairs> = Vec::new();
        let mut wrong: Vec<Pairs> = Vec::new();
        for b in 0..nb {
            let pr = pairs(&choose(&|c| (b, subtile(c.0, c.1))));
            let (n, by) = row(&pr);
            println!(
                "control one base {b}: row {n} by {by:?} r {:.3}",
                r_all(&pr)
            );
            one.push(pr);
            wrong.push(pairs(&choose(&|c| (b, (subtile(c.0, c.1) + 1) % 9))));
        }
        let mut perms: Vec<[usize; 4]> = Vec::new();
        for a in 0..4 {
            for b in 0..4 {
                for c in 0..4 {
                    for d in 0..4 {
                        let q = [a, b, c, d];
                        if (0..4).all(|i| q.contains(&i)) && q != [0, 1, 2, 3] {
                            perms.push(q);
                        }
                    }
                }
            }
        }
        let mut best_perm = (0, [0; 4]);
        for q in &perms {
            let pr = pairs(&choose(&|c| (q[usize::from(sub(c))], subtile(c.0, c.1))));
            let (n, by) = row(&pr);
            println!("control land_sub -> base {q:?}: row {n} by {by:?}");
            if n > best_perm.0 {
                best_perm = (n, *q);
            }
        }
        println!("best permutation {:?} row {}", best_perm.1, best_perm.0);
        let zero = pairs(&choose(&|c| (usize::from(sub(c)), 0)));
        let (n, by) = row(&zero);
        println!(
            "control cell index 0: row {n} by {by:?} r {:.3}",
            r_all(&zero)
        );
        let packeted = pairs(&choose(&pk));
        let (n, by) = row(&packeted);
        println!(
            "the packet's bucket: row {n} by {by:?} r {:.3}",
            r_all(&packeted)
        );

        // Step 2: per plain tile, r against each base and its wrong cell.
        let mean_share: Vec<[f32; 2]> = tex
            .bases
            .iter()
            .map(|b| {
                let px: Vec<[f32; 3]> = b
                    .iter()
                    .flat_map(|m| {
                        m.levels[0]
                            .rgba
                            .chunks(4)
                            .map(|q| [f32::from(q[0]), f32::from(q[1]), f32::from(q[2])])
                    })
                    .collect();
                chromaticity(mean(&px))
            })
            .collect();
        let mut confusion: BTreeMap<(u8, usize, usize), u32> = BTreeMap::new();
        let mut packet_conf: BTreeMap<(usize, usize), u32> = BTreeMap::new();
        let mut nearest: BTreeMap<(u8, usize), u32> = BTreeMap::new();
        let (mut above, mut decisive, mut none_above) = (0, 0, 0);
        let mut by_edge: BTreeMap<(bool, i32), u32> = BTreeMap::new();
        let mut gb_rule = 0;
        let mut gb_packet = 0;
        let mut gb_one = vec![0; nb];
        let mut patched: BTreeMap<(bool, Option<i32>, bool), u32> = BTreeMap::new();
        for tt in &plain {
            let c = cell(*tt);
            let r: Vec<f32> = (0..nb).map(|b| r_tile(&one[b], tt)).collect();
            let rw: Vec<f32> = (0..nb).map(|b| r_tile(&wrong[b], tt)).collect();
            let best = (0..nb)
                .max_by(|&a, &b| r[a].total_cmp(&r[b]))
                .expect("a base");
            let second = (0..nb)
                .filter(|&b| b != best)
                .map(|b| r[b])
                .fold(f32::MIN, f32::max);
            let spread = rw.iter().copied().fold(f32::MIN, f32::max)
                - rw.iter().copied().fold(f32::MAX, f32::min);
            let stands = |b: usize| r[b] > rw[b];
            if stands(best) {
                above += 1;
            }
            if r[best] - second > spread {
                decisive += 1;
            }
            if !(0..nb).any(stands) {
                none_above += 1;
            }
            let rb = usize::from(sub(c));
            let pb = pk(c).0;
            *confusion.entry((sub(c), rb, best)).or_default() += 1;
            *packet_conf.entry((pb, best)).or_default() += 1;
            let share = chromaticity(mean(&photo[tt]));
            let near = (0..nb)
                .min_by(|&a, &b| dist(share, mean_share[a]).total_cmp(&dist(share, mean_share[b])))
                .expect("a base");
            *nearest.entry((sub(c), near)).or_default() += 1;
            // Cells to the nearest cell of another land_sub, by Chebyshev distance.
            let mut d = 99;
            for dy in -4..=4i32 {
                for dx in -4..=4i32 {
                    let (nx, ny) = (c.0 + dx, c.1 + dy);
                    if (0..t.width).contains(&nx)
                        && (0..t.height).contains(&ny)
                        && sub((nx, ny)) != sub(c)
                    {
                        d = d.min(dx.abs().max(dy.abs()));
                    }
                }
            }
            *by_edge.entry((best == rb, d.min(5))).or_default() += 1;
            let lower = packet[&c].3.map(|li| li / 9 - rb as i32);
            *patched
                .entry((best == rb, lower, best + 1 == rb))
                .or_default() += 1;
            let wins = |b: usize| b == best && stands(b);
            if wins(rb) {
                gb_rule += 1;
            }
            if wins(pb) {
                gb_packet += 1;
            }
            for (b, n) in gb_one.iter_mut().enumerate() {
                if wins(b) {
                    *n += 1;
                }
            }
        }
        println!("mean shares by base: {mean_share:?}");
        println!(
            "plain tiles {}: best base above its own wrong-cell control {above}; best ahead of the second by more than the controls' spread {decisive}; no base above its control {none_above}",
            plain.len()
        );
        println!("confusion (land_sub, rule's base, best base): {confusion:?}");
        println!("confusion (packet's base, best base): {packet_conf:?}");
        println!("nearest base by mean share (land_sub, base): {nearest:?}");
        println!("(best = rule's, cells to another land_sub) : {by_edge:?}");
        println!(
            "(best = rule's, the patch's base less the rule's, best = the next base down): {patched:?}"
        );
        println!(
            "RUN1559B_3279_GROUND_BASE under the rule {gb_rule}, under the packet's list {gb_packet}, one base {gb_one:?}, of {}",
            plain.len()
        );
        assert_eq!((agree, on_own, all), RUN1559B_3279_PACKET_LIST);
        assert!(
            2 * above >= plain.len(),
            "the measure tells the bases apart"
        );
        assert_eq!((gb_rule, plain.len()), RUN1559B_3279_GROUND_BASE);
        assert_eq!(gb_one, RUN1559B_3279_GROUND_BASE_ONE);
        assert!(gb_one.iter().all(|&n| n < gb_rule));
        assert_eq!(rn, RUN1559B_3279_GROUND.0, "the row as 4127 pinned it");
    }
}
