//! **What a selected building shows** (item 4398; `docs/HUD.md` §3): the
//! original's outline about it, as geometry on the ground.
//!
//! `WallOut::draw_underlays@006410a0` draws a selected building (the
//! console's `play` group holds it, `GroupData::member`) with
//! `draw_thick_rectangle@008c6180`, in the owner's `TeamColor::get_unit`.
//! Its rectangle, by the listing at `006411a4`–`0064136d`:
//!
//! - `k` is the drawn piece's `BuildRDataStruct::build_half_tcoord`, or,
//!   where its `build_construction` is not 0, the type's `+0x234` (its
//!   `x_size`) less it — the same `k` as the foundation's
//!   [`render::draw::Foundation::back`];
//! - the centre is the building's point less `0x60 · k` on both axes;
//! - `ECX` and `EDX` are `x_size − k` and `y_size − k`, in tiles, and the
//!   rectangle's half extents are those times `0xc0 / 2`.
//!
//! `draw_thick_rectangle` walks the four sides from the `(−x, −y)` corner
//! counter-clockwise (`+x`, then `+y`, `−x`, `−y`), one vertex a tile
//! (`192.0`), and pairs each with one moved `48.0` along the previous
//! side's direction, which is outward; a corner square closes each side.
//! So the band is **48 world units wide, outside the rectangle**, every
//! vertex at `TerrainOut::find_data_z` of its own point.
//!
//! **Its texture** (item 4421): `set_texture(0, circle_gen +0x134)`, the
//! first of `CircleRender::circle_textures`, which `CircleRender::init`
//! loads from `effects_graphics.xml`'s `<SELECTIONCIRCLE>` list: the
//! install's `city_radius.tga`. Every vertex's `u` is 0; an inner one's
//! `v` is 0, an outer one's 1 (`TexArray::add`'s `xmm1`, `xmm2`; `1.0` at
//! `0xb69514`). The texture's columns are one tent, alpha 0 at both edges
//! and 255 at row 16, point-sampled ([`BandAlphas`]), so the band is a soft
//! line at its middle, not a solid fill. (`int_str_array +0x1784`,
//! `Effects`, is `set_poly_count_type`'s statistics name, not a texture.)

use render::terrain::Heightfield;

/// Half a tile, in world units: `0x60`.
pub const HALF_TILE: i32 = 0x60;

/// The band's width outside the rectangle, `48.0` in the listing.
pub const BAND: i32 = 48;

/// **The rectangle** `(x0, y0, x1, y1)`, world units, of a building at
/// `(x, y)` with a footprint of `size` tiles and the back `k`.
pub fn rectangle(x: i32, y: i32, size: (i32, i32), k: i32) -> (i32, i32, i32, i32) {
    let (cx, cy) = (x - HALF_TILE * k, y - HALF_TILE * k);
    let (hx, hy) = ((size.0 - k) * HALF_TILE, (size.1 - k) * HALF_TILE);
    (cx - hx, cy - hy, cx + hx, cy + hy)
}

/// **The band's triangles**, each corner `(x, y, z)` in world units: per
/// side, one quad a tile between the rectangle's edge and the edge moved
/// [`BAND`] out, and the corner square at the side's end, each vertex at
/// `data_z` of its point clamped to the map as the listing clamps it.
pub fn band(field: &Heightfield<'_>, r: (i32, i32, i32, i32)) -> Vec<[(i32, i32, i32); 3]> {
    band_v(field, r)
        .into_iter()
        .map(|t| t.map(|(p, _)| p))
        .collect()
}

/// A band corner: its point `(x, y, z)` and its texture `v`, 0 or 1.
pub type BandVertex = ((i32, i32, i32), u8);

/// [`band`]'s triangles with each corner's texture `v` (item 4421): 0 on
/// the rectangle's edge, 1 on the edge [`BAND`] out; in a corner square, 0
/// at the rectangle's corner and 1 at the other three, as the listing's
/// `TexArray::add` calls give them (`0, 1, 1, 1` after the strip's pair).
pub fn band_v(field: &Heightfield<'_>, r: (i32, i32, i32, i32)) -> Vec<[BandVertex; 3]> {
    let (x0, y0, x1, y1) = r;
    // Corners from (−x, −y) counter-clockwise, and each side's direction.
    let corners = [(x0, y0), (x1, y0), (x1, y1), (x0, y1)];
    let dirs = [(1, 0), (0, 1), (-1, 0), (0, -1)];
    let (mw, mh) = (field.width * 0x300, field.height * 0x300);
    let at = |x: i32, y: i32| {
        let (cx, cy) = (x.clamp(0, mw - 1), y.clamp(0, mh - 1));
        (x, y, render::trees::data_z(field, cx, cy))
    };
    let mut out = Vec::new();
    for k in 0..4 {
        let (sx, sy) = corners[k];
        let (dx, dy) = dirs[k];
        let (ox, oy) = dirs[(k + 3) % 4];
        let len = if k % 2 == 0 { x1 - x0 } else { y1 - y0 };
        // One vertex a tile, the last at the side's far corner.
        let mut steps: Vec<i32> = (0..)
            .map(|i| i * 2 * HALF_TILE)
            .take_while(|s| *s < len)
            .collect();
        steps.push(len);
        let (out_x, out_y) = (ox * BAND, oy * BAND);
        for w in steps.windows(2) {
            let (a, b) = (
                (sx + dx * w[0], sy + dy * w[0]),
                (sx + dx * w[1], sy + dy * w[1]),
            );
            let (ai, bi) = ((at(a.0, a.1), 0), (at(b.0, b.1), 0));
            let (ao, bo) = (
                (at(a.0 + out_x, a.1 + out_y), 1),
                (at(b.0 + out_x, b.1 + out_y), 1),
            );
            out.push([ai, bi, bo]);
            out.push([ai, bo, ao]);
        }
        // The corner square past the side's end.
        let e = (sx + dx * len, sy + dy * len);
        let c = [
            (at(e.0, e.1), 0),
            (at(e.0 + dx * BAND, e.1 + dy * BAND), 1),
            (at(e.0 + dx * BAND + out_x, e.1 + dy * BAND + out_y), 1),
            (at(e.0 + out_x, e.1 + out_y), 1),
        ];
        out.push([c[0], c[1], c[2]]);
        out.push([c[0], c[2], c[3]]);
    }
    out
}

/// **The band's texture, as the draw samples it** (item 4421): the alpha
/// of each row of column 0 of the first `<SELECTIONCIRCLE>`'s texture,
/// rows top-down as the card holds them. Its rgb is white, so the band's
/// colour is the vertex's (`MODULATE(TEXTURE, CURRENT)` on run1566b's
/// draw 115) and its alpha the row's times the vertex's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BandAlphas(pub Vec<u8>);

impl BandAlphas {
    /// The install's: `effects_graphics.xml`'s first `<SELECTIONCIRCLE
    /// texture=…>` (`CircleRender::init@008c48a0`, `circle_textures[0]`).
    pub fn load(root: &std::path::Path) -> Option<BandAlphas> {
        let xml = std::fs::read_to_string(root.join("Data/effects_graphics.xml")).ok()?;
        let at = xml.find("<SELECTIONCIRCLE")?;
        let rest = &xml[at..];
        let t = rest.find("texture=\"")? + "texture=\"".len();
        let name = &rest[t..t + rest[t..].find('"')?];
        let bytes = std::fs::read(rondata::artdata::resolve_art(root, name)?).ok()?;
        let img = render::tga::decode(&bytes).ok()?;
        Some(BandAlphas(
            (0..img.height)
                .map(|y| img.rgba[y * img.width * 4 + 3])
                .collect(),
        ))
    }
}

/// One triangle's part with `v` in `[lo, hi]`, as a fan of triangles: the
/// triangle clipped by the two lines, `v` affine over it as a
/// perspective-correct interpolation makes it in the world.
fn clip_v(tri: &[([f32; 3], f32); 3], lo: f32, hi: f32) -> Vec<[[f32; 3]; 3]> {
    let cut = |poly: Vec<([f32; 3], f32)>, keep: &dyn Fn(f32) -> bool, at: f32| {
        let mut out = Vec::new();
        for i in 0..poly.len() {
            let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
            if keep(a.1) {
                out.push(a);
            }
            if keep(a.1) != keep(b.1) {
                let t = (at - a.1) / (b.1 - a.1);
                let p = [0, 1, 2].map(|k| a.0[k] + (b.0[k] - a.0[k]) * t);
                out.push((p, at));
            }
        }
        out
    };
    let poly = cut(tri.to_vec(), &|v| v >= lo, lo);
    let poly = cut(poly, &|v| v <= hi, hi);
    (1..poly.len().saturating_sub(1))
        .map(|i| [poly[0].0, poly[i].0, poly[i + 1].0])
        .collect()
}

/// **The selected buildings' bands as drawn** (items 4401, 4421): for
/// each building of `selection` in the scene, [`band_v`] about its
/// [`rectangle`], every vertex in the owner's unit colour
/// (`render::silhouette::team_unit`, `TeamColor::get_unit`'s), cut at
/// each row of [`BandAlphas`] into one layer an alpha: the texel's, which
/// the GPU's underlay pass multiplies by [`crate::gpu::UNDERLAY_ALPHA`].
/// A row of alpha 0 draws nothing. Without the texture the band is one
/// layer at 255, the solid fill of item 4401. A unit draws no band here
/// (its ring is [`crate::hud::rings`]'). The listing's alpha 0 for a
/// vertex off the map is not carried.
pub fn underlay(
    world: &crate::world::World,
    selection: &[sim::observe::ObjId],
) -> Vec<crate::gpu::ScreenLayer> {
    let mut layers: std::collections::BTreeMap<u8, crate::gpu::ScreenLayer> = Default::default();
    if selection.is_empty() {
        return Vec::new();
    }
    let alphas = world.band_alphas();
    let placed = world.placed_buildings();
    let t = world.runner.terrain();
    let field = Heightfield {
        width: t.width,
        height: t.height,
        corners: &t.corners,
    };
    for id in selection {
        let Some(b) = placed.iter().find(|b| (b.who, b.o) == (id.who, id.o)) else {
            continue;
        };
        let Some(&size) = usize::try_from(b.type_index - rondata::load::BASE_BUILDTYPES)
            .ok()
            .and_then(|ti| world.names.footprints.get(ti))
        else {
            continue;
        };
        let back = b.foundation.map_or(b.half_offset, |f| f.back);
        let c = render::silhouette::team_unit(b.who).map(|v| f32::from(v) / 255.0);
        for tri in band_v(&field, rectangle(b.x, b.y, size, back)) {
            let tri = tri.map(|((x, y, z), v)| ([x as f32, y as f32, z as f32], f32::from(v)));
            let rows = alphas.map_or(1, |a| a.0.len());
            for r in 0..rows {
                let a = alphas.map_or(255, |a| a.0[r]);
                if a == 0 {
                    continue;
                }
                let (lo, hi) = (r as f32 / rows as f32, (r + 1) as f32 / rows as f32);
                let l = layers.entry(a).or_insert_with(|| crate::gpu::ScreenLayer {
                    alpha: a,
                    ..Default::default()
                });
                for t in clip_v(&tri, lo, hi) {
                    l.pos.extend(t);
                    l.col.extend([c; 3]);
                }
            }
        }
    }
    layers.into_values().filter(|l| !l.pos.is_empty()).collect()
}

/// **The scroll's pads** (item 4403), `IFaceRenderManager +0xe84`:
/// left, top, right, bottom. `setup_text@0051dd50` sets the top to
/// `scroll_tex[0]`'s width and the sides to twice it; run1559b's packet
/// holds 8, 4, 8, 4 (`scroll_tex[0]` is 4 × 4).
pub const SCROLL_PAD: (i32, i32, i32, i32) = (8, 4, 8, 4);

/// **The capital's star**, `scroll_capital_star`'s rectangle on the
/// interface texture: 22 × 21 on run1559b's packet.
pub const CAPITAL_STAR: (i32, i32) = (22, 21);

/// **The plate's font height**: `onscreen_mouseover_size`'s font
/// (`int_str_array +0x1680`, `FontFamily` lookup 36, size 14, style 1),
/// whose `D3D11GLFont` holds `mFontHeight` 16 on run1559b's packet.
pub const PLATE_FONT_HEIGHT: i32 = 16;

/// **The city's plate, on the screen** (item 4403):
/// `BuildOut::draw_overlays@0062cb30`'s rectangle, as `[x0, x1) × [y0,
/// y1)` in window pixels. The anchor is `MiscAccess::world_to_console` of
/// the building's point lowered `0x60 · (2 · y_size / 3)` in z (the
/// listing at `0062cc4a`–`0062ccae`), the console's y added; the name's
/// `text_w` wide and the font's `h` tall about it (`x0 = sx − w/2`, `y0 =
/// sy − h/2`); then `IFaceRenderManager::adjust_for_scroll@0051cd20`
/// widens it for the star when the city is a capital (3 rows each way,
/// the star's width split left and right) and by [`SCROLL_PAD`].
pub fn plate(
    camera: &render::Camera,
    b: &render::draw::PlacedBuilding,
    y_size: i32,
    text_w: i32,
    h: i32,
    capital: bool,
) -> (i32, i32, i32, i32) {
    let z = b.z - 0x60 * (2 * y_size / 3);
    let p = render::camera::world_to_screen(
        &camera.to_screen_matrix(),
        &crate::pinned_screen(),
        b.x,
        b.y,
        z,
        None,
    );
    let (mut x0, mut y0) = (p.x - text_w / 2, p.y - h / 2);
    let (mut x1, mut y1) = (x0 + text_w, y0 + h);
    if capital {
        let half = CAPITAL_STAR.0 / 2;
        y0 -= 3;
        y1 += 3;
        x0 -= half;
        x1 += CAPITAL_STAR.0 - half;
    }
    let (l, t, r, bo) = SCROLL_PAD;
    (x0 - l, x1 + r, y0 - t, y1 + bo)
}

/// **The camera's zoom level on run1559b's packet**: `Camera::zoom_level`
/// (`+0x294` of `MiscAccess::camera`'s object) reads 5 at frame 3294
/// (item 4410), which is also `to_zoom_level`.
pub const BAR_ZOOM_LEVEL: i32 = 5;

/// **A building's health bar, on the screen** (item 4410):
/// `WallOut::draw_health_bar@00640f20`'s rectangle, as `[x0, x1) × [y0,
/// y1)` in window pixels. The anchor is `world_to_console` of the
/// building's point lifted `0x180` in z, the console's y added; the bar is
/// 30 × 3 at zoom level 6, 20 × 2 at 5 and 10 × 1 otherwise, about the
/// anchor (`x0 = sx − w/2`, `y0 = sy − h/2`, each half an arithmetic
/// shift). `GraphicPieces::draw_health_bar@00908650` then lays a black
/// quad one pixel outside it, and the fill from `x0` over `w · fraction`
/// (truncated) in the health level's colour.
pub fn bar(
    camera: &render::Camera,
    b: &render::draw::PlacedBuilding,
    zoom_level: i32,
) -> (i32, i32, i32, i32) {
    let p = render::camera::world_to_screen(
        &camera.to_screen_matrix(),
        &crate::pinned_screen(),
        b.x,
        b.y,
        b.z + 0x180,
        None,
    );
    let (w, h) = match zoom_level {
        6 => (30, 3),
        5 => (20, 2),
        _ => (10, 1),
    };
    let (x0, y0) = (p.x - (w >> 1), p.y - (h >> 1));
    (x0, x0 + w, y0, y0 + h)
}

/// **`ObjectData::health_level@006534b0`**: the hits left, `hits(0)` less
/// the damage, clamped to `[0, hits(0)]`, against `hits(0)`'s 90, 75, 50,
/// 25 and 10 per cent (each `full · n / 100`, truncated): 0 at or over
/// 90 %, then 1, 2, 3, 4, and under 10 % 5, or 6 with nothing left.
pub fn health_level(left: i32, full: i32) -> i32 {
    let at = |n: i32| full * n / 100;
    if left >= at(90) {
        0
    } else if left >= at(75) {
        1
    } else if left >= at(50) {
        2
    } else if left >= at(25) {
        3
    } else if left >= at(10) {
        4
    } else if left < 1 {
        6
    } else {
        5
    }
}

/// The bar's colours: `draw_health_bar`'s `GREEN` at level 0, `YELLOW` at
/// 1 to 3, `RED` past them; the frame `BLACK`. The red is the
/// photograph's fill, `(255, 0, 0)`; the green and the yellow are taken
/// as the plain primaries (`docs/HUD.md` §6).
pub const BAR_GREEN: [f32; 3] = [0.0, 1.0, 0.0];
pub const BAR_YELLOW: [f32; 3] = [1.0, 1.0, 0.0];
pub const BAR_RED: [f32; 3] = [1.0, 0.0, 0.0];
pub const BAR_BLACK: [f32; 3] = [0.0, 0.0, 0.0];

/// **The selected buildings' health bars** (item 4410):
/// `WallOut::draw_health_bar@00640dd0`'s arm for a member of the
/// console's `play` group, on [`bar`]'s rectangle at `zoom_level`. The
/// frame is a black quad one pixel outside it; the fill runs from `x0`
/// over `trunc(w · left / full)` in single precision, `full` the
/// building's `hits(1)` (at least 1) and `left` its `hits(0)` less the
/// damage, zeroed below nothing and capped at `full` (the listing at
/// `00640fb9`–`0064108b`); its colour is [`health_level`]'s. The arms for
/// the mouse-over, `+0x24` and another's unfinished building are not
/// ported (`docs/HUD.md` §6).
pub fn bars(
    world: &crate::world::World,
    camera: &render::Camera,
    size: (i32, i32),
    selection: &[sim::observe::ObjId],
    zoom_level: i32,
) -> Vec<crate::gpu::ScreenLayer> {
    if selection.is_empty() {
        return Vec::new();
    }
    let scene = world.runner.scene(None);
    let placed = world.placed_buildings();
    let mut p = crate::overlay::Painter::new(size);
    for id in selection {
        let Some(s) = scene.buildings.iter().find(|s| s.id == *id) else {
            continue;
        };
        let Some(b) = placed.iter().find(|b| (b.who, b.o) == (id.who, id.o)) else {
            continue;
        };
        let (x0, x1, y0, y1) = bar(camera, b, zoom_level);
        p.rect(x0 - 1, y0 - 1, x1 - x0 + 2, y1 - y0 + 2, BAR_BLACK);
        let full = s.hits.max(1);
        let left = s.health.clamp(0, full);
        let fill = ((x1 - x0) as f32 * (left as f32 / full as f32)) as i32;
        let colour = match health_level(s.health, s.construct_hits) {
            0 => BAR_GREEN,
            1..=3 => BAR_YELLOW,
            _ => BAR_RED,
        };
        if fill > 0 {
            p.rect(x0, y0, fill, y1 - y0, colour);
        }
    }
    vec![crate::gpu::ScreenLayer {
        alpha: 255,
        pos: p.pos,
        col: p.col,
    }]
}

/// **The plate's font's advances** (items 4403, 4423): each character's
/// `CharacterInfo::advance` in `onscreen_mouseover_size`'s `D3D11GLFont`
/// (`fonts`' lookup 36: size 14, style 1, `mFontHeight` 16), sorted by
/// character. The font rasterises a character the first time one is drawn,
/// so a packet holds only those its game has shown: this is the union over
/// the 144 packets on disk, 65 to 73 each, no character's advance differing
/// between any two (`PLATE_FONT_PACKETS`, item 4423). A name with any other
/// character (`X`, `Z`, anything past ASCII) has no width here, and no
/// plate is drawn for it rather than one guessed (`docs/HUD.md` §6).
/// Since item 4426 the client takes every advance from the face itself
/// ([`PlateFace::advance`]); this table stays as the packets' witness.
#[cfg(test)]
pub const PLATE_ADVANCES: [(char, i32); 77] = [
    (' ', 4),
    ('!', 5),
    ('%', 12),
    ('&', 11),
    ('\'', 4),
    ('(', 6),
    (')', 6),
    ('+', 10),
    (',', 5),
    ('-', 5),
    ('.', 5),
    ('/', 7),
    ('0', 10),
    ('1', 7),
    ('2', 9),
    ('3', 9),
    ('4', 9),
    ('5', 8),
    ('6', 9),
    ('7', 8),
    ('8', 9),
    ('9', 9),
    (':', 5),
    (';', 5),
    ('<', 10),
    ('>', 10),
    ('?', 8),
    ('A', 11),
    ('B', 11),
    ('C', 10),
    ('D', 12),
    ('E', 10),
    ('F', 9),
    ('G', 11),
    ('H', 13),
    ('I', 6),
    ('J', 8),
    ('K', 11),
    ('L', 10),
    ('M', 14),
    ('N', 12),
    ('O', 11),
    ('P', 10),
    ('Q', 11),
    ('R', 11),
    ('S', 9),
    ('T', 10),
    ('U', 12),
    ('V', 11),
    ('W', 16),
    ('Y', 10),
    ('a', 8),
    ('b', 9),
    ('c', 7),
    ('d', 9),
    ('e', 8),
    ('f', 6),
    ('g', 8),
    ('h', 10),
    ('i', 5),
    ('j', 5),
    ('k', 9),
    ('l', 5),
    ('m', 14),
    ('n', 10),
    ('o', 9),
    ('p', 9),
    ('q', 9),
    ('r', 7),
    ('s', 7),
    ('t', 6),
    ('u', 9),
    ('v', 8),
    ('w', 12),
    ('x', 8),
    ('y', 8),
    ('z', 7),
];

/// One character's advance in the plate's font ([`PLATE_ADVANCES`]).
#[cfg(test)]
pub fn plate_advance(c: char) -> Option<i32> {
    PLATE_ADVANCES
        .binary_search_by_key(&c, |&(k, _)| k)
        .ok()
        .map(|i| PLATE_ADVANCES[i].1)
}

/// The name's `text_width` in the plate's font, or `None` when a
/// character's advance is not held ([`PLATE_ADVANCES`]).
#[cfg(test)]
pub fn plate_text_width(name: &str) -> Option<i32> {
    name.chars().map(plate_advance).sum()
}

/// **The plate's face** (item 4426): `Data/Fonts/georgiab.ttf`, the file
/// whose `FT_Face` the plate's `D3D11GLFont` holds (`mSGLTexFont`'s
/// `m_fontFace`: family `Georgia`, style `Bold`, on every packet).
pub const PLATE_FACE_FILE: &str = "georgiab.ttf";

/// The face's size: `mPointSize` 14, set as pixels per em.
pub const PLATE_PPEM: i32 = 14;

/// The cell's baseline row: `mAscent` 13. `render_pendings` copies a
/// glyph's bitmap into its atlas cell at row `ascent − bitmap_top`.
pub const PLATE_ASCENT: i32 = 13;

/// One glyph's atlas cell, as `FontRenderManager::render_pendings` fills
/// it: `w` columns (`CharacterInfo::pix_width`, the bitmap's width) by
/// [`PLATE_FONT_HEIGHT`] rows, the coverage one byte a texel, the top row
/// first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlyphCell {
    pub w: usize,
    pub cover: Vec<u8>,
}

/// **The plate's glyphs, off the install's face** (item 4426). The
/// renderer DLL rasterises each character with its own FreeType
/// (`FT::FontFace::RenderGlyph`: `FT_Load_Glyph(face, index, 0)`, then
/// `FT_Render_Glyph(slot, 0)`, the 8-bit coverage), copies the bitmap into
/// a 256 × 256 atlas cell at the cell's left column (the bitmap's
/// `bitmap_left` is not applied) and row [`PLATE_ASCENT`] − `bitmap_top`,
/// and `D3D11GLFont::add_character` lays the cell's quad at the pen
/// (`abs_a` 0 on every packet) for `pix_width` × 16 pixels. The atlas
/// itself is the GPU's and no packet holds it, so the cells are
/// rasterised here from the same file, hinted as the DLL's FreeType hints
/// by default (item 4427: the face's TrueType bytecode under `skrifa`'s
/// interpreter, FreeType v40's normal target), the outline drawn by
/// `ab_glyph`'s coverage rasteriser (`docs/HUD.md` §6). Each advance is
/// the glyph's linear advance at [`PLATE_PPEM`], rounded: the packets'
/// 77 characters agree on all 9,849 entries.
#[derive(Clone)]
pub struct PlateFace {
    font: std::sync::Arc<ab_glyph::FontVec>,
    bytes: std::sync::Arc<Vec<u8>>,
    hinter: std::sync::Arc<skrifa::outline::HintingInstance>,
}

/// One glyph's outline in pixels, y up, as the hinter leaves it.
#[derive(Default)]
struct PixelPath {
    segs: Vec<Seg>,
    start: (f32, f32),
    at: (f32, f32),
}

enum Seg {
    Line([(f32, f32); 2]),
    Quad([(f32, f32); 3]),
    Cubic([(f32, f32); 4]),
}

impl skrifa::outline::OutlinePen for PixelPath {
    fn move_to(&mut self, x: f32, y: f32) {
        self.start = (x, y);
        self.at = (x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.segs.push(Seg::Line([self.at, (x, y)]));
        self.at = (x, y);
    }
    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        self.segs.push(Seg::Quad([self.at, (cx0, cy0), (x, y)]));
        self.at = (x, y);
    }
    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        self.segs
            .push(Seg::Cubic([self.at, (cx0, cy0), (cx1, cy1), (x, y)]));
        self.at = (x, y);
    }
    fn close(&mut self) {
        if self.at != self.start {
            self.segs.push(Seg::Line([self.at, self.start]));
        }
        self.at = self.start;
    }
}

impl PixelPath {
    /// `FT_Outline_Get_CBox`'s box: every point, the controls too.
    fn cbox(&self) -> Option<(f32, f32, f32, f32)> {
        let pts = self.segs.iter().flat_map(|s| match s {
            Seg::Line(p) => p.to_vec(),
            Seg::Quad(p) => p.to_vec(),
            Seg::Cubic(p) => p.to_vec(),
        });
        pts.fold(None, |b, (x, y)| {
            Some(match b {
                None => (x, y, x, y),
                Some((x0, y0, x1, y1)) => (x0.min(x), y0.min(y), x1.max(x), y1.max(y)),
            })
        })
    }
}

impl std::fmt::Debug for PlateFace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PlateFace")
    }
}

impl PlateFace {
    pub fn load(root: &std::path::Path) -> Result<PlateFace, String> {
        use ab_glyph::Font;
        let path = root.join("Data").join("Fonts").join(PLATE_FACE_FILE);
        let bytes = std::fs::read(&path).map_err(|e| format!("{PLATE_FACE_FILE}: {e}"))?;
        let font = ab_glyph::FontVec::try_from_vec(bytes)
            .map_err(|e| format!("{PLATE_FACE_FILE}: {e}"))?;
        let hinter = {
            use skrifa::MetadataProvider;
            let f = skrifa::FontRef::new(font.as_slice())
                .map_err(|e| format!("{PLATE_FACE_FILE}: {e}"))?;
            skrifa::outline::HintingInstance::new(
                &f.outline_glyphs(),
                skrifa::instance::Size::new(PLATE_PPEM as f32),
                skrifa::instance::LocationRef::default(),
                skrifa::outline::HintingOptions {
                    engine: skrifa::outline::Engine::Interpreter,
                    target: skrifa::outline::SmoothMode::Normal.into(),
                },
            )
            .map_err(|e| format!("{PLATE_FACE_FILE}: the hinter: {e}"))?
        };
        let face = PlateFace {
            bytes: std::sync::Arc::new(font.as_slice().to_vec()),
            font: std::sync::Arc::new(font),
            hinter: std::sync::Arc::new(hinter),
        };
        let upm = face.font.units_per_em().ok_or("the face has no em")?;
        // Rounded, as the packet's `mAscent` 13 and `mDescent` −3 are
        // (12.84 and 3.07 pixels: a ceiling would make the font 17 high).
        let px = |u: f32| (u * PLATE_PPEM as f32 / upm).round() as i32;
        let (ascent, descent) = (
            px(face.font.ascent_unscaled()),
            px(-face.font.descent_unscaled()),
        );
        if (ascent, ascent + descent) != (PLATE_ASCENT, PLATE_FONT_HEIGHT) {
            return Err(format!(
                "{PLATE_FACE_FILE}: ascent {ascent}, height {}, the packet's {PLATE_ASCENT} and {PLATE_FONT_HEIGHT}",
                ascent + descent
            ));
        }
        Ok(face)
    }

    fn glyph(&self, c: char) -> Option<ab_glyph::GlyphId> {
        use ab_glyph::Font;
        let id = self.font.glyph_id(c);
        (id.0 != 0).then_some(id)
    }

    /// One character's advance: its linear advance at [`PLATE_PPEM`],
    /// rounded, or `None` when the face has no glyph for it.
    pub fn advance(&self, c: char) -> Option<i32> {
        use ab_glyph::Font;
        let id = self.glyph(c)?;
        let upm = self.font.units_per_em()?;
        Some((self.font.h_advance_unscaled(id) * PLATE_PPEM as f32 / upm).round() as i32)
    }

    /// The name's `text_width`, or `None` when the face lacks a glyph.
    pub fn text_width(&self, name: &str) -> Option<i32> {
        name.chars().map(|c| self.advance(c)).sum()
    }

    /// One character's atlas cell ([`GlyphCell`]); a glyph with no
    /// outline (the space) is a cell 0 wide. The hinted outline's box is
    /// floored and ceiled to whole pixels, as `FT_Render_Glyph` sizes its
    /// bitmap, and the cell's row `r` is the band `PLATE_ASCENT − r − 1 ..
    /// PLATE_ASCENT − r` above the baseline.
    pub fn cell(&self, c: char) -> Option<GlyphCell> {
        use skrifa::MetadataProvider;
        let id = self.glyph(c)?;
        let font = skrifa::FontRef::new(self.bytes.as_slice()).ok()?;
        let outline = font
            .outline_glyphs()
            .get(skrifa::GlyphId::new(u32::from(id.0)))?;
        let mut path = PixelPath::default();
        outline
            .draw(
                skrifa::outline::DrawSettings::hinted(&self.hinter, false),
                &mut path,
            )
            .ok()?;
        let Some((x0, _, x1, _)) = path.cbox() else {
            return Some(GlyphCell {
                w: 0,
                cover: Vec::new(),
            });
        };
        let left = x0.floor();
        let w = (x1.ceil() - left) as usize;
        let h = PLATE_FONT_HEIGHT as usize;
        let mut raster = ab_glyph_rasterizer::Rasterizer::new(w, h);
        let pt = |(x, y): (f32, f32)| ab_glyph_rasterizer::point(x - left, PLATE_ASCENT as f32 - y);
        for seg in &path.segs {
            match *seg {
                Seg::Line([a, b]) => raster.draw_line(pt(a), pt(b)),
                Seg::Quad([a, b, c]) => raster.draw_quad(pt(a), pt(b), pt(c)),
                Seg::Cubic([a, b, c, d]) => raster.draw_cubic(pt(a), pt(b), pt(c), pt(d)),
            }
        }
        let mut cover = vec![0u8; w * h];
        raster.for_each_pixel_2d(|x, y, v| {
            cover[y as usize * w + x as usize] = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        });
        Some(GlyphCell { w, cover })
    }
}

/// One rectangle of interface texels, RGBA8, the top row first.
#[derive(Clone, Debug)]
pub struct Texels {
    pub w: usize,
    pub h: usize,
    pub rgba: Vec<u8>,
}

impl Texels {
    fn crop(
        image: &render::tga::Image,
        (l, t, w, h): (usize, usize, usize, usize),
    ) -> Option<Texels> {
        if l + w > image.width || t + h > image.height || w == 0 || h == 0 {
            return None;
        }
        let mut rgba = Vec::with_capacity(w * h * 4);
        for y in t..t + h {
            let at = (y * image.width + l) * 4;
            rgba.extend_from_slice(&image.rgba[at..at + w * 4]);
        }
        Some(Texels { w, h, rgba })
    }

    fn at(&self, u: usize, v: usize) -> [u8; 4] {
        let i = ((v % self.h) * self.w + u % self.w) * 4;
        [
            self.rgba[i],
            self.rgba[i + 1],
            self.rgba[i + 2],
            self.rgba[i + 3],
        ]
    }
}

/// **The plate's art, off the install** (item 4410): the nine parts of
/// `IFaceRenderManager::scroll_tex`, `interface.xml`'s nine `<IFACETEX
/// name="scroll">` under `<IFACERENDERMANAGER>` on `TEX_CIV`, row-major
/// from the top-left corner; and `scroll_capital_star`, `interface.txt`'s
/// `SCROLL_CAPITAL_STAR` on `TEX_COMMON`. Each texture is the
/// `<IFACETEXLOAD>` of its id. The rectangles are the files'; the reader
/// holds the star to [`CAPITAL_STAR`] and the top-left corner to
/// [`SCROLL_PAD`]'s top, the packet's, so a file that moved them is a
/// refusal rather than a plate.
#[derive(Clone, Debug)]
pub struct ScrollArt {
    pub parts: Vec<Texels>,
    pub star: Texels,
    pub face: PlateFace,
}

/// `name="value"` of one tag, the value trimmed.
fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let at = tag.find(&format!("{name}=\""))? + name.len() + 2;
    let len = tag[at..].find('"')?;
    Some(tag[at..at + len].trim())
}

impl ScrollArt {
    pub fn load(root: &std::path::Path) -> Result<ScrollArt, String> {
        let xml = std::fs::read_to_string(root.join("Data").join("interface.xml"))
            .map_err(|e| format!("interface.xml: {e}"))?;
        let manager = xml
            .split("<IFACERENDERMANAGER>")
            .nth(1)
            .and_then(|s| s.split("</IFACERENDERMANAGER>").next())
            .ok_or("interface.xml: no <IFACERENDERMANAGER>")?;
        let tags: Vec<&str> = manager.split('<').collect();
        let texture = |id: &str| -> Result<render::tga::Image, String> {
            let file = tags
                .iter()
                .filter(|t| t.starts_with("IFACETEXLOAD"))
                .find(|t| attr(t, "id") == Some(id))
                .and_then(|t| attr(t, "file"))
                .ok_or(format!("interface.xml: no {id}"))?;
            let path =
                rondata::artdata::resolve_art(root, file).ok_or(format!("{file}: absent"))?;
            let bytes = std::fs::read(&path).map_err(|e| format!("{file}: {e}"))?;
            render::tga::decode(&bytes).map_err(|e| format!("{file}: {e:?}"))
        };
        let civ = texture("TEX_CIV")?;
        let num = |t: &str, n: &str| attr(t, n).and_then(|v| v.parse::<usize>().ok());
        let parts: Vec<Texels> = tags
            .iter()
            .filter(|t| t.starts_with("IFACETEX ") && attr(t, "name") == Some("scroll"))
            .map(|t| {
                (attr(t, "texture") == Some("TEX_CIV"))
                    .then(|| {
                        Some((
                            num(t, "left")?,
                            num(t, "top")?,
                            num(t, "width")?,
                            num(t, "height")?,
                        ))
                    })
                    .flatten()
                    .and_then(|r| Texels::crop(&civ, r))
                    .ok_or(format!("interface.xml: a scroll part off TEX_CIV: {t}"))
            })
            .collect::<Result<_, _>>()?;
        if parts.len() != 9 {
            return Err(format!(
                "interface.xml: {} scroll parts, not nine",
                parts.len()
            ));
        }
        if parts[0].w as i32 != SCROLL_PAD.1 || parts[0].h as i32 != SCROLL_PAD.1 {
            return Err(format!(
                "the scroll's corner is {} × {}, the packet's pad {}",
                parts[0].w, parts[0].h, SCROLL_PAD.1
            ));
        }
        let txt = std::fs::read_to_string(root.join("interface.txt"))
            .map_err(|e| format!("interface.txt: {e}"))?;
        let line = txt
            .lines()
            .find(|l| l.trim_start().starts_with("SCROLL_CAPITAL_STAR"))
            .ok_or("interface.txt: no SCROLL_CAPITAL_STAR")?;
        let fields: Vec<&str> = line.split(',').map(str::trim).collect();
        let digits = |s: &str| {
            s.chars()
                .take_while(char::is_ascii_digit)
                .collect::<String>()
                .parse::<usize>()
                .ok()
        };
        let (Some(&"TEX_COMMON"), Some(l), Some(t), Some(w), Some(h)) = (
            fields.get(1),
            fields.get(2).and_then(|s| digits(s)),
            fields.get(3).and_then(|s| digits(s)),
            fields.get(4).and_then(|s| digits(s)),
            fields.get(5).and_then(|s| digits(s)),
        ) else {
            return Err(format!("interface.txt: the star's line reads {line:?}"));
        };
        if (w as i32, h as i32) != CAPITAL_STAR {
            return Err(format!(
                "the star is {w} × {h}, the packet's {CAPITAL_STAR:?}"
            ));
        }
        let star = Texels::crop(&texture("TEX_COMMON")?, (l, t, w, h))
            .ok_or("the star lies off TEX_COMMON")?;
        Ok(ScrollArt {
            parts,
            star,
            face: PlateFace::load(root)?,
        })
    }
}

/// The plate's name colour: `draw_overlays` writes it white.
pub const PLATE_INK: [f32; 3] = [1.0, 1.0, 1.0];

/// Texels as [`crate::gpu::ScreenLayer`]s, one an alpha, `place(x, y)`
/// the texel of window pixel `(x, y)` or `None`; a row's run of one
/// texel is one quad.
fn layers_of(
    size: (i32, i32),
    (x0, x1, y0, y1): (i32, i32, i32, i32),
    place: impl Fn(i32, i32) -> Option<[u8; 4]>,
) -> Vec<crate::gpu::ScreenLayer> {
    let mut by_alpha: std::collections::BTreeMap<u8, crate::overlay::Painter> = Default::default();
    for y in y0..y1 {
        let mut x = x0;
        while x < x1 {
            let Some(t) = place(x, y) else {
                x += 1;
                continue;
            };
            let mut end = x + 1;
            while end < x1 && place(end, y) == Some(t) {
                end += 1;
            }
            if t[3] > 0 {
                let c = [t[0], t[1], t[2]].map(|v| f32::from(v) / 255.0);
                by_alpha
                    .entry(t[3])
                    .or_insert_with(|| crate::overlay::Painter::new(size))
                    .rect(x, y, end - x, 1, c);
            }
            x = end;
        }
    }
    by_alpha
        .into_iter()
        .map(|(alpha, p)| crate::gpu::ScreenLayer {
            alpha,
            pos: p.pos,
            col: p.col,
        })
        .collect()
}

/// **One city's plate drawn** (item 4410): `draw_scroll@0051cdb0` on
/// [`plate`]'s rectangle `r`, as layers for the screen pass.
/// `draw_2d_expander@0051c7b0` lays the nine parts: the corners at the
/// rectangle's corners, the top and bottom edges and the sides tiled from
/// their part's first texel, the body `tile_random`'s (its one texel on
/// the install's `TEX_CIV`, so the random offset draws nothing different).
/// Over it, a capital's star at the pads' inner corner, its texels times
/// the owner's `TeamColor::get_unit` (`add_smooth`'s colour); then the
/// name, at the text's rectangle past the star and 3 rows down, each
/// character's [`GlyphCell`] at the pen in [`PLATE_INK`], its coverage
/// the alpha, the pen moved by the advance (item 4426: before it, the
/// overlay's 5 × 7 stood in).
pub fn plate_layers(
    art: &ScrollArt,
    size: (i32, i32),
    r: (i32, i32, i32, i32),
    capital: bool,
    tint: [u8; 3],
    name: &str,
) -> Vec<crate::gpu::ScreenLayer> {
    let (x0, x1, y0, y1) = r;
    let (w, h) = (x1 - x0, y1 - y0);
    let (cw, ch) = (art.parts[0].w as i32, art.parts[0].h as i32);
    let (rw, bh) = (art.parts[2].w as i32, art.parts[6].h as i32);
    let mut out = layers_of(size, r, |x, y| {
        let (u, v) = (x - x0, y - y0);
        let (col, du) = if u < cw {
            (0, u)
        } else if u >= w - rw {
            (2, u - (w - rw))
        } else {
            (1, u - cw)
        };
        let (row, dv) = if v < ch {
            (0, v)
        } else if v >= h - bh {
            (2, v - (h - bh))
        } else {
            (1, v - ch)
        };
        Some(art.parts[row * 3 + col].at(du as usize, dv as usize))
    });
    let (pl, pt, _, _) = SCROLL_PAD;
    let (sx, sy) = (x0 + pl, y0 + pt);
    if capital {
        let s = &art.star;
        out.extend(layers_of(
            size,
            (sx, sx + s.w as i32, sy, sy + s.h as i32),
            |x, y| {
                let t = s.at((x - sx) as usize, (y - sy) as usize);
                let m = |k: usize| (u16::from(t[k]) * u16::from(tint[k]) / 255) as u8;
                Some([m(0), m(1), m(2), t[3]])
            },
        ));
    }
    let (tx, ty) = if capital {
        (sx + CAPITAL_STAR.0, sy + 3)
    } else {
        (sx, sy)
    };
    let cells: Vec<(i32, GlyphCell)> = name
        .chars()
        .scan(tx, |x, c| {
            let at = *x;
            *x += art.face.advance(c).unwrap_or(0);
            Some(art.face.cell(c).map(|cell| (at, cell)))
        })
        .flatten()
        .collect();
    let right = cells
        .iter()
        .map(|(at, c)| at + c.w as i32)
        .max()
        .unwrap_or(tx);
    let (cw, chh) = ((right - tx).max(0) as usize, PLATE_FONT_HEIGHT as usize);
    // Each quad blends over the last: coverage composes as alphas do.
    let mut cover = vec![0u16; cw * chh];
    for (at, cell) in &cells {
        for v in 0..chh {
            for u in 0..cell.w {
                let a = u16::from(cell.cover[v * cell.w + u]);
                let k = v * cw + (at - tx) as usize + u;
                cover[k] = cover[k] + a - cover[k] * a / 255;
            }
        }
    }
    let ink = PLATE_INK.map(|v| (v * 255.0) as u8);
    out.extend(layers_of(
        size,
        (tx, right, ty, ty + PLATE_FONT_HEIGHT),
        |x, y| {
            let a = cover[(y - ty) as usize * cw + (x - tx) as usize] as u8;
            Some([ink[0], ink[1], ink[2], a])
        },
    ));
    out
}

/// **The plates of the scene** (item 4410): `BuildOut::draw_overlays
/// @0062cb30`'s, for every finished city centre (`flags & 4`, `& 0x20`)
/// the viewer sees, selected or not, whose name `name_of(city)` gives and
/// whose width [`PlateFace::text_width`] holds. The capital's bit is the city
/// record's (`CityData +4 & 0x10`). `player_profile.prefs & 0x40000` is
/// taken as set, the default the photographs show.
pub fn plates(
    world: &crate::world::World,
    camera: &render::Camera,
    size: (i32, i32),
    name_of: &dyn Fn(usize) -> Option<String>,
) -> Vec<crate::gpu::ScreenLayer> {
    let Some(art) = world.scroll_art.as_ref() else {
        return Vec::new();
    };
    let sim = world.runner.sim();
    let scene = world.runner.scene(None);
    let placed = world.placed_buildings();
    let mut out = Vec::new();
    for s in scene
        .buildings
        .iter()
        .filter(|s| s.city && s.active && s.seen)
    {
        let Some(c) = sim
            .building_by_o(s.id.who, s.id.o)
            .and_then(|b| sim.buildings[b].city)
        else {
            continue;
        };
        let Some(b) = placed.iter().find(|b| (b.who, b.o) == (s.id.who, s.id.o)) else {
            continue;
        };
        let Some(&(_, y_size)) = usize::try_from(b.type_index - rondata::load::BASE_BUILDTYPES)
            .ok()
            .and_then(|ti| world.names.footprints.get(ti))
        else {
            continue;
        };
        let Some(name) = name_of(c) else { continue };
        let Some(text_w) = art.face.text_width(&name) else {
            continue;
        };
        let capital = sim.cities.cities.get(c).is_some_and(|k| k.capital);
        let r = plate(camera, b, y_size, text_w, PLATE_FONT_HEIGHT, capital);
        let tint = render::silhouette::team_unit(b.who);
        out.extend(plate_layers(art, size, r, capital, tint, &name));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ground::tests::{probe_at, run1559b};
    use crate::silhouette::INTERFACE;

    /// The selected city on run1559b's photograph: slot 0's object 2000,
    /// the one member of the console's `play` group in the packet
    /// (`select_groups +0x14`'s group 0: `num` 1, `buildings` 1, `who` 0,
    /// `list[0]` 2000; `Console::target_o` −1, so the outline is the
    /// selection's, not a target's).
    pub(crate) const CITY: (u8, i16) = (0, 2000);

    /// **Each pixel whose centre lies in a triangle**, the triangles
    /// projected through the still's own `to_screen_matrix` onto a
    /// `size` target, less the photograph's [`INTERFACE`].
    pub(crate) fn mask(
        camera: &render::Camera,
        size: (i32, i32),
        tris: &[[(i32, i32, i32); 3]],
    ) -> Vec<bool> {
        let (w, h) = size;
        let m = camera.to_screen_matrix().0;
        let mut out = vec![false; (w * h) as usize];
        for tri in tris {
            let mut q = [[0.0f32; 2]; 3];
            let mut behind = false;
            for (k, &(x, y, z)) in tri.iter().enumerate() {
                let (x, y, z) = (x as f32, y as f32, z as f32);
                let cx = m[0] * x + m[4] * y + m[8] * z + m[12];
                let cy = m[1] * x + m[5] * y + m[9] * z + m[13];
                let cw = m[3] * x + m[7] * y + m[11] * z + m[15];
                behind |= cw <= 0.0;
                q[k] = [
                    (cx / cw + 1.0) * w as f32 * 0.5,
                    (1.0 - cy / cw) * h as f32 * 0.5,
                ];
            }
            let edge = |a: [f32; 2], b: [f32; 2], x: f32, y: f32| {
                (b[0] - a[0]) * (y - a[1]) - (b[1] - a[1]) * (x - a[0])
            };
            let area = edge(q[0], q[1], q[2][0], q[2][1]);
            if behind || area == 0.0 {
                continue;
            }
            let lo = |c: usize| q.iter().map(|v| v[c]).fold(f32::MAX, f32::min);
            let hi = |c: usize| q.iter().map(|v| v[c]).fold(f32::MIN, f32::max);
            for py in (lo(1).floor() as i32).max(0)..=(hi(1).ceil() as i32).min(h - 1) {
                for px in (lo(0).floor() as i32).max(0)..=(hi(0).ceil() as i32).min(w - 1) {
                    let (fx, fy) = (px as f32 + 0.5, py as f32 + 0.5);
                    let b = [
                        edge(q[1], q[2], fx, fy) / area,
                        edge(q[2], q[0], fx, fy) / area,
                        edge(q[0], q[1], fx, fy) / area,
                    ];
                    let hud = INTERFACE
                        .iter()
                        .any(|r| px >= r.x0 && px < r.x1 && py >= r.y0 && py < r.y1);
                    if b.iter().all(|&v| v >= 0.0) && !hud {
                        out[(py * w + px) as usize] = true;
                    }
                }
            }
        }
        out
    }

    /// Theirs reads the line where red stands [`RED`] over both green and
    /// blue: the comparison, never the footprint (the footprint is
    /// [`band`]'s).
    pub(crate) const RED: i32 = 40;

    fn red(rgba: &[u8], i: usize) -> bool {
        let (r, g, b) = (
            rgba[i * 4] as i32,
            rgba[i * 4 + 1] as i32,
            rgba[i * 4 + 2] as i32,
        );
        r - g > RED && r - b > RED
    }

    /// The city's rectangle, and the band's footprint at an offset of
    /// `(dx, dy)` world units and a back of `k`.
    fn city_band(p: &mut crate::ground::tests::Probe, d: (i32, i32), k: Option<i32>) -> Vec<bool> {
        let b = p
            .world
            .placed_buildings()
            .into_iter()
            .find(|b| (b.who, b.o) == CITY)
            .expect("the city is in the scene");
        let ti = (b.type_index - rondata::load::BASE_BUILDTYPES) as usize;
        let size = p.world.names.footprints[ti];
        let back = k.unwrap_or(b.foundation.map_or(b.half_offset, |f| f.back));
        let r = rectangle(b.x + d.0, b.y + d.1, size, back);
        println!(
            "city: type {ti} size {size:?} back {back} point ({}, {}) rectangle {r:?}",
            b.x, b.y
        );
        let t = p.world.runner.terrain();
        let field = Heightfield {
            width: t.width,
            height: t.height,
            corners: &t.corners,
        };
        let tris = band(&field, r);
        let size = (p.photo.width as i32, p.photo.height as i32);
        mask(&p.camera, size, &tris)
    }

    /// **`RUN1559B_3279_SELECTED`** (item 4398): the selected city's band
    /// on run1559b's photograph, its footprint [`band`]'s from our own
    /// geometry. Per arm, `[footprint, theirs red]`: as the listing places
    /// it; moved one tile east; and with no back (`k` 0). The red the
    /// photograph holds on the first arm and not on the others is the
    /// row's evidence that the footprint is theirs. Then, on the first
    /// arm's footprint, `[ours red, both red]` on the client's still with
    /// nothing selected (item 4398), and then with the city selected, its
    /// band drawn by [`underlay`]: the half the drawing moves. Item 4401
    /// drew it solid, `[2132, 1404]`; item 4421 through `city_radius.tga`'s
    /// rows, the soft line theirs is: `[1395, 1300]` (ours' red off theirs
    /// 728 → 95).
    pub const RUN1559B_3279_SELECTED: ([[usize; 2]; 3], [usize; 2], [usize; 2]) = (
        [[4551, 1477], [4157, 721], [5398, 312]],
        [52, 9],
        [1395, 1300],
    );

    /// **The band's texture is one tent** (item 4421): the install's first
    /// `<SELECTIONCIRCLE>`, 32 rows, alpha 0 at the top row and 255 at row
    /// 16, rising to it and falling after it.
    #[test]
    fn the_band_s_texture_is_a_tent_peaking_at_mid_band() {
        let root = std::env::var_os("RON_INSTALL")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("../../game"));
        let Some(BandAlphas(a)) = BandAlphas::load(&root) else {
            println!("skipping: no effects_graphics.xml under {}", root.display());
            return;
        };
        assert_eq!(a.len(), 32);
        assert_eq!((a[0], a[16]), (0, 255));
        assert!(a[..=16].windows(2).all(|w| w[0] < w[1]), "{a:?}");
        assert!(a[16..].windows(2).all(|w| w[0] > w[1]), "{a:?}");
    }

    /// **The rows tile each triangle** (item 4421): [`clip_v`]'s pieces
    /// over the 32 rows cover a triangle's area once, with `v` 0 at one
    /// corner and 1 at the other two as a corner square's is, and the
    /// strip `[0, 1/2]` holds a quarter of it (`v ≤ 1/2` is a triangle a
    /// half the size on each side).
    #[test]
    fn the_rows_tile_a_triangle_once() {
        let area = |t: &[[f32; 3]; 3]| {
            let (u, w) = (
                [0, 1, 2].map(|k| t[1][k] - t[0][k]),
                [0, 1, 2].map(|k| t[2][k] - t[0][k]),
            );
            let c = [
                u[1] * w[2] - u[2] * w[1],
                u[2] * w[0] - u[0] * w[2],
                u[0] * w[1] - u[1] * w[0],
            ];
            (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt() / 2.0
        };
        let tri = [
            ([0.0, 0.0, 10.0], 0.0),
            ([48.0, 0.0, 12.0], 1.0),
            ([48.0, 48.0, 14.0], 1.0),
        ];
        let whole = area(&tri.map(|(p, _)| p));
        let rows: f32 = (0..32)
            .flat_map(|r| clip_v(&tri, r as f32 / 32.0, (r + 1) as f32 / 32.0))
            .map(|t| area(&t))
            .sum();
        assert!((rows - whole).abs() < whole * 1e-4, "{rows} of {whole}");
        let half: f32 = clip_v(&tri, 0.0, 0.5).iter().map(area).sum();
        assert!((half - whole / 4.0).abs() < whole * 1e-4, "{half}");
    }

    #[test]
    fn the_selected_city_s_band_on_run1559b() {
        let Some(mut p) = probe_at(run1559b()) else {
            return;
        };
        let arms = [
            ((0, 0), None),
            ((2 * HALF_TILE, 0), None),
            ((0, 0), Some(0)),
        ];
        let mut got = [[0; 2]; 3];
        let mut first = Vec::new();
        for (k, (d, back)) in arms.into_iter().enumerate() {
            let foot = city_band(&mut p, d, back);
            if let Some(dir) = std::env::var_os("RON_STILL_DIR") {
                let rgba: Vec<u8> = foot
                    .iter()
                    .flat_map(|f| {
                        if *f {
                            [255, 255, 255, 255]
                        } else {
                            [0, 0, 0, 255]
                        }
                    })
                    .collect();
                let out = std::path::PathBuf::from(dir).join(format!("foot-selected-{k}.bmp"));
                let size = (p.photo.width as u32, p.photo.height as u32);
                std::fs::write(&out, crate::gpu::bmp(&rgba, size)).expect("the footprint writes");
            }
            let n = foot.iter().filter(|f| **f).count();
            let r = (0..foot.len())
                .filter(|&i| foot[i] && red(&p.photo.rgba, i))
                .count();
            got[k] = [n, r];
            if k == 0 {
                first = foot;
            }
        }
        let size = (p.photo.width as u32, p.photo.height as u32);
        let camera = p.camera;
        let ours = crate::still_of(
            &mut p.world,
            &camera,
            size,
            crate::gpu::SilhouetteOrder::Before,
            wgpu::TextureFormat::Rgba8Unorm,
        );
        let selected = crate::still_selected(
            &mut p.world,
            &camera,
            size,
            crate::gpu::SilhouetteOrder::Before,
            wgpu::TextureFormat::Rgba8Unorm,
            &[sim::observe::ObjId {
                who: CITY.0,
                o: CITY.1,
            }],
        );
        if let Some(dir) = std::env::var_os("RON_STILL_DIR") {
            let out = std::path::PathBuf::from(dir).join("still-selected.bmp");
            std::fs::write(&out, crate::gpu::bmp(&selected, size)).expect("the still writes");
        }
        let count = |rgba: &[u8]| {
            let on = |i: usize| first[i] && red(rgba, i);
            let mine = (0..first.len()).filter(|&i| on(i)).count();
            let both = (0..first.len())
                .filter(|&i| on(i) && red(&p.photo.rgba, i))
                .count();
            [mine, both]
        };
        let got = (got, count(&ours), count(&selected));
        println!("RUN1559B_3279_SELECTED: {got:?}");
        assert_eq!(got, RUN1559B_3279_SELECTED);
    }

    /// **The name's width on run1559b**: `Napata`'s advances in the
    /// plate's font, 12 + 8 + 9 + 8 + 6 + 8, each its `CharacterInfo`'s
    /// `advance` in the packet's `D3D11GLFont` (item 4403).
    const NAPATA_WIDTH: i32 = 51;

    /// The plate's body reads where every channel is under [`DARK`]: the
    /// scroll's black ground (16, 15, 13 on the photograph), never the
    /// footprint.
    const DARK: i32 = 40;

    fn dark(rgba: &[u8], i: usize) -> bool {
        rgba[i * 4..i * 4 + 3].iter().all(|&c| i32::from(c) < DARK)
    }

    /// The plate's name reads where every channel is over [`LIGHT`]: the
    /// white ink on the dark body, which the photograph's translucent
    /// "Reduced!" banner leaves bare in places (item 4426).
    const LIGHT: i32 = 150;

    fn light(rgba: &[u8], i: usize) -> bool {
        rgba[i * 4..i * 4 + 3].iter().all(|&c| i32::from(c) > LIGHT)
    }

    /// The plate's footprint, its rectangle moved `d` pixels. It is not
    /// masked by [`INTERFACE`], whose "city's label" rectangle is this
    /// plate, taken out by eye for the scene's rows.
    fn plate_foot(p: &crate::ground::tests::Probe, d: (i32, i32), capital: bool) -> Vec<bool> {
        let b = p
            .world
            .placed_buildings()
            .into_iter()
            .find(|b| (b.who, b.o) == CITY)
            .expect("the city is in the scene");
        let ti = (b.type_index - rondata::load::BASE_BUILDTYPES) as usize;
        let size = p.world.names.footprints[ti];
        let (x0, x1, y0, y1) = plate(
            &p.camera,
            &b,
            size.1,
            NAPATA_WIDTH,
            PLATE_FONT_HEIGHT,
            capital,
        );
        println!(
            "plate: point ({}, {}, {}) y_size {} rect x [{x0}, {x1}) y [{y0}, {y1})",
            b.x, b.y, b.z, size.1
        );
        let (w, h) = (p.photo.width as i32, p.photo.height as i32);
        let mut out = vec![false; (w * h) as usize];
        for py in (y0 + d.1).max(0)..(y1 + d.1).min(h) {
            for px in (x0 + d.0).max(0)..(x1 + d.0).min(w) {
                out[(py * w + px) as usize] = true;
            }
        }
        out
    }

    /// **`RUN1559B_3279_PLATE`** (item 4403): the capital's plate on
    /// run1559b's photograph, its footprint [`plate`]'s from the packet's
    /// pads, star, font height and the name's advances. Per arm,
    /// `[footprint, theirs dark]`: as the listing places it; moved one
    /// plate-height (30 px) down; and without the capital's star. Then, on
    /// the first arm's footprint, `[ours dark, both dark]` on the
    /// client's still: the half the drawing moves. Item 4410 drew the
    /// plate ([`plate_layers`]): `[0, 0]` → `[1742, 1251]`; drawn 89 px
    /// east, the pin failed. Then `[theirs light, ours light, both
    /// light]`, the name's white ink (item 4426): the overlay's 5 × 7
    /// lit 9 of the photograph's 53; the face's glyphs ([`PlateFace`])
    /// light 47 (and the dark half `[1742, 1251]` → `[1537, 1233]`);
    /// hinted as the DLL's FreeType hints them (item 4427), 47 → 53 of 53,
    /// ours light 192 → 209, the dark half → `[1541, 1238]`. Made to fail:
    /// the same path drawn unhinted reads `[53, 193, 47]`.
    pub const RUN1559B_3279_PLATE: ([[usize; 2]; 3], [usize; 2], [usize; 3]) = (
        [[2670, 1285], [2670, 0], [1608, 1018]],
        [1541, 1238],
        [53, 209, 53],
    );

    #[test]
    fn the_capital_s_plate_on_run1559b() {
        let Some(mut p) = probe_at(run1559b()) else {
            return;
        };
        let arms = [((0, 0), true), ((0, 30), true), ((0, 0), false)];
        let mut got = [[0; 2]; 3];
        let mut first = Vec::new();
        for (k, (d, capital)) in arms.into_iter().enumerate() {
            let foot = plate_foot(&p, d, capital);
            let n = foot.iter().filter(|f| **f).count();
            let r = (0..foot.len())
                .filter(|&i| foot[i] && dark(&p.photo.rgba, i))
                .count();
            got[k] = [n, r];
            if k == 0 {
                first = foot;
            }
        }
        let size = (p.photo.width as u32, p.photo.height as u32);
        let camera = p.camera;
        // The capital's city is `Napata` on the packet (item 4403: city 0
        // of slot 0, `CityData +0x90`), and the sim names it so at its
        // founding (item 4418, `sim::leaders::words::founding_name`): the
        // row draws the sim's names, as the live client does.
        let sim = p.world.runner.sim();
        let napata = sim
            .building_by_o(CITY.0, CITY.1)
            .and_then(|b| sim.buildings[b].city)
            .expect("the capital is in a city");
        assert!(
            sim.cities.cities[napata].capital,
            "the city's record carries the capital's bit"
        );
        assert_eq!(
            sim.city_name(napata),
            Some("Napata"),
            "the sim's founding name is the packet's"
        );
        let names = sim.cities.names.clone();
        assert!(p.world.scroll_art.is_some(), "the plate's art loads");
        let ours = crate::still_named(
            &mut p.world,
            &camera,
            size,
            crate::gpu::SilhouetteOrder::Before,
            wgpu::TextureFormat::Rgba8Unorm,
            &[sim::observe::ObjId {
                who: CITY.0,
                o: CITY.1,
            }],
            &|c| names.get(&c).cloned(),
        );
        if let Some(dir) = std::env::var_os("RON_STILL_DIR") {
            let out = std::path::PathBuf::from(dir).join("still-plate.bmp");
            std::fs::write(&out, crate::gpu::bmp(&ours, size)).expect("the still writes");
        }
        let on = |i: usize| first[i] && dark(&ours, i);
        let mine = (0..first.len()).filter(|&i| on(i)).count();
        let both = (0..first.len())
            .filter(|&i| on(i) && dark(&p.photo.rgba, i))
            .count();
        let lit = |rgba: &[u8], i: usize| first[i] && light(rgba, i);
        let theirs_lit = (0..first.len()).filter(|&i| lit(&p.photo.rgba, i)).count();
        let ours_lit = (0..first.len()).filter(|&i| lit(&ours, i)).count();
        let both_lit = (0..first.len())
            .filter(|&i| lit(&ours, i) && lit(&p.photo.rgba, i))
            .count();
        let got = (got, [mine, both], [theirs_lit, ours_lit, both_lit]);
        println!("RUN1559B_3279_PLATE: {got:?}");
        assert_eq!(got, RUN1559B_3279_PLATE);
    }

    /// The bar's footprint: [`bar`]'s rectangle one pixel out on every
    /// side (the black quad), moved `d` pixels, the point lifted `lift`.
    fn bar_foot(p: &crate::ground::tests::Probe, d: (i32, i32), lift: bool) -> Vec<bool> {
        let mut b = p
            .world
            .placed_buildings()
            .into_iter()
            .find(|b| (b.who, b.o) == CITY)
            .expect("the city is in the scene");
        if !lift {
            b.z -= 0x180;
        }
        let (x0, x1, y0, y1) = bar(&p.camera, &b, BAR_ZOOM_LEVEL);
        println!(
            "bar: point ({}, {}, {}) rect x [{x0}, {x1}) y [{y0}, {y1})",
            b.x, b.y, b.z
        );
        let (w, h) = (p.photo.width as i32, p.photo.height as i32);
        let mut out = vec![false; (w * h) as usize];
        for py in (y0 - 1 + d.1).max(0)..(y1 + 1 + d.1).min(h) {
            for px in (x0 - 1 + d.0).max(0)..(x1 + 1 + d.0).min(w) {
                out[(py * w + px) as usize] = true;
            }
        }
        out
    }

    /// The bar's black reads where every channel is under [`BLACK`]: the
    /// frame `GraphicPieces::draw_health_bar` lays in `BLACK`, never the
    /// footprint.
    const BLACK: i32 = 12;

    fn black(rgba: &[u8], i: usize) -> bool {
        rgba[i * 4..i * 4 + 3].iter().all(|&c| i32::from(c) < BLACK)
    }

    /// **`RUN1559B_3279_BAR`** (item 4410): the capital's health bar on
    /// run1559b's photograph, its footprint [`bar`]'s at the packet's zoom
    /// level, one pixel out. Per arm, `[footprint, theirs black]`: as the
    /// listing places it; moved one bar (4 px) down; and without the
    /// `0x180` lift. Then, on the first arm's footprint, `[ours black,
    /// both black, both red]` on the client's still: the half the drawing
    /// moves (item 4410: `[0, 0, 0]` before the draw, `[86, 86, 2]` with
    /// it; the fill drawn green read 0 red, and the bar drawn without the
    /// lift read 0 black, and each failed the pin).
    pub const RUN1559B_3279_BAR: ([[usize; 2]; 3], [usize; 3]) =
        ([[88, 86], [88, 0], [88, 0]], [86, 86, 2]);

    #[test]
    fn the_capital_s_bar_on_run1559b() {
        let Some(mut p) = probe_at(run1559b()) else {
            return;
        };
        let scene = p.world.runner.scene(None);
        let s = scene
            .buildings
            .iter()
            .find(|s| (s.id.who, s.id.o) == CITY)
            .expect("the city is in the scene");
        println!(
            "bar: health {} hits {} construct_hits {} level {}",
            s.health,
            s.hits,
            s.construct_hits,
            health_level(s.health, s.construct_hits)
        );
        let arms = [((0, 0), true), ((0, 4), true), ((0, 0), false)];
        let mut got = [[0; 2]; 3];
        let mut first = Vec::new();
        for (k, (d, lift)) in arms.into_iter().enumerate() {
            let foot = bar_foot(&p, d, lift);
            let n = foot.iter().filter(|f| **f).count();
            let r = (0..foot.len())
                .filter(|&i| foot[i] && black(&p.photo.rgba, i))
                .count();
            got[k] = [n, r];
            if k == 0 {
                first = foot;
            }
        }
        let size = (p.photo.width as u32, p.photo.height as u32);
        let camera = p.camera;
        let ours = crate::still_selected(
            &mut p.world,
            &camera,
            size,
            crate::gpu::SilhouetteOrder::Before,
            wgpu::TextureFormat::Rgba8Unorm,
            &[sim::observe::ObjId {
                who: CITY.0,
                o: CITY.1,
            }],
        );
        if let Some(dir) = std::env::var_os("RON_STILL_DIR") {
            let out = std::path::PathBuf::from(dir).join("still-bar.bmp");
            std::fs::write(&out, crate::gpu::bmp(&ours, size)).expect("the still writes");
        }
        let on = |i: usize| first[i] && black(&ours, i);
        let mine = (0..first.len()).filter(|&i| on(i)).count();
        let both = (0..first.len())
            .filter(|&i| on(i) && black(&p.photo.rgba, i))
            .count();
        let reds = (0..first.len())
            .filter(|&i| first[i] && red(&ours, i) && red(&p.photo.rgba, i))
            .count();
        let got = (got, [mine, both, reds]);
        println!("RUN1559B_3279_BAR: {got:?}");
        assert_eq!(got, RUN1559B_3279_BAR);
    }

    /// One packet's line: `run map`, `[height, point, style]`, and each
    /// held `(code, advance, [pix_width, pix_height, abs_a])`, the last
    /// `None` on a file written before item 4426.
    type PacketFont = (String, [i32; 3], Vec<(u32, i32, Option<[i32; 3]>)>);

    /// The packets' plate font (`tools/render/plate_font_packet.py` over
    /// every `frame-snapshot.bin` under `~/ron-golden` and `~/ron-data`):
    /// per line `run map packet_frame height point style` and then
    /// `code:advance:pix_width:pix_height:abs_a` for each character the
    /// font holds, at `$RON_PLATE_FONT` or item 4426's file.
    fn packet_fonts() -> Option<Vec<PacketFont>> {
        let path = std::env::var_os("RON_PLATE_FONT")
            .map(std::path::PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|h| {
                    std::path::PathBuf::from(h)
                        .join("ron-data/lab-experiments/2026-11-19-item-4426/plate-font.txt")
                })
            })?;
        let Ok(text) = std::fs::read_to_string(&path) else {
            println!("skipping: no packet plate font at {}", path.display());
            return None;
        };
        Some(
            text.lines()
                .map(|l| {
                    let f: Vec<&str> = l.split_whitespace().collect();
                    let n = |i: usize| f[i].parse::<i32>().expect("a number");
                    let held = f[6..]
                        .iter()
                        .map(|e| {
                            let v: Vec<i32> =
                                e.split(':').map(|x| x.parse().expect("a number")).collect();
                            let cell = (v.len() == 5).then(|| [v[2], v[3], v[4]]);
                            (v[0] as u32, v[1], cell)
                        })
                        .collect();
                    (format!("{} {}", f[0], f[1]), [n(3), n(4), n(5)], held)
                })
                .collect(),
        )
    }

    /// **`PLATE_FONT_PACKETS`** (item 4423): [`PLATE_ADVANCES`] against
    /// the plate's `D3D11GLFont` on every packet on disk, as `[entries
    /// agreeing, entries held, packets]`. Each packet holds the characters
    /// its game has drawn (65 to 73 of them), so the agreement is per
    /// entry; the table is also exactly the characters some packet holds,
    /// so none of its 77 is a guess. Before item 4423 the table held `N`,
    /// `a`, `p`, `t` (4 characters). Made to fail: `o`'s advance 9 → 10
    /// parts on all 144 packets.
    pub const PLATE_FONT_PACKETS: [usize; 3] = [9849, 9849, 144];

    #[test]
    fn the_plate_s_advances_are_the_packets_font() {
        let Some(packets) = packet_fonts() else {
            return;
        };
        let mut seen = std::collections::BTreeSet::new();
        let (mut agree, mut held) = (0, 0);
        for (run, font, chars) in &packets {
            assert_eq!(
                *font,
                [PLATE_FONT_HEIGHT, 14, 1],
                "{run}: the plate's font is height 16, point 14, style 1"
            );
            for &(c, a, _) in chars {
                let c = char::from_u32(c).expect("a character");
                held += 1;
                seen.insert(c);
                if plate_advance(c) == Some(a) {
                    agree += 1;
                } else {
                    println!("{run}: {c:?} theirs {a}, ours {:?}", plate_advance(c));
                }
            }
        }
        let table: std::collections::BTreeSet<char> =
            PLATE_ADVANCES.iter().map(|&(c, _)| c).collect();
        assert_eq!(seen, table, "the table is the characters the packets hold");
        assert!(PLATE_ADVANCES.windows(2).all(|w| w[0].0 < w[1].0));
        let got = [agree, held, packets.len()];
        println!("PLATE_FONT_PACKETS: {got:?}");
        assert_eq!(got, PLATE_FONT_PACKETS);
        assert_eq!(plate_text_width("Napata"), Some(NAPATA_WIDTH));
        // London: 10 + 9 + 10 + 9 + 9 + 10.
        assert_eq!(plate_text_width("London"), Some(57));
        assert_eq!(plate_text_width("Zela"), None);
    }

    /// **`PLATE_FACE_PACKETS`** (item 4426): [`PlateFace`] off the
    /// install's `georgiab.ttf` against the plate's `D3D11GLFont` on every
    /// packet, as `[advances agreeing, cells agreeing, entries held,
    /// packets]`. A cell agrees when its width is the entry's `pix_width`,
    /// and the entry's `pix_height` is 16 and its `abs_a` 0. Made to fail:
    /// the advance truncated rather than rounded, 5,322 of 9,849 agree.
    /// Hinted (item 4427), every cell keeps the packet's width.
    pub const PLATE_FACE_PACKETS: [usize; 4] = [9849, 9849, 9849, 144];

    #[test]
    fn the_plate_s_face_is_the_packets_font() {
        let Some(root) = std::env::var_os("RON_INSTALL").map(std::path::PathBuf::from) else {
            println!("skipping: no RON_INSTALL for the plate's face");
            return;
        };
        let Some(packets) = packet_fonts() else {
            return;
        };
        let face = PlateFace::load(&root).expect("the plate's face loads");
        let (mut adv, mut cells, mut held) = (0, 0, 0);
        for (run, _, chars) in &packets {
            for &(c, a, cell) in chars {
                let ch = char::from_u32(c).expect("a character");
                let [pw, ph, abs_a] = cell.expect("the file holds each entry's cell");
                held += 1;
                if face.advance(ch) == Some(a) {
                    adv += 1;
                } else {
                    println!(
                        "{run}: {ch:?} advance theirs {a}, ours {:?}",
                        face.advance(ch)
                    );
                }
                let w = face.cell(ch).map(|g| g.w as i32);
                if w == Some(pw) && ph == PLATE_FONT_HEIGHT && abs_a == 0 {
                    cells += 1;
                } else {
                    println!("{run}: {ch:?} cell theirs {pw}x{ph}+{abs_a}, ours {w:?}");
                }
            }
        }
        let got = [adv, cells, held, packets.len()];
        println!("PLATE_FACE_PACKETS: {got:?}");
        assert_eq!(got, PLATE_FACE_PACKETS);
        // The table's 77 are the face's; X and Z, which no packet holds,
        // take the face's own (the derivation, not a reading).
        for &(c, a) in &PLATE_ADVANCES {
            assert_eq!(face.advance(c), Some(a), "{c:?}");
        }
        assert_eq!(face.text_width("Napata"), Some(51));
        assert!(face.text_width("Zela").is_some());
    }
}
