//! **The selected panel's frame** (item 4748; `docs/RENDERER.md` §6.18):
//! what `IFaceSelected::on_redraw@00826420` draws over the bar's right
//! window for one selected object, in `<IFACESELECTED name="main">`'s
//! rectangles, the window at `right_window`'s corner.
//!
//! In order:
//! - **`misc_border`**: `IFaceRenderManager::draw_border@0051d260` over
//!   each of the two `misc_border` rectangles, in white, with
//!   `<IFACEBORDER name="misc_border">`'s corners and edges (25 × 25 each,
//!   `TEX_CIV`). The edges' alpha is the shadow inside the frame.
//! - **The name**: `draw_name@008205a0` fills `name` with the owner's
//!   `TeamColor::get_unit` (the font's vtable `+0x58`), then lays
//!   `player_color` (`TEX_CIV` (386, 84), 126 × 46, the wings round the
//!   picture) at its rectangle under `IFACERENDERTYPE_ALPHA` in the same
//!   colour, and for a city's capital `capital_star` (`TEX_COMMON`
//!   (278, 158), 22 × 21) at its own, in the same colour as the photograph
//!   registers it.
//! - **`misc_tile`** by `tile_random`, each rectangle with the texture of
//!   its index, from the texture's first row; then **`misc`** by
//!   `add_smooth` at each rectangle's size, which is its texture's.
//!
//! Not drawn here (4748a, 4748b): the name's text, the `object` button's
//! picture and its hit points, and `draw_stat`'s six figures.

use super::caps::{Atlas, Layer, Tint};
use super::left::{Crop, Fill, Piece, Span, block, named_tex, pairs, resolve, tags};
use crate::selection::attr;

/// **What the panel shows of one object**: its owner's colour and
/// whether it is a capital.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Panel {
    pub team: [u8; 3],
    pub capital: bool,
}

/// **The panel's layout**, in the window's own coordinates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectedLayout {
    /// `misc_border`'s corners 0..3 and edges 0..3 (top, right, bottom,
    /// left), `TEX_CIV`.
    pub corner: [Crop; 4],
    pub edge: [Crop; 4],
    /// The two `misc_border` rectangles.
    pub borders: Vec<Span>,
    /// `name`, which the owner's colour fills.
    pub name: Span,
    /// `player_color`'s rectangle and crop (`TEX_CIV`).
    pub player_color: (Span, Crop),
    /// `capital_star`'s rectangle and crop (`TEX_COMMON`).
    pub capital_star: (Span, Crop),
    /// `misc_tile` and `misc`, each rectangle with its texture.
    pub tiles: Vec<Piece>,
    pub misc: Vec<Piece>,
}

/// The window's size: `IFaceMain::position_windows@00809250` sizes it
/// from its `main` rectangle, 294 × 158.
fn rect_named(tags: &[&str], name: &str, parent: (i32, i32)) -> Result<Span, String> {
    tags.iter()
        .find(|t| t.starts_with("IFACERECT ") && attr(t, "name") == Some(name))
        .and_then(|t| resolve(t, parent))
        .ok_or(format!("IFACESELECTED: no rectangle {name}"))
}

fn common_tex(tags: &[&str], name: &str) -> Result<Crop, String> {
    tags.iter()
        .find(|t| t.starts_with("IFACETEX ") && attr(t, "name") == Some(name))
        .and_then(|t| super::caps::common_crop(t))
        .ok_or(format!("IFACESELECTED: no texture {name} off TEX_COMMON"))
}

/// **`draw_border@0051d260`** around `r` with `corner` and `side` (top,
/// right, bottom, left): the corners 1:1 at the rectangle's corners, the
/// top and bottom sides tiled between them, then the left and right sides
/// between the corners' heights, each from its first texel (item 4742's).
pub fn border(corner: [Crop; 4], side: [Crop; 4], r: Span) -> Vec<Piece> {
    let (l, t, rt, b) = r;
    let [c0, c1, c2, c3] = corner.map(|c| (c.2 as i32, c.3 as i32, c));
    let [s0, s1, s2, s3] = side;
    let one = |crop: Crop, x: i32, y: i32| Piece {
        crop,
        span: (x, y, x + crop.2 as i32, y + crop.3 as i32),
        fill: Fill::Tile,
    };
    let tile = |crop: Crop, span: Span| Piece {
        crop,
        span,
        fill: Fill::Tile,
    };
    vec![
        one(c0.2, l, t),
        one(c1.2, rt - c1.0, t),
        one(c2.2, rt - c2.0, b - c2.1),
        one(c3.2, l, b - c3.1),
        tile(s0, (l + c0.0, t, rt - c1.0, t + s0.3 as i32)),
        tile(s2, (l + c3.0, b - s2.3 as i32, rt - c2.0, b)),
        tile(s3, (l, t + c0.1, l + s3.2 as i32, b - c3.1)),
        tile(s1, (rt - s1.2 as i32, t + c1.1, rt, b - c2.1)),
    ]
}

impl SelectedLayout {
    /// The panel of `xml`'s `<IFACESELECTED name="main">`.
    pub fn parse(xml: &str) -> Result<SelectedLayout, String> {
        let sel = tags(block(
            xml,
            "<IFACESELECTED name=\"main\"",
            "</IFACESELECTED>",
        )?);
        let main = rect_named(&sel, "main", (0, 0))?;
        let parent = (main.2 - main.0, main.3 - main.1);
        let frame = tags(block(
            xml,
            "<IFACEBORDER name=\"misc_border\">",
            "</IFACEBORDER>",
        )?);
        let tex = |name: &str| named_tex(&frame, name);
        let corner = [
            tex("corner0")?,
            tex("corner1")?,
            tex("corner2")?,
            tex("corner3")?,
        ];
        let edge = [tex("edge0")?, tex("edge1")?, tex("edge2")?, tex("edge3")?];
        let borders: Vec<Span> = sel
            .iter()
            .filter(|t| t.starts_with("IFACERECT ") && attr(t, "name") == Some("misc_border"))
            .map(|t| resolve(t, parent).ok_or("misc_border: a rectangle that does not resolve"))
            .collect::<Result<_, _>>()?;
        Ok(SelectedLayout {
            corner,
            edge,
            borders,
            name: rect_named(&sel, "name", parent)?,
            player_color: (
                rect_named(&sel, "player_color", parent)?,
                named_tex(&sel, "player_color")?,
            ),
            capital_star: (
                rect_named(&sel, "capital_star", parent)?,
                common_tex(&sel, "capital_star")?,
            ),
            tiles: pairs(&sel, "misc_tile", parent, (0, 0), Fill::Tile)?,
            misc: pairs(&sel, "misc", parent, (0, 0), Fill::Tile)?,
        })
    }

    /// **The panel's layers** at `at`, in drawing order, for an object
    /// whose owner's colour is `team` and which is a capital or not.
    pub fn layers(&self, at: (i32, i32), team: [u8; 3], capital: bool) -> Vec<Layer> {
        let mv = |p: Piece| Piece {
            span: (
                p.span.0 + at.0,
                p.span.1 + at.1,
                p.span.2 + at.0,
                p.span.3 + at.1,
            ),
            ..p
        };
        let one = |(span, crop): (Span, Crop)| {
            mv(Piece {
                crop,
                span,
                fill: Fill::Tile,
            })
        };
        let white = |atlas, pieces| Layer {
            atlas,
            tint: Tint::White,
            pieces,
        };
        let frame: Vec<Piece> = self
            .borders
            .iter()
            .flat_map(|&r| border(self.corner, self.edge, r))
            .map(mv)
            .collect();
        let mut v = vec![white(Atlas::Civ, frame)];
        v.push(Layer {
            atlas: Atlas::Solid(team),
            tint: Tint::White,
            pieces: vec![mv(Piece {
                crop: (0, 0, 1, 1),
                span: self.name,
                fill: Fill::Tile,
            })],
        });
        v.push(Layer {
            atlas: Atlas::Civ,
            tint: Tint::Modulate(team),
            pieces: vec![one(self.player_color)],
        });
        if capital {
            v.push(Layer {
                atlas: Atlas::Common,
                tint: Tint::Modulate(team),
                pieces: vec![one(self.capital_star)],
            });
        }
        v.push(white(
            Atlas::Civ,
            self.tiles.iter().copied().map(mv).collect(),
        ));
        v.push(white(
            Atlas::Civ,
            self.misc.iter().copied().map(mv).collect(),
        ));
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The layout as interface.xml writes it, on a cut of its own text.
    #[test]
    fn the_panel_lays_out() {
        let xml = r#"<IFACESELECTED name="main" left="0" top="0" width="294" height="161">
    <IFACERECT name="main" left="0" top="0" width="294" height="158"/>
    <IFACERECT name="name" left="0" top="0" width="292" height="14"/>
    <IFACETEX name="player_color" texture="TEX_CIV" left="386" top="84" width="126" height="46"/>
    <IFACETEX name="capital_star" texture="TEX_COMMON" left="278" top="158" width="22" height="21"/>
    <IFACERECT name="player_color" left="0" top="18" width="126" height="46"/>
    <IFACERECT name="capital_star" left="54" top="71" width="22" height="21"/>
    <IFACERECT name="misc_border" left="0" top="18" width="129" height="142"/>
    <IFACERECT name="misc_border" left="133" top="18" width="159" height="142"/>
    <IFACERECT name="misc" left="0" top="12" width="6" height="8"/>
    <IFACERECT name="misc" left="127" bottom_inverse="-1" width="8" height="-6"/>
    <IFACETEX name="misc" texture="TEX_CIV" left="10" top="17" width="6" height="8"/>
    <IFACETEX name="misc" texture="TEX_CIV" left="17" top="34" width="8" height="6"/>
    <IFACERECT name="misc_tile" left="6" top="14" width="284" height="4"/>
    <IFACETEX name="misc_tile" texture="TEX_CIV" left="51" top="0" width="103" height="4"/>
  </IFACESELECTED>
  <IFACEBORDER name="misc_border">
      <IFACETEX name="corner0" texture="TEX_CIV" left="0" top="293" width="25" height="25"/>
      <IFACETEX name="corner1" texture="TEX_CIV" left="52" top="293" width="25" height="25"/>
      <IFACETEX name="corner2" texture="TEX_CIV" left="52" top="345" width="25" height="25"/>
      <IFACETEX name="corner3" texture="TEX_CIV" left="0" top="345" width="25" height="25"/>
      <IFACETEX name="edge0" texture="TEX_CIV" left="26" top="293" width="25" height="25"/>
      <IFACETEX name="edge1" texture="TEX_CIV" left="52" top="319" width="25" height="25"/>
      <IFACETEX name="edge2" texture="TEX_CIV" left="26" top="345" width="25" height="25"/>
      <IFACETEX name="edge3" texture="TEX_CIV" left="0" top="319" width="25" height="25"/>
  </IFACEBORDER>"#;
        let l = SelectedLayout::parse(xml).expect("the panel parses");
        assert_eq!(l.borders, vec![(0, 18, 129, 160), (133, 18, 292, 160)]);
        assert_eq!(l.name, (0, 0, 292, 14));
        assert_eq!(l.player_color, ((0, 18, 126, 64), (386, 84, 126, 46)));
        assert_eq!(l.capital_star, ((54, 71, 76, 92), (278, 158, 22, 21)));
        // `bottom_inverse="-1"` against the window's 158 rows, six back.
        assert_eq!(l.misc[1].span, (127, 153, 135, 159));
        assert_eq!(l.tiles[0].span, (6, 14, 290, 18));
        // The corners at the rectangle's corners, the edges between.
        let b = border(l.corner, l.edge, l.borders[0]);
        assert_eq!(b[2].span, (104, 135, 129, 160));
        assert_eq!(b[4].span, (25, 18, 104, 43));
        assert_eq!(b[7].span, (104, 43, 129, 135));
        let layers = l.layers((655, 917), [255, 0, 0], true);
        assert_eq!(layers.len(), 6);
        assert_eq!(layers[1].atlas, Atlas::Solid([255, 0, 0]));
        assert_eq!(layers[1].pieces[0].span, (655, 917, 947, 931));
        assert_eq!(l.layers((0, 0), [255, 0, 0], false).len(), 5);
    }
}
