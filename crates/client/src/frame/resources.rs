//! **The resources box at the top left of the screen** (item 4742;
//! `docs/RENDERER.md` §6.16): `IFaceResources::on_redraw@0081b3b0`, which
//! draws `draw_market_area@00818f20`, `draw_resource_area@00818cc0` and
//! `draw_infra_area@00818270` in that order, then its three button windows
//! (`on_button_redraw@0081af70`).
//!
//! `<IFACERESOURCES>`'s rectangles are moved by `adjust_rects@00819a90` for
//! the goods the leader has (`LeaderData::type_avail`): the resource area
//! grows to the goods' rows and moves right of the market area, the infra
//! area right of the resource area. With the resource area open and the
//! other two closed, as on run1814b's photograph, three goods put the box
//! at (0..147, 0..86), the market area (0..17, 0..66), the resource area
//! (16..131, 0..86) and the infra area (130..147, 21..86).
//!
//! Each area is framed by the render manager's `draw_border@0051d260`
//! (`corner_tex0..3`, `side_tex0..3`). The resource area's top row is the
//! fold button, a thick divider, and `draw_cap@00819370`'s commerce cap;
//! a thick divider under it; then a row per good (`draw_resource@00818840`)
//! in `IFaceMainBase::good_order` (food, timber, metal, oil, wealth,
//! knowledge), thin dividers between.

use super::caps::{Atlas, BACK_GREY, CapLayout, Layer, Tint, common_crop, paint_layers, txt_crop};
use super::left::{Crop, Fill, Piece, Span, block, tags};
use crate::selection::{Texels, attr};

/// `IFaceMainBase::init@00812360`'s `good_order`: the order the rows are
/// drawn in, by `TypeIndex` (food 0, timber 1, wealth 2, knowledge 3,
/// metal 4, oil 5).
pub const GOOD_ORDER: [usize; 6] = [0, 1, 4, 5, 2, 3];

/// The goods `draw_resource` and `draw_cap` count out of `type_avail`:
/// `TypeIndex` 2 and 3, which `on_redraw` leaves out of the market's.
const WEALTH: usize = 2;
const KNOWLEDGE: usize = 3;

/// `GREY`, the colour `draw_resource` writes a good's rate in.
pub const GREY: [u8; 3] = [0x80, 0x80, 0x80];
/// `GREEN`, the colour `draw_cap` writes the commerce cap in. The
/// photograph's brightest texel of `+70` is (0, 255, 0).
pub const GREEN: [u8; 3] = [0, 255, 0];

/// `IFaceResources::flags` on run1814b's photograph: the resource area
/// open (`& 2`), the market and infra areas closed. Where the flags start
/// is not read.
pub const PHOTOGRAPH_FLAGS: u32 = 2;

/// **The goods the box shows**, by `TypeIndex`: each available good's
/// stockpile and income (`LeaderDataEncrypt::bucket`, `::income` in
/// sixteenths), and the least `resource_cap` of the available goods.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Goods {
    pub shown: [Option<(i32, i32)>; 6],
    /// `draw_cap`'s minimum, from 15,984 (999 × 16), in sixteenths.
    pub cap: i32,
}

impl Goods {
    /// `draw_cap`'s figure: the least cap over sixteen, toward zero.
    pub fn cap_shown(&self) -> i32 {
        self.cap / 16
    }

    /// `draw_cap`'s line: a `+` before a positive figure.
    pub fn cap_line(&self) -> String {
        let c = self.cap_shown();
        if c > 0 {
            format!("+{c}")
        } else {
            c.to_string()
        }
    }
}

/// **The box's layout**: `<IFACERESOURCES>`'s rectangles and crops, and
/// the render manager's bars and tees off `interface.txt`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceLayout {
    /// The digits (`numbers_texs[0]`) and the item back, as the population
    /// box reads them.
    pub caps: CapLayout,
    pub rects: std::collections::BTreeMap<String, Span>,
    pub corner: [Crop; 4],
    pub side: [Crop; 4],
    pub resource_button_tex: Crop,
    pub up_arrow_tex: Crop,
    pub down_arrow_tex: Crop,
    pub cap_tex: Crop,
    pub first_good_tex: Crop,
    /// `top_vert_button_tex0`, `mid_vert_button_tex0`,
    /// `bottom_vert_button_tex0`.
    pub vert_button: [Crop; 3],
    /// The render manager's `THICK_VERT_BAR`, `THICK_HORIZ_BAR`,
    /// `THIN_HORIZ_BAR`.
    pub thick_vert_bar: Crop,
    pub thick_horiz_bar: Crop,
    pub thin_horiz_bar: Crop,
    /// `THICK_TEE[0..3]` and `THIN_TEE[0..3]`.
    pub thick_tee: [Crop; 4],
    pub thin_tee: [Crop; 4],
}

/// The rectangles `adjust_rects` leaves, in the box's coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rects {
    pub main: Span,
    pub market: Span,
    pub market_button: Span,
    pub resource: Span,
    pub resource_button: Span,
    pub infra: Span,
    pub infra_button: Span,
}

fn n(tag: &str, k: &str) -> Option<i32> {
    attr(tag, k)?.trim().parse().ok()
}

fn tex_of(tag: &str, texture: &str) -> Option<Crop> {
    (attr(tag, "texture").map(str::trim) == Some(texture)).then_some(())?;
    let u = |k| n(tag, k).and_then(|v| usize::try_from(v).ok());
    Some((u("left")?, u("top")?, u("width")?, u("height")?))
}

impl ResourceLayout {
    /// The box of `xml`'s `<IFACERESOURCES>` and `txt`, `interface.txt`.
    pub fn parse(xml: &str, txt: &str) -> Result<ResourceLayout, String> {
        let caps = CapLayout::parse(xml, txt)?;
        let res = tags(block(xml, "<IFACERESOURCES ", "</IFACERESOURCES>")?);
        let mut rects = std::collections::BTreeMap::new();
        for t in res.iter().filter(|t| t.starts_with("IFACERECT ")) {
            let (Some(name), Some(l), Some(tp), Some(w), Some(h)) = (
                attr(t, "name"),
                n(t, "left"),
                n(t, "top"),
                n(t, "width"),
                n(t, "height"),
            ) else {
                continue;
            };
            rects.insert(name.to_string(), (l, tp, l + w, tp + h));
        }
        let tex = |name: &str, texture: &str| {
            res.iter()
                .find(|t| t.starts_with("IFACETEX ") && attr(t, "name") == Some(name))
                .and_then(|t| tex_of(t, texture))
                .ok_or(format!("interface.xml: no {name} off {texture}"))
        };
        let civ = |name: &str| tex(name, "TEX_CIV");
        let common = |name: &str| {
            res.iter()
                .find(|t| t.starts_with("IFACETEX ") && attr(t, "name") == Some(name))
                .and_then(|t| common_crop(t))
                .ok_or(format!("interface.xml: no {name} off TEX_COMMON"))
        };
        let four = |stem: &str| -> Result<[Crop; 4], String> {
            Ok([
                civ(&format!("{stem}0"))?,
                civ(&format!("{stem}1"))?,
                civ(&format!("{stem}2"))?,
                civ(&format!("{stem}3"))?,
            ])
        };
        let tees = |stem: &str| -> Result<[Crop; 4], String> {
            Ok([
                txt_crop(txt, &format!("{stem}[0]"), "TEX_CIV")?,
                txt_crop(txt, &format!("{stem}[1]"), "TEX_CIV")?,
                txt_crop(txt, &format!("{stem}[2]"), "TEX_CIV")?,
                txt_crop(txt, &format!("{stem}[3]"), "TEX_CIV")?,
            ])
        };
        let layout = ResourceLayout {
            caps,
            rects,
            corner: four("corner_tex")?,
            side: four("side_tex")?,
            resource_button_tex: civ("resource_button_tex")?,
            up_arrow_tex: civ("up_arrow_tex")?,
            down_arrow_tex: civ("down_arrow_tex")?,
            cap_tex: common("cap_tex")?,
            first_good_tex: common("first_good_tex")?,
            vert_button: [
                civ("top_vert_button_tex0")?,
                civ("mid_vert_button_tex0")?,
                civ("bottom_vert_button_tex0")?,
            ],
            thick_vert_bar: txt_crop(txt, "THICK_VERT_BAR", "TEX_CIV")?,
            thick_horiz_bar: txt_crop(txt, "THICK_HORIZ_BAR", "TEX_CIV")?,
            thin_horiz_bar: txt_crop(txt, "THIN_HORIZ_BAR", "TEX_CIV")?,
            thick_tee: tees("THICK_TEE")?,
            thin_tee: tees("THIN_TEE")?,
        };
        for name in [
            "market_rect0",
            "market_rect1",
            "market_button_rect0",
            "market_button_rect1",
            "market_title_rect",
            "market_sell_rect",
            "resource_rect0",
            "resource_rect1",
            "resource_button_rect0",
            "resource_button_rect1",
            "resource_cap_rect",
            "resource_item_rect",
            "infra_rect0",
            "infra_rect1",
            "infra_button_rect0",
            "infra_button_rect1",
        ] {
            layout.rect(name)?;
        }
        Ok(layout)
    }

    fn rect(&self, name: &str) -> Result<Span, String> {
        self.rects
            .get(name)
            .copied()
            .ok_or(format!("interface.xml: no rectangle {name}"))
    }

    fn r(&self, name: &str) -> Span {
        self.rects.get(name).copied().unwrap_or_default()
    }

    /// **`adjust_rects@00819a90`** for `goods` goods shown, `market` of
    /// them the market's, under `flags`: the areas' rectangles in the
    /// box, which the window then moves to its own corner.
    pub fn adjust(&self, goods: i32, market: i32, flags: u32) -> Rects {
        let open = flags & 2 != 0;
        let market_open = flags & 3 == 3;
        let infra_open = flags & 6 == 6;
        let pick = |stem: &str, one: bool| self.r(&format!("{stem}{}", u8::from(one)));
        let h = |s: Span| s.3 - s.1;
        let (m0, title, sell) = (
            self.r("market_rect0"),
            self.r("market_title_rect"),
            self.r("market_sell_rect"),
        );
        let top = m0.1 + 12 + h(title);
        let res_bottom = (h(sell) + 3) * goods - 3 + top;
        let market_bottom = (h(sell) + 3) * market - 3 + top;
        let mut market_rect = pick("market_rect", market_open);
        market_rect.3 = market_bottom;
        let mut market_button = pick("market_button_rect", market_open);
        market_button.3 = market_bottom - self.r("market_button_rect0").1;
        let mut resource = pick("resource_rect", open);
        if open {
            resource.3 = res_bottom;
        }
        let dx = if open {
            market_rect.2 - resource.0 - 1
        } else {
            -resource.0
        };
        resource.0 += dx;
        resource.2 += dx;
        let mut infra = pick("infra_rect", infra_open);
        infra.3 = res_bottom;
        let dx = resource.2 - infra.0 - 1;
        infra.0 += dx;
        infra.2 += dx;
        let mut infra_button = pick("infra_button_rect", infra_open);
        let ib0 = self.r("infra_button_rect0");
        let i0 = {
            let mut i = self.r("infra_rect0");
            i.3 = res_bottom;
            i
        };
        infra_button.3 = if infra_open {
            -2 * ib0.1 - i0.1 + infra_button.1 + i0.3
        } else {
            (i0.3 - i0.1) - ib0.1
        };
        let infra_button = (
            infra_button.0 + infra.0,
            infra_button.1 + infra.1,
            infra_button.2 + infra.0,
            infra_button.3 + infra.1,
        );
        let rb = pick("resource_button_rect", open);
        let resource_button = (
            rb.0 + resource.0,
            rb.1 + resource.1,
            rb.2 + resource.0,
            rb.3 + resource.1,
        );
        let union = |a: Span, b: Span| (a.0.min(b.0), a.1.min(b.1), a.2.max(b.2), a.3.max(b.3));
        let main = union(union(market_rect, resource), infra);
        Rects {
            main,
            market: market_rect,
            market_button,
            resource,
            resource_button,
            infra,
            infra_button,
        }
    }

    /// **`draw_border@0051d260`** around `r`: the four corners at the
    /// rectangle's corners, then the top and bottom sides tiled between
    /// them (`tile_horiz`), then the left and right sides (`tile_vert`),
    /// each from its first texel.
    pub fn border(&self, r: Span) -> Vec<Piece> {
        let (l, t, rt, b) = r;
        let [c0, c1, c2, c3] = self.corner.map(|c| (c.2 as i32, c.3 as i32, c));
        let [s0, s1, s2, s3] = self.side;
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

    /// **`draw_vert_divider@0051d0d0`, thick**, at `x` from `y` for `h`
    /// rows: `THICK_VERT_BAR` tiled down from its first row, then
    /// `THICK_TEE[0]` and `[2]` centred on it, three rows above its top
    /// and two above its end, as item 4739 registered the thin one's.
    pub fn vert_divider(&self, x: i32, y: i32, h: i32) -> Vec<Piece> {
        let bar = self.thick_vert_bar;
        let (top, foot) = (self.thick_tee[0], self.thick_tee[2]);
        let tx = x + (bar.2 as i32 - top.2 as i32) / 2;
        vec![
            Piece {
                crop: bar,
                span: (x, y, x + bar.2 as i32, y + h),
                fill: Fill::Tile,
            },
            one(top, tx, y - 3),
            one(foot, tx, y + h - 2),
        ]
    }

    /// **`draw_horiz_divider@0051cf80`** from `x` at `y` for `w` columns,
    /// thick or thin: the bar tiled across from its first column, then
    /// the tees `[3]` and `[1]` at its ends, three columns before its left
    /// and two before its right, two rows above, as the photograph
    /// registers them.
    pub fn horiz_divider(&self, x: i32, y: i32, w: i32, thick: bool) -> Vec<Piece> {
        let (bar, tee) = if thick {
            (self.thick_horiz_bar, self.thick_tee)
        } else {
            (self.thin_horiz_bar, self.thin_tee)
        };
        vec![
            Piece {
                crop: bar,
                span: (x, y, x + w, y + bar.3 as i32),
                fill: Fill::Tile,
            },
            one(tee[3], x - 3, y - 2),
            one(tee[1], x + w - 2, y - 2),
        ]
    }

    /// **A closed area's button** (`on_button_redraw`'s vertical arm) in
    /// `r`: the top piece at its top, the bottom piece at its foot, the
    /// middle tiled between from its first row.
    pub fn vert_button(&self, r: Span) -> Vec<Piece> {
        let [top, mid, bottom] = self.vert_button;
        let (l, t, _, b) = r;
        vec![
            one(top, l, t),
            Piece {
                crop: mid,
                span: (l, t + top.3 as i32, l + mid.2 as i32, b - bottom.3 as i32),
                fill: Fill::Tile,
            },
            one(bottom, l, b - bottom.3 as i32),
        ]
    }

    /// **The box's layers** for `goods` under `flags`, in the box's
    /// coordinates, in drawing order, and the box's rectangle.
    pub fn layers(&self, goods: &Goods, flags: u32) -> (Span, Vec<Layer>) {
        let shown: Vec<usize> = GOOD_ORDER
            .into_iter()
            .filter(|&g| goods.shown[g].is_some())
            .collect();
        let n = shown.len() as i32;
        let m = shown
            .iter()
            .filter(|&&g| g != WEALTH && g != KNOWLEDGE)
            .count() as i32;
        let r = self.adjust(n, m, flags);
        let civ = |pieces: Vec<Piece>| Layer {
            atlas: Atlas::Civ,
            tint: Tint::White,
            pieces,
        };
        let common = |tint: Tint, pieces: Vec<Piece>| Layer {
            atlas: Atlas::Common,
            tint,
            pieces,
        };
        let back = |rect: Span| common(Tint::Highlight(BACK_GREY), self.caps.back_pieces(rect));
        let mut v = Vec::new();
        if flags & 2 != 0 {
            // draw_market_area: the border alone while closed.
            v.push(civ(self.border(r.market)));
        }
        // draw_resource_area.
        let (rl, rt) = (r.resource.0, r.resource.1);
        v.push(civ(self.border(r.resource)));
        let rb = r.resource_button;
        v.push(civ(self.vert_divider(rb.2, rb.1, rb.3 - rb.1)));
        // draw_cap.
        let cr = self.r("resource_cap_rect");
        let cap = (cr.0 + rl, cr.1 + rt, cr.2 + rl, cr.3 + rt);
        v.push(back(cap));
        let ch = cap.3 - cap.1 - 2;
        let ct = self.cap_tex;
        let cw = ch * ct.2 as i32 / ct.3 as i32;
        v.push(common(
            Tint::White,
            vec![Piece {
                crop: ct,
                span: (cap.0 + 1, cap.1 + 1, cap.0 + 1 + cw, cap.1 + 1 + ch),
                fill: Fill::Stretch,
            }],
        ));
        let dh = self.caps.digits[0].3 as i32;
        v.push(common(
            Tint::Modulate(GREEN),
            self.caps.write(
                &goods.cap_line(),
                (cap.0 + 5 + ct.2 as i32, cap.1 + (ch - dh) / 2),
            ),
        ));
        if flags & 2 != 0 {
            let ir = self.r("resource_item_rect");
            let ub = r.resource_button;
            let left = ub.0.min(cap.0);
            let right = ub.2.max(cap.2);
            let bottom = ub.3.max(cap.3);
            v.push(civ(self.horiz_divider(left, bottom, right - left, true)));
            let mut item = (ir.0 + rl, ir.1 + rt, ir.2 + rl, ir.3 + rt);
            for (k, &g) in shown.iter().enumerate() {
                let Some((stock, income)) = goods.shown[g] else {
                    continue;
                };
                v.push(back(item));
                let f = self.first_good_tex;
                let (fw, fh) = (f.2 as i32, f.3 as i32);
                let crop = (f.0 + (f.2 + 4) * g, f.1, f.2, f.3);
                let (px, py) = (item.0 + 1, item.1 + (item.3 - item.1 - fh) / 2);
                v.push(common(
                    Tint::White,
                    vec![Piece {
                        crop,
                        span: (px, py, px + fw, py + fh),
                        fill: Fill::Tile,
                    }],
                ));
                let y = item.1 + (fh - dh) / 2;
                let x = item.0 + 1 + fw + 7;
                let line = stock.to_string();
                v.push(common(Tint::White, self.caps.write(&line, (x, y))));
                let x = x + self.caps.width(&line);
                let rate = format!("/{}", crate::hud::rate_text(crate::hud::shown_rate(income)));
                v.push(common(Tint::Modulate(GREY), self.caps.write(&rate, (x, y))));
                if (k as i32) + 1 < n {
                    v.push(civ(self.horiz_divider(
                        item.0,
                        item.3,
                        item.2 - item.0,
                        false,
                    )));
                }
                let step = item.3 - item.1 + 3;
                item.1 += step;
                item.3 += step;
            }
            // draw_infra_area: the border alone while closed.
            v.push(civ(self.border(r.infra)));
        }
        // The button windows, over their parent: the market's, the
        // fold's, the infra's.
        v.push(civ(self.vert_button(r.market_button)));
        let rbt = self.resource_button_tex;
        let arrow = if flags & 2 != 0 {
            self.up_arrow_tex
        } else {
            self.down_arrow_tex
        };
        let mut fold = vec![one(rbt, rb.0, rb.1)];
        let aw = arrow.2 as i32;
        let ax = rb.0 + (rb.2 - rb.0 - 3 * aw) / 2;
        let ay = rb.1 + (rb.3 - rb.1 - arrow.3 as i32) / 2;
        fold.extend((0..3).map(|i| one(arrow, ax + i * aw, ay)));
        v.push(civ(fold));
        if flags & 2 != 0 {
            v.push(civ(self.vert_button(r.infra_button)));
        }
        (r.main, v)
    }

    /// **The box composed** from `common`, `TEX_COMMON`, and `civ`, the
    /// nation's `TEX_CIV`: the image and its corner on the screen, the
    /// window's own `(0, 0)`.
    pub fn compose(
        &self,
        goods: &Goods,
        flags: u32,
        common: &render::tga::Image,
        civ: &render::tga::Image,
    ) -> Result<(Texels, (i32, i32)), String> {
        let (main, layers) = self.layers(goods, flags);
        let out = paint_layers(main, &layers, common, civ)?;
        Ok((out, (main.0, main.1)))
    }
}

fn one(crop: Crop, x: i32, y: i32) -> Piece {
    Piece {
        crop,
        span: (x, y, x + crop.2 as i32, y + crop.3 as i32),
        fill: Fill::Tile,
    }
}
