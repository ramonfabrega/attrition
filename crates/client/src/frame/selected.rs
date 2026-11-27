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
//! - **The figures** (item 4750): `draw_stat@008214d0` for each stat of
//!   `on_redraw`'s order ([`STAT_ORDER`]) into the first free
//!   `stat_short` slot, row-major over two columns ([`fill`]). Column 0
//!   lays `stat_left` at the slot's right less its width and the icon at
//!   the slot's left; column 1 `stat_right` at the slot's left and the
//!   icon at its right less its width. The line is `write_cent_l` over the
//!   back's span, white with a black shadow ([`super::left::write_clock`]).
//!
//! Not drawn here (4750a): the name's text, the `object` button's picture
//! and its hit points.

use super::caps::{Atlas, Layer, Tint};
use super::left::{Crop, Fill, Piece, Span, block, named_tex, pairs, resolve, tags};
use crate::selection::attr;

/// **What the panel shows of one object**: its owner's colour, whether
/// it is a capital, and the figures `draw_stat` writes (item 4750).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Panel {
    pub team: [u8; 3],
    pub capital: bool,
    pub stats: Vec<Stat>,
    /// Item 4752: `draw_build_name`'s line, and `draw_hitpoints`' `hits/full`
    /// where the object has any.
    pub title: String,
    pub hits: Option<String>,
}

/// **One of `draw_stat`'s figures**: the stat's case (0 `armor` to 19
/// `radius`, its icon `stat_<name>` by [`STAT_ICONS`]) and its line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stat {
    pub case: usize,
    pub line: String,
}

/// `on_redraw`'s stat order (its locals at `local_58`, `.rdata`
/// `00b69820` on): the cases `draw_stat` is asked for, in turn.
pub const STAT_ORDER: [usize; 20] = [
    11, 1, 2, 0, 3, 15, 12, 13, 7, 8, 9, 10, 6, 4, 5, 14, 16, 17, 18, 19,
];

/// **Each case's icon**: `draw_stat`'s texture names, `int_str_array
/// +0x16288` on, 20 bytes apart, in `internal_strings.xml`'s order.
pub const STAT_ICONS: [&str; 20] = [
    "stat_armor",
    "stat_attack",
    "stat_antiair",
    "stat_range",
    "stat_cloak",
    "stat_blowncloak",
    "stat_entrench",
    "stat_garrison",
    "stat_power",
    "stat_craft",
    "stat_supplies",
    "stat_decoy",
    "stat_airbase",
    "stat_splash",
    "stat_infiltrated",
    "stat_buildings",
    "stat_in_supply",
    "stat_out_of_supply",
    "stat_attrition",
    "stat_radius",
];

/// **`on_redraw`'s slot fill**: each stat drawn, in turn, takes the first
/// free `stat_short` slot, row by row and column 0 before column 1, as
/// `(row, column)`. Case 9 (`craft`) takes a `stat_long` row and is not
/// placed here (4750b).
pub fn fill(stats: &[Stat]) -> Vec<(usize, usize, &Stat)> {
    let mut taken = [[false; 2]; 4];
    let mut v = Vec::new();
    for s in stats.iter().filter(|s| s.case != 9) {
        let Some((r, c)) = (0..4)
            .flat_map(|r| (0..2).map(move |c| (r, c)))
            .find(|&(r, c)| !taken[r][c])
        else {
            break;
        };
        taken[r][c] = true;
        v.push((r, c, s));
    }
    v
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
    /// Item 4750: `stat_short`'s rectangles, column 0's four rows then
    /// column 1's; `stat_left` and `stat_right` (`TEX_CIV`); each case's
    /// icon off `TEX_COMMON`, `None` where its tag does not parse.
    pub stat_short: Vec<Span>,
    pub stat_left: Crop,
    pub stat_right: Crop,
    pub stat_icons: [Option<Crop>; 20],
    /// Item 4752: `hitpoints`, `draw_hitpoints`' rectangle.
    pub hitpoints: Span,
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
            stat_short: sel
                .iter()
                .filter(|t| t.starts_with("IFACERECT ") && attr(t, "name") == Some("stat_short"))
                .map(|t| resolve(t, parent).ok_or("stat_short: a rectangle that does not resolve"))
                .collect::<Result<_, _>>()?,
            stat_left: named_tex(&sel, "stat_left")?,
            stat_right: named_tex(&sel, "stat_right")?,
            stat_icons: STAT_ICONS.map(|n| common_tex(&sel, n).ok()),
            hitpoints: rect_named(&sel, "hitpoints", parent)?,
        })
    }

    /// **The figures' places** at `at`: each stat [`fill`] places, its
    /// back's piece (`TEX_CIV`), its icon's (`TEX_COMMON`) and the span
    /// its line is centred in, and the stat.
    pub fn stat_pieces<'a>(
        &self,
        at: (i32, i32),
        stats: &'a [Stat],
    ) -> Vec<(Piece, Option<Piece>, Span, &'a Stat)> {
        let piece = |crop: Crop, x: i32, y: i32| Piece {
            crop,
            span: (
                at.0 + x,
                at.1 + y,
                at.0 + x + crop.2 as i32,
                at.1 + y + crop.3 as i32,
            ),
            fill: Fill::Tile,
        };
        fill(stats)
            .into_iter()
            .filter_map(|(r, c, s)| {
                let (l, t, rt, _) = *self.stat_short.get(c * 4 + r)?;
                let (back, bx) = if c == 0 {
                    (self.stat_left, rt - self.stat_left.2 as i32)
                } else {
                    (self.stat_right, l)
                };
                let icon = self.stat_icons[s.case].map(|i| {
                    let ix = if c == 0 { l } else { rt - i.2 as i32 };
                    piece(i, ix, t)
                });
                let b = piece(back, bx, t);
                Some((b, icon, b.span, s))
            })
            .collect()
    }

    /// **The panel's layers** at `at`, in drawing order, for an object
    /// whose owner's colour is `team` and which is a capital or not.
    pub fn layers(
        &self,
        at: (i32, i32),
        team: [u8; 3],
        capital: bool,
        stats: &[Stat],
    ) -> Vec<Layer> {
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
        // Item 4750: each figure's back, then its icon.
        for (back, icon, _, _) in self.stat_pieces(at, stats) {
            v.push(white(Atlas::Civ, vec![back]));
            v.extend(icon.map(|i| white(Atlas::Common, vec![i])));
        }
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
    <IFACERECT name="hitpoints" left="30" top="60" width="62" height="13"/>
    <IFACERECT name="stat_short" left="1" top="94" width="63" height="17"/>
    <IFACERECT name="stat_short" left="1" top="111" width="63" height="17"/>
    <IFACERECT name="stat_short" left="1" top="128" width="63" height="17"/>
    <IFACERECT name="stat_short" left="1" top="145" width="63" height="17"/>
    <IFACERECT name="stat_short" left="65" top="94" width="63" height="17"/>
    <IFACERECT name="stat_short" left="65" top="111" width="63" height="17"/>
    <IFACERECT name="stat_short" left="65" top="128" width="63" height="17"/>
    <IFACERECT name="stat_short" left="65" top="145" width="63" height="17"/>
    <IFACETEX name="stat_left" texture="TEX_CIV" left="211" top="115" width="47" height="17"/>
    <IFACETEX name="stat_right" texture="TEX_CIV" left="211" top="97" width="47" height="17"/>
    <IFACETEX name="stat_armor" texture="TEX_COMMON" left="30" top="105" width="20" height="20"/>
    <IFACETEX name="stat_attack" texture="TEX_COMMON" left="30" top="0" width="20" height="20"/>
    <IFACETEX name="stat_splash" texture="TEX_COMMON" left="72" top="42FS" width="20" height="20"/>
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
        assert_eq!(l.hitpoints, (30, 60, 92, 73));
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
        let layers = l.layers((655, 917), [255, 0, 0], true, &[]);
        assert_eq!(layers.len(), 6);
        assert_eq!(layers[1].atlas, Atlas::Solid([255, 0, 0]));
        assert_eq!(layers[1].pieces[0].span, (655, 917, 947, 931));
        assert_eq!(l.layers((0, 0), [255, 0, 0], false, &[]).len(), 5);
        // Item 4750: the icons by name; `42FS` does not parse.
        assert_eq!(l.stat_icons[0], Some((30, 105, 20, 20)));
        assert_eq!(l.stat_icons[1], Some((30, 0, 20, 20)));
        assert_eq!(l.stat_icons[13], None);
        let stat = |case: usize, line: &str| Stat {
            case,
            line: line.to_string(),
        };
        // run1814b's capital: attack, armor, range, garrison, power, radius.
        let stats = [
            stat(1, "8"),
            stat(0, "3"),
            stat(3, "10"),
            stat(7, "0/10"),
            stat(8, "0/2"),
            stat(19, "20"),
        ];
        let places: Vec<(usize, usize)> = fill(&stats).iter().map(|&(r, c, _)| (r, c)).collect();
        assert_eq!(places, [(0, 0), (0, 1), (1, 0), (1, 1), (2, 0), (2, 1)]);
        let p = l.stat_pieces((655, 917), &stats);
        // Column 0: the back at the slot's right less 47, the icon at its left.
        assert_eq!(p[0].0.span, (672, 1011, 719, 1028));
        assert_eq!(p[0].0.crop, (211, 115, 47, 17));
        assert_eq!(p[0].1.map(|i| i.span), Some((656, 1011, 676, 1031)));
        // Column 1: the back at the slot's left, the icon at its right less 20.
        assert_eq!(p[1].0.span, (720, 1011, 767, 1028));
        assert_eq!(p[1].0.crop, (211, 97, 47, 17));
        assert_eq!(p[1].1.map(|i| i.span), Some((763, 1011, 783, 1031)));
        assert_eq!(p[1].2, p[1].0.span);
        // A case with no icon parsed draws its back alone.
        assert_eq!(
            l.layers((0, 0), [255, 0, 0], false, &stats).len(),
            5 + 6 + 2
        );
    }
}
