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
    /// The `<COASTAL>` and `<OCEAN>` bases (item 4164), each as `bases`
    /// holds its own, in [`GroundTextures::flat`] after `bases`.
    pub coastal: Vec<Vec<Mips>>,
    pub ocean: Vec<Vec<Mips>>,
    /// The tileset's `COAST_BASE` (item 4200): the `BASELAND` sub a
    /// `SANDY` cell's transition patch draws ([`GroundTextures::patch_index`]).
    pub coast_base: u8,
    /// The tileset's `<CORALTEX>` (item 4200), laid over every `OCEAN`
    /// cell's patch at the patch's own alpha.
    pub coral: Option<Mips>,
}

impl GroundTextures {
    /// The tileset's textures as the client draws them: tinted at load
    /// (item 4212, §3.21).
    pub fn load(root: &Path, tileset: &str) -> Result<GroundTextures, String> {
        Self::load_tinted(root, tileset, crate::tint::of(root, tileset))
    }

    /// The files' own texels, untinted: the painter's (§3.6, the colour
    /// reader's texel alone), whose rows item 4212 leaves as they stood.
    #[cfg(test)]
    pub fn untinted(root: &Path, tileset: &str) -> Result<GroundTextures, String> {
        Self::load_tinted(root, tileset, [0; 3])
    }

    fn load_tinted(root: &Path, tileset: &str, tint: [u8; 3]) -> Result<GroundTextures, String> {
        let install = rondata::Install::new(root);
        let art = rondata::artdata::ground_art(&install, tileset).map_err(|e| format!("{e:?}"))?;
        let load = |list: &[rondata::artdata::GroundBase]| -> Result<Vec<Vec<Mips>>, String> {
            let mut bases = Vec::new();
            for b in list {
                let path = rondata::artdata::resolve_art(root, &b.texture)
                    .ok_or_else(|| format!("no file for {}", b.texture))?;
                let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
                let mut image =
                    render::tga::decode(&bytes).map_err(|e| format!("{}: {e:?}", b.texture))?;
                image.tint(tint);
                bases.push(if b.supertile {
                    (0..9).map(|k| Mips::new(cut(&image, k))).collect()
                } else {
                    vec![Mips::new(image)]
                });
            }
            Ok(bases)
        };
        let coast_base = rondata::mapgen::tileset_groups(root, tileset)
            .and_then(|g| u8::try_from(g.coast_base).ok())
            .ok_or_else(|| format!("{tileset}: no COAST_BASE"))?;
        let coral = match rondata::artdata::coral_texture(&install, tileset)
            .map_err(|e| format!("{e:?}"))?
        {
            Some(rel) => {
                let path = rondata::artdata::resolve_art(root, &rel)
                    .ok_or_else(|| format!("no file {rel}"))?;
                let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
                let mut image = render::tga::decode(&bytes).map_err(|e| format!("{rel}: {e:?}"))?;
                image.tint(tint);
                Some(Mips::new(image))
            }
            None => None,
        };
        Ok(GroundTextures {
            bases: load(&art.baseland)?,
            coastal: load(&art.coastal)?,
            ocean: load(&art.ocean)?,
            art,
            coast_base,
            coral,
        })
    }

    /// **The texture a cell's transition patch draws** (item 4200;
    /// `docs/RENDERER.md` §3.18), as an index into [`GroundTextures::flat`]:
    /// `create_transition_renderables@00862c60`'s choice. Past sub 0 the
    /// cell's own land at `land_sub − 1`; at sub 0 a `SANDY` cell
    /// (`eTILE_COASTAL`) takes `BASELAND`'s `COAST_BASE` itself, with no
    /// step down, and an `OCEAN` cell the coastal land's last texture.
    /// `BASELAND` sub-tiles are chosen as the land pass chooses them,
    /// `x mod 3 + 3 (y mod 3)`. `None` on a `BASELAND` cell at sub 0, which
    /// is never queued, and past the tileset's textures.
    pub fn patch_index(&self, land: i8, land_sub: u8, cx: i32, cy: i32) -> Option<usize> {
        use render::terrain::{BASELAND, OCEAN, SANDY};
        let count = |b: &[Vec<Mips>]| b.iter().map(Vec::len).sum::<usize>();
        let (land, sub) = match (land, land_sub.checked_sub(1)) {
            (_, Some(sub)) => (land, sub),
            (SANDY, None) => (BASELAND, self.coast_base),
            (OCEAN, None) => (
                SANDY,
                u8::try_from(self.coastal.len().checked_sub(1)?).ok()?,
            ),
            _ => return None,
        };
        match land {
            BASELAND => self.index(sub, cx, cy),
            SANDY => {
                let b = usize::from(sub);
                self.coastal.get(b)?;
                Some(count(&self.bases) + count(&self.coastal[..b]))
            }
            OCEAN => {
                let b = usize::from(sub);
                self.ocean.get(b)?;
                Some(count(&self.bases) + count(&self.coastal) + count(&self.ocean[..b]))
            }
            _ => None,
        }
    }

    /// **The texture any cell's land pass draws** (item 4164), as an index
    /// into [`GroundTextures::flat`]: `convert_land_tex_to_linear`'s bucket.
    /// On `BASELAND` it is [`GroundTextures::index`]; on `SANDY`
    /// (`eTILE_COASTAL`) and `OCEAN` (`eTILE_OCEAN`) the count of every
    /// texture of the types before, and no more — the type's first texture,
    /// neither `land_sub` nor the cell's coordinates entering. `None` past
    /// the tileset's textures or on any other kind.
    pub fn index_land(&self, land: i8, land_sub: u8, cx: i32, cy: i32) -> Option<usize> {
        let count = |b: &[Vec<Mips>]| b.iter().map(Vec::len).sum::<usize>();
        match land {
            render::terrain::BASELAND => self.index(land_sub, cx, cy),
            render::terrain::SANDY => (!self.coastal.is_empty()).then(|| count(&self.bases)),
            render::terrain::OCEAN => {
                (!self.ocean.is_empty()).then(|| count(&self.bases) + count(&self.coastal))
            }
            _ => None,
        }
    }

    /// The texture a `BASELAND` cell draws: `(base, sub-tile)` as an index
    /// into [`GroundTextures::flat`], `None` past the tileset's bases.
    pub fn index(&self, land_sub: u8, cx: i32, cy: i32) -> Option<usize> {
        let b = usize::from(land_sub);
        let n = self.bases.get(b)?.len();
        let k = if n == 9 { subtile(cx, cy) } else { 0 };
        Some(self.bases[..b].iter().map(Vec::len).sum::<usize>() + k)
    }

    /// Every sub-tile, base by base: `BASELAND`'s, then `<COASTAL>`'s and
    /// `<OCEAN>`'s (item 4164).
    pub fn flat(&self) -> Vec<Mips> {
        self.bases
            .iter()
            .chain(&self.coastal)
            .chain(&self.ocean)
            .flatten()
            .cloned()
            .collect()
    }
}

/// **The base layer's triangles** (item 4127): every `BASELAND` tile not
/// under a placed mountain, its two triangles as [`TerrainMesh`] holds
/// them, each with its corners' texcoords, its texture's index into
/// [`GroundTextures::flat`] and its tile. From item 4164 the `SANDY` and
/// `OCEAN` tiles too, each its type's first texture
/// ([`GroundTextures::index_land`]) laid as `generate_land` lays every
/// cell's, one repeat a cell.
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
    use render::terrain::TILES_PER_CELL;
    let mut out = GroundBase::default();
    let (tw, th) = (t.width * TILES_PER_CELL, t.height * TILES_PER_CELL);
    if mesh.positions.len() != (tw * th * 6) as usize {
        return out;
    }
    for ty in 0..th {
        for tx in 0..tw {
            let (cx, cy) = (tx / TILES_PER_CELL, ty / TILES_PER_CELL);
            let c = (cy * t.width + cx) as usize;
            let land = t.land.get(c).copied().unwrap_or(render::terrain::NONE);
            if mountains.contains(&(tx, ty)) {
                continue;
            }
            let Some(k) = tex.index_land(land, t.land_sub[c], cx, cy) else {
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

/// **The water's alpha at its deepest** (item 4164): `graphics.txt`'s
/// `WaterColor Solid`, the top byte `TerrainOut::create_water_verts@
/// 0086ceb0` gives a water vertex's colour (`watercolor_solid`, white).
pub const WATER_SOLID: u8 = 245;

/// **The depth below which the water is solid** (item 4164):
/// `__real_c25c0000`, −55, in `TerrainOut::get_water_fade@00869fd0`.
pub const WATER_DEEP: f32 = -55.0;

/// **The fade's top** (item 4164): `TerrainOut +0x4d80`, the registered
/// variable (range −91 to 0, `TerrainOut::registration@00865930`) that
/// `get_water_fade` compares against. Its value is not in the decompile;
/// run1564's packet holds **−25.0** at `0x00c0fe70` (`terrain`,
/// `0x00c0b0f0`, which `MiscAccess::terrain` points at, plus `0x4d80`).
pub const WATER_FADE_TOP: f32 = -25.0;

/// **A water vertex's alpha** (item 4164), `get_water_fade` on the corner
/// height `h`: [`WATER_SOLID`] at or below [`WATER_DEEP`], none above
/// [`WATER_FADE_TOP`], and between them `(h − top) / −55 · 245` truncated.
/// It is not continuous at −55: at −54.9 it is 132. The `>> 2` on a cell
/// `+0x138`'s `0x800` is not drawn.
#[cfg(test)]
pub fn water_fade(h: f32) -> u8 {
    water_fade_of(h, WATER_SOLID)
}

/// [`water_fade`] with `solid` the alpha at depth, `get_water_fade`'s
/// third argument (item 4178).
pub fn water_fade_of(h: f32, solid: u8) -> u8 {
    const TOP: f32 = WATER_FADE_TOP;
    if h <= WATER_DEEP {
        solid
    } else if h <= TOP {
        ((h - TOP) / WATER_DEEP * f32::from(solid)) as u8
    } else {
        0
    }
}

/// **Whether a cell holds water** (item 4164),
/// `TerrainOut::coord_contains_water@0086a080`: a cell on the map whose
/// `WData` flags carry `COAST` or `HALFLAND`, or whose land is `OCEAN`, or
/// — its `CoordInfo` flags not held here — a cell with any of its 5 × 5
/// corners below zero, the `& 4` arm taken as every shore cell's. Item
/// 4178: on every land, not only `SANDY`: run1564's drawn water lays 64
/// corners on `BASELAND` cells (flag `0x400`) the `SANDY`-only arm left
/// dry, and with the arm on every land none ([`RUN1564_WATER_LATTICE`]).
pub fn holds_water(t: &sim::observe::TerrainScene, cx: i32, cy: i32) -> bool {
    use render::terrain::{OCEAN, TILES_PER_CELL};
    use sim::ids::cell;
    if !(0..t.width).contains(&cx) || !(0..t.height).contains(&cy) {
        return false;
    }
    let i = (cy * t.width + cx) as usize;
    if t.flags[i] & (cell::COAST | cell::HALFLAND) != 0 || t.land[i] == OCEAN {
        return true;
    }
    let sx = t.width * TILES_PER_CELL + 1;
    (0..=TILES_PER_CELL).any(|dy| {
        (0..=TILES_PER_CELL).any(|dx| {
            let k = ((cy * TILES_PER_CELL + dy) * sx + cx * TILES_PER_CELL + dx) as usize;
            t.corners.get(k).is_some_and(|&z| f32::from_bits(z) < 0.0)
        })
    })
}

/// **The water's alpha over a point of a cell** (item 4164): `None` on a
/// cell that holds no water ([`holds_water`]); else the alpha the water
/// mesh carries there, out of 255. `create_water_verts` lays a vertex
/// every half cell, two tiles, each with [`water_fade`] of the corner
/// height there; the point's is the bilinear blend of its half cell's four.
/// `uv` is the cell-local `[0, 1]` the base layer's texcoord is
/// ([`tile_uvs`]). Item 4178: the client draws [`water_mesh`]; the painter
/// keeps this (parked 4178a).
#[cfg(test)]
pub fn water_alpha(
    t: &sim::observe::TerrainScene,
    (cx, cy): (i32, i32),
    uv: [f32; 2],
) -> Option<f32> {
    use render::terrain::TILES_PER_CELL;
    if !holds_water(t, cx, cy) {
        return None;
    }
    let sx = t.width * TILES_PER_CELL + 1;
    let at = |x: i32, y: i32| {
        let k = (y * sx + x) as usize;
        f32::from(water_fade(f32::from_bits(t.corners[k])))
    };
    let half = TILES_PER_CELL / 2;
    let (fx, fy) = (uv[0].clamp(0.0, 1.0) * 2.0, uv[1].clamp(0.0, 1.0) * 2.0);
    let (ix, iy) = ((fx as i32).min(1), (fy as i32).min(1));
    let (u, v) = (fx - ix as f32, fy - iy as f32);
    let (x0, y0) = (
        cx * TILES_PER_CELL + ix * half,
        cy * TILES_PER_CELL + iy * half,
    );
    let (a, b, c, d) = (
        at(x0, y0),
        at(x0 + half, y0),
        at(x0 + half, y0 + half),
        at(x0, y0 + half),
    );
    Some((a * (1.0 - u) + b * u) * (1.0 - v) + (d * (1.0 - u) + c * u) * v)
}

/// **The water's alpha as a texture** (item 4164): 256 texels, texel `i`
/// of alpha `i`, so an alpha `a` out of 255 is the texcoord
/// `(a + 0.5) / 256` and a linear blend of two is a linear blend of their
/// texcoords. The client's patch shader reads it as it reads an alpha.
pub fn water_ramp() -> Mips {
    let rgba = (0..=255u8).flat_map(|a| [255, 255, 255, a]).collect();
    Mips::new(render::tga::Image {
        width: 256,
        height: 1,
        kind: 2,
        bits: 32,
        footer: false,
        rgba,
    })
}

/// **The water's own lattice** (item 4178; `docs/RENDERER.md` §3.12):
/// `TerrainOut::generate_water@00861980`'s mesh. One vertex every 384
/// units, half a cell, on each of a wet cell's 3 × 3 lattice points
/// (`create_water_verts@0086ceb0` lays them for every cell
/// [`holds_water`] passes, a point shared between cells laid once). Each
/// takes [`water_fade`] of a corner height, then one pass of the 3 × 3
/// smoothing, row by row and in place.
#[derive(Clone, Debug, PartialEq)]
pub struct WaterGrid {
    /// Lattice points across and down: `2 · width + 1`, `2 · height + 1`.
    pub nx: i32,
    pub ny: i32,
    /// Each point's alpha out of 255, `None` where no wet cell lays one.
    pub alpha: Vec<Option<u8>>,
}

/// How [`water_grid_with`] reads `generate_water` (item 4178): the
/// readings the packet's diff chose between, kept so the made-to-fail can
/// name each.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WaterRead {
    /// The alpha at depth, `watercolor_solid`'s top byte.
    pub solid: u8,
    /// An odd lattice row takes its fade from the even row's corner
    /// height: `create_water_verts`' odd branch reads `get_water_fade` at
    /// `4 · (row >> 1)` tiles, the row 384 units north of its vertex.
    pub north: bool,
    /// The smoothing writes the colour array it reads, so a point's left
    /// and upper neighbours enter already smoothed.
    pub in_place: bool,
}

/// The reading the packet backs (item 4178).
pub const WATER_READ: WaterRead = WaterRead {
    solid: WATER_SOLID,
    north: true,
    in_place: true,
};

impl WaterGrid {
    /// The point `(gx, gy)`'s alpha, `None` off the lattice or where none
    /// is laid.
    pub fn at(&self, gx: i32, gy: i32) -> Option<u8> {
        if !(0..self.nx).contains(&gx) || !(0..self.ny).contains(&gy) {
            return None;
        }
        self.alpha[(gy * self.nx + gx) as usize]
    }
}

/// [`water_grid_with`] under [`WATER_READ`].
pub fn water_grid(t: &sim::observe::TerrainScene) -> WaterGrid {
    water_grid_with(t, WATER_READ)
}

/// **The water's lattice and its alpha** (item 4178): every lattice point
/// of a wet cell, [`water_fade`] (at `read.solid`) of the corner height at
/// tile `(2 · gx, 2 · gy)` — or, on an odd row under `read.north`, the
/// even row's — then `generate_water`'s pass: each point, in row order,
/// takes the mean of its own and its laid 3 × 3 neighbours' alphas (out of
/// 1), the smaller of that and its own, and stores its square, truncated.
pub fn water_grid_with(t: &sim::observe::TerrainScene, read: WaterRead) -> WaterGrid {
    use render::terrain::TILES_PER_CELL;
    let (nx, ny) = (2 * t.width + 1, 2 * t.height + 1);
    let mut laid = vec![false; (nx * ny) as usize];
    for cy in 0..t.height {
        for cx in 0..t.width {
            if !holds_water(t, cx, cy) {
                continue;
            }
            for j in 0..=2 {
                for i in 0..=2 {
                    laid[((2 * cy + j) * nx + 2 * cx + i) as usize] = true;
                }
            }
        }
    }
    let sx = t.width * TILES_PER_CELL + 1;
    let fade = |h: f32| water_fade_of(h, read.solid);
    let mut alpha: Vec<Option<u8>> = (0..ny)
        .flat_map(|gy| (0..nx).map(move |gx| (gx, gy)))
        .map(|(gx, gy)| {
            if !laid[(gy * nx + gx) as usize] {
                return None;
            }
            let ty = if read.north {
                2 * gy - 2 * (gy & 1)
            } else {
                2 * gy
            };
            let k = (ty * sx + 2 * gx) as usize;
            Some(t.corners.get(k).map_or(0, |&z| fade(f32::from_bits(z))))
        })
        .collect();
    let source = alpha.clone();
    for gy in 0..ny {
        for gx in 0..nx {
            let i = (gy * nx + gx) as usize;
            let Some(own) = alpha[i] else { continue };
            let (mut sum, mut n) = (0.0f32, 0.0f32);
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let (x, y) = (gx + dx, gy + dy);
                    if !(0..nx).contains(&x) || !(0..ny).contains(&y) {
                        continue;
                    }
                    let j = (y * nx + x) as usize;
                    let from = if read.in_place { &alpha } else { &source };
                    if let Some(a) = from[j] {
                        sum += f32::from(a) / 255.0;
                        n += 1.0;
                    }
                }
            }
            let own = f32::from(own) / 255.0;
            let m = (sum / n).min(own);
            alpha[i] = Some(((m * m * 255.0) as i32 & 0xff) as u8);
        }
    }
    WaterGrid { nx, ny, alpha }
}

/// **The water's own mesh** (item 4178; `docs/RENDERER.md` §3.12):
/// `TerrainOut::modify_water@0086e700`'s draw. Each wet cell's four
/// 384-unit quads, each four triangles about its centre, which takes the
/// mean of the quad's four corners' alphas, truncated (540 of run1564's
/// 540 drawn centres). Every vertex stands at z 0: all 1,755 of run1564's
/// do, 111 of them over ground above it.
#[derive(Clone, Debug, Default)]
pub struct WaterMesh {
    pub tris: Vec<[[f32; 3]; 3]>,
    /// Each corner's texcoord inside its cell, `[0, 1]` across it, as
    /// [`tile_uvs`] lays the base's.
    pub uvs: Vec<[[f32; 2]; 3]>,
    /// Each corner's alpha out of 255.
    pub alpha: Vec<[f32; 3]>,
    /// The cell each triangle lies on.
    pub cell: Vec<(i32, i32)>,
}

/// The lattice's spacing in world units: half a cell.
pub const WATER_STEP: f32 = 384.0;

pub fn water_mesh(t: &sim::observe::TerrainScene, g: &WaterGrid) -> WaterMesh {
    let mut out = WaterMesh::default();
    let cell = 2.0 * WATER_STEP;
    for cy in 0..t.height {
        for cx in 0..t.width {
            if !holds_water(t, cx, cy) {
                continue;
            }
            for (i, j) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                let (gx, gy) = (2 * cx + i, 2 * cy + j);
                let corner = |dx: i32, dy: i32| {
                    let a = g.at(gx + dx, gy + dy).unwrap_or(0);
                    let (x, y) = ((gx + dx) as f32 * WATER_STEP, (gy + dy) as f32 * WATER_STEP);
                    (
                        [x, y, 0.0],
                        [x / cell - cx as f32, y / cell - cy as f32],
                        u32::from(a),
                    )
                };
                let q = [corner(0, 0), corner(1, 0), corner(1, 1), corner(0, 1)];
                let mean = (q.iter().map(|c| c.2).sum::<u32>() / 4) as f32;
                let (x, y) = (
                    (gx as f32 + 0.5) * WATER_STEP,
                    (gy as f32 + 0.5) * WATER_STEP,
                );
                let centre = (
                    [x, y, 0.0],
                    [x / cell - cx as f32, y / cell - cy as f32],
                    mean,
                );
                for k in 0..4 {
                    let (a, b) = (&q[k], &q[(k + 1) % 4]);
                    out.tris.push([centre.0, a.0, b.0]);
                    out.uvs.push([centre.1, a.1, b.1]);
                    out.alpha.push([centre.2, a.2 as f32, b.2 as f32]);
                    out.cell.push((cx, cy));
                }
            }
        }
    }
    out
}

/// **The ocean floor's plane** (item 4174; `docs/RENDERER.md` §3.12):
/// `TerrainOut::build_ocean_floor@0086c7f0` lays one flat vertex per cell
/// corner at `coast_depth + land_height − 10`; every one of run1564's 624
/// drawn floor vertices (`scene_ocean_floor`) stands at −283.
#[cfg(test)]
pub const FLOOR_Z: f32 = -283.0;

/// **The floor's texture repeat** (item 4174), in world units: run1564's
/// floor texcoords are `xy / 6144` plus an offset on the first layer and
/// `−xy / 6144` plus another on the second, exactly — one repeat every
/// eight cells, the second layer mirrored. The offsets drift with
/// `modify_ocean_floor_texture@0086e320`'s clock and are not drawn.
pub const FLOOR_REPEAT: f32 = 6144.0;

/// The floor's two texcoords at world `(x, y)` (item 4174).
#[cfg(test)]
pub fn floor_uvs(x: f32, y: f32) -> ([f32; 2], [f32; 2]) {
    (
        [x / FLOOR_REPEAT, y / FLOOR_REPEAT],
        [-x / FLOOR_REPEAT, -y / FLOOR_REPEAT],
    )
}

/// **`OceanVars`' floor bytes** (item 4174; `TerrainData +0x5524`, read on
/// run1564's packet): `floor*_top` (255, 255, 255) and `floor*_bot`
/// (64, 64, 64), alphas 1.
pub const FLOOR_TOP: u8 = 255;
pub const FLOOR_BOT: u8 = 64;

/// **The land's depth fade begins** (item 4192; `docs/RENDERER.md` §3.16): `__real_c2500000`, −52, in
/// `TerrainOut::light_land_vert@00866e50`.
pub const LAND_FADE_TOP: f32 = -52.0;

/// **`coast_depth`** (item 4192): terrain `+0x5630`, −303 on run1564's
/// packet. `build_ocean_floor`'s plane is `coast_depth + land_height − 10`,
/// [`FLOOR_Z`]'s −283.
pub const COAST_DEPTH: f32 = -303.0;

/// **`land_height`** (item 4192): the global at `0x00cbe54c`
/// (`sbl/rise_z.map`), 30.0 on run1564's packet.
pub const LAND_HEIGHT: f32 = 30.0;

/// **A land corner's alpha** (item 4192; `docs/RENDERER.md` §3.16): `TerrainOut::light_land_vert@00866e50` on the corner
/// height `h`. At or above 0 the corner keeps `generate_land`'s white,
/// 255. Below 0 the alpha is `(1 − m · (1 / span)) · 255`, truncated, with
/// `m = min(|−52 − min(h, −52)|, span)` and `span = |coast_depth +
/// land_height + 2 + 52|`, 219. So it is 255 down to −52 and 0 from −271.
/// Every land-pass draw blends at it (`base_terrain_rs`, source alpha over,
/// depth written): on run1564's packet it is `master_land`'s alpha on
/// 58,081 of 58,081 corners, and the sand layer's on 4,928 of 4,928
/// vertices ([`RUN1564_LAND_FADE`]).
pub fn land_alpha(h: f32) -> u8 {
    land_alpha_of(h, LAND_HEIGHT)
}

/// [`land_alpha`] at another `land_height`: the made-to-fail side of
/// [`RUN1564_LAND_FADE`].
pub fn land_alpha_of(h: f32, land_height: f32) -> u8 {
    if h >= 0.0 {
        return 255;
    }
    let span = (COAST_DEPTH + land_height + 2.0 - LAND_FADE_TOP).abs();
    let m = (LAND_FADE_TOP - h.min(LAND_FADE_TOP)).abs().min(span);
    ((1.0 - m * (1.0 / span)) * 255.0) as u8
}

/// **`OceanVars`' water bytes** (item 4174): `top*_top` and `top*_bot`,
/// read on run1564's packet as (0, 159, 255) at the screen's top and
/// (0, 111, 218) at its foot. `oceanred` … (255, 255, 255) is the white
/// `create_water_verts` lays; `TerrainOut::modify_water@0086e700` replaces
/// it with these each frame.
pub const WATER_TOP: [u8; 3] = [0, 159, 255];
pub const WATER_BOT: [u8; 3] = [0, 111, 218];

/// **The screen's ramp** (item 4174): `TerrainOut::determine_frustum_
/// intersect@00854770`'s two ground points — under the screen's top-left
/// pixel and its bottom-left — and the fraction a world point lies along
/// the view's depth between them: `((P − TL)·(1, 1)/√2) / |BL − TL|`,
/// clamped to [0, 1]. `modify_ocean_floor@00873960` tints the floor and
/// `modify_water` the water by it; the lateral axis does not enter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenRamp {
    /// `TL · (1, 1)/√2`.
    pub d0: f32,
    /// `|BL − TL|`.
    pub len: f32,
}

impl ScreenRamp {
    /// The ramp of `camera` on a `(w, h)` screen: each corner unprojected
    /// onto the plane z = 0 through `to_screen_matrix`, as
    /// `RenderWindow::mouse_to_xy_plane` takes it.
    pub fn of(camera: &render::Camera, (w, h): (u32, u32)) -> ScreenRamp {
        let m = camera.to_screen_matrix().0;
        let ground = |px: f32, py: f32| {
            let sx = 2.0 * px / w as f32 - 1.0;
            let sy = 1.0 - 2.0 * py / h as f32;
            let (a, b, e) = (m[0] - sx * m[3], m[4] - sx * m[7], sx * m[15] - m[12]);
            let (c, d, f) = (m[1] - sy * m[3], m[5] - sy * m[7], sy * m[15] - m[13]);
            let det = a * d - b * c;
            [(e * d - b * f) / det, (a * f - e * c) / det]
        };
        let tl = ground(0.0, 0.0);
        let bl = ground(0.0, h as f32);
        ScreenRamp {
            d0: (tl[0] + tl[1]) * std::f32::consts::FRAC_1_SQRT_2,
            len: (bl[0] - tl[0]).hypot(bl[1] - tl[1]),
        }
    }

    /// A ramp that lays every point at its middle, 0.5: what the ground
    /// is drawn at before a screen is set.
    pub const MIDDLE: ScreenRamp = ScreenRamp {
        d0: -1.0e12,
        len: 2.0e12,
    };

    /// The fraction at world `(x, y)`.
    pub fn at(&self, x: f32, y: f32) -> f32 {
        (((x + y) * std::f32::consts::FRAC_1_SQRT_2 - self.d0) / self.len).clamp(0.0, 1.0)
    }
}

/// **The floor's vertex grey** at ramp fraction `f` (item 4174):
/// `modify_ocean_floor`'s lerp of [`FLOOR_TOP`] to [`FLOOR_BOT`], times
/// 0.6 (`__real_3f19999a`), truncated — 153 at the top, 38 at the foot.
pub fn floor_grey(f: f32) -> u8 {
    let v = f32::from(FLOOR_TOP) * (1.0 - f) + f32::from(FLOOR_BOT) * f;
    (v / 255.0 * 0.6 * 255.0) as u8
}

/// **The water's vertex colour** at ramp fraction `f` (item 4174):
/// `modify_water`'s lerp of [`WATER_TOP`] to [`WATER_BOT`], truncated.
pub fn water_rgb(f: f32) -> [u8; 3] {
    [0, 1, 2].map(|k| {
        let v = f32::from(WATER_TOP[k]) * (1.0 - f) + f32::from(WATER_BOT[k]) * f;
        (v / 255.0 * 255.0) as u8
    })
}

/// **The floor's stage colour** (item 4174): the card's draw 36 on run1567,
/// `MODULATE2X(TEXTURE, DIFFUSE)` then `MODULATE(TEXTURE, CURRENT)` — the
/// first layer's texel doubled under the vertex grey, clamped, times the
/// second's — on 0–255 channels.
pub fn floor_stage(t0: [f32; 3], t1: [f32; 3], grey: u8) -> [f32; 3] {
    let g = f32::from(grey) / 255.0;
    [0, 1, 2].map(|k| (2.0 * t0[k] * g).min(255.0) * t1[k] / 255.0)
}

/// **The floor's two layers as one texture** (item 4178): `render_ocean_floor`
/// binds `<OCEAN>`'s texture twice, the second layer at the first's
/// texcoord mirrored about `c` (the two layers' offsets summed, per axis,
/// in repeats), and [`floor_stage`] combines them under `grey`. Texel
/// `(x, y)` is the stage's colour at the first layer's texel `(x, y)`;
/// the first layer's texcoord samples it.
pub fn floor_texture(tex: &render::tga::Image, c: [f32; 2], grey: u8) -> render::tga::Image {
    let (w, h) = (tex.width as i64, tex.height as i64);
    let at = |x: i64, y: i64| {
        let i = (y.rem_euclid(h) * w + x.rem_euclid(w)) as usize * 4;
        [0, 1, 2].map(|k| f32::from(tex.rgba[i + k]))
    };
    let (mx, my) = (
        (c[0] * w as f32).round() as i64 - 1,
        (c[1] * h as f32).round() as i64 - 1,
    );
    let mut rgba = Vec::with_capacity(tex.rgba.len());
    for y in 0..h {
        for x in 0..w {
            let f = floor_stage(at(x, y), at(mx - x, my - y), grey);
            rgba.extend(f.map(|v| v.round().clamp(0.0, 255.0) as u8));
            rgba.push(255);
        }
    }
    render::tga::Image {
        width: tex.width,
        height: tex.height,
        kind: 2,
        bits: 32,
        footer: false,
        rgba,
    }
}

/// **The water's colour as a texture** (item 4178): 256 texels, texel `i`
/// [`water_rgb`] at fraction `i / 255`, so a vertex at fraction `f` takes
/// the texcoord `(255 f + 0.5) / 256` and the rasteriser's blend of two is
/// the card's Gouraud blend of their colours.
pub fn water_colour_ramp() -> Mips {
    let rgba = (0..=255u32)
        .flat_map(|i| {
            let c = water_rgb(i as f32 / 255.0);
            [c[0], c[1], c[2], 255]
        })
        .collect();
    Mips::new(render::tga::Image {
        width: 256,
        height: 1,
        kind: 2,
        bits: 32,
        footer: false,
        rgba,
    })
}

/// A ramp fraction as a texcoord into a 256-texel ramp (item 4178).
pub fn ramp_uv(f: f32) -> [f32; 2] {
    [(f.clamp(0.0, 1.0) * 255.0 + 0.5) / 256.0, 0.5]
}

/// **The floor's shade** (item 4178): the floor is baked at its brightest
/// vertex grey ([`floor_grey`] at 0, 153) and drawn opaque; a black layer
/// over it at alpha `1 − grey / 153` leaves `texel · grey / 153`, the
/// stage's product wherever `MODULATE2X` does not clamp. Out of 255.
pub fn floor_shade(f: f32) -> f32 {
    let top = f32::from(floor_grey(0.0));
    255.0 * (1.0 - f32::from(floor_grey(f)) / top)
}

/// **The floor's layers' offsets on run1564's packet** (item 4178), each
/// layer's `uv − xy / 6144` (first) and `uv + xy / 6144` (second), summed:
/// (1.178 + 8.750, 1.176 + 8.680) at logger frame 3294. They drift with
/// `modify_ocean_floor_texture`'s clock, which is not read.
pub const FLOOR_MIRROR: [f32; 2] = [9.928, 9.856];

/// **The transition patch laid** (item 4139; `docs/RENDERER.md` §3.9):
/// every `BASELAND` tile not under a placed mountain whose cell
/// [`crate::patch::patches`] patches, its two triangles as [`ground_base`]
/// gives them, each corner's texcoord into the patch's base (the base of
/// `land_sub − 1`, the cell's own sub-tile) and into the alpha (the
/// patch's grid point, in eighths of the alpha), the patch's base's index
/// into [`GroundTextures::flat`] and the alpha's index.
#[derive(Clone, Debug, Default)]
pub struct GroundPatch {
    pub tris: Vec<[[f32; 3]; 3]>,
    pub uvs: Vec<[[f32; 2]; 3]>,
    pub alpha_uvs: Vec<[[f32; 2]; 3]>,
    pub of: Vec<usize>,
    pub alpha: Vec<usize>,
    pub tile: Vec<(i32, i32)>,
    /// Each triangle's cell's patch, for [`patch_alpha`].
    pub patch: Vec<crate::patch::Patch>,
}

pub fn ground_patch(
    t: &sim::observe::TerrainScene,
    mountains: &std::collections::BTreeSet<(i32, i32)>,
    tex: &GroundTextures,
    mesh: &render::terrain::TerrainMesh,
    choice: &[Option<crate::patch::Patch>],
) -> GroundPatch {
    use render::terrain::{BASELAND, OCEAN, SANDY, TILES_PER_CELL};
    let mut out = GroundPatch::default();
    let (tw, th) = (t.width * TILES_PER_CELL, t.height * TILES_PER_CELL);
    if mesh.positions.len() != (tw * th * 6) as usize || choice.len() != t.land.len() {
        return out;
    }
    for ty in 0..th {
        for tx in 0..tw {
            let (cx, cy) = (tx / TILES_PER_CELL, ty / TILES_PER_CELL);
            let c = (cy * t.width + cx) as usize;
            if !matches!(t.land.get(c), Some(&(BASELAND | SANDY | OCEAN)))
                || mountains.contains(&(tx, ty))
            {
                continue;
            }
            let Some(q) = choice[c] else {
                continue;
            };
            let Some(k) = tex.patch_index(t.land[c], t.land_sub[c], cx, cy) else {
                continue;
            };
            let i = ((ty * tw + tx) * 6) as usize;
            let [a, b, cc, d] = tile_uvs(tx, ty);
            // The tile's corners on the patch's 5 × 5 grid, vertex `j·5 + i`.
            let (gi, gj) = (tx.rem_euclid(4) as usize, ty.rem_euclid(4) as usize);
            let g = |di: usize, dj: usize| {
                let v = q.uv[(gj + dj) * 5 + gi + di];
                [v[0] as f32 / 8.0, v[1] as f32 / 8.0]
            };
            let [ga, gb, gc, gd] = [g(0, 0), g(1, 0), g(1, 1), g(0, 1)];
            for (j, (uv, auv)) in [([a, b, d], [ga, gb, gd]), ([b, cc, d], [gb, gc, gd])]
                .into_iter()
                .enumerate()
            {
                let v = &mesh.positions[i + j * 3..i + j * 3 + 3];
                out.tris.push([v[0], v[1], v[2]]);
                out.uvs.push(uv);
                out.alpha_uvs.push(auv);
                out.of.push(k);
                out.alpha.push(q.alpha);
                out.tile.push((tx, ty));
                out.patch.push(q);
            }
        }
    }
    out
}

/// **The patch's weight at a point of its cell** (§3.9, the pass as
/// read): the alpha's A at the patch's texcoord for the cell's `(u, v)` in
/// `[0, 1]`, on level 0, the nearest texel, clamped — stage 1's
/// `MIP_POINT` filters and `TEXADD_CLAMP` — over 255. The patch's base is
/// laid over the base at this weight (`SRCALPHA`, `INVSRCALPHA`).
#[cfg_attr(not(test), allow(dead_code))]
pub fn patch_alpha(alpha: &Mips, q: &crate::patch::Patch, uv: [f32; 2]) -> f32 {
    let e = |k: usize, c: usize| q.uv[k][c] as f32 / 8.0;
    let au = e(0, 0) + uv[0] * (e(4, 0) - e(0, 0)) + uv[1] * (e(20, 0) - e(0, 0));
    let av = e(0, 1) + uv[0] * (e(4, 1) - e(0, 1)) + uv[1] * (e(20, 1) - e(0, 1));
    let im = &alpha.levels[0];
    let x = ((au * im.width as f32).floor() as i64).clamp(0, im.width as i64 - 1) as usize;
    let y = ((av * im.height as f32).floor() as i64).clamp(0, im.height as i64 - 1) as usize;
    f32::from(im.rgba[(y * im.width + x) * 4 + 3]) / 255.0
}

/// The 14 transition alphas, in name order (`Transitions::init`'s table),
/// each with its eight orientations' codes.
pub fn load_alphas(root: &Path) -> Result<Vec<(Mips, [u32; 8])>, String> {
    let dir = root.join("terrain art").join("transitions");
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .filter(|n| n.ends_with(".tga"))
        .collect();
    names.sort();
    names
        .into_iter()
        .map(|n| {
            let codes = crate::patch::bitcodes(&n).ok_or_else(|| format!("{n}: not a code"))?;
            let bytes = std::fs::read(dir.join(&n)).map_err(|e| format!("{n}: {e}"))?;
            let image = render::tga::decode(&bytes).map_err(|e| format!("{n}: {e:?}"))?;
            Ok((Mips::new(image), codes))
        })
        .collect()
}

/// **The patch's choice on the world the client opens** (§3.8, §3.9): our
/// cells, the tileset's `COAST_BASE` and `BASELAND` count read through
/// `rondata::mapgen::tileset_groups`, and the texture fractal on
/// `World.seed`.
pub fn our_patches(
    t: &sim::observe::TerrainScene,
    root: &Path,
    tileset: &str,
    seed: u32,
    codes: &[[u32; 8]],
) -> Option<Vec<Option<crate::patch::Patch>>> {
    let numbers = rondata::mapgen::tileset_groups(root, tileset)?;
    let cells: Vec<(i8, u8)> = t
        .land
        .iter()
        .zip(&t.land_sub)
        .map(|(&l, &s)| (l, s))
        .collect();
    let frac = sim::mapgen::Fractal::init(t.width + 1, t.height + 1, 0, 2, seed);
    Some(crate::patch::patches(
        &crate::patch::Ground {
            width: t.width,
            height: t.height,
            cells: &cells,
            coast_base: u8::try_from(numbers.coast_base).ok()?,
            base_subs: u8::try_from(numbers.base_tiles).ok()?,
        },
        &frac,
        codes,
    ))
}

/// **`RUN1559B_3279_GROUND`** (item 4127; `docs/RENDERER.md` §3.6, "The
/// ground's base layer"): of the scored tiles on run1559b's frame 3279 at
/// its pinned camera, those whose drawn mean colour is the photograph's
/// within the colour reader's 0.05 in (r, g)/(r + g + b). Item 4129: it
/// backs the tileset and the mapping, not the choice of base — every tile
/// drawn in base 2 scores 1,053 and in base 0 1,052; the choice's row is
/// [`RUN1559B_3279_GROUND_BASE`]. Item 4139: re-pinned with the
/// transition patch laid over the base, our own choice on our own cells
/// (§3.9); the base alone is [`RUN1559B_3279_GROUND_BARE`]. Item 4141:
/// 1,051 → 1,097 with a mountain's own draw over the ground beneath it
/// (§3.10): the 106 under a mountain from 59 to 105, 46 gained and none
/// lost, (12, 141) the one left; the grey is [`RUN1559B_3279_GROUND_GREY`].
#[cfg(test)]
pub const RUN1559B_3279_GROUND: (usize, usize) = (1097, 1109);

/// **`RUN1559B_3279_GROUND` on the client's still** (item 4171;
/// `docs/RENDERER.md` §3.15): the painter's 1,097 of 1,109,
/// tile for tile, no crossing. On the draw before (an sRGB target) 1,003:
/// 94 of the 106 under a mountain cross down, `ADDSIGNED` taken in linear
/// light — tile (6, 144), theirs (104, 100, 56), the painter's (116, 97,
/// 65), the still's (22, 18, 6). Item 4199: **1,097 → 1,104**, the roads
/// drawn (`crate::roads`, §3.19): seven plain tiles under the capital's
/// ring cross up, the asphalt's mean where grass stood.
#[cfg(test)]
pub const RUN1559B_3279_GROUND_STILL: (usize, usize) = (1104, 1109);

/// **`RUN1564_3279_GROUND` on the client's still** (item 4171):
/// `(agree, scored)`, then by kind `[agree, scored]`, `KINDS`' order.
/// Item 4178: **1,237 → 1,373**, the water on its own mesh at z 0 with
/// `generate_water`'s alpha and `modify_water`'s colour, the floor under
/// it: `SANDY` 295 → 664 of 772, `OCEAN` 233 → 0 of 437 (the water's red,
/// 4174a), the land unchanged. Tile (159, 194) at pixel (158, 6), theirs
/// (46, 113, 162), was (114, 195, 207) on the still and parts; tile
/// (163, 189), theirs (62, 124, 161), the still's (55, 186, 230) before.
/// Item 4192: **1,373 → 1,024**, the land pass at `light_land_vert`'s
/// depth alpha over the clear colour ([`land_alpha`], 4,928 of 4,928 of
/// the sand layer's vertices, [`RUN1564_LAND_FADE`]): `SANDY` 664 → 315.
/// The deep sand had agreed only because it lay opaque under the water;
/// at −273 its alpha is 0, and what is left there is the water over black,
/// short of the photograph's red as `OCEAN` is (4174a). Tile (159, 194) at
/// pixel (158, 6), theirs (46, 113, 162), the still's (65, 168, 217) →
/// (1, 117, 188), parts. Its luminance gains 317 ([`GROUND_LUMA_STILL`]).
/// Item 4200: **1,024 → 1,028**, the coast's transition patches
/// (`docs/RENDERER.md` §3.18; [`crate::patch::RUN1564_3294_COAST_PATCH`]):
/// `COAST_BASE`'s grass over 319 `SANDY` cells' edges, and on 120 `OCEAN`
/// cells `sand.tga` with the tileset's coral over it at the patch's alpha.
/// `SANDY` 315 → 319: tile (171, 206) at pixel (148, 333), theirs (149,
/// 154, 118), the base's sand (126, 191, 215) on the painter, the still's
/// (164, 161, 124). `OCEAN` stays 0: with the sand patch and no coral it
/// read 132 (the journal), the coral's cyan taking the sand's red back.
#[cfg(test)]
/// Item 4212: **1,028 → 1,021**, the land lit at the frame's white, §3.21: plain 677 → 679, `ROCK` 32 → 31, `SANDY` 319 → 311.
pub const RUN1564_3279_GROUND_STILL: ((usize, usize), [[u32; 2]; 7]) = (
    (1021, 1920),
    [
        [679, 679],
        [0, 0],
        [31, 32],
        [0, 0],
        [0, 0],
        [311, 772],
        [0, 437],
    ],
);

/// A still's brightness row: `(agree, scored)` and by kind.
#[cfg(test)]
pub type LumaStill = ((u32, usize), [[u32; 2]; 7]);

/// **The ground's brightness on the client's still** (item 4178):
/// [`RUN1564_3279_GROUND_LUMA`]'s reader on the still's tiles, run1559b
/// then run1564, `(agree, scored)` and by kind. run1564 **791 → 1,001**:
/// at the base the still read the painter's 791 (`OCEAN` 2, `SANDY` 83);
/// with the floor and the water drawn, `OCEAN` 2 → 215 of 437 and `SANDY`
/// 83 → 80 of 772. run1559b 1,076, unchanged. Item 4192: run1564 **1,001
/// → 1,318**, the land's depth fade ([`land_alpha`]): `SANDY` 80 → 397 of
/// 772, the deep sand now the water over the clear colour; run1559b
/// unchanged. Item 4200: run1564 **1,318 → 1,377**, the coast's patches:
/// `SANDY` 397 → 499 (the beach's edge under `COAST_BASE`'s grass),
/// `OCEAN` 215 → 172 (the sand and coral patches under the water, brighter
/// than the floor); run1559b unchanged. Item 4199: run1559b **1,076 →
/// 1,075**, the roads drawn (§3.19): plain 980 → 979 of 986, one ring tile
/// whose asphalt is 0.89 of ours (parked 4199b); run1564 unchanged.
#[cfg(test)]
/// Item 4212: run1559b **1,075 → 1,029** (plain 979 → 952, under a mountain 80 → 61); run1564 **1,377 → 1,396** (`ROCK` 27 → 21, `SANDY` 499 → 397, `OCEAN` 172 → 299): the land lit at the frame's white, §3.21.
pub const GROUND_LUMA_STILL: [LumaStill; 2] = [
    (
        (1029, 1109),
        [
            [952, 986],
            [61, 106],
            [16, 17],
            [0, 0],
            [0, 0],
            [0, 0],
            [0, 0],
        ],
    ),
    (
        (1396, 1920),
        [
            [679, 679],
            [0, 0],
            [21, 32],
            [0, 0],
            [0, 0],
            [397, 772],
            [299, 437],
        ],
    ),
];

/// Both ground rows on the draw before item 4171 (the silhouette after,
/// an sRGB target): the rows' made-to-fail, run1559b then run1564. Item
/// 4178: run1564's 1,286 → 834, the water's colour taken to linear light.
/// Item 4192: run1564's 834 → 927, the land's depth fade. Item 4200:
/// run1564's 927 → 970, the coast's patches. Item 4199: run1559b's 1,003
/// → 1,010, the roads drawn (§3.19).
#[cfg(test)]
/// Item 4212: run1559b's 1,010 → 992, run1564's 970 → 1,043, the land lit at the frame's white, §3.21.
/// On the tree merged with 4208 (the gaia figures' shipped variant), run1564's 1,043 → 1,044.
pub const GROUND_STILL_SRGB: [(usize, usize); 2] = [(992, 1109), (1044, 1920)];

/// **`RUN1559B_3279_GROUND_LUMA`** (item 4172; `docs/RENDERER.md` §3.14,
/// "On brightness", its rule committed before this count): of
/// [`RUN1559B_3279_GROUND`]'s 1,109 scored tiles, those whose drawn mean
/// luminance is the photograph's within `LUMA_READER`: **1,075**; by kind
/// `[agree, scored]` plain, under a mountain, `ROCK`. The painter's
/// ground sits under the photograph's (median 0.90 plain, 0.89 under a
/// mountain); the mountain's body, the pass's box, is eight scored tiles
/// at 0.83 to 0.92, agreeing on the painter where the client's still read
/// it at 0.37 before item 4172's `fs_mountain`.
#[cfg(test)]
pub const RUN1559B_3279_GROUND_LUMA: ((usize, usize), [[u32; 2]; 3]) =
    ((1075, 1109), [[980, 986], [79, 106], [16, 17]]);

/// **`RUN1564_3279_GROUND_LUMA`** (item 4172): [`RUN1564_3279_GROUND`]'s
/// tiles on the same reader, `(agree, scored)` and by kind, `KINDS`'
/// order: **791 of 1,920**. The land agrees (plain 679 of 679, `ROCK` 27
/// of 32); the water and the shore are too bright, `OCEAN` 2 of 437
/// (median 1.69, the lake's tiles 1.8 to 2.0) and `SANDY` 83 of 772
/// (median 1.55): the row item 4174 carries beside its own.
#[cfg(test)]
pub const RUN1564_3279_GROUND_LUMA: ((usize, usize), [[u32; 2]; 7]) = (
    (791, 1920),
    [
        [679, 679],
        [0, 0],
        [27, 32],
        [0, 0],
        [0, 0],
        [83, 772],
        [2, 437],
    ],
);

/// [`RUN1559B_3279_GROUND`] with the patch laid and the mountains in the
/// debug grey, as item 4139 pinned it (item 4141).
#[cfg(test)]
pub const RUN1559B_3279_GROUND_GREY: (usize, usize) = (1051, 1109);

/// **`RUN1559B_3279_MOUNTAIN_VALUES`** (item 4141, stage 0; `docs/RENDERER.md`
/// §3.10): the scored tiles under a placed mountain (all 106 on range 1,
/// template 14, at cell (4, 36)); each candidate's agreement under the row's
/// bar, `[grey, the ground as if no mountain stood, BASETEX's mean,
/// ALPHATEX's mean]`; the nearest by tile, the same order; and the ties.
#[cfg(test)]
pub const RUN1559B_3279_MOUNTAIN_VALUES: (u32, [u32; 4], [u32; 4], u32) =
    (106, [59, 76, 60, 0], [41, 59, 6, 0], 0);

/// **`RUN1559B_3279_MOUNTAIN_READS`** (item 4141, stage 1; §3.10): the
/// mountain pass read by its `RenderState` (`MountainsOut::mountain_blend`),
/// each read's agreement on the 106 and the tiles where it is strictly
/// nearer than the ground (none tie): as read, the main pass alone, 1 − A,
/// stage 1's colour ignored, the alpha unstretched, and over the grey.
#[cfg(test)]
pub const RUN1559B_3279_MOUNTAIN_READS: ([u32; 6], [u32; 6], u32) =
    ([105, 105, 97, 62, 88, 98], [85, 91, 90, 35, 62, 77], 106);

/// **`RUN1559B_3279_UNSCORED`** (item 4143, stage 0; `docs/RENDERER.md`
/// §7.1): what [`RUN1559B_3279_GROUND`]'s selection leaves out on frame
/// 3279. Every tile in the photograph's view is a `BASELAND` cell's: by
/// kind — plain, under a placed mountain, `ROCK` — `[in view, scored, short
/// of the bar, no ground pixel]`; then the tiles in view that are `SANDY`,
/// `OCEAN` or flagged `COAST` or `HALFLAND`, none; then the pixels off the
/// heightfield, the corner past the map's edge (the photograph's mean there
/// (64, 59, 33), no water). The water, the coast and the cliffs hold no
/// scoreable tile here, so the item's kill holds before a reading (parked
/// 4143a).
#[cfg(test)]
pub const RUN1559B_3279_UNSCORED: ([[u32; 4]; 3], u32, u32) = (
    [[1690, 986, 42, 662], [143, 106, 9, 28], [18, 17, 0, 1]],
    0,
    213_931,
);

/// **run1564's camera** (item 4160): `Camera::loc_x` and `loc_y` off the
/// packet at logger frame 3294 (`tools/render/packet_points.py`'s `state 0`
/// line), as their bits: (36960, 38496), tile (192, 200)'s centre. Every
/// other field is [`crate::pinned_camera`]'s.
#[cfg(test)]
pub const RUN1564_LOC: (u32, u32) = (0x4710_6000, 0x4716_6000);

/// **`RUN1564_3279_CENSUS`** (item 4160; `docs/RENDERER.md` §7.1): the
/// second photograph's tiles in view by kind, `ground::tests::KINDS`'
/// order — plain, under a mountain, `ROCK`, `CLIFF`, a `BASELAND` shore,
/// `SANDY`, `OCEAN` — each `[in view, scored, short of the bar, no ground
/// pixel]`; then the pixels off the heightfield. Three kinds the first
/// photograph shows none of are above zero and scoreable: 488 `OCEAN` tiles
/// in view (437 scored), 901 `SANDY` (772) and 32 `ROCK` (32, against
/// run1559b's 17). No cell of this map carries `CLIFF`, so a cliff is in no
/// photograph of it; no `BASELAND` cell carries `COAST` or `HALFLAND`.
/// The 44 tiles under a mountain show no ground pixel; nothing is off the
/// heightfield.
#[cfg(test)]
pub const RUN1564_3279_CENSUS: ([[u32; 4]; 7], u32) = (
    [
        [1105, 679, 7, 419],
        [44, 0, 0, 44],
        [32, 32, 0, 0],
        [0, 0, 0, 0],
        [0, 0, 0, 0],
        [901, 772, 14, 115],
        [488, 437, 1, 50],
    ],
    0,
);

/// **`RUN1564_3279_SHORE`** (item 4160): the shore in register — `[plain
/// tiles the photograph shows blue, OCEAN tiles it does not]` (a scored
/// tile's ground pixels' mean with `b > r`), at run1564's camera, then with
/// it moved 384, 768 and 1,536 units (two, four and eight tiles) along +x,
/// +y, −x and −y. None at the camera; 0 to 3 at two tiles, up to 40 at
/// four, up to 142 at eight. The shore holds the camera to about two tiles,
/// the width of the `SANDY` belt the waterline falls in, and no closer.
#[cfg(test)]
pub const RUN1564_3279_SHORE: [[u32; 2]; 13] = [
    [0, 0],
    [0, 0],
    [3, 0],
    [0, 0],
    [0, 0],
    [16, 0],
    [40, 0],
    [0, 0],
    [3, 0],
    [127, 0],
    [142, 0],
    [0, 8],
    [39, 24],
];

/// **`RUN1564_3279_GROUND`** (item 4160): [`RUN1559B_3279_GROUND`]'s row,
/// unchanged, on the second photograph: `(agree, scored)`; then by kind,
/// `[agree, scored]`, `KINDS`' order; then the row on the base alone.
/// 709 of 1,920: the land the client textures agrees on 709 of 711 (plain
/// 677 of 679, `ROCK` 32 of 32), and the water and the shore, which it
/// draws in the debug colours, on none of their 1,209 — the debt the next
/// render item reads (`TerrainOut::render_water`).
///
/// **Item 4164: 709 → 1,236.** `SANDY` and `OCEAN` draw their type's first
/// base (`sand.tga`, `ocean_floor_5.tga`) and the water lies over every
/// cell that holds it, the ocean's base at `get_water_fade`'s alpha:
/// `SANDY` 0 → 294 of 772, `OCEAN` 0 → 233 of 437, the land unchanged; on
/// the base alone 584 → 872.
///
/// **Item 4178: 1,236 → 1,233 on the painter**, plain 677 → 674: the
/// painter still lays item 4164's water on the seabed's triangles, and
/// [`holds_water`]'s corner arm now takes every land (the packet's lattice,
/// [`RUN1564_WATER_LATTICE`]), so three plain tiles of `BASELAND` shore
/// cells take it; the client's still, which draws item 4178's water, holds
/// plain at 677 ([`RUN1564_3279_GROUND_STILL`]; parked 4178a).
#[cfg(test)]
pub const RUN1564_3279_GROUND: ((usize, usize), [[u32; 2]; 7], usize) = (
    (1233, 1920),
    [
        [674, 679],
        [0, 0],
        [32, 32],
        [0, 0],
        [0, 0],
        [294, 772],
        [233, 437],
    ],
    872,
);

/// **`RUN1564_3279_WATER`** (item 4164): `[SANDY, OCEAN]` agreeing under
/// [`RUN1564_3279_GROUND`]'s rule with the water's surface as (1) none, the
/// land pass alone; (2) the ocean's base, the draw; (3) `water0000.tga`,
/// the first of the 32 frames `graphics.txt` names, which `render_water`
/// binds to stage 0. The photograph's water is the floor's colour: the
/// frames' navy agrees on no `OCEAN` tile.
#[cfg(test)]
pub const RUN1564_3279_WATER: [[u32; 2]; 3] = [[55, 233], [294, 233], [112, 0]];

/// The drawn lattice's counts, then each reading's (item 4178).
#[cfg(test)]
pub type WaterLattice = ((usize, usize, usize), [(u32, u32, u32); 5]);

/// **`RUN1564_LAND_FADE`** (item 4192; `docs/RENDERER.md` §3.16): run1564's land off its packet
/// (`tools/render/sand_packet.py`, logger frame 3294) against our terrain.
/// `[corners, heights ours within 0.01, alphas land_alpha of their height,
/// alphas land_alpha of ours, sand vertices, on a SANDY cell of ours, their
/// alpha land_alpha of our corner]`. At `land_height` 0 (span 249) the
/// formula agrees on 47,874 corners.
#[cfg(test)]
pub const RUN1564_LAND_FADE: [usize; 7] = [58081, 58081, 58081, 58081, 4928, 4928, 4928];

/// [`RUN1564_LAND_FADE`]'s sand vertices whose alpha [`land_alpha_of`] gives at
/// `land_height` 0: the made-to-fail side of the sand layer's count.
#[cfg(test)]
const ZERO_SAND: usize = 1077;

/// **`RUN1564_WATER_LATTICE`** (item 4178; `docs/RENDERER.md` §3.12): run1564's
/// drawn water off its packet (`modify_water`'s statics at logger frame
/// 3294) against [`water_grid_with`] on our terrain. The drawn lattice: 597
/// distinct corners, 540 centres, and 80 corners whose ground stands above
/// 0 yet whose vertex is at z 0 (so not `max(0, ground)`). Then for each
/// reading — [`WATER_READ`]; odd rows at their own height; the smoothing
/// from a copy; that at solid 220; in place at 220 — `[corners within
/// one, corners we do not lay, centres their four corners' mean]`. Before
/// [`holds_water`]'s corner arm took every land, read 0 left 64 corners
/// unlaid and agreed on 526.
#[cfg(test)]
pub const RUN1564_WATER_LATTICE: WaterLattice = (
    (597, 540, 80),
    [
        (597, 0, 540),
        (404, 0, 540),
        (142, 0, 540),
        (346, 0, 540),
        (141, 0, 540),
    ],
);

/// **`RUN1564_3279_CARD`** (item 4174; `docs/RENDERER.md` §3.12): the
/// second photograph's ground under [`RUN1564_3279_GROUND`]'s rule and
/// 4172's luminance reader, `(ground, luma)`, with the floor and the water
/// (1) as drawn, (2) as the card's stages leave them, unlit, and (3)
/// through the lit branch and the tone map — every input the packet's
/// (`the_floor_and_the_water_off_the_packet`). Both card models lift the
/// luminance row (791 → 1,426, 1,072) and drop the scored row (1,236 →
/// 793, 814): `OCEAN` agrees on 0 of 437 under either, its red channel 1
/// to 20 against the photograph's ~40 (lake tile (182, 191): theirs (38,
/// 97, 146), lit (1, 91, 180)). Neither is drawn (parked 4174a). Item
/// 4178: (1,236, 793, 814) → (1,233, 784, 813) on the wider corner arm of
/// [`holds_water`], the luminance unchanged.
#[cfg(test)]
pub const RUN1564_3279_CARD: [(usize, usize); 3] = [(1233, 791), (784, 1426), (813, 1072)];

/// [`RUN1559B_3279_GROUND`] on the base layer alone, as item 4127 pinned it.
#[cfg(test)]
pub const RUN1559B_3279_GROUND_BARE: (usize, usize) = (949, 1109);

/// The same row on the debug colour, the floor (item 4127).
#[cfg(test)]
pub const RUN1559B_3279_GROUND_FLOOR: (usize, usize) = (92, 1109);

/// **`RUN1559B_3279_GROUND_BASE`** (item 4129; `docs/RENDERER.md` §3.6,
/// "Which base a tile draws"): of the 1,003 scored plain tiles, those where
/// the base our draw lays is the best of the four by the per-tile Pearson r
/// of the photograph's luminance with each base's own sub-tile at the read
/// mapping, and stands above its own control, the same base with the
/// sub-tile's cell index one on. New, 758 of 1,003; its controls, every
/// tile one base, are [`RUN1559B_3279_GROUND_BASE_ONE`]. Item 4139:
/// re-pinned with the rule's draw carrying the transition patch laid
/// (§3.9) in the rule's base's place; the rule alone is
/// [`RUN1559B_3279_GROUND_BASE_BARE`].
#[cfg(test)]
pub const RUN1559B_3279_GROUND_BASE: (usize, usize) = (951, 1003);

/// [`RUN1559B_3279_GROUND_BASE`] on the rule's base alone, as item 4129
/// pinned it.
#[cfg(test)]
pub const RUN1559B_3279_GROUND_BASE_BARE: (usize, usize) = (758, 1003);

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

/// **The transition patch's controls** (item 4132; `docs/RENDERER.md`
/// §3.6, "The transition patch"): of the 503 scored plain tiles whose cell
/// carries a patch on run1559b's packet, those each draw wins — the highest
/// per-tile luminance r of the four and above its own wrong-cell control —
/// for (a) the base alone, (b) the patch's base alone, (c) the two at a
/// uniform half, (d) a linear ramp toward the neighbours of the patch's
/// base. New; counted before the alpha was read. The bar the patch must
/// beat is (a)'s 204.
#[cfg(test)]
pub const RUN1559B_3279_PATCH_CONTROLS: ([u32; 4], u32) = ([204, 148, 31, 115], 503);

/// **The packet's own patch under the read alpha** (item 4132): the same
/// tiles where the patch `create_transition_renderables` left on the packet
/// (its alpha's bucket and `trans_texcoords1`, `tools/render/patches.py`),
/// sampled from the alpha file's A channel and laying the base of
/// `land_sub − 1` by that weight, wins against (a)-(d) — 395 of 503,
/// against (a) 15, (b) 41, (c) 5, (d) 42 in the same pool — and the same
/// alpha turned a quarter turn wins 201 in its place. New. The choice is
/// the packet's, not a rule of ours: this is the draw's ceiling under the
/// read alpha, not `RUN1559B_3279_GROUND_BLEND` (parked 4132a).
#[cfg(test)]
pub const RUN1559B_3279_GROUND_BLEND_PACKET: (u32, [u32; 4], u32, u32) =
    (395, [15, 41, 5, 42], 201, 503);

/// **`RUN1559B_3279_GROUND_BLEND`** (item 4139; `docs/RENDERER.md`
/// §3.9): the patch from our own choice (`crate::patch::patches`) under
/// the pass as read — the patch's base over ours at A, the alpha's level
/// 0, nearest texel, clamped — on the same 503 tiles as
/// [`RUN1559B_3279_GROUND_BLEND_PACKET`], in the pool of (a)-(d): `(wins
/// on the packet's cells, the controls' wins beside it, wins on our own
/// cells with our own patch base, the controls' wins beside it, of)`.
/// New. Stage 0 (the wiring, 4132's bilinear read) gave 395 and 395,
/// equal to the packet's own; the pass's nearest texel moves 24 tiles in
/// and 17 out. No scored tile sits on the 79 cells where our cells or our
/// choice part from the packet's, so the two counts agree.
#[cfg(test)]
pub const RUN1559B_3279_GROUND_BLEND: (u32, [u32; 4], u32, [u32; 4], u32) =
    (402, [15, 32, 5, 44], 402, [15, 32, 5, 44], 503);

/// **Stage 2's reads of the alpha** (item 4139; §3.9), our choice on the
/// packet's cells, each against (a)-(d) on the 503: `([A level 0
/// bilinear, 1 − A level 0 bilinear, A level 0 point, 1 − A level 0
/// point, A the pass's level bilinear, A the pass's level point, 1 − A
/// the pass's level point], the tiles whose pass's level is 0)`. New: A
/// is kept and 1 − A falls; the pass's level (stage 1's `MIP_POINT`, bias
/// −1.5, λ −1.27 to −0.47) is 0 on all 503.
#[cfg(test)]
pub const RUN1559B_3279_PATCH_READS: ([u32; 7], u32) = ([395, 120, 402, 122, 395, 402, 122], 503);

/// **The drawn patches alone** (item 4139; §3.9): of the 503, the 376
/// tiles whose cell our choice patches; `((e) the packet's patch under
/// 4132's read, (g) ours under the pass as read, the controls (a)-(d)'s
/// wins in their own pool, of)`. New. The other 127 are emptied patches
/// where every read draws the base alone, a tie with (a) that `max_by`
/// gives the later candidate: 120 of 4132's 395 and of the 402 are those.
#[cfg(test)]
pub const RUN1559B_3279_GROUND_BLEND_DRAWN: (u32, u32, [u32; 4], u32) =
    (275, 282, [84, 147, 31, 112], 376);

/// **A mountain's own draw** (item 4141; `docs/RENDERER.md` §3.10): the
/// art `TerrainOut::render_mountains` binds. Stage 0 is the tileset's
/// `<MOUNTAINS>` `BASETEX` (`mountain_base`, the main pass) or `ALPHATEX`
/// (`mountain_alpha`, the ring pass); stage 1 is the range's
/// `MAIN_ALPHA_TEX` or `RING_ALPHA_TEX`, indexed by template.
pub struct MountainArt {
    pub plain: Mips,
    pub blend: Mips,
    pub templates: Vec<Option<MountainTemplateArt>>,
}

/// One template's tiles (to find a range's template by them), its two
/// alphas and the lit bounding box of its `TEMPLATE_TEX`, `[x0, y0, x1,
/// y1]` in its pixels: `MountainRange::init`'s rect, over which the alpha
/// texcoord runs from 0 to 1.
pub struct MountainTemplateArt {
    pub tiles: Vec<(i32, i32)>,
    pub main: Mips,
    pub ring: Mips,
    pub lit: [f32; 4],
}

impl MountainArt {
    /// Item 4212 (§3.21): `mountain_base` and `mountain_alpha` take the
    /// tileset's tint (`TileSet::init@0087b370`).
    pub fn load(root: &Path, tileset: &str) -> Result<MountainArt, String> {
        Self::load_tinted(root, tileset, crate::tint::of(root, tileset))
    }

    /// The files' own texels, untinted: the painter's.
    #[cfg(test)]
    pub fn untinted(root: &Path, tileset: &str) -> Result<MountainArt, String> {
        Self::load_tinted(root, tileset, [0; 3])
    }

    fn load_tinted(root: &Path, tileset: &str, tint: [u8; 3]) -> Result<MountainArt, String> {
        let install = rondata::Install::new(root);
        let load = |rel: &str| -> Result<Mips, String> {
            let path =
                rondata::artdata::resolve_art(root, rel).ok_or_else(|| format!("no file {rel}"))?;
            let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let mut image = render::tga::decode(&bytes).map_err(|e| format!("{rel}: {e:?}"))?;
            image.tint(tint);
            Ok(Mips::new(image))
        };
        let (base, alpha) =
            rondata::artdata::mountain_textures(&install, tileset).map_err(|e| format!("{e:?}"))?;
        let shapes = rondata::mountains::templates(&install);
        let files = rondata::mountains::template_files(&install);
        let alphas = rondata::mountains::alpha_files(&install);
        let templates = (0..files.len())
            .map(|k| {
                let (main, ring) = alphas.get(k)?;
                let img = rondata::mountains::template_image(root, &files[k])?.0;
                let (mut x0, mut y0, mut x1, mut y1) = (img.width, img.height, 0, 0);
                for y in 0..img.height {
                    for x in 0..img.width {
                        if img.alpha(x, y) != 0 {
                            (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
                        }
                    }
                }
                Some(MountainTemplateArt {
                    tiles: shapes.get(k)?.tiles.clone(),
                    main: load(main).ok()?,
                    ring: load(ring).ok()?,
                    lit: [x0 as f32, y0 as f32, x1 as f32, y1 as f32],
                })
            })
            .collect();
        Ok(MountainArt {
            plain: load(&base)?,
            blend: load(&alpha)?,
            templates,
        })
    }

    /// Each placed range's template and cell: the template whose tiles,
    /// laid at `4 · loc`, are the range's in the template's own order.
    pub fn ranges(&self, t: &sim::observe::TerrainScene) -> Vec<Option<(usize, (i32, i32))>> {
        use render::terrain::TILES_PER_CELL;
        t.mountains
            .iter()
            .map(|r| {
                self.templates.iter().enumerate().find_map(|(k, tp)| {
                    let tp = tp.as_ref()?;
                    let (ox, oy) = *tp.tiles.first()?;
                    let (lx, ly) = (r.first()?.x - ox, r.first()?.y - oy);
                    (lx % TILES_PER_CELL == 0
                        && ly % TILES_PER_CELL == 0
                        && tp.tiles.len() == r.len()
                        && tp
                            .tiles
                            .iter()
                            .zip(r)
                            .all(|(o, q)| (lx + o.0, ly + o.1) == (q.x, q.y)))
                    .then_some((k, (lx / TILES_PER_CELL, ly / TILES_PER_CELL)))
                })
            })
            .collect()
    }

    /// The pass's two layers for `template`, in `render_mountains`'s
    /// order: the ring (`ALPHATEX` under `RING_ALPHA_TEX`), then the main
    /// (`BASETEX` under `MAIN_ALPHA_TEX`).
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn layers(&self, template: usize) -> Option<[(&Mips, &Mips); 2]> {
        let tp = self.templates.get(template)?.as_ref()?;
        Some([(&self.blend, &tp.ring), (&self.plain, &tp.main)])
    }

    /// The alpha's texcoord at template pixel `px`: the lit box mapped
    /// onto [0, 1].
    pub fn alpha_uv(&self, template: usize, px: [f32; 2]) -> [f32; 2] {
        let Some(Some(tp)) = self.templates.get(template) else {
            return [0.0, 0.0];
        };
        let b = tp.lit;
        [
            (px[0] - b[0]) / (b[2] - b[0]),
            (px[1] - b[1]) / (b[3] - b[1]),
        ]
    }

    /// **The pass on the CPU** at template pixel `px` over `under` (0 to
    /// 255): per layer, stage 0's texel at `px / 16` (one repeat a cell,
    /// wrapped) and stage 1's at [`MountainArt::alpha_uv`] (clamped,
    /// linear); `TOP_ADDSIGNED`, so the colour is `t0 + t1 − ½`, and A is
    /// `t0.a · t1.a`; a layer whose A is 0 fails the alpha test
    /// (`CMP_GREATER`, ref 0), and one that passes is laid `SRCALPHA`
    /// over `INVSRCALPHA`. The painter's; the client's is `gpu`'s
    /// `fs_mountain`.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn over(&self, template: usize, px: [f32; 2], under: [f32; 3]) -> [f32; 3] {
        let Some(layers) = self.layers(template) else {
            return under;
        };
        let [au, av] = self.alpha_uv(template, px);
        let mut f = under;
        for (t0m, t1m) in layers {
            let t0 = t0m.sample(0, px[0] / 16.0, px[1] / 16.0);
            let im = &t1m.levels[0];
            let (hu, hv) = (0.5 / im.width as f32, 0.5 / im.height as f32);
            let t1 = t1m.sample(0, au.clamp(hu, 1.0 - hu), av.clamp(hv, 1.0 - hv));
            let a = t0[3] * t1[3] / (255.0 * 255.0);
            if a <= 0.0 {
                continue;
            }
            f = [0, 1, 2].map(|c| (t0[c] + t1[c] - 127.5).clamp(0.0, 255.0) * a + f[c] * (1.0 - a));
        }
        f
    }
}

/// **A mountain's triangles** (item 4141; §3.10): every tile of a placed
/// range whose template reads, its two triangles as [`ground_base`]
/// gives them, each corner in its range's template pixels (`4 · (tile −
/// 4 · loc) + 128`, the vertex `MountainRange::init` lays there), and the
/// template. A tile two ranges share is the first range's.
#[derive(Clone, Debug, Default)]
pub struct GroundMountain {
    pub tris: Vec<[[f32; 3]; 3]>,
    pub uvs: Vec<[[f32; 2]; 3]>,
    pub template: Vec<usize>,
    pub tile: Vec<(i32, i32)>,
}

pub fn ground_mountain(
    t: &sim::observe::TerrainScene,
    art: &MountainArt,
    mesh: &render::terrain::TerrainMesh,
) -> GroundMountain {
    use render::terrain::TILES_PER_CELL;
    let mut out = GroundMountain::default();
    let (tw, th) = (t.width * TILES_PER_CELL, t.height * TILES_PER_CELL);
    if mesh.positions.len() != (tw * th * 6) as usize {
        return out;
    }
    let mut seen = std::collections::BTreeSet::new();
    for (r, placed) in t.mountains.iter().zip(art.ranges(t)) {
        let Some((k, (lx, ly))) = placed else {
            continue;
        };
        for q in r {
            if !(0..tw).contains(&q.x) || !(0..th).contains(&q.y) || !seen.insert((q.x, q.y)) {
                continue;
            }
            let (x, y) = (
                4.0 * (q.x - TILES_PER_CELL * lx) as f32 + 128.0,
                4.0 * (q.y - TILES_PER_CELL * ly) as f32 + 128.0,
            );
            let (a, b, c, d) = ([x, y], [x + 4.0, y], [x + 4.0, y + 4.0], [x, y + 4.0]);
            let i = ((q.y * tw + q.x) * 6) as usize;
            for (j, uv) in [[a, b, d], [b, c, d]].into_iter().enumerate() {
                let v = &mesh.positions[i + j * 3..i + j * 3 + 3];
                out.tris.push([v[0], v[1], v[2]]);
                out.uvs.push(uv);
                out.template.push(k);
                out.tile.push((q.x, q.y));
            }
        }
    }
    out
}

#[cfg(test)]
pub(crate) mod tests {
    /// Tiles, as the tests list them gained and lost.
    type Tiles = Vec<(i32, i32)>;
    use super::*;
    use crate::silhouette::{INTERFACE, PHOTO_FRAME, TRACED, frame_scene, photo_path};
    use crate::world::{TreePixels, World};
    use render::silhouette::{HUE, Object, chromaticity, inside, rasterize_cut};
    use render::terrain::{BASELAND, TILES_PER_CELL, land_colour};
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

    /// **Item 4164: `get_water_fade`, as read and as the packet sets it**:
    /// solid at and below −55, nothing above −25, a line between that does
    /// not meet 245 at −55; and the ramp hands an alpha back as itself.
    /// **The water and the floor at the card** (item 4168;
    /// `docs/RENDERER.md` §3.12): run1567's card trace (`tools/trace/
    /// card.h`, run1564's take with the probe in place of the packet)
    /// holds the pass drawn after frame 3279. Its one draw with
    /// `SCREEN_REFLECTIONS` is the water, `TerrainOut::render_water`'s
    /// "Normal Water"; its draw of `ocean_floor_5.tga` the floor. Each
    /// field is asserted against what §3.12 reads and what the lit branch
    /// takes: the `PerMaterial` rows are `gAmbient`, `gDiffuse`,
    /// `gSpecular`, `gEmissive`, `gTexFactor`, then `gPower`
    /// (`sbl/d3dgl.pdb`), the light's `Ambient` and `Diffuse` its second
    /// and third rows (`ShaderLight`). `RON_CARD_TRACE_1567` moves it.
    /// **The floor and the water off run1564's packet** (item 4174;
    /// `docs/RENDERER.md` §3.12): `tools/render/water_packet.py`'s dump of
    /// the meshes the original drew on frame 3294 — `scene_ocean_floor`
    /// (the card's draw 36) and `modify_water`'s statics (draw 162). Every
    /// floor vertex at [`FLOOR_Z`], its texcoords [`floor_uvs`] plus one
    /// offset a layer, its grey [`floor_grey`] of our [`ScreenRamp`] at
    /// run1564's camera; every water vertex's normal (0, 0, 1) and its rgb
    /// [`water_rgb`] of the same ramp; each within a byte.
    #[test]
    fn the_floor_and_the_water_off_the_packet() {
        let path = std::env::var_os("RON_WATER_PACKET_1564")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let home = std::env::var_os("HOME").unwrap_or_default();
                PathBuf::from(home)
                    .join("ron-data/lab-experiments/2026-11-12-item-4174/run1564-water.txt")
            });
        let Ok(text) = std::fs::read_to_string(&path) else {
            println!("skipping: no water packet dump at {}", path.display());
            return;
        };
        let ramp = ScreenRamp::of(&run1564().camera, (1920, 1080));
        let num = |s: &str| s.parse::<f32>().expect("a float");
        let hex = |s: &str| u32::from_str_radix(s, 16).expect("a colour");
        let (mut floors, mut waters, mut grey_off, mut rgb_off) = (0, 0, 0u32, 0u32);
        let mut offsets: Vec<[f32; 4]> = Vec::new();
        let mut worst = (0i32, String::new());
        for line in text.lines() {
            let f: Vec<&str> = line.split_whitespace().collect();
            match f.first() {
                Some(&"ocean") => {
                    assert!(line.contains("floor_top 255,255,255 floor_bot 64,64,64"));
                    assert!(line.contains("top 0,0,159,111,255,218"), "{line}");
                }
                Some(&"floor") => {
                    floors += 1;
                    let (x, y, z) = (num(f[1]), num(f[2]), num(f[3]));
                    assert_eq!(z, FLOOR_Z, "a floor vertex at {x}, {y}");
                    let c = hex(f[4]);
                    let (a, b) = floor_uvs(x, y);
                    offsets.push([
                        num(f[5]) - a[0],
                        num(f[6]) - a[1],
                        num(f[7]) - b[0],
                        num(f[8]) - b[1],
                    ]);
                    let ours = i32::from(floor_grey(ramp.at(x, y)));
                    let theirs = (c & 0xff) as i32;
                    assert!(c >> 24 == 0x99 && (c >> 8) & 0xff == c & 0xff, "{c:08x}");
                    let d = (ours - theirs).abs();
                    if d > 1 {
                        grey_off += 1;
                    }
                    if d > worst.0 {
                        worst = (d, format!("floor ({x}, {y}): ours {ours}, theirs {theirs}"));
                    }
                }
                Some(&"water") => {
                    waters += 1;
                    let (x, y) = (num(f[1]), num(f[2]));
                    let c = hex(f[4]);
                    let n = [num(f[5]), num(f[6]), num(f[7])];
                    assert_eq!(n, [0.0, 0.0, 1.0], "the water's normal at {x}, {y}");
                    let ours = water_rgb(ramp.at(x, y));
                    let theirs = [c & 0xff, (c >> 8) & 0xff, (c >> 16) & 0xff];
                    let d = (0..3)
                        .map(|k| (i32::from(ours[k]) - theirs[k] as i32).abs())
                        .max()
                        .unwrap();
                    if d > 1 {
                        rgb_off += 1;
                    }
                    if d > worst.0 {
                        worst = (
                            d,
                            format!("water ({x}, {y}): ours {ours:?}, theirs {theirs:?}"),
                        );
                    }
                }
                _ => {}
            }
        }
        let spread = |k: usize| {
            let v: Vec<f32> = offsets.iter().map(|o| o[k]).collect();
            let (lo, hi) = v
                .iter()
                .fold((f32::MAX, f32::MIN), |(l, h), &x| (l.min(x), h.max(x)));
            hi - lo
        };
        println!(
            "run1564 packet: {floors} floor and {waters} water vertices; our ramp d0 {:.1} len \
             {:.1}; greys off by more than 1: {grey_off}, water rgb: {rgb_off}; worst {} ({}); \
             texcoord offsets spread {:.2e} {:.2e} {:.2e} {:.2e}",
            ramp.d0,
            ramp.len,
            worst.0,
            worst.1,
            spread(0),
            spread(1),
            spread(2),
            spread(3)
        );
        assert_eq!((floors, waters), (624, 1755));
        assert!(
            (0..4).all(|k| spread(k) < 1e-3),
            "one texcoord offset a layer"
        );
        assert_eq!((grey_off, rgb_off), (0, 0), "{}", worst.1);
    }

    #[test]
    fn the_water_and_the_floor_at_the_card() {
        use render::silhouette::SCENE_AMBIENT;
        use rondata::trace::card::{
            self, TA_CURRENT, TA_DIFFUSE, TA_TEXTURE, TOP_MODULATE, TOP_MODULATE2X, TOP_SELECTARG2,
        };
        let path = std::env::var_os("RON_CARD_TRACE_1567")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let home = std::env::var_os("HOME").unwrap_or_default();
                PathBuf::from(home).join("ron-golden/run1567/map-14/rontrace.log")
            });
        let Ok(bytes) = std::fs::read(&path) else {
            println!("skipping: no card trace at {}", path.display());
            return;
        };
        let pass: Vec<card::CardDraw> = card::draws(&bytes)
            .into_iter()
            .filter(|d| d.frame == 3279)
            .collect();
        let water: Vec<&card::CardDraw> = pass
            .iter()
            .filter(|d| d.has("SCREEN_REFLECTIONS"))
            .collect();
        assert_eq!(water.len(), 1, "one water draw on the pass");
        let w = water[0];
        let floor: Vec<&card::CardDraw> = pass
            .iter()
            .filter(|d| d.file(0) == "ocean_floor_5.tga")
            .collect();
        assert_eq!(floor.len(), 1, "one floor draw on the pass");
        let f = floor[0];
        let near = |a: [f32; 4], b: [f32; 4]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-4);
        let blends = |d: &card::CardDraw| d.enables() & 4 != 0;
        const COMPLEMENT: i32 = i32::MIN; // TA_COMPLEMENT, 0x80000000
        let ambient = [SCENE_AMBIENT[0], SCENE_AMBIENT[1], SCENE_AMBIENT[2], 0.558];

        // The water: the vertex's white and its fade, nothing of the texture.
        for o in [
            "ENABLE_LIGHTING",
            "SPEC_MODEL_TORRANCE",
            "ENABLE_VERTEX_COLOR_DIFFUSE",
        ] {
            assert!(w.has(o), "the water draws with {o}");
        }
        for o in [
            "ANIMATE_WATER_NORMALS",
            "NORMAL_MAP",
            "NO_HDR",
            "ENABLE_FOG",
        ] {
            assert!(!w.has(o), "the water draws without {o}");
        }
        assert_eq!(w.colour_op(0), (TOP_SELECTARG2, TA_TEXTURE, TA_DIFFUSE));
        assert_eq!(w.alpha_op(0), (TOP_SELECTARG2, TA_TEXTURE, TA_DIFFUSE));
        assert_eq!(w.colour_op(1), (TOP_SELECTARG2, TA_DIFFUSE, TA_CURRENT));
        assert_eq!(
            w.alpha_op(1),
            (TOP_SELECTARG2, TA_TEXTURE | COMPLEMENT, TA_CURRENT),
            "stage 1's alpha selects the current, not the fog fade's complement"
        );
        assert_eq!(w.file(1), "fow_fade.tga");
        assert!(
            blends(w) && (w.src_blend(), w.dst_blend()) == (5, 6),
            "source alpha over"
        );
        let m = w.material();
        assert!(
            near(m[0], [0.2, 0.2, 0.2, 1.0]),
            "the water's ambient {:?}",
            m[0]
        );
        assert!(
            near(m[1], [1.0, 1.0, 1.5, 1.0]),
            "the water's diffuse {:?}",
            m[1]
        );
        assert!(
            near(m[2], [40.0, 40.0, 40.0, 1.0]),
            "the water's specular {:?}",
            m[2]
        );
        assert!(near(m[3], [0.0, 0.0, 0.0, 1.0]), "no emissive");
        let power = f32::from_le_bytes(w.cbuf[0x1b0..0x1b4].try_into().unwrap());
        assert!((power - 0.3).abs() < 1e-6, "the roughness, gPower {power}");
        let l = w.light();
        assert!(near(
            l[0],
            [
                -std::f32::consts::FRAC_1_SQRT_2,
                0.0,
                -std::f32::consts::FRAC_1_SQRT_2,
                0.0
            ]
        ));
        assert!(
            near(l[2], [1.0; 4]),
            "the water's light is white {:?}",
            l[2]
        );
        assert!(near(w.scene_ambient(), ambient), "the scene's ambient");

        // The floor: lit, opaque, its texture twice and doubled.
        assert!(f.has("ENABLE_LIGHTING") && f.has("ENABLE_VERTEX_COLOR_DIFFUSE"));
        assert!(!f.has("NORMAL_MAP"), "the floor has no normal map");
        assert_eq!(f.colour_op(0), (TOP_MODULATE2X, TA_TEXTURE, TA_DIFFUSE));
        assert_eq!(f.colour_op(1), (TOP_MODULATE, TA_TEXTURE, TA_CURRENT));
        assert_eq!(f.file(1), "ocean_floor_5.tga");
        assert!(!blends(f), "the floor is opaque");
        let m = f.material();
        assert!(
            near(m[0], [1.0; 4]) && near(m[1], [1.0; 4]),
            "the floor's material is white"
        );
        assert!(near(m[2], [0.0, 0.0, 0.0, 1.0]), "no specular");
        let l = f.light();
        assert!(near(l[0], w.light()[0]), "one light direction");
        assert!(
            near(l[1], [0.02, 0.06, 0.36, 2.0]),
            "the light's ambient {:?}",
            l[1]
        );
        assert!(
            near(l[2], [1.36; 4]),
            "the ground's light diffuse {:?}",
            l[2]
        );
        assert!(near(f.scene_ambient(), ambient), "the scene's ambient");
        println!(
            "run1567, the pass after 3279: {} draws; the water draw {}, the floor draw {}",
            pass.len(),
            w.index,
            f.index
        );
    }

    /// **The lake's layers at the card** (item 4213; `docs/RENDERER.md`
    /// §3.18): run1567's pass after 3279, the setters of every draw that
    /// lays the lake between the floor (36) and the land's last decal, and
    /// of the surf after the water. Draws 121–130 pair `sand.tga` and
    /// `coralblend.tga` under one mask each, `MODULATE(TEXTURE, DIFFUSE)`
    /// then the mask's alpha alone, lit at the land's light; 131 lays the
    /// coral over the sand layer at the complement of its vertex alpha,
    /// stage 1 off; 163–169 are the surf, `TL_FAKE_LIGHT` at `gTLAlpha`
    /// 1, the texture's alpha times the vertex's times the mask's.
    #[test]
    fn the_lake_s_layers_at_the_card() {
        use rondata::trace::card::{
            self, TA_CURRENT, TA_DIFFUSE, TA_TEXTURE, TOP_MODULATE, TOP_SELECTARG2,
        };
        const TOP_DISABLE: i32 = 0;
        const TOP_SELECTARG1: i32 = 1;
        const COMPLEMENT: i32 = i32::MIN;
        let path = std::env::var_os("RON_CARD_TRACE_1567")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let home = std::env::var_os("HOME").unwrap_or_default();
                PathBuf::from(home).join("ron-golden/run1567/map-14/rontrace.log")
            });
        let Ok(bytes) = std::fs::read(&path) else {
            println!("skipping: no card trace at {}", path.display());
            return;
        };
        let pass: Vec<card::CardDraw> = card::draws(&bytes)
            .into_iter()
            .filter(|d| d.frame == 3279)
            .collect();
        let at = |i: u32| pass.iter().find(|d| d.index == i).expect("the draw");
        let land_light = |d: &card::CardDraw| {
            let (m, l) = (d.material(), d.light());
            d.has("ENABLE_LIGHTING")
                && d.has("ENABLE_VERTEX_COLOR_DIFFUSE")
                && m[0] == [1.0; 4]
                && m[1] == [1.0; 4]
                && l[2] == [1.36; 4]
        };
        let tl_alpha =
            |d: &card::CardDraw| f32::from_le_bytes(d.cbuf[0x1bc..0x1c0].try_into().unwrap());
        for i in 121..=130 {
            let d = at(i);
            let sand = i % 2 == 1;
            assert_eq!(
                d.file(0),
                if sand { "sand.tga" } else { "coralblend.tga" },
                "draw {i}"
            );
            assert_eq!(
                d.names[1],
                at(i - u32::from(!sand)).names[1],
                "draw {i}: one mask a pair"
            );
            assert_eq!(
                d.colour_op(0),
                (TOP_MODULATE, TA_TEXTURE, TA_DIFFUSE),
                "draw {i}"
            );
            assert_eq!(
                d.colour_op(1),
                (TOP_SELECTARG2, TA_TEXTURE, TA_CURRENT),
                "draw {i}"
            );
            assert_eq!(
                d.alpha_op(0),
                (TOP_SELECTARG1, TA_TEXTURE, TA_DIFFUSE),
                "draw {i}"
            );
            assert_eq!(
                d.alpha_op(1),
                (TOP_SELECTARG1, TA_TEXTURE, TA_CURRENT),
                "draw {i}: the mask alone"
            );
            assert!(land_light(d), "draw {i} at the land's light");
            assert_eq!(
                d.has("NORMAL_MAP"),
                sand,
                "draw {i}: the sand's material map, not the coral's"
            );
        }
        let c = at(131);
        assert_eq!((c.file(0), c.file(1)), ("coralblend.tga", "coralblend.tga"));
        assert_eq!(c.colour_op(0), (TOP_MODULATE, TA_TEXTURE, TA_DIFFUSE));
        assert_eq!(
            c.alpha_op(0),
            (TOP_SELECTARG2, TA_TEXTURE, TA_DIFFUSE | COMPLEMENT),
            "the sand's fade, complemented"
        );
        assert_eq!(
            (c.colour_op(1).0, c.alpha_op(1).0),
            (TOP_DISABLE, TOP_DISABLE)
        );
        assert!(land_light(c), "the coral layer at the land's light");
        for i in 163..=169 {
            let d = at(i);
            assert!(d.file(0).starts_with("surf_"), "draw {i}: {}", d.file(0));
            assert!(
                d.has("TL_FAKE_LIGHT") && !d.has("ENABLE_LIGHTING"),
                "draw {i}"
            );
            assert_eq!(
                d.colour_op(0),
                (TOP_MODULATE, TA_TEXTURE, TA_CURRENT),
                "draw {i}"
            );
            assert_eq!(
                d.alpha_op(0),
                (TOP_MODULATE, TA_TEXTURE, TA_CURRENT),
                "draw {i}"
            );
            assert_eq!(
                d.alpha_op(1),
                (TOP_MODULATE, TA_TEXTURE, TA_CURRENT),
                "draw {i}"
            );
            assert_eq!(tl_alpha(d), 1.0, "draw {i}'s gTLAlpha");
        }
        println!(
            "run1567, the pass after 3279: the lake's patches 121–130, the coral layer 131 and the surf 163–169 as read"
        );
    }

    /// **The water's lattice off the packet** (item 4178): run1564's
    /// drawn water (`modify_water`'s statics, `tools/render/water_packet.py`)
    /// against [`water_grid`] on our own terrain. Every drawn corner is a
    /// lattice point we lay; its alpha is ours within one; every centre is
    /// its quad's four corners' mean; every vertex stands at
    /// `max(0, ground)`. Each other reading of `generate_water` is counted
    /// beside it and must agree on fewer.
    #[test]
    fn the_water_s_lattice_off_the_packet() {
        let path = std::env::var_os("RON_WATER_PACKET_1564")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let home = std::env::var_os("HOME").unwrap_or_default();
                PathBuf::from(home)
                    .join("ron-data/lab-experiments/2026-11-12-item-4174/run1564-water.txt")
            });
        let Ok(text) = std::fs::read_to_string(&path) else {
            println!("skipping: no water packet dump at {}", path.display());
            return;
        };
        let Some(p) = probe_at(run1564()) else { return };
        let t = p.world.runner.terrain();
        let num = |s: &str| s.parse::<f32>().expect("a float");
        let mut corners: BTreeMap<(i32, i32), u8> = BTreeMap::new();
        let mut centres: Vec<((i32, i32), u8)> = Vec::new();
        let (mut lines, mut off_zero) = (0, 0);
        // Item 4213: what the lit water draw (162) takes from each vertex.
        let (mut up, mut no_red, mut most) = (0usize, 0usize, 0u8);
        let mut above: std::collections::BTreeSet<(i32, i32)> = Default::default();
        let sx = t.width * render::terrain::TILES_PER_CELL + 1;
        for line in text.lines() {
            let f: Vec<&str> = line.split_whitespace().collect();
            if f.first() != Some(&"water") {
                continue;
            }
            lines += 1;
            let (x, y, z) = (num(f[1]), num(f[2]), num(f[3]));
            let colour = u32::from_str_radix(f[4], 16).expect("a colour");
            let a = (colour >> 24) as u8;
            off_zero += usize::from(z != 0.0);
            up += usize::from((num(f[5]), num(f[6]), num(f[7])) == (0.0, 0.0, 1.0));
            no_red += usize::from(colour & 0xff == 0);
            most = most.max(a);
            let (gx, gy) = ((x / WATER_STEP) as i32, (y / WATER_STEP) as i32);
            if x % WATER_STEP == 0.0 && y % WATER_STEP == 0.0 {
                corners.insert((gx, gy), a);
                let k = (2 * gy * sx + 2 * gx) as usize;
                if f32::from_bits(t.corners[k]) > 0.0 {
                    above.insert((gx, gy));
                }
            } else {
                centres.push(((gx, gy), a));
            }
        }
        let reads = [
            WATER_READ,
            WaterRead {
                north: false,
                ..WATER_READ
            },
            WaterRead {
                in_place: false,
                ..WATER_READ
            },
            WaterRead {
                solid: 220,
                in_place: false,
                ..WATER_READ
            },
            WaterRead {
                solid: 220,
                ..WATER_READ
            },
        ];
        let mut counts = Vec::new();
        for (r, read) in reads.iter().enumerate() {
            let g = water_grid_with(&t, *read);
            let (mut agree, mut unlaid, mut worst) = (0, 0, (0, (0, 0), 0, 0));
            for (&(gx, gy), &theirs) in &corners {
                let Some(ours) = g.at(gx, gy) else {
                    unlaid += 1;
                    continue;
                };
                let d = (i32::from(ours) - i32::from(theirs)).abs();
                if d <= 1 {
                    agree += 1;
                }
                if d > worst.0 {
                    worst = (d, (gx, gy), theirs, ours);
                }
            }
            let mut centre_ok = 0;
            for &((gx, gy), theirs) in &centres {
                let q = [(0, 0), (1, 0), (0, 1), (1, 1)]
                    .map(|(dx, dy)| corners.get(&(gx + dx, gy + dy)).copied());
                if q.iter().all(Option::is_some) {
                    let mean = q.iter().map(|a| u32::from(a.unwrap())).sum::<u32>() / 4;
                    centre_ok += u32::from((mean as i32 - i32::from(theirs)).abs() <= 1);
                }
            }
            println!(
                "run1564 water lattice, read {r} {read:?}: corners {agree} of {} within one, {unlaid} unlaid, worst {worst:?}; centres their corners' mean {centre_ok} of {}",
                corners.len(),
                centres.len()
            );
            counts.push((agree, unlaid, centre_ok));
        }
        println!(
            "RUN1564_WATER_LATTICE: {counts:?}; {lines} drawn vertices, {off_zero} off z 0, {} corners over ground above 0",
            above.len()
        );
        assert_eq!(
            (lines, off_zero),
            (1755, 0),
            "every drawn water vertex at z 0"
        );
        // Item 4213 (§3.18): the lit water's own inputs carry no red and
        // no tilt — every normal straight up, every red byte 0 — and its
        // deepest alpha, 189, lets 26 % of what lies beneath through.
        assert_eq!(
            (up, no_red, most),
            (1755, 1755, 189),
            "the water's normals, red bytes and deepest alpha"
        );
        assert_eq!(
            (corners.len(), centres.len(), above.len()),
            RUN1564_WATER_LATTICE.0,
            "the drawn lattice"
        );
        assert_eq!(counts, RUN1564_WATER_LATTICE.1);
        assert!(
            counts[1..].iter().all(|c| c.0 < counts[0].0),
            "every other reading agrees on fewer"
        );
    }

    /// **The land's depth fade off the packet** (item 4192; §3.16): run1564's `master_land` corners and its land pass's
    /// one filled helper, the sand layer (`tools/render/sand_packet.py`),
    /// against [`land_alpha`] on our terrain ([`RUN1564_LAND_FADE`]).
    #[test]
    fn the_land_fade_off_the_packet() {
        let path = std::env::var_os("RON_SAND_PACKET_1564")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let home = std::env::var_os("HOME").unwrap_or_default();
                PathBuf::from(home)
                    .join("ron-data/lab-experiments/2026-11-12-item-4192/run1564-sand.txt")
            });
        let Ok(text) = std::fs::read_to_string(&path) else {
            println!("skipping: no sand packet dump at {}", path.display());
            return;
        };
        let Some(p) = probe_at(run1564()) else { return };
        let t = p.world.runner.terrain();
        let num = |s: &str| s.parse::<f32>().expect("a float");
        let sx = t.width * render::terrain::TILES_PER_CELL + 1;
        let ours = |ix: i32, iy: i32| f32::from_bits(t.corners[(iy * sx + ix) as usize]);
        let mut got = [0usize; 7];
        let (mut zero, mut zero_sand) = (0, 0);
        let mut worst: Option<(i32, i32, f32, f32)> = None;
        for line in text.lines() {
            let f: Vec<&str> = line.split_whitespace().collect();
            match f.first().copied() {
                Some("land") => {
                    assert_eq!(
                        f[f.iter().position(|w| *w == "side").unwrap() + 1],
                        format!("{sx}")
                    );
                    let at = |k: &str| num(f[f.iter().position(|w| *w == k).unwrap() + 1]);
                    assert_eq!(
                        (at("coast_depth"), at("land_height")),
                        (COAST_DEPTH, LAND_HEIGHT)
                    );
                }
                Some("corner") => {
                    let (ix, iy): (i32, i32) = (f[1].parse().unwrap(), f[2].parse().unwrap());
                    let (h, a) = (num(f[3]), f[4].parse::<u8>().unwrap());
                    let us = ours(ix, iy);
                    got[0] += 1;
                    if (us - h).abs() <= 0.01 {
                        got[1] += 1;
                    } else if worst.is_none_or(|w| (w.3 - w.2).abs() < (us - h).abs()) {
                        worst = Some((ix, iy, h, us));
                    }
                    got[2] += usize::from(land_alpha(h) == a);
                    got[3] += usize::from(land_alpha(us) == a);
                    zero += usize::from(land_alpha_of(h, 0.0) == a);
                }
                Some("sand") => {
                    let (x, y) = (num(f[2]), num(f[3]));
                    let a = (u32::from_str_radix(f[5], 16).unwrap() >> 24) as u8;
                    let (ix, iy) = ((x / 192.0) as i32, (y / 192.0) as i32);
                    got[4] += 1;
                    // A vertex on a cell's edge touches the cells either
                    // side of it: one of them is the cell it was laid for.
                    let cells = |v: f32| [((v - 1.0) / 768.0) as i32, (v / 768.0) as i32];
                    let near = cells(x).iter().any(|&cx| {
                        cells(y).iter().any(|&cy| {
                            t.land[(cy * t.width + cx) as usize] == render::terrain::SANDY
                        })
                    });
                    got[5] += usize::from(near);
                    got[6] += usize::from(land_alpha(ours(ix, iy)) == a);
                    zero_sand += usize::from(land_alpha_of(ours(ix, iy), 0.0) == a);
                }
                _ => {}
            }
        }
        println!(
            "RUN1564_LAND_FADE: {got:?}; at land_height 0, {zero} corners and {zero_sand} sand vertices; the worst height {worst:?}"
        );
        assert_eq!(got, RUN1564_LAND_FADE);
        assert_eq!(
            (zero, zero_sand),
            (47874, ZERO_SAND),
            "made to fail: land_height 0"
        );
    }

    #[test]
    fn the_water_fades_as_get_water_fade_fades() {
        assert_eq!(water_fade(-200.0), WATER_SOLID);
        assert_eq!(water_fade(-55.0), 245);
        assert_eq!(water_fade(-54.9), 133);
        assert_eq!(water_fade(-40.0), 66);
        assert_eq!(water_fade(-25.0), 0);
        assert_eq!(water_fade(10.0), 0);
        let ramp = water_ramp();
        for a in [0u8, 66, 133, 245] {
            let u = (f32::from(a) + 0.5) / 256.0;
            let got = ramp.sample(0, u, 0.5)[3];
            assert!((got - f32::from(a)).abs() < 0.01, "{a}: {got}");
        }
    }

    /// The probe's state at the photograph's frame.
    pub(crate) struct Probe {
        pub world: World,
        pub photo: render::tga::Image,
        pub camera: render::Camera,
        pub root: PathBuf,
        /// The photograph's traced polygons, masked out of the ground.
        pub traced: Traced,
    }

    /// A photograph's traced polygons, as [`TRACED`] holds run1559b's.
    pub(crate) type Traced = &'static [(&'static str, &'static [(i32, i32)])];

    /// **A photograph of the original and what it was taken at** (item
    /// 4160): its file, the camera its packet holds, the frame our walk is
    /// drawn at, and its traces.
    pub(crate) struct Photo {
        pub path: PathBuf,
        pub camera: render::Camera,
        pub frame: i64,
        pub traced: Traced,
    }

    /// run1559b's photograph at the client's pinned camera, frame 3279.
    pub(crate) fn run1559b() -> Photo {
        Photo {
            path: photo_path(),
            camera: crate::pinned_camera(),
            frame: PHOTO_FRAME,
            traced: &TRACED,
        }
    }

    /// **run1564's photograph** (item 4160; `docs/RUNS.md` run1564):
    /// run1559b's take with `camera 192,200` staged on frame 3200, the
    /// title bar's 32 rows cropped. Its packet at logger frame 3294 holds
    /// the pinned camera with `loc` at (36960, 38496), the centre of tile
    /// (192, 200), and nothing else changed; it is drawn at
    /// [`PHOTO_FRAME`], as run1559b's photograph is (assumed, not
    /// clocked: no mover is in view). No polygon is traced on it.
    pub(crate) fn run1564() -> Photo {
        let path = std::env::var_os("RON_PHOTO_RUN1564")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let home = std::env::var_os("HOME").unwrap_or_default();
                PathBuf::from(home)
                    .join("ron-data/lab-experiments/2026-11-12-item-4160/run1564-3294.tga")
            });
        let mut camera = crate::pinned_camera();
        camera.loc_x = f32::from_bits(RUN1564_LOC.0);
        camera.loc_y = f32::from_bits(RUN1564_LOC.1);
        Photo {
            path,
            camera,
            frame: PHOTO_FRAME,
            traced: &[],
        }
    }

    pub(crate) fn probe() -> Option<Probe> {
        probe_at(run1559b())
    }

    pub(crate) fn probe_at(shot: Photo) -> Option<Probe> {
        let root = std::env::var_os("RON_INSTALL")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("../../game"));
        let path = shot.path;
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
        world.run_to(shot.frame);
        world.blend = crate::world::Blend::Last;
        world.run_to(shot.frame + 1);
        Some(Probe {
            world,
            photo,
            camera: shot.camera,
            root,
            traced: shot.traced,
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
        let tex = GroundTextures::untinted(&p.root, &tileset).expect("the ground's textures load");
        // Item 4141: with a mountain's own art the ground is laid beneath
        // every mountain tile, and each range's tiles twice over it.
        let art = MountainArt::untinted(&p.root, &tileset).expect("the mountains' art loads");
        let range = ground_mountain(&t, &art, &full);
        assert_eq!(
            range.tile.len(),
            2 * mountains.len(),
            "every mountain tile, once"
        );
        let none = std::collections::BTreeSet::new();
        let base = ground_base(&t, &none, &tex, &full);
        let (rest, ground, laid) = p.world.drawn_terrain();
        let on_mountain: usize = laid
            .iter()
            .filter(|m| m.mountain)
            .map(|m| m.positions.len())
            .sum();
        assert_eq!(
            on_mountain,
            2 * range.tris.len() * 3,
            "the ring and the main"
        );
        let first = laid
            .iter()
            .position(|m| m.mountain)
            .expect("a mountain is drawn");
        assert!(
            laid[first..].iter().all(|m| m.mountain),
            "the mountains after the patch"
        );
        // Item 4164: the water before the mountains. Item 4178: on its own
        // mesh, sixteen triangles over every cell that holds it, and the
        // floor's shade over every `OCEAN` tile, once.
        let water: Vec<_> = laid
            .iter()
            .filter(|m| m.piece == crate::world::WATER_PIECE)
            .collect();
        let wet = (0..t.height)
            .flat_map(|cy| (0..t.width).map(move |cx| (cx, cy)))
            .filter(|&(cx, cy)| holds_water(&t, cx, cy))
            .count();
        assert_eq!(water.len(), 1, "one water mesh");
        assert!(wet > 0, "the map holds water");
        assert_eq!(
            water[0].positions.len(),
            wet * 16 * 3,
            "every wet cell, once"
        );
        assert!(
            water[0].positions.iter().all(|p| p[2] == 0.0),
            "the water at z 0"
        );
        let ocean = tex.index_land(render::terrain::OCEAN, 0, 0, 0);
        let sea = base.of.iter().filter(|&&k| Some(k) == ocean).count();
        let shade: Vec<_> = laid
            .iter()
            .filter(|m| m.piece == crate::world::FLOOR_PIECE - 1)
            .collect();
        assert_eq!(shade.len(), 1, "one floor shade");
        assert_eq!(shade[0].positions.len(), sea * 3, "every OCEAN tile, once");
        // Item 4192: and the land's faded layers, one per texture whose
        // triangles reach below the fade's top, each before the shade.
        let faded: Vec<_> = laid
            .iter()
            .filter(|m| {
                m.alpha == crate::world::WATER_ALPHA
                    && m.piece != crate::world::WATER_PIECE
                    && m.piece != crate::world::FLOOR_PIECE - 1
            })
            .collect();
        assert!(!faded.is_empty(), "the lake's floor fades");
        assert_eq!(
            laid.iter()
                .filter(|m| m.alpha == crate::world::WATER_ALPHA)
                .count(),
            2 + faded.len(),
            "the water, the shade and the faded land ride the alpha ramp"
        );
        assert!(
            laid.iter()
                .position(|m| m.piece == crate::world::FLOOR_PIECE - 1)
                == Some(faded.len()),
            "the faded land before the shade"
        );
        let mut faded_of = std::collections::BTreeMap::<usize, usize>::new();
        for m in &faded {
            assert!(m.piece < 0 && m.piece > crate::world::FLOOR_PIECE);
            assert_eq!(m.positions.len(), m.alpha_uvs.len());
            assert_eq!(m.positions.len(), m.uvs.len());
            assert!(
                m.positions
                    .chunks(3)
                    .all(|tri| tri.iter().any(|q| land_alpha(q[2]) < 255)),
                "a faded triangle reaches below the top"
            );
            *faded_of.entry((-1 - m.piece) as usize).or_default() += m.positions.len();
        }
        let fade_drawn: usize = faded_of.values().sum();
        assert!(
            laid.iter()
                .position(|m| m.alpha == crate::world::WATER_ALPHA)
                < Some(first)
        );
        let laid: usize = laid
            .iter()
            .filter(|m| !m.mountain && m.alpha != crate::world::WATER_ALPHA)
            .map(|m| m.positions.len())
            .sum();
        assert!(laid > 0, "the transition patch is laid");
        let drawn: usize = ground.iter().map(|m| m.positions.len()).sum();
        assert!(drawn > 0, "the ground is textured");
        assert_eq!(drawn + fade_drawn, base.tris.len() * 3);
        assert_eq!(
            rest.positions.len() + drawn + fade_drawn,
            full.positions.len()
        );
        for m in ground {
            assert_eq!(m.positions.len(), m.uvs.len());
            // Item 4178: the `OCEAN` tiles draw the floor, not their base.
            let k = if m.piece == crate::world::FLOOR_PIECE {
                ocean.expect("the floor is the ocean's")
            } else {
                (-1 - m.piece) as usize
            };
            let tris = base.of.iter().filter(|&&o| o == k).count();
            let fade = if m.piece == crate::world::FLOOR_PIECE {
                0
            } else {
                faded_of.get(&k).copied().unwrap_or(0)
            };
            assert_eq!(m.positions.len() + fade, tris * 3, "texture {k}");
        }
        assert!(
            ground
                .iter()
                .all(|m| ocean != Some((-1 - m.piece) as usize)),
            "no OCEAN tile in the land pass"
        );
        assert_eq!(p.world.terrain().positions, full.positions);
        p.world.debug_ground = true;
        let (all, none, unlaid) = p.world.drawn_terrain();
        assert_eq!(
            (all.positions.len(), none.len(), unlaid.len()),
            (full.positions.len(), 0, 0)
        );
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
        ground_why(p).0
    }

    /// Why a pixel of the photograph is not a ground pixel, the order
    /// [`ground_why`] tests them (item 4143, stage 0); `0` is ground.
    pub(crate) const WHY: [&str; 6] = [
        "ground",
        "an object's pixel",
        "an object's window",
        "the interface",
        "a trace",
        "off the terrain",
    ];

    /// Each pixel's tile, `None` off the heightfield.
    pub(crate) type PixelTiles = Vec<Option<(i32, i32)>>;

    /// [`ground`], with each pixel's tile before the masks and the first
    /// mask that drops it, an index of [`WHY`] (item 4143).
    pub(crate) fn ground_why(p: &mut Probe) -> (Ground, PixelTiles, Vec<u8>) {
        let size = (p.photo.width, p.photo.height);
        let (w, h) = size;
        p.world.tree_pixels = TreePixels::Scene;
        let scene = frame_scene(&mut p.world, &p.camera, size);
        let mut why = vec![0u8; w * h];
        for y in 0..h {
            for x in 0..w {
                if scene.keys.at(x, y) != 0 {
                    why[y * w + x] = 1;
                }
            }
        }
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
                    let q = &mut why[y as usize * w + x as usize];
                    if *q == 0 {
                        *q = 2;
                    }
                }
            }
        }
        for y in 0..h {
            for x in 0..w {
                let (xi, yi) = (x as i32, y as i32);
                let q = &mut why[y * w + x];
                if *q != 0 {
                    continue;
                }
                if INTERFACE
                    .iter()
                    .any(|r| (r.x0..r.x1).contains(&xi) && (r.y0..r.y1).contains(&yi))
                {
                    *q = 3;
                } else if p.traced.iter().any(|(_, poly)| inside(poly, xi, yi)) {
                    *q = 4;
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
        let any: Vec<Option<(i32, i32)>> = (0..w * h)
            .map(|i| {
                let k = tiles.keys[i];
                (k != 0).then(|| {
                    let n = k as i32 - 1;
                    (n % tw, n / tw)
                })
            })
            .collect();
        for (q, a) in why.iter_mut().zip(&any) {
            if *q == 0 && a.is_none() {
                *q = 5;
            }
        }
        let tile = (0..w * h).map(|i| any[i].filter(|_| why[i] == 0)).collect();
        (Ground { tile, tw, th }, any, why)
    }

    /// **Item 4143, stage 0: what the ground's row leaves out.** Every
    /// tile in the photograph's view, by its cell's kind — `WData.land`,
    /// the shore and rock bits of `WData.flags`, a placed mountain — and
    /// by the first mask that drops each of its pixels ([`WHY`]); a tile
    /// scores at [`HUE`]'s `min_pixels` ground pixels.
    #[test]
    fn what_the_ground_s_row_leaves_out() {
        use sim::ids::cell;
        let Some(mut p) = probe() else { return };
        let (g, any, why) = ground_why(&mut p);
        let t = p.world.runner.terrain();
        let mountains: std::collections::BTreeSet<(i32, i32)> =
            t.mountains.iter().flatten().map(|q| (q.x, q.y)).collect();
        let kind = |(tx, ty): (i32, i32)| {
            let i = ((ty / TILES_PER_CELL) * t.width + tx / TILES_PER_CELL) as usize;
            let f = t.flags[i] & (cell::COAST | cell::HALFLAND | cell::ROCK);
            (t.land[i], f, mountains.contains(&(tx, ty)))
        };
        // Per tile: its pixels by reason.
        let mut per: BTreeMap<(i32, i32), [u32; 6]> = BTreeMap::new();
        for (a, q) in any.iter().zip(&why) {
            if let Some(tt) = a {
                per.entry(*tt).or_default()[*q as usize] += 1;
            }
        }
        // Off the terrain: what the photograph shows there, and where.
        let w = p.photo.width;
        let (mut off, mut sum, mut bbox) = (0u32, [0f32; 3], (usize::MAX, usize::MAX, 0, 0));
        for (i, q) in why.iter().enumerate() {
            if *q == 5 {
                off += 1;
                for (c, v) in sum.iter_mut().enumerate() {
                    *v += f32::from(p.photo.rgba[i * 4 + c]);
                }
                let (x, y) = (i % w, i / w);
                bbox = (bbox.0.min(x), bbox.1.min(y), bbox.2.max(x), bbox.3.max(y));
            }
        }
        let m = sum.map(|v| v / off.max(1) as f32);
        println!(
            "off the terrain: {off} pixels in {bbox:?}, the photograph's mean ({:.0}, {:.0}, {:.0})",
            m[0], m[1], m[2]
        );
        // By kind: (tiles in view, scored, short of the bar, no ground
        // pixel), and the pixels by reason.
        type Row = ([u32; 4], [u32; 6]);
        let mut by: BTreeMap<(i8, u16, bool), Row> = BTreeMap::new();
        for (tt, n) in &per {
            let e = by.entry(kind(*tt)).or_default();
            e.0[0] += 1;
            match n[0] {
                0 => e.0[3] += 1,
                k if k >= HUE.min_pixels => e.0[1] += 1,
                _ => e.0[2] += 1,
            }
            for (a, b) in e.1.iter_mut().zip(n) {
                *a += b;
            }
        }
        for ((l, f, m), (tiles, px)) in &by {
            println!(
                "kind land {l} flags {f:#x} mountain {m}: tiles [in view, scored, short, none] {tiles:?}; pixels by {WHY:?} {px:?}"
            );
        }
        // The whole map's cells by kind, for the denominators.
        let mut cells: BTreeMap<(i8, u16), u32> = BTreeMap::new();
        for (l, f) in t.land.iter().zip(&t.flags) {
            *cells
                .entry((*l, f & (cell::COAST | cell::HALFLAND | cell::ROCK)))
                .or_default() += 1;
        }
        println!("the map's cells by (land, flags): {cells:?}");
        let scored = per.values().filter(|n| n[0] >= HUE.min_pixels).count();
        assert_eq!(scored, RUN1559B_3279_GROUND.1, "the row's selection");
        assert_eq!(
            scored,
            g.tile
                .iter()
                .flatten()
                .fold(BTreeMap::<_, u32>::new(), |mut m, tt| {
                    *m.entry(*tt).or_default() += 1;
                    m
                })
                .values()
                .filter(|n| **n >= HUE.min_pixels)
                .count(),
            "ground_why's tiles are ground's"
        );
        let land = [(0, false), (0, true), (cell::ROCK, false)]
            .map(|(f, m)| by.get(&(BASELAND, f, m)).map_or([0; 4], |r| r.0));
        let other: u32 = by
            .iter()
            .filter(|((l, f, _), _)| *l != BASELAND || f & !cell::ROCK != 0)
            .map(|(_, r)| r.0[0])
            .sum();
        assert_eq!((land, other, off), RUN1559B_3279_UNSCORED);
    }

    /// The kinds [`census`] counts a tile by (item 4160): its cell's
    /// `WData.land`, the `CLIFF`, `ROCK`, `COAST` and `HALFLAND` bits of its
    /// flags, and a placed mountain.
    pub(crate) const KINDS: [&str; 7] = [
        "plain",
        "under a mountain",
        "ROCK",
        "CLIFF",
        "a BASELAND shore (COAST or HALFLAND)",
        "SANDY",
        "OCEAN",
    ];

    /// One tile's index in [`KINDS`].
    pub(crate) fn kind_of(
        t: &sim::observe::TerrainScene,
        mountains: &std::collections::BTreeSet<(i32, i32)>,
        (tx, ty): (i32, i32),
    ) -> usize {
        use sim::ids::cell;
        let i = ((ty / TILES_PER_CELL) * t.width + tx / TILES_PER_CELL) as usize;
        let f = t.flags[i];
        match t.land[i] {
            render::terrain::OCEAN => 6,
            render::terrain::SANDY => 5,
            _ if mountains.contains(&(tx, ty)) => 1,
            _ if f & cell::CLIFF != 0 => 3,
            _ if f & (cell::COAST | cell::HALFLAND) != 0 => 4,
            _ if f & cell::ROCK != 0 => 2,
            _ => 0,
        }
    }

    /// **A photograph's census by kind** (item 4160): every tile in view by
    /// [`KINDS`], `[in view, scored, short of the bar, no ground pixel]`,
    /// the pixels by [`WHY`], and the pixels off the heightfield.
    pub(crate) struct Census {
        pub tiles: [[u32; 4]; 7],
        pub pixels: [u32; 6],
        pub off: u32,
        /// Per kind, the scored tiles whose ground pixels' mean on the
        /// photograph is blue (`b > r`): where the photograph shows water.
        pub blue: [u32; 7],
    }

    pub(crate) fn census(p: &mut Probe) -> Census {
        let (_, any, why) = ground_why(p);
        let t = p.world.runner.terrain();
        let mountains: std::collections::BTreeSet<(i32, i32)> =
            t.mountains.iter().flatten().map(|q| (q.x, q.y)).collect();
        let mut per: BTreeMap<(i32, i32), [u32; 6]> = BTreeMap::new();
        let mut rb: BTreeMap<(i32, i32), (u64, u64)> = BTreeMap::new();
        let mut pixels = [0u32; 6];
        for (i, (a, q)) in any.iter().zip(&why).enumerate() {
            pixels[*q as usize] += 1;
            if let Some(tt) = a {
                per.entry(*tt).or_default()[*q as usize] += 1;
                if *q == 0 {
                    let e = rb.entry(*tt).or_default();
                    e.0 += u64::from(p.photo.rgba[i * 4]);
                    e.1 += u64::from(p.photo.rgba[i * 4 + 2]);
                }
            }
        }
        let mut blue = [0u32; 7];
        let mut tiles = [[0u32; 4]; 7];
        for (tt, n) in &per {
            let e = &mut tiles[kind_of(&t, &mountains, *tt)];
            e[0] += 1;
            match n[0] {
                0 => e[3] += 1,
                k if k >= HUE.min_pixels => {
                    e[1] += 1;
                    if rb.get(tt).is_some_and(|(r, b)| b > r) {
                        blue[kind_of(&t, &mountains, *tt)] += 1;
                    }
                }
                _ => e[2] += 1,
            }
        }
        Census {
            tiles,
            pixels,
            off: pixels[5],
            blue,
        }
    }

    /// **Item 4160: the second photograph's census.** run1564's frame by
    /// kind, beside [`RUN1559B_3279_UNSCORED`]: water, the shore and rock
    /// are in view, scoreable under the row's selection.
    #[test]
    fn the_second_photograph_s_census() {
        let Some(mut p) = probe_at(run1564()) else {
            return;
        };
        let c = census(&mut p);
        for (k, row) in KINDS.iter().zip(&c.tiles) {
            println!("run1564 {k}: [in view, scored, short, none] {row:?}");
        }
        println!("run1564 pixels by {WHY:?}: {:?}", c.pixels);
        println!(
            "run1564 scored tiles the photograph shows blue, by kind: {:?}",
            c.blue
        );
        assert_eq!((c.tiles, c.off), RUN1564_3279_CENSUS);
        // The shore in register: plain land the photograph shows blue and
        // water it does not, at the camera and with it moved.
        let wrong = |k: &Census| [k.blue[0], k.tiles[6][1] - k.blue[6]];
        let mut shore = vec![wrong(&c)];
        for d in [384.0, 768.0, 1536.0] {
            for (dx, dy) in [(d, 0.0), (0.0, d), (-d, 0.0), (0.0, -d)] {
                p.camera.loc_x += dx;
                p.camera.loc_y += dy;
                let k = census(&mut p);
                println!(
                    "run1564 camera off by ({dx}, {dy}): blue by kind {:?}, scored {:?}, wrong {:?}",
                    k.blue,
                    k.tiles.map(|r| r[1]),
                    wrong(&k)
                );
                shore.push(wrong(&k));
                p.camera.loc_x -= dx;
                p.camera.loc_y -= dy;
            }
        }
        assert_eq!(shore, RUN1564_3279_SHORE);
    }

    /// **Item 4160: the ground's row on the second photograph**, its rule
    /// as committed on the first, by kind.
    #[test]
    fn the_ground_on_the_second_photograph() {
        let Some(mut p) = probe_at(run1564()) else {
            return;
        };
        let Some(r) = ground_row(&mut p) else { return };
        let t = p.world.runner.terrain();
        let mountains: std::collections::BTreeSet<(i32, i32)> =
            t.mountains.iter().flatten().map(|q| (q.x, q.y)).collect();
        let mut by = [[0u32; 2]; 7];
        for (tt, ok) in &r.scored {
            let e = &mut by[kind_of(&t, &mountains, *tt)];
            e[1] += 1;
            if *ok {
                e[0] += 1;
            }
        }
        for (k, row) in KINDS.iter().zip(&by) {
            println!("run1564 ground {k}: [agree, scored] {row:?}");
        }
        // The value diff: each kind's first scored tile, the photograph's
        // mean against ours, and where it sits.
        for (k, name) in KINDS.iter().enumerate() {
            let Some((tt, ok)) = r
                .scored
                .iter()
                .find(|(tt, _)| kind_of(&t, &mountains, *tt) == k)
            else {
                continue;
            };
            let (th, us, at) = r.means[tt];
            println!(
                "run1564 sample {name}: tile {tt:?} at pixel {at:?}, theirs ({:.0}, {:.0}, {:.0}) share {:?}, ours {:?} share {:?}, agrees {ok}",
                th[0],
                th[1],
                th[2],
                chromaticity(th),
                us,
                us.map(chromaticity)
            );
        }
        println!(
            "RUN1564_3279_GROUND: {} of {}, the base alone {}",
            r.row,
            r.scored.len(),
            r.bare
        );
        assert_eq!(((r.row, r.scored.len()), by, r.bare), RUN1564_3279_GROUND);
        // Item 4172: the same tiles on brightness; the lake and the beach
        // are the pass's boxes.
        let l = luma_row(&p, &r);
        print_luma(
            "RUN1564_3279_GROUND_LUMA",
            &p,
            &r,
            &l,
            &[
                ("the lake", (900, 282, 1020, 362)),
                ("the beach", (700, 672, 760, 692)),
            ],
        );
        assert_eq!((l.0, l.1), RUN1564_3279_GROUND_LUMA);
    }

    /// **Item 4174: the floor and the water as the card draws them**, on
    /// the second photograph: [`RUN1564_3279_GROUND`]'s rule and 4172's
    /// luminance reader on the draw as it stood, then under [`Card`]'s two
    /// models, by kind.
    #[test]
    fn the_card_s_water_on_the_second_photograph() {
        let Some(mut p) = probe_at(run1564()) else {
            return;
        };
        let t = p.world.runner.terrain();
        let mountains: std::collections::BTreeSet<(i32, i32)> =
            t.mountains.iter().flatten().map(|q| (q.x, q.y)).collect();
        let floor = ocean_floor(&p.root);
        let mut rows = Vec::new();
        for (name, card) in [
            ("as drawn", None),
            ("unlit", Some(Card::Unlit)),
            ("lit", Some(Card::Lit)),
        ] {
            let Some(r) = ground_row_card(&mut p, Some(&floor), card) else {
                return;
            };
            let mut by = [[0u32; 2]; 7];
            for (tt, ok) in &r.scored {
                let e = &mut by[kind_of(&t, &mountains, *tt)];
                e[1] += 1;
                if *ok {
                    e[0] += 1;
                }
            }
            let l = luma_row(&p, &r);
            for tt in [(182, 191), (161, 191), (159, 193)] {
                if let Some((th, us, at)) = r.means.get(&tt) {
                    println!(
                        "4174 {name} tile {tt:?} at {at:?}: theirs ({:.0}, {:.0}, {:.0}) {:?}, ours {:?} {:?}",
                        th[0],
                        th[1],
                        th[2],
                        chromaticity(*th),
                        us,
                        us.map(chromaticity)
                    );
                }
            }
            println!(
                "4174 {name}: ground {} of {} (SANDY {:?}, OCEAN {:?}), luma {} (SANDY {:?}, OCEAN {:?})",
                r.row,
                r.scored.len(),
                by[5],
                by[6],
                l.0.0,
                l.1[5],
                l.1[6]
            );
            rows.push((r.row, l.0.0));
        }
        println!("4174 rows: {rows:?}");
        assert_eq!(rows, RUN1564_3279_CARD);
    }

    /// **Item 4164: the water on the second photograph**, the read draw
    /// against its controls, by kind.
    #[test]
    fn the_water_on_the_second_photograph() {
        let Some(mut p) = probe_at(run1564()) else {
            return;
        };
        let floor = ocean_floor(&p.root);
        let file = |rel: &str| {
            let bytes = std::fs::read(p.root.join(rel)).expect("the install's file");
            Mips::new(render::tga::decode(&bytes).expect("it decodes"))
        };
        let frame = file("terrain art/newwater/water0000.tga");
        let t = p.world.runner.terrain();
        let mountains: std::collections::BTreeSet<(i32, i32)> =
            t.mountains.iter().flatten().map(|q| (q.x, q.y)).collect();
        let mut rows = Vec::new();
        for (name, surface) in [
            ("the land pass alone", None),
            ("the water as the ocean base", Some(&floor)),
            ("the water as water0000", Some(&frame)),
        ] {
            let Some(r) = ground_row_with(&mut p, surface) else {
                return;
            };
            let mut by = [[0u32; 2]; 7];
            for (tt, ok) in &r.scored {
                let e = &mut by[kind_of(&t, &mountains, *tt)];
                e[1] += 1;
                if *ok {
                    e[0] += 1;
                }
            }
            println!(
                "run1564 water, {name}: {} of {}, SANDY {:?}, OCEAN {:?}",
                r.row,
                r.scored.len(),
                by[5],
                by[6]
            );
            rows.push([by[5][0], by[6][0]]);
            // SANDY by whether the photograph shows the tile blue, and
            // whether we lay water on it.
            let mut split = BTreeMap::new();
            for (tt, ok) in &r.scored {
                if kind_of(&t, &mountains, *tt) != 5 {
                    continue;
                }
                let (th, us, _) = r.means[tt];
                let wet = holds_water(&t, tt.0 / TILES_PER_CELL, tt.1 / TILES_PER_CELL);
                let e = split
                    .entry((th[2] > th[0], wet))
                    .or_insert((0u32, 0u32, [0f32; 3], [0f32; 3], 0u32));
                e.1 += 1;
                if *ok {
                    e.0 += 1;
                }
                let us = us.unwrap_or_default();
                for (i, (a, b)) in th.iter().zip(us).enumerate() {
                    e.2[i] += a;
                    e.3[i] += b;
                }
                e.4 += 1;
            }
            for ((blue, wet), (a, n, th, us, k)) in &split {
                let k = *k as f32;
                println!(
                    "  SANDY photo blue {blue}, water laid {wet}: {a} of {n}, theirs {:?} ours {:?}",
                    th.map(|v| (v / k) as i32),
                    us.map(|v| (v / k) as i32)
                );
            }
        }
        println!("RUN1564_3279_WATER: {rows:?}");
        assert_eq!(rows, RUN1564_3279_WATER);
    }

    /// One scored tile on the still (item 4171): the photograph's mean,
    /// the still's, and whether they agree.
    pub(crate) type StillTile = ((i32, i32), [f32; 3], [f32; 3], bool);

    /// **The ground's row on the client's still** (item 4171;
    /// `docs/RENDERER.md` §3.15): [`ground_row`]'s tiles and
    /// their pixels, scored as there, ours the mean of `still` over every
    /// ground pixel of the tile.
    pub(crate) fn still_ground_row(p: &mut Probe, still: &[u8]) -> Vec<StillTile> {
        let g = ground(p);
        let px = |rgba: &[u8], i: usize| [0, 1, 2].map(|c| f32::from(rgba[i * 4 + c]));
        type Px = Vec<[f32; 3]>;
        let mut tiles: BTreeMap<(i32, i32), (Px, Px)> = BTreeMap::new();
        for (i, tt) in g.tile.iter().enumerate() {
            let Some(tt) = *tt else { continue };
            let e = tiles.entry(tt).or_default();
            e.0.push(px(&p.photo.rgba, i));
            e.1.push(px(still, i));
        }
        tiles
            .into_iter()
            .filter(|(_, (th, _))| th.len() as u32 >= HUE.min_pixels)
            .map(|(tt, (th, us))| {
                let (a, b) = (mean(&th), mean(&us));
                (
                    tt,
                    a,
                    b,
                    dist(chromaticity(a), chromaticity(b)) <= HUE.distance,
                )
            })
            .collect()
    }

    /// **The ground's rows on the client's still** (item 4171, §3.15, committed before this count): on both photographs, the still
    /// beside the painter's [`ground_row`], by kind, every tile whose verdict
    /// the still changes counted by kind and the first of each named with
    /// both sides' means.
    #[test]
    fn the_ground_on_the_client_s_still() {
        use crate::gpu::SilhouetteOrder;
        let mut got = Vec::new();
        for (name, shot) in [("run1559b", run1559b()), ("run1564", run1564())] {
            let Some(mut p) = probe_at(shot) else { return };
            let size = (p.photo.width as u32, p.photo.height as u32);
            let camera = p.camera;
            let still = crate::still_of(
                &mut p.world,
                &camera,
                size,
                SilhouetteOrder::Before,
                wgpu::TextureFormat::Rgba8Unorm,
            );
            if let Some(dir) = std::env::var_os("RON_STILL_DIR") {
                let out = PathBuf::from(dir).join(format!("still-{name}.bmp"));
                std::fs::write(&out, crate::gpu::bmp(&still, size)).expect("the still writes");
            }
            // Made to fail: the draw before item 4171, on an sRGB target.
            let srgb = crate::still_of(
                &mut p.world,
                &camera,
                size,
                SilhouetteOrder::After,
                wgpu::TextureFormat::Rgba8UnormSrgb,
            );
            let tiles = still_ground_row(&mut p, &still);
            let before = still_ground_row(&mut p, &srgb);
            let before = (before.iter().filter(|q| q.3).count(), before.len());
            let Some(r) = ground_row(&mut p) else { return };
            let t = p.world.runner.terrain();
            let mountains: std::collections::BTreeSet<(i32, i32)> =
                t.mountains.iter().flatten().map(|q| (q.x, q.y)).collect();
            let painter: BTreeMap<(i32, i32), bool> = r.scored.iter().copied().collect();
            assert_eq!(
                tiles.iter().map(|q| q.0).collect::<Vec<_>>(),
                r.scored.iter().map(|q| q.0).collect::<Vec<_>>(),
                "{name}: the still scores the painter's tiles"
            );
            let mut by = [[0u32; 2]; 7];
            let mut crossed = [[0u32; 2]; 7];
            for (tt, th, us, ok) in &tiles {
                let k = kind_of(&t, &mountains, *tt);
                by[k][1] += 1;
                by[k][0] += u32::from(*ok);
                let was = painter[tt];
                if was != *ok {
                    let c = &mut crossed[k][usize::from(*ok)];
                    if *c == 0 {
                        let (_, ours, at) = r.means[tt];
                        println!(
                            "{name} {} crosses {}: tile {tt:?} at pixel {at:?}, theirs ({:.0}, {:.0}, {:.0}) {:?}, the painter's {:?} {:?}, the still's ({:.0}, {:.0}, {:.0}) {:?}",
                            KINDS[k],
                            if *ok { "up" } else { "down" },
                            th[0],
                            th[1],
                            th[2],
                            chromaticity(*th),
                            ours,
                            ours.map(chromaticity),
                            us[0],
                            us[1],
                            us[2],
                            chromaticity(*us),
                        );
                    }
                    *c += 1;
                }
            }
            let row = (by.iter().map(|k| k[0] as usize).sum::<usize>(), tiles.len());
            // Item 4178: the same tiles on brightness, the painter's
            // reader on the still's means.
            let luma = {
                use render::silhouette::{LUMA_READER, luminance};
                let mut lby = [[0u32; 2]; 7];
                for (tt, th, us, _) in &tiles {
                    let k = kind_of(&t, &mountains, *tt);
                    lby[k][1] += 1;
                    lby[k][0] += u32::from(LUMA_READER.agrees(luminance(*us), luminance(*th)));
                }
                let n = lby.iter().map(|k| k[0]).sum::<u32>();
                println!("{name} still luma {n} of {} by {lby:?}", tiles.len());
                ((n, tiles.len()), lby)
            };
            for (k, (b, c)) in KINDS.iter().zip(by.iter().zip(&crossed)) {
                println!("{name} still {k}: [agree, scored] {b:?}, crossed [down, up] {c:?}");
            }
            println!(
                "{name}: the ground on the still {row:?}, the painter's ({}, {}), the draw before {before:?}",
                r.row,
                r.scored.len()
            );
            got.push((row, by, before, luma));
        }
        assert_eq!(got[0].0, RUN1559B_3279_GROUND_STILL);
        assert_eq!((got[1].0, got[1].1), RUN1564_3279_GROUND_STILL);
        assert_eq!([got[0].2, got[1].2], GROUND_STILL_SRGB);
        assert_eq!([got[0].3, got[1].3], GROUND_LUMA_STILL);
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
        let tex = GroundTextures::untinted(&p.root, &tileset).expect("the ground's textures load");
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
        let Some(mut p) = probe() else { return };
        let Some(r) = ground_row(&mut p) else { return };
        assert_eq!((r.bare, r.scored.len()), RUN1559B_3279_GROUND_BARE);
        assert_eq!((r.row, r.scored.len()), RUN1559B_3279_GROUND);
        // Item 4172: the same tiles on brightness; the mountain's body is
        // the pass's box (1480, 150) 80 × 60, its eight tiles.
        let l = luma_row(&p, &r);
        print_luma(
            "RUN1559B_3279_GROUND_LUMA",
            &p,
            &r,
            &l,
            &[
                ("the mountain's body", (1480, 150, 1560, 210)),
                ("grass", (660, 430, 720, 470)),
            ],
        );
        assert_eq!((l.0, [l.1[0], l.1[1], l.1[2]]), RUN1559B_3279_GROUND_LUMA);
        let body: Vec<f32> =
            l.2.iter()
                .filter(|(tt, _, _)| {
                    let at = r.means[tt].2;
                    (1480..1560).contains(&at.0) && (150..210).contains(&at.1)
                })
                .map(|&(_, yo, yt)| yo / yt)
                .collect();
        assert!(
            !body.is_empty() && body.iter().all(|q| (0.8..1.0).contains(q)),
            "the mountain's body is in the row and agrees on the painter: {body:?}"
        );
        // Made to fail: a white light half again as strong on our side.
        let bright =
            l.2.iter()
                .filter(|&&(_, yo, yt)| render::silhouette::LUMA_READER.agrees(1.5 * yo, yt))
                .count();
        println!("RUN1559B_3279_GROUND_LUMA under a light 1.5 times as strong: {bright}");
        assert!(bright * 2 < l.0.0, "a white light moves the row");
    }

    /// [`luma_row`]'s answer: `(agree, scored)`, by kind, and each scored
    /// tile's luminance, ours then the photograph's.
    pub(crate) type LumaRow = ((usize, usize), [[u32; 2]; 7], Vec<((i32, i32), f32, f32)>);

    /// **The ground's brightness** (item 4172; `docs/RENDERER.md` §3.14,
    /// "On brightness", committed before its count): of the ground
    /// row's scored tiles, those whose drawn mean luminance agrees with the
    /// photograph's under [`render::silhouette::LUMA_READER`]; a tile we
    /// draw no pixel of parts. By kind, [`KINDS`]' order, `[agree, scored]`.
    /// A named box on the photograph, `x0, y0, x1, y1`.
    pub(crate) type Boxed<'a> = (&'a str, (i32, i32, i32, i32));

    pub(crate) fn luma_row(p: &Probe, r: &GroundRow) -> LumaRow {
        use render::silhouette::{LUMA_READER, luminance};
        let t = p.world.runner.terrain();
        let mountains: std::collections::BTreeSet<(i32, i32)> =
            t.mountains.iter().flatten().map(|q| (q.x, q.y)).collect();
        let mut by = [[0u32; 2]; 7];
        let mut row = 0;
        let mut ys = Vec::new();
        for (tt, _) in &r.scored {
            let (th, us, _) = r.means[tt];
            let yt = luminance(th);
            let yo = us.map_or(0.0, luminance);
            let ok = us.is_some() && LUMA_READER.agrees(yo, yt);
            let e = &mut by[kind_of(&t, &mountains, *tt)];
            e[1] += 1;
            if ok {
                e[0] += 1;
                row += 1;
            }
            ys.push((*tt, yo, yt));
        }
        ((row, r.scored.len()), by, ys)
    }

    /// [`luma_row`]'s value diff: by kind, the mean of the ratio ours over
    /// the photograph's and its spread, and the tiles whose centroid lies in
    /// `boxes` (`x0, y0, x1, y1`, named).
    pub(crate) fn print_luma(
        label: &str,
        p: &Probe,
        r: &GroundRow,
        row: &LumaRow,
        boxes: &[Boxed<'_>],
    ) {
        let t = p.world.runner.terrain();
        let mountains: std::collections::BTreeSet<(i32, i32)> =
            t.mountains.iter().flatten().map(|q| (q.x, q.y)).collect();
        let mut ratios: [Vec<f32>; 7] = Default::default();
        for &(tt, yo, yt) in &row.2 {
            ratios[kind_of(&t, &mountains, tt)].push(yo / yt.max(1.0));
        }
        for (k, v) in ratios.iter_mut().enumerate() {
            if v.is_empty() {
                continue;
            }
            v.sort_by(|a, b| a.total_cmp(b));
            println!(
                "{label} {}: [agree, scored] {:?}; ours / theirs: min {:.2} p10 {:.2} median {:.2} p90 {:.2} max {:.2}",
                KINDS[k],
                row.1[k],
                v[0],
                v[v.len() / 10],
                v[v.len() / 2],
                v[v.len() * 9 / 10],
                v[v.len() - 1]
            );
        }
        for (name, (x0, y0, x1, y1)) in boxes {
            for &(tt, yo, yt) in &row.2 {
                let (th, us, at) = r.means[&tt];
                if (*x0..*x1).contains(&at.0) && (*y0..*y1).contains(&at.1) {
                    println!(
                        "{label} {name}: tile {tt:?} at {at:?}, theirs ({:.0}, {:.0}, {:.0}) Y {yt:.1}, ours {:?} Y {yo:.1}, ratio {:.2}",
                        th[0],
                        th[1],
                        th[2],
                        us.map(|u| u.map(|v| v.round())),
                        yo / yt.max(1.0)
                    );
                }
            }
        }
        println!("{label}: {:?} of {:?}", row.0, row.1);
    }

    /// The ground row on one photograph (item 4160): on the base alone,
    /// with the patch and the mountains laid, and each scored tile with
    /// whether it agrees.
    /// A scored tile's mean colour, theirs then ours, and its centroid.
    pub(crate) type TileMean = ([f32; 3], Option<[f32; 3]>, (i32, i32));

    pub(crate) struct GroundRow {
        pub bare: usize,
        pub row: usize,
        pub scored: Vec<((i32, i32), bool)>,
        /// Each scored tile's mean colour, the photograph's and ours (`None`
        /// where we draw no textured pixel), and its pixels' centroid.
        pub means: BTreeMap<(i32, i32), TileMean>,
    }

    /// [`RUN1559B_3279_GROUND`]'s count on `p`'s photograph and camera, as
    /// item 4127 committed its rule and 4139 and 4141 laid the patch and the
    /// mountains over it; `None` with no world seed.
    /// **The painter's patches** (item 4200): the choice on `BASELAND`
    /// cells alone, as before the coast arm. The painter draws neither the
    /// coral the card lays over an `OCEAN` patch nor the land's fade
    /// (item 4192), so its rows keep land 0's patches; the coast's are the
    /// client's still's (`docs/RENDERER.md` §3.18).
    pub(crate) fn painter_choice(
        t: &sim::observe::TerrainScene,
        mut choice: Vec<Option<crate::patch::Patch>>,
    ) -> Vec<Option<crate::patch::Patch>> {
        for (c, q) in choice.iter_mut().enumerate() {
            if t.land.get(c) != Some(&render::terrain::BASELAND) {
                *q = None;
            }
        }
        choice
    }

    pub(crate) fn ground_row(p: &mut Probe) -> Option<GroundRow> {
        let floor = ocean_floor(&p.root);
        ground_row_with(p, Some(&floor))
    }

    /// The `<OCEAN>` base's first texture, the water's surface the row
    /// draws (item 4164).
    pub(crate) fn ocean_floor(root: &std::path::Path) -> Mips {
        let tileset = crate::world::lobby_tileset(
            root,
            &crate::world::gamelog_dir().join(crate::world::START_DUMP),
        )
        .expect("the lobby's tileset");
        let tex = GroundTextures::untinted(root, &tileset).expect("the ground's textures load");
        tex.ocean[0][0].clone()
    }

    /// [`ground_row`] with `surface` the water's texture, laid over every
    /// cell that holds water at [`water_alpha`]; `None` lays no water.
    pub(crate) fn ground_row_with(p: &mut Probe, surface: Option<&Mips>) -> Option<GroundRow> {
        ground_row_card(p, surface, None)
    }

    /// **How the card's floor and water are counted** (item 4174): `Unlit`
    /// composites the stages' colours as they leave the texture stages;
    /// `Lit` runs each through the Extended Edition's lit branch (the
    /// install's `ps_global.hlsl`, read: the stage colour to the 1.45,
    /// times 5 × (the material diffuse × the light × N·L / π + the material
    /// ambient × the scene's ambient); no specular, no screen reflection),
    /// blends the water over the floor in HDR, and tone-maps at the clamp.
    /// A land pixel under the water enters at the HDR value the tone map
    /// takes to its drawn colour.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub(crate) enum Card {
        Unlit,
        Lit,
    }

    /// The tone map at the clamp, 0–255 in and out (item 4174).
    pub(crate) fn tone(lin: f32) -> f32 {
        use render::silhouette::{EXPOSURE, MIN_LUMINANCE, WHITE_POINT, filmic};
        let white = 1.0 / filmic(MIN_LUMINANCE * WHITE_POINT);
        255.0 * (filmic(EXPOSURE * lin) * white).clamp(0.0, 1.0)
    }

    /// [`tone`]'s inverse on a drawn byte, by bisection (item 4174).
    pub(crate) fn untone(c: f32) -> f32 {
        let (mut lo, mut hi) = (0.0f32, 64.0f32);
        for _ in 0..40 {
            let mid = 0.5 * (lo + hi);
            if tone(mid) < c { lo = mid } else { hi = mid }
        }
        0.5 * (lo + hi)
    }

    /// The lit branch's HDR value of a stage colour (item 4174): `diffuse`
    /// the material diffuse × the light's, `ambient` the material ambient.
    pub(crate) fn lit(stage: [f32; 3], diffuse: [f32; 3], ambient: f32) -> [f32; 3] {
        use render::silhouette::SCENE_AMBIENT;
        let n_dot_l = std::f32::consts::FRAC_1_SQRT_2;
        [0, 1, 2].map(|k| {
            let c = (stage[k] / 255.0).clamp(0.0, 1.0).powf(1.45);
            c * 5.0 * (diffuse[k] * n_dot_l / std::f32::consts::PI + ambient * SCENE_AMBIENT[k])
        })
    }

    /// [`ground_row_with`], and with `card` the floor and the water as the
    /// card draws them (item 4174): an `OCEAN` tile, which the land pass
    /// does not draw, shows the floor ([`floor_stage`] of the base sampled at
    /// [`floor_uvs`], under [`floor_grey`] of the [`ScreenRamp`]); every
    /// wet tile takes the water, [`water_rgb`] of the ramp at its alpha.
    pub(crate) fn ground_row_card(
        p: &mut Probe,
        surface: Option<&Mips>,
        card: Option<Card>,
    ) -> Option<GroundRow> {
        use render::silhouette::{Paint, Rect, paint};
        let g = ground(p);
        let size = (p.photo.width, p.photo.height);
        let ramp = ScreenRamp::of(&p.camera, (size.0 as u32, size.1 as u32));
        let ocean = ocean_floor(&p.root);
        let (w, _) = size;
        let tileset = crate::world::lobby_tileset(
            &p.root,
            &crate::world::gamelog_dir().join(crate::world::START_DUMP),
        )
        .expect("the lobby's tileset");
        let tex = GroundTextures::untinted(&p.root, &tileset).expect("the ground's textures load");
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
        // Item 4164: the water's surface, sampled where the base lies.
        let zeros = vec![0; base.tris.len()];
        let water = surface.map(|m| {
            paint(
                &object,
                &Paint::Textures {
                    uvs: &base.uvs,
                    mips: std::slice::from_ref(m),
                    of: &zeros,
                },
                &p.camera,
                size,
                window,
            )
        });
        // Item 4139 (§3.9): the transition patch, our own choice on our own
        // cells, laid over the base at the alpha's A.
        let alphas = load_alphas(&p.root).expect("the alphas load");
        let codes: Vec<[u32; 8]> = alphas.iter().map(|a| a.1).collect();
        let seed = crate::patch::tests::world_seed()?;
        let choice = painter_choice(
            &t,
            our_patches(&t, &p.root, &tileset, seed, &codes).expect("the tileset"),
        );
        let patch = ground_patch(&t, &mountains, &tex, &mesh, &choice);
        let laid = paint(
            &Object {
                key: 1,
                tris: patch.tris.clone(),
            },
            &Paint::Textures {
                uvs: &patch.uvs,
                mips: &flat,
                of: &patch.of,
            },
            &p.camera,
            size,
            window,
        );
        // Item 4141 (§3.10): a mountain's own draw, over the ground §3.9
        // lays beneath it.
        let none = std::collections::BTreeSet::new();
        let under_base = ground_base(&t, &none, &tex, &mesh);
        let under_patch = ground_patch(&t, &none, &tex, &mesh, &choice);
        let art = MountainArt::untinted(&p.root, &tileset).expect("the mountains' art loads");
        let range = ground_mountain(&t, &art, &mesh);
        let painted = |tris: &[[[f32; 3]; 3]], uvs: &[[[f32; 2]; 3]], of: &[usize]| {
            paint(
                &Object {
                    key: 1,
                    tris: tris.to_vec(),
                },
                &Paint::Textures {
                    uvs,
                    mips: &flat,
                    of,
                },
                &p.camera,
                size,
                window,
            )
        };
        let laid_under = painted(&under_base.tris, &under_base.uvs, &under_base.of);
        let laid_under_patch = painted(&under_patch.tris, &under_patch.uvs, &under_patch.of);
        let laid_range = painted(&range.tris, &range.uvs, &vec![0; range.tris.len()]);
        let ww = (window.x1 - window.x0) as usize;
        let mountain_at = |j: usize, tt: (i32, i32)| {
            let q = laid_range[j].filter(|q| range.tile[q.tri as usize] == tt)?;
            let c0 = laid_under[j]
                .filter(|u| under_base.tile[u.tri as usize] == tt)?
                .rgb;
            let under = laid_under_patch[j]
                .filter(|u| under_patch.tile[u.tri as usize] == tt)
                .map_or(c0, |u| {
                    let k = u.tri as usize;
                    let a =
                        patch_alpha(&alphas[under_patch.alpha[k]].0, &under_patch.patch[k], u.uv);
                    [0, 1, 2].map(|i| c0[i] * (1.0 - a) + u.rgb[i] * a)
                });
            Some(art.over(range.template[q.tri as usize], q.uv, under))
        };
        let cell = |(tx, ty): (i32, i32)| (tx / TILES_PER_CELL, ty / TILES_PER_CELL);
        let sub = |(cx, cy): (i32, i32)| t.land_sub[(cy * t.width + cx) as usize];
        let photo_at = |i: usize| [0, 1, 2].map(|c| f32::from(p.photo.rgba[i * 4 + c]));
        type Px = Vec<[f32; 3]>;
        let mut tiles: BTreeMap<(i32, i32), (Px, Px)> = BTreeMap::new();
        let mut bare: BTreeMap<(i32, i32), Px> = BTreeMap::new();
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
            let Some(c0) = drawn else { continue };
            bare.entry(tt).or_default().push(c0);
            let c = laid[j]
                .filter(|q| patch.tile[q.tri as usize] == tt)
                .map_or(c0, |q| {
                    let k = q.tri as usize;
                    let a = patch_alpha(&alphas[patch.alpha[k]].0, &patch.patch[k], q.uv);
                    [0, 1, 2].map(|i| c0[i] * (1.0 - a) + q.rgb[i] * a)
                });
            let c = if mountains.contains(&tt) {
                let Some(m) = mountain_at(j, tt) else {
                    continue;
                };
                m
            } else {
                c
            };
            // Item 4174: the floor and the water as the card draws them.
            if let (Some(model), Some(q)) = (card, ours[j])
                && !mountains.contains(&tt)
            {
                let (cx, cy) = cell(tt);
                let (wx, wy) = ((cx as f32 + q.uv[0]) * 768.0, (cy as f32 + q.uv[1]) * 768.0);
                let f = ramp.at(wx, wy);
                let land = t.land[(cy * t.width + cx) as usize];
                let lin = |c: [f32; 3]| c.map(untone);
                let under = if land == render::terrain::OCEAN {
                    let (a, b) = floor_uvs(wx, wy);
                    let t0 = ocean.sample(0, a[0], a[1]);
                    let t1 = ocean.sample(0, b[0], b[1]);
                    let st =
                        floor_stage([t0[0], t0[1], t0[2]], [t1[0], t1[1], t1[2]], floor_grey(f));
                    match model {
                        Card::Unlit => st,
                        Card::Lit => lit(st, [1.36; 3], 1.0),
                    }
                } else {
                    match model {
                        Card::Unlit => c,
                        Card::Lit => lin(c),
                    }
                };
                let w = water_rgb(f).map(f32::from);
                let over = match model {
                    Card::Unlit => w,
                    Card::Lit => lit(w, [1.0, 1.0, 1.5], 0.2),
                };
                let a = water_alpha(&t, cell(tt), q.uv).unwrap_or(0.0) / 255.0;
                let mixed = [0, 1, 2].map(|i| under[i] * (1.0 - a) + over[i] * a);
                let c = match model {
                    Card::Unlit => mixed,
                    Card::Lit if land == render::terrain::OCEAN || a > 0.0 => mixed.map(tone),
                    Card::Lit => c,
                };
                e.1.push(c);
                continue;
            }
            // Item 4164: the water over the cell, at its alpha.
            let c = match (&water, ours[j]) {
                (Some(wp), Some(q)) if !mountains.contains(&tt) => {
                    match (water_alpha(&t, cell(tt), q.uv), wp[j]) {
                        (Some(a), Some(wq)) => {
                            let a = a / 255.0;
                            [0, 1, 2].map(|i| c[i] * (1.0 - a) + wq.rgb[i] * a)
                        }
                        _ => c,
                    }
                }
                _ => c,
            };
            e.1.push(c);
            if !mountains.contains(&tt) {
                if let Some(k) = control[j].filter(|q| base.tile[q.tri as usize] == tt) {
                    lum.push((l(them), l(c0), l(k.rgb)));
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
        // The row on the base alone, as 4127 pinned it, for the value diff.
        let row_bare = scored
            .iter()
            .filter(|(tt, (th, _))| {
                bare.get(tt).is_some_and(|us| {
                    dist(chromaticity(mean(us)), chromaticity(mean(th))) <= HUE.distance
                })
            })
            .count();
        let mut moved: (Tiles, Tiles) = Default::default();
        let mut agree: Vec<((i32, i32), bool)> = Vec::new();
        let mut means = BTreeMap::new();
        let mut centroid: BTreeMap<(i32, i32), (i32, i32, i32)> = BTreeMap::new();
        for (i, tt) in g.tile.iter().enumerate() {
            if let Some(tt) = tt {
                let e = centroid.entry(*tt).or_default();
                *e = (e.0 + (i % w) as i32, e.1 + (i / w) as i32, e.2 + 1);
            }
        }
        let mut by: BTreeMap<(u8, bool), (u32, u32)> = BTreeMap::new();
        let mut parts: BTreeMap<&str, u32> = BTreeMap::new();
        for (tt, (th, us)) in &scored {
            let m = mountains.contains(tt);
            let c = cell(**tt);
            let e = by.entry((sub(c), m)).or_default();
            e.1 += 1;
            let ok = !us.is_empty()
                && dist(chromaticity(mean(us)), chromaticity(mean(th))) <= HUE.distance;
            let was = bare.get(*tt).is_some_and(|b| {
                dist(chromaticity(mean(b)), chromaticity(mean(th))) <= HUE.distance
            });
            if ok != was {
                if ok { &mut moved.0 } else { &mut moved.1 }.push(**tt);
            }
            agree.push((**tt, ok));
            means.insert(
                **tt,
                (
                    mean(th),
                    (!us.is_empty()).then(|| mean(us)),
                    centroid.get(*tt).map_or((0, 0), |c| (c.0 / c.2, c.1 / c.2)),
                ),
            );
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
        println!(
            "the patch laid: {} triangles; the row on the base alone {row_bare}; gained {} {:?}; lost {} {:?}",
            patch.tris.len(),
            moved.0.len(),
            moved.0,
            moved.1.len(),
            moved.1
        );
        println!("parting by what it would take: {parts:?}");
        println!("per pixel, by land_sub (agree, lit): {pixel:?}");
        println!(
            "mapping's check over {} pixels: r {r:.3} against the half-period control's {rc:.3}",
            lum.len()
        );
        Some(GroundRow {
            bare: row_bare,
            row,
            scored: agree,
            means,
        })
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

    /// The packet's `COAST_BASE` and `BASELAND` count (item 4139;
    /// `tools/render/land_buckets.py`'s `tileset` row).
    fn packet_tileset() -> Option<(u8, u8)> {
        let path = std::env::var_os("HOME").map(|h| {
            PathBuf::from(h)
                .join("ron-data/lab-experiments/2026-11-11-item-4139/run1559b-buckets.txt")
        })?;
        let Ok(text) = std::fs::read_to_string(&path) else {
            println!("skipping: no packet tileset at {}", path.display());
            return None;
        };
        let f: Vec<&str> = text.lines().next()?.split_whitespace().collect();
        let at = |k: &str| -> Option<u8> {
            let i = f.iter().position(|x| *x == k)?;
            f.get(i + 1)?.parse().ok()
        };
        Some((at("coast_base")?, at("bases")?))
    }

    /// The packet's own patch per cell with alpha geometry,
    /// `tools/render/patches.py`'s `patch` rows: `(cx, cy)` → (the alpha's
    /// bucket, `trans_texcoords1` at vertices 0, 4 and 20) (item 4132).
    #[allow(clippy::type_complexity)]
    fn packet_patches() -> Option<BTreeMap<(i32, i32), (usize, [f32; 2], [f32; 2], [f32; 2])>> {
        let path = std::env::var_os("RON_PATCHES")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|h| {
                    PathBuf::from(h)
                        .join("ron-data/lab-experiments/2026-11-11-item-4132/run1559b-patches.txt")
                })
            })?;
        let Ok(text) = std::fs::read_to_string(&path) else {
            println!("skipping: no packet patches at {}", path.display());
            return None;
        };
        let mut out = BTreeMap::new();
        for line in text.lines() {
            let f: Vec<&str> = line.split_whitespace().collect();
            if f.first() != Some(&"patch") {
                continue;
            }
            let at = |k: &str| f.iter().position(|x| *x == k).map(|i| f[i + 1]);
            let Some(uv) = f.iter().position(|x| *x == "uv") else {
                continue;
            };
            let uv: Vec<[f32; 2]> = f[uv + 1..]
                .iter()
                .filter_map(|q| {
                    let (u, v) = q.split_once(',')?;
                    Some([u.parse().ok()?, v.parse().ok()?])
                })
                .collect();
            if uv.len() != 25 || at("land") != Some("0") {
                continue;
            }
            let n = |i: usize| f[i].parse::<i32>().expect("a number");
            let bucket = at("alpha").and_then(|a| a.parse().ok()).expect("an alpha");
            out.insert((n(1), n(2)), (bucket, uv[0], uv[4], uv[20]));
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
        let tex = GroundTextures::untinted(&p.root, &tileset).expect("the ground's textures load");
        assert!(tex.bases.iter().all(|b| b.len() == 9), "four supertiles");
        let nb = tex.bases.len();
        let mesh = p.world.terrain().clone();
        let t = p.world.runner.terrain();
        let mountains: std::collections::BTreeSet<(i32, i32)> =
            t.mountains.iter().flatten().map(|q| (q.x, q.y)).collect();
        // The `BASELAND` tiles alone: the bases this test chooses among
        // (item 4164 lays `SANDY` and `OCEAN` in the same layer).
        let base = {
            let all = ground_base(&t, &mountains, &tex, &mesh);
            let mut b = GroundBase::default();
            for i in 0..all.tris.len() {
                let (tx, ty) = all.tile[i];
                let c = ((ty / TILES_PER_CELL) * t.width + tx / TILES_PER_CELL) as usize;
                if t.land[c] == render::terrain::BASELAND {
                    b.tris.push(all.tris[i]);
                    b.uvs.push(all.uvs[i]);
                    b.of.push(all.of[i]);
                    b.tile.push(all.tile[i]);
                }
            }
            b
        };
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
        // Item 4139 (§3.9): the rule's draw with our patch laid over it.
        let alphas = load_alphas(&p.root).expect("the alphas load");
        let codes: Vec<[u32; 8]> = alphas.iter().map(|a| a.1).collect();
        let Some(seed) = crate::patch::tests::world_seed() else {
            return;
        };
        let choice = painter_choice(
            &t,
            our_patches(&t, &p.root, &tileset, seed, &codes).expect("the tileset"),
        );
        let patch = ground_patch(&t, &mountains, &tex, &mesh, &choice);
        let laid_paint = paint(
            &Object {
                key: 1,
                tris: patch.tris.clone(),
            },
            &Paint::Textures {
                uvs: &patch.uvs,
                mips: &flat,
                of: &patch.of,
            },
            &p.camera,
            size,
            window,
        );
        let laid: Pairs = {
            let ours = draw(&base.of);
            let mut out: Pairs = BTreeMap::new();
            for (i, tt) in g.tile.iter().enumerate() {
                let Some(tt) = *tt else { continue };
                let (x, y) = ((i % w) as i32, (i / w) as i32);
                let j = (y - window.y0) as usize * ww + (x - window.x0) as usize;
                let Some(b) = ours[j].filter(|q| base.tile[q.tri as usize] == tt) else {
                    continue;
                };
                let c = laid_paint[j]
                    .filter(|q| patch.tile[q.tri as usize] == tt)
                    .map_or(b.rgb, |q| {
                        let k = q.tri as usize;
                        let a = patch_alpha(&alphas[patch.alpha[k]].0, &patch.patch[k], q.uv);
                        [0, 1, 2].map(|i| b.rgb[i] * (1.0 - a) + q.rgb[i] * a)
                    });
                out.entry(tt).or_default().push((photo_at(i), c));
            }
            out
        };
        let (ln, lby) = row(&laid);
        println!(
            "the rule with the patch laid: row {ln} by {lby:?} r {:.3}",
            r_all(&laid)
        );
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
        let mut gb_laid = 0;
        let mut laid_moved: (Tiles, Tiles) = Default::default();
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
            // The laid draw stands in for the rule's base: best of the four
            // with it in base `rb`'s place, and above `rb`'s control.
            let rl = r_tile(&laid, tt);
            let laid_wins = rl > rw[rb] && (0..nb).filter(|&b| b != rb).all(|b| rl >= r[b]);
            if laid_wins {
                gb_laid += 1;
            }
            if laid_wins != wins(rb) {
                if laid_wins {
                    &mut laid_moved.0
                } else {
                    &mut laid_moved.1
                }
                .push(*tt);
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
        println!(
            "RUN1559B_3279_GROUND_BASE with the patch laid {gb_laid} (the rule alone {gb_rule}); gained {} {:?}; lost {} {:?}",
            laid_moved.0.len(),
            laid_moved.0,
            laid_moved.1.len(),
            laid_moved.1
        );
        assert_eq!((gb_rule, plain.len()), RUN1559B_3279_GROUND_BASE_BARE);
        assert_eq!((gb_laid, plain.len()), RUN1559B_3279_GROUND_BASE);
        assert_eq!(
            ln, RUN1559B_3279_GROUND_GREY.0,
            "the row with the patch laid"
        );
        assert_eq!(gb_one, RUN1559B_3279_GROUND_BASE_ONE);
        assert!(gb_one.iter().all(|&n| n < gb_rule));
        assert_eq!(rn, RUN1559B_3279_GROUND_BARE.0, "the row as 4127 pinned it");
    }

    /// One patched plain tile's pixels for [`the_transition_patch`]: per
    /// ground pixel, the photograph, the base's texel, the patch's base's
    /// texel, each one cell index on (the wrong-cell control), and the
    /// texcoord across the cell.
    #[derive(Clone, Copy)]
    struct PatchPixel {
        photo: [f32; 3],
        base: [f32; 3],
        patch: [f32; 3],
        wbase: [f32; 3],
        wpatch: [f32; 3],
        /// The patch's base from our own cells' `land_sub − 1`, and its
        /// wrong-cell control (item 4139).
        opatch: [f32; 3],
        wopatch: [f32; 3],
        uv: [f32; 2],
        /// The pixel (item 4139: the footprint the pass's level is read
        /// from).
        xy: (i32, i32),
    }

    fn lum(c: [f32; 3]) -> f32 {
        0.3 * c[0] + 0.59 * c[1] + 0.11 * c[2]
    }

    fn pearson(v: &[(f32, f32)]) -> f32 {
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
    }

    fn mix(a: [f32; 3], b: [f32; 3], w: f32) -> [f32; 3] {
        [0, 1, 2].map(|k| a[k] * (1.0 - w) + b[k] * w)
    }

    /// **The transition patch, against its controls** (item 4132;
    /// `docs/RENDERER.md` §3.6, "The transition patch"): on the plain tiles
    /// whose cell carries a patch on run1559b's packet, Pearson's r of the
    /// photograph's luminance with each candidate draw's, each beside its
    /// wrong-cell control; a candidate wins a tile when its r is the
    /// highest of the candidates and above its own control.
    #[test]
    fn the_transition_patch() {
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
        let tex = GroundTextures::untinted(&p.root, &tileset).expect("the ground's textures load");
        assert!(tex.bases.iter().all(|b| b.len() == 9), "four supertiles");
        let mesh = p.world.terrain().clone();
        let t = p.world.runner.terrain();
        let mountains: std::collections::BTreeSet<(i32, i32)> =
            t.mountains.iter().flatten().map(|q| (q.x, q.y)).collect();
        let base = ground_base(&t, &mountains, &tex, &mesh);
        let flat = tex.flat();
        let cell = |(tx, ty): (i32, i32)| (tx / TILES_PER_CELL, ty / TILES_PER_CELL);
        let sub = |(cx, cy): (i32, i32)| t.land_sub[(cy * t.width + cx) as usize];
        let patch_of = |c: (i32, i32)| packet.get(&c).and_then(|q| q.3);
        let on = |k: usize| 9 * (k / 9) + (k % 9 + 1) % 9;

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
        let of_patch: Vec<usize> = base
            .tile
            .iter()
            .zip(&base.of)
            .map(|(&tt, &k)| patch_of(cell(tt)).map_or(k, |li| li as usize))
            .collect();
        let of_wbase: Vec<usize> = base.of.iter().map(|&k| on(k)).collect();
        let of_wpatch: Vec<usize> = of_patch.iter().map(|&k| on(k)).collect();
        // Item 4139: our own choice (`crate::patch::patches`), on the
        // packet's cells and on ours; the patch's base from our cells.
        let (Some(pk), Some(seed)) = (
            crate::patch::tests::packet(),
            crate::patch::tests::world_seed(),
        ) else {
            return;
        };
        let codes: Vec<[u32; 8]> = pk
            .alpha_names
            .iter()
            .map(|n| crate::patch::bitcodes(n).expect("an alpha's code"))
            .collect();
        let frac = sim::mapgen::Fractal::init(pk.width + 1, pk.height + 1, 0, 2, seed);
        let numbers = rondata::mapgen::tileset_groups(&p.root, &tileset).expect("the tileset");
        let (coast_base, base_subs) = (numbers.coast_base as u8, numbers.base_tiles as u8);
        // Stage 1 (parked 4134b): the tileset's two numbers from our data,
        // against the packet's `COAST_BASE` and `BASELAND` count.
        if let Some(theirs) = packet_tileset() {
            println!(
                "the tileset's numbers: ours {:?}, the packet's {theirs:?}",
                (coast_base, base_subs)
            );
            assert_eq!((coast_base, base_subs), theirs, "COAST_BASE and the bases");
        }
        let ground_of = |cells: &[(i8, u8)]| {
            crate::patch::patches(
                &crate::patch::Ground {
                    width: pk.width,
                    height: pk.height,
                    cells,
                    coast_base,
                    base_subs,
                },
                &frac,
                &codes,
            )
        };
        let our_cells: Vec<(i8, u8)> = t
            .land
            .iter()
            .zip(&t.land_sub)
            .map(|(&l, &s)| (l, s))
            .collect();
        let choice_packet = ground_of(&pk.cells);
        let choice_ours = ground_of(&our_cells);
        // The perturbation: the fractal's seed one on, on the packet's cells.
        let frac_off = sim::mapgen::Fractal::init(pk.width + 1, pk.height + 1, 0, 2, seed + 1);
        let choice_off = crate::patch::patches(
            &crate::patch::Ground {
                width: pk.width,
                height: pk.height,
                cells: &pk.cells,
                coast_base,
                base_subs,
            },
            &frac_off,
            &codes,
        );
        let parting_cells: std::collections::BTreeSet<(i32, i32)> = (0..pk.height)
            .flat_map(|y| (0..pk.width).map(move |x| (x, y)))
            .filter(|&(x, y)| {
                let i = (y * pk.width + x) as usize;
                choice_ours[i] != choice_packet[i] || our_cells[i] != pk.cells[i]
            })
            .collect();
        let of_opatch: Vec<usize> = base
            .tile
            .iter()
            .zip(&base.of)
            .map(|(&tt, &k)| {
                let (cx, cy) = cell(tt);
                let s = sub((cx, cy));
                if choice_ours[(cy * t.width + cx) as usize].is_some() && s > 0 {
                    tex.index(s - 1, cx, cy).unwrap_or(k)
                } else {
                    k
                }
            })
            .collect();
        let of_wopatch: Vec<usize> = of_opatch.iter().map(|&k| on(k)).collect();
        let [db, dp, dwb, dwp, dop, dwop] = [
            &base.of,
            &of_patch,
            &of_wbase,
            &of_wpatch,
            &of_opatch,
            &of_wopatch,
        ]
        .map(|of| draw(of));
        let ww = (window.x1 - window.x0) as usize;
        let photo_at = |i: usize| [0, 1, 2].map(|c| f32::from(p.photo.rgba[i * 4 + c]));
        let mut tiles: BTreeMap<(i32, i32), Vec<PatchPixel>> = BTreeMap::new();
        let mut count: BTreeMap<(i32, i32), u32> = BTreeMap::new();
        for (i, tt) in g.tile.iter().enumerate() {
            let Some(tt) = *tt else { continue };
            *count.entry(tt).or_default() += 1;
            let (x, y) = ((i % w) as i32, (i / w) as i32);
            let j = (y - window.y0) as usize * ww + (x - window.x0) as usize;
            let mine = |d: &[Option<render::silhouette::Texel>]| {
                d[j].filter(|q| base.tile[q.tri as usize] == tt)
            };
            if let (Some(b), Some(pp), Some(wb), Some(wp), Some(op), Some(wop)) = (
                mine(&db),
                mine(&dp),
                mine(&dwb),
                mine(&dwp),
                mine(&dop),
                mine(&dwop),
            ) {
                tiles.entry(tt).or_default().push(PatchPixel {
                    photo: photo_at(i),
                    base: b.rgb,
                    patch: pp.rgb,
                    wbase: wb.rgb,
                    wpatch: wp.rgb,
                    opatch: op.rgb,
                    wopatch: wop.rgb,
                    uv: b.uv,
                    xy: (x, y),
                });
            }
        }
        let plain: Vec<(i32, i32)> = count
            .iter()
            .filter(|(tt, n)| **n >= HUE.min_pixels && !mountains.contains(tt))
            .map(|(&tt, _)| tt)
            .filter(|tt| base.tile.contains(tt))
            .collect();
        let patched: Vec<(i32, i32)> = plain
            .iter()
            .copied()
            .filter(|&tt| patch_of(cell(tt)).is_some())
            .collect();
        println!(
            "plain tiles {}, of them patched on the packet {}",
            plain.len(),
            patched.len()
        );

        // The ramp's direction: the sum of the offsets to the neighbours
        // (eight-connected) whose land_sub is the patch's base.
        let toward = |c: (i32, i32)| -> [f32; 2] {
            let Some(li) = patch_of(c) else {
                return [0.0, 0.0];
            };
            let pb = (li / 9) as u8;
            let mut d = [0.0f32, 0.0];
            for dy in -1..=1i32 {
                for dx in -1..=1i32 {
                    let (nx, ny) = (c.0 + dx, c.1 + dy);
                    if (dx, dy) != (0, 0)
                        && (0..t.width).contains(&nx)
                        && (0..t.height).contains(&ny)
                        && sub((nx, ny)) == pb
                    {
                        d[0] += dx as f32;
                        d[1] += dy as f32;
                    }
                }
            }
            let n = (d[0] * d[0] + d[1] * d[1]).sqrt();
            if n > 0.0 { [d[0] / n, d[1] / n] } else { d }
        };
        // The packet's own patch (`tools/render/patches.py`): per cell, the
        // alpha's bucket and `trans_texcoords1` at the cell's corners
        // (vertex `j·5 + i`, `i` along world x, as its `trans_texcoords0`
        // and verts show), an affine map over the cell.
        let patches = packet_patches().unwrap_or_default();
        let alphas: Vec<(String, Mips)> = {
            let dir = p.root.join("terrain art").join("transitions");
            let mut names: Vec<String> = std::fs::read_dir(&dir)
                .map(|d| {
                    d.filter_map(|e| e.ok()?.file_name().into_string().ok())
                        .filter(|n| n.ends_with(".tga"))
                        .collect()
                })
                .unwrap_or_default();
            names.sort();
            names
                .into_iter()
                .map(|n| {
                    let bytes = std::fs::read(dir.join(&n)).expect("an alpha file");
                    let image = render::tga::decode(&bytes).expect("the alpha decodes");
                    (n, Mips::new(image))
                })
                .collect()
        };
        for (n, m) in &alphas {
            let im = &m.levels[0];
            let range = |c: usize| {
                im.rgba
                    .chunks(4)
                    .fold((255u8, 0u8), |a, q| (a.0.min(q[c]), a.1.max(q[c])))
            };
            println!(
                "alpha file {n}: {} x {}, r {:?} g {:?} b {:?} a {:?}",
                im.width,
                im.height,
                range(0),
                range(1),
                range(2),
                range(3)
            );
        }
        let read_alpha = |c: (i32, i32), uv: [f32; 2], turn: bool| -> f32 {
            let Some(&(bucket, t0, t4, t20)) = patches.get(&c) else {
                return 0.0;
            };
            let Some((_, m)) = alphas.get(bucket) else {
                return 0.0;
            };
            let (u, v) = if turn {
                (uv[1], 1.0 - uv[0])
            } else {
                (uv[0], uv[1])
            };
            let au = t0[0] + u * (t4[0] - t0[0]) + v * (t20[0] - t0[0]);
            let av = t0[1] + u * (t4[1] - t0[1]) + v * (t20[1] - t0[1]);
            m.sample(0, au, av)[3] / 255.0
        };
        // How the alpha is read (item 4139, stage 2): `level` the mip level,
        // `point` nearest texel with clamped addressing (the pass's
        // `TEXADD_CLAMP` and `MIP_POINT`), else bilinear wrapped as 4132
        // read it; `invert` weights the patch's base by 1 − A.
        #[derive(Clone, Copy)]
        struct Read {
            point: bool,
            invert: bool,
            named: bool,
        }
        let alpha_at = |choice: &[Option<crate::patch::Patch>],
                        c: (i32, i32),
                        uv: [f32; 2],
                        level: usize,
                        how: Read|
         -> f32 {
            let Some(q) = choice[(c.1 * t.width + c.0) as usize] else {
                return 0.0;
            };
            let Some((_, m)) = alphas.get(q.alpha) else {
                return 0.0;
            };
            let e = |k: usize, c: usize| q.uv[k][c] as f32 / 8.0;
            let au = e(0, 0) + uv[0] * (e(4, 0) - e(0, 0)) + uv[1] * (e(20, 0) - e(0, 0));
            let av = e(0, 1) + uv[0] * (e(4, 1) - e(0, 1)) + uv[1] * (e(20, 1) - e(0, 1));
            let level = if how.named {
                level.min(m.levels.len() - 1)
            } else {
                0
            };
            let a = if how.point {
                let im = &m.levels[level];
                let x = ((au * im.width as f32).floor() as i64).clamp(0, im.width as i64 - 1);
                let y = ((av * im.height as f32).floor() as i64).clamp(0, im.height as i64 - 1);
                f32::from(im.rgba[(y as usize * im.width + x as usize) * 4 + 3]) / 255.0
            } else {
                m.sample(level, au, av)[3] / 255.0
            };
            if how.invert { 1.0 - a } else { a }
        };
        // Our choice under the pass as stage 2 reads it: A at the pass's
        // level, nearest texel, clamped.
        let our_alpha =
            |choice: &[Option<crate::patch::Patch>], c: (i32, i32), uv: [f32; 2], lv: usize| {
                alpha_at(
                    choice,
                    c,
                    uv,
                    lv,
                    Read {
                        point: true,
                        invert: false,
                        named: true,
                    },
                )
            };
        println!(
            "packet patches with alpha geometry {}, alpha files {}",
            patches.len(),
            alphas.len()
        );
        type Weight<'a> = Box<dyn Fn((i32, i32), [f32; 2], usize) -> f32 + 'a>;
        let candidates: Vec<(&str, Weight)> = vec![
            ("(a) the base alone", Box::new(|_, _, _| 0.0)),
            ("(b) the patch's base alone", Box::new(|_, _, _| 1.0)),
            ("(c) a uniform half", Box::new(|_, _, _| 0.5)),
            (
                "(d) a linear ramp toward the patch's base",
                Box::new(|c, uv, _| {
                    let d = toward(c);
                    (0.5 + (uv[0] - 0.5) * d[0] + (uv[1] - 0.5) * d[1]).clamp(0.0, 1.0)
                }),
            ),
            (
                "(e) the packet's patch under the read alpha",
                Box::new(|c, uv, _| read_alpha(c, uv, false)),
            ),
            (
                "(f) the read alpha turned a quarter turn",
                Box::new(|c, uv, _| read_alpha(c, uv, true)),
            ),
            (
                "(g) our choice on the packet's cells, the pass as read",
                Box::new(|c, uv, lv| our_alpha(&choice_packet, c, uv, lv)),
            ),
            (
                "(h) our choice on our cells, our patch base, the pass as read",
                Box::new(|c, uv, lv| our_alpha(&choice_ours, c, uv, lv)),
            ),
            (
                "(i) control: our choice, the fractal's seed one on",
                Box::new(|c, uv, lv| our_alpha(&choice_off, c, uv, lv)),
            ),
            (
                "(j) A, level 0, bilinear wrapped (4132's read; (g))",
                Box::new(|c, uv, lv| {
                    alpha_at(
                        &choice_packet,
                        c,
                        uv,
                        lv,
                        Read {
                            point: false,
                            invert: false,
                            named: false,
                        },
                    )
                }),
            ),
            (
                "(k) 1 − A, level 0, bilinear",
                Box::new(|c, uv, lv| {
                    alpha_at(
                        &choice_packet,
                        c,
                        uv,
                        lv,
                        Read {
                            point: false,
                            invert: true,
                            named: false,
                        },
                    )
                }),
            ),
            (
                "(l) A, level 0, point clamped",
                Box::new(|c, uv, lv| {
                    alpha_at(
                        &choice_packet,
                        c,
                        uv,
                        lv,
                        Read {
                            point: true,
                            invert: false,
                            named: false,
                        },
                    )
                }),
            ),
            (
                "(m) 1 − A, level 0, point clamped",
                Box::new(|c, uv, lv| {
                    alpha_at(
                        &choice_packet,
                        c,
                        uv,
                        lv,
                        Read {
                            point: true,
                            invert: true,
                            named: false,
                        },
                    )
                }),
            ),
            (
                "(n) A, the pass's level, bilinear",
                Box::new(|c, uv, lv| {
                    alpha_at(
                        &choice_packet,
                        c,
                        uv,
                        lv,
                        Read {
                            point: false,
                            invert: false,
                            named: true,
                        },
                    )
                }),
            ),
            (
                "(o) A, the pass's level, point clamped (the pass as read)",
                Box::new(|c, uv, lv| {
                    alpha_at(
                        &choice_packet,
                        c,
                        uv,
                        lv,
                        Read {
                            point: true,
                            invert: false,
                            named: true,
                        },
                    )
                }),
            ),
            (
                "(p) 1 − A, the pass's level, point clamped",
                Box::new(|c, uv, lv| {
                    alpha_at(
                        &choice_packet,
                        c,
                        uv,
                        lv,
                        Read {
                            point: true,
                            invert: true,
                            named: true,
                        },
                    )
                }),
            ),
        ];
        // The candidates whose patch layer is our cells' base (item 4139).
        let ours_layer = |k: usize| k == 7;
        // Step 1's table is (a)-(d) alone; (e) is counted against (a)-(d),
        // and (f), its control, against (a)-(d) in (e)'s place.
        let mut pools: Vec<(&str, Vec<usize>)> = vec![
            ("step 1, (a)-(d)", vec![0, 1, 2, 3]),
            ("(a)-(e)", vec![0, 1, 2, 3, 4]),
            ("(a)-(d), (f)", vec![0, 1, 2, 3, 5]),
            ("(a)-(d), (g)", vec![0, 1, 2, 3, 6]),
            ("(a)-(d), (h)", vec![0, 1, 2, 3, 7]),
            ("(a)-(d), (i)", vec![0, 1, 2, 3, 8]),
        ];
        // Stage 2's table (item 4139): each read of the alpha, on our choice
        // on the packet's cells, against (a)-(d).
        let stage2: Vec<usize> = (9..candidates.len()).collect();
        // The level the pass names (item 4139): stage 1's `MIP_POINT` with
        // `mipmap_bias` −1.5, the nearest level to λ − 1.5, λ the log2 of
        // the alpha's texels per pixel side: the cell's uv per pixel (the
        // base's texcoord across neighbouring pixels of the tile) times the
        // 64 alpha texels a cell spans (a quadrant of 128).
        let mut levels: BTreeMap<(i32, i32), (f32, usize)> = BTreeMap::new();
        for &tt in &patched {
            let px = &tiles[&tt];
            let at: BTreeMap<(i32, i32), [f32; 2]> = px.iter().map(|q| (q.xy, q.uv)).collect();
            let mut dets: Vec<f32> = px
                .iter()
                .filter_map(|q| {
                    let (x, y) = q.xy;
                    let (r, d) = (at.get(&(x + 1, y))?, at.get(&(x, y + 1))?);
                    let (du, dv) = (
                        [r[0] - q.uv[0], r[1] - q.uv[1]],
                        [d[0] - q.uv[0], d[1] - q.uv[1]],
                    );
                    Some((du[0] * dv[1] - du[1] * dv[0]).abs())
                })
                .collect();
            dets.sort_by(f32::total_cmp);
            let det = dets.get(dets.len() / 2).copied().unwrap_or(0.0);
            let lambda = if det > 0.0 {
                0.5 * (det * 64.0 * 64.0).log2()
            } else {
                0.0
            };
            let level = (lambda - 1.5).round().clamp(0.0, 3.0) as usize;
            levels.insert(tt, (lambda, level));
        }
        let mut by_level: BTreeMap<usize, u32> = BTreeMap::new();
        for (_, lv) in levels.values() {
            *by_level.entry(*lv).or_default() += 1;
        }
        let lambdas: Vec<f32> = levels.values().map(|q| q.0).collect();
        println!(
            "the pass's level by tile {by_level:?}; λ min {:.2} max {:.2}",
            lambdas.iter().copied().fold(f32::MAX, f32::min),
            lambdas.iter().copied().fold(f32::MIN, f32::max)
        );
        for &k in &stage2 {
            pools.push((candidates[k].0, vec![0, 1, 2, 3, k]));
        }
        let r_of =
            |px: &[PatchPixel], c: (i32, i32), lv: usize, k: usize, wt: &Weight, wrong: bool| {
                let v: Vec<(f32, f32)> = px
                    .iter()
                    .map(|q| {
                        let a = wt(c, q.uv, lv);
                        let (pt, wpt) = if ours_layer(k) {
                            (q.opatch, q.wopatch)
                        } else {
                            (q.patch, q.wpatch)
                        };
                        let d = if wrong {
                            mix(q.wbase, wpt, a)
                        } else {
                            mix(q.base, pt, a)
                        };
                        (lum(q.photo), lum(d))
                    })
                    .collect();
                pearson(&v)
            };
        let nc = candidates.len();
        let mut rs: BTreeMap<(i32, i32), (Vec<f32>, Vec<f32>)> = BTreeMap::new();
        for &tt in &patched {
            let c = cell(tt);
            let px = &tiles[&tt];
            let r: Vec<f32> = candidates
                .iter()
                .enumerate()
                .map(|(k, (_, wt))| r_of(px, c, levels[&tt].1, k, wt, false))
                .collect();
            let rw: Vec<f32> = candidates
                .iter()
                .enumerate()
                .map(|(k, (_, wt))| r_of(px, c, levels[&tt].1, k, wt, true))
                .collect();
            rs.insert(tt, (r, rw));
        }
        let mut blend = (0, [0u32; 4], 0);
        let mut step1 = [0u32; 4];
        let mut ours: [(u32, [u32; 4], Tiles); 2] = Default::default();
        let mut off = 0;
        let mut stage2_wins: Vec<(usize, u32, [u32; 4])> = Vec::new();
        for (pool_name, pool) in &pools {
            let mut wins = vec![0u32; nc];
            let mut wins_by: BTreeMap<(usize, u8, i32), u32> = BTreeMap::new();
            let mut none = 0;
            for &tt in &patched {
                let c = cell(tt);
                let (r, rw) = &rs[&tt];
                let best = *pool
                    .iter()
                    .max_by(|&&a, &&b| r[a].total_cmp(&r[b]))
                    .expect("one");
                let column = patch_of(c).map_or(0, |li| li / 9 - i32::from(sub(c)));
                if r[best] > rw[best] {
                    wins[best] += 1;
                    *wins_by.entry((best, sub(c), column)).or_default() += 1;
                } else {
                    none += 1;
                }
            }
            let names: Vec<String> = pool
                .iter()
                .map(|&k| format!("{} {}", &candidates[k].0[..3], wins[k]))
                .collect();
            println!(
                "{pool_name}: wins {names:?}, none above its control {none}, of {}",
                patched.len()
            );
            println!("  by (candidate, land_sub, the patch's base less ours): {wins_by:?}");
            if pool.len() == 4 {
                step1 = [wins[0], wins[1], wins[2], wins[3]];
            }
            if pool.len() == 5 && pool[4] == 4 {
                blend = (wins[4], [wins[0], wins[1], wins[2], wins[3]], blend.2);
            }
            if pool.len() == 5 && pool[4] == 5 {
                blend.2 = wins[5];
            }
            if pool.len() == 5 && pool[4] == 8 {
                off = wins[8];
            }
            if pool.len() == 5 && pool[4] >= 9 {
                stage2_wins.push((pool[4], wins[pool[4]], [wins[0], wins[1], wins[2], wins[3]]));
            }
            if pool.len() == 5 && (6..8).contains(&pool[4]) {
                let k = pool[4];
                ours[k - 6] = (wins[k], [wins[0], wins[1], wins[2], wins[3]], Vec::new());
            }
        }
        for (k, (name, _)) in candidates.iter().enumerate() {
            let m: f64 =
                rs.values().map(|q| f64::from(q.0[k])).sum::<f64>() / rs.len().max(1) as f64;
            println!("{name}: mean r {m:.3}");
        }
        println!(
            "RUN1559B_3279_GROUND_BLEND (packet's patch) {blend:?} of {}",
            patched.len()
        );
        // Whether candidate `cand` wins tile `tt` against (a)-(d).
        let won = |tt: (i32, i32), cand: usize| {
            let (r, rw) = &rs[&tt];
            let best = *[0, 1, 2, 3, cand]
                .iter()
                .max_by(|&&a, &&b| r[a].total_cmp(&r[b]))
                .expect("one");
            best == cand && r[cand] > rw[cand]
        };
        let moved = |from: usize, to: usize| -> (Tiles, Tiles) {
            (
                patched
                    .iter()
                    .copied()
                    .filter(|&tt| won(tt, to) && !won(tt, from))
                    .collect(),
                patched
                    .iter()
                    .copied()
                    .filter(|&tt| won(tt, from) && !won(tt, to))
                    .collect(),
            )
        };
        for (k, from, (n, ctl, _)) in [(0, 4, &ours[0]), (1, 6, &ours[1])] {
            let (gained, lost) = moved(from, 6 + k);
            println!(
                "RUN1559B_3279_GROUND_BLEND ({}) {n} with {ctl:?}; against {}: gained {} {gained:?}, lost {} {lost:?}",
                ["our choice, packet's cells", "our choice, our cells"][k],
                &candidates[from].0[..3],
                gained.len(),
                lost.len()
            );
        }
        // Stage 3 (item 4139): the tiles the packet's own patch loses, under
        // 4132's read (e) and under the pass as read (g), by alpha, by
        // land_sub, by the patch's base less ours, by which control wins, by
        // the tile's place in its cell and by its scored pixels.
        for (k, name) in [(4, "(e), 4132's read"), (6, "(g), the pass as read")] {
            let lost: Vec<(i32, i32)> = patched.iter().copied().filter(|&tt| !won(tt, k)).collect();
            let mut by: [BTreeMap<String, u32>; 6] = Default::default();
            for &tt in &lost {
                let c = cell(tt);
                let alpha =
                    choice_packet[(c.1 * t.width + c.0) as usize].map_or(-1, |q| q.alpha as i32);
                let (r, rw) = &rs[&tt];
                let best = *[0, 1, 2, 3, k]
                    .iter()
                    .max_by(|&&a, &&b| r[a].total_cmp(&r[b]))
                    .expect("one");
                let winner = if best != k && r[best] > rw[best] {
                    candidates[best].0[..3].to_string()
                } else {
                    "none above its control".to_string()
                };
                let edge = |q: i32| q == 0 || q == 3;
                let (ex, ey) = (edge(tt.0.rem_euclid(4)), edge(tt.1.rem_euclid(4)));
                let place = match (ex, ey) {
                    (true, true) => "corner",
                    (false, false) => "interior",
                    _ => "side",
                };
                let n = tiles[&tt].len();
                let size = if n < 40 {
                    "<40"
                } else if n < 80 {
                    "40-79"
                } else {
                    ">=80"
                };
                let column = patch_of(c).map_or(0, |li| li / 9 - i32::from(sub(c)));
                for (m, key) in [
                    format!("{alpha:02}"),
                    format!("{}", sub(c)),
                    format!("{column}"),
                    winner,
                    place.to_string(),
                    size.to_string(),
                ]
                .into_iter()
                .enumerate()
                {
                    *by[m].entry(key).or_default() += 1;
                }
            }
            println!(
                "stage 3, the tiles {name} loses: {} of {}",
                lost.len(),
                patched.len()
            );
            for (m, label) in [
                "alpha",
                "land_sub",
                "patch base less ours",
                "won by",
                "place in cell",
                "scored pixels",
            ]
            .iter()
            .enumerate()
            {
                println!("  by {label}: {:?}", by[m]);
            }
        }
        // The drawn patches alone (item 4139): the tiles whose cell our
        // choice patches on the packet's cells. On the rest (the emptied
        // patches, `landindex` 0 in 4129's list) every alpha read draws the
        // base alone, equal to (a), and `max_by` gives the tie to the later
        // candidate, so a read "wins" there by its index.
        let drawn: Vec<(i32, i32)> = patched
            .iter()
            .copied()
            .filter(|&tt| {
                let c = cell(tt);
                choice_packet[(c.1 * t.width + c.0) as usize].is_some()
            })
            .collect();
        let ties = patched
            .iter()
            .filter(|tt| !drawn.contains(tt))
            .filter(|&&tt| {
                let r = &rs[&tt].0;
                r[4] == r[0] && r[6] == r[0]
            })
            .count();
        let control_wins = |k: usize| {
            drawn
                .iter()
                .filter(|&&tt| {
                    let (r, rw) = &rs[&tt];
                    let best = *[0, 1, 2, 3]
                        .iter()
                        .max_by(|&&a, &&b| r[a].total_cmp(&r[b]))
                        .expect("one");
                    best == k && r[k] > rw[k]
                })
                .count() as u32
        };
        let on_drawn = |k: usize| drawn.iter().filter(|&&tt| won(tt, k)).count() as u32;
        let drawn_row = (
            on_drawn(4),
            on_drawn(6),
            [0, 1, 2, 3].map(control_wins),
            drawn.len() as u32,
        );
        println!(
            "the drawn patches alone: (e) {}, (g) {}, the controls (a)-(d) in their own pool {:?}, of {}; the undrawn {}, ties with (a) under (e) and (g) {ties}",
            drawn_row.0,
            drawn_row.1,
            drawn_row.2,
            drawn_row.3,
            patched.len() - drawn.len()
        );
        assert_eq!(
            ties,
            patched.len() - drawn.len(),
            "every undrawn tile is a tie"
        );
        assert_eq!(
            drawn_row, RUN1559B_3279_GROUND_BLEND_DRAWN,
            "RUN1559B_3279_GROUND_BLEND_DRAWN"
        );
        // The same tables over all 503, for the rates.
        let mut all_by: [BTreeMap<String, u32>; 2] = Default::default();
        for &tt in &patched {
            let c = cell(tt);
            let alpha =
                choice_packet[(c.1 * t.width + c.0) as usize].map_or(-1, |q| q.alpha as i32);
            *all_by[0].entry(format!("{alpha:02}")).or_default() += 1;
            *all_by[1].entry(format!("{}", sub(c))).or_default() += 1;
        }
        println!(
            "  all 503 by alpha: {:?}; by land_sub: {:?}",
            all_by[0], all_by[1]
        );
        let on_parting: Vec<(i32, i32)> = patched
            .iter()
            .copied()
            .filter(|&tt| parting_cells.contains(&cell(tt)))
            .collect();
        println!(
            "cells where our cells or our choice part from the packet's {}; scored patched tiles on them {}: {on_parting:?}",
            parting_cells.len(),
            on_parting.len()
        );
        println!("(i) the seed one on: {off} of {}", patched.len());
        println!("stage 2, our choice on the packet's cells, each read against (a)-(d):");
        for (k, n, ctl) in &stage2_wins {
            println!("  {}: {n} with {ctl:?}", candidates[*k].0);
        }
        let reads: Vec<u32> = stage2_wins.iter().map(|q| q.1).collect();
        assert_eq!(
            (reads.as_slice(), by_level.get(&0).copied()),
            (
                RUN1559B_3279_PATCH_READS.0.as_slice(),
                Some(RUN1559B_3279_PATCH_READS.1)
            ),
            "RUN1559B_3279_PATCH_READS"
        );
        assert!(off < ours[0].0, "the draw follows our choice");
        assert_eq!(
            (
                ours[0].0,
                ours[0].1,
                ours[1].0,
                ours[1].1,
                patched.len() as u32
            ),
            RUN1559B_3279_GROUND_BLEND,
            "RUN1559B_3279_GROUND_BLEND"
        );
        assert_eq!((step1, patched.len() as u32), RUN1559B_3279_PATCH_CONTROLS);
        assert_eq!(
            (blend.0, blend.1, blend.2, patched.len() as u32),
            RUN1559B_3279_GROUND_BLEND_PACKET
        );
        assert!(
            blend.1.iter().all(|&n| n < blend.0) && blend.2 < blend.0,
            "the read alpha beats every control and its quarter turn"
        );
    }

    /// **Stage 0 of item 4141** (`docs/RENDERER.md` §3.10): the scored
    /// tiles under a placed mountain, by range and template, with the
    /// photograph's mean colour beside each candidate's: the debug grey,
    /// the ground §3.9 draws as if no mountain stood there, and the means of
    /// the tileset's `<MOUNTAINS>` `BASETEX` and `ALPHATEX`. Each candidate's
    /// agreement under the row's bar, and the nearest by tile, a tie a tie.
    #[test]
    fn a_mountain_s_ground_its_values() {
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
        let tex = GroundTextures::untinted(&p.root, &tileset).expect("the ground's textures load");
        let mesh = p.world.terrain().clone();
        let t = p.world.runner.terrain();
        let mountains: std::collections::BTreeSet<(i32, i32)> =
            t.mountains.iter().flatten().map(|q| (q.x, q.y)).collect();
        // Each range's template: the one whose tiles, laid at a cell, are the
        // range's tiles in the template's own order.
        let install = rondata::Install::new(&p.root);
        let templates = rondata::mountains::templates(&install);
        let range_of: Vec<(usize, (i32, i32))> = t
            .mountains
            .iter()
            .map(|r| {
                templates
                    .iter()
                    .enumerate()
                    .find_map(|(k, tp)| {
                        let (ox, oy) = *tp.tiles.first()?;
                        let (lx, ly) = (r.first()?.x - ox, r.first()?.y - oy);
                        (lx % TILES_PER_CELL == 0
                            && ly % TILES_PER_CELL == 0
                            && tp.tiles.len() == r.len()
                            && tp
                                .tiles
                                .iter()
                                .zip(r)
                                .all(|(o, q)| (lx + o.0, ly + o.1) == (q.x, q.y)))
                        .then_some((k, (lx / TILES_PER_CELL, ly / TILES_PER_CELL)))
                    })
                    .expect("every range is a template laid at a cell")
            })
            .collect();
        let mut range_at: BTreeMap<(i32, i32), usize> = BTreeMap::new();
        for (i, r) in t.mountains.iter().enumerate() {
            for q in r {
                range_at.entry((q.x, q.y)).or_insert(i);
            }
        }
        // The ground as if no mountain stood: §3.9's base and patch.
        let none = std::collections::BTreeSet::new();
        let base = ground_base(&t, &none, &tex, &mesh);
        let alphas = load_alphas(&p.root).expect("the alphas load");
        let codes: Vec<[u32; 8]> = alphas.iter().map(|a| a.1).collect();
        let Some(seed) = crate::patch::tests::world_seed() else {
            return;
        };
        let choice = painter_choice(
            &t,
            our_patches(&t, &p.root, &tileset, seed, &codes).expect("the tileset"),
        );
        let patch = ground_patch(&t, &none, &tex, &mesh, &choice);
        let flat = tex.flat();
        let mut window = Rect::new(i32::MAX, i32::MAX, 0, 0);
        for (i, tt) in g.tile.iter().enumerate() {
            if tt.is_some_and(|tt| mountains.contains(&tt)) {
                let (x, y) = ((i % w) as i32, (i / w) as i32);
                window = Rect::new(
                    window.x0.min(x),
                    window.y0.min(y),
                    window.x1.max(x + 1),
                    window.y1.max(y + 1),
                );
            }
        }
        let laid_base = paint(
            &Object {
                key: 1,
                tris: base.tris.clone(),
            },
            &Paint::Textures {
                uvs: &base.uvs,
                mips: &flat,
                of: &base.of,
            },
            &p.camera,
            size,
            window,
        );
        let laid_patch = paint(
            &Object {
                key: 1,
                tris: patch.tris.clone(),
            },
            &Paint::Textures {
                uvs: &patch.uvs,
                mips: &flat,
                of: &patch.of,
            },
            &p.camera,
            size,
            window,
        );
        let ww = (window.x1 - window.x0) as usize;
        // Stage 1: each mountain tile's two triangles, their corners in the
        // range's template pixels (`4 · (tile − 4 · loc) + 128`).
        let (tw, _) = (t.width * TILES_PER_CELL, t.height * TILES_PER_CELL);
        let mut m_tris = Vec::new();
        let mut m_uvs = Vec::new();
        let mut m_tile = Vec::new();
        for (tt, &r) in &range_at {
            let (_, (lx, ly)) = range_of[r];
            let (x, y) = (
                4.0 * (tt.0 - TILES_PER_CELL * lx) as f32 + 128.0,
                4.0 * (tt.1 - TILES_PER_CELL * ly) as f32 + 128.0,
            );
            let (a, b, c, d) = ([x, y], [x + 4.0, y], [x + 4.0, y + 4.0], [x, y + 4.0]);
            let i = ((tt.1 * tw + tt.0) * 6) as usize;
            for (j, uv) in [[a, b, d], [b, c, d]].into_iter().enumerate() {
                let v = &mesh.positions[i + j * 3..i + j * 3 + 3];
                m_tris.push([v[0], v[1], v[2]]);
                m_uvs.push(uv);
                m_tile.push(*tt);
            }
        }
        let load = |rel: &str| {
            let path = rondata::artdata::resolve_art(&p.root, rel).expect("the file");
            Mips::new(render::tga::decode(&std::fs::read(path).expect("read")).expect("decodes"))
        };
        let dir = r".\terrain art\tilesets\common\common_mountains\";
        let plain_tex = load(&format!("{dir}t_mountain_plain.tga"));
        let blend_tex = load(&format!("{dir}t_mountain_blend.tga"));
        let files = rondata::mountains::template_files(&install);
        let alpha_files = |k: usize| {
            let f = files[k].replace("_disp_", "_mountain_shad_");
            (load(&f), load(&files[k].replace("_disp_", "_ring_shad_")))
        };
        let shads: BTreeMap<usize, (Mips, Mips)> =
            range_of.iter().map(|&(k, _)| (k, alpha_files(k))).collect();
        // The lit bounding box of each template's image (`init`'s rect).
        let boxes: BTreeMap<usize, [f32; 4]> = range_of
            .iter()
            .map(|&(k, _)| {
                let img = rondata::mountains::template_image(&p.root, &files[k])
                    .expect("image")
                    .0;
                let (mut x0, mut y0, mut x1, mut y1) = (img.width, img.height, 0, 0);
                for y in 0..img.height {
                    for x in 0..img.width {
                        if img.alpha(x, y) != 0 {
                            (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
                        }
                    }
                }
                (k, [x0 as f32, y0 as f32, x1 as f32, y1 as f32])
            })
            .collect();
        println!("template boxes {boxes:?}");
        let flat_m = vec![plain_tex.clone()];
        let m_of = vec![0usize; m_tris.len()];
        let laid_m = paint(
            &Object {
                key: 1,
                tris: m_tris.clone(),
            },
            &Paint::Textures {
                uvs: &m_uvs,
                mips: &flat_m,
                of: &m_of,
            },
            &p.camera,
            size,
            window,
        );
        let clamped = |m: &Mips, u: f32, v: f32| {
            let im = &m.levels[0];
            let (hu, hv) = (0.5 / im.width as f32, 0.5 / im.height as f32);
            m.sample(0, u.clamp(hu, 1.0 - hu), v.clamp(hv, 1.0 - hv))
        };
        // The reads: (name, ring drawn, ADDSIGNED, 1 − A, stretched, over the ground).
        let reads: [(&str, bool, bool, bool, bool, bool); 6] = [
            ("as read", true, true, false, true, true),
            ("main alone", false, true, false, true, true),
            ("1 - A", true, true, true, true, true),
            ("stage 1 colour ignored", true, false, false, true, true),
            ("alpha unstretched", true, true, false, false, true),
            ("over the grey", true, true, false, true, false),
        ];
        let mut m_px: BTreeMap<(i32, i32), Vec<Vec<[f32; 3]>>> = BTreeMap::new();
        type Px = Vec<[f32; 3]>;
        let mut tiles: BTreeMap<(i32, i32), (Px, Px)> = BTreeMap::new();
        for (i, tt) in g.tile.iter().enumerate() {
            let Some(tt) = *tt else { continue };
            if !mountains.contains(&tt) {
                continue;
            }
            {
                let (x, y) = ((i % w) as i32, (i / w) as i32);
                let j = (y - window.y0) as usize * ww + (x - window.x0) as usize;
                let ground_c = {
                    let c0 = laid_base[j]
                        .filter(|q| base.tile[q.tri as usize] == tt)
                        .map(|q| q.rgb);
                    c0.map(|c0| {
                        laid_patch[j]
                            .filter(|q| patch.tile[q.tri as usize] == tt)
                            .map_or(c0, |q| {
                                let k = q.tri as usize;
                                let a =
                                    patch_alpha(&alphas[patch.alpha[k]].0, &patch.patch[k], q.uv);
                                [0, 1, 2].map(|i| c0[i] * (1.0 - a) + q.rgb[i] * a)
                            })
                    })
                };
                if let (Some(q), Some(gc)) =
                    (laid_m[j].filter(|q| m_tile[q.tri as usize] == tt), ground_c)
                {
                    let k = range_of[range_at[&tt]].0;
                    let (shad, ring) = &shads[&k];
                    let bx = boxes[&k];
                    let (px, py) = (q.uv[0], q.uv[1]);
                    let e = m_px
                        .entry(tt)
                        .or_insert_with(|| vec![Vec::new(); reads.len()]);
                    for (n, &(_, ring_on, add, inv, stretch, over)) in reads.iter().enumerate() {
                        let (au, av) = if stretch {
                            (
                                (px - bx[0]) / (bx[2] - bx[0]),
                                (py - bx[1]) / (bx[3] - bx[1]),
                            )
                        } else {
                            (px / 256.0, py / 256.0)
                        };
                        let mut f = if over {
                            gc
                        } else {
                            render::terrain::MOUNTAIN.map(|v| v * 255.0)
                        };
                        let layers: Vec<(&Mips, &Mips)> = if ring_on {
                            vec![(&blend_tex, ring), (&plain_tex, shad)]
                        } else {
                            vec![(&plain_tex, shad)]
                        };
                        for (t0m, t1m) in layers {
                            let t0 = t0m.sample(0, px / 16.0, py / 16.0);
                            let t1 = clamped(t1m, au, av);
                            let rgb = [0, 1, 2].map(|c| {
                                if add {
                                    (t0[c] + t1[c] - 127.5).clamp(0.0, 255.0)
                                } else {
                                    t0[c]
                                }
                            });
                            let mut a = t0[3] * t1[3] / (255.0 * 255.0);
                            if a <= 0.0 {
                                continue;
                            }
                            if inv {
                                a = 1.0 - a;
                            }
                            f = [0, 1, 2].map(|c| rgb[c] * a + f[c] * (1.0 - a));
                        }
                        e[n].push(f);
                    }
                }
            }
            let (x, y) = ((i % w) as i32, (i / w) as i32);
            let j = (y - window.y0) as usize * ww + (x - window.x0) as usize;
            let e = tiles.entry(tt).or_default();
            e.0.push([0, 1, 2].map(|c| f32::from(p.photo.rgba[i * 4 + c])));
            let Some(c0) = laid_base[j]
                .filter(|q| base.tile[q.tri as usize] == tt)
                .map(|q| q.rgb)
            else {
                continue;
            };
            let c = laid_patch[j]
                .filter(|q| patch.tile[q.tri as usize] == tt)
                .map_or(c0, |q| {
                    let k = q.tri as usize;
                    let a = patch_alpha(&alphas[patch.alpha[k]].0, &patch.patch[k], q.uv);
                    [0, 1, 2].map(|i| c0[i] * (1.0 - a) + q.rgb[i] * a)
                });
            e.1.push(c);
        }
        let texture_mean = |rel: &str| {
            let path = rondata::artdata::resolve_art(&p.root, rel).expect("the file");
            let img = render::tga::decode(&std::fs::read(path).expect("read")).expect("decodes");
            let n = (img.width * img.height) as f32;
            let m = [0, 1, 2].map(|c| img.rgba.chunks(4).map(|q| f32::from(q[c])).sum::<f32>() / n);
            println!(
                "{rel}: {}×{} bits {} mean {m:?}",
                img.width, img.height, img.bits
            );
            m
        };
        let plain =
            texture_mean(r".\terrain art\tilesets\common\common_mountains\t_mountain_plain.tga");
        let blend =
            texture_mean(r".\terrain art\tilesets\common\common_mountains\t_mountain_blend.tga");
        let names = ["grey", "ground", "plain", "blend"];
        let mut agree = [0u32; 4];
        let mut nearest = [0u32; 4];
        let mut ties = 0u32;
        let mut by_template: BTreeMap<usize, (u32, [u32; 4])> = BTreeMap::new();
        let mut scored = 0u32;
        println!("tile\trange\ttemplate\tn\tphoto\tgrey\tground\tplain\tblend");
        for (tt, (th, us)) in &tiles {
            if (th.len() as u32) < HUE.min_pixels {
                continue;
            }
            scored += 1;
            let them = chromaticity(mean(th));
            let ground = (!us.is_empty()).then(|| chromaticity(mean(us)));
            let cands = [
                Some(chromaticity(render::terrain::MOUNTAIN)),
                ground,
                Some(chromaticity(plain)),
                Some(chromaticity(blend)),
            ];
            let d: Vec<f32> = cands
                .iter()
                .map(|c| c.map_or(f32::INFINITY, |c| dist(c, them)))
                .collect();
            let r = range_at[tt];
            let e = by_template.entry(range_of[r].0).or_default();
            e.0 += 1;
            for k in 0..4 {
                if d[k] <= HUE.distance {
                    agree[k] += 1;
                    e.1[k] += 1;
                }
            }
            let best = d.iter().cloned().fold(f32::INFINITY, f32::min);
            let at: Vec<usize> = (0..4).filter(|&k| d[k] == best).collect();
            if at.len() > 1 {
                ties += 1;
            } else {
                nearest[at[0]] += 1;
            }
            println!(
                "{},{}\t{r}\t{}\t{}\t{them:.3?}\t{:.3}\t{:.3}\t{:.3}\t{:.3}",
                tt.0,
                tt.1,
                range_of[r].0,
                th.len(),
                d[0],
                d[1],
                d[2],
                d[3]
            );
        }
        let mut read_agree = vec![0u32; reads.len()];
        let mut read_beats = vec![(0u32, 0u32); reads.len()];
        for (tt, (th, _)) in &tiles {
            if (th.len() as u32) < HUE.min_pixels {
                continue;
            }
            let them = chromaticity(mean(th));
            let us = &tiles[tt].1;
            let ground_d = (!us.is_empty()).then(|| dist(chromaticity(mean(us)), them));
            for (n, px) in m_px
                .get(tt)
                .map(|v| v.as_slice())
                .unwrap_or(&[])
                .iter()
                .enumerate()
            {
                if px.is_empty() {
                    continue;
                }
                let d = dist(chromaticity(mean(px)), them);
                if d <= HUE.distance {
                    read_agree[n] += 1;
                }
                // Against the best control, the ground: (strictly nearer, tied).
                if let Some(gd) = ground_d {
                    if d < gd {
                        read_beats[n].0 += 1;
                    } else if d == gd {
                        read_beats[n].1 += 1;
                    }
                }
            }
        }
        for (n, r) in reads.iter().enumerate() {
            println!(
                "READ {:?}: agree {} of {scored}; nearer than the ground {}, tied {}",
                r.0, read_agree[n], read_beats[n].0, read_beats[n].1
            );
        }
        println!("ranges (template, loc): {range_of:?}");
        println!("by template (scored, agree {names:?}): {by_template:?}");
        println!(
            "A_MOUNTAIN_S_VALUES: {scored} scored; agree {agree:?}; nearest {nearest:?}, ties {ties}"
        );
        assert_eq!(
            (scored, agree, nearest, ties),
            RUN1559B_3279_MOUNTAIN_VALUES
        );
        let reads_got = (
            read_agree.clone().try_into().expect("six reads"),
            read_beats
                .iter()
                .map(|b| b.0)
                .collect::<Vec<_>>()
                .try_into()
                .expect("six"),
            scored,
        );
        assert_eq!(reads_got, RUN1559B_3279_MOUNTAIN_READS);
        assert!(
            read_beats.iter().all(|b| b.1 == 0),
            "no read ties the ground"
        );
        assert!(
            read_agree[0] > agree.iter().copied().max().unwrap_or(0),
            "the read beats every stage 0 candidate"
        );
    }
}
