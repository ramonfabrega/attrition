//! **The bar's right side** (item 4738; `docs/RENDERER.md` §6.17): what
//! `<IFACEMAINBASE>` draws right of the minimap's window, under the
//! selected panel.
//!
//! - **The stone strip**: `bar` (`TEX_CIV` (390, 131), 122 × 34), tiled
//!   from `mid_window`'s right edge to the frame's, each tile mirrored.
//!   `IFaceMainBase::on_redraw@00811600` copies the texture with its left
//!   and right swapped before `tile_horiz`. On run1814b's photograph every
//!   column of x 648..1920 is the crop's column `(769 − x) mod 122`, 32 of
//!   32 rows at y 883 (the last two lie under the panel).
//! - **The dark panel**: `back` (`TEX_CIV` (0, 371), 160 × 141), which
//!   `on_window_redraw@00811b60` lays by `tile_random` over `right_window`
//!   ((655, 34) to the right edge less 14, 161 rows). Both of its bands
//!   start at the crop's first row on the photograph (`Random::get`'s pick
//!   of the alternate is not modelled; the alternate lies off the atlas).
//! - **The buttons** `stats`, `chat`, `diplo`, `ping` and
//!   `advanced_options`, each its `tex` 1:1 at its rectangle.

use super::left::{Fill, Piece, Span, block, named_tex, paint, resolve, tags};
use crate::selection::{Texels, attr};

/// The right side's buttons, in interface.xml's order.
pub const BUTTONS: [&str; 5] = ["stats", "chat", "diplo", "ping", "advanced_options"];

/// **The right side's layout**, resolved on a frame of any width.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RightLayout {
    /// The bar's height (`<IFACEMAINBASE height>`).
    pub height: i32,
    /// `mid_window`'s right edge: where the strip's tiles start.
    pub x0: i32,
    pub bar: (usize, usize, usize, usize),
    pub back: (usize, usize, usize, usize),
    /// `right_window`'s tag, resolved against the bar's width.
    right_window: String,
    /// Each button's tag and its `tex`.
    buttons: Vec<(String, (usize, usize, usize, usize))>,
}

impl RightLayout {
    pub fn parse(xml: &str) -> Result<RightLayout, String> {
        let base = tags(block(xml, "<IFACEMAINBASE ", "</IFACEMAINBASE>")?);
        let height = super::num(base[0], "height").ok_or("IFACEMAINBASE: no height")?;
        let rect = |name: &str| {
            base.iter()
                .find(|t| t.starts_with("IFACERECT ") && attr(t, "name") == Some(name))
                .map(|t| t.to_string())
                .ok_or(format!("interface.xml: no rectangle {name}"))
        };
        let mid = rect("mid_window")?;
        let (_, _, x0, _) = resolve(&mid, (i32::MAX / 2, height)).ok_or("mid_window")?;
        let mut buttons = Vec::new();
        for name in BUTTONS {
            let i = base
                .iter()
                .position(|t| t.starts_with("BUTTON ") && attr(t, "name") == Some(name))
                .ok_or(format!("IFACEMAINBASE: no button {name}"))?;
            let tex = named_tex(&base[i + 1..i + 2], "tex")?;
            buttons.push((base[i].to_string(), tex));
        }
        Ok(RightLayout {
            height,
            x0,
            bar: named_tex(&base, "bar")?,
            back: named_tex(&base, "back")?,
            right_window: rect("right_window")?,
            buttons,
        })
    }

    /// Item 4748: `right_window` on a frame `width` wide, in the bar's
    /// coordinates: the selected panel's window lies at its corner.
    pub fn window(&self, width: i32) -> Option<Span> {
        resolve(&self.right_window, (width, self.height))
    }

    /// The pieces on a frame `width` wide, in the bar's coordinates.
    pub fn pieces(&self, width: i32) -> Vec<Piece> {
        let parent = (width, self.height);
        let mut v = vec![Piece {
            crop: self.bar,
            span: (self.x0, 0, width, self.bar.3 as i32),
            fill: Fill::Mirror,
        }];
        if let Some(span) = resolve(&self.right_window, parent) {
            v.push(Piece {
                crop: self.back,
                span,
                fill: Fill::Tile,
            });
        }
        for (tag, crop) in &self.buttons {
            if let Some(span) = resolve(tag, parent) {
                v.push(Piece {
                    crop: *crop,
                    span,
                    fill: Fill::Tile,
                });
            }
        }
        v
    }

    /// The right side composed on a frame `width` wide, from `x0` to the
    /// right edge, and its corner in the bar.
    pub fn compose(
        &self,
        atlas: &render::tga::Image,
        width: i32,
    ) -> Result<(Texels, (i32, i32)), String> {
        let w = (width - self.x0).max(0);
        let shifted: Vec<Piece> = self
            .pieces(width)
            .into_iter()
            .map(|p| {
                let (a, b, c, d): Span = p.span;
                Piece {
                    span: (a - self.x0, b, c - self.x0, d),
                    ..p
                }
            })
            .collect();
        let mut out = Texels {
            w: w as usize,
            h: self.height as usize,
            rgba: vec![0u8; (w * self.height) as usize * 4],
        };
        paint(&mut out, &shifted, atlas)?;
        Ok((out, (self.x0, 0)))
    }
}
