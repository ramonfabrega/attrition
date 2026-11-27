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
pub mod slots;

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
        for (i, t) in tags.iter().enumerate() {
            if !t.starts_with("BUTTON ") {
                continue;
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
        })
    }
}

/// **The minimap's window drawn**: the back and its buttons, cut from one
/// nation's atlas.
pub struct MinimapFrame {
    pub map: Rect,
    pub back: Texels,
    pub buttons: Vec<(Rect, Texels)>,
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
    /// interface.xml's own `TEX_CIV` file, the fallback.
    default_civ: String,
    /// Item 4727: the clock's face, and `rules.xml`'s `gamespeeds`, the
    /// names `TurnControl::speed` indexes (`game_speeds`).
    face: Option<crate::selection::PlateFace>,
    pub speeds: Vec<String>,
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
        let default_civ = xml
            .split('<')
            .filter(|t| t.starts_with("IFACETEXLOAD"))
            .find(|t| attr(t, "id") == Some("TEX_CIV"))
            .and_then(|t| attr(t, "file"))
            .ok_or("interface.xml: no TEX_CIV")?
            .to_string();
        let (file, ppem, ascent, height) = left::CLOCK_FACE;
        let face = crate::selection::PlateFace::load_face(root, file, ppem, ascent, height).ok();
        let speeds = std::fs::read_to_string(root.join("Data").join("rules.xml"))
            .ok()
            .and_then(|x| {
                let b = x
                    .split("<CATEGORIES id=\"gamespeeds\"")
                    .nth(1)?
                    .split("</CATEGORIES>")
                    .next()?
                    .to_string();
                Some(
                    b.split('<')
                        .filter(|t| t.starts_with("CATEGORY "))
                        .filter_map(|t| attr(t, "name").map(String::from))
                        .collect(),
                )
            })
            .unwrap_or_default();
        Ok(FrameArt {
            root: root.to_path_buf(),
            layout,
            left,
            banner,
            right,
            default_civ,
            face,
            speeds,
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
    /// `None` off the window.
    pub fn texel(&self, (ox, oy): (i32, i32), x: i32, y: i32) -> Option<[u8; 4]> {
        let (u, v) = (x - ox, y - oy);
        for ((bl, bt, bw, bh), t) in self.buttons.iter().rev() {
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
    pub fn layers(
        &self,
        size: (i32, i32),
        line: Option<&str>,
        slots: &[slots::Slot],
        figures: Option<caps::Figures>,
    ) -> Vec<crate::gpu::ScreenLayer> {
        let o = self.origin(size);
        let (x1, y1) = (
            (o.0 + self.back.w as i32).min(size.0),
            (o.1 + self.back.h as i32).min(size.1),
        );
        // Item 4738: the bar's right side, the parent's, under the
        // minimap's window, whose back runs to x 655.
        let mut v = Vec::new();
        if let Some((t, c)) = self.right_side(size) {
            let (x1, y1) = (
                (c.0 + t.w as i32).min(size.0),
                (c.1 + t.h as i32).min(size.1),
            );
            v.extend(layers_of(size, (c.0.max(0), x1, c.1.max(0), y1), |x, y| {
                texel_of(&t, c, x, y).filter(|t| t[3] > 0)
            }));
        }
        v.extend(layers_of(size, (o.0.max(0), x1, o.1.max(0), y1), |x, y| {
            self.texel(o, x, y)
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
        // Item 4737: the banner at the screen's top.
        let c = self.banner_layout.origin(size.0);
        let (x1, y1) = (
            (c.0 + self.banner.w as i32).min(size.0),
            (c.1 + self.banner.h as i32).min(size.1),
        );
        v.extend(layers_of(size, (c.0.max(0), x1, c.1.max(0), y1), |x, y| {
            texel_of(&self.banner, c, x, y)
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
    /// `size` frame, and its corner on the screen.
    pub fn right_side(&self, size: (i32, i32)) -> Option<(Texels, (i32, i32))> {
        let (t, (x, y)) = self.right.compose(&self.civ, size.0).ok()?;
        let b = self.bar_origin(size);
        Some((t, (b.0 + x, b.1 + y)))
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
