//! **The banner at the top of the screen** (item 4737; `docs/RENDERER.md`
//! §6.14): `IFaceMainBase::draw_caption@0080f360`'s caption ends.
//!
//! `draw_caption` finds `<IFACEMAINBASE>`'s three `TEX_CIV` crops,
//! `caption_ends_left`, `caption_ends_mid` and `caption_ends_right`. The
//! banner is as wide as the left, ten middles and the right, and its left
//! edge is the window's width less that, halved. It hands the rectangle,
//! from the screen's top, to the render manager's `draw_expander`, in white.
//! On run1814b's photograph each crop registers 1:1: the left at (750, 0),
//! the first middle at (855, 0) and the right at (1065, 0), every opaque
//! texel within two levels.
//!
//! Item 4745: the caption's text, the tribe's name, `": "` and the age's,
//! `write_cent_l` in the banner's span from y −2 to 23, in
//! [`CAPTION_FACE`] and [`CAPTION_INK`] with no shadow.

use super::left::{Fill, Piece, block, compose, named_tex, tags};
use crate::selection::Texels;

/// `draw_caption`'s count of middles: the width is the left's, ten
/// middles' and the right's.
pub const MID_REPEAT: i32 = 10;

/// **The caption's face** (item 4745), `(file, ppem, mAscent,
/// mFontHeight)` as [`super::left::CLOCK_FACE`]: `draw_caption` asks
/// `FontFamily::get@00522810(fonts, 14, -1)`, the integer overload, and a
/// style of −1 takes `def_style`, 1, bold (item 4752). The 1280 list's
/// first size-14 bold entry is `georgiab.ttf` `B`: the plate's face
/// ([`crate::selection::PLATE_FACE_FILE`]), ascent 13 and height 16.
pub const CAPTION_FACE: (&str, i32, i32, i32) = ("georgiab.ttf", 14, 13, 16);

/// The caption's colour, `text_color[0]` `0xffd4d4d4`; `text_shadow[0]` is
/// `COLOR_NONE`, so no shadow is drawn.
pub const CAPTION_INK: [u8; 3] = [0xd4, 0xd4, 0xd4];

/// The caption's rectangle's top and bottom: `draw_caption` sets the
/// expander's rectangle's y to −2 and 23 before `write_cent_l`; x is the
/// banner's own span.
pub const CAPTION_Y: (i32, i32) = (-2, 23);

/// **The caption's line** (item 4745): `draw_caption`'s `msg`, the tribe's
/// name (`tribes[leader +0xc] +0x18`) and `loc_str_array +0x9d44`
/// (`translated_strings.xml`'s 2013th, `": "`), then the age's name: the
/// age type's (`types[leader +0x6eb8 → +0xdc ^ 0x62766] +0x74`), or, with
/// no age type, the first `startingtechs` category's, `Ancient Age`. A
/// leader with no tribe has no prefix.
pub fn caption_line(tribe: Option<&str>, age: &str) -> String {
    match tribe {
        Some(t) => format!("{t}: {age}"),
        None => age.to_string(),
    }
}

/// **The banner laid out**, in its own coordinates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BannerLayout {
    /// Its width and height: the three crops' widths, the middle's ten
    /// times, and the left's height.
    pub size: (i32, i32),
    /// The left, the middles tiled from the left's edge, and the right.
    pub pieces: Vec<Piece>,
}

impl BannerLayout {
    /// The banner of `xml`'s `<IFACEMAINBASE>`.
    pub fn parse(xml: &str) -> Result<BannerLayout, String> {
        let base = tags(block(xml, "<IFACEMAINBASE ", "</IFACEMAINBASE>")?);
        let left = named_tex(&base, "caption_ends_left")?;
        let mid = named_tex(&base, "caption_ends_mid")?;
        let right = named_tex(&base, "caption_ends_right")?;
        let (lw, mw, rw) = (left.2 as i32, mid.2 as i32, right.2 as i32);
        let x1 = lw + mw * MID_REPEAT;
        let size = (x1 + rw, left.3 as i32);
        let piece = |crop: (usize, usize, usize, usize), x0, x1| Piece {
            crop,
            span: (x0, 0, x1, crop.3 as i32),
            fill: Fill::Tile,
        };
        Ok(BannerLayout {
            size,
            pieces: vec![
                piece(left, 0, lw),
                piece(mid, lw, x1),
                piece(right, x1, size.0),
            ],
        })
    }

    /// **The banner composed** from `atlas`.
    pub fn compose(&self, atlas: &render::tga::Image) -> Result<Texels, String> {
        compose(self.size, &self.pieces, atlas)
    }

    /// Its corner on a frame `width` wide: centred, truncated as the
    /// original's integer halving is, at the screen's top.
    pub fn origin(&self, width: i32) -> (i32, i32) {
        ((width - self.size.0) / 2, 0)
    }
}
