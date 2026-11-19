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
//! vertex at `TerrainOut::find_data_z` of its own point. The colour, the
//! texture (`int_str_array +0x1784`) and the alpha are the draw's, not
//! this module's (§3, "What is not established").

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
            let (ai, bi) = (at(a.0, a.1), at(b.0, b.1));
            let (ao, bo) = (at(a.0 + out_x, a.1 + out_y), at(b.0 + out_x, b.1 + out_y));
            out.push([ai, bi, bo]);
            out.push([ai, bo, ao]);
        }
        // The corner square past the side's end.
        let e = (sx + dx * len, sy + dy * len);
        let c = [
            at(e.0, e.1),
            at(e.0 + dx * BAND, e.1 + dy * BAND),
            at(e.0 + dx * BAND + out_x, e.1 + dy * BAND + out_y),
            at(e.0 + out_x, e.1 + out_y),
        ];
        out.push([c[0], c[1], c[2]]);
        out.push([c[0], c[2], c[3]]);
    }
    out
}

/// **The selected buildings' bands as drawn** (item 4401): for each
/// building of `selection` in the scene, [`band`] about its
/// [`rectangle`], every vertex in the owner's unit colour
/// (`render::silhouette::team_unit`, `TeamColor::get_unit`'s), as world
/// triangles for the GPU's underlay pass, which blends them at
/// [`crate::gpu::UNDERLAY_ALPHA`]. A unit draws no band here (its ring is
/// [`crate::hud::rings`]'). The listing's alpha 0 for a vertex off the map
/// is not carried: the pass's alpha is one constant.
pub fn underlay(
    world: &crate::world::World,
    selection: &[sim::observe::ObjId],
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>) {
    let (mut pos, mut col) = (Vec::new(), Vec::new());
    if selection.is_empty() {
        return (pos, col);
    }
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
        for tri in band(&field, rectangle(b.x, b.y, size, back)) {
            for (x, y, z) in tri {
                pos.push([x as f32, y as f32, z as f32]);
                col.push(c);
            }
        }
    }
    (pos, col)
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

/// **The plate's font's advances** (item 4403): each character's
/// `CharacterInfo::advance` in `onscreen_mouseover_size`'s `D3D11GLFont`
/// on run1559b's packet, for the characters `Napata` holds and no others.
/// A name with any other character has no width here, and no plate is
/// drawn for it rather than one guessed (`docs/HUD.md` §6).
pub const PLATE_ADVANCES: [(char, i32); 4] = [('N', 12), ('a', 8), ('p', 9), ('t', 6)];

/// The name's `text_width` in the plate's font, or `None` when a
/// character's advance is not held ([`PLATE_ADVANCES`]).
pub fn plate_text_width(name: &str) -> Option<i32> {
    name.chars()
        .map(|c| PLATE_ADVANCES.iter().find(|(k, _)| *k == c).map(|(_, a)| a))
        .sum()
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
        Ok(ScrollArt { parts, star })
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
/// character at its advance. The glyphs are not the font's: the overlay's
/// 5 × 7 stands in, white, on the font's height (`docs/HUD.md` §6).
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
    let mut p = crate::overlay::Painter::new(size);
    let mut x = tx;
    for c in name.chars() {
        p.text(
            x,
            ty + (PLATE_FONT_HEIGHT - 7) / 2,
            &c.to_string(),
            1,
            PLATE_INK,
        );
        x += PLATE_ADVANCES
            .iter()
            .find(|(k, _)| *k == c)
            .map_or(0, |(_, a)| *a);
    }
    out.push(crate::gpu::ScreenLayer {
        alpha: 255,
        pos: p.pos,
        col: p.col,
    });
    out
}

/// **The plates of the scene** (item 4410): `BuildOut::draw_overlays
/// @0062cb30`'s, for every finished city centre (`flags & 4`, `& 0x20`)
/// the viewer sees, selected or not, whose name `name_of(city)` gives and
/// whose width [`plate_text_width`] holds. The capital's bit is the city
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
        let Some(text_w) = plate_text_width(&name) else {
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
    /// band drawn by [`underlay`] (item 4401): the half the drawing moves.
    pub const RUN1559B_3279_SELECTED: ([[usize; 2]; 3], [usize; 2], [usize; 2]) = (
        [[4551, 1477], [4157, 721], [5398, 312]],
        [52, 9],
        [2132, 1404],
    );

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
    /// east, the pin failed.
    pub const RUN1559B_3279_PLATE: ([[usize; 2]; 3], [usize; 2]) =
        ([[2670, 1285], [2670, 0], [1608, 1018]], [1742, 1251]);

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
        // of slot 0, `CityData +0x90`); the sim does not carry a founding
        // name, so the row names it, and only it.
        let sim = p.world.runner.sim();
        let napata = sim
            .building_by_o(CITY.0, CITY.1)
            .and_then(|b| sim.buildings[b].city)
            .expect("the capital is in a city");
        assert!(
            sim.cities.cities[napata].capital,
            "the city's record carries the capital's bit"
        );
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
            &|c| (c == napata).then(|| "Napata".to_string()),
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
        let got = (got, [mine, both]);
        println!("RUN1559B_3279_PLATE: {got:?}");
        assert_eq!(got, RUN1559B_3279_PLATE);
    }
}
