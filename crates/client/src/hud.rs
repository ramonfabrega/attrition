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
//! - **The bar** (item 4198, `docs/HUD.md` §4) prints the issuing slot's
//!   leader record from the scene's [`sim::observe::LeaderScene`]: each
//!   good's stockpile and its rate, the population over its cap, the age,
//!   the nation and the frame. The numbers are the ones the original's
//!   resource bar and population box read (`bucket`, `income`, `control`,
//!   `pop_cap`), at the record's own scale; the rate is shown as the
//!   original shows it, `(income + 8) / 16`, and written as it writes it
//!   (item 4203): `0` with no sign, otherwise a `+` before the number, so
//!   a falling rate reads `+-3` ([`rate_text`]).
//! - **The clock** (item 4203, `docs/HUD.md` §4): the original's
//!   `Game::tick`, one more each fifteenth frame, so `frame / 15` seconds,
//!   written `hh:mm:ss` as `say_timer@007f1b00` writes it ([`clock`]).
//! - **The ring** (item 4203, `docs/HUD.md` §3): each selected unit's
//!   figures circled on the ground at the original's radius, a selected
//!   building's footprint outlined, in the owner's colour ([`rings`]).
//!
//! - **The entry line** (item 4250, `docs/CLIENT_INPUT.md` §12): Return
//!   opens it over the menu's row, it shows what is typed after `SAY:`,
//!   and Return sends it, as the original's chat box does
//!   ([`crate::input::Input::chat`]); Escape closes it. Drawn only while
//!   open ([`line_rect`]).
//! - **The pause notice** (item 4252, `docs/CLIENT_INPUT.md` §13):
//!   `PAUSED`, centred at the top while the slot's pause stands
//!   ([`pause_rect`]).
//!
//! Where the buttons stand is ours, anchored to the window's bottom-left
//! corner, and pinned as pixels at the scripts' 1920×1080 by the tests.
//! The HUD is drawn in a window, and under `--still` only with `--select`;
//! never on a row's still ([`crate::still_of`] draws no overlay).

use crate::input::{BUILD_KEYS, Event, Input, TRAIN_KEYS, View};
use crate::overlay::{self, Painter};
use rondata::recgame::Order;
use sim::observe::{LeaderScene, Scene};

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

/// The six goods, in the leader record's order (`LeaderDataEncrypt`'s
/// per-good arrays; [`sim::economy::Resource`]).
pub const GOODS: [&str; 6] = ["FOOD", "TIMBER", "WEALTH", "KNOWLEDGE", "METAL", "OIL"];

/// The install's names, as the interface shows them (`NAME`): one a unit
/// type, one a building type.
#[derive(Clone, Debug, Default)]
pub struct Names {
    pub units: Vec<String>,
    pub builds: Vec<String>,
    pub techs: Vec<String>,
    /// Each unit type's selection ring radius, in position units:
    /// `CIRCLE_RADIUS` (`ObjectTypeData +0x234` on a unit type) times
    /// `unit_block_radius`, as `UnitOut::draw_underlays@00614100` passes
    /// it to `draw_unit_circles@00606a10` (item 4203).
    pub rings: Vec<i32>,
    /// Each building type's footprint, in tiles (`+0x234`, `+0x238`).
    pub footprints: Vec<(i32, i32)>,
    /// The panel's roster (item 4290), which the window hands the layer.
    pub roster: std::sync::Arc<crate::input::Roster>,
}

impl Names {
    pub fn new(loaded: &rondata::load::Loaded) -> Names {
        Names {
            units: loaded.unit_names.clone(),
            builds: loaded.build_names.clone(),
            techs: loaded.tech_names.clone(),
            rings: loaded
                .unit_types
                .iter()
                .map(|t| t.combat.circle_radius * UNIT_BLOCK_RADIUS)
                .collect(),
            footprints: loaded
                .build_types
                .iter()
                .map(|b| (b.x_size, b.y_size))
                .collect(),
            roster: std::sync::Arc::new(crate::input::Roster::new(loaded)),
        }
    }

    /// The name of the type a `TypeIndex` names: a unit's from
    /// `BASE_UNITTYPES` (0x32, the Citizen first), a building's from
    /// `BASE_BUILDTYPES` (`rondata::load::Loaded::type_index`), a
    /// technology's from `BASE_TECHTYPES` (0x220; The Art of War, the
    /// attacked script's research, item 4195).
    pub fn of_type_index(&self, ti: i32) -> String {
        use rondata::load::{BASE_BUILDTYPES, BASE_TECHTYPES, BASE_UNITTYPES};
        let at = |table: &[String], base: i32| {
            usize::try_from(ti - base)
                .ok()
                .and_then(|i| table.get(i).cloned())
        };
        let name = if ti >= BASE_TECHTYPES {
            at(&self.techs, BASE_TECHTYPES)
        } else if ti >= BASE_BUILDTYPES {
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

/// The ink of a greyed button of the roster (item 4334): ours.
pub const GREYED: [f32; 3] = [0.42, 0.44, 0.48];

/// The size of one of the roster's buttons (item 4290).
pub const ROSTER_BUTTON: (i32, i32) = (150, 24);
/// The roster's buttons a row: six, so the rows end left of the
/// right-hand buttons' columns (at 1920, `x` 932 against their 980).
pub const ROSTER_ROW: i32 = 6;

/// One of the roster's buttons (item 4290, `docs/CLIENT_INPUT.md` §32):
/// the event a press sends ([`crate::input::Event::Train`] or
/// [`crate::input::Event::Place`]), the button as the original's panel
/// adds it (type, option, `disable`; item 4334), and where it stands. A
/// button with `disable` set is drawn greyed and a press on it orders
/// nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RosterButton {
    pub ev: Event,
    pub button: crate::input::Button,
    pub rect: Rect,
}

impl RosterButton {
    /// The `TypeIndex` it queues or chooses.
    pub fn ty(&self) -> i32 {
        self.button.ty
    }

    /// Greyed: the original's `disable` set.
    pub fn greyed(&self) -> bool {
        self.button.disable != 0
    }
}

/// The label of one of the citizen's page buttons (item 4339, §32.7), a
/// button with no type: ours, the original draws its icons.
pub fn page_label(option: i32) -> Option<&'static str> {
    use crate::input::{
        OPTION_BACK, OPTION_BUILD, OPTION_BUILD_MILITARY, OPTION_BUILD_WONDER, OPTION_CANCEL,
        OPTION_NEXT,
    };
    Some(match option {
        OPTION_BUILD => "BUILD",
        OPTION_BUILD_MILITARY => "MILITARY",
        OPTION_BUILD_WONDER => "WONDERS",
        OPTION_NEXT => "NEXT",
        OPTION_BACK => "BACK",
        OPTION_CANCEL => "CANCEL",
        _ => return None,
    })
}

/// **The roster's rows** (item 4290): a button for each of
/// [`Input::panel`]'s, [`ROSTER_ROW`] a row from the left, the rows
/// stacked upward from above the menu's. Where they stand is ours. With
/// `sim`, a building's buttons are the original's panel (item 4334).
pub fn roster_menu(
    input: &Input,
    scene: &Scene,
    sim: Option<&sim::Sim>,
    size: (i32, i32),
) -> Vec<RosterButton> {
    let (w, h) = ROSTER_BUTTON;
    let base = size.1 - 2 * MARGIN - BUTTON.1;
    input
        .panel(scene, sim)
        .into_iter()
        .enumerate()
        .map(|(k, (ev, button))| {
            let (col, row) = (k as i32 % ROSTER_ROW, k as i32 / ROSTER_ROW);
            RosterButton {
                ev,
                button,
                rect: Rect {
                    x: MARGIN + col * (w + MARGIN / 2),
                    y: base - (row + 1) * (h + MARGIN / 2),
                    w,
                    h,
                },
            }
        })
        .collect()
}

/// The size of one of the market's buttons: narrower than the menu's, so
/// four goods' eight fit beside it at 1920.
pub const MARKET_BUTTON: (i32, i32) = (180, 36);

/// One of the market's buttons (item 4259, `docs/CLIENT_INPUT.md` §16):
/// the good, whether it buys, the price it shows, whether it is live
/// ([`crate::input::trades`]) and where it stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MarketButton {
    pub good: usize,
    pub buy: bool,
    pub price: i32,
    pub live: bool,
    pub rect: Rect,
}

impl MarketButton {
    /// The event a press on it sends.
    pub fn event(&self) -> Event {
        if self.buy {
            Event::Buy(self.good)
        } else {
            Event::Sell(self.good)
        }
    }

    /// Its label: the verb, the good, the price.
    pub fn label(&self) -> String {
        let verb = if self.buy { "BUY" } else { "SELL" };
        format!("{verb} {} {}", GOODS[self.good], self.price)
    }
}

/// **The market's row**: with the slot's own finished market the
/// selection, a sell and a buy button for each good its panel offers
/// (`BuildOut::make_options@00629ed0`'s `OPTION_SELL`, `OPTION_BUY`), in
/// the goods' order; otherwise none. One row along the bottom edge,
/// anchored to the right corner. Where it stands is ours.
pub fn market_menu(input: &Input, scene: &Scene, size: (i32, i32)) -> Vec<MarketButton> {
    let Some(offer) = crate::input::market(scene, input.player, &input.selection) else {
        return Vec::new();
    };
    let rows: Vec<(usize, bool, i32)> = offer
        .iter()
        .enumerate()
        .filter_map(|(g, o)| o.map(|(buy, sell)| [(g, false, sell), (g, true, buy)]))
        .flatten()
        .collect();
    let (w, h) = MARKET_BUTTON;
    let n = rows.len() as i32;
    rows.into_iter()
        .enumerate()
        .map(|(k, (good, buy, price))| MarketButton {
            good,
            buy,
            price,
            live: crate::input::trades(scene, input.player, &input.selection, good, buy),
            rect: Rect {
                x: size.0 - (n - k as i32) * (w + MARGIN),
                y: size.1 - MARGIN - h,
                w,
                h,
            },
        })
        .collect()
}

/// One of the city's gather buttons (item 4261, `docs/CLIENT_INPUT.md`
/// §17): the type it gathers to and where it stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CityButton {
    pub t: i32,
    pub rect: Rect,
}

impl CityButton {
    /// Its label: the gatherer it sends to.
    pub fn label(&self) -> &'static str {
        match self.t {
            0x1a1 => "GATHER FARM",
            0x1a2 => "GATHER WOOD",
            0x1a3 => "GATHER MINE",
            _ => "GATHER ALL",
        }
    }
}

/// **The city's row**: with the slot's own city centre the selection, its
/// panel's four gather buttons (`BuildOut::make_options@00629ed0`'s
/// `OPTION_CITY_FARM_GATHER`, `_WOODCUTTER_`, `_MINE_`, `_ALL_`), in that
/// order; otherwise none. One row along the bottom edge, anchored to the
/// right corner, as the market's. Where it stands is ours.
pub fn city_menu(input: &Input, scene: &Scene, size: (i32, i32)) -> Vec<CityButton> {
    if !crate::input::city_panel(scene, input.player, &input.selection) {
        return Vec::new();
    }
    let (w, h) = MARKET_BUTTON;
    let n = crate::input::CITY_GATHER_TYPES.len() as i32;
    crate::input::CITY_GATHER_TYPES
        .iter()
        .enumerate()
        .map(|(k, &t)| CityButton {
            t,
            rect: Rect {
                x: size.0 - (n - k as i32) * (w + MARGIN),
                y: size.1 - MARGIN - h,
                w,
                h,
            },
        })
        .collect()
}

/// One of a building's queue buttons (item 4265, `docs/CLIENT_INPUT.md`
/// §19): the queue slot it stands for and where it stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QueueButton {
    pub slot: i32,
    pub rect: Rect,
}

impl QueueButton {
    /// Its label: the slot, counted from one as a person reads a queue.
    pub fn label(&self) -> String {
        format!("CANCEL {}", self.slot + 1)
    }
}

/// **The queue's row**: with one finished building of the slot's own the
/// selection, a button per live entry of its production queue
/// (`IFaceSelected::on_button_clicked@00826e00`'s `OPTION_QUEUE`, by
/// slot), in queue order; otherwise none. One row above the bottom edge,
/// so the city's row keeps its place. Where it stands, and its labels,
/// are ours.
pub fn queue_menu(input: &Input, scene: &Scene, size: (i32, i32)) -> Vec<QueueButton> {
    if !input.building {
        return Vec::new();
    }
    let n = crate::input::queue_panel(scene, input.player, &input.selection) as i32;
    let (w, h) = MARKET_BUTTON;
    (0..n)
        .map(|k| QueueButton {
            slot: k,
            rect: Rect {
                x: size.0 - (n - k) * (w + MARGIN),
                y: size.1 - 2 * (MARGIN + h),
                w,
                h,
            },
        })
        .collect()
}

/// **The city's name button** (item 4268, `docs/CLIENT_INPUT.md` §21):
/// with a selection whose first object is a finished building of the
/// slot's own in a city ([`crate::input::renames`]), one button, RENAME,
/// that opens the name for editing (`Options::exec@007188c0`'s option
/// `0x117`); otherwise none. Three rows above the bottom edge, at the
/// right. Where it stands, and its label, are ours: the original opens it
/// from the city's name on the selected panel.
pub fn rename_button(input: &Input, scene: &Scene, size: (i32, i32)) -> Option<Rect> {
    if !input.building || input.line.is_some() {
        return None;
    }
    crate::input::renames(scene, input.player, &input.selection)?;
    let (w, h) = MARKET_BUTTON;
    Some(Rect {
        x: size.0 - (w + MARGIN),
        y: size.1 - 3 * (MARGIN + h),
        w,
        h,
    })
}

/// **The unit's garrison button** (item 4273, `docs/CLIENT_INPUT.md`
/// §23): with a selection of units one of whose members may enter a
/// building of its owner's ([`sim::observe::UnitScene::enters`]), one
/// button, GARRISON (`OPTION_GARRISON`, `Options::exec@007188c0`'s
/// `do_garrison`); otherwise none. Two rows above the bottom edge, at the
/// right. Where it stands, its label, and when it shows are ours: the
/// original's `UnitOut::make_options@00615fc0` offers it by the unit's
/// type, a predicate not read.
pub fn garrison_button(input: &Input, scene: &Scene, size: (i32, i32)) -> Option<Rect> {
    if input.building || input.line.is_some() {
        return None;
    }
    input
        .selection
        .iter()
        .filter_map(|id| scene.units.iter().find(|u| u.id == *id))
        .any(|u| u.owner == input.player && !u.enters.is_empty())
        .then_some(())?;
    let (w, h) = MARKET_BUTTON;
    Some(Rect {
        x: size.0 - (w + MARGIN),
        y: size.1 - 2 * (MARGIN + h),
        w,
        h,
    })
}

/// **The building's infinite-queue button** (item 4276,
/// `docs/CLIENT_INPUT.md` §24): with a selection of buildings one of which
/// is a training building of the slot's own
/// ([`sim::observe::BuildingScene::trains`]), one button, INFINITE
/// (`OPTION_INFINITE`, `IFaceSelected::on_button_clicked@00826e00`);
/// otherwise none. Three rows above the bottom edge, left of RENAME. Where
/// it stands, its label, and when it shows are ours: which of the
/// original's panel buttons send it is read only as far as their names.
pub fn infinite_button(input: &Input, scene: &Scene, size: (i32, i32)) -> Option<Rect> {
    if !input.building || input.line.is_some() {
        return None;
    }
    input
        .selection
        .iter()
        .filter_map(|id| scene.buildings.iter().find(|b| b.id == *id))
        .any(|b| b.owner == input.player && b.trains)
        .then_some(())?;
    let (w, h) = MARKET_BUTTON;
    Some(Rect {
        x: size.0 - 2 * (w + MARGIN),
        y: size.1 - 3 * (MARGIN + h),
        w,
        h,
    })
}

/// **The stance button** (item 4282, `docs/CLIENT_INPUT.md` §27): with a
/// selection of the slot's own, units, or buildings one of which is a
/// training building ([`sim::observe::BuildingScene::trains`]), one
/// button, STANCE, three rows above the bottom edge, third from the right;
/// otherwise none. A press is [`Event::Stance`], `Options::do_stance(-1)`.
/// Where it stands, its label, and when it shows are ours.
pub fn stance_button(input: &Input, scene: &Scene, size: (i32, i32)) -> Option<Rect> {
    if input.selection.is_empty() || input.line.is_some() {
        return None;
    }
    let own = if input.building {
        input
            .selection
            .iter()
            .filter_map(|id| scene.buildings.iter().find(|b| b.id == *id))
            .any(|b| b.owner == input.player && b.trains)
    } else {
        input.selection.iter().any(|id| id.who == input.player)
    };
    own.then_some(())?;
    let (w, h) = MARKET_BUTTON;
    Some(Rect {
        x: size.0 - 3 * (w + MARGIN),
        y: size.1 - 3 * (MARGIN + h),
        w,
        h,
    })
}

/// **The disband button** (item 4287, `docs/CLIENT_INPUT.md` §29): with a
/// selection one of whose members is the slot's own, unit or building, one
/// button, DISBAND, three rows above the bottom edge, fourth from the
/// right; otherwise none. A press is [`Event::Disband`],
/// `Options::do_disband(0)`. Where it stands, its label, and when it shows
/// are ours: the original's `UnitOut::make_options@00615fc0` offers
/// `OPTION_DISBAND` for a unit only under a profile preference, and
/// `BuildOut::make_options@00629ed0` for a building under predicates not
/// read; the `OPTION_DISBAND` key is the other actor.
pub fn disband_button(input: &Input, scene: &Scene, size: (i32, i32)) -> Option<Rect> {
    if input.selection.is_empty() || input.line.is_some() {
        return None;
    }
    let own = if input.building {
        input
            .selection
            .iter()
            .filter_map(|id| scene.buildings.iter().find(|b| b.id == *id))
            .any(|b| b.owner == input.player)
    } else {
        input
            .selection
            .iter()
            .filter_map(|id| scene.units.iter().find(|u| u.id == *id))
            .any(|u| u.owner == input.player)
    };
    own.then_some(())?;
    let (w, h) = MARKET_BUTTON;
    Some(Rect {
        x: size.0 - 4 * (w + MARGIN),
        y: size.1 - 3 * (MARGIN + h),
        w,
        h,
    })
}

/// **The auto-explore button** (item 4288, `docs/CLIENT_INPUT.md` §31):
/// with a unit selection one of whose members is the slot's own, one
/// button, EXPLORE, three rows above the bottom edge, fifth from the
/// right; otherwise none. A press is [`Event::AutoExplore`],
/// `Options::exec@007188c0` case `0x21`, `Options::do_auto_explore@0071cbf0`
/// with the option's object, −1. Where it stands, its label, and when it
/// shows are ours: the original's `UnitOut::make_options@00615fc0` offers
/// `OPTION_AUTO_EXPLORE` for a unit that is not a plane (bar the type's
/// `0x20`) and, without the profile's `prefs & 2`, not a caravan; the
/// key map's `OPTION_AUTO_EXPLORE` is the other actor.
pub fn auto_explore_button(input: &Input, scene: &Scene, size: (i32, i32)) -> Option<Rect> {
    if input.selection.is_empty() || input.line.is_some() || input.building {
        return None;
    }
    input
        .selection
        .iter()
        .filter_map(|id| scene.units.iter().find(|u| u.id == *id))
        .any(|u| u.owner == input.player)
        .then_some(())?;
    let (w, h) = MARKET_BUTTON;
    Some(Rect {
        x: size.0 - 5 * (w + MARGIN),
        y: size.1 - 3 * (MARGIN + h),
        w,
        h,
    })
}

/// **The building panel's garrisoned units** (item 4280,
/// `docs/CLIENT_INPUT.md` §26): with a building selection, one icon per
/// unit of the slot's own inside a selected building of its own
/// ([`crate::input::garrisoned`]), right to left from the right edge, four
/// rows above the bottom; otherwise none. A press on the `k`th is
/// [`Event::ComeOut`]`(k)`. Where the icons stand and their labels are
/// ours.
pub fn come_out_menu(input: &Input, scene: &Scene, size: (i32, i32)) -> Vec<(usize, Rect)> {
    if !input.building || input.line.is_some() {
        return Vec::new();
    }
    let n = crate::input::garrisoned(scene, input.player, &input.selection).len();
    let (w, h) = MARKET_BUTTON;
    (0..n)
        .map(|k| {
            (
                k,
                Rect {
                    x: size.0 - (k as i32 + 1) * (w + MARGIN),
                    y: size.1 - 4 * (MARGIN + h),
                    w,
                    h,
                },
            )
        })
        .collect()
}

/// One of a unit's craft buttons (item 4264, `docs/CLIENT_INPUT.md` §18):
/// the craft's row and where it stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpellButton {
    pub t: i32,
    pub rect: Rect,
}

impl SpellButton {
    /// Its label: the craft's name as the crate names it, upper case.
    pub fn label(&self) -> String {
        sim::ids::spell::NAMED_SPELLS
            .iter()
            .find(|(_, t)| *t == self.t)
            .map_or_else(|| format!("CRAFT {}", self.t), |(n, _)| n.to_uppercase())
    }
}

/// **The unit's craft row**: with a unit selection, one button per craft a
/// member offers ([`sim::observe::UnitScene::crafts`]), in row order, each
/// once; otherwise none. The row stands where the market's does. Where it
/// stands, and that it is one row for the whole selection, is ours.
pub fn spell_menu(input: &Input, scene: &Scene, size: (i32, i32)) -> Vec<SpellButton> {
    if input.building {
        return Vec::new();
    }
    let mut crafts: Vec<i32> = input
        .selection
        .iter()
        .filter_map(|id| scene.units.iter().find(|u| u.id == *id))
        .flat_map(|u| u.crafts.iter().map(|c| c.t))
        .collect();
    crafts.sort_unstable();
    crafts.dedup();
    let (w, h) = MARKET_BUTTON;
    let n = crafts.len() as i32;
    crafts
        .into_iter()
        .enumerate()
        .map(|(k, t)| SpellButton {
            t,
            rect: Rect {
                x: size.0 - (n - k as i32) * (w + MARGIN),
                y: size.1 - MARGIN - h,
                w,
                h,
            },
        })
        .collect()
}

/// One event through the HUD and then the layer: the window's own path
/// (`App::input`), and the one the tests mint through. A press on one of
/// the market's or the city's buttons is that button's event; anything
/// else goes through [`route`].
pub fn event(
    input: &mut Input,
    size: (i32, i32),
    frame: i32,
    ev: Event,
    view: &View,
) -> Option<Order> {
    let market = match ev {
        Event::Press { x, y } => market_menu(input, view.scene, size)
            .into_iter()
            .find(|b| b.rect.contains(x, y))
            .map(|b| b.event())
            .or_else(|| {
                city_menu(input, view.scene, size)
                    .into_iter()
                    .find(|b| b.rect.contains(x, y))
                    .map(|b| Event::CityGather(b.t))
            })
            .or_else(|| {
                spell_menu(input, view.scene, size)
                    .into_iter()
                    .find(|b| b.rect.contains(x, y))
                    .map(|b| Event::Spell(b.t))
            })
            .or_else(|| {
                queue_menu(input, view.scene, size)
                    .into_iter()
                    .find(|b| b.rect.contains(x, y))
                    .map(|b| Event::Unqueue(b.slot))
            })
            .or_else(|| {
                rename_button(input, view.scene, size)
                    .filter(|r| r.contains(x, y))
                    .map(|_| Event::Rename)
            })
            .or_else(|| {
                garrison_button(input, view.scene, size)
                    .filter(|r| r.contains(x, y))
                    .map(|_| Event::Garrison)
            })
            .or_else(|| {
                infinite_button(input, view.scene, size)
                    .filter(|r| r.contains(x, y))
                    .map(|_| Event::Infinite)
            })
            .or_else(|| {
                stance_button(input, view.scene, size)
                    .filter(|r| r.contains(x, y))
                    .map(|_| Event::Stance)
            })
            .or_else(|| {
                disband_button(input, view.scene, size)
                    .filter(|r| r.contains(x, y))
                    .map(|_| Event::Disband)
            })
            .or_else(|| {
                auto_explore_button(input, view.scene, size)
                    .filter(|r| r.contains(x, y))
                    .map(|_| Event::AutoExplore)
            })
            .or_else(|| {
                come_out_menu(input, view.scene, size)
                    .into_iter()
                    .find(|(_, r)| r.contains(x, y))
                    .map(|(k, _)| Event::ComeOut(k))
            })
            .or_else(|| {
                roster_menu(input, view.scene, view.sim, size)
                    .into_iter()
                    .find(|b| b.rect.contains(x, y) && !b.greyed())
                    .map(|b| b.ev)
            }),
        _ => None,
    };
    // A press on a greyed button of the roster orders nothing (item 4334):
    // the clear ones were taken above.
    if let (None, Event::Press { x, y }) = (market, ev)
        && roster_menu(input, view.scene, view.sim, size)
            .iter()
            .any(|b| b.rect.contains(x, y))
    {
        return None;
    }
    let ev = market.unwrap_or_else(|| route(input, size, ev));
    input.event(frame, ev, view)
}

/// **One turn through the HUD** (item 4302, `docs/CLIENT_INPUT.md` §36):
/// each of the turn's events through [`event`], in order, and what they
/// minted as one package ([`crate::input::one_package`]).
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "the window writes no file; the tests mint turns")
)]
pub fn turn(
    input: &mut Input,
    size: (i32, i32),
    frame: i32,
    events: &[Event],
    view: &View,
) -> Option<Order> {
    let orders: Vec<Order> = events
        .iter()
        .filter_map(|&ev| event(input, size, frame, ev, view))
        .collect();
    crate::input::one_package(orders)
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

/// **The bar**: one player's leader record as the HUD prints it, read
/// from the scene alone (`docs/HUD.md` §4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bar {
    /// Each good's stockpile (`bucket`) and its rate as the original's bar
    /// writes it, `(income + 8) / 16`; and whether it is at its cap.
    pub goods: [(i32, i32, bool); 6],
    /// `control` of `pop_cap`.
    pub pop: (i32, i32),
    /// The age a person counts, the first being 1: `ages + 1`.
    pub age: i32,
    /// The nation's name.
    pub nation: String,
    /// The frame the scene is at; the clock is read from it ([`clock`]).
    pub frame: i64,
}

/// The rate the original's bar writes for an `income` in sixteenths:
/// `IFaceResources::draw_resource@00818840` adds 8 and divides by 16,
/// rounding toward zero.
pub fn shown_rate(income: i32) -> i32 {
    (income + 8) / 16
}

/// **A shown rate as the original writes it** (item 4203).
/// `IFaceResources::draw_resource@00818840` writes `internal_strings.xml`
/// entry 404 (`int_str_array +0x1f90`, the string `0`) for a zero
/// `income`, the shown rate's digits otherwise, and then puts a `+` in
/// front **unless the string is that `0`**. So a rate that rounds to zero
/// reads `0` with no sign, a rising one `+40`, and a falling one `+-3`:
/// the sign is not chosen by the value, only withheld from zero.
pub fn rate_text(shown: i32) -> String {
    if shown == 0 {
        "0".into()
    } else {
        format!("+{shown}")
    }
}

/// `UNIT_BLOCK_RADIUS` as loaded: one `UCoord`, 48 position units
/// (`rondata::load`'s).
pub const UNIT_BLOCK_RADIUS: i32 = 48;

/// Frames to one second of the game's clock: `Game::do_frame@00591ef0`
/// raises `Game::tick` (`+0x560`) by one after the frame count's own
/// step whenever the count is a multiple of 15, and `Setup::build_game`
/// and `Game::clear` set it to 0. No other writer.
pub const FRAMES_PER_TICK: i64 = 15;

/// **The game's clock in seconds**, `Game::tick` on frame `frame`: the
/// frames counted to it, a fifteenth each. 109 of 109 packets on disk hold
/// exactly this (`docs/HUD.md` §5).
pub fn clock_seconds(frame: i64) -> i64 {
    frame / FRAMES_PER_TICK
}

/// **The clock as the original writes it**, `say_timer@007f1b00` on
/// seconds: hours, minutes and seconds, each through `say_timer_num`
/// (`0` put in front of a number under 10), joined by `:`. The bar's own
/// line writes it after the speed's name and `: ` (`Normal: 00:03:38` on
/// run1559b's photograph); the speed is not ours to name.
pub fn clock(frame: i64) -> String {
    let s = clock_seconds(frame);
    format!("{:02}:{:02}:{:02}", s / 3600, s % 3600 / 60, s % 60)
}

impl Bar {
    /// The bar for `who`'s record in `scene`, `None` when the scene holds
    /// no such slot.
    pub fn of(scene: &Scene, who: sim::Player) -> Option<Bar> {
        let l: &LeaderScene = scene.leaders.iter().find(|l| l.who == who)?;
        Some(Bar {
            goods: std::array::from_fn(|g| (l.bucket[g], shown_rate(l.income[g]), l.over_cap[g])),
            pop: (l.control, l.pop_cap),
            age: l.ages + 1,
            nation: if l.nation.is_empty() {
                format!("TRIBE {}", l.tribe)
            } else {
                l.nation.clone()
            },
            frame: scene.frame,
        })
    }

    /// The printed text: `FOOD 200 0 … POP 6/25 AGE 1 NATION Greeks
    /// FRAME 7000 TIME 00:07:46`, each number after its word, each rate as
    /// [`rate_text`] writes it. A good at its cap is marked `CAP`.
    pub fn text(&self) -> String {
        let mut s = String::new();
        for (name, (stock, rate, capped)) in GOODS.iter().zip(self.goods) {
            s += &format!("{name} {stock} {} ", rate_text(rate));
            if capped {
                s += "CAP ";
            }
        }
        s += &format!(
            "POP {}/{} AGE {} NATION {} FRAME {} TIME {}",
            self.pop.0,
            self.pop.1,
            self.age,
            self.nation,
            self.frame,
            clock(self.frame)
        );
        s
    }
}

/// Where the bar stands for its text: along the window's top edge, at the
/// right, clear of the debug overlay's lines at the top-left (seen on the
/// still: at the left it covered the overlay's frame line).
pub fn bar_rect(text: &str, size: (i32, i32)) -> Rect {
    let w = text.chars().count() as i32 * overlay::advance(SCALE) + 4 * SCALE;
    Rect {
        x: (size.0 - MARGIN - w).max(MARGIN),
        y: MARGIN,
        w,
        h: overlay::line_height(SCALE) + 4 * SCALE,
    }
}

/// **A selection's ring**: a closed line on the ground, in world
/// coordinates, and the slot whose colour it takes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ring {
    pub owner: sim::Player,
    pub points: Vec<(i32, i32, i32)>,
}

/// The points a ring's circle is drawn with.
pub const RING_POINTS: usize = 32;

/// **The selection's rings** (item 4203; `docs/HUD.md` §3), in world
/// coordinates, from the scene alone. A unit: one circle each figure, at
/// the figure's place, its radius the type's
/// ([`Names::rings`]; `UnitOut::draw_underlays@00614100` hands
/// `draw_unit_circles@00606a10` `unit_block_radius × +0x234`, and that
/// draws one circle each figure in the owner's `TeamColor::get_iface`).
/// A building: its footprint's rectangle about its place, **ours**: the
/// original draws `draw_funky_rectangle_circle` at `BuildData::max_range`
/// (vtable `+0x130`), not ported. An object no longer in the scene draws
/// nothing; a type with no radius draws no circle.
pub fn rings(input: &Input, scene: &Scene, names: &Names) -> Vec<Ring> {
    let mut out = Vec::new();
    for id in &input.selection {
        if let Some(u) = scene.units.iter().find(|u| u.id == *id) {
            let r = u.ty.and_then(|t| names.rings.get(t)).copied().unwrap_or(0);
            if r <= 0 {
                continue;
            }
            let places: Vec<(i32, i32, i32)> = if u.figures.is_empty() {
                vec![(u.pos.x, u.pos.y, u.z)]
            } else {
                u.figures.iter().map(|f| (f.pos.x, f.pos.y, f.z)).collect()
            };
            for (x, y, z) in places {
                let points = (0..RING_POINTS)
                    .map(|k| {
                        let a = std::f64::consts::TAU * k as f64 / RING_POINTS as f64;
                        let dx = (f64::from(r) * a.cos()).round() as i32;
                        let dy = (f64::from(r) * a.sin()).round() as i32;
                        (x + dx, y + dy, z)
                    })
                    .collect();
                out.push(Ring {
                    owner: u.owner,
                    points,
                });
            }
        } else if let Some(b) = scene.buildings.iter().find(|b| b.id == *id) {
            let (w, h) =
                b.ty.and_then(|t| names.footprints.get(t))
                    .copied()
                    .unwrap_or((1, 1));
            let (hx, hy) = (
                w * sim::ids::scale::UNITS_PER_TILE / 2,
                h * sim::ids::scale::UNITS_PER_TILE / 2,
            );
            let (x, y, z) = (b.pos.x, b.pos.y, b.z);
            out.push(Ring {
                owner: b.owner,
                points: vec![
                    (x - hx, y - hy, z),
                    (x + hx, y - hy, z),
                    (x + hx, y + hy, z),
                    (x - hx, y + hy, z),
                ],
            });
        }
    }
    out
}

/// The console's line as drawn (item 4283): `>`, the prompt the original's
/// console echoes a command with, what it holds, and a cursor.
pub fn console_text(line: &str) -> String {
    format!(">{line}_")
}

/// The entry line's text as drawn: `SAY: `, what it holds, and a cursor.
pub fn line_text(line: &str) -> String {
    format!("SAY: {line}_")
}

/// **Where the entry line stands** on a `size` window: one row above the
/// menu's, from the left margin, as wide as the panel at least and as its
/// text needs.
pub fn line_rect(text: &str, size: (i32, i32)) -> Rect {
    let w = (text.chars().count() as i32 * overlay::advance(SCALE) + 4 * SCALE).max(PANEL_WIDTH);
    let h = overlay::line_height(SCALE) + 4 * SCALE;
    Rect {
        x: MARGIN,
        y: size.1 - 2 * MARGIN - BUTTON.1 - h,
        w,
        h,
    }
}

/// The pause notice's text (ours: `IFaceMainBase::do_pause_notice`'s own
/// is not read; item 4252).
pub const PAUSE_TEXT: &str = "PAUSED";

/// **Where the pause notice stands** on a `size` window: centred, one
/// margin below the top.
pub fn pause_rect(size: (i32, i32)) -> Rect {
    let w = PAUSE_TEXT.chars().count() as i32 * overlay::advance(SCALE) + 4 * SCALE;
    let h = overlay::line_height(SCALE) + 4 * SCALE;
    Rect {
        x: (size.0 - w) / 2,
        y: MARGIN,
        w,
        h,
    }
}

/// The button's label: its key, then the type's name.
pub fn label(b: &Button, names: &Names) -> String {
    format!("{} {}", b.key, names.of_type_index(b.ty))
}

/// **The HUD drawn**: the selection's rings, then the bar, the panel and
/// the menu, as quads on `p`. `project` takes a world point to its pixel,
/// `None` off the camera; a ring is drawn two pixels wide in the owner's
/// unit colour.
pub fn paint(
    p: &mut Painter,
    input: &Input,
    scene: &Scene,
    sim: Option<&sim::Sim>,
    names: &Names,
    size: (i32, i32),
    project: &dyn Fn(i32, i32, i32) -> Option<(i32, i32)>,
) {
    for ring in rings(input, scene, names) {
        let c = render::silhouette::team_unit(ring.owner);
        let c = c.map(|v| f32::from(v) / 255.0);
        let px: Vec<Option<(i32, i32)>> = ring
            .points
            .iter()
            .map(|&(x, y, z)| project(x, y, z))
            .collect();
        for k in 0..px.len() {
            if let (Some(a), Some(b)) = (px[k], px[(k + 1) % px.len()]) {
                p.line(a, b, SCALE, c);
            }
        }
    }
    if let Some(bar) = Bar::of(scene, input.player) {
        let text = bar.text();
        let r = bar_rect(&text, size);
        p.rect(r.x, r.y, r.w, r.h, overlay::PANEL);
        p.text(r.x + 2 * SCALE, r.y + 2 * SCALE, &text, SCALE, overlay::INK);
    }
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
    if let Some(line) = &input.line {
        let text = if input.console {
            console_text(line)
        } else {
            line_text(line)
        };
        let r = line_rect(&text, size);
        p.rect(r.x, r.y, r.w, r.h, overlay::PANEL);
        p.rect(r.x, r.y, r.w, SCALE, overlay::INK);
        p.text(r.x + 2 * SCALE, r.y + 2 * SCALE, &text, SCALE, overlay::INK);
    }
    if input.paused {
        let r = pause_rect(size);
        p.rect(r.x, r.y, r.w, r.h, overlay::PANEL);
        p.text(
            r.x + 2 * SCALE,
            r.y + 2 * SCALE,
            PAUSE_TEXT,
            SCALE,
            overlay::INK,
        );
    }
    for b in market_menu(input, scene, size) {
        let r = b.rect;
        p.rect(r.x, r.y, r.w, r.h, [0.12, 0.14, 0.18]);
        let ink = if b.live {
            overlay::INK
        } else {
            [0.45, 0.45, 0.45]
        };
        p.rect(r.x, r.y, r.w, SCALE, ink);
        let ty = r.y + (r.h - 7 * SCALE) / 2;
        p.text(r.x + 3 * SCALE, ty, &b.label(), SCALE, ink);
    }
    for b in city_menu(input, scene, size) {
        let r = b.rect;
        p.rect(r.x, r.y, r.w, r.h, [0.12, 0.14, 0.18]);
        p.rect(r.x, r.y, r.w, SCALE, overlay::INK);
        let ty = r.y + (r.h - 7 * SCALE) / 2;
        p.text(r.x + 3 * SCALE, ty, b.label(), SCALE, overlay::INK);
    }
    for b in spell_menu(input, scene, size) {
        let r = b.rect;
        p.rect(r.x, r.y, r.w, r.h, [0.12, 0.14, 0.18]);
        p.rect(r.x, r.y, r.w, SCALE, overlay::INK);
        let ty = r.y + (r.h - 7 * SCALE) / 2;
        p.text(r.x + 3 * SCALE, ty, &b.label(), SCALE, overlay::INK);
    }
    for b in queue_menu(input, scene, size) {
        let r = b.rect;
        p.rect(r.x, r.y, r.w, r.h, [0.12, 0.14, 0.18]);
        p.rect(r.x, r.y, r.w, SCALE, overlay::INK);
        let ty = r.y + (r.h - 7 * SCALE) / 2;
        p.text(r.x + 3 * SCALE, ty, &b.label(), SCALE, overlay::INK);
    }
    if let Some(r) = rename_button(input, scene, size) {
        p.rect(r.x, r.y, r.w, r.h, [0.12, 0.14, 0.18]);
        p.rect(r.x, r.y, r.w, SCALE, overlay::INK);
        let ty = r.y + (r.h - 7 * SCALE) / 2;
        p.text(r.x + 3 * SCALE, ty, "RENAME", SCALE, overlay::INK);
    }
    if let Some(r) = garrison_button(input, scene, size) {
        p.rect(r.x, r.y, r.w, r.h, [0.12, 0.14, 0.18]);
        p.rect(r.x, r.y, r.w, SCALE, overlay::INK);
        let ty = r.y + (r.h - 7 * SCALE) / 2;
        p.text(r.x + 3 * SCALE, ty, "GARRISON", SCALE, overlay::INK);
    }
    if let Some(r) = infinite_button(input, scene, size) {
        p.rect(r.x, r.y, r.w, r.h, [0.12, 0.14, 0.18]);
        p.rect(r.x, r.y, r.w, SCALE, overlay::INK);
        let ty = r.y + (r.h - 7 * SCALE) / 2;
        p.text(r.x + 3 * SCALE, ty, "INFINITE", SCALE, overlay::INK);
    }
    if let Some(r) = stance_button(input, scene, size) {
        p.rect(r.x, r.y, r.w, r.h, [0.12, 0.14, 0.18]);
        p.rect(r.x, r.y, r.w, SCALE, overlay::INK);
        let ty = r.y + (r.h - 7 * SCALE) / 2;
        p.text(r.x + 3 * SCALE, ty, "STANCE", SCALE, overlay::INK);
    }
    if let Some(r) = disband_button(input, scene, size) {
        p.rect(r.x, r.y, r.w, r.h, [0.12, 0.14, 0.18]);
        p.rect(r.x, r.y, r.w, SCALE, overlay::INK);
        let ty = r.y + (r.h - 7 * SCALE) / 2;
        p.text(r.x + 3 * SCALE, ty, "DISBAND", SCALE, overlay::INK);
    }
    if let Some(r) = auto_explore_button(input, scene, size) {
        p.rect(r.x, r.y, r.w, r.h, [0.12, 0.14, 0.18]);
        p.rect(r.x, r.y, r.w, SCALE, overlay::INK);
        let ty = r.y + (r.h - 7 * SCALE) / 2;
        p.text(r.x + 3 * SCALE, ty, "EXPLORE", SCALE, overlay::INK);
    }
    let inside = crate::input::garrisoned(scene, input.player, &input.selection);
    for (k, r) in come_out_menu(input, scene, size) {
        p.rect(r.x, r.y, r.w, r.h, [0.12, 0.14, 0.18]);
        p.rect(r.x, r.y, r.w, SCALE, overlay::INK);
        let ty = r.y + (r.h - 7 * SCALE) / 2;
        let label = format!("OUT {}", inside[k].o);
        p.text(r.x + 3 * SCALE, ty, &label, SCALE, overlay::INK);
    }
    for b in roster_menu(input, scene, sim, size) {
        let r = b.rect;
        // A greyed button (the original's `disable`): its ink dimmed.
        let ink = if b.greyed() { GREYED } else { overlay::INK };
        p.rect(r.x, r.y, r.w, r.h, [0.12, 0.14, 0.18]);
        p.rect(r.x, r.y, r.w, SCALE / 2, ink);
        let name: String = match page_label(b.button.option) {
            Some(l) if b.ty() < 0 => l.into(),
            _ => names.of_type_index(b.ty()).chars().take(12).collect(),
        };
        p.text(r.x + 2, r.y + (r.h - 7) / 2, &name, 1, ink);
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
