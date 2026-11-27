//! **The population box at the top of the screen** (item 4739;
//! `docs/RENDERER.md` §6.15): `IFaceMainBase::draw_cap_area@00810370`,
//! `draw_pop_cap@0080ff90` and `draw_city_cap@0080fbe0`.
//!
//! `<IFACEMAINBASE>`'s `pop_cap` and `city_cap` rectangles are written
//! against the frame's right edge (`right_inverse`), from the screen's top.
//! In each, `draw_pop_cap` and `draw_city_cap` lay:
//! - the picture of the same name, a `TEX_COMMON` crop, `add_smooth`ed to
//!   the rectangle's height at its own aspect, the width truncated
//!   (29 · 17 / 19 = 25), at the rectangle's corner;
//! - the figure, the leader's population `/` its cap (`LeaderData +0x940`
//!   over `+0x7e4`) or its cities `/` their limit
//!   (`LeaderData::get_total_cities` over `get_city_limit`), in the render
//!   manager's `numbers_texs[0]`: `interface.txt`'s `NUMBERS_TEX[0][0..9]`,
//!   `SLASH_TEX[0]`, `PLUS_TEX[0]` and `MINUS_TEX[0]`, each a `TEX_COMMON`
//!   crop drawn 1:1 and advanced by its width (`IFaceRenderManager::write@0051f660`).
//!   The figure is centred between the picture's right and the
//!   rectangle's right less one, and between its top and bottom by the
//!   digits' height, both halvings truncated.
//!
//! On run1814b's photograph the digits of `6/25` and `1/1` register where
//! that puts them, (1748, 5) and (1835, 5), every opaque texel within two
//! levels.

use super::left::{Crop, Fill, Piece, Span, block, num, over, paint, resolve, tags};
use crate::selection::{Texels, attr};

/// The figures the box writes: population over its cap, cities over their
/// limit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Figures {
    pub pop: i32,
    pub pop_cap: i32,
    pub cities: i32,
    pub city_limit: i32,
}

/// **The box's layout**: the rectangles' tags, resolved at a frame's
/// size, and the crops of `TEX_COMMON` it draws.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapLayout {
    pub cap_area: String,
    pub pop_cap: String,
    pub city_cap: String,
    /// The pictures, `pop_cap` and `city_cap`'s `IFACETEX` off `TEX_COMMON`.
    pub pop_tex: Crop,
    pub city_tex: Crop,
    /// `numbers_texs[0]`: the ten digits, the slash, the plus, the minus.
    pub digits: [Crop; 13],
    /// `IFaceResources::item_back_tex`, `<IFACERESOURCES>`'s
    /// `resource_item_tex0`, `1` and `2` off `TEX_COMMON`: the back's left
    /// end, its middle and its right end.
    pub back: [Crop; 3],
    /// `draw_vert_divider`'s pieces off `TEX_CIV`: `THIN_VERT_BAR`, and
    /// `THIN_TEE[0]` and `THIN_TEE[2]`, its top and its foot.
    pub thin_vert_bar: Crop,
    pub thin_tee: [Crop; 2],
}

/// Which atlas a layer is cut from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Atlas {
    /// `TEX_COMMON`, every nation's.
    Common,
    /// `TEX_CIV`, the nation's.
    Civ,
}

/// **One layer of the box**: pieces off one atlas, drawn in white or in
/// the highlight's grey ([`highlight`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layer {
    pub atlas: Atlas,
    pub grey: Option<u8>,
    pub pieces: Vec<Piece>,
}

/// `draw_pop_cap`'s colour for the back, `0xff323232`, the grey of each
/// channel.
pub const BACK_GREY: u8 = 0x32;

/// **`IFACERENDERTYPE_HIGHLIGHT`'s modulate**: the texel times the colour,
/// doubled, held to 255. On run1814b's photograph the back's middle row,
/// the texture's (113, 105, 107) under `0x32`, reads (44, 41, 42).
pub fn highlight(texel: u8, colour: u8) -> u8 {
    ((u32::from(texel) * u32::from(colour) * 2 + 127) / 255).min(255) as u8
}

/// `IFaceRenderManager::char_to_index@0051f4a0`: a digit its value, `/`
/// 10, `+` 11, `-` 12; anything else 0, where the original reports.
pub fn char_to_index(c: char) -> usize {
    match c {
        '0'..='9' => c as usize - '0' as usize,
        '/' => 10,
        '+' => 11,
        '-' => 12,
        _ => 0,
    }
}

fn common_crop(tag: &str) -> Option<Crop> {
    (attr(tag, "texture") == Some("TEX_COMMON")).then_some(())?;
    let n = |k| num(tag, k).and_then(|v| usize::try_from(v).ok());
    Some((n("left")?, n("top")?, n("width")?, n("height")?))
}

/// The crop of `interface.txt`'s line `key`, `<key>, <texture>, <l> xleft,
/// <t> ytop, <w> xwidth, <h> yheight`.
fn txt_crop(txt: &str, key: &str, texture: &str) -> Result<Crop, String> {
    let line = txt
        .lines()
        .find(|l| l.split(',').next().map(str::trim) == Some(key))
        .ok_or(format!("interface.txt: no {key}"))?;
    let f: Vec<&str> = line.split(',').map(str::trim).collect();
    let n = |i: usize| -> Option<usize> {
        f.get(i)?
            .chars()
            .take_while(char::is_ascii_digit)
            .collect::<String>()
            .parse()
            .ok()
    };
    match (f.get(1), n(2), n(3), n(4), n(5)) {
        (Some(&tex), Some(l), Some(t), Some(w), Some(h)) if tex == texture => Ok((l, t, w, h)),
        _ => Err(format!("interface.txt: {key} reads {line:?}")),
    }
}

impl CapLayout {
    /// The box of `xml`'s `<IFACEMAINBASE>` and `txt`, `interface.txt`.
    pub fn parse(xml: &str, txt: &str) -> Result<CapLayout, String> {
        let base = tags(block(xml, "<IFACEMAINBASE ", "</IFACEMAINBASE>")?);
        let rect = |name: &str| {
            base.iter()
                .find(|t| t.starts_with("IFACERECT ") && attr(t, "name") == Some(name))
                .map(|t| t.trim().to_string())
                .ok_or(format!("interface.xml: no rectangle {name}"))
        };
        let tex = |name: &str| {
            base.iter()
                .find(|t| t.starts_with("IFACETEX ") && attr(t, "name") == Some(name))
                .and_then(|t| common_crop(t))
                .ok_or(format!("interface.xml: no texture {name} off TEX_COMMON"))
        };
        let mut digits = [(0, 0, 0, 0); 13];
        for (i, d) in digits.iter_mut().enumerate() {
            let key = match i {
                0..=9 => format!("NUMBERS_TEX[0][{i}]"),
                10 => "SLASH_TEX[0]".into(),
                11 => "PLUS_TEX[0]".into(),
                _ => "MINUS_TEX[0]".into(),
            };
            *d = txt_crop(txt, &key, "TEX_COMMON")?;
        }
        let res = tags(block(xml, "<IFACERESOURCES ", "</IFACERESOURCES>")?);
        let item = |name: &str| {
            res.iter()
                .find(|t| t.starts_with("IFACETEX ") && attr(t, "name") == Some(name))
                .and_then(|t| common_crop(t))
                .ok_or(format!("interface.xml: no {name} off TEX_COMMON"))
        };
        Ok(CapLayout {
            back: [
                item("resource_item_tex0")?,
                item("resource_item_tex1")?,
                item("resource_item_tex2")?,
            ],
            thin_vert_bar: txt_crop(txt, "THIN_VERT_BAR", "TEX_CIV")?,
            thin_tee: [
                txt_crop(txt, "THIN_TEE[0]", "TEX_CIV")?,
                txt_crop(txt, "THIN_TEE[2]", "TEX_CIV")?,
            ],
            cap_area: rect("cap_area")?,
            pop_cap: rect("pop_cap")?,
            city_cap: rect("city_cap")?,
            pop_tex: tex("pop_cap")?,
            city_tex: tex("city_cap")?,
            digits,
        })
    }

    /// `cap_area`, `pop_cap` and `city_cap` on a `size` frame.
    pub fn rects(&self, size: (i32, i32)) -> Option<[Span; 3]> {
        Some([
            resolve(&self.cap_area, size)?,
            resolve(&self.pop_cap, size)?,
            resolve(&self.city_cap, size)?,
        ])
    }

    /// **`line` in `numbers_texs[0]`** from `(x, y)`, each glyph 1:1.
    pub fn write(&self, line: &str, (mut x, y): (i32, i32)) -> Vec<Piece> {
        line.chars()
            .map(|c| {
                let crop = self.digits[char_to_index(c)];
                let (w, h) = (crop.2 as i32, crop.3 as i32);
                let p = Piece {
                    crop,
                    span: (x, y, x + w, y + h),
                    fill: Fill::Tile,
                };
                x += w;
                p
            })
            .collect()
    }

    /// `write_width`: the line's width, its glyphs' summed.
    pub fn width(&self, line: &str) -> i32 {
        line.chars()
            .map(|c| self.digits[char_to_index(c)].2 as i32)
            .sum()
    }

    /// **The back in `rect`** (`IFaceResources::draw_back@0081a920`, the
    /// render manager's `draw_expander@0051c450`): the left end and the
    /// right end at their own widths, stretched to the rectangle's height,
    /// and between them the middle tiled from the left end's edge
    /// (`tile_horiz@0051f1d0`), the last tile cut to the width that
    /// remains, each stretched only in height.
    pub fn back_pieces(&self, rect: Span) -> Vec<Piece> {
        let (x0, y0, x1, y1) = rect;
        let [l, m, r] = self.back;
        let (lw, mw, rw) = (l.2 as i32, m.2 as i32, r.2 as i32);
        let piece = |crop: Crop, a: i32, b: i32| Piece {
            crop,
            span: (a, y0, b, y1),
            fill: Fill::Stretch,
        };
        let mut v = vec![piece(l, x0, x0 + lw), piece(r, x1 - rw, x1)];
        let mut x = x0 + lw;
        while x < x1 - rw {
            let w = mw.min(x1 - rw - x);
            v.push(piece((m.0, m.1, w as usize, m.3), x, x + w));
            x += mw;
        }
        v
    }

    /// **One cap's layers**: the back, then the picture and the figure
    /// in `rect`.
    fn cap(&self, rect: Span, picture: Crop, line: &str) -> [Layer; 2] {
        let (x0, y0, x1, y1) = rect;
        let h = y1 - y0;
        let w = picture.2 as i32 * h / picture.3 as i32;
        let mut v = vec![Piece {
            crop: picture,
            span: (x0, y0, x0 + w, y1),
            fill: Fill::Stretch,
        }];
        let (tx0, tx1) = (x0 + w, x1 - 1);
        let x = tx0 + ((tx1 - tx0) - self.width(line)) / 2;
        let y = y0 + (h - self.digits[0].3 as i32) / 2;
        v.extend(self.write(line, (x, y)));
        [
            Layer {
                atlas: Atlas::Common,
                grey: Some(BACK_GREY),
                pieces: self.back_pieces(rect),
            },
            Layer {
                atlas: Atlas::Common,
                grey: None,
                pieces: v,
            },
        ]
    }

    /// **`draw_vert_divider`, thin** (`IFaceRenderManager::draw_vert_divider@0051d0d0`), at
    /// `x` from `y` for `h` rows: `THIN_VERT_BAR` tiled down from its
    /// first row (`tile_random`'s start, as run1814b's photograph shows
    /// it), then `THIN_TEE[0]` over its top and `THIN_TEE[2]` over its
    /// foot, each 1:1, centred on the bar, as the photograph registers
    /// them: the top three rows above the bar, the foot two rows above its
    /// end.
    pub fn divider(&self, x: i32, y: i32, h: i32) -> Layer {
        let bar = self.thin_vert_bar;
        let [top, foot] = self.thin_tee;
        let tx = x + (bar.2 as i32 - top.2 as i32) / 2;
        let one = |crop: Crop, (x, y): (i32, i32)| Piece {
            crop,
            span: (x, y, x + crop.2 as i32, y + crop.3 as i32),
            fill: Fill::Tile,
        };
        Layer {
            atlas: Atlas::Civ,
            grey: None,
            pieces: vec![
                Piece {
                    crop: bar,
                    span: (x, y, x + bar.2 as i32, y + h),
                    fill: Fill::Tile,
                },
                one(top, (tx, y - 3)),
                one(foot, (tx, y + h - 2)),
            ],
        }
    }

    /// **The box's layers** on a `size` frame, in screen coordinates, in
    /// drawing order: `draw_pop_cap`'s back, picture and figure and its
    /// divider at the rectangle's right less one, then `draw_city_cap`'s,
    /// which draws no divider.
    pub fn layers(&self, size: (i32, i32), f: Figures) -> Option<Vec<Layer>> {
        let [_, pop, city] = self.rects(size)?;
        let mut v: Vec<Layer> = self
            .cap(pop, self.pop_tex, &format!("{}/{}", f.pop, f.pop_cap))
            .into();
        v.push(self.divider(pop.2 - 1, pop.1, pop.3 - pop.1));
        v.extend(self.cap(
            city,
            self.city_tex,
            &format!("{}/{}", f.cities, f.city_limit),
        ));
        Some(v)
    }

    /// The composed box's span on a `size` frame: `cap_area` grown by
    /// the border's reach, held to the frame.
    pub fn span(&self, size: (i32, i32)) -> Option<Span> {
        let [(x0, y0, x1, y1), ..] = self.rects(size)?;
        Some((
            (x0 - 8).max(0),
            (y0 - 8).max(0),
            (x1 + 8).min(size.0),
            (y1 + 8).min(size.1),
        ))
    }

    /// **The box composed** on a `size` frame from `common`, `TEX_COMMON`,
    /// and `civ`, the nation's `TEX_CIV`: the image and its corner on the
    /// frame. Each layer is painted alone, greyed where it is, and laid
    /// over the layers before it.
    pub fn compose(
        &self,
        size: (i32, i32),
        f: Figures,
        common: &render::tga::Image,
        civ: &render::tga::Image,
    ) -> Result<(Texels, (i32, i32)), String> {
        let (x0, y0, x1, y1) = self.span(size).ok_or("the box does not resolve")?;
        let (w, h) = ((x1 - x0).max(0) as usize, (y1 - y0).max(0) as usize);
        let blank = || Texels {
            w,
            h,
            rgba: vec![0u8; w * h * 4],
        };
        let mut out = blank();
        for layer in self.layers(size, f).ok_or("the box does not resolve")? {
            let pieces: Vec<Piece> = layer
                .pieces
                .iter()
                .map(|p| Piece {
                    span: (p.span.0 - x0, p.span.1 - y0, p.span.2 - x0, p.span.3 - y0),
                    ..*p
                })
                .collect();
            let mut one = blank();
            let atlas = match layer.atlas {
                Atlas::Common => common,
                Atlas::Civ => civ,
            };
            paint(&mut one, &pieces, atlas)?;
            let src = one.rgba.as_chunks::<4>().0;
            for (d, t) in out.rgba.as_chunks_mut::<4>().0.iter_mut().zip(src) {
                let mut t = *t;
                if let Some(g) = layer.grey {
                    for c in &mut t[..3] {
                        *c = highlight(*c, g);
                    }
                }
                over(d, t);
            }
        }
        Ok((out, (x0, y0)))
    }
}
