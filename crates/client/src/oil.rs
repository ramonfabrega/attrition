//! **The land oil's terrain patches** (item 4309; `docs/RENDERER.md` §3.24,
//! "The land's oil").
//!
//! `TerrainOil::create_doobers@008a28e0` gives a cell an `OilPatchData` at
//! load, row by row, and `TerrainOil::add_rocky_patch@008a2b30` fills its two
//! lists from `internal_random`:
//! - `rock_patches_rough` (`+0x1c`): the four 2 × 2 tiles that cover the
//!   cell, then one at `(4x + b, 4y + a)`, `a` and `b` draws on `[0, 3)`;
//! - `rock_patches_always` (`+0x0`): nine tries of `(4x − 2 + c, 4y − 2 + d)`,
//!   `c` and `d` draws on `[0, 7)`, each kept when its 2 × 2 lies on the map
//!   and the tile is not already in the first list (`add_unique`).
//!
//! `TerrainOil::add_wcoord_for_render@008a26f0` lays them through
//! `TerrainOut::add_weather_underlay@00874d00` on every drawn cell whose
//! patch has `is_vis` (`+0x39`, 1 at load):
//! - a 2 × 2 `rough` patch at each tile of the always list;
//! - on an `OIL` cell (`WData::flags & 0x800`) the console can use oil on,
//!   one 4 × 4 `oil` patch on the cell, unless the cell's middle tile is a
//!   building's (`TData::mask & 3 == 3`);
//! - otherwise a `rough` patch at each tile of the rough list.
//!
//! `TerrainOut::render_weathering(0, 3)` draws classes 2 and 3 as it draws
//! the snowcap's class 4 (`crate::snowcap`): the texture at both stages,
//! `TL_FAKE_LIGHT`, blended source alpha over.

use std::sync::Arc;

use render::silhouette::Mips;
use sim::world::cell;

/// `internal_random`'s draws before `create_doobers`' first
/// (`docs/RENDERER.md` §3.26, run1567's trace): `RiverDooberType::init`'s
/// 45, `random_map_pool`'s `(4W + 1)²`, and `RiverDooberType::init`'s 45
/// again. The two 45s are the GreatLakes map's.
pub fn draws_before(width_cells: i32) -> u32 {
    let w = (4 * width_cells + 1).max(0) as u32;
    crate::snowcap::DRAWS_BEFORE_POOL + w * w + 45
}

/// One cell's `OilPatchData`, its lists as tiles.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Patch {
    pub cell: (i32, i32),
    pub always: Vec<(i32, i32)>,
    pub rough: Vec<(i32, i32)>,
}

/// **The cells `create_doobers` gives a patch**, row by row: a cell not on
/// water (`WData::land` 1 or 2 with `flags & 0x100` clear is water, and
/// there the `ROCK` bit is cleared instead) with `cell::ROCK`, or holding a
/// good the console's leader cannot yet use (`unusable`, the cell's `x`
/// and `y`). An `Oil` good stands in no cell's chain, so a land oil cell
/// takes its patch by its `ROCK` bit.
pub fn cells(
    terrain: &sim::observe::TerrainScene,
    unusable: &dyn Fn(i32, i32) -> bool,
) -> Vec<(i32, i32)> {
    let mut out = Vec::new();
    for y in 0..terrain.height {
        for x in 0..terrain.width {
            let i = (y * terrain.width + x) as usize;
            let flags = terrain.flags.get(i).copied().unwrap_or(0);
            let water = flags & cell::HALFLAND == 0 && matches!(terrain.land.get(i), Some(1 | 2));
            if !water && (flags & cell::ROCK != 0 || unusable(x, y)) {
                out.push((x, y));
            }
        }
    }
    out
}

/// **The cells holding a good the console cannot use at load**: a live
/// good other than `Oil` (which stands in no cell's chain, so
/// `ObjectsData::find_good_at` never answers it) whose type `usable` calls
/// unavailable, at its cell (`pos / 768`).
pub fn unusable_cells(
    goods: &[sim::world::Good],
    usable: &dyn Fn(usize) -> bool,
) -> std::collections::BTreeSet<(i32, i32)> {
    goods
        .iter()
        .filter(|g| g.alive && g.ty != crate::goods::OIL && !usable(g.ty))
        .map(|g| (g.pos.x.div_euclid(768), g.pos.y.div_euclid(768)))
        .collect()
}

/// **The map's patches as `create_doobers` builds them** off our terrain and
/// goods, `usable` the load's `type_avail`, drawing from `internal_random`
/// after [`draws_before`].
pub fn build(
    terrain: &sim::observe::TerrainScene,
    goods: &[sim::world::Good],
    usable: &dyn Fn(usize) -> bool,
) -> Vec<Patch> {
    let unusable = unusable_cells(goods, usable);
    let picked = cells(terrain, &|x, y| unusable.contains(&(x, y)));
    let tiles = (
        terrain.width * render::terrain::TILES_PER_CELL,
        terrain.height * render::terrain::TILES_PER_CELL,
    );
    let mut rng = crate::surf::InternalRandom::after(draws_before(terrain.width));
    patches(&picked, tiles, &mut rng)
}

/// **`add_rocky_patch`'s lists** for each of `cells` in order, drawing from
/// `rng`, on a map `tiles` wide and high.
pub fn patches(
    cells: &[(i32, i32)],
    tiles: (i32, i32),
    rng: &mut crate::surf::InternalRandom,
) -> Vec<Patch> {
    let on = |x: i32, y: i32| (0..tiles.0).contains(&x) && (0..tiles.1).contains(&y);
    cells
        .iter()
        .map(|&(cx, cy)| {
            let (x, y) = (4 * cx, 4 * cy);
            let mut rough = vec![(x, y), (x + 2, y), (x, y + 2), (x + 2, y + 2)];
            let a = rng.get(0, 3);
            let b = rng.get(0, 3);
            rough.push((x + b, y + a));
            let mut always = Vec::new();
            for _ in 0..9 {
                let tx = rng.get(0, 7) + x - 2;
                let ty = rng.get(0, 7) + y - 2;
                if on(tx, ty)
                    && on(tx + 1, ty + 1)
                    && !rough.contains(&(tx, ty))
                    && !always.contains(&(tx, ty))
                {
                    always.push((tx, ty));
                }
            }
            Patch {
                cell: (cx, cy),
                always,
                rough,
            }
        })
        .collect()
}

/// **What `add_wcoord_for_render` lays for one patch**: the corner tile of
/// each 2 × 2 `rough` patch, and whether the 4 × 4 `oil` patch is laid on
/// the cell. `oil_shown` is the console's `type_avail(OIL, 1)` or the
/// game's semaphore bit.
pub fn laid(
    p: &Patch,
    terrain: &sim::observe::TerrainScene,
    oil_shown: bool,
) -> (Vec<(i32, i32)>, bool) {
    let (cx, cy) = p.cell;
    let i = (cy * terrain.width + cx) as usize;
    let mut rough = p.always.clone();
    let flags = terrain.flags.get(i).copied().unwrap_or(0);
    if flags & cell::OIL != 0 && oil_shown {
        let tw = terrain.width * render::terrain::TILES_PER_CELL;
        let mid = ((4 * cy + 2) * tw + 4 * cx + 2) as usize;
        let mask = terrain.tiles.get(mid).copied().unwrap_or(0);
        let building = mask & sim::world::tile::OBJECT == sim::world::tile::OBJECT_BUILDING;
        return (rough, !building);
    }
    rough.extend_from_slice(&p.rough);
    (rough, false)
}

/// **`add_weather_underlay`'s texcoords** for a `side`-tile patch with
/// `variants` 0 (the `oil` patch), or 6 (the `rough` one, `param_8`), and
/// no odd-corner step (`param_10` 0): at the corner `(i, j)` tiles in, `u =
/// i / side · su + ou`, `v = j / side · sv + ov`, UV0 and UV1 alike. With
/// six variants `su` is ½ and `sv` ⅓, and the variant `random_map_pool` at
/// the corner tile picks `(ou, ov)`: 0 `(0, 0)`, 1 `(½, 0)`, 2 `(0, ⅓)`, 3
/// `(½, ⅓)`, 4 `(0, ⅔)`, 5 `(½, ⅔)` (the listing, `874ec0`–`874f23`, the
/// jump table at `8751f0`). Single precision, as the listing's.
pub fn uv(side: i32, variant: Option<i32>, (i, j): (i32, i32)) -> [f32; 2] {
    let (mut du, mut dv) = (1.0 / side as f32, 1.0 / side as f32);
    let (mut ou, mut ov) = (0.0f32, 0.0f32);
    if let Some(k) = variant {
        du *= 0.5;
        dv *= 1.0 / 3.0f32;
        ou = if k & 1 != 0 { 0.5 } else { 0.0 };
        ov = match k / 2 {
            1 => 1.0 / 3.0f32,
            2 => 2.0 / 3.0f32,
            _ => 0.0,
        };
    }
    [i as f32 * du + ou, j as f32 * dv + ov]
}

/// **The variant** `add_weather_underlay` takes for a six-variant patch at
/// `corner` (`874e21`–`874e75`): `random_map_pool[x · (4W + 1) + y] · 6 /
/// 100`, held under 6 − ½ (5 where it reaches it), truncated.
pub fn variant(pool: &[u8], tiles_wide: i32, corner: (i32, i32)) -> i32 {
    let stride = (tiles_wide + 1) as usize;
    pool.get(corner.0.max(0) as usize * stride + corner.1.max(0) as usize)
        .map_or(0, |&p| {
            let n = 6.0f32;
            let mut q = f32::from(p) * (n / 100.0);
            if n - 0.5 <= q {
                q = n - 1.0;
            }
            q as i32
        })
}

/// The `rough` and `oil` underlays' art, tinted as every underlay is
/// (`TileSet::init@0087b370`).
pub struct OilArt {
    pub rough: Arc<Mips>,
    pub oil: Arc<Mips>,
}

impl OilArt {
    /// `None` where the tileset names no `rough` or `oil` underlay or a
    /// texture does not read.
    pub fn load(install: &rondata::Install, tileset: &str, tint: [u8; 3]) -> Option<OilArt> {
        let (_, list) = rondata::artdata::underlay_textures(install, tileset).ok()?;
        let read = |id: &str| -> Option<Arc<Mips>> {
            let file = &list.iter().find(|(k, _)| k.eq_ignore_ascii_case(id))?.1;
            let bytes = std::fs::read(rondata::artdata::resolve_art(install.root(), file)?).ok()?;
            let mut img = render::tga::decode(&bytes).ok()?;
            img.tint(tint);
            Some(Arc::new(Mips::new(img)))
        };
        Some(OilArt {
            rough: read("rough")?,
            oil: read("oil")?,
        })
    }
}

/// One underlay patch to lay: its corner tile, side and variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Quad {
    pub corner: (i32, i32),
    pub side: i32,
    pub variant: Option<i32>,
}

/// **Every patch the map's oil lays**, `rough` first then `oil`, as
/// `render_weathering(0, 3)` draws class 2 before class 3.
pub fn quads(
    patches: &[Patch],
    terrain: &sim::observe::TerrainScene,
    oil_shown: bool,
    pool: &[u8],
) -> (Vec<Quad>, Vec<Quad>) {
    let tw = terrain.width * render::terrain::TILES_PER_CELL;
    let (mut rough, mut oil) = (Vec::new(), Vec::new());
    for p in patches {
        let (tiles, laid_oil) = laid(p, terrain, oil_shown);
        rough.extend(tiles.into_iter().map(|t| Quad {
            corner: t,
            side: 2,
            variant: Some(variant(pool, tw, t)),
        }));
        if laid_oil {
            oil.push(Quad {
                corner: (4 * p.cell.0, 4 * p.cell.1),
                side: 4,
                variant: None,
            });
        }
    }
    (rough, oil)
}

/// **One class's patches as a [`crate::world::PatchMesh`]**, on the
/// ground's own corners (`positions`, `[a, b, d]`, `[b, c, d]` per tile of a
/// `tiles`-wide map) with the land's N·L by `nl`, as the snowcap's.
pub fn mesh(
    piece: i32,
    alpha: usize,
    mips: &Arc<Mips>,
    quads: &[Quad],
    positions: &[[f32; 3]],
    tiles: (i32, i32),
    nl: &dyn Fn(&[f32; 3]) -> f32,
) -> Option<crate::world::PatchMesh> {
    let (tw, th) = tiles;
    if positions.len() != (tw * th * 6) as usize {
        return None;
    }
    let mut m = crate::world::PatchMesh {
        piece,
        maps: None,
        positions: Vec::new(),
        uvs: Vec::new(),
        alpha_uvs: Vec::new(),
        mips: mips.clone(),
        alpha,
        alpha_mips: mips.clone(),
        mountain: false,
        light: Vec::new(),
    };
    for q in quads {
        for j in 0..q.side {
            for i in 0..q.side {
                let (x, y) = (q.corner.0 + i, q.corner.1 + j);
                if !(0..tw).contains(&x) || !(0..th).contains(&y) {
                    continue;
                }
                let k = ((y * tw + x) * 6) as usize;
                let (pa, pb, pd, pc) = (
                    positions[k],
                    positions[k + 1],
                    positions[k + 2],
                    positions[k + 4],
                );
                let at = |di, dj| uv(q.side, q.variant, (i + di, j + dj));
                let (a, b, c, d) = (at(0, 0), at(1, 0), at(1, 1), at(0, 1));
                for (p, t) in [(pa, a), (pb, b), (pd, d), (pb, b), (pc, c), (pd, d)] {
                    m.positions.push(p);
                    m.uvs.push(t);
                    m.alpha_uvs.push(t);
                    m.light.push(nl(&p));
                }
            }
        }
    }
    (!m.positions.is_empty()).then_some(m)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A packet patch: its cell, `is_vis`, flags, middle byte and lists.
    struct Theirs {
        cell: (i32, i32),
        vis: u8,
        flags: u16,
        b138: u8,
        always: Vec<(i32, i32)>,
        rough: Vec<(i32, i32)>,
    }

    /// **run1564's `TerrainOil` at frame 3294** (`tools/render/oil_packet.py`
    /// on the packet). `None`, saying so, where the dump is absent.
    fn packet() -> Option<Vec<Theirs>> {
        let path = std::env::var_os("RON_OIL_RUN1564")
            .map(std::path::PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|h| {
                    std::path::PathBuf::from(h)
                        .join("ron-data/lab-experiments/2026-11-16-item-4309/run1564-oil.txt")
                })
            })?;
        let Ok(text) = std::fs::read_to_string(&path) else {
            println!("skipping: no oil dump at {}", path.display());
            return None;
        };
        let tiles = |w: &[&str]| -> Vec<(i32, i32)> {
            w[1..]
                .iter()
                .map(|t| {
                    let (a, b) = t.split_once(',').unwrap();
                    (a.parse().unwrap(), b.parse().unwrap())
                })
                .collect()
        };
        let mut out: Vec<Theirs> = Vec::new();
        for l in text.lines() {
            let w: Vec<&str> = l.split_whitespace().collect();
            match w.first().copied() {
                Some("cell") => out.push(Theirs {
                    cell: (w[1].parse().unwrap(), w[2].parse().unwrap()),
                    vis: w[6].parse().unwrap(),
                    flags: u16::from_str_radix(w[10].trim_start_matches("0x"), 16).unwrap(),
                    b138: u8::from_str_radix(w[12].trim_start_matches("0x"), 16).unwrap(),
                    always: Vec::new(),
                    rough: Vec::new(),
                }),
                Some("always") => out.last_mut().unwrap().always = tiles(&w),
                Some("rough") => out.last_mut().unwrap().rough = tiles(&w),
                _ => {}
            }
        }
        Some(out)
    }

    /// **run1564's oil patches are the packet's** (item 4319): the 18 cells
    /// `create_doobers` picks off our terrain and goods (16 by `ROCK`, the
    /// Rubber's and the Aluminum's by the load's `type_avail`), their flags
    /// and middle tiles, and every tile of both lists `add_rocky_patch`
    /// draws from `internal_random` at [`draws_before`]. Made to fail: one
    /// draw early, the lists part.
    #[test]
    fn the_oil_patches_are_the_packet_s() {
        use crate::ground::tests::{probe_at, run1564};
        let Some(theirs) = packet() else {
            return;
        };
        let Some(p) = probe_at(run1564()) else {
            return;
        };
        let t = p.world.runner.terrain();
        let goods = p.world.runner.goods();
        let runner = &p.world.runner;
        let ours = build(&t, &goods, &|ty| runner.type_avail_at_load(ty));
        let tw = t.width * render::terrain::TILES_PER_CELL;
        let mut agree = [0usize; 4];
        for th in &theirs {
            let i = (th.cell.1 * t.width + th.cell.0) as usize;
            let mid = ((4 * th.cell.1 + 2) * tw + 4 * th.cell.0 + 2) as usize;
            agree[0] += usize::from(t.flags[i] == th.flags && th.vis == 1);
            agree[1] += usize::from(t.tiles[mid] as u8 == th.b138);
            if let Some(o) = ours.iter().find(|o| o.cell == th.cell) {
                agree[2] += usize::from(o.rough == th.rough);
                agree[3] += usize::from(o.always == th.always);
                if o.always != th.always || o.rough != th.rough {
                    println!("4319: cell {:?} ours {o:?}", th.cell);
                }
            }
        }
        let cells_ours: Vec<_> = ours.iter().map(|o| o.cell).collect();
        let cells_theirs: Vec<_> = theirs.iter().map(|o| o.cell).collect();
        println!(
            "4319: cells ours {} theirs {}; flags, middle, rough, always agree {agree:?}",
            cells_ours.len(),
            cells_theirs.len()
        );
        assert_eq!(cells_ours, cells_theirs, "the cells, in order");
        assert_eq!(agree, [18; 4]);
        let picked: Vec<_> = ours.iter().map(|o| o.cell).collect();
        let tiles = (tw, t.height * render::terrain::TILES_PER_CELL);
        let mut early = crate::surf::InternalRandom::after(draws_before(t.width) - 1);
        let parted = patches(&picked, tiles, &mut early);
        let same = parted.iter().zip(&ours).filter(|(a, b)| a == b).count();
        assert!(same < 3, "one draw early still agrees on {same}");
    }
}
