//! **The `Oil` good's billboard on the lake** (item 4312; `docs/RENDERER.md`
//! §3.24, "The `Oil` good's billboard").
//!
//! `ObjectsOut::draw_good_underlays@00654b60` walks the map's goods and
//! calls each one's `GoodOut::draw_underlays@0066e070`. An underlay piece
//! (`RData::flags & 0x20`, the `<GOOD>`'s `underlay="1"`) that is not a
//! herd's goes to `GoodTypeOut::draw@0066fd70`, which hands
//! `GraphicPieces::draw_good@008e9080` the good's own position. An `Oil`
//! good's piece is a `model="BillBoard"` sprite on `rmaps[0xd]`
//! (`GraphicPieces::init_good_data@008efa60`, `init_good_map@008f7800`):
//! `ZSpriteAccumMethod::execute@0091e350` lays it as one quad of the
//! texture's own size, upright, across the screen's diagonal, at the
//! translation `draw_good` set: the position plus `192 · ⌊sprite_center +
//! ½⌋` on x and y, `sprite_center` the `<GOOD>`'s `offset`. The quad is
//! `tex_w · sprite_x_width` wide and `tex_h · sprite_z_height` tall, the two
//! scales `Scene::render@008b3270` sets so that one texel covers
//! `fixed_zooms[6] / zoom_distance` pixels.
//!
//! The card (run1567, pass 3279) draws it as draws 139–140, before the fish
//! and the water: `TL_FAKE_LIGHT`, the vertex white where the good's cell
//! is seen, blended source alpha over.

use std::sync::Arc;

use render::silhouette::Mips;

/// `TypeIndex::OIL`, the good's type ([`sim::world::OIL_GOOD`]).
pub const OIL: usize = sim::world::OIL_GOOD;

/// The world units `draw_good` moves the sprite by, a step of its
/// `sprite_center` on x and on y (`192.0`, `0x43400000`).
pub const CENTER_STEP: f32 = 192.0;

/// The `<GOOD name="Oil">` billboard's art: its texture, tinted at load as
/// every good piece is (`RData::set_param` of `GraphicPieces::tint_color`),
/// and its `offset` (`sprite_center`).
pub struct OilArt {
    pub mips: Arc<Mips>,
    pub size: (u32, u32),
    pub offset: i32,
}

impl OilArt {
    /// `None` where `effects_graphics.xml` names no `Oil` billboard or its
    /// texture does not read.
    pub fn load(root: &std::path::Path, tint: [u8; 3]) -> Option<OilArt> {
        let xml = std::fs::read_to_string(root.join("Data/effects_graphics.xml")).ok()?;
        let tag = xml
            .match_indices("<GOOD ")
            .filter_map(|(i, _)| Some(&xml[i..i + xml[i..].find('>')?]))
            .find(|t| attr(t, "name") == Some("Oil"))?;
        if !attr(tag, "model")?.eq_ignore_ascii_case("BillBoard") {
            return None;
        }
        let file = attr(tag, "texture")?;
        let offset = attr(tag, "offset")
            .and_then(|o| o.parse().ok())
            .unwrap_or(0);
        let bytes = std::fs::read(rondata::artdata::resolve_art(root, file)?).ok()?;
        let mut img = render::tga::decode(&bytes).ok()?;
        img.tint(tint);
        let size = (img.width as u32, img.height as u32);
        Some(OilArt {
            mips: Arc::new(Mips::new(img)),
            size,
            offset,
        })
    }
}

fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let key = format!(" {name}=\"");
    let i = tag.find(&key)? + key.len();
    Some(&tag[i..i + tag[i..].find('"')?])
}

/// **`Scene::render`'s sprite scales** on `camera` at `size`:
/// `sprite_x_width`, the world units along x a sprite's texel spans, and
/// `sprite_z_height`, the world units along z. Each is `fixed_zooms[6] /
/// zoom_distance · ±768` over the pixels 768 world units cross: on x
/// between `(0, 768, 0)` and `(768, 0, 0)`, on z between `(768, 0, 0)` and
/// `(768, 0, 768)`, through `world_to_console`.
pub fn sprite_scales(camera: &render::Camera, size: (u32, u32), nearest_zoom: f32) -> (f32, f32) {
    let m = camera.to_screen_matrix();
    let screen = render::Screen {
        scaled_xs: size.0 as i32,
        scaled_ys: size.1 as i32,
        natural_ratio: 1.0,
        console_y: 0,
    };
    let at = |x, y, z| {
        render::camera::world_to_console(&m, &screen, x, y, z, Some((size.0 as i32, size.1 as i32)))
    };
    let ratio = nearest_zoom / camera.zoom_distance;
    let (a, b, c) = (at(0, 768, 0), at(768, 0, 0), at(768, 0, 768));
    let x = ratio * 768.0 / (b.x - a.x) as f32;
    let z = ratio * -768.0 / (c.y - b.y) as f32;
    (x, z)
}

/// **The billboards as one [`crate::world::PatchMesh`]**: one upright quad a
/// live `Oil` good, its foot across the screen's diagonal at the good's
/// position plus [`CENTER_STEP`] times the `offset`, `tex_w · x_width` wide
/// and `tex_h · z_height` tall (`ZSpriteAccumMethod::execute`), the
/// texture's top row at the top. `None` with no such good.
///
/// `GoodTypeOut::draw` keeps an `Oil` good only on a cell whose
/// `WData::land` is 1 or 2 and whose `WData::flags & 0x100` is clear
/// ([`drawn_on`]); a land patch of oil is the terrain's own
/// (`TerrainOil::add_rocky_patch`).
pub fn mesh(
    art: &OilArt,
    goods: &[sim::world::Good],
    terrain: &sim::observe::TerrainScene,
    (x_width, z_height): (f32, f32),
) -> Option<crate::world::PatchMesh> {
    let step = CENTER_STEP * art.offset as f32;
    let w = art.size.0 as f32 * x_width;
    let h = art.size.1 as f32 * z_height;
    let mut m = crate::world::PatchMesh {
        piece: crate::world::OIL_PIECE,
        maps: None,
        positions: Vec::new(),
        uvs: Vec::new(),
        alpha_uvs: Vec::new(),
        mips: art.mips.clone(),
        alpha: crate::world::OIL_ALPHA,
        alpha_mips: art.mips.clone(),
        mountain: false,
        light: Vec::new(),
    };
    for g in goods
        .iter()
        .filter(|g| g.ty == OIL && g.alive && drawn_on(terrain, g.pos))
    {
        let (x, y, z) = (g.pos.x as f32 + step, g.pos.y as f32 + step, 0.0);
        let left_foot = [x - w / 2.0, y + w / 2.0, z];
        let right_foot = [x + w / 2.0, y - w / 2.0, z];
        let left_top = [left_foot[0], left_foot[1], z + h];
        let right_top = [right_foot[0], right_foot[1], z + h];
        let (lf, rf, lt, rt) = ([0.0, 1.0], [1.0, 1.0], [0.0, 0.0], [1.0, 0.0]);
        m.positions.extend_from_slice(&[
            left_top, right_top, left_foot, left_foot, right_top, right_foot,
        ]);
        m.uvs.extend_from_slice(&[lt, rt, lf, lf, rt, rf]);
        m.alpha_uvs.extend_from_slice(&[lt, rt, lf, lf, rt, rf]);
    }
    (!m.positions.is_empty()).then_some(m)
}

/// The cell of `GoodTypeOut::draw`'s test for an `Oil` good at `pos`
/// (`div_3_table[pos >> 8]`, a cell of 768 units): its `land` 1 or 2 and its
/// `flags & 0x100` clear.
pub fn drawn_on(terrain: &sim::observe::TerrainScene, pos: sim::Pos) -> bool {
    let (cx, cy) = (pos.x.div_euclid(768), pos.y.div_euclid(768));
    if !(0..terrain.width).contains(&cx) || !(0..terrain.height).contains(&cy) {
        return false;
    }
    let i = (cy * terrain.width + cx) as usize;
    matches!(terrain.land.get(i), Some(1 | 2))
        && terrain.flags.get(i).is_some_and(|f| f & 0x100 == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The quad covers the texture at the camera's sprite scale** (item
    /// 4312): on the pinned camera, a 256 × 256 billboard's feet project
    /// `256 · fixed_zooms[6] / zoom_distance` pixels apart across the
    /// screen, and its top as far above them; a good on `BASELAND` is not
    /// drawn. Seen red with the scales' 768 halved: 85 px.
    #[test]
    fn the_billboard_spans_the_texture_at_the_sprite_scale() {
        let camera = crate::pinned_camera();
        let size = (1920, 1080);
        let zoom = crate::PINNED_SPRITE_ZOOM.0;
        let (xw, zh) = sprite_scales(&camera, size, zoom);
        let img = render::tga::Image {
            width: 256,
            height: 256,
            kind: 2,
            bits: 32,
            footer: false,
            rgba: vec![255; 256 * 256 * 4],
        };
        let art = OilArt {
            mips: Arc::new(Mips::new(img)),
            size: (256, 256),
            offset: 1,
        };
        let terrain = |land: i8| sim::observe::TerrainScene {
            width: 64,
            height: 64,
            land: vec![land; 64 * 64],
            flags: vec![0; 64 * 64],
            ..Default::default()
        };
        let at = sim::Pos::new(30 * 768 + 384, 30 * 768 + 384);
        let good = [sim::world::Good {
            pos: at,
            ty: OIL,
            alive: true,
        }];
        let m = mesh(&art, &good, &terrain(render::terrain::OCEAN), (xw, zh)).expect("drawn");
        assert!(mesh(&art, &good, &terrain(render::terrain::BASELAND), (xw, zh)).is_none());
        let mx = camera.to_screen_matrix();
        let screen = crate::pinned_screen();
        let px = |p: [f32; 3]| {
            render::camera::world_to_console(
                &mx,
                &screen,
                p[0] as i32,
                p[1] as i32,
                p[2] as i32,
                Some((1920, 1080)),
            )
        };
        let (lt, rt, lf) = (px(m.positions[0]), px(m.positions[1]), px(m.positions[2]));
        let want = 256.0 * zoom / camera.zoom_distance;
        println!(
            "4312: across {} up {} want {want:.1}",
            rt.x - lt.x,
            lf.y - lt.y
        );
        assert!(((rt.x - lt.x) as f32 - want).abs() <= 2.0);
        assert!(((lf.y - lt.y) as f32 - want).abs() <= 2.0);
        assert_eq!(lt.y, rt.y, "the quad faces the screen");
    }

    /// **The box round run1564's land oil** (item 4309): the photograph's
    /// pixels `[x0, x1) × [y0, y1)` that hold the `TerrainOil` patch south-east
    /// of the lake (`crop-1564-rock.png`), clear of the interface.
    pub const RUN1564_ROUGH_BOX: (usize, usize, usize, usize) = (1440, 1800, 680, 880);

    /// **An oil pit's pixel** (item 4309): the `oil` underlay
    /// (`d_rough_oil.tga`) is black pools ringed with rock, and nothing
    /// else in the box reads under luma 60. The photograph's grass and sand
    /// there read 130 to 180.
    pub fn pit(rgb: [u8; 3]) -> bool {
        let [r, g, b] = rgb.map(u32::from);
        (299 * r + 587 * g + 114 * b) / 1000 < 60
    }

    /// **`RUN1564_3279_ROUGH`** (item 4309; `docs/RENDERER.md` §3.24,
    /// "The land's oil"): in [`RUN1564_ROUGH_BOX`] on run1564's still,
    /// `(both, theirs, ours)` of the pixels [`pit`] calls an oil pit's.
    /// Minted at the first act at 0 of 2,387: `TerrainOil`'s `rough` and
    /// `oil` underlays are not drawn. Item 4319: **0 → 2,287** of 2,387,
    /// ours 0 → 2,483: the underlays drawn (`crate::oil`), the oil's 4 × 4
    /// on cell (52, 49) and twenty 2 × 2 `rough` patches off our own
    /// `internal_random`; ours' pools read softer at the rim (4319a).
    /// Item 4310: 2,287 → 2,289, ours 2,483 → 2,486: the faded land blended over black in HDR (item 4310, `docs/RENDERER.md` §3.27, "The faded land in HDR") moves the frame's mean, so its white.
    /// Item 4338: 2,289 → 2,290, 2,486 → 2,490: the surf's lists blended on the HDR target under the card's point sampler (item 4338, `docs/RENDERER.md` §3.30).
    /// Item 4379: 2,290 → 2,287 (a named fall of three), ours 2,490 → 2,483: the national borders blended on the HDR target through `TL_FAKE_LIGHT`, before the water and the shade (item 4379, `docs/RENDERER.md` §3.29, "The band's colour"), move the frame's mean, so its white.
    pub const RUN1564_3279_ROUGH: [usize; 3] = [2287, 2387, 2483];

    #[test]
    fn the_land_oil_s_pits_on_run1564() {
        use crate::ground::tests::{probe_at, run1564};
        let Some(mut p) = probe_at(run1564()) else {
            return;
        };
        let size = (p.photo.width as u32, p.photo.height as u32);
        let camera = p.camera;
        let mx = camera.to_screen_matrix();
        let screen = crate::pinned_screen();
        for g in p.world.runner.goods().iter().filter(|g| g.ty == OIL) {
            let c = render::camera::world_to_console(
                &mx,
                &screen,
                g.pos.x,
                g.pos.y,
                0,
                Some((size.0 as i32, size.1 as i32)),
            );
            if (0..size.0 as i32).contains(&c.x) && (0..size.1 as i32).contains(&c.y) {
                println!(
                    "4309: oil good at ({}, {}) cell ({}, {}) alive {} pixel ({}, {})",
                    g.pos.x,
                    g.pos.y,
                    g.pos.x / 768,
                    g.pos.y / 768,
                    g.alive,
                    c.x,
                    c.y
                );
            }
        }
        let ours = crate::still_of(
            &mut p.world,
            &camera,
            size,
            crate::gpu::SilhouetteOrder::Before,
            wgpu::TextureFormat::Rgba8Unorm,
        );
        if let Some(dir) = std::env::var_os("RON_STILL_DIR") {
            let out = std::path::PathBuf::from(dir).join("still-run1564-rough.bmp");
            std::fs::write(&out, crate::gpu::bmp(&ours, size)).expect("the still writes");
        }
        let w = p.photo.width;
        let px = |rgba: &[u8], i: usize| [rgba[i * 4], rgba[i * 4 + 1], rgba[i * 4 + 2]];
        let (x0, x1, y0, y1) = RUN1564_ROUGH_BOX;
        let mut got = [0; 3];
        for y in y0..y1 {
            for x in x0..x1 {
                let i = y * w + x;
                let (a, b) = (pit(px(&p.photo.rgba, i)), pit(px(&ours, i)));
                got[0] += usize::from(a && b);
                got[1] += usize::from(a);
                got[2] += usize::from(b);
            }
        }
        println!("RUN1564_3279_ROUGH: {got:?}");
        assert_eq!(got, RUN1564_3279_ROUGH);
    }
}
