//! **The paved base under a city** (item 4190; `docs/RENDERER.md` §3.16):
//! a building's footprint weathering, laid by
//! `WallOut::draw_weathering@0063dde0` through
//! `Terrain::add_weathering@00850120` and drawn by
//! `TerrainOut::render_weathering@00852f70` — not `RoadsOut`'s.

use sim::cities::build::{BuildDomain, BuildType, is_city};
use sim::movement::roads::connects_to_roads;
use sim::observe::BuildingScene;

/// A tile's world units (`div_3_table[pos >> 6]`, 192 a tile).
const UNITS_PER_TILE: i32 = render::terrain::UNITS_PER_TILE;

/// **Set A's step** (`__real_3e800000` at `0xb69484`): the base texture
/// repeats every four tiles.
pub const WEATHER_UV: f32 = 0.25;

/// `add_weathering`'s age gate: the owner's ages past this (`5 < age`).
pub const WEATHER_AGE: i32 = 5;

/// `add_weathering`'s side gate: one side past this (`2 < sx || 2 < sy`).
pub const WEATHER_SIDE: i32 = 2;

/// **One building's weathering** (§3.16): the corner tile
/// `WallData::tile_corner@00643440` answers and the `(sx, sy)`
/// `WallOut::draw_weathering` passes. It lays `(sx − 1) × (sy − 1)` tiles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Footprint {
    pub corner: (i32, i32),
    pub size: (i32, i32),
}

/// **Whether and where a building is weathered** (§3.16), as
/// `WallOut::draw_weathering@0063dde0` asks it of a building
/// `ObjectsOut::draw_weathering@00654a60` walks:
/// - none on a water building (`ObjectTypeData::domain` 1);
/// - the type's own `x_size`, `y_size` for a city (`is(VILLAGE)`) or a type
///   that `connects_to_roads`;
/// - else `size + 1`, for a non-gathering type in a city
///   (`BuildData::city >= 0`). The scene carries no `city`, so that arm
///   lays nothing here (parked 4190b);
/// - then `add_weathering`'s gate: the owner's ages past [`WEATHER_AGE`],
///   one side past [`WEATHER_SIDE`].
///
/// The corner is the building's tile less `size >> 1` on each axis, by
/// the type's own size whichever arm passed.
pub fn footprint(types: &[BuildType], b: &BuildingScene) -> Option<Footprint> {
    let t = b.ty?;
    let ty = types.get(t)?;
    if ty.domain() == BuildDomain::Water {
        return None;
    }
    if !(is_city(types, t) || connects_to_roads(types, t)) {
        return None;
    }
    let size = (ty.x_size, ty.y_size);
    if b.age <= WEATHER_AGE || !(size.0 > WEATHER_SIDE || size.1 > WEATHER_SIDE) {
        return None;
    }
    let tile = (b.pos.x / UNITS_PER_TILE, b.pos.y / UNITS_PER_TILE);
    Some(Footprint {
        corner: (tile.0 - (ty.x_size >> 1), tile.1 - (ty.y_size >> 1)),
        size,
    })
}

impl Footprint {
    /// The tiles it lays, `[cx, cx + sx − 1) × [cy, cy + sy − 1)`, row by
    /// row as `add_weathering` walks them.
    pub fn tiles(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        let (cx, cy) = self.corner;
        let (sx, sy) = self.size;
        (cy..cy + sy - 1).flat_map(move |y| (cx..cx + sx - 1).map(move |x| (x, y)))
    }

    /// **Set A, the base's texcoords** (UV0: `render_weathering` hands
    /// `Terrain +0x46e8` to the vertex buffer first) at the tile's corners
    /// `(x, y)`, `(x + 1, y)`, `(x + 1, y + 1)`, `(x, y + 1)`: `d · q`
    /// from the corner, `q` [`WEATHER_UV`]. The far corner's v is
    /// `dy · q + 1 / (sy − 1)`, not `+ q`: the listing adds `[ebp + 0x1c]`,
    /// the spent age argument's slot, where `1 / (sy − 1)` was kept for set
    /// B (`0x850321`, `0x850853`). The two agree only at `sy = 5`.
    pub fn uvs(&self, (x, y): (i32, i32)) -> [[f32; 2]; 4] {
        let q = WEATHER_UV;
        let u = (x - self.corner.0) as f32 * q;
        let v = (y - self.corner.1) as f32 * q;
        let far = 1.0 / (self.size.1 - 1) as f32;
        [[u, v], [u + q, v], [u + q, v + far], [u, v + q]]
    }
}

/// **The frame's weathering** (§3.16): every building's [`footprint`], in
/// the scene's store order.
pub fn footprints(types: &[BuildType], buildings: &[BuildingScene]) -> Vec<Footprint> {
    buildings
        .iter()
        .filter_map(|b| footprint(types, b))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::ground::tests::{Photo, Probe, ground_why, probe_at, run1559b};
    use render::silhouette::{HUE, LUMA_READER, chromaticity, luminance};
    use std::collections::{BTreeMap, BTreeSet};

    /// run1570's packet's `loc` (item 4183, `docs/RENDERER.md` §3.13):
    /// (3168, 30816), the capital's centre, as single bits.
    const RUN1570_LOC: (u32, u32) = (0x4546_0000, 0x46f0_c000);

    /// **run1570's photograph** (item 4183; `docs/RUNS.md` run1570):
    /// run1559b's take paused on 3331, 1920 × 1080 with no title bar, at
    /// the pinned camera with run1570's `loc`. The photographed frame is
    /// not clocked (4183b): it is drawn at 3316, run1559b's own offset
    /// from its packet's 3294 to the 3279 its movers show (assumed). The
    /// row's tiles lie under standing buildings; only the movers' masks
    /// follow the frame.
    pub(crate) fn run1570() -> Photo {
        let path = std::env::var_os("RON_PHOTO_RUN1570")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let home = std::env::var_os("HOME").unwrap_or_default();
                PathBuf::from(home)
                    .join("ron-data/lab-experiments/2026-11-12-item-4190/run1570-3331.tga")
            });
        let mut camera = crate::pinned_camera();
        camera.loc_x = f32::from_bits(RUN1570_LOC.0);
        camera.loc_y = f32::from_bits(RUN1570_LOC.1);
        Photo {
            path,
            camera,
            frame: RUN1570_FRAME,
            traced: &[],
        }
    }

    /// The frame run1570's photograph is drawn at ([`run1570`]).
    const RUN1570_FRAME: i64 = 3331 - (3294 - crate::silhouette::PHOTO_FRAME);

    /// One scored tile: the tile, the photograph's mean, the still's, and
    /// its pixel count.
    type PlazaTile = ((i32, i32), [f32; 3], [f32; 3], usize);

    /// **The row's pixels** (item 4190; §3.16, the rule committed before
    /// any count), each its tile: every tile the frame's weathering lays by
    /// [`super::footprints`] — unmoved, whether or not it is drawn — with
    /// at least `HUE.min_pixels` ground pixels on the photograph
    /// (`ground_why`'s ground or an object's window, as the shadows' row
    /// takes them); the photograph's mean and the still's over them.
    fn plaza_pixels(p: &mut Probe) -> Vec<Option<(i32, i32)>> {
        let scene = p.world.runner.scene(None);
        let laid: BTreeSet<(i32, i32)> =
            super::footprints(p.world.build_types_for_test(), &scene.buildings)
                .iter()
                .flat_map(|f| f.tiles().collect::<Vec<_>>())
                .collect();
        let (_, any, why) = ground_why(p);
        any.into_iter()
            .zip(why)
            .map(|(t, w)| t.filter(|t| laid.contains(t) && (w == 0 || w == 2)))
            .collect()
    }

    /// The photograph's mean and the still's over [`plaza_pixels`]'s
    /// tiles, those with at least `HUE.min_pixels`.
    fn plaza_tiles(p: &Probe, pixels: &[Option<(i32, i32)>], still: &[u8]) -> Vec<PlazaTile> {
        let size = (p.photo.width, p.photo.height);
        let px = |rgba: &[u8], i: usize| [0, 1, 2].map(|c| f32::from(rgba[i * 4 + c]));
        let mut tiles: BTreeMap<(i32, i32), (Vec<[f32; 3]>, Vec<[f32; 3]>)> = BTreeMap::new();
        for i in 0..size.0 * size.1 {
            let Some(tt) = pixels[i] else {
                continue;
            };
            let e = tiles.entry(tt).or_default();
            e.0.push(px(&p.photo.rgba, i));
            e.1.push(px(still, i));
        }
        let mean = |v: &[[f32; 3]]| {
            let n = v.len().max(1) as f32;
            [0, 1, 2].map(|c| v.iter().map(|q| q[c]).sum::<f32>() / n)
        };
        tiles
            .into_iter()
            .filter(|(_, (th, _))| th.len() as u32 >= HUE.min_pixels)
            .map(|(tt, (th, us))| (tt, mean(&th), mean(&us), th.len()))
            .collect()
    }

    /// Both counts, `[hue, luma]`, and the tiles printed with both sides.
    fn count(name: &str, tiles: &[PlazaTile]) -> [(usize, usize); 2] {
        let (mut hue, mut luma) = (0, 0);
        for (tt, th, us, n) in tiles {
            let (p, q) = (chromaticity(*us), chromaticity(*th));
            let h = ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2)).sqrt() <= HUE.distance;
            let l = LUMA_READER.agrees(luminance(*us), luminance(*th));
            hue += usize::from(h);
            luma += usize::from(l);
            println!(
                "{name} tile {tt:?} px {n}: theirs ({:.0}, {:.0}, {:.0}) Y {:.1}, the still's ({:.0}, {:.0}, {:.0}) Y {:.1}: hue {h} luma {l}",
                th[0],
                th[1],
                th[2],
                luminance(*th),
                us[0],
                us[1],
                us[2],
                luminance(*us),
            );
        }
        let n = tiles.len();
        println!("{name}: hue {hue} of {n}, luma {luma} of {n}");
        [(hue, n), (luma, n)]
    }

    fn still(p: &mut Probe, name: &str) -> Vec<u8> {
        let size = (p.photo.width as u32, p.photo.height as u32);
        let camera = p.camera;
        let s = crate::still_of(
            &mut p.world,
            &camera,
            size,
            crate::gpu::SilhouetteOrder::Before,
            wgpu::TextureFormat::Rgba8Unorm,
        );
        if let Some(dir) = std::env::var_os("RON_STILL_DIR") {
            let out = PathBuf::from(dir).join(format!("still-{name}.bmp"));
            std::fs::write(&out, crate::gpu::bmp(&s, size)).expect("the still writes");
        }
        s
    }

    /// The row on one photograph: as drawn before (no weathering), drawn,
    /// and the made-to-fail (every footprint moved by its own width).
    fn rows(p: &mut Probe, name: &str) -> [[(usize, usize); 2]; 3] {
        // The selection first: `ground_why` sets the trees' pixels the
        // stills after it are drawn with.
        let pixels = plaza_pixels(p);
        let mut out = [[(0, 0); 2]; 3];
        for (k, (on, shift, what)) in [
            (false, (0, 0), "before"),
            (true, (0, 0), "drawn"),
            (true, (1, 0), "moved"),
        ]
        .into_iter()
        .enumerate()
        {
            p.world.weathering = on;
            p.world.weathering_shift = shift;
            let s = still(p, &format!("{name}-{what}"));
            out[k] = count(&format!("{name} {what}"), &plaza_tiles(p, &pixels, &s));
        }
        p.world.weathering = true;
        p.world.weathering_shift = (0, 0);
        out
    }

    /// **The paved base's row** (item 4190; §3.16): on run1559b's and
    /// run1570's photographs, as drawn before, drawn, and moved.
    #[test]
    fn the_paved_base_under_a_city() {
        let Some(mut p) = probe_at(run1559b()) else {
            return;
        };
        let a = rows(&mut p, "run1559b");
        let Some(mut p) = probe_at(run1570()) else {
            return;
        };
        let b = rows(&mut p, "run1570");
        println!("{a:?} {b:?}");
    }

    fn card_trace_path() -> PathBuf {
        std::env::var_os("RON_CARD_TRACE")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let home = std::env::var_os("HOME").unwrap_or_default();
                PathBuf::from(home).join("ron-golden/run1566b/map-14/rontrace.log")
            })
    }

    /// **Draw 90 is the weathering's** (item 4190): on run1566b's card
    /// trace, the pass after 3279, draw 90 binds `Modern_Weathering.tga`
    /// and `age0_22.tga` (`tilesets.xml` `<COMMON><WEATHERING>`, not
    /// `roads.xml`) under enables `0x1c204`, lit, stage 0
    /// `MODULATE(TEXTURE, DIFFUSE)` with A `SELECTARG1(TEXTURE)`, stage 1
    /// disabled — so `age0_22`'s alpha never reaches the pixel. Its return
    /// address lies in `RoadsOut::render` because the card flushes one draw
    /// late: `Scene::render@008b3270` calls `render_weathering(0, 3)` just
    /// before `render_roads`, and draw 91 (`modern_road_pieces.tga`, enables
    /// `0x20004`, `RoadsOut`'s own state) flushes in `Game::loop_render`.
    #[test]
    fn draw_90_is_the_weathering_s() {
        use rondata::trace::card;
        let Ok(bytes) = std::fs::read(card_trace_path()) else {
            println!("skipping: no card trace at {}", card_trace_path().display());
            return;
        };
        let all = card::draws(&bytes);
        let at = |i: u32| {
            all.iter()
                .find(|d| d.frame == 3279 && d.index == i)
                .expect("the pass after 3279 holds the draw")
        };
        let (w, r) = (at(90), at(91));
        assert_eq!(
            [w.file(0), w.file(1)],
            ["Modern_Weathering.tga", "age0_22.tga"]
        );
        assert_eq!((w.enables(), w.src_blend(), w.dst_blend()), (0x1c204, 5, 6));
        assert_eq!(
            w.colour_op(0),
            (card::TOP_MODULATE, card::TA_TEXTURE, card::TA_DIFFUSE)
        );
        assert_eq!(w.colour_op(1).0, 0, "stage 1 disabled");
        assert_eq!(w.alpha_op(0), (1, card::TA_TEXTURE, card::TA_CURRENT));
        assert!(w.has("ENABLE_LIGHTING") && w.has("TEX1_USE_UV1"));
        // The flush: draw 90 returns into `RoadsOut::render`
        // (0x8943d0..0x894720), draw 91 into `Game::loop_render`.
        assert!((0x8943d0..0x894720).contains(&w.frames[0]));
        assert_eq!(r.file(0), "modern_road_pieces.tga");
        assert_eq!(r.enables(), 0x20004);
        assert_eq!(r.frames[0], 0x591a45);
    }
}
