//! **A plain HUD: the menu and the panel** (item 4197, `docs/HUD.md` §2,
//! §3, §5). Text and rectangles in the debug overlay's font
//! ([`crate::overlay`]), in the lobby page's manner; **not the original's
//! interface**, whose windows stay on the cut list.
//!
//! - **The menu** is one button for each key the selection can order
//!   from: units selected, a [`BUILD_KEYS`] row; a building selected, a
//!   [`TRAIN_KEYS`] row; nothing selected, none. **A button is its key's
//!   event and nothing else**: [`route`] turns a press inside a button's
//!   rectangle into the key it shows, and the input layer's arms are
//!   untouched, so a button can mint nothing a key cannot. A press on a
//!   button never reaches the layer's press arm: it never clears the
//!   selection and never places.
//! - **The panel** prints the selection from what the scene carries: one
//!   line an object, its type's name, `health` of its full health, and an
//!   unfinished building's construction (`construct_hits`) of the full as
//!   well. Nothing is scaled or computed beyond the join to a name.
//!
//! Where the buttons stand is ours, anchored to the window's bottom-left
//! corner, and pinned as pixels at the scripts' 1920×1080 by the tests.
//! The HUD is drawn in a window, and under `--still` only with `--select`;
//! never on a row's still ([`crate::still_of`] draws no overlay).

use crate::input::{BUILD_KEYS, Event, Input, TRAIN_KEYS, View};
use crate::overlay::{self, Painter};
use rondata::recgame::Order;
use sim::observe::Scene;

/// The font's scale: the debug overlay's.
pub const SCALE: i32 = 2;
/// The gap between the HUD's boxes and the window's edge, in pixels.
pub const MARGIN: i32 = 8;
/// A button's size, in pixels.
pub const BUTTON: (i32, i32) = (240, 36);
/// The lines the panel holds before it closes with a count of the rest.
pub const PANEL_LINES: usize = 6;
/// The panel's least width, in pixels; a longer line widens it.
pub const PANEL_WIDTH: i32 = 480;

/// The install's names, as the interface shows them (`NAME`): one a unit
/// type, one a building type.
#[derive(Clone, Debug, Default)]
pub struct Names {
    pub units: Vec<String>,
    pub builds: Vec<String>,
}

impl Names {
    pub fn new(loaded: &rondata::load::Loaded) -> Names {
        Names {
            units: loaded.unit_names.clone(),
            builds: loaded.build_names.clone(),
        }
    }

    /// The name of the type a `TypeIndex` names: a unit's from
    /// `BASE_UNITTYPES` (0x32, the Citizen first), a building's from
    /// `BASE_BUILDTYPES` (`rondata::load::Loaded::type_index`).
    pub fn of_type_index(&self, ti: i32) -> String {
        use rondata::load::{BASE_BUILDTYPES, BASE_UNITTYPES};
        let at = |table: &[String], base: i32| {
            usize::try_from(ti - base)
                .ok()
                .and_then(|i| table.get(i).cloned())
        };
        let name = if ti >= BASE_BUILDTYPES {
            at(&self.builds, BASE_BUILDTYPES)
        } else {
            at(&self.units, BASE_UNITTYPES)
        };
        name.unwrap_or_else(|| format!("TYPE {ti}"))
    }
}

/// A rectangle on the window, in pixels: its top-left corner and size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub fn contains(&self, px: i32, py: i32) -> bool {
        (self.x..self.x + self.w).contains(&px) && (self.y..self.y + self.h).contains(&py)
    }

    /// The pixel at its centre.
    #[cfg(test)]
    pub fn centre(&self) -> (i32, i32) {
        (self.x + self.w / 2, self.y + self.h / 2)
    }
}

/// One button of the menu: the key it is, the `TypeIndex` that key
/// orders, and where it stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Button {
    pub key: char,
    pub ty: i32,
    pub rect: Rect,
}

/// **The menu** for the session's state on a `size` window: a building
/// selected, a button each [`TRAIN_KEYS`] row; units selected, each
/// [`BUILD_KEYS`] row; nothing selected, none. In one row along the
/// bottom edge, left to right in the table's order.
pub fn menu(input: &Input, size: (i32, i32)) -> Vec<Button> {
    if input.selection.is_empty() {
        return Vec::new();
    }
    let rows = if input.building {
        TRAIN_KEYS
    } else {
        BUILD_KEYS
    };
    let (w, h) = BUTTON;
    rows.iter()
        .enumerate()
        .map(|(k, &(key, ty))| Button {
            key,
            ty,
            rect: Rect {
                x: MARGIN + k as i32 * (w + MARGIN),
                y: size.1 - MARGIN - h,
                w,
                h,
            },
        })
        .collect()
}

/// **The window asks the HUD first**: a press inside a button is that
/// button's key; every other event is the layer's, unchanged.
pub fn route(input: &Input, size: (i32, i32), ev: Event) -> Event {
    match ev {
        Event::Press { x, y } => menu(input, size)
            .into_iter()
            .find(|b| b.rect.contains(x, y))
            .map_or(ev, |b| Event::Key(b.key)),
        _ => ev,
    }
}

/// One event through the HUD and then the layer: the window's own path
/// (`App::input`), and the one the tests mint through.
pub fn event(
    input: &mut Input,
    size: (i32, i32),
    frame: i32,
    ev: Event,
    view: &View,
) -> Option<Order> {
    let ev = route(input, size, ev);
    input.event(frame, ev, view)
}

/// One line of the panel: the object's name and the numbers it prints,
/// each with the word before it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    pub name: String,
    /// `health` of its full health.
    pub health: (i32, i32),
    /// An unfinished building's `construct_hits` of its full health.
    pub building: Option<(i32, i32)>,
    /// A unit's figures, where it has more than one.
    pub figures: Option<usize>,
}

impl Line {
    pub fn text(&self) -> String {
        let mut s = format!("{} {}/{}", self.name, self.health.0, self.health.1);
        if let Some((c, full)) = self.building {
            s += &format!(" BUILDING {c}/{full}");
        }
        if let Some(n) = self.figures {
            s += &format!(" FIGURES {n}");
        }
        s
    }
}

/// **The panel's lines** for the selection, from the scene alone: a unit,
/// its type's name and `health` of `max_health`; a building, its type's
/// name, `health` of `hits`, and unfinished, `construct_hits` of `hits`.
/// An object no longer in the scene prints nothing.
pub fn panel(input: &Input, scene: &Scene, names: &Names) -> Vec<Line> {
    let name = |table: &[String], ty: Option<usize>| {
        ty.and_then(|t| table.get(t))
            .cloned()
            .unwrap_or_else(|| "OBJECT".into())
    };
    input
        .selection
        .iter()
        .filter_map(|id| {
            if let Some(u) = scene.units.iter().find(|u| u.id == *id) {
                return Some(Line {
                    name: name(&names.units, u.ty),
                    health: (u.health, u.max_health),
                    building: None,
                    figures: (u.figures.len() > 1).then_some(u.figures.len()),
                });
            }
            let b = scene.buildings.iter().find(|b| b.id == *id)?;
            Some(Line {
                name: name(&names.builds, b.ty),
                health: (b.health, b.hits),
                building: (!b.active).then_some((b.construct_hits, b.hits)),
                figures: None,
            })
        })
        .collect()
}

/// The panel's printed text: its lines, capped at [`PANEL_LINES`] with a
/// closing count of the rest.
pub fn panel_text(lines: &[Line]) -> Vec<String> {
    let mut out: Vec<String> = lines.iter().take(PANEL_LINES).map(Line::text).collect();
    if lines.len() > PANEL_LINES {
        out.push(format!("AND {} MORE", lines.len() - PANEL_LINES));
    }
    out
}

/// Where the panel stands for its printed lines: above the menu's row,
/// at the left edge.
pub fn panel_rect(text: &[String], size: (i32, i32)) -> Rect {
    let h = text.len() as i32 * overlay::line_height(SCALE) + 4 * SCALE;
    let longest = text.iter().map(|l| l.chars().count()).max().unwrap_or(0) as i32;
    Rect {
        x: MARGIN,
        y: size.1 - 2 * MARGIN - BUTTON.1 - h,
        w: PANEL_WIDTH.max(longest * overlay::advance(SCALE) + 4 * SCALE),
        h,
    }
}

/// The button's label: its key, then the type's name.
pub fn label(b: &Button, names: &Names) -> String {
    format!("{} {}", b.key, names.of_type_index(b.ty))
}

/// **The HUD drawn**: the panel and the menu as quads on `p`.
pub fn paint(p: &mut Painter, input: &Input, scene: &Scene, names: &Names, size: (i32, i32)) {
    let text = panel_text(&panel(input, scene, names));
    if !text.is_empty() {
        let r = panel_rect(&text, size);
        p.rect(r.x, r.y, r.w, r.h, overlay::PANEL);
        for (row, line) in text.iter().enumerate() {
            p.text(
                r.x + 2 * SCALE,
                r.y + 2 * SCALE + row as i32 * overlay::line_height(SCALE),
                line,
                SCALE,
                overlay::INK,
            );
        }
    }
    for b in menu(input, size) {
        let r = b.rect;
        p.rect(r.x, r.y, r.w, r.h, [0.12, 0.14, 0.18]);
        p.rect(r.x, r.y, r.w, SCALE, overlay::INK);
        let ty = r.y + (r.h - 7 * SCALE) / 2;
        p.text(r.x + 3 * SCALE, ty, &label(&b, names), SCALE, overlay::INK);
    }
}

#[cfg(test)]
mod tests;
