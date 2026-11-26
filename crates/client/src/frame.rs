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

pub mod left;

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
}

/// **The frame's art**: interface.xml's layout, and each nation's atlas as
/// it is first asked for.
pub struct FrameArt {
    root: std::path::PathBuf,
    layout: MinimapLayout,
    left: left::LeftLayout,
    /// interface.xml's own `TEX_CIV` file, the fallback.
    default_civ: String,
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
        let default_civ = xml
            .split('<')
            .filter(|t| t.starts_with("IFACETEXLOAD"))
            .find(|t| attr(t, "id") == Some("TEX_CIV"))
            .and_then(|t| attr(t, "file"))
            .ok_or("interface.xml: no TEX_CIV")?
            .to_string();
        Ok(FrameArt {
            root: root.to_path_buf(),
            layout,
            left,
            default_civ,
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

    /// The left window's texel at screen `(x, y)`, or `None` off it.
    pub fn left_texel(&self, (ox, oy): (i32, i32), x: i32, y: i32) -> Option<[u8; 4]> {
        let (u, v) = (x - ox, y - oy);
        let (w, h) = (self.left.w as i32, self.left.h as i32);
        ((0..w).contains(&u) && (0..h).contains(&v)).then(|| self.left.at(u as usize, v as usize))
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

    /// The window as layers for the screen pass.
    pub fn layers(&self, size: (i32, i32)) -> Vec<crate::gpu::ScreenLayer> {
        let o = self.origin(size);
        let (x1, y1) = (
            (o.0 + self.back.w as i32).min(size.0),
            (o.1 + self.back.h as i32).min(size.1),
        );
        let mut v = layers_of(size, (o.0.max(0), x1, o.1.max(0), y1), |x, y| {
            self.texel(o, x, y)
        });
        // Item 4724: the bar's left window.
        let b = self.bar_origin(size);
        let (x1, y1) = (
            (b.0 + self.left.w as i32).min(size.0),
            (b.1 + self.left.h as i32).min(size.1),
        );
        v.extend(layers_of(size, (b.0.max(0), x1, b.1.max(0), y1), |x, y| {
            self.left_texel(b, x, y)
        }));
        v
    }
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
