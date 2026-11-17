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
}
