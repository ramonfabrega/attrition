//! **A person's input, as orders** (item 4145, `docs/CLIENT_INPUT.md`): a press,
//! a right press and a key, under the camera the client draws with, turned
//! into packages in the minted script's own vocabulary
//! (`rondata::recgame::Order`: a frame, the issuing slot, a `group` and one
//! command behind it), which reach the walk through the turn pump
//! (`rondata::free::FreeRunner::issue`) and a recording through
//! `recgame::minted_packages` — the door the script uses
//! (`docs/RECGAME.md` §9.2). Nothing here needs a window.
//!
//! - **A press selects** the drawn object of the issuing slot whose anchor
//!   (its point at its own height, through `world_to_screen`) is nearest
//!   the pixel, within [`PICK_RADIUS`]; a press on nothing clears the
//!   selection. A press while a building is chosen places it instead.
//! - **A drag selects a box** ([`View::boxed`], item 4180,
//!   `docs/CLIENT_INPUT.md` §7): every unit of the issuing slot whose anchor
//!   falls within [`BOX_MARGIN`] of the dragged rectangle, as
//!   `Console::on_left_up@007cde90` takes them — in object order, listed by
//!   `TypeIndex` as `SelectGroup::add@00716c10` inserts them, and filtered
//!   by the default profile's civilian rule.
//! - **A right press** on the ground, with units selected, moves them: a
//!   `move_to` with the right-click's fields the script carries, and for
//!   more than one unit the formation `Console::down@007cd8f0` reads off
//!   the selection ([`View::formation`]). On an
//!   object it gives what `Console::execute_at_cursor` issues there
//!   ([`Input::on_object`], item 4179, `docs/CLIENT_INPUT.md` §6): help to
//!   build one's own site, gather at one's own camp with room, repair, or
//!   attack another slot's object.
//! - **A key** with a building selected trains the unit [`TRAIN_KEYS`]
//!   binds it to; with units selected it chooses the building
//!   [`BUILD_KEYS`] binds, which the next press places.
//!
//! **The ground under a pixel** ([`ground_at`]) is the camera's inverse
//! onto the heightfield, and an integer point: the continuous solution is
//! rounded to the nearest integer (`f64::round`), and kept only when the
//! original's own forward arithmetic (`render::camera::world_to_screen`)
//! takes it back to the same pixel; otherwise the nearest integer point
//! that does is taken, and where none does (a pixel finer than the integer
//! grid, at a close zoom) the rounded point stands. No float reaches an
//! order's bytes but through that rounding.

use rondata::commands::Command;
use rondata::recgame::Order;
use sim::Pos;
use sim::observe::{ObjId, Scene, TerrainScene};

/// The world units of one tile (`UNITS_PER_TILE`), the grain of the tile
/// heights an object stands at.
const TILE: i32 = 192;
/// The tiles of one cell.
const TILES_PER_CELL: i32 = 4;

/// How near a press must fall to an object's anchor to select it, in
/// pixels. Ours: the original's own hit test (a piece's drawn pixels) is
/// not read (`docs/CLIENT_INPUT.md` §4).
pub const PICK_RADIUS: i32 = 24;

/// The keys that train or research, and what — each a `queue_up` of the
/// `TypeIndex` on the selected building: the Citizen (50); The Art of War
/// (572), the attacked script's research at the Library; the Slingers
/// (82), its soldier at the Barracks (item 4195). Ours, not the original's
/// hotkeys.
pub const TRAIN_KEYS: &[(char, i32)] = &[('c', 50), ('a', 572), ('s', 82)];
/// The keys that choose a building to place: the Farm (`TypeIndex` 417)
/// and the Barracks (427, item 4195). Ours, not the original's hotkeys.
pub const BUILD_KEYS: &[(char, i32)] = &[('f', 417), ('b', 427)];

/// How far the box test reaches past the dragged rectangle, in pixels: the
/// original's own, `BoundingBox::around_bounding_volume@00544860`'s ±30
/// (`0x41f00000`) on each side of the sorted `select_x`..`select_x2`,
/// `select_y`..`select_y2`.
pub const BOX_MARGIN: i32 = 30;
/// The most members a selection takes: `SelectGroup::add`'s `num < 0x80`.
pub const SELECT_CAP: usize = 0x80;
/// How far the cursor must move between a left press and its release for
/// the window to send a [`Event::Drag`] rather than a [`Event::Press`], in
/// pixels. Ours: the original's `clicking` threshold is not read.
pub const DRAG_SLOP: i32 = 4;

/// **The command kinds this layer mints** (item 4249,
/// `docs/CLIENT_INPUT.md` §11), by `Command::name`: what [`Input::event`]
/// can put in an order. `the_layer_mints_what_it_says` reads this file's
/// own source and holds the list to the `Command::` values it builds.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "the census and the source scan read it; the window does not"
    )
)]
pub const MINTS: &[&str] = &[
    "group",
    "move_to",
    "attack",
    "follow",
    "guard",
    "swarm_around",
    "gather",
    "build",
    "queue_up",
    "gather_point",
    "chat",
    "pause",
    "sell",
    "buy",
    "city_gather",
];

/// **The entry line's text limit**, in UTF-16 units: `ChatBox::exec
/// @0093cf80` sets the edit box's `text_limit` to `0x80`.
pub const CHAT_LIMIT: usize = 0x80;
/// **The cheat prefix** `ChatBox::on_modal_end@0093d560` tests a line
/// against (`get_cheat_string@00497c00`, a localised string; the install's
/// `CHEAT `, compared without case by `String::ignore`'s `_wcsnicmp` over
/// its length). Every chat the recordings on disk carry begins with it
/// (`docs/CLIENT_INPUT.md` §12).
pub const CHEAT_PREFIX: &str = "CHEAT ";

/// One input event, at a screen pixel where it has one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    Press {
        x: i32,
        y: i32,
    },
    /// A left press at `(x0, y0)`, a drag, and the release at `(x1, y1)`.
    Drag {
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
    },
    RightPress {
        x: i32,
        y: i32,
    },
    /// A right press on the minimap at the world point it names
    /// (`WorldMap::on_right_up@008c7050`, item 4194): a `move_to` with the
    /// minimap's `form`/`width` −1/−1, the fields the scripts before
    /// [`rondata::script::clicked_main_view`] carry. Ours: the minimap is
    /// not drawn, so the event carries the point rather than a pixel.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "the window draws no minimap; the tests send it")
    )]
    MapRightPress {
        x: i32,
        y: i32,
    },
    Key(char),
    /// Return: opens the entry line, or sends what it holds
    /// (`OPTION_CHAT`, `Options::exec@007188c0`'s `ChatBox::exec(0)`; item
    /// 4250, `docs/CLIENT_INPUT.md` §12).
    Enter,
    /// Takes the entry line's last character.
    Backspace,
    /// Closes the entry line and sends nothing (`on_modal_end` with a
    /// nonzero `result`).
    Escape,
    /// The pause key (`OPTION_PAUSE`, `Options::exec@007188c0`'s
    /// `TurnControl::issue_toggle_pause@00956b40`; item 4252,
    /// `docs/CLIENT_INPUT.md` §13).
    Pause,
    /// A press on the market's sell button for a good (`OPTION_SELL`,
    /// `Options::exec@007188c0`'s `LeaderOut::issue_sell@006d3eb0`; item
    /// 4259, `docs/CLIENT_INPUT.md` §16). The HUD's alone: no key sends it.
    Sell(usize),
    /// A press on the market's buy button for a good (`OPTION_BUY`,
    /// `LeaderOut::issue_buy@006d3ef0`).
    Buy(usize),
    /// A press on one of the city's four gather buttons, by the type it
    /// gathers to (`OPTION_CITY_FARM_GATHER` and its three,
    /// `Options::exec@007188c0`'s `do_city_gather(0x1a1, 0x1a2, 0x1a3 or
    /// −1)`; item 4261, `docs/CLIENT_INPUT.md` §17). The HUD's alone.
    CityGather(i32),
}

/// **The city's gather buttons' types** (item 4261): Farm, Woodcutter,
/// Mine and all three, in `make_options`' order, as `Options::exec` passes
/// each to `do_city_gather`.
pub const CITY_GATHER_TYPES: [i32; 4] = [0x1a1, 0x1a2, 0x1a3, -1];

/// **Whether the selection's panel is a city's** (item 4261,
/// `docs/CLIENT_INPUT.md` §17): one building of `player`'s own, a city's
/// centre (`+8 & 0x20`), whose panel `BuildOut::make_options@00629ed0`
/// gives the four gather buttons. The profile's preference the original
/// also asks (`prefs & 2`) is taken as set.
pub fn city_panel(scene: &Scene, player: u8, selection: &[ObjId]) -> bool {
    let [id] = selection else {
        return false;
    };
    scene
        .buildings
        .iter()
        .any(|b| b.id == *id && b.owner == player && b.city)
}

/// **Whether a city's gather button issues** (item 4261): the gate of
/// `Options::do_city_gather@0071cdb0`, a selection of buildings
/// (`GroupData::buildings`) of which one takes a gather point
/// (`COUNT_GATHER_POINT`, its training arm as [`View::takes_gather_point`]
/// reads it) and one is a `VILLAGE` (`COUNT_TYPE` 0x19e).
pub fn city_gathers(scene: &Scene, selection: &[ObjId]) -> bool {
    let of = |id: &ObjId| scene.buildings.iter().find(|b| b.id == *id);
    !selection.is_empty()
        && selection.iter().all(|id| of(id).is_some())
        && selection.iter().any(|id| of(id).is_some_and(|b| b.trains))
        && selection.iter().any(|id| of(id).is_some_and(|b| b.village))
}

/// **The market the selection is** (item 4259, `docs/CLIENT_INPUT.md`
/// §16): one building of `player`'s own, a finished market, whose panel
/// `BuildOut::make_options@00629ed0` fills; the leader's offer per good,
/// `(buy, sell)` where it has a button ([`sim::observe::LeaderScene::market`]).
pub fn market(scene: &Scene, player: u8, selection: &[ObjId]) -> Option<[Option<(i32, i32)>; 6]> {
    let [id] = selection else {
        return None;
    };
    let b = scene.buildings.iter().find(|b| b.id == *id)?;
    if b.owner != player || !b.market || !b.active {
        return None;
    }
    Some(scene.leaders.iter().find(|l| l.who == player)?.market)
}

/// Whether the market's button for `good` is live, as `make_options` adds
/// it: a sale with a hundred of the good in the bucket, a purchase with
/// the wealth for its price. A button the original adds disabled mints
/// nothing here.
pub fn trades(scene: &Scene, player: u8, selection: &[ObjId], good: usize, buy: bool) -> bool {
    let Some((price, _)) = market(scene, player, selection).and_then(|m| *m.get(good)?) else {
        return false;
    };
    let Some(l) = scene.leaders.iter().find(|l| l.who == player) else {
        return false;
    };
    if buy {
        l.bucket[sim::economy::Resource::Wealth as usize] >= price
    } else {
        l.bucket[good] >= 100
    }
}

/// What the layer sees: the camera's matrix and screen as the draw takes
/// them, the frame's scene and the ground's tile heights.
pub struct View<'a> {
    pub matrix: render::camera::Mat4,
    pub screen: render::Screen,
    /// The size `world_to_screen` is called with: `None` takes `graphsys`'s.
    pub size: Option<(i32, i32)>,
    pub scene: &'a Scene,
    pub terrain: &'a TerrainScene,
}

impl View<'_> {
    /// A world point at height `z`, as a pixel.
    pub fn pixel(&self, x: i32, y: i32, z: i32) -> (i32, i32) {
        let p = render::camera::world_to_screen(&self.matrix, &self.screen, x, y, z, self.size);
        (p.x, p.y)
    }

    /// The map's extent in world units, `(width, height)`: the bound the
    /// minimap's right press holds its point to (`worldc +0x18`, `+0x1c`
    /// cells of `0xc0`, `WorldMap::on_right_up@008c7050`).
    pub fn map_extent(&self) -> (i32, i32) {
        let t = self.terrain;
        (
            t.width * TILES_PER_CELL * TILE,
            t.height * TILES_PER_CELL * TILE,
        )
    }

    /// The height an object stands at on the ground at `(x, y)`: its
    /// tile's height, never below zero (`sim::World::object_z`, whose
    /// ocean arm is not carried by the scene: `docs/CLIENT_INPUT.md` §4).
    pub fn ground_z(&self, x: i32, y: i32) -> i32 {
        let t = self.terrain;
        let (w, h) = (t.width * TILES_PER_CELL, t.height * TILES_PER_CELL);
        let (tx, ty) = (x.div_euclid(TILE), y.div_euclid(TILE));
        if tx < 0 || ty < 0 || tx >= w || ty >= h {
            return 0;
        }
        t.tile_z
            .get((ty * w + tx) as usize)
            .copied()
            .unwrap_or(0)
            .max(0)
    }

    /// Every drawn object of `who`, with its anchor pixel.
    fn anchors(&self, who: u8) -> impl Iterator<Item = (ObjId, bool, (i32, i32))> + '_ {
        let units = self.scene.units.iter().map(|u| (u.id, false, u.pos, u.z));
        let builds = self
            .scene
            .buildings
            .iter()
            .map(|b| (b.id, true, b.pos, b.z));
        units
            .chain(builds)
            .filter(move |(id, ..)| id.who == who)
            .map(|(id, b, p, z)| (id, b, self.pixel(p.x, p.y, z)))
    }
}

/// `ObjectData::is_worker@0046fa10`: the type is `0x32`..`0x35`, the
/// Citizens and the Scholars.
fn is_worker(ti: i32) -> bool {
    (0x32..=0x35).contains(&ti)
}

/// `UnitData::is_merchant@0046d370`: the three merchant types.
fn is_merchant(ti: i32) -> bool {
    matches!(ti, 0x3d | 0x3e | 400)
}

/// The Merchant and the fishermen, `GroupData::count(COUNT_TYPE, ·)`'s two
/// types in `Console::on_left_up`'s civilian rule (`MERCHANT`, `FISHERMEN`).
const MERCHANT: i32 = 0x3d;
const FISHERMEN: i32 = 0x13d;
const MERCHANT_FLEET: i32 = 0x13e;

impl View<'_> {
    /// **A box** (`Console::on_left_up@007cde90`, `docs/CLIENT_INPUT.md`
    /// §7): the issuing slot's units the dragged rectangle takes, in the
    /// order the selection lists them.
    ///
    /// 1. The rectangle is sorted, and a unit is in it when its anchor is
    ///    within [`BOX_MARGIN`] of it (the original projects each figure's
    ///    bounding box centre; ours is the anchor).
    /// 2. The units are walked in object order, as the loop walks the
    ///    object array, and the first [`SELECT_CAP`] are added. Each add
    ///    inserts the unit before the first member whose `TypeIndex` is
    ///    greater (`SelectGroup::add`), so the list is sorted by type and
    ///    in object order within one.
    /// 3. The default profile sets `0x200000` (`PlayerProfile::default_mouse
    ///    @005d01f0`), so the box is filtered: when every member but the
    ///    Merchants and fishermen is a worker, and there is a worker, only
    ///    the workers stay; otherwise the civilians go — the workers, the
    ///    merchants and the fishing and merchant fleets.
    pub fn boxed(&self, who: u8, (x0, y0): (i32, i32), (x1, y1): (i32, i32)) -> Vec<ObjId> {
        let (l, r) = (x0.min(x1) - BOX_MARGIN, x0.max(x1) + BOX_MARGIN);
        let (t, b) = (y0.min(y1) - BOX_MARGIN, y0.max(y1) + BOX_MARGIN);
        let mut inside: Vec<_> = self
            .scene
            .units
            .iter()
            .filter(|u| u.id.who == who)
            .filter(|u| {
                let (ax, ay) = self.pixel(u.pos.x, u.pos.y, u.z);
                l < ax && ax < r && t < ay && ay < b
            })
            .map(|u| (u.id, u.type_index))
            .collect();
        inside.sort_by_key(|&(id, _)| id.o);
        inside.truncate(SELECT_CAP);
        inside.sort_by_key(|&(_, ti)| ti);
        let workers = inside.iter().filter(|(_, ti)| is_worker(*ti)).count();
        let special = inside
            .iter()
            .filter(|(_, ti)| matches!(*ti, MERCHANT | FISHERMEN))
            .count();
        let keep: fn(i32) -> bool = if inside.len() - special == workers {
            if workers == 0 { |_| true } else { is_worker }
        } else {
            |ti| !(is_worker(ti) || is_merchant(ti) || matches!(ti, FISHERMEN | MERCHANT_FLEET))
        };
        inside
            .into_iter()
            .filter(|&(_, ti)| keep(ti))
            .map(|(id, _)| id)
            .collect()
    }

    /// **The formation a right press moves a selection in**: what
    /// `Console::down@007cd8f0` writes when the right button goes down,
    /// `drag_form` and `drag_form_mod`, which `Console::execute_at_cursor`
    /// hands `GroupOut::issue_move_to` as the order's `form` and `width`.
    ///
    /// - `form` is `GroupData::get_form@0070b9f0` clamped up to 0: the
    ///   `form` byte every member shares, −1 as soon as two differ.
    /// - `width` is `GroupData::get_form_mod_option@0070bd00`: for more
    ///   than one member the mean of their `form_width` bytes that are not
    ///   −1, and 50 (`0x32`) when none is; for one, its own byte, or 50.
    ///
    /// The members are the selection's units on the map; a plane's own
    /// test in the width's mean is not carried (`docs/CLIENT_INPUT.md` §7.4).
    pub fn formation(&self, selection: &[ObjId]) -> (i8, i8) {
        let members: Vec<_> = selection
            .iter()
            .filter_map(|id| self.scene.units.iter().find(|u| u.id == *id))
            .collect();
        let mut form = -1;
        for (k, u) in members.iter().enumerate() {
            if k > 0 && u.form != form {
                form = -1;
                break;
            }
            form = u.form;
        }
        let set: Vec<i32> = members
            .iter()
            .map(|u| i32::from(u.form_width))
            .filter(|&w| w != -1)
            .collect();
        let width = if set.is_empty() {
            0x32
        } else {
            set.iter().sum::<i32>() / set.len() as i32
        };
        (form.max(0), width as i8)
    }
}

/// What a right press finds under the cursor ([`View::target`]): another
/// slot's unit or building, or one of the issuing slot's own buildings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hit {
    /// Another slot's unit or building: `Console::execute_at_cursor`'s
    /// enemy arm (`docs/CLIENT_INPUT.md` §6), and for a unit the follow
    /// arm of a selection that cannot attack (§10).
    Enemy { id: ObjId, unit: bool },
    /// One of the issuing slot's own units: the guard arm (§10).
    Friend(ObjId),
    /// One of the issuing slot's buildings, as the dispatch tests it: built
    /// (`is_active`), damaged (`ObjectData +0x24`), and for a gather type
    /// whether a gatherer can join (`num_gatherers < gather_max`).
    Own {
        id: ObjId,
        active: bool,
        damaged: bool,
        /// `None` for a building that is not a gather type; `Some(true)`
        /// for one with room.
        room: Option<bool>,
    },
}

impl View<'_> {
    /// The object a right press at `(px, py)` names for `who`, as
    /// `Console::execute_at_cursor` looks: another slot's unit first
    /// (`find_target(SEARCH_ENEMY, units)`), then a building of `who`'s
    /// (`find_target(SEARCH_ALLIED, buildings)`), then another slot's
    /// building (the attack's search, units and buildings), then one of
    /// `who`'s own units (the follow arm's `find_target(SEARCH_ALL, units)`,
    /// which `LeaderData::is_ally` answers yes for the slot itself: item
    /// 4202). Each look takes the nearest anchor within [`PICK_RADIUS`], as
    /// a press does (ours: the original's ±5 px box and bounding volumes are
    /// not read), and only an object the viewer sees:
    /// `Console::find_target_daemon@007c5930` admits none whose `is_seen`
    /// (vslot `+0x48`) answers no, so a press on what the fog hides falls
    /// to the ground (§10).
    pub fn target(&self, who: u8, px: i32, py: i32) -> Option<Hit> {
        let near = |p: Pos, z: i32| {
            let (ax, ay) = self.pixel(p.x, p.y, z);
            let d = i64::from(ax - px).pow(2) + i64::from(ay - py).pow(2);
            (d <= i64::from(PICK_RADIUS).pow(2)).then_some(d)
        };
        let nearest =
            |it: &mut dyn Iterator<Item = (i64, ObjId)>| it.min_by_key(|&(d, id)| (d, id));
        let enemy_unit = nearest(
            &mut self
                .scene
                .units
                .iter()
                .filter(|u| u.id.who != who && u.seen)
                .filter_map(|u| Some((near(u.pos, u.z)?, u.id))),
        );
        if let Some((_, id)) = enemy_unit {
            return Some(Hit::Enemy { id, unit: true });
        }
        let own = self
            .scene
            .buildings
            .iter()
            .filter(|b| b.id.who == who)
            .filter_map(|b| Some((near(b.pos, b.z)?, b)))
            .min_by_key(|&(d, b)| (d, b.id));
        if let Some((_, b)) = own {
            return Some(Hit::Own {
                id: b.id,
                active: b.active,
                damaged: b.health < b.hits,
                room: b.gather.map(|(n, max)| max.is_none_or(|m| n < m)),
            });
        }
        let enemy_building = nearest(
            &mut self
                .scene
                .buildings
                .iter()
                .filter(|b| b.id.who != who && b.seen)
                .filter_map(|b| Some((near(b.pos, b.z)?, b.id))),
        );
        if let Some((_, id)) = enemy_building {
            return Some(Hit::Enemy { id, unit: false });
        }
        let friend = nearest(
            &mut self
                .scene
                .units
                .iter()
                .filter(|u| u.id.who == who)
                .filter_map(|u| Some((near(u.pos, u.z)?, u.id))),
        );
        friend.map(|(_, id)| Hit::Friend(id))
    }

    /// `GroupData::count(COUNT_GATHER_POINT)` of a building selection, as
    /// far as nonzero, on its training arm alone: whether a selected
    /// building trains ([`sim::observe::BuildingScene::trains`]; item 4249,
    /// `docs/CLIENT_INPUT.md` §11). Ours: the garrison-limit and type-ability
    /// arms of the count are not read.
    pub fn takes_gather_point(&self, selection: &[ObjId]) -> bool {
        selection
            .iter()
            .any(|id| self.scene.buildings.iter().any(|b| b.id == *id && b.trains))
    }

    /// [`trades`] on the view's scene.
    pub fn trades(&self, player: u8, selection: &[ObjId], good: usize, buy: bool) -> bool {
        trades(self.scene, player, selection, good, buy)
    }

    /// Whether a right press at `(px, py)` names any object the viewer
    /// sees, of any slot, unit or building: the gather point's first look,
    /// `find_target(SEARCH_ALL, …, FILTER_ALL)`, then its friendly one
    /// (item 4249). Each look is [`View::target`]'s nearest anchor within
    /// [`PICK_RADIUS`].
    pub fn names_object(&self, px: i32, py: i32) -> bool {
        let near = |p: Pos, z: i32| {
            let (ax, ay) = self.pixel(p.x, p.y, z);
            i64::from(ax - px).pow(2) + i64::from(ay - py).pow(2) <= i64::from(PICK_RADIUS).pow(2)
        };
        self.scene.units.iter().any(|u| u.seen && near(u.pos, u.z))
            || self
                .scene
                .buildings
                .iter()
                .any(|b| b.seen && near(b.pos, b.z))
    }

    /// `GroupData::count(COUNT_ATTACK)` of a selection, as far as nonzero:
    /// whether any member on the map attacks ([`sim::observe::UnitScene::attacks`]).
    pub fn attacks(&self, selection: &[ObjId]) -> bool {
        selection
            .iter()
            .any(|id| self.scene.units.iter().any(|u| u.id == *id && u.attacks))
    }
}

/// `OrderIndex::BUILD_AT` and `REPAIR`, the `swarm_around` kinds
/// (`sim::ids::order`).
const BUILD_AT: i32 = sim::ids::order::BUILD_AT as i32;
const REPAIR: i32 = sim::ids::order::REPAIR as i32;
/// `QUEUE_NEW`: every arm's queue with no modifier held.
const QUEUE_NEW: i32 = 2;

/// The continuous ground point under a pixel: the camera's equations
/// solved for `(x, y)` at a height, the height re-read at the solution
/// until it holds.
fn continuous(view: &View, px: i32, py: i32) -> Option<(f64, f64)> {
    let m = view.matrix.0.map(f64::from);
    let (w, h) = view.size.unwrap_or((
        view.screen.scaled_xs,
        (view.screen.scaled_ys as f32 * view.screen.natural_ratio) as i32,
    ));
    let sx = 2.0 * f64::from(px) / f64::from(w) - 1.0;
    let sy = 1.0 - 2.0 * f64::from(py - i32::from(view.screen.console_y)) / f64::from(h);
    let mut z = 0.0;
    let mut at = None;
    for _ in 0..16 {
        // sx·(m3 x + m7 y + m11 z + m15) = m0 x + m4 y + m8 z + m12, and
        // the same in y: linear in (x, y) for either projection.
        let (a, b) = (m[0] - sx * m[3], m[4] - sx * m[7]);
        let (c, d) = (m[1] - sy * m[3], m[5] - sy * m[7]);
        let e = sx * (m[11] * z + m[15]) - (m[8] * z + m[12]);
        let f = sy * (m[11] * z + m[15]) - (m[9] * z + m[13]);
        let det = a * d - b * c;
        if det == 0.0 {
            return None;
        }
        let (x, y) = ((e * d - b * f) / det, (a * f - e * c) / det);
        let nz = f64::from(view.ground_z(x.round() as i32, y.round() as i32));
        at = Some((x, y));
        if nz == z {
            break;
        }
        z = nz;
    }
    at
}

/// **The ground under a pixel**, as an integer point the forward
/// projection takes back to that pixel (the module note).
pub fn ground_at(view: &View, px: i32, py: i32) -> Option<Pos> {
    let (x, y) = continuous(view, px, py)?;
    let (rx, ry) = (x.round() as i32, y.round() as i32);
    let back = |x: i32, y: i32| view.pixel(x, y, view.ground_z(x, y)) == (px, py);
    if back(rx, ry) {
        return Some(Pos { x: rx, y: ry });
    }
    let mut best: Option<(f64, Pos)> = None;
    for dy in -16..=16 {
        for dx in -16..=16 {
            let (qx, qy) = (rx + dx, ry + dy);
            if back(qx, qy) {
                let d = (f64::from(qx) - x).powi(2) + (f64::from(qy) - y).powi(2);
                if best.is_none_or(|(b, _)| d < b) {
                    best = Some((d, Pos { x: qx, y: qy }));
                }
            }
        }
    }
    // A pixel finer than the integer grid holds no point of its own: the
    // rounded solution, whose pixel is a neighbour's.
    Some(best.map_or(Pos { x: rx, y: ry }, |(_, p)| p))
}

/// Every integer point within `r` of `at` the forward projection puts on
/// the pixel `at` projects to: one pixel's footprint on the ground there.
#[cfg(test)]
pub fn footprint(view: &View, at: Pos, r: i32) -> Vec<Pos> {
    let px = view.pixel(at.x, at.y, view.ground_z(at.x, at.y));
    let mut out = Vec::new();
    for dy in -r..=r {
        for dx in -r..=r {
            let (x, y) = (at.x + dx, at.y + dy);
            if view.pixel(x, y, view.ground_z(x, y)) == px {
                out.push(Pos { x, y });
            }
        }
    }
    out
}

/// A right press's `move_to` at `at`, the script's right-click fields with
/// the arm's `form` and `width`.
fn move_to(at: Pos, form: i8, width: i8) -> Command {
    Command::MoveTo {
        to_x: at.x,
        to_y: at.y,
        set_angle: 0,
        angle: 0,
        orders: 1,
        queued: 2,
        form,
        width,
        disembark: 0,
    }
}

/// The session's input state: who issues, and what is selected or chosen.
#[derive(Clone, Debug, Default)]
pub struct Input {
    /// The issuing slot.
    pub player: u8,
    /// The selection, in the order it was made.
    pub selection: Vec<ObjId>,
    /// Whether the selection is a building.
    pub building: bool,
    /// The building chosen to place, by `TypeIndex`.
    pub placing: Option<i32>,
    /// The chat entry line, open with what it holds, or closed (item 4250).
    pub line: Option<String>,
    /// Whether the slot's own pause stands (item 4252): the turn control's
    /// bit, as the layer last set it. While it stands the window steps no
    /// frame, and every order is minted on the frame it stopped on.
    pub paused: bool,
}

impl Input {
    pub fn new(player: u8) -> Input {
        Input {
            player,
            ..Input::default()
        }
    }

    /// The `group` the selection makes.
    fn group(&self) -> Command {
        Command::Group {
            who: self.player as i8,
            objects: self.selection.iter().map(|id| id.o).collect(),
        }
    }

    /// **A right press on an object** (item 4179, `docs/CLIENT_INPUT.md` §6):
    /// the command `Console::execute_at_cursor` issues for a selection of
    /// units with no modifier held, or none.
    ///
    /// - another slot's unit or building, with a member that attacks:
    ///   `attack(ox, whom, ignore 0, QUEUE_NEW)` (`GroupOut::issue_attack`);
    /// - another slot's unit, with none (`COUNT_ATTACK` 0, item 4202):
    ///   `follow(ox, whom, QUEUE_NEW)` (`GroupOut::issue_follow`); the layer
    ///   names no ally, so it never draws the arm's `guard` there;
    /// - another slot's building, with none: nothing here, and the press
    ///   falls to the ground (ours: the arms past the follow are not read);
    /// - one of the slot's own units: `guard(ox, whom, QUEUE_NEW)`
    ///   (`GroupOut::issue_guard`; `is_ally` of the slot itself);
    /// - one's own unfinished building: `swarm_around(ox, whom, QUEUE_NEW,
    ///   BUILD_AT)`, citizens sent to help build it;
    /// - one's own finished gather building with room: `gather(ox,
    ///   QUEUE_NEW)`;
    /// - one's own finished building that is damaged, and full or not a
    ///   gather type: `swarm_around(…, REPAIR)`;
    /// - one's own finished building that is undamaged and not a gather
    ///   building with room (the city, a full camp): nothing.
    fn on_object(&self, hit: Hit, attacks: bool) -> Option<Command> {
        match hit {
            Hit::Enemy { id, .. } if attacks => Some(Command::Attack {
                ox: i32::from(id.o),
                whom: i32::from(id.who),
                ignore: 0,
                queued: QUEUE_NEW,
            }),
            Hit::Enemy { id, unit: true } => Some(Command::Follow {
                ox: i32::from(id.o),
                whom: i32::from(id.who),
                queued: QUEUE_NEW,
            }),
            Hit::Enemy { unit: false, .. } => None,
            Hit::Friend(id) => Some(Command::Guard {
                ox: i32::from(id.o),
                whom: i32::from(id.who),
                queued: QUEUE_NEW,
            }),
            Hit::Own {
                id,
                active,
                damaged,
                room,
            } => {
                let swarm = |orders| Command::SwarmAround {
                    ox: i32::from(id.o),
                    whom: i32::from(id.who),
                    queued: QUEUE_NEW,
                    orders,
                };
                if !active {
                    Some(swarm(BUILD_AT))
                } else if room == Some(true) {
                    Some(Command::Gather {
                        ox: i32::from(id.o),
                        queued: QUEUE_NEW,
                    })
                } else if damaged {
                    Some(swarm(REPAIR))
                } else {
                    None
                }
            }
        }
    }

    /// **A line sent from the entry line** (item 4250, `docs/CLIENT_INPUT.md`
    /// §12): the `chat` the original's chat box issues for it in a solo
    /// game, or none.
    ///
    /// - A line that begins with [`CHEAT_PREFIX`] is `on_modal_end`'s cheat
    ///   arm: `CommandManager::issue_chat(who, line, 0, 0, 1 << who)`.
    /// - Any other line is the plain box's (`special_flags` 0), through
    ///   `LeaderOut::issue_chat@006d2c50`'s default arm: the recipients are
    ///   the slot itself and every live leader whose `chat_status` toward
    ///   the slot is 0, which `Game::init_teams` leaves on every seated slot
    ///   of a game that is not cooperative; with fewer than three nations the
    ///   text goes unchanged; `taunt` is `Taunts::check_for_order`'s −1, no
    ///   taunt holding the index 0 that a line not beginning with a digit
    ///   converts to; `taunt_num` 0.
    /// - `CommandManager::issue_chat@00942b10` trims the spaces at both ends
    ///   and adds nothing for a line left empty.
    ///
    /// Ours, and none minted: a line whose first character picks the
    /// targeted arms (`*`, `!`, `;`, `:`, `@`), a game of three nations or
    /// more (the prefixes are localised strings not read), and a line that
    /// begins with a digit or a sign (a taunt, parked as 4250a). Every
    /// leader the scene carries is taken as live and seated.
    pub fn chat(&self, frame: i32, line: &str, view: &View) -> Option<Order> {
        let own = 1i32 << self.player;
        let text = line.trim_matches(' ').to_string();
        let cheat = line
            .get(..CHEAT_PREFIX.len())
            .is_some_and(|p| p.eq_ignore_ascii_case(CHEAT_PREFIX));
        let cmd = if cheat {
            Command::Chat {
                bits: own,
                taunt: 0,
                taunt_num: 0,
                text,
            }
        } else {
            let first = line.chars().next()?;
            let starts_taunt = text
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_digit() || c == '-' || c == '+');
            if "*!;:@".contains(first) || view.scene.leaders.len() >= 3 || starts_taunt {
                return None;
            }
            let bits = view
                .scene
                .leaders
                .iter()
                .fold(own, |b, l| b | 1i32 << l.who);
            Command::Chat {
                bits,
                taunt: -1,
                taunt_num: 0,
                text,
            }
        };
        if matches!(&cmd, Command::Chat { text, .. } if text.is_empty()) {
            return None;
        }
        Some((frame, i32::from(self.player), vec![cmd]))
    }

    /// One event on `frame` (the recording frame the package is consumed
    /// on): the order it makes, if any.
    pub fn event(&mut self, frame: i32, ev: Event, view: &View) -> Option<Order> {
        let play = i32::from(self.player);
        match ev {
            Event::Enter => match self.line.take() {
                None => {
                    self.line = Some(String::new());
                    None
                }
                Some(line) => self.chat(frame, &line, view),
            },
            Event::Backspace => {
                if let Some(line) = &mut self.line {
                    line.pop();
                }
                None
            }
            Event::Escape => {
                self.line = None;
                None
            }
            Event::Pause => {
                // `issue_toggle_pause`: `pause(state)`, the bit's complement,
                // in a package of its own; the bit itself turns when the
                // package is processed, in a solo game the same turn.
                let state = u8::from(!self.paused);
                self.paused = !self.paused;
                Some((frame, play, vec![Command::Pause { state }]))
            }
            Event::Sell(good) | Event::Buy(good) => {
                // The market's options (item 4259): `issue_sell` and
                // `issue_buy` write the console's leader, the good and
                // the flags, 0 for the plain button, in a package of their
                // own with no group.
                let buy = matches!(ev, Event::Buy(_));
                if !view.trades(self.player, &self.selection, good, buy) {
                    return None;
                }
                let (who, good) = (play, good as i32);
                let cmd = if buy {
                    Command::Buy {
                        who,
                        good,
                        flags: 0,
                    }
                } else {
                    Command::Sell {
                        who,
                        good,
                        flags: 0,
                    }
                };
                Some((frame, play, vec![cmd]))
            }
            Event::CityGather(t) => {
                // `do_city_gather`: the selection's `group` and
                // `city_gather(t, 2)`, `GroupOut::issue_city_gather@0070ac70`'s
                // `queued` 2 whatever the keys.
                if !self.building || !city_gathers(view.scene, &self.selection) {
                    return None;
                }
                let cmd = Command::CityGather { t, queued: 2 };
                Some((frame, play, vec![self.group(), cmd]))
            }
            Event::Key(k) if self.line.is_some() => {
                let line = self.line.as_mut().unwrap();
                if line.encode_utf16().count() + k.len_utf16() <= CHAT_LIMIT {
                    line.push(k);
                }
                None
            }
            Event::Press { x, y } => {
                if let Some(ty) = self.placing {
                    let at = ground_at(view, x, y)?;
                    self.placing = None;
                    let cmd = Command::Build {
                        x: at.x,
                        y: at.y,
                        x2: at.x,
                        y2: at.y,
                        build: ty,
                        queued: 2,
                    };
                    return Some((frame, play, vec![self.group(), cmd]));
                }
                let hit = view
                    .anchors(self.player)
                    .map(|(id, b, (ax, ay))| {
                        let d = i64::from(ax - x).pow(2) + i64::from(ay - y).pow(2);
                        (d, id, b)
                    })
                    .filter(|(d, ..)| *d <= i64::from(PICK_RADIUS).pow(2))
                    .min_by_key(|(d, id, _)| (*d, *id));
                match hit {
                    Some((_, id, b)) => {
                        self.selection = vec![id];
                        self.building = b;
                    }
                    None => self.selection.clear(),
                }
                None
            }
            Event::Drag { x0, y0, x1, y1 } => {
                self.placing = None;
                self.selection = view.boxed(self.player, (x0, y0), (x1, y1));
                self.building = false;
                None
            }
            Event::RightPress { x, y } => {
                if self.selection.is_empty() {
                    return None;
                }
                if self.building {
                    // The building arm (`execute_at_cursor`, item 4249): a
                    // selection that takes a gather point, and no object
                    // under the press, names the ground, `action` 0 and
                    // `add_to_end` the shift's, 0 here. On an object the
                    // arms' actions 1 to 3 are not read: nothing.
                    if !view.takes_gather_point(&self.selection) || view.names_object(x, y) {
                        return None;
                    }
                    let at = ground_at(view, x, y)?;
                    let cmd = Command::GatherPoint {
                        x: at.x,
                        y: at.y,
                        action: 0,
                        add_to_end: 0,
                    };
                    return Some((frame, play, vec![self.group(), cmd]));
                }
                let hit = view.target(self.player, x, y);
                if let Some(cmd) =
                    hit.and_then(|h| self.on_object(h, view.attacks(&self.selection)))
                {
                    return Some((frame, play, vec![self.group(), cmd]));
                }
                if matches!(hit, Some(Hit::Own { .. })) {
                    // One's own finished, undamaged building with no room:
                    // nothing (§6.1).
                    return None;
                }
                let at = ground_at(view, x, y)?;
                // The main view's formation, for one unit as for many
                // (`Console::down@007cd8f0`; item 4194).
                let (form, width) = view.formation(&self.selection);
                Some((frame, play, vec![self.group(), move_to(at, form, width)]))
            }
            Event::MapRightPress { x, y } => {
                // The minimap's arm: the point on the map, the selection
                // able to act, `MOVE_TO` with −1/−1.
                let (w, h) = view.map_extent();
                if self.selection.is_empty()
                    || self.building
                    || !(0..w).contains(&x)
                    || !(0..h).contains(&y)
                {
                    return None;
                }
                let cmd = move_to(Pos { x, y }, -1, -1);
                Some((frame, play, vec![self.group(), cmd]))
            }
            Event::Key(k) => {
                if self.selection.is_empty() {
                    return None;
                }
                if self.building {
                    let &(_, unit) = TRAIN_KEYS.iter().find(|(c, _)| *c == k)?;
                    let cmd = Command::QueueUp { unit, num: 1 };
                    Some((frame, play, vec![self.group(), cmd]))
                } else {
                    let &(_, ty) = BUILD_KEYS.iter().find(|(c, _)| *c == k)?;
                    self.placing = Some(ty);
                    None
                }
            }
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use rondata::free::FreeRunner;
    use rondata::script;

    /// The script's lobby, built from its choices, or why not.
    pub(crate) fn lobby() -> Result<FreeRunner, String> {
        let root = std::env::var_os("RON_INSTALL")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| "../../game".into());
        let install = rondata::Install::new(&root);
        if !install.looks_valid() {
            return Err(format!("no install at {}", root.display()));
        }
        let path = crate::world::gamelog_dir().join(script::LOBBY_DUMP);
        if !path.is_file() {
            return Err(format!("no lobby at {}", path.display()));
        }
        let loaded = rondata::load::load(&install).map_err(|e| e.to_string())?;
        FreeRunner::from_lobby(&loaded, &path, sim::Tuning::RON).map_err(|e| e.to_string())
    }

    /// The camera the client opens with ([`crate::pinned_camera`]), or —
    /// `Some((target, z))` — the closer one: the same camera looked at the
    /// target at its ground height (`loc`, `loc_z_adjust`) and zoomed by
    /// [`CLOSER`] (`--zoom`).
    pub(crate) fn camera(at: Option<(Pos, i32)>) -> render::Camera {
        let mut c = crate::pinned_camera();
        if let Some((t, z)) = at {
            (c.loc_x, c.loc_y, c.loc_z_adjust) = (t.x as f32, t.y as f32, z as f32);
            c.parallel_distance /= CLOSER;
        }
        c
    }

    /// The closer camera's zoom: where one pixel's footprint on the ground
    /// at both targets is one integer point (stage 0's table).
    const CLOSER: f32 = 8.0;

    pub(crate) fn view<'a>(
        c: &render::Camera,
        scene: &'a Scene,
        terrain: &'a TerrainScene,
    ) -> View<'a> {
        View {
            matrix: c.to_screen_matrix(),
            screen: crate::pinned_screen(),
            size: None,
            scene,
            terrain,
        }
    }

    /// **Stage 0's table** (`docs/CLIENT_INPUT.md` §2), on each order's frame of
    /// the built start walked under the script: `(frame, object, its point
    /// and height, its pixel under the opening camera)`.
    type Object = (i32, i16, (i32, i32, i32), (i32, i32));
    const OBJECTS: [Object; 3] = [
        (30, script::SCOUT_OBJECT, (3480, 31896, 545), (1006, 629)),
        (60, script::CAPITAL, (3168, 30816, 536), (1127, 521)),
        (90, script::BUILDER, (4008, 28296, 511), (1655, 394)),
    ];
    /// Each target: `(frame, the script's point, its ground height, its
    /// pixel under the opening camera, the ground the inverse returns
    /// there, the footprint's points and extent; its pixel under the
    /// closer camera, and the footprint's points there)`.
    type Target = (
        i32,
        (i32, i32),
        i32,
        (i32, i32),
        (i32, i32),
        usize,
        (i32, i32),
        (i32, i32),
        usize,
    );
    const TARGETS: [Target; 2] = [
        (
            30,
            (5880, 31896),
            313,
            (1384, 862),
            (5883, 31896),
            39,
            (9, 9),
            (960, 540),
            1,
        ),
        (
            90,
            (3552, 29856),
            526,
            (1338, 478),
            (3554, 29860),
            46,
            (10, 10),
            (960, 541),
            1,
        ),
    ];

    pub(crate) fn object(scene: &Scene, o: i16) -> (Pos, i32) {
        let u = scene.units.iter().find(|u| u.id.who == 0 && u.id.o == o);
        let b = scene
            .buildings
            .iter()
            .find(|b| b.id.who == 0 && b.id.o == o);
        u.map(|u| (u.pos, u.z))
            .or(b.map(|b| (b.pos, b.z)))
            .expect("the script's object is on the map")
    }

    /// **Stage 0, pinned**: each object's pixel and each target's round
    /// trip, on the order's frame, under the opening camera and the closer
    /// one; and the height rule the inverse stands on, every unit of the
    /// scene at its tile's height. Seen red: the Scout's pixel pinned at
    /// (1006, 628).
    #[test]
    fn minted_stage_zero_pixels_and_round_trips() {
        let mut r = match lobby() {
            Ok(r) => r,
            Err(e) => {
                println!("skipping: {e}");
                return;
            }
        };
        let orders = script::script();
        for (k, order) in orders.iter().enumerate() {
            while r.frame() < i64::from(order.0) {
                r.step();
            }
            let (scene, terrain) = (r.scene(None), r.terrain());
            let open = camera(None);
            let v = view(&open, &scene, &terrain);
            let stand = scene
                .units
                .iter()
                .filter(|u| v.ground_z(u.pos.x, u.pos.y) == u.z)
                .count();
            assert_eq!(stand, scene.units.len(), "frame {}", order.0);
            let (f, o, at, px) = OBJECTS[k];
            assert_eq!(f, order.0);
            let (pos, z) = object(&scene, o);
            assert_eq!((pos.x, pos.y, z), at, "frame {f}: object {o}");
            assert_eq!(
                v.pixel(pos.x, pos.y, z),
                px,
                "frame {f}: object {o}'s pixel"
            );
            for t in TARGETS.iter().filter(|t| t.0 == f) {
                let &(_, (x, y), tz, tp, back, n, ext, cp, cn) = t;
                assert_eq!(v.ground_z(x, y), tz);
                assert_eq!(v.pixel(x, y, tz), tp, "frame {f}: target's pixel");
                let b = ground_at(&v, tp.0, tp.1).unwrap();
                assert_eq!((b.x, b.y), back, "frame {f}: the inverse");
                let fp = footprint(&v, Pos { x, y }, 24);
                let span = |g: fn(&Pos) -> i32| {
                    fp.iter().map(g).max().unwrap() - fp.iter().map(g).min().unwrap() + 1
                };
                assert_eq!(fp.len(), n, "frame {f}: footprint");
                assert_eq!((span(|p| p.x), span(|p| p.y)), ext);
                assert!(fp.contains(&b), "the inverse is in the pixel's footprint");
                let close = camera(Some((Pos { x, y }, tz)));
                let cv = view(&close, &scene, &terrain);
                assert_eq!(cv.pixel(x, y, tz), cp, "frame {f}: target, closer");
                for p in [px, tp, cp] {
                    assert!(
                        (0..1920).contains(&p.0) && (0..1080).contains(&p.1),
                        "{p:?} off screen"
                    );
                }
                let cb = ground_at(&cv, cp.0, cp.1).unwrap();
                assert_eq!((cb.x, cb.y), (x, y), "frame {f}: the inverse, closer");
                assert_eq!(footprint(&cv, Pos { x, y }, 24).len(), cn);
            }
            r.issue(order);
        }
    }

    /// The events a person makes for the script, each at stage 0's pixel
    /// on the order's frame, the targets under `cam`'s camera.
    /// An event on a frame, under the opening camera or the closer one.
    pub(crate) type Scripted = (i32, Option<(Pos, i32)>, Event);

    ///
    /// The Scout's move is the minimap's (item 4194): a right press on it
    /// at the ground stage 0's inverse names under the event's camera — the
    /// script's point under the closer one, the clicked point under the
    /// opening one — whose `form`/`width` are the −1/−1 every file played
    /// before [`script::clicked_main_view`] carries. [`main_view_events`]
    /// is the same list with the move pressed on the main view.
    pub(crate) fn events(closer: bool) -> Vec<Scripted> {
        let o = |k: usize| OBJECTS[k].3;
        let (t0, t1) = (TARGETS[0], TARGETS[1]);
        let tp = |t: Target| {
            if closer { t.7 } else { t.3 }
        };
        let cam = |t: Target| closer.then_some((Pos { x: t.1.0, y: t.1.1 }, t.2));
        let named = if closer { t0.1 } else { t0.4 };
        vec![
            (
                30,
                None,
                Event::Press {
                    x: o(0).0,
                    y: o(0).1,
                },
            ),
            (
                30,
                cam(t0),
                Event::MapRightPress {
                    x: named.0,
                    y: named.1,
                },
            ),
            (
                60,
                None,
                Event::Press {
                    x: o(1).0,
                    y: o(1).1,
                },
            ),
            (60, None, Event::Key('c')),
            (
                90,
                None,
                Event::Press {
                    x: o(2).0,
                    y: o(2).1,
                },
            ),
            (90, None, Event::Key('f')),
            (
                90,
                cam(t1),
                Event::Press {
                    x: tp(t1).0,
                    y: tp(t1).1,
                },
            ),
        ]
    }

    /// **The clicked events on the main view** (item 4194): [`events`] at
    /// the opening camera with the Scout's move a right press at its
    /// target's pixel, (1384, 862), as the main view takes it.
    pub(crate) fn main_view_events() -> Vec<Scripted> {
        let mut e = events(false);
        let t0 = TARGETS[0];
        e[1] = (
            30,
            None,
            Event::RightPress {
                x: t0.3.0,
                y: t0.3.1,
            },
        );
        e
    }

    /// The events through the layer on the walk, each minted order issued
    /// to the walk as it is made; the orders, and the walk at frame 299.
    fn mint(closer: bool) -> Option<(Vec<Order>, FreeRunner)> {
        mint_events(events(closer))
    }

    /// [`mint`] on any list of events.
    fn mint_events(events: Vec<Scripted>) -> Option<(Vec<Order>, FreeRunner)> {
        mint_with(events, |input, frame, ev, view| {
            input.event(frame, ev, view)
        })
    }

    /// [`mint_events`] with each event handed to `through` (the layer
    /// alone, or the HUD and then the layer: item 4197).
    pub(crate) fn mint_with(
        events: Vec<Scripted>,
        mut through: impl FnMut(&mut Input, i32, Event, &View) -> Option<Order>,
    ) -> Option<(Vec<Order>, FreeRunner)> {
        let mut r = match lobby() {
            Ok(r) => r,
            Err(e) => {
                println!("skipping: {e}");
                return None;
            }
        };
        let mut input = Input::new(0);
        let mut out = Vec::new();
        for (frame, at, ev) in events {
            while r.frame() < i64::from(frame) {
                r.step();
            }
            // The issuing slot's own sight: what the fog hides is no
            // target (item 4202).
            let (scene, terrain) = (r.scene(Some(input.player)), r.terrain());
            let c = camera(at);
            if let Some(order) = through(&mut input, frame, ev, &view(&c, &scene, &terrain)) {
                r.issue(&order);
                out.push(order);
            }
        }
        while r.frame() < 299 {
            r.step();
        }
        Some((out, r))
    }

    /// Orders whose every field is the script's, and where one parts, each
    /// coordinate's distance from the script's.
    fn count(ours: &[Order]) -> (usize, Vec<(i32, i32, i32)>) {
        let script = script::script();
        assert_eq!(ours.len(), script.len(), "one order per scripted order");
        let mut parts = Vec::new();
        let whole = ours
            .iter()
            .zip(&script)
            .filter(|(a, b)| {
                let at = |o: &Order| match o.2[1] {
                    Command::MoveTo { to_x, to_y, .. } => Some((to_x, to_y)),
                    Command::Build { x, y, .. } => Some((x, y)),
                    _ => None,
                };
                if let (Some(p), Some(q)) = (at(a), at(b))
                    && p != q
                {
                    parts.push((a.0, p.0 - q.0, p.1 - q.1));
                }
                a == b
            })
            .count();
        (whole, parts)
    }

    /// **`INPUT_MINTED`** (`docs/CLIENT_INPUT.md` §3): the script's three orders
    /// minted from events at stage 0's pixels, the targets clicked under
    /// the closer camera, of 3. Was 0 (no input layer), now 3.
    const INPUT_MINTED: usize = 3;
    /// The same events with every target clicked under the camera the
    /// client opens with: the `queue_up` whole, the move and the build each
    /// one pixel's rounding off the script — `(frame, dx, dy)`.
    const INPUT_MINTED_OPENING: (usize, [(i32, i32, i32); 2]) = (1, [(30, 3, 0), (90, 2, 4)]);
    /// The minted file the original played (`docs/RECGAME.md` §9.1).
    pub(crate) const MINTED_RCX: &str =
        "ron-golden/recordings/item4059-minted-greatlakes-24000.rcx";
    const MINTED_RCX_BYTES: usize = 59_280;

    /// **The count**: synthetic events through the layer mint the script's
    /// orders field for field, the file written from them is the file the
    /// original played byte for byte, and the walk that took them stands
    /// where the script's effects put it on frame 299 (`docs/RECGAME.md`
    /// §9.3: the Scout on its point, the Farm's site `0/2007` at
    /// (3456, 29760)). Seen red: `INPUT_MINTED_OPENING`'s orders written in
    /// the file's place part from its bytes.
    #[test]
    fn minted_from_events_the_script_s_orders() {
        let Some((ours, r)) = mint(true) else { return };
        let (whole, parts) = count(&ours);
        assert_eq!((whole, parts), (INPUT_MINTED, vec![]), "{ours:?}");
        assert_eq!(ours, script::script());
        let scene = r.scene(None);
        let scout = object(&scene, script::SCOUT_OBJECT).0;
        assert_eq!((scout.x, scout.y), (script::SCOUT_TO.x, script::SCOUT_TO.y));
        let site = object(&scene, 2007).0;
        assert_eq!((site.x, site.y), (3456, 29760));

        let home = std::env::var_os("HOME").unwrap_or_default();
        let path = std::path::Path::new(&home).join(MINTED_RCX);
        let Ok(file) = std::fs::read(&path) else {
            println!("skipping the bytes: no {}", path.display());
            return;
        };
        assert_eq!(file.len(), MINTED_RCX_BYTES);
        let rec = rondata::recgame::read(&path.to_string_lossy()).unwrap();
        let written = |orders: &[Order]| {
            let lo = rondata::commands::decode(&rec.packages[0].data).unwrap();
            let last = rec.packages.last().unwrap().frame;
            let mut ours = rec.clone();
            ours.packages = rondata::recgame::minted_packages(lo[0].clone(), orders, last).unwrap();
            rondata::recgame::write_gz(&ours)
        };
        assert!(written(&ours) == file, "the file from the minted orders");
        let (opening, _) = mint(false).unwrap();
        assert!(
            written(&opening) != file,
            "the opening camera's orders are not the file's"
        );
    }

    /// The same events with the targets clicked under the opening camera:
    /// one order whole, the two with a coordinate each off by the pixel's
    /// rounding, the Farm's site still snapped to the script's.
    #[test]
    fn minted_from_events_under_the_opening_camera() {
        let Some((ours, r)) = mint(false) else { return };
        let (whole, parts) = count(&ours);
        assert_eq!(
            (whole, parts),
            (INPUT_MINTED_OPENING.0, INPUT_MINTED_OPENING.1.to_vec())
        );
        let scene = r.scene(None);
        let site = object(&scene, 2007).0;
        assert_eq!((site.x, site.y), (3456, 29760));
    }

    /// **`INPUT_CLICKED`** (`docs/CLIENT_INPUT.md` §5, item 4149; §9, item
    /// 4194): the same seven events, every target clicked under the
    /// opening camera at the pixel the script's own target projects to and
    /// the Scout's move pressed on the main view, mint
    /// [`script::clicked_main_view`] field for field, of 3. Was 0 (no
    /// clicked script); 3 of 3 on [`script::clicked`] while one unit's move
    /// kept the minimap's −1/−1; 3 of 3 on the main view's file since item
    /// 4194, where [`script::clicked`] counts 2 (the move's `form`/`width`).
    const INPUT_CLICKED: usize = 3;
    /// [`script::clicked`] against the main view's events: the move parts
    /// on `form`/`width` alone (item 4194; "fails today" was the layer
    /// minting −1/−1).
    const INPUT_CLICKED_MINIMAP_ON_MAIN: usize = 2;
    /// The clicked file (`docs/CLIENT_INPUT.md` §5): the minted file's
    /// lobby, `leader_options` and closing frame with the clicked orders,
    /// written by rondata's `the_clicked_minted_packages_round_trip` under
    /// `RON_RCX_OUT`; the file the original's playback takes open (run
    /// 1563).
    pub(crate) const CLICKED_RCX: &str =
        "ron-golden/recordings/item4149-clicked-greatlakes-24000.rcx";
    pub(crate) const CLICKED_RCX_BYTES: usize = 59_280;
    /// The main view's clicked file (item 4194, `docs/CLIENT_INPUT.md` §9):
    /// [`CLICKED_RCX`] with the move's `form`/`width` (0, 50), written by
    /// rondata's `the_clicked_main_view_minted_packages_round_trip` under
    /// `RON_RCX_OUT`; the file run1572 takes open.
    pub(crate) const CLICKED_MAIN_RCX: &str =
        "ron-golden/recordings/item4194-clicked-main-view-greatlakes-24000.rcx";
    pub(crate) const CLICKED_MAIN_RCX_BYTES: usize = 59_278;

    /// The file the lobby's recording makes with `orders`' packages.
    fn written(orders: &[Order]) -> Option<Vec<u8>> {
        let home = std::env::var_os("HOME").unwrap_or_default();
        let path = std::path::Path::new(&home).join(MINTED_RCX);
        let rec = rondata::recgame::read(&path.to_string_lossy()).ok()?;
        let lo = rondata::commands::decode(&rec.packages[0].data).unwrap();
        let last = rec.packages.last().unwrap().frame;
        let mut w = rec.clone();
        w.packages = rondata::recgame::minted_packages(lo[0].clone(), orders, last).unwrap();
        Some(rondata::recgame::write_gz(&w))
    }

    /// **The clicked count**: the main view's events at the opening camera
    /// mint the main view's clicked script, every field; the walk that took
    /// them stands on frame 299 with the Scout on its clicked point and the
    /// Farm's site `0/2007` where the first script's stands; and the file
    /// written from them is the main view's clicked file byte for byte, not
    /// the played clicked file. The minimap's list mints [`script::clicked`]
    /// whole and writes the played file (item 4194).
    #[test]
    fn minted_clicked_at_the_opening_camera() {
        let Some((ours, r)) = mint_events(main_view_events()) else {
            return;
        };
        let main = script::clicked_main_view();
        let whole = |a: &[Order], b: &[Order]| a.iter().zip(b).filter(|(a, b)| a == b).count();
        assert_eq!(
            (ours.len(), whole(&ours, &main)),
            (main.len(), INPUT_CLICKED),
            "{ours:?}"
        );
        assert_eq!(ours, main);
        assert_eq!(
            whole(&ours, &script::clicked()),
            INPUT_CLICKED_MINIMAP_ON_MAIN,
            "the played clicked script against the main view's events"
        );
        let scene = r.scene(None);
        let scout = object(&scene, script::SCOUT_OBJECT).0;
        assert_eq!(
            (scout.x, scout.y),
            (script::CLICKED_SCOUT_TO.x, script::CLICKED_SCOUT_TO.y)
        );
        let site = object(&scene, 2007).0;
        assert_eq!((site.x, site.y), (3456, 29760));
        let (minimap, _) = mint(false).unwrap();
        assert_eq!(minimap, script::clicked(), "the minimap's list");

        let home = std::env::var_os("HOME").unwrap_or_default();
        let read = |p: &str| std::fs::read(std::path::Path::new(&home).join(p));
        let (Ok(played), Ok(file)) = (read(CLICKED_RCX), read(CLICKED_MAIN_RCX)) else {
            println!("skipping the bytes: no {CLICKED_RCX} or {CLICKED_MAIN_RCX}");
            return;
        };
        assert_eq!(
            (played.len(), file.len()),
            (CLICKED_RCX_BYTES, CLICKED_MAIN_RCX_BYTES)
        );
        let w = written(&ours).unwrap();
        assert!(w == file, "the file from the main view's clicked orders");
        assert!(w != played, "the main view's file is not the played one");
        assert!(
            written(&minimap).unwrap() == played,
            "the minimap's list writes the played clicked file"
        );
    }

    /// **Stage 0 of the right presses** (`docs/CLIENT_INPUT.md` §6), on each
    /// press's frame of the walk under the clicked script, under the opening
    /// camera: `(frame, the unit pressed, its anchor's pixel, the building
    /// right-pressed, its anchor's pixel, what the layer reads there)`.
    pub(crate) type Right = (i32, i16, (i32, i32), i16, (i32, i32), Hit);
    pub(crate) const RIGHT: [Right; 2] = [
        (
            script::RIGHT_BUILD_FRAME,
            script::HELPER,
            (1678, 468),
            script::FARM_SITE,
            (1338, 463),
            Hit::Own {
                id: ObjId {
                    who: 0,
                    o: script::FARM_SITE,
                },
                active: false,
                damaged: true,
                room: Some(true),
            },
        ),
        (
            script::RIGHT_GATHER_FRAME,
            script::TRAINED,
            (976, 602),
            script::WOODCUTTER,
            (1640, 438),
            Hit::Own {
                id: ObjId {
                    who: 0,
                    o: script::WOODCUTTER,
                },
                active: true,
                damaged: false,
                room: Some(true),
            },
        ),
    ];

    /// The clicked script's seven events, then each right press: a press on
    /// its unit and a right press on its building, at [`RIGHT`]'s pixels.
    pub(crate) fn right_events() -> Vec<Scripted> {
        let mut e = events(false);
        for (frame, _, unit, _, building, _) in RIGHT {
            e.push((
                frame,
                None,
                Event::Press {
                    x: unit.0,
                    y: unit.1,
                },
            ));
            e.push((
                frame,
                None,
                Event::RightPress {
                    x: building.0,
                    y: building.1,
                },
            ));
        }
        e
    }

    /// **`INPUT_RIGHT_PRESSED`** (`docs/CLIENT_INPUT.md` §6, item 4179): the
    /// clicked script's events and two right presses on objects, under the
    /// opening camera, mint [`script::right_pressed`] field for field, of 5.
    /// Was 0 (no right press on an object).
    const INPUT_RIGHT_PRESSED: usize = 5;

    /// **The right-pressed count**: stage 0's pixels pinned on each press's
    /// frame (each anchor's, and what the layer reads under the building's),
    /// the events mint the right-pressed script, and on 299 the walk that
    /// took them holds the helper's and the trained Citizen's orders as
    /// the orders put them.
    #[test]
    fn minted_right_pressed_at_the_opening_camera() {
        let Ok(mut r) = lobby() else {
            println!("skipping: no lobby");
            return;
        };
        let clicked = script::clicked();
        let mut next = 0;
        let mut stage = Vec::new();
        for (frame, unit, _, building, _, _) in RIGHT {
            while r.frame() < i64::from(frame) {
                while next < clicked.len() && i64::from(clicked[next].0) == r.frame() {
                    r.issue(&clicked[next]);
                    next += 1;
                }
                r.step();
            }
            let (scene, terrain) = (r.scene(None), r.terrain());
            let c = camera(None);
            let v = view(&c, &scene, &terrain);
            let (u, uz) = object(&scene, unit);
            let (b, bz) = object(&scene, building);
            let up = v.pixel(u.x, u.y, uz);
            let bp = v.pixel(b.x, b.y, bz);
            stage.push((
                frame,
                unit,
                up,
                building,
                bp,
                v.target(0, bp.0, bp.1).unwrap(),
            ));
        }
        assert_eq!(stage, RIGHT.to_vec(), "stage 0 of the right presses");

        let Some((ours, r)) = mint_events(right_events()) else {
            return;
        };
        let right = script::right_pressed();
        let whole = ours.iter().zip(&right).filter(|(a, b)| a == b).count();
        assert_eq!(
            (ours.len(), whole),
            (right.len(), INPUT_RIGHT_PRESSED),
            "{ours:?}"
        );
        assert_eq!(ours, right);
        drop(r);

        let home = std::env::var_os("HOME").unwrap_or_default();
        let (Ok(clicked_file), Ok(file)) = (
            std::fs::read(std::path::Path::new(&home).join(CLICKED_RCX)),
            std::fs::read(std::path::Path::new(&home).join(RIGHT_PRESSED_RCX)),
        ) else {
            println!("skipping the bytes: no {CLICKED_RCX} or {RIGHT_PRESSED_RCX}");
            return;
        };
        assert_eq!(file.len(), RIGHT_PRESSED_RCX_BYTES);
        let path = std::path::Path::new(&home).join(MINTED_RCX);
        let rec = rondata::recgame::read(&path.to_string_lossy()).unwrap();
        let lo = rondata::commands::decode(&rec.packages[0].data).unwrap();
        let last = rec.packages.last().unwrap().frame;
        let mut written = rec.clone();
        written.packages = rondata::recgame::minted_packages(lo[0].clone(), &ours, last).unwrap();
        let written = rondata::recgame::write_gz(&written);
        assert!(written == file, "the file from the right-pressed orders");
        assert!(written != clicked_file, "not the clicked file");
    }

    /// The right-pressed file (`docs/CLIENT_INPUT.md` §6): the clicked file's
    /// lobby, `leader_options` and closing frame with the right-pressed
    /// orders, written by rondata's `the_right_pressed_minted_packages_round_trip`
    /// under `RON_RCX_OUT`; the file the original's playback takes open
    /// (run 1568).
    const RIGHT_PRESSED_RCX: &str =
        "ron-golden/recordings/item4179-right-pressed-greatlakes-24000.rcx";
    const RIGHT_PRESSED_RCX_BYTES: usize = 59_312;

    /// **Stage 0 of the box** (item 4180, `docs/CLIENT_INPUT.md` §7.2), on
    /// frame 240 of the walk under the right-pressed script, at the
    /// opening camera: the drag's corners, each boxed unit's anchor, the
    /// nearest anchor of slot 0's other units to the grown rectangle, the
    /// right press's pixel and the ground the layer reads there.
    const BOX_DRAG: ((i32, i32), (i32, i32)) = ((740, 450), (910, 610));
    const BOX_ANCHORS: [(i16, (i32, i32)); 3] = [(3, (886, 587)), (4, (765, 522)), (5, (795, 473))];
    /// The nearest other unit of slot 0 on 240, and its anchor: `0/6`,
    /// walking to the Woodcutter's camp, 245 px right of the grown box.
    const BOX_NEAREST_OUT: (i16, (i32, i32)) = (6, (1185, 616));
    const BOX_RIGHT: (i32, i32) = (700, 600);

    /// The right-pressed events, then the box: a drag round [`BOX_ANCHORS`]
    /// and a right press on the ground at [`BOX_RIGHT`].
    fn box_events() -> Vec<Scripted> {
        let mut e = right_events();
        let ((x0, y0), (x1, y1)) = BOX_DRAG;
        e.push((script::BOX_FRAME, None, Event::Drag { x0, y0, x1, y1 }));
        e.push((
            script::BOX_FRAME,
            None,
            Event::RightPress {
                x: BOX_RIGHT.0,
                y: BOX_RIGHT.1,
            },
        ));
        e
    }

    /// **`INPUT_BOXED`** (item 4180): the right-pressed events and a box
    /// moved, under the opening camera, mint [`script::boxed`] field for
    /// field, of 6. Was 0 (no box, no group of more than one unit).
    const INPUT_BOXED: usize = 6;

    /// **The boxed count**: stage 0's pixels pinned on 240, the box takes
    /// the three Citizens in object order and nothing else, the right press
    /// reads the ground and their formation, and the events mint the boxed
    /// script; the file written from the orders is the boxed file. Seen
    /// red: the script's `BOX_WIDTH` set to −1 gave 5 of 6.
    #[test]
    fn minted_boxed_at_the_opening_camera() {
        let Ok(mut r) = lobby() else {
            println!("skipping: no lobby");
            return;
        };
        let right = script::right_pressed();
        let mut next = 0;
        while r.frame() < i64::from(script::BOX_FRAME) {
            while next < right.len() && i64::from(right[next].0) == r.frame() {
                r.issue(&right[next]);
                next += 1;
            }
            r.step();
        }
        let (scene, terrain) = (r.scene(None), r.terrain());
        let c = camera(None);
        let v = view(&c, &scene, &terrain);
        let anchors: Vec<_> = script::BOXED
            .iter()
            .map(|&o| {
                let (p, z) = object(&scene, o);
                (o, v.pixel(p.x, p.y, z))
            })
            .collect();
        assert_eq!(anchors, BOX_ANCHORS.to_vec(), "the boxed anchors");
        let ((x0, y0), (x1, y1)) = BOX_DRAG;
        let grown = |(x, y): (i32, i32)| {
            (x - (x0.min(x1) - BOX_MARGIN))
                .min(x0.max(x1) + BOX_MARGIN - x)
                .min((y - (y0.min(y1) - BOX_MARGIN)).min(y0.max(y1) + BOX_MARGIN - y))
        };
        let nearest = scene
            .units
            .iter()
            .filter(|u| u.id.who == 0 && !script::BOXED.contains(&u.id.o))
            .map(|u| (u.id.o, v.pixel(u.pos.x, u.pos.y, u.z)))
            .max_by_key(|&(_, p)| grown(p))
            .unwrap();
        assert_eq!(nearest, BOX_NEAREST_OUT, "the nearest unit left out");
        assert!(grown(nearest.1) < 0, "outside the grown box");
        let ids: Vec<i16> = v
            .boxed(0, BOX_DRAG.0, BOX_DRAG.1)
            .iter()
            .map(|id| id.o)
            .collect();
        assert_eq!(ids, script::BOXED.to_vec(), "the box takes the three");
        let members: Vec<_> = script::BOXED
            .iter()
            .map(|&o| {
                scene
                    .units
                    .iter()
                    .find(|u| u.id.who == 0 && u.id.o == o)
                    .unwrap()
            })
            .map(|u| (u.type_index, u.form, u.form_width))
            .collect();
        assert_eq!(members, vec![(50, 9, -1); 3], "TI, form, form_width");
        assert_eq!(v.target(0, BOX_RIGHT.0, BOX_RIGHT.1), None, "the ground");
        let at = ground_at(&v, BOX_RIGHT.0, BOX_RIGHT.1).unwrap();
        assert_eq!(at, script::BOX_TO, "the ground under the right press");
        drop(r);

        let Some((ours, r)) = mint_events(box_events()) else {
            return;
        };
        let boxed = script::boxed();
        let whole = ours.iter().zip(&boxed).filter(|(a, b)| a == b).count();
        assert_eq!((ours.len(), whole), (boxed.len(), INPUT_BOXED), "{ours:?}");
        assert_eq!(ours, boxed);
        drop(r);

        let home = std::env::var_os("HOME").unwrap_or_default();
        let (Ok(right_file), Ok(file)) = (
            std::fs::read(std::path::Path::new(&home).join(RIGHT_PRESSED_RCX)),
            std::fs::read(std::path::Path::new(&home).join(BOXED_RCX)),
        ) else {
            println!("skipping the bytes: no {RIGHT_PRESSED_RCX} or {BOXED_RCX}");
            return;
        };
        assert_eq!(file.len(), BOXED_RCX_BYTES);
        let path = std::path::Path::new(&home).join(MINTED_RCX);
        let rec = rondata::recgame::read(&path.to_string_lossy()).unwrap();
        let lo = rondata::commands::decode(&rec.packages[0].data).unwrap();
        let last = rec.packages.last().unwrap().frame;
        let mut written = rec.clone();
        written.packages = rondata::recgame::minted_packages(lo[0].clone(), &ours, last).unwrap();
        let written = rondata::recgame::write_gz(&written);
        assert!(written == file, "the file from the boxed orders");
        assert!(written != right_file, "not the right-pressed file");
    }

    /// The boxed file (`docs/CLIENT_INPUT.md` §7.3), written by rondata's
    /// `the_boxed_minted_packages_round_trip` under `RON_RCX_OUT`; the file
    /// the original's playback takes open (run 1569).
    const BOXED_RCX: &str = "ron-golden/recordings/item4180-boxed-greatlakes-24000.rcx";
    const BOXED_RCX_BYTES: usize = 59_338;

    /// **Stage 0 of the attacked script** (item 4195, `docs/CLIENT_INPUT.md`
    /// §8.2): on each package's frame of the walk under
    /// [`script::attacked`], the anchor each press and the right press land
    /// on at the opening camera, `(frame, (who, o), pixel)`. Every one is on
    /// the 1920×1080 screen, so the script's camera is the client's own.
    /// An anchor of the attacked script: `(frame, (who, o), pixel)`.
    type AttackAnchor = (i32, (u8, i16), (i32, i32));
    const ATTACK_ANCHORS: [AttackAnchor; 5] = [
        (
            script::ATTACK_RESEARCH_FRAME,
            (0, script::LIBRARY_OBJECT),
            (1006, 397),
        ),
        (
            script::BARRACKS_FRAME,
            (0, script::BARRACKS_BUILDER),
            (735, 523),
        ),
        (
            script::TRAIN_FRAME,
            (0, script::BARRACKS_OBJECT),
            (614, 458),
        ),
        (script::ATTACK_FRAME, (0, script::SLINGER), (509, 513)),
        (script::ATTACK_FRAME, script::ATTACKED, (1645, 884)),
    ];
    /// The Barracks' drop pixel: the pixel of (1248, 32160) at the opening
    /// camera, whose ground under the layer's inverse is
    /// [`script::BARRACKS_AT`].
    const BARRACKS_DROP: (i32, i32) = (614, 475);

    /// The boxed events, then the fifth script's: a press on the Library
    /// and `a`; a press on the idle Citizen, `b` and a press at the drop
    /// pixel; a press on the Barracks and `s`; a press on the Slinger and a
    /// right press on slot 1's Scout.
    fn attack_events() -> Vec<Scripted> {
        let mut e = box_events();
        let press = |k: usize| {
            let (f, _, (x, y)) = ATTACK_ANCHORS[k];
            (f, None, Event::Press { x, y })
        };
        let (fa, _, (ax, ay)) = ATTACK_ANCHORS[4];
        e.extend([
            press(0),
            (script::ATTACK_RESEARCH_FRAME, None, Event::Key('a')),
            press(1),
            (script::BARRACKS_FRAME, None, Event::Key('b')),
            (
                script::BARRACKS_FRAME,
                None,
                Event::Press {
                    x: BARRACKS_DROP.0,
                    y: BARRACKS_DROP.1,
                },
            ),
            press(2),
            (script::TRAIN_FRAME, None, Event::Key('s')),
            press(3),
            (fa, None, Event::RightPress { x: ax, y: ay }),
        ]);
        e
    }

    /// **`INPUT_ATTACKED`** (item 4195): the boxed events and the fifth
    /// script's, under the opening camera, mint [`script::attacked`] field
    /// for field, of 10. Was 0 (no file carried an attack).
    const INPUT_ATTACKED: usize = 10;

    /// **The attacked count**: stage 0's anchors pinned on each package's
    /// frame, the drop pixel's ground, and what the right press finds under
    /// it — slot 1's Scout, an enemy unit, and nothing else within the pick;
    /// then the events mint the attacked script, and the file written from
    /// the orders is the attacked file. Seen red: the script's attack
    /// `ignore` set to 1 gave 9 of 10.
    #[test]
    fn minted_attacked_at_the_opening_camera() {
        let Ok(mut r) = lobby() else {
            println!("skipping: no lobby");
            return;
        };
        let attacked = script::attacked();
        let c = camera(None);
        let mut next = 0;
        let mut anchors = Vec::new();
        for f in 0..=script::ATTACK_FRAME {
            if ATTACK_ANCHORS.iter().any(|a| a.0 == f) {
                let (scene, terrain) = (r.scene(Some(0)), r.terrain());
                let v = view(&c, &scene, &terrain);
                for &(af, (who, o), _) in ATTACK_ANCHORS.iter().filter(|a| a.0 == f) {
                    let u = scene.units.iter().find(|u| u.id == ObjId { who, o });
                    let b = scene.buildings.iter().find(|b| b.id == ObjId { who, o });
                    let (p, z) = u.map(|u| (u.pos, u.z)).or(b.map(|b| (b.pos, b.z))).unwrap();
                    anchors.push((af, (who, o), v.pixel(p.x, p.y, z)));
                }
                if f == script::BARRACKS_FRAME {
                    let p = Pos { x: 1248, y: 32160 };
                    assert_eq!(v.pixel(p.x, p.y, v.ground_z(p.x, p.y)), BARRACKS_DROP);
                    let at = ground_at(&v, BARRACKS_DROP.0, BARRACKS_DROP.1);
                    assert_eq!(at, Some(script::BARRACKS_AT), "the drop pixel's ground");
                }
                if f == script::ATTACK_FRAME {
                    let (_, (who, o), (x, y)) = ATTACK_ANCHORS[4];
                    assert_eq!(
                        v.target(0, x, y),
                        Some(Hit::Enemy {
                            id: ObjId { who, o },
                            unit: true
                        }),
                        "the right press finds slot 1's Scout"
                    );
                    let slinger = scene
                        .units
                        .iter()
                        .find(|u| u.id.who == 0 && u.id.o == script::SLINGER);
                    assert_eq!(slinger.map(|u| u.type_index), Some(script::SLINGERS_TYPE));
                    let scout = scene
                        .units
                        .iter()
                        .find(|u| u.id == ObjId { who, o })
                        .unwrap();
                    assert_eq!((scout.type_index, scout.health), (69, 50), "TI, hp");
                }
            }
            while next < attacked.len() && attacked[next].0 == f {
                r.issue(&attacked[next]);
                next += 1;
            }
            r.step();
        }
        assert_eq!(anchors, ATTACK_ANCHORS.to_vec(), "stage 0's anchors");
        drop(r);

        let Some((ours, r)) = mint_events(attack_events()) else {
            return;
        };
        let whole = ours.iter().zip(&attacked).filter(|(a, b)| a == b).count();
        assert_eq!(
            (ours.len(), whole),
            (attacked.len(), INPUT_ATTACKED),
            "{ours:?}"
        );
        assert_eq!(ours, attacked);
        drop(r);

        let home = std::env::var_os("HOME").unwrap_or_default();
        let (Ok(boxed_file), Ok(file)) = (
            std::fs::read(std::path::Path::new(&home).join(BOXED_RCX)),
            std::fs::read(std::path::Path::new(&home).join(ATTACKED_RCX)),
        ) else {
            println!("skipping the bytes: no {BOXED_RCX} or {ATTACKED_RCX}");
            return;
        };
        assert_eq!(file.len(), ATTACKED_RCX_BYTES);
        let path = std::path::Path::new(&home).join(MINTED_RCX);
        let rec = rondata::recgame::read(&path.to_string_lossy()).unwrap();
        let lo = rondata::commands::decode(&rec.packages[0].data).unwrap();
        let last = rec.packages.last().unwrap().frame;
        let mut written = rec.clone();
        written.packages = rondata::recgame::minted_packages(lo[0].clone(), &ours, last).unwrap();
        let written = rondata::recgame::write_gz(&written);
        assert!(written == file, "the file from the attacked orders");
        assert!(written != boxed_file, "not the boxed file");
    }

    /// The attacked file (`docs/CLIENT_INPUT.md` §8), written by rondata's
    /// `the_attacked_minted_packages_round_trip` under `RON_RCX_OUT`; the
    /// file the original's playback takes open (run 1571).
    const ATTACKED_RCX: &str = "ron-golden/recordings/item4195-attacked-greatlakes-24000.rcx";
    const ATTACKED_RCX_BYTES: usize = 59_404;

    /// **Stage 0 of the followed script** (item 4202, `docs/CLIENT_INPUT.md`
    /// §10): on the fog press's frame and the follow's, under
    /// [`script::followed`], the anchor each press and right press lands on
    /// at the opening camera, `(frame, (who, o), pixel)`.
    const FOLLOW_ANCHORS: [AttackAnchor; 4] = [
        (script::FOG_FRAME, (0, script::SCOUT_OBJECT), (1384, 862)),
        (script::FOG_FRAME, script::ATTACKED, (1652, 882)),
        (script::FOLLOW_FRAME, (0, script::SCOUT_OBJECT), (1399, 860)),
        (script::FOLLOW_FRAME, script::ATTACKED, (1637, 883)),
    ];

    /// The attacked events with the Scout's two presses: on
    /// [`script::FOG_FRAME`] a press on it and a right press on `1/0`, which
    /// the fog hides; on [`script::FOLLOW_FRAME`] the same on the seen
    /// `1/0`.
    fn follow_events() -> Vec<Scripted> {
        let mut e = attack_events();
        let ev = |k: usize, right: bool| {
            let (f, _, (x, y)) = FOLLOW_ANCHORS[k];
            let ev = if right {
                Event::RightPress { x, y }
            } else {
                Event::Press { x, y }
            };
            (f, None, ev)
        };
        let at = e
            .iter()
            .position(|s| s.0 == script::ATTACK_FRAME)
            .expect("the attack's press");
        e.splice(at..at, [ev(0, false), ev(1, true)]);
        e.extend([ev(2, false), ev(3, true)]);
        e
    }

    /// **`INPUT_FOLLOWED`** (item 4202): the attacked events and the
    /// Scout's four, under the opening camera, mint [`script::followed`]
    /// field for field, of 12. Was 10: the layer minted the fog press as an
    /// attack, wanting `is_seen`, and the follow as an attack, wanting
    /// `COUNT_ATTACK`.
    const INPUT_FOLLOWED: usize = 12;

    /// **The followed count**: stage 0's anchors on the two frames; on the
    /// fog press's frame `1/0` is under the pixel and unseen, so the right
    /// press finds nothing and names the ground; on the follow's it is
    /// seen, and the Scout has no attack; then the events mint the followed
    /// script, and the file written from them is the followed file.
    #[test]
    fn minted_followed_at_the_opening_camera() {
        let Ok(mut r) = lobby() else {
            println!("skipping: no lobby");
            return;
        };
        let followed = script::followed();
        let c = camera(None);
        let mut next = 0;
        let mut anchors = Vec::new();
        for f in 0..=script::FOLLOW_FRAME {
            if FOLLOW_ANCHORS.iter().any(|a| a.0 == f) {
                let (scene, terrain) = (r.scene(Some(0)), r.terrain());
                let v = view(&c, &scene, &terrain);
                for &(af, (who, o), _) in FOLLOW_ANCHORS.iter().filter(|a| a.0 == f) {
                    let u = scene
                        .units
                        .iter()
                        .find(|u| u.id == ObjId { who, o })
                        .unwrap();
                    anchors.push((af, (who, o), v.pixel(u.pos.x, u.pos.y, u.z)));
                }
                let (who, o) = script::ATTACKED;
                let enemy = scene
                    .units
                    .iter()
                    .find(|u| u.id == ObjId { who, o })
                    .unwrap();
                let scout = scene
                    .units
                    .iter()
                    .find(|u| {
                        u.id == ObjId {
                            who: 0,
                            o: script::SCOUT_OBJECT,
                        }
                    })
                    .unwrap();
                let (ex, ey) = v.pixel(enemy.pos.x, enemy.pos.y, enemy.z);
                println!(
                    "FOLLOW STAGE0 f{f} enemy {:?} seen {} scout {:?} attacks {} target {:?} ground {:?} form {:?}",
                    enemy.pos,
                    enemy.seen,
                    scout.pos,
                    scout.attacks,
                    v.target(0, ex, ey),
                    ground_at(&v, ex, ey),
                    v.formation(&[scout.id])
                );
                assert_eq!(scout.type_index, 69, "the Scout");
                assert!(!scout.attacks, "a Scout has no attack");
                if f == script::FOG_FRAME {
                    assert!(!enemy.seen, "the fog hides 1/0");
                    assert_eq!(v.target(0, ex, ey), None, "the press finds nothing");
                    assert_eq!(ground_at(&v, ex, ey), Some(script::FOG_TO));
                    // Seen, the same pixel would draw the follow: the fog
                    // test is what turns it into a move.
                    let mut lit = scene.clone();
                    lit.units.iter_mut().for_each(|u| u.seen = true);
                    let lv = view(&c, &lit, &terrain);
                    assert_eq!(
                        lv.target(0, ex, ey),
                        Some(Hit::Enemy {
                            id: enemy.id,
                            unit: true
                        })
                    );
                } else {
                    assert!(enemy.seen, "1/0 is seen");
                    assert_eq!(
                        v.target(0, ex, ey),
                        Some(Hit::Enemy {
                            id: enemy.id,
                            unit: true
                        })
                    );
                }
            }
            while next < followed.len() && followed[next].0 == f {
                r.issue(&followed[next]);
                next += 1;
            }
            r.step();
        }
        assert_eq!(anchors, FOLLOW_ANCHORS.to_vec(), "stage 0's anchors");
        drop(r);

        let Some((ours, r)) = mint_events(follow_events()) else {
            return;
        };
        let whole = ours.iter().zip(&followed).filter(|(a, b)| a == b).count();
        assert_eq!(
            (ours.len(), whole),
            (followed.len(), INPUT_FOLLOWED),
            "{ours:?}"
        );
        assert_eq!(ours, followed);
        drop(r);

        let home = std::env::var_os("HOME").unwrap_or_default();
        let (Ok(attacked_file), Ok(file)) = (
            std::fs::read(std::path::Path::new(&home).join(ATTACKED_RCX)),
            std::fs::read(std::path::Path::new(&home).join(FOLLOWED_RCX)),
        ) else {
            println!("skipping the bytes: no {ATTACKED_RCX} or {FOLLOWED_RCX}");
            return;
        };
        assert_eq!(file.len(), FOLLOWED_RCX_BYTES);
        let written = written(&ours).unwrap();
        assert!(written == file, "the file from the followed orders");
        assert!(written != attacked_file, "not the attacked file");
    }

    /// **Stage 0 of the gathered script** (item 4249): on
    /// [`script::GATHER_FRAME`] the capital's anchor, and the bare ground
    /// pixel the right press names, at the opening camera.
    const GATHER_ANCHORS: [AttackAnchor; 2] = [
        (script::GATHER_FRAME, (0, script::CAPITAL), (1127, 521)),
        (script::GATHER_FRAME, (0, -1), (1277, 611)),
    ];

    /// The followed events and the capital's three: on
    /// [`script::GATHER_FRAME`] a press on it and a right press on the
    /// ground; on [`script::GATHER_TRAIN_FRAME`] the key `c`.
    fn gather_events() -> Vec<Scripted> {
        let mut e = follow_events();
        let (f, _, (cx, cy)) = GATHER_ANCHORS[0];
        let (_, _, (gx, gy)) = GATHER_ANCHORS[1];
        e.extend([
            (f, None, Event::Press { x: cx, y: cy }),
            (f, None, Event::RightPress { x: gx, y: gy }),
            (script::GATHER_TRAIN_FRAME, None, Event::Key('c')),
        ]);
        e
    }

    /// **`INPUT_GATHERED`** (item 4249): the followed events and the
    /// capital's three, under the opening camera, mint [`script::gathered`]
    /// field for field, of 14. Seen red with the building arm minting
    /// nothing, as the layer did before: 13 orders, 12 whole in place.
    const INPUT_GATHERED: usize = 14;

    /// **The gathered count**: stage 0's anchors on the rally point's
    /// frame — the capital trains, and no object is under the ground's
    /// pixel, whose inverse is [`script::GATHER_AT`]; then the events mint
    /// the gathered script, and the file written from them is the gathered
    /// file.
    #[test]
    fn minted_gathered_at_the_opening_camera() {
        let Ok(mut r) = lobby() else {
            println!("skipping: no lobby");
            return;
        };
        let gathered = script::gathered();
        let c = camera(None);
        let mut next = 0;
        for f in 0..=script::GATHER_FRAME {
            if f == script::GATHER_FRAME {
                let (scene, terrain) = (r.scene(Some(0)), r.terrain());
                let v = view(&c, &scene, &terrain);
                let id = ObjId {
                    who: 0,
                    o: script::CAPITAL,
                };
                let cap = scene.buildings.iter().find(|b| b.id == id).unwrap();
                let (gx, gy) = GATHER_ANCHORS[1].2;
                println!(
                    "GATHER STAGE0 f{f} capital {:?} z {} pixel {:?} trains {} ground {:?} names {}",
                    cap.pos,
                    cap.z,
                    v.pixel(cap.pos.x, cap.pos.y, cap.z),
                    cap.trains,
                    ground_at(&v, gx, gy),
                    v.names_object(gx, gy)
                );
                assert_eq!(v.pixel(cap.pos.x, cap.pos.y, cap.z), GATHER_ANCHORS[0].2);
                assert!(v.takes_gather_point(&[id]), "the capital trains");
                assert!(!v.names_object(gx, gy), "bare ground");
                assert_eq!(ground_at(&v, gx, gy), Some(script::GATHER_AT));
            }
            while next < gathered.len() && gathered[next].0 == f {
                r.issue(&gathered[next]);
                next += 1;
            }
            r.step();
        }
        drop(r);

        let Some((ours, r)) = mint_events(gather_events()) else {
            return;
        };
        let whole = ours.iter().zip(&gathered).filter(|(a, b)| a == b).count();
        assert_eq!(
            (ours.len(), whole),
            (gathered.len(), INPUT_GATHERED),
            "{ours:?}"
        );
        assert_eq!(ours, gathered);
        drop(r);

        let home = std::env::var_os("HOME").unwrap_or_default();
        let (Ok(followed_file), Ok(file)) = (
            std::fs::read(std::path::Path::new(&home).join(FOLLOWED_RCX)),
            std::fs::read(std::path::Path::new(&home).join(GATHERED_RCX)),
        ) else {
            println!("skipping the bytes: no {FOLLOWED_RCX} or {GATHERED_RCX}");
            return;
        };
        assert_eq!(file.len(), GATHERED_RCX_BYTES);
        let written = written(&ours).unwrap();
        assert!(written == file, "the file from the gathered orders");
        assert!(written != followed_file, "not the followed file");
    }

    /// The gathered file (`docs/CLIENT_INPUT.md` §11), written by rondata's
    /// `the_gathered_minted_packages_round_trip` under `RON_RCX_OUT`; the
    /// file the original's playback takes open (run 1598).
    const GATHERED_RCX: &str = "ron-golden/recordings/item4249-gathered-greatlakes-24000.rcx";
    const GATHERED_RCX_BYTES: usize = 59_459;

    /// The followed file (`docs/CLIENT_INPUT.md` §10), written by rondata's
    /// `the_followed_minted_packages_round_trip` under `RON_RCX_OUT`; the
    /// file the original's playback takes open (run 1573).
    const FOLLOWED_RCX: &str = "ron-golden/recordings/item4202-followed-greatlakes-24000.rcx";
    const FOLLOWED_RCX_BYTES: usize = 59_435;

    /// A line typed into the entry line and sent: Return, each character,
    /// Return.
    fn typed(frame: i32, line: &str) -> Vec<Scripted> {
        let mut e = vec![(frame, None, Event::Enter)];
        e.extend(line.chars().map(|c| (frame, None, Event::Key(c))));
        e.push((frame, None, Event::Enter));
        e
    }

    /// The gathered events and the line [`script::CHAT_TEXT`] typed on
    /// [`script::CHAT_FRAME`] (item 4250).
    fn chat_events() -> Vec<Scripted> {
        let mut e = gather_events();
        e.extend(typed(script::CHAT_FRAME, script::CHAT_TEXT));
        e
    }

    /// **`INPUT_CHATTED`** (item 4250, `docs/CLIENT_INPUT.md` §12): the
    /// gathered events and the typed line, under the opening camera, mint
    /// [`script::chatted`] field for field, of 15. Seen red with the
    /// line's Return minting nothing: 14 orders, 14 whole in place.
    const INPUT_CHATTED: usize = 15;

    /// **The chatted count**: on the chat's frame the scene seats two
    /// leaders, so the plain box names both; then the events mint the
    /// chatted script, and the file written from them is the chatted file.
    #[test]
    fn minted_chatted_at_the_opening_camera() {
        let Some((ours, r)) = mint_events(chat_events()) else {
            return;
        };
        let leaders: Vec<u8> = r.scene(Some(0)).leaders.iter().map(|l| l.who).collect();
        println!("CHAT STAGE0 f{} leaders {leaders:?}", r.frame());
        assert_eq!(leaders, vec![0, 1], "two nations, both seated");
        let chatted = script::chatted();
        let whole = ours.iter().zip(&chatted).filter(|(a, b)| a == b).count();
        assert_eq!(
            (ours.len(), whole),
            (chatted.len(), INPUT_CHATTED),
            "{ours:?}"
        );
        assert_eq!(ours, chatted);
        drop(r);

        let home = std::env::var_os("HOME").unwrap_or_default();
        let (Ok(gathered_file), Ok(file)) = (
            std::fs::read(std::path::Path::new(&home).join(GATHERED_RCX)),
            std::fs::read(std::path::Path::new(&home).join(CHATTED_RCX)),
        ) else {
            println!("skipping the bytes: no {GATHERED_RCX} or {CHATTED_RCX}");
            return;
        };
        assert_eq!(file.len(), CHATTED_RCX_BYTES);
        let written = written(&ours).unwrap();
        assert!(written == file, "the file from the chatted orders");
        assert!(written != gathered_file, "not the gathered file");
    }

    /// The chatted file (`docs/CLIENT_INPUT.md` §12), written by rondata's
    /// `the_chatted_minted_packages_round_trip` under `RON_RCX_OUT`; the
    /// file the original's playback takes open (run 1599).
    const CHATTED_RCX: &str = "ron-golden/recordings/item4250-chatted-greatlakes-24000.rcx";
    const CHATTED_RCX_BYTES: usize = 59_491;

    /// **Stage 0 of the paused script** (item 4252): on
    /// [`script::PAUSE_FRAME`] the bare ground pixel the right press names
    /// while the game stands paused, at the opening camera; the capital is
    /// still the selection the gathered events made.
    const PAUSE_ANCHOR: AttackAnchor = (script::PAUSE_FRAME, (0, -1), (1320, 650));

    /// The chatted events and the pause's three, all on
    /// [`script::PAUSE_FRAME`]: the pause key, a right press on the ground,
    /// the pause key again.
    fn pause_events() -> Vec<Scripted> {
        let mut e = chat_events();
        let (f, _, (x, y)) = PAUSE_ANCHOR;
        e.extend([
            (f, None, Event::Pause),
            (f, None, Event::RightPress { x, y }),
            (f, None, Event::Pause),
        ]);
        e
    }

    /// **`INPUT_PAUSED`** (item 4252, `docs/CLIENT_INPUT.md` §13): the
    /// chatted events and the pause's three, under the opening camera, mint
    /// [`script::paused`] field for field, of 18.
    const INPUT_PAUSED: usize = 18;

    /// **The paused count**: stage 0 on the pause's frame — the capital
    /// still takes a rally point, and no object is under the ground's
    /// pixel, whose inverse is [`script::PAUSE_GATHER_AT`]; then the events
    /// mint the paused script, three packages on one frame, and the file
    /// written from them is the paused file.
    #[test]
    fn minted_paused_at_the_opening_camera() {
        let Ok(mut r) = lobby() else {
            println!("skipping: no lobby");
            return;
        };
        let chatted = script::chatted();
        let c = camera(None);
        let mut next = 0;
        while r.frame() < i64::from(script::PAUSE_FRAME) {
            let f = r.frame() as i32;
            while next < chatted.len() && chatted[next].0 == f {
                r.issue(&chatted[next]);
                next += 1;
            }
            r.step();
        }
        {
            let (scene, terrain) = (r.scene(Some(0)), r.terrain());
            let v = view(&c, &scene, &terrain);
            let id = ObjId {
                who: 0,
                o: script::CAPITAL,
            };
            let (gx, gy) = PAUSE_ANCHOR.2;
            println!(
                "PAUSE STAGE0 f{} ground {:?} names {}",
                r.frame(),
                ground_at(&v, gx, gy),
                v.names_object(gx, gy)
            );
            assert!(v.takes_gather_point(&[id]), "the capital trains");
            assert!(!v.names_object(gx, gy), "bare ground");
            assert_eq!(ground_at(&v, gx, gy), Some(script::PAUSE_GATHER_AT));
        }
        drop(r);

        let Some((ours, r)) = mint_events(pause_events()) else {
            return;
        };
        let paused = script::paused();
        let whole = ours.iter().zip(&paused).filter(|(a, b)| a == b).count();
        assert_eq!(
            (ours.len(), whole),
            (paused.len(), INPUT_PAUSED),
            "{ours:?}"
        );
        assert_eq!(ours, paused);
        drop(r);

        let home = std::env::var_os("HOME").unwrap_or_default();
        let (Ok(chatted_file), Ok(file)) = (
            std::fs::read(std::path::Path::new(&home).join(CHATTED_RCX)),
            std::fs::read(std::path::Path::new(&home).join(PAUSED_RCX)),
        ) else {
            println!("skipping the bytes: no {CHATTED_RCX} or {PAUSED_RCX}");
            return;
        };
        assert_eq!(file.len(), PAUSED_RCX_BYTES);
        let written = written(&ours).unwrap();
        assert!(written == file, "the file from the paused orders");
        assert!(written != chatted_file, "not the chatted file");
    }

    /// The paused file (`docs/CLIENT_INPUT.md` §13), written by rondata's
    /// `the_paused_minted_packages_round_trip` under `RON_RCX_OUT`; the
    /// file the original's playback takes open (run 1601).
    const PAUSED_RCX: &str = "ron-golden/recordings/item4252-paused-greatlakes-24000.rcx";
    const PAUSED_RCX_BYTES: usize = 59_515;

    /// The window the market's buttons are pinned at (`crate::hud`'s
    /// tests' size).
    const HUD_SIZE: (i32, i32) = (1920, 1080);

    /// **Stage 0 of the marketed script** (item 4259): on
    /// [`script::SELL_FRAME`] slot 0's Market [`script::MARKET_OBJECT`]'s
    /// anchor pixel at the opening camera.
    const MARKET_PIXEL: (i32, i32) = (1248, 389);
    /// The market's sell-food and buy-timber buttons at [`HUD_SIZE`], with
    /// two goods offered (food and timber, the Ancient age's).
    const SELL_FOOD_AT: (i32, i32) = (1258, 1054);
    const BUY_TIMBER_AT: (i32, i32) = (1822, 1054);

    /// The paused events and the market's three: a press on the Market,
    /// then its sell button for food; two frames on, its buy button for
    /// timber.
    fn market_events() -> Vec<Scripted> {
        let mut e = pause_events();
        let press = |(x, y): (i32, i32)| Event::Press { x, y };
        e.extend([
            (script::SELL_FRAME, None, press(MARKET_PIXEL)),
            (script::SELL_FRAME, None, press(SELL_FOOD_AT)),
            (script::BUY_FRAME, None, press(BUY_TIMBER_AT)),
        ]);
        e
    }

    /// **`INPUT_MARKETED`** (item 4259, `docs/CLIENT_INPUT.md` §16): the
    /// paused events and the market's three, through the HUD and the layer
    /// under the opening camera, mint [`script::marketed`] field for
    /// field, of 20.
    const INPUT_MARKETED: usize = 20;

    /// **The marketed count**: stage 0 on the sale's frame — the Market is
    /// slot 0's, finished, and its row offers food and timber with both
    /// buttons live where the events press them; then the events mint the
    /// marketed script, and the file written from them is the marketed
    /// file.
    #[test]
    fn minted_marketed_at_the_opening_camera() {
        let Ok(mut r) = lobby() else {
            println!("skipping: no lobby");
            return;
        };
        let paused = script::paused();
        let c = camera(None);
        let mut next = 0;
        while r.frame() < i64::from(script::SELL_FRAME) {
            let f = r.frame() as i32;
            while next < paused.len() && paused[next].0 == f {
                r.issue(&paused[next]);
                next += 1;
            }
            r.step();
        }
        {
            let (scene, terrain) = (r.scene(Some(0)), r.terrain());
            let v = view(&c, &scene, &terrain);
            let id = ObjId {
                who: 0,
                o: script::MARKET_OBJECT,
            };
            let anchor = v.anchors(0).find(|(a, ..)| *a == id).map(|(_, _, p)| p);
            let mut input = Input::new(0);
            input.selection = vec![id];
            input.building = true;
            let row: Vec<(String, bool, (i32, i32))> =
                crate::hud::market_menu(&input, &scene, HUD_SIZE)
                    .iter()
                    .map(|b| (b.label(), b.live, b.rect.centre()))
                    .collect();
            println!("MARKET STAGE0 f{} anchor {anchor:?} row {row:?}", r.frame());
            assert_eq!(anchor, Some(MARKET_PIXEL), "the Market's pixel");
            let at = |p: (i32, i32)| {
                row.iter()
                    .find(|(_, _, c)| *c == p)
                    .map(|(l, live, _)| (l.clone(), *live))
            };
            assert_eq!(at(SELL_FOOD_AT), Some(("SELL FOOD 81".into(), true)));
            assert_eq!(at(BUY_TIMBER_AT), Some(("BUY TIMBER 107".into(), true)));
        }
        drop(r);

        let Some((ours, r)) = mint_with(market_events(), |input, frame, ev, view| {
            crate::hud::event(input, HUD_SIZE, frame, ev, view)
        }) else {
            return;
        };
        let marketed = script::marketed();
        let whole = ours.iter().zip(&marketed).filter(|(a, b)| a == b).count();
        assert_eq!(
            (ours.len(), whole),
            (marketed.len(), INPUT_MARKETED),
            "{ours:?}"
        );
        assert_eq!(ours, marketed);
        drop(r);

        let home = std::env::var_os("HOME").unwrap_or_default();
        let (Ok(paused_file), Ok(file)) = (
            std::fs::read(std::path::Path::new(&home).join(PAUSED_RCX)),
            std::fs::read(std::path::Path::new(&home).join(MARKETED_RCX)),
        ) else {
            println!("skipping the bytes: no {PAUSED_RCX} or {MARKETED_RCX}");
            return;
        };
        assert_eq!(file.len(), MARKETED_RCX_BYTES);
        let written = written(&ours).unwrap();
        assert!(written == file, "the file from the marketed orders");
        assert!(written != paused_file, "not the paused file");
    }

    /// The marketed file (`docs/CLIENT_INPUT.md` §16), written by rondata's
    /// `the_marketed_minted_packages_round_trip` under `RON_RCX_OUT`; the
    /// file the original's playback takes open (run 1604).
    const MARKETED_RCX: &str = "ron-golden/recordings/item4259-marketed-greatlakes-24000.rcx";
    const MARKETED_RCX_BYTES: usize = 59_535;

    /// **Stage 0 of the city-gathered script** (item 4261): on
    /// [`script::CITY_GATHER_FRAME`] the capital's anchor pixel at the
    /// opening camera, and the city row's FARM button at [`HUD_SIZE`].
    const CAPITAL_PIXEL: (i32, i32) = (1127, 521);
    const CITY_FARM_AT: (i32, i32) = (1258, 1054);

    /// The marketed events and the city's two: a press on the capital, then
    /// its FARM gather button.
    fn city_gather_events() -> Vec<Scripted> {
        let mut e = market_events();
        let press = |(x, y): (i32, i32)| Event::Press { x, y };
        e.extend([
            (script::CITY_GATHER_FRAME, None, press(CAPITAL_PIXEL)),
            (script::CITY_GATHER_FRAME, None, press(CITY_FARM_AT)),
        ]);
        e
    }

    /// **`INPUT_CITY_GATHERED`** (item 4261, `docs/CLIENT_INPUT.md` §17):
    /// the marketed events and the city's two, through the HUD and the
    /// layer under the opening camera, mint [`script::city_gathered`] field
    /// for field, of 21. Seen red with the layer's `queued` 0: 21 orders,
    /// 20 whole.
    const INPUT_CITY_GATHERED: usize = 21;

    /// **The city-gathered count**: stage 0 on the gather's frame — the
    /// capital is slot 0's city centre, it trains and is a `VILLAGE`, and
    /// its row holds the four buttons, FARM where the event presses it;
    /// then the events mint the city-gathered script, and the file written
    /// from them is the city-gathered file.
    #[test]
    fn minted_city_gathered_at_the_opening_camera() {
        let Ok(mut r) = lobby() else {
            println!("skipping: no lobby");
            return;
        };
        let marketed = script::marketed();
        let c = camera(None);
        let mut next = 0;
        while r.frame() < i64::from(script::CITY_GATHER_FRAME) {
            let f = r.frame() as i32;
            while next < marketed.len() && marketed[next].0 == f {
                r.issue(&marketed[next]);
                next += 1;
            }
            r.step();
        }
        {
            let (scene, terrain) = (r.scene(Some(0)), r.terrain());
            let v = view(&c, &scene, &terrain);
            let id = ObjId {
                who: 0,
                o: script::CAPITAL,
            };
            let anchor = v.anchors(0).find(|(a, ..)| *a == id).map(|(_, _, p)| p);
            let b = scene.buildings.iter().find(|b| b.id == id).unwrap();
            let mut input = Input::new(0);
            input.selection = vec![id];
            input.building = true;
            let row: Vec<(&str, (i32, i32))> = crate::hud::city_menu(&input, &scene, HUD_SIZE)
                .iter()
                .map(|b| (b.label(), b.rect.centre()))
                .collect();
            println!(
                "CITY STAGE0 f{} anchor {anchor:?} pos {:?} z {} city {} trains {} village {} active {} row {row:?}",
                r.frame(),
                b.pos,
                b.z,
                b.city,
                b.trains,
                b.village,
                b.active
            );
            assert_eq!(anchor, Some(CAPITAL_PIXEL), "the capital's pixel");
            assert!(b.city && b.trains && b.village);
            assert_eq!(
                row.iter()
                    .find(|(_, c)| *c == CITY_FARM_AT)
                    .map(|(l, _)| *l),
                Some("GATHER FARM")
            );
            assert_eq!(row.len(), 4);
        }
        drop(r);

        let Some((ours, r)) = mint_with(city_gather_events(), |input, frame, ev, view| {
            crate::hud::event(input, HUD_SIZE, frame, ev, view)
        }) else {
            return;
        };
        let gathered = script::city_gathered();
        let whole = ours.iter().zip(&gathered).filter(|(a, b)| a == b).count();
        assert_eq!(
            (ours.len(), whole),
            (gathered.len(), INPUT_CITY_GATHERED),
            "{ours:?}"
        );
        assert_eq!(ours, gathered);
        drop(r);

        let home = std::env::var_os("HOME").unwrap_or_default();
        let (Ok(marketed_file), Ok(file)) = (
            std::fs::read(std::path::Path::new(&home).join(MARKETED_RCX)),
            std::fs::read(std::path::Path::new(&home).join(CITY_GATHERED_RCX)),
        ) else {
            println!("skipping the bytes: no {MARKETED_RCX} or {CITY_GATHERED_RCX}");
            return;
        };
        assert_eq!(file.len(), CITY_GATHERED_RCX_BYTES);
        let written = written(&ours).unwrap();
        assert!(written == file, "the file from the city-gathered orders");
        assert!(written != marketed_file, "not the marketed file");
    }

    /// The city-gathered file (`docs/CLIENT_INPUT.md` §17), written by
    /// rondata's `the_city_gathered_minted_packages_round_trip` under
    /// `RON_RCX_OUT`; the file the original's playback takes open.
    const CITY_GATHERED_RCX: &str =
        "ron-golden/recordings/item4261-city-gathered-greatlakes-24000.rcx";
    const CITY_GATHERED_RCX_BYTES: usize = 59_546;

    /// A city centre of `who`'s that trains and is a `VILLAGE`, every other
    /// field the lobby's own building's.
    fn city_building(who: u8, o: i16) -> sim::observe::BuildingScene {
        let Ok(r) = lobby() else {
            panic!("the city's edges need the lobby for a building record");
        };
        let mut b = r.scene(None).buildings[0];
        b.id = ObjId { who, o };
        b.owner = who;
        (b.city, b.trains, b.village, b.active) = (true, true, true, true);
        b
    }

    /// **`INPUT_CITY_GATHERS`** (item 4261): of the city gathers the
    /// recordings on disk carry, those the city's button mints field for
    /// field, the package's `group` and its `city_gather` whole, when its
    /// group's building is pressed and then the button of its type, of all
    /// of them. The two that part carry a `group` of no objects
    /// (`docs/CLIENT_INPUT.md` §17.4), which the panel's gate cannot pass.
    /// Seen red with the layer's `queued` 0: 0 of 20.
    const INPUT_CITY_GATHERS: (usize, usize) = (18, 20);

    #[test]
    fn minted_city_gathers_equal_the_recordings() {
        let files = recordings_on_disk();
        if files.is_empty() || lobby().is_err() {
            println!("skipping: no .rcx on disk or no lobby");
            return;
        }
        let terrain = TerrainScene::default();
        let c = crate::pinned_camera();
        let (mut whole, mut all) = (0, 0);
        for f in &files {
            let rec = rondata::recgame::read(&f.display().to_string()).unwrap();
            for p in &rec.packages {
                let Ok(cmds) = rondata::commands::decode(&p.data) else {
                    continue;
                };
                let kept: Vec<Command> =
                    cmds.into_iter().filter(rondata::input::is_input).collect();
                let Some(Command::CityGather { t, .. }) = kept
                    .iter()
                    .find(|c| matches!(c, Command::CityGather { .. }))
                else {
                    continue;
                };
                all += 1;
                let (who, objects) = match &kept[0] {
                    Command::Group { who, objects } => (*who as u8, objects.clone()),
                    _ => (p.play as u8, Vec::new()),
                };
                let scene = Scene {
                    buildings: objects.iter().map(|&o| city_building(who, o)).collect(),
                    ..Scene::default()
                };
                let v = view(&c, &scene, &terrain);
                let mut input = Input::new(who);
                input.selection = objects.iter().map(|&o| ObjId { who, o }).collect();
                input.building = true;
                let got = input.event(p.frame, Event::CityGather(*t), &v);
                if got.as_ref().map(|o| &o.2) == Some(&kept) {
                    whole += 1;
                } else {
                    println!("CITY GATHER PARTS f{} {kept:?}: {got:?}", p.frame);
                }
            }
        }
        println!("CITY GATHERS {whole} of {all}");
        assert_eq!((whole, all), INPUT_CITY_GATHERS);
    }

    /// **The city's buttons without a walk** (item 4261): a row only for
    /// the slot's own city centre; a press that issues only with a building
    /// selection holding a trainer and a `VILLAGE`; each button's type; no
    /// key sends it.
    #[test]
    fn minted_city_gather_button_edges() {
        if lobby().is_err() {
            println!("skipping: no lobby");
            return;
        }
        let terrain = TerrainScene::default();
        let c = crate::pinned_camera();
        let id = ObjId { who: 0, o: 2000 };
        let mint = |scene: &Scene, ev: Event| {
            let v = view(&c, scene, &terrain);
            let mut input = Input::new(0);
            input.selection = vec![id];
            input.building = true;
            input.event(6718, ev, &v)
        };
        let city = city_building(0, 2000);
        let one = |b| Scene {
            buildings: vec![b],
            ..Scene::default()
        };
        let row = |scene: &Scene, player: u8| {
            let mut input = Input::new(player);
            input.selection = vec![id];
            input.building = true;
            crate::hud::city_menu(&input, scene, HUD_SIZE)
                .iter()
                .map(|b| b.t)
                .collect::<Vec<_>>()
        };
        assert_eq!(row(&one(city), 0), vec![417, 418, 419, -1]);
        assert_eq!(row(&one(city), 1), Vec::<i32>::new(), "another slot's");
        assert_eq!(
            row(
                &one(sim::observe::BuildingScene {
                    city: false,
                    ..city
                }),
                0
            ),
            Vec::<i32>::new(),
            "not a centre"
        );
        for t in CITY_GATHER_TYPES {
            assert_eq!(
                mint(&one(city), Event::CityGather(t)),
                Some((
                    6718,
                    0,
                    vec![
                        Command::Group {
                            who: 0,
                            objects: vec![2000]
                        },
                        Command::CityGather { t, queued: 2 }
                    ]
                ))
            );
        }
        let no_trainer = sim::observe::BuildingScene {
            trains: false,
            ..city
        };
        let no_village = sim::observe::BuildingScene {
            village: false,
            ..city
        };
        assert_eq!(mint(&one(no_trainer), Event::CityGather(417)), None);
        assert_eq!(mint(&one(no_village), Event::CityGather(417)), None);
        assert_eq!(mint(&Scene::default(), Event::CityGather(417)), None);
        let scene = one(city);
        let v = view(&c, &scene, &terrain);
        let mut units = Input::new(0);
        units.selection = vec![id];
        assert_eq!(
            units.event(6718, Event::CityGather(417), &v),
            None,
            "a unit selection"
        );
    }

    /// A scene holding one market of `who`'s at `active`, and its leader
    /// with `market` offered and `bucket` in store.
    fn market_scene(
        who: u8,
        active: bool,
        market: [Option<(i32, i32)>; 6],
        bucket: [i32; 6],
    ) -> Scene {
        let mut b = market_building();
        b.id = ObjId { who, o: 2006 };
        b.owner = who;
        b.active = active;
        Scene {
            leaders: vec![sim::observe::LeaderScene {
                who,
                tribe: 4,
                nation: String::new(),
                ages: 0,
                bucket,
                income: [0; 6],
                over_cap: [false; 6],
                control: 0,
                pop_cap: 0,
                market,
            }],
            buildings: vec![b],
            ..Scene::default()
        }
    }

    /// A finished market, every other field the lobby's own building's.
    fn market_building() -> sim::observe::BuildingScene {
        let Ok(r) = lobby() else {
            panic!("the market's edges need the lobby for a building record");
        };
        let mut b = r.scene(None).buildings[0];
        b.market = true;
        b.active = true;
        b
    }

    /// **`INPUT_TRADED`** (item 4259): every sale and purchase the
    /// recordings on disk carry, pressed from its leader's market panel,
    /// mints the recorded order field for field, of 46.
    const INPUT_TRADED: (usize, usize) = (46, 46);

    #[test]
    fn minted_trades_equal_the_recordings() {
        let files = recordings_on_disk();
        if files.is_empty() || lobby().is_err() {
            println!("skipping: no .rcx on disk or no lobby");
            return;
        }
        let terrain = TerrainScene::default();
        let c = crate::pinned_camera();
        let (mut whole, mut all) = (0, 0);
        for f in &files {
            let rec = rondata::recgame::read(&f.display().to_string()).unwrap();
            for it in rondata::input::input(&rec) {
                let (who, good, buy) = match it.cmd {
                    Command::Sell { who, good, .. } => (who, good, false),
                    Command::Buy { who, good, .. } => (who, good, true),
                    _ => continue,
                };
                all += 1;
                let mut offer = [None; 6];
                offer[good as usize] = Some((100, 80));
                let scene = market_scene(who as u8, true, offer, [1000; 6]);
                let v = view(&c, &scene, &terrain);
                let mut input = Input::new(who as u8);
                input.selection = vec![ObjId {
                    who: who as u8,
                    o: 2006,
                }];
                input.building = true;
                let ev = if buy {
                    Event::Buy(good as usize)
                } else {
                    Event::Sell(good as usize)
                };
                let got = input.event(it.frame, ev, &v);
                if got.as_ref().map(|o| &o.2) == Some(&vec![it.cmd.clone()]) {
                    whole += 1;
                } else if all - whole <= 3 {
                    println!("TRADE PARTS {it:?}: {got:?}");
                }
            }
        }
        println!("TRADED {whole} of {all}");
        assert_eq!((whole, all), INPUT_TRADED);
    }

    /// **The market's buttons without a walk** (item 4259): a row only for
    /// the slot's own finished market, a button only for a good offered, a
    /// sale only with a hundred in store and a purchase only with the
    /// price; a dead button mints nothing; nothing a key sends.
    #[test]
    fn minted_market_button_edges() {
        if lobby().is_err() {
            println!("skipping: no lobby");
            return;
        }
        let terrain = TerrainScene::default();
        let c = crate::pinned_camera();
        let offer = [Some((106, 81)), Some((107, 82)), None, None, None, None];
        let mint = |scene: &Scene, ev: Event| {
            let v = view(&c, scene, &terrain);
            let mut input = Input::new(0);
            input.selection = vec![ObjId { who: 0, o: 2006 }];
            input.building = true;
            input.event(6714, ev, &v)
        };
        let rich = market_scene(0, true, offer, [721, 425, 249, 0, 0, 0]);
        assert_eq!(
            mint(&rich, Event::Sell(0)),
            Some((
                6714,
                0,
                vec![Command::Sell {
                    who: 0,
                    good: 0,
                    flags: 0
                }]
            ))
        );
        assert_eq!(
            mint(&rich, Event::Buy(1)),
            Some((
                6714,
                0,
                vec![Command::Buy {
                    who: 0,
                    good: 1,
                    flags: 0
                }]
            ))
        );
        assert_eq!(mint(&rich, Event::Sell(4)), None, "metal is not offered");
        assert_eq!(mint(&rich, Event::Buy(2)), None, "wealth has no button");
        let poor = market_scene(0, true, offer, [99, 425, 106, 0, 0, 0]);
        assert_eq!(mint(&poor, Event::Sell(0)), None, "under a hundred");
        assert_eq!(mint(&poor, Event::Buy(1)), None, "under the price");
        assert!(mint(&poor, Event::Buy(0)).is_some(), "at the price");
        let unfinished = market_scene(0, false, offer, [721; 6]);
        assert_eq!(mint(&unfinished, Event::Sell(0)), None, "unfinished");
        let theirs = market_scene(1, true, offer, [721; 6]);
        assert_eq!(mint(&theirs, Event::Sell(0)), None, "another slot's market");
        let mut input = Input::new(0);
        input.selection = vec![ObjId { who: 0, o: 2006 }];
        input.building = true;
        let row = crate::hud::market_menu(&input, &poor, HUD_SIZE);
        let shown: Vec<(String, bool)> = row.iter().map(|b| (b.label(), b.live)).collect();
        assert_eq!(
            shown,
            [
                ("SELL FOOD 81".to_string(), false),
                ("BUY FOOD 106".to_string(), true),
                ("SELL TIMBER 82".to_string(), true),
                ("BUY TIMBER 107".to_string(), false),
            ]
        );
        assert_eq!(row.last().unwrap().rect.centre(), BUY_TIMBER_AT);
        assert_eq!(row[0].rect.centre(), SELL_FOOD_AT);
        assert!(crate::hud::market_menu(&Input::new(0), &poor, HUD_SIZE).is_empty());
    }

    /// **The pause key without a walk** (item 4252): a first press mints
    /// `pause(1)` alone and the pause stands; a press while it stands mints
    /// `pause(0)` on the same frame and lifts it. With nothing selected and
    /// the entry line closed, the key still mints: a pause names no object.
    #[test]
    fn minted_pause_key_toggles() {
        let (scene, terrain) = (Scene::default(), TerrainScene::default());
        let c = crate::pinned_camera();
        let v = view(&c, &scene, &terrain);
        let mut input = Input::new(0);
        assert_eq!(
            input.event(7, Event::Pause, &v),
            Some((7, 0, vec![Command::Pause { state: 1 }]))
        );
        assert!(input.paused);
        assert_eq!(
            input.event(7, Event::Pause, &v),
            Some((7, 0, vec![Command::Pause { state: 0 }]))
        );
        assert!(!input.paused);
    }

    /// The recordings on disk, as `rondata::recgame`'s census reads them:
    /// the bottle's `Logs\` and the install's `external-recgames/`.
    fn recordings_on_disk() -> Vec<std::path::PathBuf> {
        let home = std::env::var("HOME").unwrap_or_default();
        let logs = std::env::var("RON_RECGAME_DIR")
            .or_else(|_| std::env::var("RON_GAMELOG_DIR"))
            .unwrap_or_else(|_| {
                format!("{home}/ron-data/AppData/Roaming/Microsoft Games/Rise of Nations/Logs")
            });
        let install = std::env::var_os("RON_INSTALL")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| "../../game".into());
        let mut out: Vec<std::path::PathBuf> = [logs.into(), install.join("external-recgames")]
            .iter()
            .filter_map(|d: &std::path::PathBuf| std::fs::read_dir(d).ok())
            .flatten()
            .filter_map(|e| {
                let p = e.ok()?.path();
                (p.extension()? == "rcx").then_some(p)
            })
            .collect();
        out.sort();
        out
    }

    /// **`INPUT_CHEATED`** (item 4250): of the chats the recordings on disk
    /// carry, those the entry line mints field for field when their text
    /// is typed into it by their own slot on their own frame, of all of
    /// them. Every one is a cheat line, the original's own mint
    /// (`ChatBox::on_modal_end`'s cheat arm). Seen red with the cheat arm
    /// taken out: 0 of 160, each parting at the plain box's `taunt` −1.
    const INPUT_CHEATED: (usize, usize) = (160, 160);

    /// The cheat arm against the original's own chats.
    #[test]
    fn minted_cheat_lines_equal_the_recordings() {
        let files = recordings_on_disk();
        if files.is_empty() {
            println!("skipping: no .rcx on disk (RON_RECGAME_DIR, RON_INSTALL)");
            return;
        }
        let (scene, terrain) = (Scene::default(), TerrainScene::default());
        let c = crate::pinned_camera();
        let v = view(&c, &scene, &terrain);
        let (mut whole, mut all) = (0, 0);
        for f in &files {
            let rec = rondata::recgame::read(&f.display().to_string()).unwrap();
            for it in rondata::input::input(&rec) {
                let Command::Chat { text, .. } = &it.cmd else {
                    continue;
                };
                all += 1;
                let mut input = Input::new(it.play as u8);
                let mut got = None;
                for (_, _, ev) in typed(it.frame, text) {
                    got = got.or(input.event(it.frame, ev, &v));
                }
                if got == Some((it.frame, it.play, vec![it.cmd.clone()])) {
                    whole += 1;
                } else if all - whole <= 3 {
                    println!("CHEAT PARTS {it:?}: {got:?}");
                }
            }
        }
        println!("CHEATED {whole} of {all}");
        assert_eq!((whole, all), INPUT_CHEATED);
    }

    /// **The entry line's edges** (item 4250): it opens on Return and takes
    /// keys from the layer while open, Backspace takes a character, Escape
    /// sends nothing, the spaces at both ends are trimmed and a blank line
    /// sends nothing, the limit holds, and the arms the layer does not read
    /// mint nothing — a targeting first character, a taunt's digit, a game
    /// of three nations.
    #[test]
    fn minted_chat_line_edges() {
        let leader = |who| sim::observe::LeaderScene {
            who,
            tribe: 0,
            nation: String::new(),
            ages: 0,
            bucket: [0; 6],
            income: [0; 6],
            over_cap: [false; 6],
            control: 0,
            pop_cap: 0,
            market: [None; 6],
        };
        let terrain = TerrainScene::default();
        let c = crate::pinned_camera();
        let two = Scene {
            leaders: vec![leader(0), leader(1)],
            ..Scene::default()
        };
        let three = Scene {
            leaders: vec![leader(0), leader(1), leader(2)],
            ..Scene::default()
        };
        let send = |scene: &Scene, evs: &[Event]| {
            let v = view(&c, scene, &terrain);
            let mut input = Input::new(1);
            input.selection = vec![ObjId { who: 1, o: 2000 }];
            input.building = true;
            let mut got = None;
            for ev in evs {
                got = got.or(input.event(7, *ev, &v));
            }
            (got, input)
        };
        let keys = |s: &str| -> Vec<Event> {
            let mut e = vec![Event::Enter];
            e.extend(s.chars().map(Event::Key));
            e.push(Event::Enter);
            e
        };
        let chat = |bits, text: &str| {
            Some((
                7,
                1,
                vec![Command::Chat {
                    bits,
                    taunt: -1,
                    taunt_num: 0,
                    text: text.to_string(),
                }],
            ))
        };
        // `c` would train at the selected building; the open line takes it.
        assert_eq!(send(&two, &keys("  cc hi ")).0, chat(3, "cc hi"));
        let mut bs = keys("hix");
        bs.insert(4, Event::Backspace);
        assert_eq!(send(&two, &bs).0, chat(3, "hi"));
        let (got, input) = send(&two, &[Event::Enter, Event::Key('h'), Event::Escape]);
        assert_eq!((got, input.line), (None, None));
        assert_eq!(send(&two, &keys("   ")).0, None);
        assert_eq!(send(&two, &keys("")).0, None);
        for s in ["*hi", "!hi", ";hi", ":hi", "@hi", "4", "12 food", "-3"] {
            assert_eq!(send(&two, &keys(s)).0, None, "{s}");
        }
        assert_eq!(send(&three, &keys("hi")).0, None);
        let long: String = std::iter::repeat_n('a', CHAT_LIMIT + 5).collect();
        let want: String = std::iter::repeat_n('a', CHAT_LIMIT).collect();
        assert_eq!(send(&two, &keys(&long)).0, chat(3, &want));
        // The cheat arm: the slot's own bit, `taunt` 0, any case.
        assert_eq!(
            send(&three, &keys("Cheat select 6 ")).0,
            Some((
                7,
                1,
                vec![Command::Chat {
                    bits: 2,
                    taunt: 0,
                    taunt_num: 0,
                    text: "Cheat select 6".to_string(),
                }],
            ))
        );
        // Closed, Return opens the line and nothing else changes.
        let (got, input) = send(&two, &[Event::Enter]);
        assert_eq!((got, input.line.as_deref()), (None, Some("")));
    }

    /// A unit for the walk-free box tests: slot `who`'s `o` of type `ti`,
    /// standing at `(x, 0)` with its formation bytes.
    fn unit(who: u8, o: i16, ti: i32, x: i32, form: i8, width: i8) -> sim::observe::UnitScene {
        sim::observe::UnitScene {
            id: ObjId { who, o },
            owner: who,
            ty: None,
            type_index: ti,
            form,
            form_width: width,
            pos: Pos { x, y: 0 },
            z: 0,
            facing: sim::movement::Angle(0),
            health: 1,
            max_health: 1,
            seen: true,
            attacks: true,
            figures: vec![],
        }
    }

    /// **The box's rule without a walk** (`docs/CLIENT_INPUT.md` §7.1): the
    /// list sorted by `TypeIndex` and in object order within one type, the
    /// default profile's civilian rule both ways, another slot's units left
    /// out, and the formation read off the members. Seen red: the type
    /// sort dropped gave `[2, 5, 7]` for `[5, 7, 2]`.
    #[test]
    fn minted_a_box_lists_filters_and_forms_as_the_original() {
        let terrain = TerrainScene::default();
        let c = crate::pinned_camera();
        let scene = |units| Scene {
            units,
            ..Scene::default()
        };
        let take = |s: &Scene| {
            let v = view(&c, s, &terrain);
            let px: Vec<_> = s
                .units
                .iter()
                .map(|u| v.pixel(u.pos.x, u.pos.y, 0))
                .collect();
            let (l, r) = (
                px.iter().map(|p| p.0).min().unwrap(),
                px.iter().map(|p| p.0).max().unwrap(),
            );
            let (t, b) = (
                px.iter().map(|p| p.1).min().unwrap(),
                px.iter().map(|p| p.1).max().unwrap(),
            );
            let ids: Vec<i16> = v.boxed(0, (l, t), (r, b)).iter().map(|id| id.o).collect();
            ids
        };
        // A Scholar (52) first in object order, then two Citizens (50), of
        // slot 0, and slot 1's Citizen among them: workers only, by type.
        let workers = scene(vec![
            unit(0, 7, 50, 3000, 9, -1),
            unit(0, 2, 52, 3100, 9, -1),
            unit(1, 3, 50, 3150, 9, -1),
            unit(0, 5, 50, 3200, 9, -1),
        ]);
        assert_eq!(take(&workers), vec![5, 7, 2], "by type, then by object");
        // A Merchant (61) with the workers: the workers stay.
        let merchant = scene(vec![
            unit(0, 2, 50, 3000, 9, -1),
            unit(0, 4, 61, 3100, 0, -1),
        ]);
        assert_eq!(take(&merchant), vec![2], "the Merchant goes");
        // A Scout (69) with the workers: the civilians go.
        let scout = scene(vec![
            unit(0, 2, 50, 3000, 9, -1),
            unit(0, 0, 69, 3100, 0, 50),
        ]);
        assert_eq!(take(&scout), vec![0], "the Citizen goes");
        // Merchants alone: no worker, so nothing is filtered.
        let merchants = scene(vec![
            unit(0, 4, 61, 3000, 0, -1),
            unit(0, 6, 61, 3100, 0, -1),
        ]);
        assert_eq!(take(&merchants), vec![4, 6], "no worker, no filter");

        let v = view(&c, &workers, &terrain);
        let id = |o| ObjId { who: 0, o };
        assert_eq!(
            v.formation(&[id(2), id(5), id(7)]),
            (9, 50),
            "shared, no width"
        );
        let mixed = scene(vec![
            unit(0, 2, 50, 0, 9, 40),
            unit(0, 3, 69, 0, 0, -1),
            unit(0, 4, 69, 0, 0, 61),
        ]);
        let v = view(&c, &mixed, &terrain);
        assert_eq!(
            v.formation(&[id(2), id(3), id(4)]),
            (0, 50),
            "differing: 0; the mean of 40 and 61"
        );
        let soldiers = scene(vec![unit(0, 3, 69, 0, -1, -1), unit(0, 4, 69, 0, -1, -1)]);
        let v = view(&c, &soldiers, &terrain);
        assert_eq!(v.formation(&[id(3), id(4)]), (0, 50), "−1 clamped to 0");
    }

    /// At the closer camera a pixel is finer than the integer grid: a
    /// neighbour of a target's pixel with no integer point of its own still
    /// names the ground, one unit or less from the target.
    #[test]
    fn minted_a_fine_pixel_still_names_the_ground() {
        let scene = Scene::default();
        let terrain = TerrainScene::default();
        let (x, y) = TARGETS[0].1;
        let c = camera(Some((Pos { x, y }, 0)));
        let v = view(&c, &scene, &terrain);
        let (px, py) = v.pixel(x, y, 0);
        let mut empty = 0;
        for (dx, dy) in [(1, 0), (0, 1), (-1, 0), (0, -1), (1, 1)] {
            let at = ground_at(&v, px + dx, py + dy).unwrap();
            assert!((at.x - x).abs() <= 1 && (at.y - y).abs() <= 1, "{at:?}");
            empty += usize::from(v.pixel(at.x, at.y, 0) != (px + dx, py + dy));
        }
        assert!(
            empty > 0,
            "every neighbour held a point: the arm is untested"
        );
    }

    /// The layer's arms without a walk: a press on nothing clears, a right
    /// press with nothing selected or a building selected mints nothing,
    /// a key with nothing selected mints nothing.
    #[test]
    fn minted_nothing_without_a_selection() {
        let scene = Scene::default();
        let terrain = TerrainScene::default();
        let c = crate::pinned_camera();
        let v = view(&c, &scene, &terrain);
        let mut input = Input::new(0);
        assert_eq!(
            input.event(1, Event::RightPress { x: 900, y: 500 }, &v),
            None
        );
        assert_eq!(input.event(1, Event::Key('c'), &v), None);
        assert_eq!(input.event(1, Event::Press { x: 900, y: 500 }, &v), None);
        assert!(input.selection.is_empty());
        input.selection = vec![ObjId { who: 0, o: 2000 }];
        input.building = true;
        assert_eq!(
            input.event(1, Event::RightPress { x: 900, y: 500 }, &v),
            None
        );
        assert_eq!(input.event(1, Event::Key('x'), &v), None);
        let order = input.event(1, Event::Key('c'), &v).unwrap();
        assert_eq!(order.2[1], Command::QueueUp { unit: 50, num: 1 });
    }

    /// **`INPUT_KINDS`** (item 4249, `docs/CLIENT_INPUT.md` §11): of the
    /// order kinds the recordings on disk carry
    /// (`rondata::input::KINDS_ON_DISK`), those the layer mints, and the
    /// share of the recordings' orders they make.
    const INPUT_KINDS: (usize, usize) = (14, 26);
    /// The orders those kinds make, of every order the recordings carry.
    const INPUT_KINDS_ORDERS: (usize, usize) = (2306, 2429);
    /// The kinds on disk the layer does not mint, most first: the next
    /// kind by share is the first.
    const NOT_MINTED: &[&str] = &[
        "leader_options",
        "spell",
        "unqueue",
        "rename_city",
        "hotkey",
        "garrison",
        "buildmask",
        "cheat_ai_toggle",
        "come_out",
        "stance",
        "disband",
        "unitmask",
    ];

    /// The census, counted against the disk's pinned kinds.
    #[test]
    fn input_kinds_of_the_recordings_on_disk() {
        let disk = rondata::input::KINDS_ON_DISK;
        let minted: Vec<&(&str, usize)> = disk.iter().filter(|k| MINTS.contains(&k.0)).collect();
        let orders: usize = disk.iter().map(|k| k.1).sum();
        let ours: usize = minted.iter().map(|k| k.1).sum();
        for (k, n) in disk {
            let mark = if MINTS.contains(k) { "minted" } else { "" };
            eprintln!(
                "KIND {k:>16} {n:>5} {:>3}.{} % {mark}",
                n * 100 / orders,
                n * 1000 / orders % 10
            );
        }
        assert_eq!((minted.len(), disk.len()), INPUT_KINDS);
        assert_eq!((ours, orders), INPUT_KINDS_ORDERS);
        let not: Vec<&str> = disk
            .iter()
            .map(|k| k.0)
            .filter(|k| !MINTS.contains(k))
            .collect();
        assert_eq!(not, NOT_MINTED);
    }

    /// **The layer mints what [`MINTS`] says** (item 4249): every
    /// `Command::` value built in this file's non-test source, by its
    /// variant's name in `Command::name`'s spelling, is [`MINTS`], no more
    /// and no fewer.
    #[test]
    fn the_layer_mints_what_it_says() {
        let src = include_str!("input.rs");
        let body: String = src[..src.find("#[cfg(test)]\npub(crate) mod tests").unwrap()]
            .lines()
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        let mut built: Vec<String> = Vec::new();
        for (i, _) in body.match_indices("Command::") {
            let rest = &body[i + "Command::".len()..];
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric())
                .collect();
            let after = rest[name.len()..].trim_start();
            // A value is built with its fields (`{`) or bare; a pattern
            // (`{ .. }`) and the `use` line are not.
            if after.starts_with("{ ..") || after.starts_with(';') || name.is_empty() {
                continue;
            }
            let mut snake = String::new();
            for (j, c) in name.chars().enumerate() {
                if c.is_ascii_uppercase() && j > 0 {
                    snake.push('_');
                }
                snake.push(c.to_ascii_lowercase());
            }
            if !built.contains(&snake) {
                built.push(snake);
            }
        }
        let mut want: Vec<String> = MINTS.iter().map(|s| s.to_string()).collect();
        built.sort();
        want.sort();
        assert_eq!(built, want);
    }
}
