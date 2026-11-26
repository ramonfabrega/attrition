//! **The bar's left window** (item 4724; `docs/RENDERER.md` §6.11): the
//! stone strip along the top of the bar, the left edge, the bottom edge,
//! the pause button, and the options grid, `<IFACEOPTIONS name="main">`,
//! with its dividers, their feet and the twenty blank slots.
//!
//! The pieces are interface.xml's, cut from the nation's `TEX_CIV` and
//! composed once into one image. `<IFACEMAINBASE>` is the bar's window.
//! Its `left_window` rectangle holds the options grid at the grid's own
//! `left`/`top`. Its `mid_window` holds the minimap's window, so the bar's
//! origin is the minimap window's origin less `mid_window`'s corner.
//!
//! How each piece is drawn was registered on run1814b's photograph at
//! frame 3294, and every opaque pixel of the composition outside the clock
//! box and the five occupied slots equals the photograph's to within two
//! levels:
//! - `bar` is tiled across the left window's width from x 0, at the bar's
//!   top (`IFaceMainBase::on_redraw@00811600`'s `tile_horiz`).
//! - The `IFACERECT`/`IFACETEX` pairs of one name are taken in order, the
//!   n-th rectangle with the n-th texture: the base's `left_tile` and
//!   `left_tile_only` and the grid's `misc_tile` tiled from the rectangle's
//!   corner, and the grid's `misc` drawn at the rectangle's size.
//! - `blank_button` is stretched into each button's 61 × 39 by bilinear
//!   samples at texel centres that read the atlas past the crop's edge,
//!   unclamped. Clamped to the crop, the slots' top and bottom rows part.
//! - The parent's pieces go down first: the grid's feet lie over the base's
//!   bottom edge.

use crate::selection::{Texels, attr};

/// A crop of the atlas: left, top, width, height.
pub type Crop = (usize, usize, usize, usize);

/// A rectangle as `(x0, y0, x1, y1)`, ends exclusive.
pub type Span = (i32, i32, i32, i32);

/// How a piece covers its rectangle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fill {
    /// The crop repeated from the rectangle's corner.
    Tile,
    /// The crop stretched to the rectangle, bilinear, unclamped.
    Stretch,
}

/// One piece of the window: a crop of `TEX_CIV`, where, and how.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Piece {
    pub crop: Crop,
    pub span: Span,
    pub fill: Fill,
}

/// **The left window laid out**, in the bar's own coordinates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeftLayout {
    /// The composed image's width and height: the left window's width and
    /// the bar's height.
    pub size: (i32, i32),
    /// `mid_window`'s corner in the bar: where the minimap's window sits.
    pub mid_window: (i32, i32),
    /// Item 4727: `timer_area2`, the clock's box, where
    /// `IFaceMainBase::draw_button_row_area@0080f7b0` writes the clock.
    pub clock: Span,
    /// In drawing order.
    pub pieces: Vec<Piece>,
}

fn num(tag: &str, name: &str) -> Option<i32> {
    attr(tag, name)?.parse().ok()
}

/// One axis of a rectangle: its low end, its high end as an absolute
/// coordinate or counted back from the parent's `extent`, and its size,
/// which interface.xml writes negative to extend back from the high end.
fn axis(
    lo: Option<i32>,
    hi: Option<i32>,
    inv: Option<i32>,
    size: Option<i32>,
    extent: i32,
) -> Option<(i32, i32)> {
    let hi = hi.or(inv.map(|i| extent - i));
    let (a, b) = match (lo, hi, size) {
        (Some(l), Some(h), _) => (l, h),
        (Some(l), None, Some(s)) => (l, l + s),
        (None, Some(h), Some(s)) => (h - s.abs(), h),
        _ => return None,
    };
    Some((a.min(b), a.max(b)))
}

/// **An `IFACERECT` or a `BUTTON` resolved** in a parent `w × h`.
pub fn resolve(tag: &str, (w, h): (i32, i32)) -> Option<Span> {
    let (x0, x1) = axis(
        num(tag, "left"),
        num(tag, "right"),
        num(tag, "right_inverse"),
        num(tag, "width"),
        w,
    )?;
    let (y0, y1) = axis(
        num(tag, "top"),
        num(tag, "bottom"),
        num(tag, "bottom_inverse"),
        num(tag, "height"),
        h,
    )?;
    Some((x0, y0, x1, y1))
}

fn crop_of(tag: &str) -> Option<Crop> {
    (attr(tag, "texture") == Some("TEX_CIV")).then_some(())?;
    let n = |k| num(tag, k).and_then(|v| usize::try_from(v).ok());
    Some((n("left")?, n("top")?, n("width")?, n("height")?))
}

fn block<'a>(xml: &'a str, open: &str, close: &str) -> Result<&'a str, String> {
    xml.split(open)
        .nth(1)
        .and_then(|s| s.split(close).next())
        .ok_or(format!("interface.xml: no {open}"))
}

/// The tags of a block, the head first.
fn tags(block: &str) -> Vec<&str> {
    block.split('<').collect()
}

/// The `IFACERECT`s and `IFACETEX`s named `name` among `tags`, paired in
/// order, the rectangles resolved in `parent` and moved by `at`.
fn pairs(
    tags: &[&str],
    name: &str,
    parent: (i32, i32),
    at: (i32, i32),
    fill: Fill,
) -> Result<Vec<Piece>, String> {
    fn named<'a>(
        tags: &'a [&'a str],
        kind: &'a str,
        name: &'a str,
    ) -> impl Iterator<Item = &'a str> {
        tags.iter()
            .filter(move |t| t.starts_with(kind) && attr(t, "name") == Some(name))
            .copied()
    }
    let rects: Vec<Span> = named(tags, "IFACERECT ", name)
        .map(|t| {
            resolve(t, parent).ok_or(format!("{name}: a rectangle that does not resolve: {t}"))
        })
        .collect::<Result<_, _>>()?;
    let crops: Vec<Crop> = named(tags, "IFACETEX ", name)
        .map(|t| crop_of(t).ok_or(format!("{name}: a texture off TEX_CIV: {t}")))
        .collect::<Result<_, _>>()?;
    if rects.is_empty() || rects.len() != crops.len() {
        return Err(format!(
            "{name}: {} rectangles, {} textures",
            rects.len(),
            crops.len()
        ));
    }
    Ok(rects
        .into_iter()
        .zip(crops)
        .map(|((x0, y0, x1, y1), crop)| Piece {
            crop,
            span: (x0 + at.0, y0 + at.1, x1 + at.0, y1 + at.1),
            fill,
        })
        .collect())
}

fn named_rect(tags: &[&str], name: &str, parent: (i32, i32)) -> Result<Span, String> {
    tags.iter()
        .find(|t| t.starts_with("IFACERECT ") && attr(t, "name") == Some(name))
        .and_then(|t| resolve(t, parent))
        .ok_or(format!("interface.xml: no rectangle {name}"))
}

fn named_tex(tags: &[&str], name: &str) -> Result<Crop, String> {
    tags.iter()
        .find(|t| t.starts_with("IFACETEX ") && attr(t, "name") == Some(name))
        .and_then(|t| crop_of(t))
        .ok_or(format!("interface.xml: no texture {name} off TEX_CIV"))
}

impl LeftLayout {
    /// The left window of `xml`'s `<IFACEMAINBASE>` and its
    /// `<IFACEOPTIONS name="main">`.
    pub fn parse(xml: &str) -> Result<LeftLayout, String> {
        let base = tags(block(xml, "<IFACEMAINBASE ", "</IFACEMAINBASE>")?);
        // The bar's width is the frame's (`percent_width="1000"`); the left
        // window's pieces are all at its left, so its own width serves.
        let height = num(base[0], "height").ok_or("IFACEMAINBASE: no height")?;
        let (lx0, ly0, lx1, _) = named_rect(&base, "left_window", (0, height))?;
        let width = lx1;
        let parent = (width, height);
        let (mx0, my0, _, _) = named_rect(&base, "mid_window", parent)?;
        let mut pieces = Vec::new();
        let bar = named_tex(&base, "bar")?;
        pieces.push(Piece {
            crop: bar,
            span: (0, 0, width, bar.3 as i32),
            fill: Fill::Tile,
        });
        // Item 4727: the clock's box, `time_back` stretched into
        // `timer_area2` (`draw_button_row_area@0080f7b0`'s `add_smooth`).
        let clock = named_rect(&base, "timer_area2", parent)?;
        pieces.push(Piece {
            crop: named_tex(&base, "time_back")?,
            span: clock,
            fill: Fill::Stretch,
        });
        for name in ["left_tile", "left_tile_only"] {
            pieces.extend(pairs(&base, name, parent, (0, 0), Fill::Tile)?);
        }
        // The pause button: its rectangle, its `tex` at that size.
        let at = base
            .iter()
            .position(|t| t.starts_with("BUTTON ") && attr(t, "name") == Some("pause"))
            .ok_or("IFACEMAINBASE: no pause button")?;
        let span = resolve(base[at], parent).ok_or("pause: a rectangle that does not resolve")?;
        let crop = base[at + 1..]
            .iter()
            .take_while(|t| !t.starts_with("/BUTTON"))
            .find(|t| t.starts_with("IFACETEX ") && attr(t, "name") == Some("tex"))
            .and_then(|t| crop_of(t))
            .ok_or("pause: no tex")?;
        pieces.push(Piece {
            crop,
            span,
            fill: Fill::Stretch,
        });
        // The options grid, at its own corner in the left window.
        let opts = tags(block(
            xml,
            "<IFACEOPTIONS name=\"main\"",
            "</IFACEOPTIONS>",
        )?);
        let head = opts[0];
        let grid = (
            lx0 + num(head, "left").unwrap_or(0),
            ly0 + num(head, "top").unwrap_or(0),
        );
        let gsize = (
            num(head, "width").ok_or("IFACEOPTIONS: no width")?,
            num(head, "height").ok_or("IFACEOPTIONS: no height")?,
        );
        pieces.extend(pairs(&opts, "misc_tile", gsize, grid, Fill::Tile)?);
        pieces.extend(pairs(&opts, "misc", gsize, grid, Fill::Stretch)?);
        let blank = named_tex(&opts, "blank_button")?;
        for (x0, y0, x1, y1) in buttons(&opts)? {
            pieces.push(Piece {
                crop: blank,
                span: (x0 + grid.0, y0 + grid.1, x1 + grid.0, y1 + grid.1),
                fill: Fill::Stretch,
            });
        }
        Ok(LeftLayout {
            size: (width, height),
            mid_window: (mx0, my0),
            clock,
            pieces,
        })
    }
}

/// The grid's buttons: the outer `POSITIONING`'s `x`, `y` and `inc_y`, each
/// inner one a row at `inc_x`, each `BUTTON` its `width` × `height`.
fn buttons(tags: &[&str]) -> Result<Vec<Span>, String> {
    let outer = tags
        .iter()
        .position(|t| t.starts_with("POSITIONING ") && attr(t, "inc_y").is_some())
        .ok_or("IFACEOPTIONS: no POSITIONING")?;
    let o = tags[outer];
    let (x, mut y) = (num(o, "x").unwrap_or(0), num(o, "y").unwrap_or(0));
    let inc_y = num(o, "inc_y").ok_or("POSITIONING: no inc_y")?;
    let mut out = Vec::new();
    let mut row: Option<(i32, i32)> = None;
    for t in &tags[outer + 1..] {
        if t.starts_with("POSITIONING ") {
            row = Some((x, num(t, "inc_x").ok_or("POSITIONING: no inc_x")?));
        } else if t.starts_with("BUTTON ") {
            let (bx, inc_x) = row.as_mut().ok_or("a BUTTON outside a row")?;
            let w = num(t, "width").ok_or("BUTTON: no width")?;
            let h = num(t, "height").ok_or("BUTTON: no height")?;
            out.push((*bx, y, *bx + w, y + h));
            *bx += *inc_x;
        } else if t.starts_with("/POSITIONING") {
            if row.take().is_some() {
                y += inc_y;
            } else {
                break;
            }
        }
    }
    Ok(out)
}

/// The atlas's texel at `(u, v)`, held to the atlas.
fn texel(atlas: &render::tga::Image, u: i64, v: i64) -> [i64; 4] {
    let u = u.clamp(0, atlas.width as i64 - 1) as usize;
    let v = v.clamp(0, atlas.height as i64 - 1) as usize;
    let i = (v * atlas.width + u) * 4;
    [0, 1, 2, 3].map(|c| i64::from(atlas.rgba[i + c]))
}

/// **A bilinear sample of `crop` stretched to `dw × dh`**, at destination
/// pixel `(x, y)`: the sample point `l + (x + ½)·sw/dw − ½`, read from the
/// atlas past the crop's edge. Exact in integers, rounded half up.
pub fn stretched(
    atlas: &render::tga::Image,
    crop: Crop,
    (dw, dh): (i64, i64),
    (x, y): (i64, i64),
) -> [u8; 4] {
    let (l, t, sw, sh) = (crop.0 as i64, crop.1 as i64, crop.2 as i64, crop.3 as i64);
    // In units of 1 / (2·dw): the sample point less the crop's corner.
    let su = (2 * x + 1) * sw - dw;
    let sv = (2 * y + 1) * sh - dh;
    let (du, dv) = (2 * dw, 2 * dh);
    let (u0, fu) = (su.div_euclid(du), su.rem_euclid(du));
    let (v0, fv) = (sv.div_euclid(dv), sv.rem_euclid(dv));
    let mut acc = [0i64; 4];
    for (dy, wy) in [(0, dv - fv), (1, fv)] {
        for (dx, wx) in [(0, du - fu), (1, fu)] {
            if wx * wy == 0 {
                continue;
            }
            let c = texel(atlas, l + u0 + dx, t + v0 + dy);
            for k in 0..4 {
                acc[k] += c[k] * wx * wy;
            }
        }
    }
    let d = du * dv;
    acc.map(|a| ((a + d / 2) / d) as u8)
}

/// `src` over `dst`, straight alpha.
fn over(dst: &mut [u8], src: [u8; 4]) {
    let a = u32::from(src[3]);
    if a == 0 {
        return;
    }
    let da = u32::from(dst[3]);
    let oa = a + da * (255 - a) / 255;
    for c in 0..3 {
        let s = u32::from(src[c]) * a;
        let d = u32::from(dst[c]) * da * (255 - a) / 255;
        dst[c] = ((s + d + oa / 2) / oa) as u8;
    }
    dst[3] = oa as u8;
}

impl LeftLayout {
    /// **The window composed** from `atlas`, the pieces in order.
    pub fn compose(&self, atlas: &render::tga::Image) -> Result<Texels, String> {
        let (w, h) = (self.size.0.max(0) as usize, self.size.1.max(0) as usize);
        let mut rgba = vec![0u8; w * h * 4];
        for p in &self.pieces {
            let (l, t, cw, ch) = p.crop;
            if l + cw > atlas.width || t + ch > atlas.height || cw == 0 || ch == 0 {
                return Err(format!("a crop off the atlas: {:?}", p.crop));
            }
            let (x0, y0, x1, y1) = p.span;
            let (sw, sh) = (i64::from(x1 - x0), i64::from(y1 - y0));
            for y in y0.max(0)..y1.min(self.size.1) {
                for x in x0.max(0)..x1.min(self.size.0) {
                    let (u, v) = (i64::from(x - x0), i64::from(y - y0));
                    let c = match p.fill {
                        Fill::Tile => {
                            let i =
                                ((t + (v as usize) % ch) * atlas.width + l + (u as usize) % cw) * 4;
                            [
                                atlas.rgba[i],
                                atlas.rgba[i + 1],
                                atlas.rgba[i + 2],
                                atlas.rgba[i + 3],
                            ]
                        }
                        Fill::Stretch => stretched(atlas, p.crop, (sw, sh), (u, v)),
                    };
                    let i = (y as usize * w + x as usize) * 4;
                    over(&mut rgba[i..i + 4], c);
                }
            }
        }
        Ok(Texels { w, h, rgba })
    }
}

/// **The clock's line** (item 4727): `draw_button_row_area@0080f7b0`'s
/// `msg`, the game speed's name, `: ` and `say_timer` of the clock's
/// seconds ([`crate::hud::clock`]).
pub fn clock_line(speed: &str, frame: i64) -> String {
    format!("{speed}: {}", crate::hud::clock(frame))
}

/// The clock's face: `fonts`' lookup 27, `clock_size`, `Data\Fonts\arialbd.ttf`
/// at 12, `mAscent` 11 and `mFontHeight` 14 on run1814b's packet.
pub const CLOCK_FACE: (&str, i32, i32, i32) = ("arialbd.ttf", 12, 11, 14);

/// **The clock written** into the window `win` (the composed left window,
/// `w` wide): `Buffer::write_cent_l@0050f850` centres `line` in `rect`,
/// its pen at `x0 + (w − text_width) / 2` and `y0 + (h − text_height) / 2`
/// (both truncated), and each glyph's cell is laid at the pen and the pen
/// moved by its advance. The shadow, black, goes down first, one pixel
/// right and one down; then the text, white.
pub fn write_clock(win: &mut Texels, face: &crate::selection::PlateFace, rect: Span, line: &str) {
    let (x0, y0, x1, y1) = rect;
    let Some(tw) = face.text_width(line) else {
        return;
    };
    let px = x0 + (x1 - x0 - tw) / 2;
    let py = y0 + (y1 - y0 - face.height()) / 2;
    let cells: Vec<(i32, crate::selection::GlyphCell)> = line
        .chars()
        .scan(px, |x, c| {
            let at = *x;
            *x += face.advance(c).unwrap_or(0);
            Some(face.cell(c).map(|cell| (at, cell)))
        })
        .flatten()
        .collect();
    let h = face.height();
    for (ink, (dx, dy)) in [([0u8, 0, 0], (1, 1)), ([255u8, 255, 255], (0, 0))] {
        for (at, cell) in &cells {
            for v in 0..h {
                for u in 0..cell.w as i32 {
                    let a = cell.cover[(v * cell.w as i32 + u) as usize];
                    let (x, y) = (at + u + dx, py + v + dy);
                    if a == 0 || !(0..win.w as i32).contains(&x) || !(0..win.h as i32).contains(&y)
                    {
                        continue;
                    }
                    let i = (y as usize * win.w + x as usize) * 4;
                    over(&mut win.rgba[i..i + 4], [ink[0], ink[1], ink[2], a]);
                }
            }
        }
    }
}
