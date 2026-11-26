//! **The bar's slots** (item 4731; `docs/RENDERER.md` §6.11): what the
//! options grid of the left window shows for a selection, and each slot's
//! picture drawn into it.
//!
//! **The list.** `BuildOut::make_options@00629ed0` adds a building's
//! options with `Options::add(option, x, y, type, disable, …)`, and
//! `setup_selected@008165b0`'s first loop gives each add its own grid cell
//! unless an earlier add already took it. [`selected`] builds the city
//! arm of `make_options` for a city centre selected alone: the rename
//! (line 159 of the export), the trains and research the roster offers
//! ([`crate::input::Roster::offered`], at each type's `grid_x`/`grid_y`,
//! `TypeData` `+0x5c`/`+0x5d`, the rules' `GRID_X`/`GRID_Y`), the rally
//! point (`OPTION_GATHER`, line 898), the alarm (line 982) and the
//! upgrades (line 1354). Any other selection gets no slots.
//!
//! **The pictures** (`IFaceButton::on_redraw@007fc730`). An option slot
//! takes `IFACEBUTTON_OPTION`'s `tex` (`TEX_ORDERS_0`, 61 × 39), moved by
//! `(w + 1)·tex_col`, `(h + 1)·tex_row` from `options.xml`'s entry for the
//! option (`OptionInfo::init@0072c530`, positional). A type slot takes the
//! type's own `TEX_ID`, `TEX_COL`, `TEX_ROW` on the same grid. A disabled
//! slot moves its rectangle by (−1, −1) and is drawn at half colour, and
//! the overlay frame goes on top at `0x80/0xff` (`0x40/0xff` disabled),
//! each channel `round(c·k / 255)`. Registered on run1814b's photograph at
//! 2,379 of 2,379 pixels per slot (item 4729).

use crate::input::{Button, Roster};
use crate::selection::attr;
use rondata::load::{BASE_BUILDTYPES, BASE_TECHTYPES, BASE_UNITTYPES};

use super::left::Crop;

/// `OPTION_ALARM` and `OPTION_ALL_CLEAR`: the city's alarm and its stand
/// down, at (4, 0).
pub const OPTION_ALARM: i32 = 28;
pub const OPTION_ALL_CLEAR: i32 = 29;
/// `OPTION_UPGRADE`, the City line's next type at (0, 3), greyed.
pub const OPTION_UPGRADE: i32 = 50;
/// `OPTION_GATHER`, the rally point at (3, 3).
pub const OPTION_GATHER: i32 = 51;
/// `OPTION_RENAME_CITY` at (3, 0).
pub const OPTION_RENAME_CITY: i32 = 279;

/// `VILLAGE`, the head of the City line (`BASE_BUILDTYPES`).
const VILLAGE: i32 = 0x19e;
/// `FORBIDDENCITY`, a city the rename skips.
const FORBIDDENCITY: i32 = 0x213;
/// The upgrade loop's end, exclusive.
const END_UPGRADES: i32 = 0x21f;

/// One add: the option, its type (−1 for none), `disable` and its cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slot {
    pub option: i32,
    pub ty: i32,
    pub disable: i32,
    pub grid: (i32, i32),
}

impl Slot {
    fn typeless(option: i32, grid: (i32, i32)) -> Slot {
        Slot {
            option,
            ty: -1,
            disable: 0,
            grid,
        }
    }
}

/// **`setup_selected`'s first loop**: each add at its own cell, unless an
/// earlier add took it; adds off the `cols × rows` grid are dropped.
pub fn place(adds: &[Slot], (cols, rows): (i32, i32)) -> Vec<Slot> {
    let mut out: Vec<Slot> = Vec::new();
    for s in adds {
        let (x, y) = s.grid;
        let on = (0..cols).contains(&x) && (0..rows).contains(&y);
        if on && !out.iter().any(|o| o.grid == s.grid) {
            out.push(*s);
        }
    }
    out
}

/// **The slots of `sel` for player `who`** on `sim`'s state: the city arm
/// of `make_options` for one of `who`'s finished city centres selected
/// alone, placed on the grid; empty for any other selection.
///
/// Its predicates, as the export reads them, and where ours stand in:
/// - the rename: the building `is(VILLAGE)`, the selection holds one
///   (`get_num_cap_const == 1`), the game is no scenario (a lobby game is
///   not), and the building is not the Forbidden City;
/// - the rally point: reached for a city by the branch whose type has no
///   `+0x2c0` slot (not read; assumed for the City line), not `is(0x208)`
///   and not type `0x1a4`;
/// - the alarm: the building is its city's centre and the selection under
///   two; `OPTION_ALL_CLEAR` when the city's alarm is up;
/// - the upgrades: each building type over `VILLAGE..0x21f` the building
///   may queue (`queue_here`), not its own type, not being researched; the
///   ending technology's age check is not built (the lobby's last age
///   passes every type).
pub fn selected(
    sim: &sim::Sim,
    roster: &Roster,
    art: &SlotArt,
    who: sim::Player,
    sel: &[sim::observe::ObjId],
) -> Vec<Slot> {
    let [id] = sel else {
        return Vec::new();
    };
    let Some((gi, b)) = sim
        .buildings
        .iter()
        .enumerate()
        .find(|(_, b)| b.owner == id.who && b.index == id.o && b.health > 0)
    else {
        return Vec::new();
    };
    if b.owner != who || !b.active {
        return Vec::new();
    }
    let Some(bty) = b.ty else {
        return Vec::new();
    };
    let ti = bty as i32 + BASE_BUILDTYPES;
    let tree = &sim.tables.tech_tree;
    let tree_of = |ti: i32| {
        sim.tables
            .build_types
            .get(usize::try_from(ti - BASE_BUILDTYPES).ok()?)?
            .tree
    };
    let Some(bt) = tree_of(ti) else {
        return Vec::new();
    };
    let is = |x: i32| tree_of(x).is_some_and(|x| tree.is(bt, x, false));
    if !is(VILLAGE) {
        return Vec::new();
    }
    let mut adds = Vec::new();
    if !is(FORBIDDENCITY) {
        adds.push(Slot::typeless(OPTION_RENAME_CITY, (3, 0)));
    }
    adds.extend(
        roster
            .offered(sim, usize::from(who), ti, b.active)
            .into_iter()
            .filter_map(
                |Button {
                     ty,
                     option,
                     disable,
                 }| {
                    Some(Slot {
                        option,
                        ty,
                        disable,
                        grid: art.grid(ty)?,
                    })
                },
            ),
    );
    adds.push(Slot::typeless(OPTION_GATHER, (3, 3)));
    let centre = b
        .city
        .and_then(|c| sim.cities.cities.get(c))
        .filter(|c| c.building == gi);
    if let Some(c) = centre {
        let option = if c.alarm {
            OPTION_ALL_CLEAR
        } else {
            OPTION_ALARM
        };
        adds.push(Slot::typeless(option, (4, 0)));
    }
    for t in VILLAGE..END_UPGRADES {
        let Some(tt) = tree_of(t) else { continue };
        if t != ti && tree.queue_here(bt, tt) && !sim.researching(who, tt) {
            adds.push(Slot {
                option: OPTION_UPGRADE,
                ty: t,
                disable: 1,
                grid: (0, 3),
            });
        }
    }
    place(&adds, art.cells)
}

/// A texture page's id and a cell on its grid.
type PageCell = (String, i32, i32);

/// **The slots' art, off the install**: each option's and each type's
/// picture, each type's cell, the button classes' rectangles and the
/// overlays, and the pages they name.
pub struct SlotArt {
    /// `options.xml`'s entries in order, `None` for one with no picture.
    options: Vec<Option<PageCell>>,
    /// Per unit record: its `GRID_X`/`GRID_Y`, and its picture (the
    /// `{GRAPH}-DEFAULT-AGE0` entry of `unit_graphics.xml`).
    units: Vec<((i32, i32), Option<PageCell>)>,
    /// Per building record: its picture (`TEX_ID`, `TEX_COL`, `TEX_ROW`).
    builds: Vec<Option<PageCell>>,
    /// Per technology record: its cell and picture.
    techs: Vec<((i32, i32), Option<PageCell>)>,
    /// `IFACEBUTTONDATA`'s `tex` rectangle (the cell (0, 0) of every page):
    /// left, top, width, height.
    base: Crop,
    /// `def_overlay_texs0` and `def_overlay_texs1` on `TEX_COMMON`.
    overlays: [Crop; 2],
    /// The grid's columns and rows (`num_x_slots`, `num_y_slots`).
    pub cells: (i32, i32),
    /// Each page named above, decoded, by its id.
    pages: std::collections::BTreeMap<String, render::tga::Image>,
}

fn tag_text<'a>(rec: &'a str, tag: &str) -> Option<&'a str> {
    let open = format!("<{tag}>");
    let at = rec.find(&open)? + open.len();
    let len = rec[at..].find('<')?;
    Some(rec[at..at + len].trim())
}

/// The records of `xml` opened by `<{tag}` and a space, `>` or `/`, in
/// order, comments removed: each record's text up to the next.
fn records(xml: &str, tag: &str) -> Vec<String> {
    let mut clean = String::with_capacity(xml.len());
    let mut rest = xml;
    while let Some(at) = rest.find("<!--") {
        clean.push_str(&rest[..at]);
        rest = rest[at..].split_once("-->").map_or("", |(_, r)| r);
    }
    clean.push_str(rest);
    let open = format!("<{tag}");
    let mut out = Vec::new();
    let mut parts = clean.split(&open);
    parts.next();
    for p in parts {
        if p.starts_with([' ', '>', '/', '\n', '\t', '\r']) {
            out.push(p.to_string());
        } else if let Some(last) = out.last_mut() {
            last.push_str(&open);
            last.push_str(p);
        }
    }
    out
}

fn cell_of(id: Option<&str>, col: Option<&str>, row: Option<&str>) -> Option<PageCell> {
    let (col, row): (i32, i32) = (col?.trim().parse().ok()?, row?.trim().parse().ok()?);
    let id = id?.trim().to_string();
    (col >= 0 && row >= 0).then_some((id, col, row))
}

fn grid_of(rec: &str) -> (i32, i32) {
    let n = |t| tag_text(rec, t).and_then(|v| v.parse().ok()).unwrap_or(-1);
    (n("GRID_X"), n("GRID_Y"))
}

fn crop_attrs(tag: &str) -> Option<Crop> {
    let n = |k| attr(tag, k).and_then(|v| v.parse::<usize>().ok());
    Some((n("left")?, n("top")?, n("width")?, n("height")?))
}

impl SlotArt {
    pub fn load(root: &std::path::Path) -> Result<SlotArt, String> {
        let data = root.join("Data");
        let read = |f: &str| {
            std::fs::read(data.join(f))
                .map(|b| String::from_utf8_lossy(&b).into_owned())
                .map_err(|e| format!("{f}: {e}"))
        };
        let options = records(&read("options.xml")?, "OPTION")
            .iter()
            .map(|r| {
                let head = r.split('>').next().unwrap_or("");
                cell_of(
                    attr(head, "tex_id"),
                    attr(head, "tex_col"),
                    attr(head, "tex_row"),
                )
            })
            .collect();
        // `unit_graphics.xml`'s pictures by the entry's name.
        let graphics = read("unit_graphics.xml")?;
        let pictures: std::collections::BTreeMap<&str, PageCell> = graphics
            .split('<')
            .filter(|t| t.starts_with("UNIT "))
            .filter_map(|t| {
                let cell = cell_of(attr(t, "tex_id"), attr(t, "tex_col"), attr(t, "tex_row"))?;
                Some((attr(t, "name")?, cell))
            })
            .collect();
        let units = records(&read("unitrules.xml")?, "UNIT")
            .iter()
            .map(|r| {
                let pic = tag_text(r, "GRAPH")
                    .and_then(|g| pictures.get(format!("{g}-DEFAULT-AGE0").as_str()))
                    .cloned();
                (grid_of(r), pic)
            })
            .collect();
        let rules_pic = |r: &str| {
            cell_of(
                tag_text(r, "TEX_ID"),
                tag_text(r, "TEX_COL"),
                tag_text(r, "TEX_ROW"),
            )
        };
        let builds = records(&read("buildingrules.xml")?, "BUILDING")
            .iter()
            .map(|r| rules_pic(r))
            .collect();
        let techs = records(&read("techrules.xml")?, "TECH")
            .iter()
            .map(|r| (grid_of(r), rules_pic(r)))
            .collect();
        let iface = read("interface.xml")?;
        let class = iface
            .split("<IFACEBUTTONCLASS>")
            .nth(1)
            .and_then(|s| s.split("</IFACEBUTTONCLASS>").next())
            .ok_or("interface.xml: no <IFACEBUTTONCLASS>")?;
        let ctags: Vec<&str> = class.split('<').collect();
        let tex_named = |name: &str, texture: &str| {
            ctags
                .iter()
                .find(|t| {
                    t.starts_with("IFACETEX ")
                        && attr(t, "name") == Some(name)
                        && attr(t, "texture") == Some(texture)
                })
                .and_then(|t| crop_attrs(t))
                .ok_or(format!("IFACEBUTTONCLASS: no {name} on {texture}"))
        };
        let overlays = [
            tex_named("def_overlay_texs0", "TEX_COMMON")?,
            tex_named("def_overlay_texs1", "TEX_COMMON")?,
        ];
        let base = tex_named("tex", "TEX_ORDERS_0")?;
        let head = iface
            .split("<IFACEOPTIONS name=\"main\"")
            .nth(1)
            .and_then(|s| s.split('>').next())
            .ok_or("interface.xml: no main IFACEOPTIONS")?;
        let n = |k| attr(head, k).and_then(|v| v.parse::<i32>().ok());
        let cells = (
            n("num_x_slots").ok_or("IFACEOPTIONS: no num_x_slots")?,
            n("num_y_slots").ok_or("IFACEOPTIONS: no num_y_slots")?,
        );
        let mut art = SlotArt {
            options,
            units,
            builds,
            techs,
            base,
            overlays,
            cells,
            pages: Default::default(),
        };
        // The pages: every id a picture names, and `TEX_COMMON`.
        let mut ids: std::collections::BTreeSet<String> = art
            .options
            .iter()
            .chain(art.builds.iter())
            .chain(art.units.iter().map(|(_, p)| p))
            .chain(art.techs.iter().map(|(_, p)| p))
            .flatten()
            .map(|(id, _, _)| id.clone())
            .collect();
        ids.insert("TEX_COMMON".into());
        for id in ids {
            let Some(file) = iface
                .split('<')
                .filter(|t| t.starts_with("IFACETEXLOAD"))
                .find(|t| attr(t, "id") == Some(id.as_str()))
                .and_then(|t| attr(t, "file"))
            else {
                continue;
            };
            let Some(path) = rondata::artdata::resolve_art(root, file) else {
                continue;
            };
            let Ok(bytes) = std::fs::read(&path) else {
                continue;
            };
            if let Ok(img) = render::tga::decode(&bytes) {
                art.pages.insert(id, img);
            }
        }
        if !art.pages.contains_key("TEX_COMMON") {
            return Err("interface.xml: TEX_COMMON does not load".into());
        }
        Ok(art)
    }

    /// The cell a type's button takes in its building's panel.
    pub fn grid(&self, ty: i32) -> Option<(i32, i32)> {
        let at = |base: i32| usize::try_from(ty - base).ok();
        let g = if ty >= BASE_TECHTYPES {
            self.techs.get(at(BASE_TECHTYPES)?)?.0
        } else if (BASE_UNITTYPES..BASE_BUILDTYPES).contains(&ty) {
            self.units.get(at(BASE_UNITTYPES)?)?.0
        } else {
            return None;
        };
        (g.0 >= 0 && g.1 >= 0).then_some(g)
    }

    /// **A slot's picture**: its page, and the crop on it, moved by
    /// (−1, −1) when disabled. A typed slot shows its type's; a typeless
    /// one its option's.
    pub fn picture(&self, s: &Slot) -> Option<(&render::tga::Image, Crop)> {
        let at = |base: i32| usize::try_from(s.ty - base).ok();
        let cell = if s.ty < 0 {
            self.options.get(usize::try_from(s.option).ok()?)?.as_ref()
        } else if s.ty >= BASE_TECHTYPES {
            self.techs.get(at(BASE_TECHTYPES)?)?.1.as_ref()
        } else if s.ty >= BASE_BUILDTYPES {
            self.builds.get(at(BASE_BUILDTYPES)?)?.as_ref()
        } else {
            self.units.get(at(BASE_UNITTYPES)?)?.1.as_ref()
        }?;
        let (id, col, row) = cell;
        let page = self.pages.get(id)?;
        let (l, t, w, h) = self.base;
        let off = usize::from(s.disable != 0);
        let x = (l + (w + 1) * *col as usize).checked_sub(off)?;
        let y = (t + (h + 1) * *row as usize).checked_sub(off)?;
        (x + w <= page.width && y + h <= page.height).then_some((page, (x, y, w, h)))
    }

    /// The overlay frame of a slot, enabled or disabled.
    pub fn overlay(&self, disabled: bool) -> Option<(&render::tga::Image, Crop)> {
        Some((
            self.pages.get("TEX_COMMON")?,
            self.overlays[usize::from(disabled)],
        ))
    }
}

/// `c·k / 255`, rounded half up.
fn scale(c: u8, k: u32) -> u8 {
    ((u32::from(c) * k + 127) / 255) as u8
}

/// **The slots drawn** into `win`, the composed left window: each slot's
/// picture 1:1 at its button's corner (`spans`, row-major, `cols` wide),
/// at colour `0xff` or `0x80` when disabled, then the overlay at `0x80`
/// or `0x40`.
pub fn draw(
    win: &mut crate::selection::Texels,
    spans: &[super::left::Span],
    cols: i32,
    art: &SlotArt,
    slots: &[Slot],
) {
    for s in slots {
        let Some(&(x0, y0, x1, y1)) = usize::try_from(s.grid.1 * cols + s.grid.0)
            .ok()
            .and_then(|i| spans.get(i))
        else {
            continue;
        };
        let disabled = s.disable != 0;
        let (kp, ko) = if disabled { (0x80, 0x40) } else { (0xff, 0x80) };
        for (layer, k) in [(art.picture(s), kp), (art.overlay(disabled), ko)] {
            let Some((page, (l, t, w, h))) = layer else {
                continue;
            };
            for v in 0..h.min((y1 - y0).max(0) as usize) {
                for u in 0..w.min((x1 - x0).max(0) as usize) {
                    let (x, y) = (x0 + u as i32, y0 + v as i32);
                    if !(0..win.w as i32).contains(&x) || !(0..win.h as i32).contains(&y) {
                        continue;
                    }
                    let i = ((t + v) * page.width + l + u) * 4;
                    let c = &page.rgba[i..i + 4];
                    let src = [scale(c[0], k), scale(c[1], k), scale(c[2], k), c[3]];
                    let j = (y as usize * win.w + x as usize) * 4;
                    super::left::over(&mut win.rgba[j..j + 4], src);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The first add of a cell keeps it; an add off the grid is dropped.
    #[test]
    fn the_first_add_takes_the_cell() {
        let s = |option, grid| Slot {
            option,
            ty: -1,
            disable: 0,
            grid,
        };
        let adds = [
            s(1, (3, 0)),
            s(2, (0, 3)),
            s(3, (0, 3)),
            s(4, (5, 0)),
            s(5, (-1, -1)),
        ];
        assert_eq!(place(&adds, (5, 4)), vec![s(1, (3, 0)), s(2, (0, 3))]);
    }

    /// Records split on the tag, not its longer kin, comments out.
    #[test]
    fn records_are_positional() {
        let xml = "<OPTIONS><!-- <OPTION a=\"x\"> --><OPTION a=\"0\"><NAME>A</NAME></OPTION>\n<OPTION a=\"1\"/></OPTIONS>";
        let r = records(xml, "OPTION");
        assert_eq!(r.len(), 2);
        assert!(r[0].starts_with(" a=\"0\""));
        assert_eq!(tag_text(&r[0], "NAME"), Some("A"));
        assert_eq!(
            cell_of(Some("T "), Some("7"), Some("6")),
            Some(("T".into(), 7, 6))
        );
        assert_eq!(cell_of(Some("T"), Some("-1"), Some("-1")), None);
    }

    /// Half and full colour, rounded: the disabled frame's 123 and 213 at
    /// `0x40` are 31 and 53 (item 4729).
    #[test]
    fn the_colour_rounds() {
        assert_eq!((scale(123, 0x40), scale(213, 0x40)), (31, 53));
        assert_eq!(scale(200, 0xff), 200);
        assert_eq!(scale(255, 0x80), 128);
    }
}
