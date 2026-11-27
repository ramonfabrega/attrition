//! **The HUD's frame** (item 4694; `docs/RENDERER.md` §6.10): the
//! original's own interface art, where the original lays it.
//!
//! The first piece is the minimap's window, `<IFACEMINIMAP name="main">` in
//! `Data/interface.xml`: its `back`, a crop of `TEX_CIV`, drawn 1:1 at the
//! window's origin (`IFaceMiniMap::on_redraw@008152e0` adds it to the
//! render manager at its own width and height, in white), and its four
//! buttons, each its own `TEX_CIV` crop at its own rectangle.
//!
//! `TEX_CIV` is the nation's: `IFaceMainBase::age_update@00810b10`
//! replaces it with `art\iface_<code>_assets.tga`, the code chosen by the
//! console's age and its nation's `Tribe::build_continent` (`+0x64`)
//! ([`civ_code`]). interface.xml's own `TEX_CIV` is the fallback.
//!
//! The window's origin is the map rectangle's (`IFACERECT name="map"`)
//! offset back from item 4581's [`crate::minimap::QUAD`]: on run1814b's
//! photograph that is (316, 884), and the back's opaque pixels there equal
//! the atlas's to within two levels on 99.9 % of them.

use crate::selection::{Texels, attr, layers_of};

pub mod banner;
pub mod caps;
pub mod left;
pub mod resources;
pub mod right;
pub mod selected;
pub mod slots;
pub mod tally;

/// **The atlas's code** for a console of `ages` ages owned (0 the Ancient)
/// whose nation's `BUILD_CONTINENT` is `build_continent`:
/// `IFaceMainBase::age_update@00810b10`'s switch, each code one of its
/// string table's (`int_str_array +0x15720` to `+0x15874`, 20 bytes
/// apart). `None` where the switch takes its default and appends no code.
pub fn civ_code(build_continent: i32, ages: i32) -> Option<&'static str> {
    Some(match ages {
        ..=2 => match build_continent {
            0 | 1 => "MD0",
            2 | 3 => "AS0",
            4 | 5 | 7 | 8 => "SA0",
            6 => "IRO",
            _ => return None,
        },
        3 | 4 => match build_continent {
            0 => "MD3",
            1 => "NE3",
            2 => "AS3",
            3 => "KO3",
            4 => "SA3",
            5 => "TR3",
            6 => "IR3",
            7 => "USA",
            8 => "EIN",
            _ => return None,
        },
        5 => "IND",
        _ => "MOD",
    })
}

/// A rectangle, left, top, width, height.
pub type Rect = (i32, i32, i32, i32);

/// **The minimap's window**, as interface.xml lays it.
pub struct MinimapLayout {
    /// The window's width and height.
    pub size: (i32, i32),
    /// `IFACERECT name="map"`, in the window.
    pub map: Rect,
    /// `IFACETEX name="back"`'s crop of `TEX_CIV`.
    pub back: (usize, usize, usize, usize),
    /// Each `BUTTON`'s rectangle in the window and its `tex`'s crop.
    pub buttons: Vec<(Rect, (usize, usize, usize, usize))>,
    /// Item 4764: the index in [`Self::buttons`] of `cycle_peasant`, the
    /// idle citizens' button, which [`idle_lit`] shows or hides.
    pub peasant: Option<usize>,
}

fn num(tag: &str, name: &str) -> Option<i32> {
    attr(tag, name)?.parse().ok()
}

fn crop_of(tag: &str) -> Option<(usize, usize, usize, usize)> {
    (attr(tag, "texture") == Some("TEX_CIV")).then_some(())?;
    let n = |k| num(tag, k).and_then(|v| usize::try_from(v).ok());
    Some((n("left")?, n("top")?, n("width")?, n("height")?))
}

impl MinimapLayout {
    /// The first `<IFACEMINIMAP name="main">` of `xml`.
    pub fn parse(xml: &str) -> Result<MinimapLayout, String> {
        let block = xml
            .split("<IFACEMINIMAP name=\"main\"")
            .nth(1)
            .and_then(|s| s.split("</IFACEMINIMAP>").next())
            .ok_or("interface.xml: no <IFACEMINIMAP name=\"main\">")?;
        let tags: Vec<&str> = block.split('<').collect();
        let head = tags[0];
        let size = (
            num(head, "width").ok_or("IFACEMINIMAP: no width")?,
            num(head, "height").ok_or("IFACEMINIMAP: no height")?,
        );
        let rect = |t: &str| -> Option<Rect> {
            Some((
                num(t, "left")?,
                num(t, "top")?,
                num(t, "width")?,
                num(t, "height")?,
            ))
        };
        let map = tags
            .iter()
            .find(|t| t.starts_with("IFACERECT") && attr(t, "name") == Some("map"))
            .and_then(|t| rect(t))
            .ok_or("IFACEMINIMAP: no map rectangle")?;
        let back = tags
            .iter()
            .find(|t| t.starts_with("IFACETEX") && attr(t, "name") == Some("back"))
            .and_then(|t| crop_of(t))
            .ok_or("IFACEMINIMAP: no back off TEX_CIV")?;
        let mut buttons = Vec::new();
        let mut peasant = None;
        for (i, t) in tags.iter().enumerate() {
            if !t.starts_with("BUTTON ") {
                continue;
            }
            if attr(t, "name") == Some("cycle_peasant") {
                peasant = Some(buttons.len());
            }
            let r = rect(t).ok_or(format!("IFACEMINIMAP: a button with no rectangle: {t}"))?;
            let tex = tags[i + 1..]
                .iter()
                .take_while(|t| !t.starts_with("/BUTTON"))
                .find(|t| t.starts_with("IFACETEX") && attr(t, "name") == Some("tex"))
                .and_then(|t| crop_of(t))
                .ok_or(format!("IFACEMINIMAP: a button with no tex: {t}"))?;
            buttons.push((r, tex));
        }
        Ok(MinimapLayout {
            size,
            map,
            back,
            buttons,
            peasant,
        })
    }
}

/// **The minimap's window drawn**: the back and its buttons, cut from one
/// nation's atlas.
pub struct MinimapFrame {
    pub map: Rect,
    pub back: Texels,
    pub buttons: Vec<(Rect, Texels)>,
    /// Item 4764: [`MinimapLayout::peasant`].
    pub peasant: Option<usize>,
    /// Item 4724: the bar's left window, composed ([`left`]).
    pub left: Texels,
    /// `mid_window`'s corner in the bar: the minimap window's place.
    pub mid_window: (i32, i32),
    /// Item 4727: `timer_area2`, the clock's box, in the bar.
    pub clock: left::Span,
    /// The clock's face ([`left::CLOCK_FACE`]), `None` where the install
    /// has none.
    pub face: Option<crate::selection::PlateFace>,
    /// Item 4731: the grid's buttons in the bar, and the slots' art.
    pub slot_spans: Vec<left::Span>,
    pub slot_art: Option<std::sync::Arc<slots::SlotArt>>,
    /// Item 4737: the banner at the screen's top ([`banner`]), composed,
    /// and its layout.
    pub banner: Texels,
    pub banner_layout: banner::BannerLayout,
    /// Item 4739: the population box's art ([`caps`]), `None` where the
    /// install has none.
    pub caps: Option<std::sync::Arc<CapArt>>,
    /// The nation's `TEX_CIV`, which the box's divider is cut from.
    pub civ: std::sync::Arc<render::tga::Image>,
    /// Item 4738: the bar's right side ([`right`]), composed on a frame's
    /// width as it is drawn.
    pub right: right::RightLayout,
    /// Item 4748: the selected panel's frame ([`selected`]), `None` where
    /// interface.xml has none.
    pub selected: Option<selected::SelectedLayout>,
    /// Item 4752: the panel's faces ([`left::PANEL_FACES`]), `arialbd` and
    /// `arial` at 10, `None` where the install has them not.
    pub panel_faces: Option<[crate::selection::PlateFace; 2]>,
    /// Item 4745: the caption's face ([`banner::CAPTION_FACE`]), `None`
    /// where the install has it not.
    pub caption_face: Option<crate::selection::PlateFace>,
}

/// **The population box's art** (item 4739): its layout and `TEX_COMMON`.
pub struct CapArt {
    pub layout: caps::CapLayout,
    pub common: render::tga::Image,
    /// Item 4742: the resources box's layout ([`resources`]).
    pub resources: resources::ResourceLayout,
}

impl CapArt {
    fn load(root: &std::path::Path, xml: &str) -> Result<CapArt, String> {
        let txt = std::fs::read_to_string(root.join("interface.txt"))
            .map_err(|e| format!("interface.txt: {e}"))?;
        let file = xml
            .split('<')
            .filter(|t| t.starts_with("IFACETEXLOAD"))
            .find(|t| attr(t, "id") == Some("TEX_COMMON"))
            .and_then(|t| attr(t, "file"))
            .ok_or("interface.xml: no TEX_COMMON")?;
        let path = rondata::artdata::resolve_art(root, file).ok_or(format!("{file}: absent"))?;
        let bytes = std::fs::read(&path).map_err(|e| format!("{file}: {e}"))?;
        Ok(CapArt {
            layout: caps::CapLayout::parse(xml, &txt)?,
            resources: resources::ResourceLayout::parse(xml, &txt)?,
            common: render::tga::decode(&bytes).map_err(|e| format!("{file}: {e:?}"))?,
        })
    }
}

/// **The frame's art**: interface.xml's layout, and each nation's atlas as
/// it is first asked for.
pub struct FrameArt {
    root: std::path::PathBuf,
    layout: MinimapLayout,
    left: left::LeftLayout,
    banner: banner::BannerLayout,
    right: right::RightLayout,
    selected: Option<selected::SelectedLayout>,
    /// interface.xml's own `TEX_CIV` file, the fallback.
    default_civ: String,
    /// Item 4727: the clock's face, and `rules.xml`'s `gamespeeds`, the
    /// names `TurnControl::speed` indexes (`game_speeds`).
    face: Option<crate::selection::PlateFace>,
    panel_faces: Option<[crate::selection::PlateFace; 2]>,
    caption_face: Option<crate::selection::PlateFace>,
    pub speeds: Vec<String>,
    /// Item 4745: `rules.xml`'s `startingtechs`, whose first is the
    /// caption's age with no age type.
    pub start_ages: Vec<String>,
    /// Item 4731: the slots' pictures, `None` where the install has none.
    pub slot_art: Option<std::sync::Arc<slots::SlotArt>>,
    caps: Option<std::sync::Arc<CapArt>>,
    cache: std::sync::Mutex<
        std::collections::BTreeMap<Option<&'static str>, Option<std::sync::Arc<MinimapFrame>>>,
    >,
}

impl FrameArt {
    pub fn load(root: &std::path::Path) -> Result<FrameArt, String> {
        let xml = std::fs::read_to_string(root.join("Data").join("interface.xml"))
            .map_err(|e| format!("interface.xml: {e}"))?;
        let layout = MinimapLayout::parse(&xml)?;
        let left = left::LeftLayout::parse(&xml)?;
        let banner = banner::BannerLayout::parse(&xml)?;
        let right = right::RightLayout::parse(&xml)?;
        let selected = selected::SelectedLayout::parse(&xml).ok();
        let default_civ = xml
            .split('<')
            .filter(|t| t.starts_with("IFACETEXLOAD"))
            .find(|t| attr(t, "id") == Some("TEX_CIV"))
            .and_then(|t| attr(t, "file"))
            .ok_or("interface.xml: no TEX_CIV")?
            .to_string();
        let (file, ppem, ascent, height) = left::CLOCK_FACE;
        let face = crate::selection::PlateFace::load_face(root, file, ppem, ascent, height).ok();
        let panel_faces = left::PANEL_FACES
            .map(|(file, ppem, ascent, height)| {
                crate::selection::PlateFace::load_face(root, file, ppem, ascent, height).ok()
            })
            .into_iter()
            .collect::<Option<Vec<_>>>()
            .and_then(|v| <[_; 2]>::try_from(v).ok());
        let (file, ppem, ascent, height) = banner::CAPTION_FACE;
        let caption_face =
            crate::selection::PlateFace::load_face(root, file, ppem, ascent, height).ok();
        let rules = std::fs::read_to_string(root.join("Data").join("rules.xml")).ok();
        let categories = |id: &str| -> Vec<String> {
            rules
                .as_deref()
                .and_then(|x| {
                    let b = x
                        .split(&format!("<CATEGORIES id=\"{id}\""))
                        .nth(1)?
                        .split("</CATEGORIES>")
                        .next()?;
                    Some(
                        b.split('<')
                            .filter(|t| t.starts_with("CATEGORY "))
                            .filter_map(|t| attr(t, "name").map(String::from))
                            .collect(),
                    )
                })
                .unwrap_or_default()
        };
        let speeds = categories("gamespeeds");
        let start_ages = categories("startingtechs");
        Ok(FrameArt {
            root: root.to_path_buf(),
            layout,
            left,
            banner,
            right,
            selected,
            default_civ,
            face,
            panel_faces,
            caption_face,
            speeds,
            start_ages,
            slot_art: slots::SlotArt::load(root).ok().map(std::sync::Arc::new),
            caps: CapArt::load(root, &xml).ok().map(std::sync::Arc::new),
            cache: Default::default(),
        })
    }

    /// The window cut from `code`'s atlas (`None`: interface.xml's own).
    pub fn frame(&self, code: Option<&'static str>) -> Option<std::sync::Arc<MinimapFrame>> {
        let mut cache = self.cache.lock().ok()?;
        cache
            .entry(code)
            .or_insert_with(|| self.cut(code).ok().map(std::sync::Arc::new))
            .clone()
    }

    fn cut(&self, code: Option<&'static str>) -> Result<MinimapFrame, String> {
        let file = code.map_or_else(
            || self.default_civ.clone(),
            |c| format!("art\\iface_{c}_assets.tga"),
        );
        let path =
            rondata::artdata::resolve_art(&self.root, &file).ok_or(format!("{file}: absent"))?;
        let bytes = std::fs::read(&path).map_err(|e| format!("{file}: {e}"))?;
        let atlas = render::tga::decode(&bytes).map_err(|e| format!("{file}: {e:?}"))?;
        let l = &self.layout;
        let back = Texels::crop(&atlas, l.back).ok_or("the back is off the atlas")?;
        if (back.w as i32) < l.size.0 || (back.h as i32) < l.size.1 {
            return Err(format!(
                "the back is smaller than the window's {:?}",
                l.size
            ));
        }
        let buttons = l
            .buttons
            .iter()
            .map(|&(r, t)| Texels::crop(&atlas, t).map(|x| (r, x)))
            .collect::<Option<Vec<_>>>()
            .ok_or("a button is off the atlas")?;
        Ok(MinimapFrame {
            map: l.map,
            back,
            buttons,
            peasant: l.peasant,
            left: self.left.compose(&atlas)?,
            mid_window: self.left.mid_window,
            clock: self.left.clock,
            face: self.face.clone(),
            slot_spans: self.left.slots.clone(),
            slot_art: self.slot_art.clone(),
            banner: self.banner.compose(&atlas)?,
            banner_layout: self.banner.clone(),
            caps: self.caps.clone(),
            civ: std::sync::Arc::new(atlas),
            right: self.right.clone(),
            selected: self.selected.clone(),
            panel_faces: self.panel_faces.clone(),
            caption_face: self.caption_face.clone(),
        })
    }
}

impl MinimapFrame {
    /// The window's origin on a `size` frame: the map rectangle's corner
    /// put on [`crate::minimap::quad`]'s left and top corners.
    pub fn origin(&self, size: (i32, i32)) -> (i32, i32) {
        let q = crate::minimap::quad(size);
        (q[3].0 - self.map.0, q[0].1 - self.map.1)
    }

    /// **The bar's origin** (item 4724): the minimap window's, less
    /// `mid_window`'s corner in the bar.
    pub fn bar_origin(&self, size: (i32, i32)) -> (i32, i32) {
        let o = self.origin(size);
        (o.0 - self.mid_window.0, o.1 - self.mid_window.1)
    }

    /// **The left window with the clock written** (item 4727): `line`
    /// centred in the clock's box ([`left::write_clock`]); the window alone
    /// with no line or no face. Item 4731: `slots` drawn into the grid
    /// ([`slots::draw`]).
    pub fn left_with(&self, line: Option<&str>, slots: &[slots::Slot]) -> Texels {
        let mut win = self.left.clone();
        if let Some(art) = &self.slot_art {
            slots::draw(&mut win, &self.slot_spans, art.cells.0, art, slots);
        }
        if let (Some(line), Some(face)) = (line, &self.face) {
            left::write_clock(&mut win, face, self.clock, line);
        }
        win
    }

    /// The window's texel at screen `(x, y)`, the buttons over the back, or
    /// `None` off the window. Item 4764: the idle citizens' button only
    /// while `idle` ([`idle_lit`]); without it the back shows there.
    pub fn texel(&self, (ox, oy): (i32, i32), x: i32, y: i32, idle: bool) -> Option<[u8; 4]> {
        let (u, v) = (x - ox, y - oy);
        for (i, ((bl, bt, bw, bh), t)) in self.buttons.iter().enumerate().rev() {
            if !idle && self.peasant == Some(i) {
                continue;
            }
            let (bu, bv) = (u - bl, v - bt);
            if (0..*bw).contains(&bu) && (0..*bh).contains(&bv) {
                let c = t.at(bu as usize, bv as usize);
                if c[3] > 0 {
                    return Some(c);
                }
            }
        }
        let (w, h) = (self.back.w as i32, self.back.h as i32);
        ((0..w).contains(&u) && (0..h).contains(&v)).then(|| self.back.at(u as usize, v as usize))
    }

    /// The window as layers for the screen pass, the clock's `line` in
    /// the bar's box.
    #[allow(clippy::too_many_arguments)]
    pub fn layers(
        &self,
        size: (i32, i32),
        line: Option<&str>,
        slots: &[slots::Slot],
        figures: Option<caps::Figures>,
        panel: Option<selected::Panel>,
        caption: Option<&str>,
        idle: bool,
    ) -> Vec<crate::gpu::ScreenLayer> {
        let o = self.origin(size);
        let (x1, y1) = (
            (o.0 + self.back.w as i32).min(size.0),
            (o.1 + self.back.h as i32).min(size.1),
        );
        // Item 4738: the bar's right side, the parent's, under the
        // minimap's window, whose back runs to x 655.
        let mut v = Vec::new();
        if let Some((t, c)) = self.right_side(size, panel) {
            let (x1, y1) = (
                (c.0 + t.w as i32).min(size.0),
                (c.1 + t.h as i32).min(size.1),
            );
            v.extend(layers_of(size, (c.0.max(0), x1, c.1.max(0), y1), |x, y| {
                texel_of(&t, c, x, y).filter(|t| t[3] > 0)
            }));
        }
        v.extend(layers_of(size, (o.0.max(0), x1, o.1.max(0), y1), |x, y| {
            self.texel(o, x, y, idle)
        }));
        // Item 4724: the bar's left window; item 4727, its clock.
        let b = self.bar_origin(size);
        let win = self.left_with(line, slots);
        let (x1, y1) = (
            (b.0 + win.w as i32).min(size.0),
            (b.1 + win.h as i32).min(size.1),
        );
        v.extend(layers_of(size, (b.0.max(0), x1, b.1.max(0), y1), |x, y| {
            texel_of(&win, b, x, y)
        }));
        // Item 4737: the banner at the screen's top; item 4745, its
        // caption.
        let c = self.banner_layout.origin(size.0);
        let banner = self.banner_with(caption);
        let (x1, y1) = (
            (c.0 + banner.w as i32).min(size.0),
            (c.1 + banner.h as i32).min(size.1),
        );
        v.extend(layers_of(size, (c.0.max(0), x1, c.1.max(0), y1), |x, y| {
            texel_of(&banner, c, x, y)
        }));
        // Item 4739: the population box.
        if let Some((t, c)) = self.cap_box(size, figures) {
            let (x1, y1) = (
                (c.0 + t.w as i32).min(size.0),
                (c.1 + t.h as i32).min(size.1),
            );
            v.extend(layers_of(size, (c.0.max(0), x1, c.1.max(0), y1), |x, y| {
                texel_of(&t, c, x, y).filter(|t| t[3] > 0)
            }));
        }
        // Item 4742: the resources box.
        if let Some((t, c)) = self.resource_box(figures) {
            let (x1, y1) = (
                (c.0 + t.w as i32).min(size.0),
                (c.1 + t.h as i32).min(size.1),
            );
            v.extend(layers_of(size, (c.0.max(0), x1, c.1.max(0), y1), |x, y| {
                texel_of(&t, c, x, y).filter(|t| t[3] > 0)
            }));
        }
        v
    }

    /// **The military tally** as layers (item 4761, [`tally`]): `lines`
    /// right-aligned above the bar's right end, in the caption's face.
    pub fn tally_layers(&self, size: (i32, i32), lines: &[String]) -> Vec<crate::gpu::ScreenLayer> {
        let Some(face) = &self.caption_face else {
            return Vec::new();
        };
        let t = tally::compose(size, face, lines);
        let c = tally::origin(size);
        let (x1, y1) = (
            (c.0 + t.w as i32).min(size.0),
            (c.1 + t.h as i32).min(size.1),
        );
        layers_of(size, (c.0.max(0), x1, c.1.max(0), y1), |x, y| {
            texel_of(&t, c, x, y).filter(|t| t[3] > 0)
        })
    }

    /// **The banner with its caption** (item 4745, [`banner`]):
    /// `write_cent_l` of `caption` in the banner's span from y −2 to 23, in
    /// the caption's face and colour, over the composed crops. The banner
    /// alone without a caption or the face.
    pub fn banner_with(&self, caption: Option<&str>) -> Texels {
        let mut t = self.banner.clone();
        if let (Some(line), Some(face)) = (caption, &self.caption_face) {
            let rect = (0, banner::CAPTION_Y.0, t.w as i32, banner::CAPTION_Y.1);
            left::write_cent_ink(&mut t, face, rect, line, banner::CAPTION_INK, false);
        }
        t
    }

    /// **The resources box composed** (item 4742, [`resources`]) with
    /// `figures`' goods, the resource area open as on run1814b's
    /// photograph, and its corner; `None` without its art or figures.
    pub fn resource_box(&self, figures: Option<caps::Figures>) -> Option<(Texels, (i32, i32))> {
        let art = self.caps.as_ref()?;
        art.resources
            .compose(
                &figures?.goods,
                resources::PHOTOGRAPH_FLAGS,
                &art.common,
                &self.civ,
            )
            .ok()
    }

    /// **The bar's right side composed** (item 4738, [`right`]) on a
    /// `size` frame, and its corner on the screen. Item 4748: `panel`'s
    /// frame ([`selected`]) over it, at `right_window`'s corner.
    pub fn right_side(
        &self,
        size: (i32, i32),
        panel: Option<selected::Panel>,
    ) -> Option<(Texels, (i32, i32))> {
        let (mut t, (x, y)) = self.right.compose(&self.civ, size.0).ok()?;
        let b = self.bar_origin(size);
        let c = (b.0 + x, b.1 + y);
        if let (Some(p), Some(l), Some(art), Some(w)) =
            (panel, &self.selected, &self.caps, self.right.window(size.0))
        {
            let at = (b.0 + w.0, b.1 + w.1);
            let layers = l.layers(at, p.team, p.capital, &p.stats);
            let span = (c.0, c.1, c.0 + t.w as i32, c.1 + t.h as i32);
            let over = caps::paint_layers(span, &layers, &art.common, &self.civ).ok()?;
            let dst = t.rgba.as_chunks_mut::<4>().0;
            for (d, s) in dst.iter_mut().zip(over.rgba.as_chunks::<4>().0) {
                left::over(d, *s);
            }
            let local = |(x0, y0, x1, y1): left::Span| (x0 - c.0, y0 - c.1, x1 - c.0, y1 - c.1);
            let moved = |(x0, y0, x1, y1): left::Span| (x0 + at.0, y0 + at.1, x1 + at.0, y1 + at.1);
            // Item 4754: the `object` button, a child of the window, over
            // its frame: the type's picture, the overlay and the bar.
            if let Some(slot) = &self.slot_art {
                selected::object_button(
                    &mut t,
                    c,
                    moved(l.object),
                    p.picture.and_then(|r| slot.building(r)),
                    slot.overlay(false),
                    p.health,
                );
            }
            // Item 4752: `draw_build_name`'s line, `FontFamily::get(fonts,
            // 12, 1)` (the clock's face), `write_cent_l` in `name`, white
            // over a black shadow.
            if let Some(face) = &self.face {
                left::write_clock(&mut t, face, local(moved(l.name)), &p.title);
            }
            if let Some([bold, plain]) = &self.panel_faces {
                // Item 4750: each figure's line, `write_cent_l` over its
                // back; item 4752: in the window's face, `FontFamily::get(
                // fonts, 10, -1)`, size 10 in `def_style` 1, `arialbd`.
                for (_, _, span, s) in l.stat_pieces(at, &p.stats) {
                    left::write_clock(&mut t, bold, local(span), &s.line);
                }
                // `draw_hitpoints`: `FontFamily::get(fonts, 10, 0)`, `arial`,
                // white, no shadow (`COLOR_NONE`), in `hitpoints`.
                if let Some(h) = &p.hits {
                    left::write_cent(&mut t, plain, local(moved(l.hitpoints)), h, false);
                }
            }
        }
        Some((t, c))
    }

    /// **The population box composed** (item 4739, [`caps`]) on a `size`
    /// frame with `figures`, and its corner; `None` without its art or
    /// figures.
    pub fn cap_box(
        &self,
        size: (i32, i32),
        figures: Option<caps::Figures>,
    ) -> Option<(Texels, (i32, i32))> {
        let art = self.caps.as_ref()?;
        art.layout
            .compose(size, figures?, &art.common, &self.civ)
            .ok()
    }
}

fn texel_of(t: &Texels, (ox, oy): (i32, i32), x: i32, y: i32) -> Option<[u8; 4]> {
    let (u, v) = (x - ox, y - oy);
    let (w, h) = (t.w as i32, t.h as i32);
    ((0..w).contains(&u) && (0..h).contains(&v)).then(|| t.at(u as usize, v as usize))
}

/// **The bar's clock line in `world`** (item 4727): the speed's name off
/// `rules.xml`'s `gamespeeds`, at the world's speed, and the clock of its
/// frame ([`left::clock_line`]). `None` without the frame's art or a name.
pub fn clock_line(world: &crate::world::World) -> Option<String> {
    let art = world.frame_art.as_ref()?;
    let speed = art.speeds.get(usize::from(world.speed))?;
    Some(left::clock_line(speed, world.runner.frame()))
}

/// **The bar's slots in `world`** for `who`'s selection `sel` (item
/// 4731, [`slots::selected`]); empty without the slots' art.
pub fn bar_slots(
    world: &crate::world::World,
    who: sim::Player,
    sel: &[sim::observe::ObjId],
) -> Vec<slots::Slot> {
    let Some(art) = world.frame_art.as_ref().and_then(|a| a.slot_art.clone()) else {
        return Vec::new();
    };
    slots::selected(world.runner.sim(), &world.names.roster, &art, who, sel)
}

/// **The selected panel in `world`** for `who`'s selection `sel` (item
/// 4748, [`selected`]): one object, its owner's `TeamColor::get_unit`, and
/// whether it is a city's capital. `None` for none or a group, which
/// `on_redraw` draws no frame for.
pub fn selected_panel(
    world: &crate::world::World,
    sel: &[sim::observe::ObjId],
) -> Option<selected::Panel> {
    let [o] = sel else {
        return None;
    };
    let sim = world.runner.sim();
    let capital = sim.cities.cities.iter().any(|c| {
        c.alive
            && c.capital
            && c.owner == o.who
            && sim
                .buildings
                .get(c.building)
                .is_some_and(|b| b.index == o.o)
    });
    let b = sim
        .buildings
        .iter()
        .find(|b| b.owner == o.who && b.index == o.o && b.health > 0);
    Some(selected::Panel {
        team: render::silhouette::team_unit(o.who),
        capital,
        stats: building_stats(sim, o),
        title: b
            .map(|b| build_title(sim, &world.names.builds, b))
            .unwrap_or_default(),
        hits: b
            .filter(|_| selected::PROFILE_PREFS_2)
            .map(|b| format!("{}/{}", b.health, b.hits)),
        // Item 4754: the `object` button's picture and bar.
        picture: b.and_then(|b| b.ty),
        health: b.map(|b| (b.health, b.hits)),
    })
}

/// **`draw_build_name@00820c30`'s line** for building `b` (item 4752):
/// with no city, the type's name; a building of a city that is no city
/// centre (`is(VILLAGE)` false), `$BUILDINGTYPE0 -- $CITYNAME1`
/// (`loc_str_array +0xa2e4`); a city centre, `$CITYNAME1 --
/// $BUILDINGTYPE0` (`+0xa30c`) over the city and the type, which writes the
/// type first; the Forbidden City, `$CITYNAME0` (`+0xa2f8`). The type's
/// name is `buildingrules.xml`'s `NAME`; the type's own vtable `+0x9c`,
/// which may take a nation's `alternatenames.xml`, is unread (4752c).
pub fn build_title(
    sim: &sim::Sim,
    names: &[String],
    b: &sim::cities::building::Building,
) -> String {
    let ty = b.ty.and_then(|t| names.get(t)).cloned().unwrap_or_default();
    let Some(c) = b.city else {
        return ty;
    };
    let city = sim.city_name(c).unwrap_or_default();
    if ty == "Forbidden City" {
        city.to_string()
    } else {
        format!("{ty} -- {city}")
    }
}

/// **`draw_stat@008214d0`'s figures for a building** (item 4750), in
/// [`selected::STAT_ORDER`]: the six cases run1814b's capital shows, each
/// under the test its case makes, and no other (4750b).
/// - 1, the attack: the type's (`+0x1e8`) over ten, and the bonus
///   `(attack + 5 − type's) / 10` after a `+` where it is not zero;
/// - 0, the armor: `armor()` under the type's (`+0x214`), else the type's
///   and the difference after a `+` where there is one;
/// - 3, the range: `max_range()` (`BuildData::max_range`);
/// - 7, the garrison: `num_inside` `/` `get_garrison_limit`, finished;
/// - 8, the arrows: `get_garrison_arrows` `/` `get_most_shots`, finished;
/// - 19, the radius: a city centre's `CityData::get_radius`.
///
/// The `+` separator (`loc_str_array +0xa258`) is unread; the arrows take
/// the building's own `garrison_attack` without the squads' sum, which no
/// public door gives (4750b).
pub fn building_stats(sim: &sim::Sim, o: &sim::observe::ObjId) -> Vec<selected::Stat> {
    use sim::combat::Obj;
    let Some((gi, b)) = sim
        .buildings
        .iter()
        .enumerate()
        .find(|(_, b)| b.owner == o.who && b.index == o.o && b.health > 0)
    else {
        return Vec::new();
    };
    let me = Obj::Building(gi);
    let p = sim.profile(me);
    let attack = sim.attack_of(me);
    let with_bonus = |base: String, bonus: i32| {
        if bonus == 0 {
            base
        } else {
            format!("{base}+{bonus}")
        }
    };
    let line = |case: usize| -> Option<String> {
        match case {
            1 => (p.attack != 0 && p.max_range != 0)
                .then(|| with_bonus((p.attack / 10).to_string(), (5 - p.attack + attack) / 10)),
            0 => {
                let armor = sim.armor_of(me);
                Some(if armor < p.armor {
                    armor.to_string()
                } else {
                    with_bonus(p.armor.to_string(), armor - p.armor)
                })
            }
            3 => (p.attack != 0 && p.max_range != 0).then(|| sim.max_range_of(me).to_string()),
            7 => {
                let limit = sim.garrison_limit(gi);
                (b.active && limit != 0).then(|| format!("{}/{limit}", sim.num_inside(gi)))
            }
            8 => (b.active && p.attack != 0).then(|| {
                let arrows = sim::combat::garrison_arrows(
                    attack,
                    p.base_arrows,
                    p.most_shots,
                    b.garrison_attack,
                );
                format!("{arrows}/{}", p.most_shots)
            }),
            19 => b
                .city
                .filter(|&c| sim.cities.cities[c].building == gi)
                .map(|c| sim.radius_of(c).to_string()),
            _ => None,
        }
    };
    selected::STAT_ORDER
        .iter()
        .filter_map(|&case| {
            Some(selected::Stat {
                case,
                line: line(case)?,
            })
        })
        .collect()
}

/// **The population box's figures in `world`** for `who` (item 4739):
/// `LeaderData +0x940` over `+0x7e4`, the muster's `control` over its
/// `cap`, and `get_total_cities` over `get_city_limit`.
pub fn cap_figures(world: &crate::world::World, who: sim::Player) -> Option<caps::Figures> {
    let sim = world.runner.sim();
    let m = sim.roster.muster.get(who as usize)?;
    let l = sim.economy.ledgers.get(who as usize)?;
    // Item 4742: each good `type_avail` gives, its stockpile and income;
    // `draw_cap`'s least `resource_cap` of them, from 15,984 (999 × 16).
    let avail: [bool; 6] = std::array::from_fn(|g| sim.type_avail(who, g) != 0);
    let goods = resources::Goods {
        shown: std::array::from_fn(|g| avail[g].then(|| (l.bucket[g], l.income[g]))),
        cap: (0..6)
            .filter(|&g| avail[g])
            .map(|g| l.cap[g])
            .fold(15_984, i32::min),
    };
    Some(caps::Figures {
        pop: m.control,
        pop_cap: m.cap,
        cities: sim.total_cities(who),
        city_limit: sim.city_limit(who),
        goods,
    })
}

/// **The caption's line in `world`** for `who` (item 4745,
/// [`banner::caption_line`]): the tribe's name and the age's, the age type
/// `ages` owned names (`TechTree::ages`, Classical first), or with none
/// owned `rules.xml`'s first `startingtechs`.
pub fn caption(world: &crate::world::World, who: sim::Player) -> Option<String> {
    let art = world.frame_art.as_ref()?;
    let sim = world.runner.sim();
    let tree = &sim.tables.tech_tree;
    let p = sim.tech.get(who as usize)?;
    let tribe = tree.tribes.get(p.tribe).map(|t| t.name.as_str());
    let age = match usize::try_from(p.ages).ok()?.checked_sub(1) {
        None => art.start_ages.first()?.clone(),
        Some(n) => tree.types.get((*tree.ages.get(n)?)?)?.name.clone(),
    };
    Some(banner::caption_line(tribe, &age))
}

/// **The military tally's lines in `world`** for `who` (item 4761,
/// [`tally::military_lines`]): the units of the scene and the cities
/// alive and owned, `city_num`'s count.
pub fn tally_lines(world: &crate::world::World, who: sim::Player) -> Vec<String> {
    let sim = world.runner.sim();
    let cities = sim
        .cities
        .cities
        .iter()
        .filter(|c| c.alive && c.owner == who)
        .count() as i32;
    let scene = world.runner.scene(None);
    tally::military_lines(&scene, &world.names.units, who, cities)
}

/// **Whether the idle citizens' button is lit** for `who` in `world` (item
/// 4764; `docs/RENDERER.md` §6.25): `IFaceMiniMap::update_visibility@008154c0`,
/// which `Game::loop_render` reaches on every rendered frame through
/// `IFaceMainBase::update_visibility`, enables `cycle_peasant` while the
/// leader's `leader_flags & 0x80000` is set and otherwise disables it and
/// sets its `flags & 8`, on which `IFaceButton::on_redraw@007fc730` draws
/// nothing. The bit is the sim's [`sim::leaders::LeadersState::citizen_thought`].
pub fn idle_lit(world: &crate::world::World, who: sim::Player) -> bool {
    let sim = world.runner.sim();
    sim.leaders
        .citizen_thought
        .get(who as usize)
        .copied()
        .unwrap_or(false)
}

/// **The console's frame** in `world`: the atlas its nation and age pick.
pub fn console_frame(
    world: &crate::world::World,
    who: sim::Player,
) -> Option<std::sync::Arc<MinimapFrame>> {
    let art = world.frame_art.as_ref()?;
    let sim = world.runner.sim();
    let code = sim.tech.get(who as usize).and_then(|p| {
        let t = sim.tables.tech_tree.tribes.get(p.tribe)?;
        civ_code(t.build_continent, p.ages)
    });
    art.frame(code).or_else(|| art.frame(None))
}

#[cfg(test)]
mod tests;
