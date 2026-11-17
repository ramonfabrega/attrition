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
/// a [`Event::Drag`] to box rather than click, in pixels, the larger of the
/// two legs. Since item 4318 the window sends any release that moved as a
/// drag and the layer asks this after an armed pick, which takes the
/// release and the press either way (§38.2). Ours: the original's
/// `click_drag_low_sensitive` and `click_drag_high_sensitive` are not read.
pub const DRAG_SLOP: i32 = 4;
/// `Console::click_drag_right_sensitive` (`+0x1d00`), the screen distance
/// a right press must move to drag: `Console::Console@007cbf60` sets 6,
/// and `System::init@00599700` reads a profile value over it, whose
/// default is not read (item 4295, `docs/CLIENT_INPUT.md` §34.1).
pub const RIGHT_DRAG_SLOP: i32 = 6;

/// **Whether a right press moved far enough to drag** (item 4295, §34.1):
/// `Console::on_mouse_move@007cef20` clears `clicking` once
/// `vector_dist(|select_x2 − select_x|, |select_y2 − select_y|)`, in
/// screen pixels, passes [`RIGHT_DRAG_SLOP`]. Ours: tested once, at the
/// release, where the original tests every move, so a drag that comes
/// back to its press is a press here and a drag there.
pub fn right_dragged(press: (i32, i32), release: (i32, i32)) -> bool {
    sim::world::vector_dist((release.0 - press.0).abs(), (release.1 - press.1).abs())
        > RIGHT_DRAG_SLOP
}

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
    "spell",
    "unqueue",
    "rename_city",
    "hotkey",
    "garrison",
    "buildmask",
    "come_out",
    "stance",
    "cheat_ai_toggle",
    "disband",
    "unitmask",
    "leader_options",
];

/// **The auto-explore button's mask and set** (item 4288,
/// `docs/CLIENT_INPUT.md` §31): `Options::exec@007188c0` case `0x21` calls
/// `Options::do_auto_explore@0071cbf0` with the option's object, which
/// `UnitOut::make_options@00615fc0` adds as −1, and that calls
/// `GroupOut::issue_unitmask@00708880(0x100, −1)`: command `0x20`, 9 bytes.
/// `Group::action_unitmask@006fcb90` never reads the set; it toggles the
/// member's `unit_masks & 0x100`.
pub const AUTO_EXPLORE_MASK: i32 = 0x100;
pub const AUTO_EXPLORE_SET: i32 = -1;

/// **The disband button's `all`**: `Options::exec@007188c0` case `0xf`,
/// `Options::do_disband@0071df20(0)`, so `Group::action_disband@0070e260`
/// stops after the first member it disbands (item 4287,
/// `docs/CLIENT_INPUT.md` §29). Case `0x10` (`OPTION_DISBAND_ALL`) passes 1;
/// it is not minted.
pub const DISBAND_ONE: i32 = 0;

/// **The stance button's option**: `Options::do_stance@00721480`'s −1, the
/// cycle, which `Group::action_stance@0070d440` steps one option forward
/// from the group's modal one (item 4282, `docs/CLIENT_INPUT.md` §27).
pub const STANCE_CYCLE: i32 = -1;

/// **`OPTION_INFINITE`'s mask**, the infinite queue's bit of
/// `WallData::build_masks`, which `Options::exec@007188c0`'s case `0x40`
/// passes to `GroupOut::issue_buildmask@00708820` (item 4276,
/// `docs/CLIENT_INPUT.md` §24).
pub const INFINITE_MASK: i32 = 0x40;

/// **A player's hot-key slots**: `Console::on_key_down@007cc210` numbers a
/// group `who × 0x12` plus the key's, `0`..`9` and F1..F8 (item 4270,
/// `docs/CLIENT_INPUT.md` §22).
pub const HOTKEY_SLOTS: u8 = 0x12;
/// **The camera fields of a hot key that carries a selection**:
/// `CommandManager::issue_hotkey@00941fd0` writes `x` and `y` as −1.0
/// and `zoom` 0 when its camera flag is clear; the bits of −1.0f, a
/// constant the layer never computes.
pub const HOTKEY_NO_CAMERA: u32 = 0xbf80_0000;

/// **The entry line's text limit**, in UTF-16 units: `ChatBox::exec
/// @0093cf80` sets the edit box's `text_limit` to `0x80`.
pub const CHAT_LIMIT: usize = 0x80;
/// **The rename box's text limit**, in UTF-16 units: `Options::exec
/// @007188c0`'s `0x117` arm opens the city's name in `BasePop::edit` with
/// `0x15` (item 4268, `docs/CLIENT_INPUT.md` §21).
pub const RENAME_LIMIT: usize = 0x15;
/// **The units a rename carries**: `CommandManager::issue_rename_city
/// @009411d0` copies a name of more than `0x13` units as `0x14`.
pub const RENAME_SENT: usize = 0x14;
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
    /// **A right drag** (item 4295, `docs/CLIENT_INPUT.md` §34): a right
    /// press at `(x0, y0)`, a drag past the window's slop, and the release
    /// at `(x1, y1)`. `Console::on_right_up@007cd3c0`'s drag arm: a
    /// `move_to` to the press's ground point, `set_angle` 1, and the angle
    /// `find_angle` takes from the press's ground point to the release's.
    RightDrag {
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
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
    /// A press on one of a unit's craft buttons, by the craft's row
    /// (`OPTION_SPELL`, `Options::exec@007188c0`'s
    /// `Options::do_spell@0071d7a0`; item 4264, `docs/CLIENT_INPUT.md`
    /// §18). An untargeted craft issues at once; a targeted one waits for
    /// the next left press, the pick. The HUD's alone.
    Spell(i32),
    /// A press on one of a building's queue buttons, by the slot it
    /// stands for (`OPTION_QUEUE`, `IFaceSelected::on_button_clicked
    /// @00826e00`'s `Options::exec(OPTION_QUEUE, slot)`, whose 0xa6 arm
    /// calls `BuildOut::issue_unqueue@00629e70` per finished building of
    /// the selection; item 4265, `docs/CLIENT_INPUT.md` §19). The HUD's
    /// alone.
    Unqueue(i32),
    /// A press on the city's name (`Options::exec@007188c0`'s option
    /// `0x117`, which opens the city's name in a popup, `popup_id` 6; item
    /// 4268, `docs/CLIENT_INPUT.md` §21): the entry line opens as the
    /// popup's edit box, and Return sends what it holds as
    /// `Popup::on_modal_end@007c13f0`'s case 6 does. The HUD's alone.
    Rename,
    /// Ctrl and a hot key, by the key's number: `0`..`9`, and F1..F8 as
    /// 10..17 (`Console::on_key_down@007cc210`; item 4270,
    /// `docs/CLIENT_INPUT.md` §22). With a selection it is
    /// `GroupOut::issue_hotkey@00708800`: the selection's `group` and
    /// `hotkey(who × 18 + key, 0, 0, −1.0, −1.0, 0)`. Ours: with none, the
    /// original's screen group (the camera's floats) is not sent.
    Hotkey(u8),
    /// A press on a unit's garrison button (`OPTION_GARRISON`,
    /// `Options::exec@007188c0`'s `Options::do_garrison@0071d6b0`; item
    /// 4273, `docs/CLIENT_INPUT.md` §23): with a selection of units it
    /// sets `selecting_spot`, and the next left press, the pick, names the
    /// building. The HUD's alone.
    Garrison,
    /// A press on a building's infinite-queue button (`OPTION_INFINITE`,
    /// `IFaceSelected::on_button_clicked@00826e00`; item 4276,
    /// `docs/CLIENT_INPUT.md` §24): `Options::exec@007188c0`'s case `0x40`
    /// with a selection of buildings is `GroupOut::issue_buildmask
    /// @00708820(0x40)`, the selection's `group` and `buildmask(0x40, 1)`.
    /// The HUD's alone.
    Infinite,
    /// A press on the `k`th garrisoned unit's icon of the selected
    /// buildings ([`garrisoned`]; item 4280, `docs/CLIENT_INPUT.md` §26):
    /// `Options::exec@007188c0` case `0xaa`, whose unit inside a building
    /// is `UnitOut::issue_come_out@00607be0`, command `0x31` with no
    /// group. The HUD's alone.
    ComeOut(usize),
    /// A press on the selection's stance button (item 4282,
    /// `docs/CLIENT_INPUT.md` §27): `Options::exec@007188c0` case `0x11`,
    /// `Options::do_stance@00721480(-1)`, which writes the selection's
    /// `group` and `stance(-1)` (`GroupOut::issue_stance@0070b100`
    /// inlined). The HUD's alone.
    Stance,
    /// A press on the selection's disband button (item 4287,
    /// `docs/CLIENT_INPUT.md` §29): `Options::exec@007188c0` case `0xf`,
    /// `Options::do_disband@0071df20(0)`, which writes the selection's
    /// `group` and `disband(0)` (`CommandManager::issue_disband@00941ad0`).
    /// The HUD's alone.
    Disband,
    /// A press on the selection's auto-explore button (item 4288,
    /// `docs/CLIENT_INPUT.md` §31): `Options::exec@007188c0` case `0x21`,
    /// `Options::do_auto_explore@0071cbf0(−1)`, which writes the
    /// selection's `group` and `unitmask(0x100, −1)`
    /// (`GroupOut::issue_unitmask@00708880`). The HUD's alone.
    AutoExplore,
    /// A press on a citizen's halt button (item 4346, §32.9):
    /// `Options::exec@007188c0` case `0x19`, `Options::do_halt@0071c580`,
    /// which for a unit selection that `can_activate` writes its `group`
    /// and `halt` (`GroupOut::issue_halt@0070ad70`). The HUD's alone.
    Halt,
    /// A press on a citizen's alarm button, `OPTION_ALARM_PEASANT` (item
    /// 4346, §32.9): `Options::exec@007188c0` case `0x1b`,
    /// `Options::do_alarm@0071c3f0`, which for a selection that
    /// `can_activate`, in a game whose semaphore's `0x10` bit is clear,
    /// writes its `group` and `alarm` (`0x1b`). The HUD's alone.
    Alarm,
    /// A press on a standard-panel button whose order the layer does not
    /// issue, by its option (item 4346, §32.9): `OPTION_REPAIR`,
    /// `OPTION_MOVE_TO` and `OPTION_ATTACK_TO` arm a pick
    /// (`selecting_spot`) whose spot `Options::picked_spot@00721c40` turns
    /// into `execute_at_cursor(0, 3 or 1)` or the repair arm; and
    /// `OPTION_TRANSPORT`'s `set_transport`, which asks
    /// `GroupData::can_transport`. Consumed; it orders nothing.
    Unbuilt(i32),
    /// A press on one of a building's train or research buttons, by the
    /// `TypeIndex` it queues (item 4290, `docs/CLIENT_INPUT.md` §32): the
    /// panel's `OPTION_TRAIN` and `OPTION_RESEARCH`, which
    /// `BuildOut::make_options@00629ed0` adds for every type
    /// `BuildTypeData::queue_here` admits; the `queue_up` a [`TRAIN_KEYS`]
    /// key writes. The HUD's alone ([`Input::panel`]).
    Train(i32),
    /// A press on one of the citizen's build buttons, by the `TypeIndex` it
    /// chooses (item 4290): `UnitOut::make_options@00615fc0`'s
    /// `BUILD_OPTIONS` page; the next press places it, as a
    /// [`BUILD_KEYS`] key's choice does. The HUD's alone.
    Place(i32),
    /// A press on one of the citizen's page buttons (item 4339, §32.7), by
    /// its option: [`OPTION_BUILD`], [`OPTION_BUILD_MILITARY`] or
    /// [`OPTION_BUILD_WONDER`] opens that page; on a page, [`OPTION_NEXT`],
    /// [`OPTION_BACK`] and [`OPTION_CANCEL`] turn it or close it, as
    /// `Options::exec@007188c0`'s cases 0 to 2, 0x41, 0x42 and 0x43 set the
    /// mode. The HUD's alone.
    Page(i32),
    /// The console's key (item 4283, `docs/CLIENT_INPUT.md` §28): opens
    /// the entry line as `ConsoleWin`'s, whose Return runs the line's
    /// word through `ConsoleWin::parse_cmd@007d6470` and `run_cmd
    /// @007d6a70`'s console half. Ours: the `~` key.
    Console,
}

/// `spell(type, ox, whom, x, y)`, as `CommandManager::issue_spell@00941b80`
/// writes it (item 4264).
fn spell(t: i32, ox: i32, whom: i32, at: Pos) -> Command {
    Command::Spell {
        ox,
        whom,
        spell: t,
        x: at.x,
        y: at.y,
    }
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

/// **The queue the selection's panel shows** (item 4265,
/// `docs/CLIENT_INPUT.md` §19): one building of `player`'s own, finished,
/// and the live entries of its production queue (`BuildData::queued`), one
/// queue button each; 0 for anything else. Ours: the panel's other rows
/// and a selection of several buildings are not drawn.
pub fn queue_panel(scene: &Scene, player: u8, selection: &[ObjId]) -> usize {
    let [id] = selection else {
        return 0;
    };
    scene
        .buildings
        .iter()
        .find(|b| b.id == *id && b.owner == player && b.active)
        .map_or(0, |b| b.queued)
}

/// **The city the selection can rename** (item 4268,
/// `docs/CLIENT_INPUT.md` §21): `Options::exec@007188c0`'s `0x117` arm
/// takes the group's first object (`GroupData::get_o`) and opens the
/// popup only for a building (`ObjectData +0x8 & 0x20`), finished (the
/// type's `is_active`), in a city (`BuildData +0x72 >= 0`). The layer asks
/// too that it be `player`'s own (`Group::can_activate` is not read), and
/// does not read the arm's `ObjectData::is` test. Its `o`, or none.
pub fn renames(scene: &Scene, player: u8, selection: &[ObjId]) -> Option<i16> {
    let id = selection.first()?;
    scene
        .buildings
        .iter()
        .find(|b| b.id == *id && b.owner == player && b.active && b.in_city)
        .map(|b| b.id.o)
}

/// **What a garrison pick issues** (item 4273, `docs/CLIENT_INPUT.md`
/// §23), on the building `b` it found: `Options::picked_spot@00721c40`'s
/// `OPTION_GARRISON` arm asks for a garrison limit
/// (`BuildTypeData::get_garrison_limit`, nonzero), then
/// `GroupData::test_garrison@0070b320`: a selection of units, not empty;
/// the first member on the map that `UnitData::can_garrison@006099a0`
/// admits (the building finished, `+0x4c`, and its type admitting the
/// member's, [`sim::observe::UnitScene::enters`]) decides, by whether
/// its slots fit (`num_inside + slots <= limit`). Then
/// `GroupOut::issue_garrison@0070a9b0(ox, whom, QUEUE_NEW)`. Ours: the
/// arm's second form, for a selection holding type `0x136`, is not read;
/// a building of an ally is not named, since the scene names no alliance.
pub fn garrisons(
    scene: &Scene,
    player: u8,
    selection: &[ObjId],
    b: &sim::observe::BuildingScene,
) -> Option<Command> {
    if b.garrison_limit == 0 || selection.is_empty() {
        return None;
    }
    let first = selection.iter().find_map(|id| {
        let u = scene
            .units
            .iter()
            .find(|u| u.id == *id && u.owner == player)?;
        (b.active && b.owner == u.owner && u.enters.contains(&b.id.o)).then_some(u)
    })?;
    (b.inside + first.garrison_cost <= b.garrison_limit).then_some(Command::Garrison {
        ox: i32::from(b.id.o),
        whom: i32::from(b.id.who),
        queued: QUEUE_NEW,
    })
}

/// **The `rename_city` a closed popup sends** (item 4268): `on_modal_end`'s
/// case 6 trims the spaces at both ends and sends nothing for a line left
/// empty; `CommandManager::issue_rename_city@009411d0` writes the console's
/// `who`, the object and the name cut to [`RENAME_SENT`] units, in a
/// package of its own. A command built here pads with zeros past the
/// terminator where the original's carries its stack (`after_nul`).
pub fn rename(who: u8, o: i16, line: &str) -> Option<Command> {
    let text = line.trim_matches(' ');
    if text.is_empty() {
        return None;
    }
    let mut name = String::new();
    for c in text.chars() {
        if name.encode_utf16().count() + c.len_utf16() > RENAME_SENT {
            break;
        }
        name.push(c);
    }
    Some(Command::RenameCity {
        who: i32::from(who),
        o: i32::from(o),
        name,
        after_nul: Vec::new(),
    })
}

/// **What a queue button issues** (item 4265): `Options::exec@007188c0`'s
/// `OPTION_QUEUE` arm walks the selection and, for each finished building
/// (`ObjectData::is_active_build`), calls `BuildOut::issue_unqueue
/// @00629e70(slot)`: `unqueue(who, o, slot, uid)`, with no `group`. The
/// layer asks that every member be a building of `player`'s in the scene
/// (`Group::can_activate` is not read); an unfinished one issues nothing,
/// and a selection with no finished building nothing at all.
pub fn unqueues(scene: &Scene, player: u8, selection: &[ObjId], slot: i32) -> Option<Vec<Command>> {
    let of = |id: &ObjId| scene.buildings.iter().find(|b| b.id == *id);
    if selection.is_empty()
        || !selection
            .iter()
            .all(|id| of(id).is_some_and(|b| b.owner == player))
    {
        return None;
    }
    let cmds: Vec<Command> = selection
        .iter()
        .filter_map(of)
        .filter(|b| b.active)
        .map(|b| Command::Unqueue {
            who: i32::from(b.owner),
            o: i32::from(b.id.o),
            unit: slot,
            uid: b.uid as i16,
        })
        .collect();
    (!cmds.is_empty()).then_some(cmds)
}

/// **The garrisoned units a building selection's panel shows** (item
/// 4280, `docs/CLIENT_INPUT.md` §26): of each selected building of the
/// slot's own, in selection order, the units its chain holds
/// ([`sim::observe::Scene::garrisons`]), each a unit of the slot's own.
/// Which of them the original's panel lays out, and in what order, is
/// read only as far as `Options::exec`'s arm: ours.
pub fn garrisoned(
    scene: &Scene,
    player: u8,
    selection: &[ObjId],
) -> Vec<sim::observe::GarrisonScene> {
    selection
        .iter()
        .filter(|id| {
            scene
                .buildings
                .iter()
                .any(|b| b.id == **id && b.owner == player)
        })
        .flat_map(|id| {
            scene
                .garrisons
                .iter()
                .filter(move |g| g.building == id.o && g.owner == player)
                .copied()
        })
        .collect()
}

/// **The icon's order** (item 4280): `Options::exec@007188c0` case
/// `0xaa` names the unit; `ObjectData::get_inside` finds its container,
/// and a container that is not a unit (vslot `0x18` answering 0, a
/// building) gives `UnitOut::issue_come_out@00607be0`: `come_out(who, o,
/// uid)`, the unit's own `+9`, `+0xa` and `+0x30`, with no group. A
/// container that is a unit is `do_eject_all`, which is not minted.
pub fn come_out(g: &sim::observe::GarrisonScene) -> Command {
    Command::ComeOut {
        who: i32::from(g.owner),
        o: i32::from(g.o),
        uid: g.uid as i16,
    }
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
    /// The walk itself, when the window has one: what a building's panel
    /// asks of the leader ([`Roster::offered`], item 4334). `None` in a
    /// scene built by hand, whose panel is the static roster.
    pub sim: Option<&'a sim::Sim>,
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

    /// **The box's buildings** (`Console::on_left_up@007cde90`, §7.1's
    /// second step; item 4318): walked only when the box took no unit.
    /// The issuing slot's buildings whose anchor is within [`BOX_MARGIN`]
    /// of the rectangle, in object order, the first [`SELECT_CAP`], each
    /// inserted before the first member of a greater type
    /// (`SelectGroup::add`): sorted by type, the building's `ty` being its
    /// `TypeIndex` less `BASE_BUILDTYPES`, and in object order within one.
    pub fn boxed_buildings(
        &self,
        who: u8,
        (x0, y0): (i32, i32),
        (x1, y1): (i32, i32),
    ) -> Vec<ObjId> {
        let (l, r) = (x0.min(x1) - BOX_MARGIN, x0.max(x1) + BOX_MARGIN);
        let (t, b) = (y0.min(y1) - BOX_MARGIN, y0.max(y1) + BOX_MARGIN);
        let mut inside: Vec<_> = self
            .scene
            .buildings
            .iter()
            .filter(|b| b.id.who == who)
            .filter(|bd| {
                let (ax, ay) = self.pixel(bd.pos.x, bd.pos.y, bd.z);
                l < ax && ax < r && t < ay && ay < b
            })
            .map(|b| (b.id, b.ty))
            .collect();
        inside.sort_by_key(|&(id, _)| id.o);
        inside.truncate(SELECT_CAP);
        inside.sort_by_key(|&(_, ty)| ty);
        inside.into_iter().map(|(id, _)| id).collect()
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

/// What a building selection's right press finds ([`View::gather_target`],
/// item 4301): a building of any slot, or one of the slot's own units.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GatherHit {
    /// The first look's building: its position is the gather point's,
    /// `action` 2 when its owner is the issuer's enemy and 1 otherwise.
    /// `university` is an active University (`is(0x1a4)`, `is_active`).
    Building {
        id: ObjId,
        pos: Pos,
        enemy: bool,
        university: bool,
    },
    /// The second look's unit: the point is `(captain, who)`, `action` 3.
    Unit { captain: i16, who: u8 },
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

    /// The building of `who`'s a pick at `(px, py)` names, as
    /// `Options::picked_spot@00721c40`'s `OPTION_GARRISON` arm looks
    /// (`find_target(SEARCH_ALLIED, buildings)`): the nearest anchor within
    /// [`PICK_RADIUS`], as [`View::target`]'s own look takes it (item 4273).
    pub fn own_building(&self, who: u8, px: i32, py: i32) -> Option<&sim::observe::BuildingScene> {
        self.scene
            .buildings
            .iter()
            .filter(|b| b.id.who == who)
            .filter_map(|b| {
                let (ax, ay) = self.pixel(b.pos.x, b.pos.y, b.z);
                let d = i64::from(ax - px).pow(2) + i64::from(ay - py).pow(2);
                (d <= i64::from(PICK_RADIUS).pow(2)).then_some((d, b))
            })
            .min_by_key(|&(d, b)| (d, b.id))
            .map(|(_, b)| b)
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

    /// **What a building selection's right press names** (item 4301,
    /// `docs/CLIENT_INPUT.md` §11.4), as `Console::execute_at_cursor`'s
    /// gather arm looks: first `find_target(SEARCH_ALL, buildings)`, a seen
    /// building of any slot; then `find_target(SEARCH_FRIENDLY, units)`,
    /// one of `who`'s own units (`Search::valid_search` case 1: the slot
    /// itself). Each look takes the nearest anchor within [`PICK_RADIUS`],
    /// as [`View::target`]'s do (ours: not the ±5 px box and bounding
    /// volumes). `None` is the ground.
    pub fn gather_target(&self, who: u8, px: i32, py: i32) -> Option<GatherHit> {
        let near = |p: Pos, z: i32| {
            let (ax, ay) = self.pixel(p.x, p.y, z);
            let d = i64::from(ax - px).pow(2) + i64::from(ay - py).pow(2);
            (d <= i64::from(PICK_RADIUS).pow(2)).then_some(d)
        };
        let building = self
            .scene
            .buildings
            .iter()
            .filter(|b| b.seen)
            .filter_map(|b| Some((near(b.pos, b.z)?, b)))
            .min_by_key(|&(d, b)| (d, b.id));
        if let Some((_, b)) = building {
            let enemy = self
                .scene
                .leaders
                .iter()
                .find(|l| l.who == who)
                .is_some_and(|l| l.enemies.contains(&b.owner));
            return Some(GatherHit::Building {
                id: b.id,
                pos: b.pos,
                enemy,
                university: b.university && b.active,
            });
        }
        self.scene
            .units
            .iter()
            .filter(|u| u.id.who == who && u.seen)
            .filter_map(|u| Some((near(u.pos, u.z)?, u)))
            .min_by_key(|&(d, u)| (d, u.id))
            .map(|(_, u)| GatherHit::Unit {
                captain: u.captain,
                who: u.id.who,
            })
    }

    /// `GroupData::count(COUNT_TYPE, VILLAGE)` of a selection, as far as
    /// nonzero: a selected building of the Village lineage, a city
    /// ([`sim::observe::BuildingScene::village`]; item 4301).
    pub fn selects_city(&self, selection: &[ObjId]) -> bool {
        selection.iter().any(|id| {
            self.scene
                .buildings
                .iter()
                .any(|b| b.id == *id && b.village)
        })
    }

    /// `GroupData::count(COUNT_ATTACK)` of a selection, as far as nonzero:
    /// whether any member on the map attacks ([`sim::observe::UnitScene::attacks`]).
    pub fn attacks(&self, selection: &[ObjId]) -> bool {
        selection
            .iter()
            .any(|id| self.scene.units.iter().any(|u| u.id == *id && u.attacks))
    }

    /// `Console::on_right_up@007cd3c0`'s first look (item 4295, §34.1):
    /// `GroupData::count(COUNT_TYPE, 0x3b)`, then `(COUNT_TYPE, 0x13e)`,
    /// a Caravan or a Merchant Fleet among the members on the map. Ours:
    /// the member's own `TypeIndex`, not the lineage `ObjectData::is`
    /// reads.
    pub fn trades_drag(&self, selection: &[ObjId]) -> bool {
        let traders = [
            sim::ids::ty::CARAVAN as i32,
            sim::ids::ty::MERCHANTFLEET as i32,
        ];
        selection.iter().any(|id| {
            self.scene
                .units
                .iter()
                .any(|u| u.id == *id && traders.contains(&u.type_index))
        })
    }
}

/// `OrderIndex::BUILD_AT` and `REPAIR`, the `swarm_around` kinds
/// (`sim::ids::order`).
const BUILD_AT: i32 = sim::ids::order::BUILD_AT as i32;
const REPAIR: i32 = sim::ids::order::REPAIR as i32;
/// The attack-move's `orders` byte, `OrderIndex::ATTACK_TO` (item 4297).
const ATTACK_TO: i8 = sim::ids::order::ATTACK_TO as i8;
/// `QUEUE_NEW`: every arm's queue with no modifier held.
const QUEUE_NEW: i32 = 2;
/// `QUEUE_LAST`: the shift's queue, the order after the ones standing
/// (item 4293, `docs/CLIENT_INPUT.md` §33).
const QUEUE_LAST: i32 = 1;
/// `QUEUE_FIRST`: the shift with alt in the dispatch, alt alone on the
/// minimap and in the placement (§33.1).
const QUEUE_FIRST: i32 = 0;

/// **The modifiers held at an event** (item 4293, `docs/CLIENT_INPUT.md`
/// §33): the three `GetKeyState` reads, shift `0x10`, ctrl `0x11` and alt
/// `0x12`, that each issuer takes at the press. The window sets them;
/// the layer reads them where the original's issuer does.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mods {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

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
/// the arm's `form` and `width`, and the issuer's queue.
fn move_to(at: Pos, form: i8, width: i8, queued: i32) -> Command {
    Command::MoveTo {
        to_x: at.x,
        to_y: at.y,
        set_angle: 0,
        angle: 0,
        orders: 1,
        queued: queued as i8,
        form,
        width,
        disembark: 0,
    }
}

/// **The attack-move's `move_to`** (item 4297, `docs/CLIENT_INPUT.md`
/// §35.1): `ATTACK_TO` to `at`, no facing, `form` and `width` −1, as
/// `Console::execute_at_cursor@007c6630` and `WorldMap::on_right_up@008c7050`
/// both issue it under ctrl, with the issuer's queue.
fn attack_to(at: Pos, queued: i32) -> Command {
    Command::MoveTo {
        to_x: at.x,
        to_y: at.y,
        set_angle: 0,
        angle: 0,
        orders: ATTACK_TO,
        queued: queued as i8,
        form: -1,
        width: -1,
        disembark: 0,
    }
}

/// **The building types whose minimap press launches a patrol** (item
/// 4302, §36.2): `WorldMap::on_right_up@008c7050` counts the selection's
/// `TypeIndex` `0x1bf` and `0x208` and issues `launch_patrol` for either,
/// the gather point for neither; as indices into the building types.
const PATROL_LAUNCHERS: [usize; 2] = [
    (0x1bf - rondata::load::BASE_BUILDTYPES) as usize,
    (0x208 - rondata::load::BASE_BUILDTYPES) as usize,
];

/// **One turn's package** (item 4302, `docs/CLIENT_INPUT.md` §36): every
/// order the turn's events minted, in the order they were issued, as one
/// package. The original's `issue_*` calls each append their `group` and
/// command to `CommandManager::local_package` (`add_group@0094bb60`,
/// `add_command@0094bae0`), and the single-player arm of
/// `process_turn@0093ef10` writes it once a turn when it holds anything.
/// The turn's orders share its frame and its seat; `None` when none was
/// minted.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "the window writes no file; the tests mint turns")
)]
pub fn one_package(orders: impl IntoIterator<Item = Order>) -> Option<Order> {
    let mut out: Option<Order> = None;
    for (frame, play, cmds) in orders {
        match &mut out {
            None => out = Some((frame, play, cmds)),
            Some(o) => {
                assert_eq!((o.0, o.1), (frame, play), "one turn, one frame, one seat");
                o.2.extend(cmds);
            }
        }
    }
    out
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
    /// The citizen's panel (item 4339, §32.7): `None` is `STANDARD_OPTIONS`,
    /// the page buttons; `Some(k)` is `BUILD_OPTIONS` at `mode_data` `k`,
    /// page `k`'s buildings. A new selection returns it to `None`, as
    /// `SelectGroup::clear` and `update_info` reach `Options::reset`.
    pub page: Option<i32>,
    /// The chat entry line, open with what it holds, or closed (item 4250).
    pub line: Option<String>,
    /// Whether the slot's own pause stands (item 4252): the turn control's
    /// bit, as the layer last set it. While it stands the window steps no
    /// frame, and every order is minted on the frame it stopped on.
    pub paused: bool,
    /// The craft whose button was pressed and whose target the next left
    /// press picks (`Options::selecting_spot`, item 4264).
    pub casting: Option<i32>,
    /// The last group this session sent (item 4267).
    pub sent: Sent,
    /// The city building whose name the entry line edits, while the
    /// rename popup is open (item 4268); `None` while the line is chat's.
    pub renaming: Option<i16>,
    /// Whether the garrison button was pressed and the next left press
    /// picks the building (`Options::selecting_spot` with
    /// `OPTION_GARRISON`, item 4273).
    pub garrisoning: bool,
    /// Whether the open entry line is the console's (item 4283): its
    /// Return runs a console word instead of sending a chat.
    pub console: bool,
    /// The player's profile prefs the frame-0 `leader_options` is made of
    /// (item 4289, `docs/CLIENT_INPUT.md` §26.1): the fresh profile's, and
    /// no control changes them (DECISIONS 136 §4).
    pub prefs: rondata::prefs::Prefs,
    /// The panel's roster (item 4290, `docs/CLIENT_INPUT.md` §32): what
    /// each building type's panel trains and researches and what the
    /// citizen's build pages offer. Empty until the window sets it from
    /// the install ([`Roster::new`]).
    pub roster: std::sync::Arc<Roster>,
    /// The modifiers held (item 4293, `docs/CLIENT_INPUT.md` §33), as the
    /// window last saw them.
    pub mods: Mods,
}

/// **The panel's roster** (item 4290, `docs/CLIENT_INPUT.md` §32): the
/// buttons the original's panels offer, read from the type tables with
/// the panels' static predicates, so [`Roster::trains`] offers every type
/// its building may ever queue. [`Roster::offered`] runs the leader's
/// (items 4315, 4321 and 4323), and the HUD draws it when the window
/// hands it the sim (item 4334).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Roster {
    /// Per building type (an index into the building types), the
    /// `TypeIndex`es its panel trains or researches, in `TypeIndex` order:
    /// every `t` for which `BuildTypeData::queue_here(b, t)` holds, within
    /// the two loops `BuildOut::make_options@00629ed0` runs it in (the
    /// units `0x32..0x192`, the technologies `BASE_AGETYPES..0x275`).
    pub trains: Vec<Vec<i32>>,
    /// The citizen's build pages (`UnitOut::make_options@00615fc0`'s
    /// `BUILD_OPTIONS` loop over `BASE_BUILDTYPES..=0x21e`), by their
    /// static predicates: `preq[0]` and `preq[1]` not disabled, and the
    /// line's base (no `FROM`) or a type with `BUILD_FLAGS & 4`. The page
    /// (`+0x14`), the leader's masks and `type_avail` are not read.
    pub builds: Vec<i32>,
    /// **The citizen's page loop's candidates** (item 4337,
    /// `docs/CLIENT_INPUT.md` §32.6): each building `TypeIndex` over
    /// `BASE_BUILDTYPES..=0x21e` whose `preq[0]` and `preq[1]` are not
    /// disabled, its `cat` (the page: `E` 0, `M` 1, `W` 2) and whether its
    /// `obj_masks` carries `0x4000000`. [`Roster::build_pages`] runs the
    /// leader's predicates over them.
    pub pages: Vec<(i32, i32, bool)>,
    /// The technologies `BuildOut::make_options@00629ed0`'s research loop
    /// runs over (`BASE_AGETYPES..0x275`, item 4323): each one's
    /// `TypeIndex`, tree id and `show[2]` (`SHOW0`, `SHOW1`).
    pub techs: Vec<(i32, usize, [sim::tech::Preq; 2])>,
    /// The Library's tree id: `make_options` adds every age and epoch
    /// (`0x220..0x243`) a Library's panel has not offered, greyed.
    pub library: Option<usize>,
}

/// `LIBRARY` (`enums/TypeIndex.txt` of the export), whose panel the
/// research loop's tail fills with the ages and the epochs.
const LIBRARY: i32 = 0x1b3;
/// `BASE_FINALTYPES`, the end of the Library's tail.
const BASE_FINALTYPES: i32 = 0x243;
/// The end of the research loop (the spells start at `0x275`).
const END_TECHTYPES: i32 = 0x275;
/// `TypeData::is_gov_type@004706a0`: `0x26e < type < 0x275`.
const GOV_TYPES: std::ops::Range<i32> = 0x26f..0x275;

/// The `TypeIndex`es `BuildOut::make_options@00629ed0`'s train loop names
/// (`enums/TypeIndex.txt` of the export).
const MILITIA: i32 = 0x42;
const RIFLEMAN: i32 = 0x64;
const INFANTRY: i32 = 0x66;
const MECHINFANTRY: i32 = 0x68;
const MACHINEGUN: i32 = 0x7b;
const BOMBARDSHIP: i32 = 0x15a;
const DREADNOUGHT: i32 = 0x15c;
const AIRCRAFTCARRIER: i32 = 0x15f;

/// `OPTION_TRAIN` and `OPTION_RESEARCH` (`OptionIndex`, the export's
/// `enums/`), the two kinds of button the panel's loops add.
pub const OPTION_TRAIN: i32 = 47;
pub const OPTION_RESEARCH: i32 = 48;
/// `OPTION_BUILD`, the citizen's first build page: what `UnitOut::make_options@00615fc0`
/// adds a page's buildings under (`mode_data`, 0), and, with type −1, the
/// standard panel's button that opens it (item 4339, §32.7).
pub const OPTION_BUILD: i32 = 0;
/// `OPTION_BUILD_MILITARY` and `OPTION_BUILD_WONDER`: the second and third
/// pages, as [`OPTION_BUILD`] the first (`mode_data` 1 and 2).
pub const OPTION_BUILD_MILITARY: i32 = 1;
pub const OPTION_BUILD_WONDER: i32 = 2;
/// The page's own buttons (item 4339, §32.7), as the page loop adds them
/// after its buildings: `OPTION_BACK` at grid (2, 3), `OPTION_NEXT` at
/// (3, 3) and `OPTION_CANCEL` at (4, 3) (`panels.py --citizens`' rows; the
/// values are the TSV's, not the add order).
pub const OPTION_NEXT: i32 = 65;
pub const OPTION_BACK: i32 = 66;
pub const OPTION_CANCEL: i32 = 67;
/// **The standard panel's other adds** (item 4346, `docs/CLIENT_INPUT.md`
/// §32.9), the PDB's `OptionType` (`LF_ENUMERATE`): what
/// `UnitOut::make_options@00615fc0` adds a citizen beside its page buttons.
pub const OPTION_REPAIR: i32 = 3;
pub const OPTION_SPELL: i32 = 4;
pub const OPTION_MOVE_TO: i32 = 9;
pub const OPTION_ATTACK_TO: i32 = 10;
pub const OPTION_TRANSPORT: i32 = 14;
pub const OPTION_HALT: i32 = 25;
pub const OPTION_GARRISON: i32 = 26;
pub const OPTION_ALARM_PEASANT: i32 = 27;
pub const OPTION_AUTO_EXPLORE: i32 = 33;

/// **A citizen's standard panel** (item 4346, §32.9): `UnitOut::make_options
/// @00615fc0` in `STANDARD_OPTIONS` mode for unit `u` of slot `who`, each
/// add `(event, button)` in the add order. The page buttons (item 4339);
/// `OPTION_REPAIR`; `OPTION_ALARM_PEASANT`, the unit not inside a carrier;
/// an `OPTION_SPELL` for each craft the unit offers
/// ([`sim::observe::UnitScene::crafts`]), typed with the craft; then
/// `OPTION_GARRISON`, `OPTION_HALT`, `OPTION_TRANSPORT` greyed while the
/// leader's transport level (`leader_flags` `0x100` 3, `0x200` 2, `0x400`
/// 1) is under the unit's `transport_type`, `OPTION_MOVE_TO`,
/// `OPTION_ATTACK_TO` and `OPTION_AUTO_EXPLORE`. The predicates of those
/// last six are a citizen's on every held packet (a land unit, not a plane,
/// with an attack, the profile's `prefs & 2` clear) and are not built for
/// another unit. `OPTION_REPAIR`, `OPTION_MOVE_TO`, `OPTION_ATTACK_TO` arm
/// a pick and `OPTION_TRANSPORT` writes `set_transport`: [`Event::Unbuilt`].
fn standard_panel(sim: &sim::Sim, scene: &Scene, u: usize) -> Vec<(Event, Button)> {
    let unit = &sim.units[u];
    let typeless = |option, disable| Button {
        ty: -1,
        option,
        disable,
    };
    let mut out: Vec<(Event, Button)> = (0..pages_of(sim))
        .map(|o| (Event::Page(o), typeless(o, 0)))
        .collect();
    out.push((Event::Unbuilt(OPTION_REPAIR), typeless(OPTION_REPAIR, 0)));
    out.push((Event::Alarm, typeless(OPTION_ALARM_PEASANT, 0)));
    let id = ObjId {
        who: unit.owner,
        o: unit.index,
    };
    if let Some(s) = scene.units.iter().find(|s| s.id == id) {
        out.extend(s.crafts.iter().map(|c| {
            let b = Button {
                ty: c.t,
                option: OPTION_SPELL,
                disable: 0,
            };
            (Event::Spell(c.t), b)
        }));
    }
    out.push((Event::Garrison, typeless(OPTION_GARRISON, 0)));
    out.push((Event::Halt, typeless(OPTION_HALT, 0)));
    let low = sim.transport_level(unit.owner) < sim.unit_transport_type(u);
    out.push((
        Event::Unbuilt(OPTION_TRANSPORT),
        typeless(OPTION_TRANSPORT, i32::from(low)),
    ));
    out.push((Event::Unbuilt(OPTION_MOVE_TO), typeless(OPTION_MOVE_TO, 0)));
    out.push((
        Event::Unbuilt(OPTION_ATTACK_TO),
        typeless(OPTION_ATTACK_TO, 0),
    ));
    out.push((Event::AutoExplore, typeless(OPTION_AUTO_EXPLORE, 0)));
    out
}

/// The `TypeIndex`es `UnitOut::make_options@00615fc0`'s page loop names
/// (`enums/TypeIndex.txt` of the export): `VILLAGE`, the City line whose
/// types `at_city_limit` greys; `FORBIDDENCITY`, exempt; the wonders,
/// `BASE_WONDERTYPES..=SPACEPROGRAM` (`BuildTypeData::already_built`'s
/// range, and `is_wonder_type`'s).
const VILLAGE: i32 = 0x19e;
const FORBIDDENCITY: i32 = 0x213;
const WONDERS: std::ops::RangeInclusive<i32> = 0x20e..=0x21e;
/// `PEASANTS` and `PEASANTSKOREAN`: the citizens `make_options` offers the
/// build pages to.
const CITIZENS: [i32; 2] = [0x32, 0x33];

/// **The pages a citizen turns through** (item 4339, §32.7): three, or two
/// in a game whose `semaphore[2] & 2` is set (a Conquer-the-World or
/// scenario game, [`sim::ai::Lobby::conquest`]), where `make_options`
/// adds no `OPTION_BUILD_WONDER` and `Options::exec`'s `OPTION_NEXT` and
/// `OPTION_BACK` turn modulo `~(semaphore[2] >> 1) & 1 | 2`.
fn pages_of(sim: &sim::Sim) -> i32 {
    if sim.ai.lobby.conquest { 2 } else { 3 }
}

/// **One press of a typeless page button** (items 4339, 4343, §32.7,
/// §32.8): the page `Options::exec@007188c0` leaves a citizen's panel on,
/// `None` for `STANDARD_OPTIONS`, from `page` with `n` pages. Cases 0 to 2
/// open their page (under two pages, `OPTION_BUILD_WONDER` opens nothing);
/// 0x41 turns forward modulo `n`, 0x42 back with the wrap, both on a page
/// only; 0x43 returns a page to the standard panel. Every arm is run on
/// packets by `panels.py --turns` (`PAGE_TURNS_RUN`).
fn turn(option: i32, page: Option<i32>, n: i32) -> Option<i32> {
    match (option, page) {
        (OPTION_BUILD..=OPTION_BUILD_WONDER, _) if option < n => Some(option),
        (OPTION_NEXT, Some(k)) => Some((k + 1) % n),
        (OPTION_BACK, Some(k)) => Some(if k < 1 { n - 1 } else { k - 1 }),
        (OPTION_CANCEL, Some(_)) => None,
        (_, page) => page,
    }
}

/// **A panel button** (item 4326, `docs/CLIENT_INPUT.md` §32.4): what
/// `Options::add@0072c290` takes of one of `make_options`' train and
/// research adds, the grid aside: the type, the option and `disable`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Button {
    pub ty: i32,
    pub option: i32,
    pub disable: i32,
}

impl Roster {
    /// **The panel through the leader's predicates** (items 4315 and 4321,
    /// `docs/CLIENT_INPUT.md` §32.4): building type `building`'s roster for
    /// slot `who` on `sim`'s state. An unfinished building (`active`
    /// false) offers nothing: both loops stand inside `make_options`'
    /// `is_active` block. `BuildOut::make_options@00629ed0`'s train loop
    /// over the units `0x32..0x192`, in its order:
    ///
    /// 1. `obj_masks & 0x4000000` (a patriot or hero) is never offered;
    /// 2. a unit whose line's predecessor (`+0x3c`, the type's `FROM`,
    ///    through `get_graft` when the tribe cannot have it) fails
    ///    `LeaderData::check_predecessor` is not offered, unless the unit
    ///    carries `unit_flags & 0x200` and the leader `has_preq` it;
    /// 3. a Bombard Ship's line short of the Dreadnought, once the
    ///    Aircraft Carrier is `type_avail`, is not offered; a Machine Gun's
    ///    line only while the leader owns the Rifleman, the Infantry or the
    ///    Mechanized Infantry (each through its graft);
    /// 4. `BuildData::can_make@0062db10`'s value, which for a unit on a
    ///    finished building is `type_avail(t, 1)`: 0 offers a greyed research button under `type_eligible(t, 1)`
    ///    and the age gate (`get_age` at most the leader's `ages + 1`, or
    ///    the type `is` the Militia), unless the unit is not `unit_flags &
    ///    0x80` without a `FROM` and is being researched; 2 offers research
    ///    unless `researching`; 4 offers training unless `unit_flags &
    ///    0x8000`.
    ///
    /// Then the technologies, [`Roster::researched`] (item 4323). Each
    /// button carries its option and `disable` (item 4326): `can_make` 0
    /// greys (`disable` 2), as `OPTION_TRAIN` for a unit `unit_flags &
    /// 0x80` without a `FROM` and `OPTION_RESEARCH` otherwise; 2 is
    /// `OPTION_RESEARCH` and 4 `OPTION_TRAIN`, both `disable` 0 on a
    /// building `is_neutralized` (vtable `+0x180`, `Build`'s answering 0)
    /// leaves clear. The HUD draws it ([`Input::panel`], item 4334).
    pub fn offered(&self, sim: &sim::Sim, who: usize, building: i32, active: bool) -> Vec<Button> {
        use rondata::load::{BASE_BUILDTYPES, BASE_UNITTYPES};
        let all = self
            .trains
            .get((building - BASE_BUILDTYPES) as usize)
            .cloned()
            .unwrap_or_default();
        let (tree, setup) = (&sim.tables.tech_tree, &sim.tables.setup);
        // Both loops stand inside `make_options`' `is_active` block: an
        // unfinished building offers neither.
        if !active {
            return Vec::new();
        }
        let Some(p) = sim.tech.get(who) else {
            return all
                .into_iter()
                .map(|ty| Button {
                    ty,
                    option: OPTION_TRAIN,
                    disable: 0,
                })
                .collect();
        };
        // A unit's `TypeIndex` to its record and its tree id.
        let unit = |ti: i32| {
            let u = sim
                .tables
                .unit_types
                .get(usize::try_from(ti - BASE_UNITTYPES).ok()?)?;
            Some((u, u.tree?))
        };
        let id = |ti: i32| unit(ti).map(|(_, t)| t);
        let owns = |ti: i32| tree.get_graft(setup, p, id(ti)).is_some_and(|x| p.tech[x]);
        let button = |ty: i32, option: i32, disable: i32| {
            Some(Button {
                ty,
                option,
                disable,
            })
        };
        let mut out: Vec<Button> = all
            .into_iter()
            .filter_map(|ti| {
                if !(BASE_UNITTYPES..BASE_BUILDTYPES).contains(&ti) {
                    return None;
                }
                let (u, t) = unit(ti)?;
                let flags = u.cols.unit_flags;
                if u.combat.obj_masks & 0x400_0000 != 0 {
                    return None;
                }
                if !(flags & 0x200 != 0 && tree.has_preq(setup, p, t))
                    && !tree.check_predecessor(
                        setup,
                        p,
                        tree.get_graft(setup, p, tree.types[t].from),
                    )
                {
                    return None;
                }
                let is = |x: i32| id(x).is_some_and(|x| tree.is(t, x, false));
                if is(BOMBARDSHIP)
                    && !is(DREADNOUGHT)
                    && id(AIRCRAFTCARRIER).is_some_and(|c| sim.type_avail(who as _, c) != 0)
                {
                    return None;
                }
                if is(MACHINEGUN) && !(owns(RIFLEMAN) || owns(INFANTRY) || owns(MECHINFANTRY)) {
                    return None;
                }
                match sim.type_avail(who as _, t) {
                    0 => {
                        if tree.type_eligible(setup, p, t, true) == 0 {
                            return None;
                        }
                        let free = flags & 0x80 != 0 && tree.types[t].from.is_none();
                        if !free && sim.researching(who as _, t) {
                            return None;
                        }
                        if !(tree.age_in(setup, t) <= p.ages + 1 || is(MILITIA)) {
                            return None;
                        }
                        // `OPTION_RESEARCH - free`, greyed with 2.
                        button(ti, OPTION_RESEARCH - i32::from(free), 2)
                    }
                    2 => {
                        if sim.researching(who as _, t) {
                            return None;
                        }
                        button(ti, OPTION_RESEARCH, 0)
                    }
                    _ => {
                        if flags & 0x8000 != 0 {
                            return None;
                        }
                        button(ti, OPTION_TRAIN, 0)
                    }
                }
            })
            .collect();
        if let Some(b) = sim
            .tables
            .build_types
            .get((building - BASE_BUILDTYPES) as usize)
            .and_then(|b| b.tree)
        {
            out.extend(
                self.researched(sim, who, b)
                    .into_iter()
                    .map(|(ty, disable)| Button {
                        ty,
                        option: OPTION_RESEARCH,
                        disable,
                    }),
            );
        }
        out.sort_unstable();
        out.dedup();
        out
    }

    /// **The citizen's build pages** (item 4337, `docs/CLIENT_INPUT.md`
    /// §32.6): what `UnitOut::make_options@00615fc0` adds in
    /// `BUILD_OPTIONS` mode for slot `who`, its three pages (`mode_data`
    /// 0, 1, 2, each button's option) at once. The loop reads only the
    /// citizen's slot. For each of [`Roster::pages`], in order:
    ///
    /// 1. a type with a `FROM` is offered only with `obj_masks &
    ///    0x4000000`, or with `BUILD_FLAGS & 4` and `type_avail(t, 1)` 4;
    /// 2. a type the leader's `obs_flags` marks, or whose `obs` (`+0x4c`)
    ///    the leader `has_tech`, is not;
    /// 3. a wonder `already_built` is not;
    /// 4. a type that `is(VILLAGE)`, not `is(FORBIDDENCITY)` nor the
    ///    Forbidden City itself, is greyed `at_city_limit`;
    /// 5. its page: `cat` 0 or 1 on page `cat` unless a wonder; `cat` 2
    ///    on page 2, a wonder only with no site of it placed
    ///    (`get_queued`);
    /// 6. `disable` 1 when `type_avail(t, 1)` is under 4 or the limit greys
    ///    it, else 0.
    pub fn build_pages(&self, sim: &sim::Sim, who: usize) -> Vec<Button> {
        use rondata::load::BASE_BUILDTYPES;
        let (tree, setup) = (&sim.tables.tech_tree, &sim.tables.setup);
        let Some(p) = sim.tech.get(who) else {
            return Vec::new();
        };
        let rec = |ti: i32| usize::try_from(ti - BASE_BUILDTYPES).ok();
        let id = |ti: i32| sim.tables.build_types.get(rec(ti)?)?.tree;
        let (village, forbidden) = (id(VILLAGE), id(FORBIDDENCITY));
        let at_limit = sim.at_city_limit(who as sim::Player);
        let line = &tree.roles.wonder_line;
        let mut out = Vec::new();
        for &(ti, cat, hero) in &self.pages {
            let (Some(r), Some(t)) = (rec(ti), id(ti)) else {
                continue;
            };
            let bt = &sim.tables.build_types[r];
            let avail = sim.type_avail(who as sim::Player, t);
            if bt.from.is_some()
                && !hero
                && !(bt.has(sim::cities::build::flags::UPGRADES) && avail >= 4)
            {
                continue;
            }
            if p.obs[t] || tree.has_tech_p(setup, p, tree.types[t].obs) {
                continue;
            }
            let wonder = WONDERS.contains(&ti);
            let built = line
                .iter()
                .position(|&w| w == t)
                .is_some_and(|k| sim.cities.wonders_built & (1 << k) != 0);
            if wonder && built {
                continue;
            }
            let limited = village.is_some_and(|v| tree.is(t, v, false))
                && !forbidden.is_some_and(|f| tree.is(t, f, false))
                && ti != FORBIDDENCITY
                && at_limit;
            let on_page = match cat {
                0 | 1 => !wonder,
                2 => !wonder || sim.num_sites_of(who as sim::Player, r) == 0,
                _ => false,
            };
            if !on_page {
                continue;
            }
            out.push(Button {
                ty: ti,
                option: cat,
                disable: i32::from(avail < 4 || limited),
            });
        }
        out.sort_unstable();
        out
    }

    /// **The research loop** (item 4323, `docs/CLIENT_INPUT.md` §32.4):
    /// the technologies `BuildOut::make_options@00629ed0` offers on
    /// building `b` (a tree id) for slot `who`, its three passes over
    /// `BASE_AGETYPES..0x275` taken as one set (the passes order the
    /// buttons and grey them; each type reaching the add is added in
    /// exactly one). For each technology `t`:
    ///
    /// 1. held (`has_tech`) and queued here: offered only for a
    ///    government (`is_gov_type`, greyed);
    /// 2. else `researching` anywhere: not offered;
    /// 3. else, with `avail = type_avail(t, 1)`: not queued here, it is
    ///    offered only through the building's `upgrade` (`+0x44`, through
    ///    its graft) queueing it, and at `avail` 0 only with `show[0]` or
    ///    `show[1]` held;
    /// 4. `avail` under 2 asks `show[0]` or `show[1]` held, or
    ///    `BuildData::show_later_tech@0062d920` to depth 5, and of a
    ///    government `type_eligible(t, 1)`.
    ///
    /// Then a Library (`is(LIBRARY)`) adds every age and epoch
    /// (`0x220..0x243`) not yet on its panel, greyed.
    ///
    /// Each with its `disable` (item 4326): the arms gather flags, the
    /// `upgrade` arm 5 (and 8 at `avail` 1), the `avail < 2` arm 3, and a
    /// button is clear (0) only with no flag set, a type queued here at
    /// `avail` 2 or more; the governments and the Library's tail are 1.
    fn researched(&self, sim: &sim::Sim, who: usize, b: usize) -> Vec<(i32, i32)> {
        let (tree, setup) = (&sim.tables.tech_tree, &sim.tables.setup);
        let Some(p) = sim.tech.get(who) else {
            return Vec::new();
        };
        let has = |q: sim::tech::Preq| tree.has_tech_p(setup, p, q);
        let up = tree.get_graft(setup, p, tree.types[b].upgrade);
        let mut out = Vec::new();
        for &(ti, t, show) in &self.techs {
            if tree.has_tech(setup, p, t) && tree.queue_here(b, t) {
                if GOV_TYPES.contains(&ti) {
                    out.push((ti, 1));
                }
                continue;
            }
            if sim.researching(who as _, t) {
                continue;
            }
            let avail = sim.type_avail(who as _, t);
            let shown = has(show[0]) || has(show[1]);
            let here = tree.queue_here(b, t);
            if !here {
                if !up.is_some_and(|u| tree.queue_here(u, t)) {
                    continue;
                }
                if avail == 0 && !shown {
                    continue;
                }
            }
            if avail < 2 {
                if !shown && !self.show_later_tech(sim, p, b, t, 5) {
                    continue;
                }
                if GOV_TYPES.contains(&ti) && tree.type_eligible(setup, p, t, true) == 0 {
                    continue;
                }
            }
            out.push((ti, i32::from(!(here && avail >= 2))));
        }
        if self.library.is_some_and(|l| tree.is(b, l, false)) {
            for &(ti, ..) in &self.techs {
                if ti < BASE_FINALTYPES && !out.iter().any(|&(x, _)| x == ti) {
                    out.push((ti, 1));
                }
            }
        }
        out
    }

    /// `BuildData::show_later_tech@0062d920`: technology `t` is shown
    /// ahead of its time on building `b` when the game's ending age
    /// reaches its `AGE` and each prerequisite (`get_preq(i, who)`) is
    /// held, or is itself shown later at `depth − 1` without being queued
    /// here or at the Library. A disabled slot answers no.
    fn show_later_tech(
        &self,
        sim: &sim::Sim,
        p: &sim::tech::PlayerTech,
        b: usize,
        t: usize,
        depth: i32,
    ) -> bool {
        use sim::tech::Preq;
        let (tree, setup) = (&sim.tables.tech_tree, &sim.tables.setup);
        if setup.ending < tree.types[t].age {
            return false;
        }
        for i in 0..tree.num_preq(t) {
            match tree.get_preq(setup, Some(p), t, i) {
                Preq::Disabled => return false,
                Preq::None => {}
                Preq::Of(q) => {
                    if tree.has_tech(setup, p, q) {
                        continue;
                    }
                    if tree.queue_here(b, q)
                        || (tree.types[q].where_.is_some() && tree.types[q].where_ == self.library)
                        || depth - 1 < 0
                        || !self.show_later_tech(sim, p, b, q, depth - 1)
                    {
                        return false;
                    }
                }
            }
        }
        true
    }

    pub fn new(loaded: &rondata::load::Loaded) -> Roster {
        use rondata::load::{BASE_BUILDTYPES, BASE_TECHTYPES};
        let nb = loaded.build_types.len() as i32;
        let candidates: Vec<(i32, usize)> = (0x32..0x192)
            .chain(BASE_TECHTYPES..0x275)
            .filter_map(|t| Some((t, tree_id(loaded, t)?)))
            .collect();
        let trains = (0..nb)
            .map(|b| {
                let Some(bid) = tree_id(loaded, BASE_BUILDTYPES + b) else {
                    return Vec::new();
                };
                candidates
                    .iter()
                    .filter(|&&(_, t)| loaded.tree.queue_here(bid, t))
                    .map(|&(ti, _)| ti)
                    .collect()
            })
            .collect();
        let builds = (BASE_BUILDTYPES..=0x21e.min(BASE_BUILDTYPES + nb - 1))
            .filter(|&ti| {
                let (Some(t), Some(b)) = (tree_id(loaded, ti), loaded.build_of_type_index(ti))
                else {
                    return false;
                };
                let def = &loaded.tree.types[t];
                let bt = &loaded.build_types[b];
                def.preq[0] != sim::tech::Preq::Disabled
                    && def.preq[1] != sim::tech::Preq::Disabled
                    && (bt.from.is_none() || bt.has(sim::cities::build::flags::UPGRADES))
            })
            .collect();
        let pages = (BASE_BUILDTYPES..=0x21e.min(BASE_BUILDTYPES + nb - 1))
            .filter_map(|ti| {
                let (t, b) = (tree_id(loaded, ti)?, loaded.build_of_type_index(ti)?);
                let def = &loaded.tree.types[t];
                if def.preq[0] == sim::tech::Preq::Disabled
                    || def.preq[1] == sim::tech::Preq::Disabled
                {
                    return None;
                }
                let hero = loaded.build_types[b]
                    .combat
                    .as_ref()
                    .is_some_and(|c| c.obj_masks & 0x400_0000 != 0);
                Some((ti, *loaded.build_cat.get(b)?, hero))
            })
            .collect();
        let techs = (BASE_TECHTYPES..END_TECHTYPES)
            .filter_map(|ti| {
                let t = tree_id(loaded, ti)?;
                let show = *loaded.tech_show.get((ti - BASE_TECHTYPES) as usize)?;
                Some((ti, t, show))
            })
            .collect();
        let library = tree_id(loaded, LIBRARY);
        Roster {
            trains,
            builds,
            pages,
            techs,
            library,
        }
    }

    /// The building types (indices) whose panel queues `ti`.
    #[cfg_attr(not(test), expect(dead_code, reason = "the census reads it"))]
    pub fn trained_at(&self, ti: i32) -> Vec<usize> {
        (0..self.trains.len())
            .filter(|&b| self.trains[b].contains(&ti))
            .collect()
    }
}

/// The tree id a `TypeIndex` names: the inverse of
/// `rondata::load::Loaded::type_index`.
fn tree_id(loaded: &rondata::load::Loaded, ti: i32) -> Option<usize> {
    use rondata::load::{BASE_BUILDTYPES, BASE_TECHTYPES, BASE_UNITTYPES};
    let (g, u) = (loaded.good_tree.len(), loaded.unit_tree.len());
    let b = u + loaded.build_tree.len();
    let id = if ti >= BASE_TECHTYPES {
        usize::try_from(ti - BASE_TECHTYPES).ok().map(|i| g + b + i)
    } else if ti >= BASE_BUILDTYPES {
        usize::try_from(ti - BASE_BUILDTYPES)
            .ok()
            .map(|i| g + u + i)
    } else {
        usize::try_from(ti - BASE_UNITTYPES).ok().map(|i| g + i)
    }?;
    (id < loaded.tree.types.len() && loaded.type_index(id) == ti).then_some(id)
}

/// **The last group sent** (item 4267, `docs/CLIENT_INPUT.md` §20):
/// `CommandPackage`'s `last_who_sent`, `last_objects_sent` and
/// `last_uids_sent`, which `init_class@0094c440` sets to −1 and no objects
/// when a game starts. `CommandPackage::add_group@0094bb60` writes a group
/// whose `who`, objects and their uids (`ObjectData +0x30`) all equal these
/// as a count of zero, and any other group in full, keeping it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sent {
    pub who: i8,
    /// Each object and its uid, in the group's order.
    pub objects: Vec<(i16, u16)>,
}

impl Default for Sent {
    fn default() -> Sent {
        Sent {
            who: -1,
            objects: Vec::new(),
        }
    }
}

impl Sent {
    /// `add_group`'s write: the `group` for `who`'s `objects`, as no
    /// objects when it repeats the last one sent.
    pub fn add_group(&mut self, who: i8, objects: Vec<(i16, u16)>) -> Command {
        if who == self.who && objects == self.objects {
            return Command::Group {
                who,
                objects: Vec::new(),
            };
        }
        let list = objects.iter().map(|&(o, _)| o).collect();
        (self.who, self.objects) = (who, objects);
        Command::Group { who, objects: list }
    }
}

/// An object's uid as the scene holds it; `u16::MAX` for one it does not
/// (ours: `add_group` drops a dead object, and the layer writes it).
fn uid_of(scene: &sim::observe::Scene, id: ObjId) -> u16 {
    scene
        .units
        .iter()
        .find(|u| u.id == id)
        .map(|u| u.uid)
        .or_else(|| scene.buildings.iter().find(|b| b.id == id).map(|b| b.uid))
        .unwrap_or(u16::MAX)
}

/// **The selection's craft** (item 4264, `docs/CLIENT_INPUT.md` §18): the
/// craft `t` as a unit of the selection offers it, `None` when no member
/// does — the layer's stand-in for `GroupData::get_spellcaster` and
/// `get_valid_spellcaster`, which `do_spell` asks before it issues.
pub fn selection_craft(
    scene: &sim::observe::Scene,
    selection: &[ObjId],
    t: i32,
) -> Option<sim::observe::Craft> {
    selection.iter().find_map(|id| {
        let u = scene.units.iter().find(|u| u.id == *id)?;
        u.crafts.iter().copied().find(|c| c.t == t)
    })
}

impl Input {
    pub fn new(player: u8) -> Input {
        Input {
            player,
            ..Input::default()
        }
    }

    /// **The panel's buttons for the selection** (item 4290,
    /// `docs/CLIENT_INPUT.md` §32), each its event, its option and its
    /// `disable`: a [`Event::Train`] for a building selection and a
    /// [`Event::Place`] for units. A building selected, the panel of the
    /// first of the slot's own buildings in it: with `sim`, the original's
    /// ([`Roster::offered`] on the building's state, item 4334), and
    /// without, the static roster, every type a clear `OPTION_TRAIN`.
    /// Units selected: with `sim` and one of the slot's citizens among
    /// them, the original's three build pages at once ([`Roster::build_pages`],
    /// item 4337), else nothing; without `sim`, the static pages, clear,
    /// for any unit selection, as [`BUILD_KEYS`] does.
    pub fn panel(&self, scene: &Scene, sim: Option<&sim::Sim>) -> Vec<(Event, Button)> {
        if self.selection.is_empty() || self.line.is_some() {
            return Vec::new();
        }
        let clear = |ty, option| Button {
            ty,
            option,
            disable: 0,
        };
        if !self.building {
            let Some(sim) = sim else {
                return self
                    .roster
                    .builds
                    .iter()
                    .map(|&t| (Event::Place(t), clear(t, OPTION_BUILD)))
                    .collect();
            };
            // With the walk (item 4337): the original's three pages, for a
            // selection holding one of the slot's own citizens.
            let citizen = self.selection.iter().find_map(|id| {
                if id.who != self.player {
                    return None;
                }
                sim.units.iter().position(|u| {
                    u.alive()
                        && u.owner == self.player
                        && u.index == id.o
                        && u.ty.is_some_and(|t| {
                            CITIZENS.contains(&(t as i32 + rondata::load::BASE_UNITTYPES))
                        })
                })
            });
            let Some(citizen) = citizen else {
                return Vec::new();
            };
            // One page at a time (item 4339): the standard arm's page
            // buttons, or page `k`'s buildings and its three buttons.
            let page = |option| Button {
                ty: -1,
                option,
                disable: 0,
            };
            // The standard panel whole (item 4346): the page buttons and
            // the citizen's other adds.
            let Some(k) = self.page else {
                return standard_panel(sim, scene, citizen);
            };
            let mut out: Vec<(Event, Button)> = self
                .roster
                .build_pages(sim, usize::from(self.player))
                .into_iter()
                .filter(|x| x.option == k)
                .map(|x| (Event::Place(x.ty), x))
                .collect();
            out.extend(
                [OPTION_BACK, OPTION_NEXT, OPTION_CANCEL].map(|o| (Event::Page(o), page(o))),
            );
            return out;
        }
        let Some(b) = self.selection.iter().find_map(|id| {
            scene
                .buildings
                .iter()
                .find(|b| b.id == *id && b.owner == self.player && b.ty.is_some())
        }) else {
            return Vec::new();
        };
        let ty = b.ty.unwrap_or_default();
        match sim {
            Some(sim) => self
                .roster
                .offered(
                    sim,
                    usize::from(self.player),
                    ty as i32 + rondata::load::BASE_BUILDTYPES,
                    b.active,
                )
                .into_iter()
                .map(|x| (Event::Train(x.ty), x))
                .collect(),
            None => self
                .roster
                .trains
                .get(ty)
                .map(|r| {
                    r.iter()
                        .map(|&t| (Event::Train(t), clear(t, OPTION_TRAIN)))
                        .collect()
                })
                .unwrap_or_default(),
        }
    }

    /// **The frame-0 order** (item 4289, `docs/CLIENT_INPUT.md` §26.1): no
    /// button issues it. `Game::run@00584590`'s `LeaderOptionOut::update`
    /// issues `MiscAccess::my_leader_option` once, at the game's start,
    /// after `LeaderOption::synch_with_prefs` has read the profile; the
    /// order's `who` is `MiscAccess::console->who`, the human player's
    /// `who`. That is not always the package's `play` ([`Input::player`]):
    /// the 2017 sample's human is `who` 3 on play 0, and every lobby of
    /// ours is `who` 0 on play 0.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "the census mints it; no window writes a file yet")
    )]
    pub fn leader_options(&self, who: u8) -> Command {
        self.prefs.issue_leader_option(i32::from(who))
    }

    /// The `group` the selection makes, as `add_group` writes it: no
    /// objects when it repeats the last group sent ([`Sent`]).
    fn group(&mut self, scene: &sim::observe::Scene) -> Command {
        let objects = self
            .selection
            .iter()
            .map(|&id| (id.o, uid_of(scene, id)))
            .collect();
        self.sent.add_group(self.player as i8, objects)
    }

    /// **A left press's selection** (`Console::on_left_up@007cde90`'s
    /// click, §1): the issuing slot's object whose anchor is nearest the
    /// pixel within [`PICK_RADIUS`], or none, which clears it.
    fn press_select(&mut self, view: &View, x: i32, y: i32) {
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
        self.page = None;
    }

    /// **The dispatch's queue** (item 4293, `docs/CLIENT_INPUT.md` §33.1):
    /// `Console::execute_at_cursor@007c6630` sets `QUEUE_NEW` and, with
    /// shift held, `alt ^ 1`: `QUEUE_LAST`, or `QUEUE_FIRST` with alt too.
    /// Alt is read only while the profile's `prefs & 0x400` is clear, as
    /// the fresh profile's is.
    fn dispatch_queue(&self) -> i32 {
        let alt = self.mods.alt && self.prefs.prefs & 0x400 == 0;
        match (self.mods.shift, alt) {
            (false, _) => QUEUE_NEW,
            (true, false) => QUEUE_LAST,
            (true, true) => QUEUE_FIRST,
        }
    }

    /// **The minimap's queue** (§33.1): `WorldMap::on_right_up@008c7050`
    /// sets `QUEUE_LAST` with shift, else `QUEUE_FIRST` with alt, else
    /// `QUEUE_NEW`; `Options::do_formation@007215b0` reads the same.
    fn map_queue(&self) -> i32 {
        if self.mods.shift {
            QUEUE_LAST
        } else if self.mods.alt {
            QUEUE_FIRST
        } else {
            QUEUE_NEW
        }
    }

    /// **The placement's queue, and whether it stays armed** (§33.1):
    /// `Options::picked_spot@00721c40`'s build arm. A wonder
    /// (`TypeData::is_wonder_type`) resets the placement and takes the
    /// minimap's queue. Any other type stays armed with shift or ctrl
    /// held, and then its queue is `QUEUE_FIRST` with ctrl, overruled by
    /// `QUEUE_LAST` with shift; with neither, it resets with the
    /// minimap's queue.
    fn placed_queue(&self, ty: i32) -> (i32, bool) {
        let q = self.map_queue();
        if sim::core::type_index::is_wonder_type(ty) || !(self.mods.shift || self.mods.ctrl) {
            return (q, false);
        }
        let q = if self.mods.shift {
            QUEUE_LAST
        } else {
            QUEUE_FIRST
        };
        (q, true)
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
        let queued = self.dispatch_queue();
        match hit {
            Hit::Enemy { id, .. } if attacks => Some(Command::Attack {
                ox: i32::from(id.o),
                whom: i32::from(id.who),
                ignore: 0,
                queued,
            }),
            Hit::Enemy { id, unit: true } => Some(Command::Follow {
                ox: i32::from(id.o),
                whom: i32::from(id.who),
                queued,
            }),
            Hit::Enemy { unit: false, .. } => None,
            Hit::Friend(id) => Some(Command::Guard {
                ox: i32::from(id.o),
                whom: i32::from(id.who),
                queued,
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
                    queued,
                    orders,
                };
                if !active {
                    Some(swarm(BUILD_AT))
                } else if room == Some(true) {
                    Some(Command::Gather {
                        ox: i32::from(id.o),
                        queued,
                    })
                } else if damaged {
                    Some(swarm(REPAIR))
                } else {
                    None
                }
            }
        }
    }

    /// **The right press's spy arms** (item 4303, `docs/CLIENT_INPUT.md`
    /// §37): `Console::execute_at_cursor@007c6630`'s Scout-line arm and then
    /// its Spy arm, asked after the attack-move and before §6's arms, each
    /// on another slot's object under the cursor. Each issues
    /// `GroupOut::issue_spell(t, ox, whom, 0, 0)`, the point (0, 0):
    ///
    /// - **the Scout line** (a member `is(SCOUT)`): Counterintelligence on
    ///   a Spy; else, with a member `is(COMMANDO)`, Sabotage on a building,
    ///   Sabotage on a vehicle (`has_objmask(0x200000)`), the Sniper on any
    ///   other unit `is_valid_target` passes;
    /// - **the Spy** (a member `is(SPY)`): Counterintelligence on a Spy; on
    ///   another unit the Informer on a citizen (`is_peasant`), else Bribe
    ///   where `is_valid_target` passes; on a building Bribe when the cell
    ///   under it is the issuer's (`WorldData::get_whose`), else the
    ///   Informer.
    ///
    /// `None` when neither arm takes the object, and the dispatch goes on.
    /// `Some` of the commands when one does, which is empty when it
    /// refused: an arm that has taken the object sets the dispatch's flag
    /// (`local_34`), issued or not, and the dispatch returns. The Scout
    /// line's issue falls through to the Spy arm, so a selection holding
    /// both writes both, the second `group` the repeat (§20).
    fn spy_arms(&mut self, scene: &Scene, hit: Hit) -> Option<Vec<Command>> {
        use sim::ids::spell::{BRIBE, COUNTERINTEL, INFORMER, SABOTAGE, SNIPER};
        let Hit::Enemy { id, unit } = hit else {
            return None;
        };
        let (scout, commando, spy) = scene
            .units
            .iter()
            .filter(|u| self.selection.contains(&u.id))
            .fold((false, false, false), |(a, b, c), u| {
                (a || u.spy.scout_line, b || u.spy.commando, c || u.spy.spy)
            });
        let marks = if unit {
            scene.units.iter().find(|u| u.id == id)?.spy
        } else {
            scene.buildings.iter().find(|b| b.id == id)?.spy
        };
        let (mut taken, mut crafts) = (false, Vec::new());
        if scout && (marks.spy || commando) {
            taken = true;
            crafts.extend(if marks.spy {
                Some(COUNTERINTEL)
            } else if !unit || marks.vehicle {
                Some(SABOTAGE)
            } else {
                marks.sniper.then_some(SNIPER)
            });
        }
        if spy {
            taken = true;
            crafts.extend(if marks.spy {
                Some(COUNTERINTEL)
            } else if unit && marks.peasant {
                Some(INFORMER)
            } else if unit {
                marks.bribe.then_some(BRIBE)
            } else if marks.viewer_ground {
                Some(BRIBE)
            } else {
                Some(INFORMER)
            });
        }
        if !taken {
            return None;
        }
        let at = Pos { x: 0, y: 0 };
        let (ox, whom) = (i32::from(id.o), i32::from(id.who));
        let mut out = Vec::new();
        for t in crafts {
            out.push(self.group(scene));
            out.push(spell(t, ox, whom, at));
        }
        Some(out)
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

    /// **The console's line, sent** (item 4283, `docs/CLIENT_INPUT.md`
    /// §28): `ConsoleWin::parse_cmd@007d6470` matches the line's first
    /// token against the command table and hands the rest to `run_cmd
    /// @007d6a70`, whose console half is the only one that holds `ai`
    /// (case `0xc`):
    ///
    /// - no argument: `CommandManager::issue_cheat_ai_toggle@00940fa0`;
    /// - `on`: the same, only while `GameAccess::ai_off` is set;
    /// - `off`: the same, only while it is clear;
    /// - `debug` and anything else: no command (the debug arm and
    ///   `invalid_use_msg`).
    ///
    /// The command is `0x40` alone, no group: the package carries it and
    /// nothing else. Every other console word mints nothing here: the
    /// console's other commands are a developer's, and none is an order a
    /// recording carries. Ours: the word and its argument compared without
    /// ASCII case, split on spaces.
    pub fn console_line(&self, frame: i32, line: &str, view: &View) -> Option<Order> {
        let mut words = line.split(' ').filter(|w| !w.is_empty());
        if !words.next()?.eq_ignore_ascii_case("ai") {
            return None;
        }
        let off = view.scene.ai_off;
        let issue = match words.next() {
            None => true,
            Some(a) if a.eq_ignore_ascii_case("on") => off,
            Some(a) if a.eq_ignore_ascii_case("off") => !off,
            Some(_) => false,
        };
        issue.then(|| (frame, i32::from(self.player), vec![Command::CheatAiToggle]))
    }

    /// **`Options::picked_spot@00721c40`, as the left button reaches it**
    /// (item 4304, `docs/CLIENT_INPUT.md` §38): `None` when no pick is
    /// armed, else what the armed pick does. `Console::left_click@007c3bc0`
    /// (a click) and `Console::on_left_up@007cde90` (a drag, ahead of the
    /// box) both call it as `picked_spot(drag_x2, drag_y2, drag_x, drag_y)`:
    /// the cursor's ground point at the release, then the press's.
    /// `Console::down@007cd8f0` writes the press's to both, and every
    /// `on_mouse_move` rewrites `drag_x2`. The garrison and the spell read
    /// the release's point alone; the build arm hands both to
    /// `issue_build`, its `(x, y)` and `(x2, y2)`.
    fn picked_spot(
        &mut self,
        frame: i32,
        view: &View,
        (x, y): (i32, i32),
        press: (i32, i32),
    ) -> Option<Option<Order>> {
        let play = i32::from(self.player);
        if self.garrisoning {
            // `picked_spot`'s `OPTION_GARRISON` arm: no building under the
            // pick returns before `reset`, and the pick stands; any other
            // spends it.
            let Some(b) = view.own_building(self.player, x, y) else {
                return Some(None);
            };
            self.garrisoning = false;
            let cmd = garrisons(view.scene, self.player, &self.selection, b);
            return Some(cmd.map(|cmd| (frame, play, vec![self.group(view.scene), cmd])));
        }
        if let Some(t) = self.casting.take() {
            // `picked_spot`'s `OPTION_SPELL` arm: `reset`, then a ground
            // craft (`& 8`) at the point, else the object
            // `SpellTypeData::find_target` names, at the point.
            let spelled = (|| {
                let at = ground_at(view, x, y)?;
                let craft = selection_craft(view.scene, &self.selection, t)?;
                let cmd = if craft.ground {
                    spell(t, -1, -1, at)
                } else {
                    let id = match view.target(self.player, x, y)? {
                        Hit::Enemy { id, .. } | Hit::Friend(id) | Hit::Own { id, .. } => id,
                    };
                    spell(t, i32::from(id.o), i32::from(id.who), at)
                };
                Some((frame, play, vec![self.group(view.scene), cmd]))
            })();
            return Some(spelled);
        }
        let ty = self.placing?;
        // The build arm: `issue_build(x, y, x2, y2)`, the release's ground
        // point and the press's. `process_build@00948110` hands both to
        // `Group::action_build@00707510`, whose only reads of `x2, y2` are
        // its pushes into `GroupData::validate_build@00708620`, which reads
        // neither (the listing): the press's point is carried, never used.
        let (Some(at), Some(from)) = (ground_at(view, x, y), ground_at(view, press.0, press.1))
        else {
            return Some(None);
        };
        let (queued, armed) = self.placed_queue(ty);
        if !armed {
            self.placing = None;
        }
        let cmd = Command::Build {
            x: at.x,
            y: at.y,
            x2: from.x,
            y2: from.y,
            build: ty,
            queued,
        };
        Some(Some((frame, play, vec![self.group(view.scene), cmd])))
    }

    /// One event on `frame` (the recording frame the package is consumed
    /// on): the order it makes, if any.
    pub fn event(&mut self, frame: i32, ev: Event, view: &View) -> Option<Order> {
        let play = i32::from(self.player);
        match ev {
            Event::Enter => match (self.line.take(), self.renaming.take()) {
                (None, _) => {
                    self.line = Some(String::new());
                    None
                }
                (Some(line), Some(o)) => Some((frame, play, vec![rename(self.player, o, &line)?])),
                (Some(line), None) if std::mem::take(&mut self.console) => {
                    self.console_line(frame, &line, view)
                }
                (Some(line), None) => self.chat(frame, &line, view),
            },
            Event::Console => {
                // Ours: the key opens the console's line when no line is
                // open, and is a typed `~` inside one.
                if self.line.is_some() {
                    return None;
                }
                self.line = Some(String::new());
                self.console = true;
                None
            }
            Event::Backspace => {
                if let Some(line) = &mut self.line {
                    line.pop();
                }
                None
            }
            Event::Escape => {
                self.line = None;
                self.renaming = None;
                self.console = false;
                None
            }
            Event::Rename => {
                // Option `0x117`: the popup opens on the city's name. Ours:
                // the box opens empty, because the scene does not carry the
                // founding name the original's box starts with.
                if !self.building || self.line.is_some() {
                    return None;
                }
                self.renaming = Some(renames(view.scene, self.player, &self.selection)?);
                self.line = Some(String::new());
                None
            }
            Event::Hotkey(key) => {
                // `on_key_down`'s ctrl arm with no shift and a selection:
                // `GroupOut::issue_hotkey(who × 0x12 + key)`, whose
                // `CommandManager::issue_hotkey` writes the group and the
                // command with `clear` 0 (a group rides) and no camera.
                if key >= HOTKEY_SLOTS || self.selection.is_empty() || self.line.is_some() {
                    return None;
                }
                let cmd = Command::Hotkey {
                    group: i32::from(self.player) * i32::from(HOTKEY_SLOTS) + i32::from(key),
                    clear: 0,
                    valid: 0,
                    x: HOTKEY_NO_CAMERA,
                    y: HOTKEY_NO_CAMERA,
                    zoom: 0,
                };
                Some((frame, play, vec![self.group(view.scene), cmd]))
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
                Some((frame, play, vec![self.group(view.scene), cmd]))
            }
            Event::Unqueue(slot) => {
                // `OPTION_QUEUE`: one `unqueue` per finished building of
                // the selection, in a package with no group.
                if !self.building {
                    return None;
                }
                let cmds = unqueues(view.scene, self.player, &self.selection, slot)?;
                Some((frame, play, cmds))
            }
            Event::Infinite => {
                // `Options::exec`'s case `0x40`: a selection of buildings
                // flashes and `issue_buildmask(0x40)` writes its `group` and
                // `buildmask(0x40, set 1)`; whether a member takes the bit
                // is `process_buildmask`'s (`WallData::valid_buildmask`).
                // Any other selection falls through to the formation arm,
                // which is not minted.
                if !self.building || self.selection.is_empty() || self.line.is_some() {
                    return None;
                }
                let cmd = Command::Buildmask {
                    mask: INFINITE_MASK,
                    set: 1,
                };
                Some((frame, play, vec![self.group(view.scene), cmd]))
            }
            Event::Stance => {
                // `do_stance(-1)`: a selection that `can_activate` writes
                // its `group` and `stance(-1)`; a negative option passes
                // the combat-only test. Whether a member takes the option
                // is `action_stance`'s (its stance kind).
                if self.selection.is_empty() || self.line.is_some() {
                    return None;
                }
                let cmd = Command::Stance {
                    stance: STANCE_CYCLE,
                };
                Some((frame, play, vec![self.group(view.scene), cmd]))
            }
            Event::Disband => {
                // `do_disband(0)`: a selection that `can_activate`, in a
                // game whose semaphore's `0x10` bit is clear, writes its
                // `group` and `disband(0)` unless `validate_disband`
                // refuses — never for a unit group; for buildings, the
                // walk's `disband_refused`, which the layer leaves to the
                // walk as `action_disband` re-asks it.
                if self.selection.is_empty() || self.line.is_some() {
                    return None;
                }
                let cmd = Command::Disband { all: DISBAND_ONE };
                Some((frame, play, vec![self.group(view.scene), cmd]))
            }
            Event::AutoExplore => {
                // `do_auto_explore(−1)`: a selection that `can_activate`,
                // in a game whose semaphore's `0x10` bit is clear
                // (`issue_unitmask`'s own test), writes its `group` and
                // `unitmask(0x100, −1)`. The flash and the two sounds are
                // the console's.
                if self.selection.is_empty() || self.line.is_some() || self.building {
                    return None;
                }
                let cmd = Command::Unitmask {
                    mask: AUTO_EXPLORE_MASK,
                    set: AUTO_EXPLORE_SET,
                };
                Some((frame, play, vec![self.group(view.scene), cmd]))
            }
            Event::Halt => {
                // `do_halt`: a unit selection (`+0x49` clear) that
                // `can_activate` writes its `group` and `halt`; the sound
                // is the console's.
                if self.selection.is_empty() || self.line.is_some() || self.building {
                    return None;
                }
                Some((frame, play, vec![self.group(view.scene), Command::Halt]))
            }
            Event::Alarm => {
                // `do_alarm`: a selection that `can_activate`, in a game
                // whose semaphore's `0x10` bit is clear, writes its `group`
                // and `alarm`; it asks nothing of the members' kind.
                if self.selection.is_empty() || self.line.is_some() {
                    return None;
                }
                Some((frame, play, vec![self.group(view.scene), Command::Alarm]))
            }
            Event::Unbuilt(_) => None,
            Event::ComeOut(k) => {
                if !self.building || self.line.is_some() {
                    return None;
                }
                let g = *garrisoned(view.scene, self.player, &self.selection).get(k)?;
                Some((frame, play, vec![come_out(&g)]))
            }
            Event::Garrison => {
                // `do_garrison`: a selection that can act and is not of
                // buildings sets `selecting_spot`.
                if self.building || self.selection.is_empty() {
                    return None;
                }
                self.placing = None;
                self.casting = None;
                self.garrisoning = true;
                None
            }
            Event::Spell(t) => {
                // `do_spell`: a selection that can act, holding a caster of
                // the craft; an untargeted one is `target_spell(t, −1, −1,
                // 0, 0)`; a targeted one sets `selecting_spot`.
                if self.building {
                    return None;
                }
                let craft = selection_craft(view.scene, &self.selection, t)?;
                if craft.targeted {
                    self.placing = None;
                    self.garrisoning = false;
                    self.casting = Some(t);
                    return None;
                }
                Some((
                    frame,
                    play,
                    vec![self.group(view.scene), spell(t, -1, -1, Pos { x: 0, y: 0 })],
                ))
            }
            Event::Key(k) if self.line.is_some() => {
                let limit = if self.renaming.is_some() {
                    RENAME_LIMIT
                } else {
                    CHAT_LIMIT
                };
                let line = self.line.as_mut().unwrap();
                if line.encode_utf16().count() + k.len_utf16() <= limit {
                    line.push(k);
                }
                None
            }
            Event::Press { x, y } => {
                if let Some(o) = self.picked_spot(frame, view, (x, y), (x, y)) {
                    return o;
                }
                self.press_select(view, x, y);
                None
            }
            Event::Drag { x0, y0, x1, y1 } => {
                // `Console::on_left_up@007cde90`: an armed pick takes the
                // release before the box, `picked_spot` at the release with
                // the press beside it (item 4304, `docs/CLIENT_INPUT.md`
                // §38), a click's (`left_click`) or a drag's alike: since
                // item 4318 the window sends a release that moved at all as
                // a drag, and the slop is asked here, after the pick.
                if let Some(o) = self.picked_spot(frame, view, (x1, y1), (x0, y0)) {
                    return o;
                }
                if (x1 - x0).abs().max((y1 - y0).abs()) < DRAG_SLOP {
                    self.press_select(view, x0, y0);
                    return None;
                }
                // The units first; the buildings only when the box took
                // no unit (§7.1, item 4318).
                self.selection = view.boxed(self.player, (x0, y0), (x1, y1));
                self.building = false;
                self.page = None;
                if self.selection.is_empty() {
                    self.selection = view.boxed_buildings(self.player, (x0, y0), (x1, y1));
                    self.building = !self.selection.is_empty();
                }
                None
            }
            Event::RightPress { x, y } => {
                self.casting = None;
                self.garrisoning = false;
                if self.selection.is_empty() {
                    return None;
                }
                if self.building {
                    // The building arm (`execute_at_cursor`, items 4249 and
                    // 4301, §11.4): a selection that takes a gather point
                    // names, in order, a building of any slot (`action` 2
                    // on an enemy's, 1 otherwise, at its position), one of
                    // the slot's own units (`action` 3, `(captain, who)`),
                    // or the ground (`action` 0). Every arm's `add_to_end`
                    // is ctrl or shift (item 4293, §33.1). Refused: an
                    // active University of one's own for a city, and a
                    // building of the selection under the profile's
                    // `prefs & 0x202` clear.
                    if !view.takes_gather_point(&self.selection) {
                        return None;
                    }
                    let add_to_end = i32::from(self.mods.shift || self.mods.ctrl);
                    let cmd = match view.gather_target(self.player, x, y) {
                        Some(GatherHit::Building {
                            id,
                            pos,
                            enemy,
                            university,
                        }) => {
                            if university
                                && id.who == self.player
                                && view.selects_city(&self.selection)
                            {
                                return None;
                            }
                            if self.selection.contains(&id) && self.prefs.prefs & 0x202 == 0 {
                                return None;
                            }
                            Command::GatherPoint {
                                x: pos.x,
                                y: pos.y,
                                action: 1 + i32::from(enemy),
                                add_to_end,
                            }
                        }
                        Some(GatherHit::Unit { captain, who }) => Command::GatherPoint {
                            x: i32::from(captain),
                            y: i32::from(who),
                            action: 3,
                            add_to_end,
                        },
                        None => {
                            let at = ground_at(view, x, y)?;
                            Command::GatherPoint {
                                x: at.x,
                                y: at.y,
                                action: 0,
                                add_to_end,
                            }
                        }
                    };
                    return Some((frame, play, vec![self.group(view.scene), cmd]));
                }
                if self.mods.ctrl {
                    // Ctrl is the dispatch's attack-move, asked before any
                    // object (`execute_at_cursor`'s first arm, item 4297,
                    // §35.1): `ATTACK_TO` to the release's ground point with
                    // −1/−1. With alt read too (under the profile's `0x400`,
                    // as the queue's), it is `MOVE_TO` with the press's
                    // formation, `drag_form`/`drag_form_mod`.
                    let at = ground_at(view, x, y)?;
                    let queued = self.dispatch_queue();
                    let cmd = if self.mods.alt && self.prefs.prefs & 0x400 == 0 {
                        let (form, width) = view.formation(&self.selection);
                        move_to(at, form, width, queued)
                    } else {
                        attack_to(at, queued)
                    };
                    return Some((frame, play, vec![self.group(view.scene), cmd]));
                }
                let hit = view.target(self.player, x, y);
                if let Some(cmds) = hit.and_then(|h| self.spy_arms(view.scene, h)) {
                    // The spy arms took the object: the dispatch stops,
                    // whether or not a craft was issued (item 4303, §37).
                    return (!cmds.is_empty()).then_some((frame, play, cmds));
                }
                if let Some(cmd) =
                    hit.and_then(|h| self.on_object(h, view.attacks(&self.selection)))
                {
                    return Some((frame, play, vec![self.group(view.scene), cmd]));
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
                Some((
                    frame,
                    play,
                    vec![
                        self.group(view.scene),
                        move_to(at, form, width, self.dispatch_queue()),
                    ],
                ))
            }
            Event::RightDrag { x0, y0, x1, y1 } => {
                self.casting = None;
                self.garrisoning = false;
                // `Console::on_right_up@007cd3c0` with `dragging` 2 (item
                // 4295, §34.1): a selection that can act; a building's
                // leader moves nothing. Ctrl is not read.
                if self.selection.is_empty() || self.building {
                    return None;
                }
                // A Caravan or a Merchant Fleet in the selection looks for
                // a market at the press first, the trade route; not built,
                // so nothing where the press names an object (ours: the
                // trade's own search is not read).
                if view.trades_drag(&self.selection) && view.names_object(x0, y0) {
                    return None;
                }
                // `drag_x`/`drag_y` are the press's ground point
                // (`Console::down@007cd8f0`), `drag_x2`/`drag_y2` the
                // release's, each `convert_mouse_to_terrain`'s.
                let at = ground_at(view, x0, y0)?;
                let to = ground_at(view, x1, y1)?;
                let angle = sim::movement::find_angle(to.x - at.x, to.y - at.y).0;
                let (form, width) = view.formation(&self.selection);
                let Command::MoveTo {
                    to_x,
                    to_y,
                    orders,
                    queued,
                    disembark,
                    ..
                } = move_to(at, form, width, self.map_queue())
                else {
                    unreachable!("move_to mints a MoveTo")
                };
                Some((
                    frame,
                    play,
                    vec![
                        self.group(view.scene),
                        Command::MoveTo {
                            to_x,
                            to_y,
                            set_angle: 1,
                            angle,
                            orders,
                            queued,
                            form,
                            width,
                            disembark,
                        },
                    ],
                ))
            }
            Event::MapRightPress { x, y } => {
                // The minimap's arm: the point on the map, the selection
                // able to act, `MOVE_TO` with −1/−1. With ctrl it is
                // `ATTACK_TO` in its place, alt or not (item 4297, §35.1).
                let (w, h) = view.map_extent();
                if self.selection.is_empty() || !(0..w).contains(&x) || !(0..h).contains(&y) {
                    return None;
                }
                if self.building {
                    // A building's arm (item 4302, §36.2): the gather
                    // point at the map point, `action` 0, `add_to_end`
                    // with shift or ctrl. A selection holding a patrol's
                    // launcher issues `launch_patrol`, which is not ours.
                    let launches = view.scene.buildings.iter().any(|b| {
                        self.selection.contains(&b.id)
                            && b.ty.is_some_and(|t| PATROL_LAUNCHERS.contains(&t))
                    });
                    if launches {
                        return None;
                    }
                    let cmd = Command::GatherPoint {
                        x,
                        y,
                        action: 0,
                        add_to_end: i32::from(self.mods.shift || self.mods.ctrl),
                    };
                    return Some((frame, play, vec![self.group(view.scene), cmd]));
                }
                let at = Pos { x, y };
                let cmd = if self.mods.ctrl {
                    attack_to(at, self.map_queue())
                } else {
                    move_to(at, -1, -1, self.map_queue())
                };
                Some((frame, play, vec![self.group(view.scene), cmd]))
            }
            Event::Train(unit) => {
                if self.selection.is_empty() || !self.building {
                    return None;
                }
                let cmd = Command::QueueUp { unit, num: 1 };
                Some((frame, play, vec![self.group(view.scene), cmd]))
            }
            Event::Page(option) => {
                if self.selection.is_empty() || self.building {
                    return None;
                }
                self.page = turn(option, self.page, view.sim.map_or(3, pages_of));
                None
            }
            Event::Place(ty) => {
                if !self.selection.is_empty() && !self.building {
                    self.garrisoning = false;
                    self.placing = Some(ty);
                }
                None
            }
            Event::Key(k) => {
                if self.selection.is_empty() {
                    return None;
                }
                if self.building {
                    let &(_, unit) = TRAIN_KEYS.iter().find(|(c, _)| *c == k)?;
                    let cmd = Command::QueueUp { unit, num: 1 };
                    Some((frame, play, vec![self.group(view.scene), cmd]))
                } else {
                    let &(_, ty) = BUILD_KEYS.iter().find(|(c, _)| *c == k)?;
                    self.garrisoning = false;
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
            sim: None,
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
            let last = rec.packages.last().unwrap().frame;
            let mut ours = rec.clone();
            ours.packages =
                rondata::recgame::minted_packages(Input::default().leader_options(0), orders, last)
                    .unwrap();
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
        let last = rec.packages.last().unwrap().frame;
        let mut w = rec.clone();
        w.packages =
            rondata::recgame::minted_packages(Input::default().leader_options(0), orders, last)
                .unwrap();
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
        let last = rec.packages.last().unwrap().frame;
        let mut written = rec.clone();
        written.packages =
            rondata::recgame::minted_packages(Input::default().leader_options(0), &ours, last)
                .unwrap();
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
        let last = rec.packages.last().unwrap().frame;
        let mut written = rec.clone();
        written.packages =
            rondata::recgame::minted_packages(Input::default().leader_options(0), &ours, last)
                .unwrap();
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
        let last = rec.packages.last().unwrap().frame;
        let mut written = rec.clone();
        written.packages =
            rondata::recgame::minted_packages(Input::default().leader_options(0), &ours, last)
                .unwrap();
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
        let written = written(&script::full_groups(&ours)).unwrap();
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
        let written = written(&script::full_groups(&ours)).unwrap();
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
        let written = written(&script::full_groups(&ours)).unwrap();
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
        let written = written(&script::full_groups(&ours)).unwrap();
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
        let written = written(&script::full_groups(&ours)).unwrap();
        assert!(written == file, "the file from the city-gathered orders");
        assert!(written != marketed_file, "not the marketed file");
    }

    /// The city-gathered file (`docs/CLIENT_INPUT.md` §17), written by
    /// rondata's `the_city_gathered_minted_packages_round_trip` under
    /// `RON_RCX_OUT`; the file the original's playback takes open.
    const CITY_GATHERED_RCX: &str =
        "ron-golden/recordings/item4261-city-gathered-greatlakes-24000.rcx";
    const CITY_GATHERED_RCX_BYTES: usize = 59_546;

    /// **The cancel's button** (item 4265, `docs/CLIENT_INPUT.md` §19): on
    /// [`script::UNQUEUE_FRAME`] the capital's queue row at [`HUD_SIZE`]
    /// holds one button, CANCEL 1, here.
    const UNQUEUE_AT: (i32, i32) = (1822, 1010);

    /// The city-gathered events and the cancel's one: the capital is still
    /// selected from the gather, so a press on its first queue button.
    fn unqueue_events() -> Vec<Scripted> {
        let mut e = city_gather_events();
        let (x, y) = UNQUEUE_AT;
        e.push((script::UNQUEUE_FRAME, None, Event::Press { x, y }));
        e
    }

    /// **`INPUT_UNQUEUED`** (item 4265, `docs/CLIENT_INPUT.md` §19): the
    /// city-gathered events and the cancel's press, through the HUD and the
    /// layer under the opening camera, mint [`script::unqueued`] field for
    /// field, of 22.
    const INPUT_UNQUEUED: usize = 22;

    /// **The unqueued count**: stage 0 on the cancel's frame — the capital
    /// is slot 0's, finished, with one entry in its queue (the gathered
    /// script's Citizen) and `uid` 0, and its queue row holds one button
    /// where the event presses it; then the events mint the unqueued
    /// script, and the file written from them is the unqueued file.
    #[test]
    fn minted_unqueued_at_the_opening_camera() {
        let Ok(mut r) = lobby() else {
            println!("skipping: no lobby");
            return;
        };
        let gathered = script::city_gathered();
        let mut next = 0;
        while r.frame() < i64::from(script::UNQUEUE_FRAME) {
            let f = r.frame() as i32;
            while next < gathered.len() && gathered[next].0 == f {
                r.issue(&gathered[next]);
                next += 1;
            }
            r.step();
        }
        {
            let scene = r.scene(Some(0));
            let id = ObjId {
                who: 0,
                o: script::CAPITAL,
            };
            let b = scene.buildings.iter().find(|b| b.id == id).unwrap();
            let mut input = Input::new(0);
            input.selection = vec![id];
            input.building = true;
            let row: Vec<(String, (i32, i32))> = crate::hud::queue_menu(&input, &scene, HUD_SIZE)
                .iter()
                .map(|b| (b.label(), b.rect.centre()))
                .collect();
            println!(
                "UNQUEUE STAGE0 f{} active {} queued {} uid {} row {row:?}",
                r.frame(),
                b.active,
                b.queued,
                b.uid
            );
            assert!(b.active);
            assert_eq!((b.queued, b.uid), (1, 0));
            assert_eq!(row, vec![("CANCEL 1".to_string(), UNQUEUE_AT)]);
        }
        drop(r);

        let Some((ours, r)) = mint_with(unqueue_events(), |input, frame, ev, view| {
            crate::hud::event(input, HUD_SIZE, frame, ev, view)
        }) else {
            return;
        };
        let unqueued = script::unqueued();
        let whole = ours.iter().zip(&unqueued).filter(|(a, b)| a == b).count();
        assert_eq!(
            (ours.len(), whole),
            (unqueued.len(), INPUT_UNQUEUED),
            "{ours:?}"
        );
        assert_eq!(ours, unqueued);
        drop(r);

        let home = std::env::var_os("HOME").unwrap_or_default();
        let (Ok(gathered_file), Ok(file)) = (
            std::fs::read(std::path::Path::new(&home).join(CITY_GATHERED_RCX)),
            std::fs::read(std::path::Path::new(&home).join(UNQUEUED_RCX)),
        ) else {
            println!("skipping the bytes: no {CITY_GATHERED_RCX} or {UNQUEUED_RCX}");
            return;
        };
        assert_eq!(file.len(), UNQUEUED_RCX_BYTES);
        let as_layer = written(&ours).unwrap();
        let written = written(&script::full_groups(&ours)).unwrap();
        assert!(written == file, "the file from the unqueued orders");
        assert!(written != gathered_file, "not the city-gathered file");

        // As `add_group` writes it (item 4267): the repeated file.
        let Ok(repeated) = std::fs::read(std::path::Path::new(&home).join(REPEATED_RCX)) else {
            println!("skipping the bytes: no {REPEATED_RCX}");
            return;
        };
        assert_eq!(repeated.len(), REPEATED_RCX_BYTES);
        assert!(
            as_layer == repeated,
            "the file from the unqueued orders as the layer writes them"
        );
    }

    /// **The repeated file** (item 4267, `docs/CLIENT_INPUT.md` §20): the
    /// unqueued orders as the layer mints them, each `group` that repeats
    /// the last one sent as no objects; written by rondata's
    /// `the_repeated_minted_packages_round_trip` under `RON_RCX_OUT`, and
    /// the file run1607 takes open.
    const REPEATED_RCX: &str = "ron-golden/recordings/item4267-repeated-greatlakes-24000.rcx";
    const REPEATED_RCX_BYTES: usize = 59_564;

    /// The unqueued file (`docs/CLIENT_INPUT.md` §19), written by rondata's
    /// `the_unqueued_minted_packages_round_trip` under `RON_RCX_OUT`.
    const UNQUEUED_RCX: &str = "ron-golden/recordings/item4265-unqueued-greatlakes-24000.rcx";
    const UNQUEUED_RCX_BYTES: usize = 59_559;

    /// A finished building of `who`'s with `queued` entries and `uid`,
    /// every other field the lobby's own building's.
    fn queue_building(who: u8, o: i16, queued: usize, uid: u16) -> sim::observe::BuildingScene {
        let Ok(r) = lobby() else {
            panic!("the queue's edges need the lobby for a building record");
        };
        let mut b = r.scene(None).buildings[0];
        b.id = ObjId { who, o };
        b.owner = who;
        (b.active, b.queued, b.uid) = (true, queued, uid);
        b
    }

    /// **`INPUT_UNQUEUES`** (item 4265): of the cancels the recordings on
    /// disk carry, those a press on the building's queue button for the
    /// recorded slot mints field for field, the package's orders whole,
    /// with the building it names selected, of all of them. Every one is
    /// the 2017 sample's, slot 3's, with no `group`, as the arm writes it.
    /// Seen red pinned (0, 13).
    const INPUT_UNQUEUES: (usize, usize) = (13, 13);

    /// **Every recorded cancel a queue button issues** (item 4265): each
    /// recorded `unqueue`'s building selected, with the recorded `uid` and
    /// a queue long enough for its slot, then its queue button pressed.
    #[test]
    fn minted_unqueues_equal_the_recordings() {
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
                let Some(&Command::Unqueue { who, o, unit, uid }) =
                    kept.iter().find(|c| matches!(c, Command::Unqueue { .. }))
                else {
                    continue;
                };
                all += 1;
                let (who, o) = (who as u8, o as i16);
                let scene = Scene {
                    buildings: vec![queue_building(who, o, unit as usize + 1, uid as u16)],
                    ..Scene::default()
                };
                let v = view(&c, &scene, &terrain);
                let mut input = Input::new(who);
                input.selection = vec![ObjId { who, o }];
                input.building = true;
                let got = input.event(p.frame, Event::Unqueue(unit), &v);
                if got.as_ref().map(|o| &o.2) == Some(&kept) {
                    whole += 1;
                } else {
                    println!("UNQUEUE PARTS f{} {kept:?}: {got:?}", p.frame);
                }
            }
        }
        println!("UNQUEUES {whole} of {all}");
        assert_eq!((whole, all), INPUT_UNQUEUES);
    }

    /// **The queue's buttons without a walk** (item 4265): a row only for
    /// one finished building of the slot's own, a button per live entry;
    /// one `unqueue` per finished building of a selection; nothing for
    /// another slot's building, a unit selection or an unfinished building.
    #[test]
    fn minted_unqueue_button_edges() {
        if lobby().is_err() {
            println!("skipping: no lobby");
            return;
        }
        let terrain = TerrainScene::default();
        let c = crate::pinned_camera();
        let mint = |scene: &Scene, ids: &[i16], building: bool, slot: i32| {
            let v = view(&c, scene, &terrain);
            let mut input = Input::new(0);
            input.selection = ids.iter().map(|&o| ObjId { who: 0, o }).collect();
            input.building = building;
            input.event(7, Event::Unqueue(slot), &v).map(|o| o.2)
        };
        let scene = |bs: Vec<sim::observe::BuildingScene>| Scene {
            buildings: bs,
            ..Scene::default()
        };
        let row = |s: &Scene, ids: &[i16]| {
            let mut input = Input::new(0);
            input.selection = ids.iter().map(|&o| ObjId { who: 0, o }).collect();
            input.building = true;
            crate::hud::queue_menu(&input, s, HUD_SIZE)
                .iter()
                .map(|b| b.slot)
                .collect::<Vec<_>>()
        };
        let three = scene(vec![queue_building(0, 2000, 3, 0)]);
        assert_eq!(row(&three, &[2000]), vec![0, 1, 2]);
        assert_eq!(
            row(&scene(vec![queue_building(0, 2000, 0, 0)]), &[2000]),
            Vec::<i32>::new()
        );
        let cancel = |o: i32, slot: i32, uid: i16| Command::Unqueue {
            who: 0,
            o,
            unit: slot,
            uid,
        };
        assert_eq!(
            mint(&three, &[2000], true, 2),
            Some(vec![cancel(2000, 2, 0)])
        );
        // Through the HUD: a press on the second button is slot 1.
        {
            let v = view(&c, &three, &terrain);
            let mut input = Input::new(0);
            input.selection = vec![ObjId { who: 0, o: 2000 }];
            input.building = true;
            let (x, y) = crate::hud::queue_menu(&input, &three, HUD_SIZE)[1]
                .rect
                .centre();
            let got = crate::hud::event(&mut input, HUD_SIZE, 7, Event::Press { x, y }, &v);
            assert_eq!(got.map(|o| o.2), Some(vec![cancel(2000, 1, 0)]));
        }
        // Two buildings: one cancel each, in the selection's order; an
        // unfinished one issues nothing, and alone nothing at all.
        let mut site = queue_building(0, 2008, 1, 15);
        site.active = false;
        let two = scene(vec![
            queue_building(0, 2000, 1, 0),
            queue_building(0, 2006, 1, 6),
            site,
        ]);
        assert_eq!(
            mint(&two, &[2006, 2000, 2008], true, 0),
            Some(vec![cancel(2006, 0, 6), cancel(2000, 0, 0)])
        );
        assert_eq!(mint(&two, &[2008], true, 0), None);
        assert_eq!(row(&two, &[2008]), Vec::<i32>::new());
        assert_eq!(row(&two, &[2000, 2006]), Vec::<i32>::new());
        // Another slot's building, a unit selection, an empty scene.
        let theirs = scene(vec![queue_building(1, 2000, 1, 0)]);
        assert_eq!(mint(&theirs, &[2000], true, 0), None);
        assert_eq!(mint(&three, &[2000], false, 0), None);
        assert_eq!(mint(&Scene::default(), &[2000], true, 0), None);
    }

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
    /// of them. The two whose `group` carries no objects are
    /// `add_group`'s repeat (item 4267, `docs/CLIENT_INPUT.md` §20): the
    /// layer, its last group sent the recording's last full one from that
    /// slot, mints them whole. Seen red with the layer's `queued` 0: 0 of
    /// 20; at item 4261's pin, 18 of 20, before the repeat.
    const INPUT_CITY_GATHERS: (usize, usize) = (20, 20);

    /// **The last group a recording's sender sent** (item 4267): for each
    /// package's slot, the last `group` it carried in full, `(who,
    /// objects)`. A recorded group of no objects is `add_group`'s repeat of
    /// it, so the selection behind it is that list.
    type LastSent = std::collections::BTreeMap<i32, Group>;

    /// A recorded `group` in full: `(who, objects)`.
    type Group = (i8, Vec<i16>);

    /// The selection behind a recorded package's `group` and the layer's
    /// [`Sent`] before it, its uids the scene's: `(who, objects, sent)`.
    fn behind(last: &LastSent, play: i32, kept: &[Command]) -> (u8, Vec<i16>, Option<Group>) {
        let before = last.get(&play).cloned();
        match kept.first() {
            Some(Command::Group { who, objects }) if !objects.is_empty() => {
                (*who as u8, objects.clone(), before)
            }
            Some(Command::Group { who, .. }) => (
                *who as u8,
                before.as_ref().map(|b| b.1.clone()).unwrap_or_default(),
                before,
            ),
            _ => (play as u8, Vec::new(), before),
        }
    }

    /// The layer's [`Sent`] for `before`, each object's uid the scene's.
    fn sent_for(before: &Option<Group>, scene: &Scene) -> Sent {
        match before {
            Some((who, objects)) => Sent {
                who: *who,
                objects: objects
                    .iter()
                    .map(|&o| {
                        let id = ObjId { who: *who as u8, o };
                        (o, uid_of(scene, id))
                    })
                    .collect(),
            },
            None => Sent::default(),
        }
    }

    /// The tracker after a recorded package: a `group` in full is the
    /// sender's last.
    fn note_sent(last: &mut LastSent, play: i32, kept: &[Command]) {
        if let Some(Command::Group { who, objects }) = kept.first()
            && !objects.is_empty()
        {
            last.insert(play, (*who, objects.clone()));
        }
    }

    /// A unit for the craft tests (item 4264): slot `who`'s `o` at `(x,
    /// 0)`, offering `crafts`.
    fn caster(who: u8, o: i16, x: i32, crafts: &[sim::observe::Craft]) -> sim::observe::UnitScene {
        sim::observe::UnitScene {
            crafts: crafts.to_vec(),
            ..unit(who, o, 0x36, x, 0, 0)
        }
    }

    /// **Every recorded spell a craft button issues** (item 4264,
    /// `docs/CLIENT_INPUT.md` §18): `(whole, of)` — the recorded spells with
    /// no target (`ox`, `whom` −1 and the point (0, 0), `do_spell`'s
    /// `target_spell(t, −1, −1, 0, 0)`), each minted from its group's units
    /// offering the craft and a press on its button. The five whose
    /// `group` is empty are `CommandPackage::add_group`'s repeat, which the
    /// layer writes since item 4267 (`docs/CLIENT_INPUT.md` §20), its last
    /// group sent the recording's last full one from that slot. Seen red
    /// with the arm's point (1, 1): 0 of 6; at item 4264's pin, 1 of 6,
    /// before the repeat.
    const INPUT_SPELLS: (usize, usize) = (6, 6);

    #[test]
    fn minted_spells_equal_the_recordings() {
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
            let mut last = LastSent::new();
            for p in &rec.packages {
                let Ok(cmds) = rondata::commands::decode(&p.data) else {
                    continue;
                };
                let kept: Vec<Command> =
                    cmds.into_iter().filter(rondata::input::is_input).collect();
                let (who, objects, before) = behind(&last, p.play, &kept);
                note_sent(&mut last, p.play, &kept);
                let Some(&Command::Spell {
                    ox: -1, spell: t, ..
                }) = kept.iter().find(|c| matches!(c, Command::Spell { .. }))
                else {
                    continue;
                };
                all += 1;
                let craft = sim::observe::Craft {
                    t,
                    targeted: false,
                    ground: false,
                };
                let scene = Scene {
                    units: objects
                        .iter()
                        .map(|&o| caster(who, o, 0, &[craft]))
                        .collect(),
                    ..Scene::default()
                };
                let v = view(&c, &scene, &terrain);
                let mut input = Input::new(who);
                input.selection = objects.iter().map(|&o| ObjId { who, o }).collect();
                input.sent = sent_for(&before, &scene);
                let got = input.event(p.frame, Event::Spell(t), &v);
                if got.as_ref().map(|o| &o.2) == Some(&kept) {
                    whole += 1;
                } else {
                    println!("SPELL PARTS f{} {kept:?}: {got:?}", p.frame);
                }
            }
        }
        println!("SPELLS {whole} of {all}");
        assert_eq!((whole, all), INPUT_SPELLS);
    }

    /// The crafts the right press's spy arms issue (item 4303,
    /// `docs/CLIENT_INPUT.md` §37), each at the point (0, 0).
    const SPY_CRAFTS: [i32; 5] = [
        sim::ids::spell::BRIBE,
        sim::ids::spell::COUNTERINTEL,
        sim::ids::spell::INFORMER,
        sim::ids::spell::SABOTAGE,
        sim::ids::spell::SNIPER,
    ];

    /// **Every recorded spell the right press's spy arms issue** (item
    /// 4303, `docs/CLIENT_INPUT.md` §37): `(whole, of)` — the recorded
    /// spells on an object at the point (0, 0), each minted from its
    /// group's units and a right press on the target. The 2017 sample's
    /// game is not walked, so the scene is built from the craft: Bribe's
    /// and the Informer's group are Spies, the Sniper's and Sabotage's the
    /// Scout line with a Commando; the target is a unit below object 2000
    /// and a building at or past it, and carries the marks its craft's arm
    /// asks (Bribe valid, the Sniper valid, a vehicle for Sabotage on a
    /// unit, ground not the issuer's for the Informer on a building). An
    /// empty `group` is the repeat (§20). Seen red with the arm's point
    /// (1, 1): (0, 11).
    const INPUT_SPY_SPELLS: (usize, usize) = (11, 11);

    #[test]
    fn minted_spy_spells_equal_the_recordings() {
        use sim::ids::spell::{BRIBE, INFORMER, SABOTAGE, SNIPER};
        let files = recordings_on_disk();
        let Ok(r) = lobby() else {
            println!("skipping: no lobby");
            return;
        };
        if files.is_empty() {
            println!("skipping: no .rcx on disk");
            return;
        }
        let terrain = TerrainScene::default();
        let base = r.scene(None).buildings[0];
        drop(r);
        let c = crate::pinned_camera();
        let (mut whole, mut all) = (0, 0);
        for f in &files {
            let rec = rondata::recgame::read(&f.display().to_string()).unwrap();
            let mut last = LastSent::new();
            for p in &rec.packages {
                let Ok(cmds) = rondata::commands::decode(&p.data) else {
                    continue;
                };
                let kept: Vec<Command> =
                    cmds.into_iter().filter(rondata::input::is_input).collect();
                let (who, objects, before) = behind(&last, p.play, &kept);
                note_sent(&mut last, p.play, &kept);
                let Some(&Command::Spell {
                    ox,
                    whom,
                    spell: t,
                    x: 0,
                    y: 0,
                }) = kept.iter().find(|c| matches!(c, Command::Spell { .. }))
                else {
                    continue;
                };
                if ox < 0 || !SPY_CRAFTS.contains(&t) {
                    continue;
                }
                all += 1;
                let caster = sim::observe::SpyMarks {
                    spy: matches!(t, BRIBE | INFORMER),
                    scout_line: matches!(t, SNIPER | SABOTAGE),
                    commando: matches!(t, SNIPER | SABOTAGE),
                    ..Default::default()
                };
                let mark = sim::observe::SpyMarks {
                    bribe: t == BRIBE,
                    sniper: t == SNIPER,
                    vehicle: t == SABOTAGE,
                    ..Default::default()
                };
                let id = ObjId {
                    who: whom as u8,
                    o: ox as i16,
                };
                let at = Pos {
                    x: 768 * 20,
                    y: 768 * 20,
                };
                let mut units: Vec<_> = objects
                    .iter()
                    .map(|&o| sim::observe::UnitScene {
                        spy: caster,
                        ..unit(who, o, 0x3a, 0, 0, 0)
                    })
                    .collect();
                let mut buildings = Vec::new();
                if ox < 2000 {
                    units.push(sim::observe::UnitScene {
                        spy: mark,
                        pos: at,
                        ..unit(id.who, id.o, 0x36, 0, 0, 0)
                    });
                } else {
                    let mut b = base;
                    (b.id, b.owner, b.pos, b.z, b.seen) = (id, id.who, at, 0, true);
                    b.spy = mark;
                    buildings.push(b);
                }
                let scene = Scene {
                    units,
                    buildings,
                    ..Scene::default()
                };
                let v = view(&c, &scene, &terrain);
                let (px, py) = v.pixel(at.x, at.y, 0);
                let mut input = Input::new(who);
                input.selection = objects.iter().map(|&o| ObjId { who, o }).collect();
                input.sent = sent_for(&before, &scene);
                let got = input.event(p.frame, Event::RightPress { x: px, y: py }, &v);
                if got.as_ref().map(|o| &o.2) == Some(&kept) {
                    whole += 1;
                } else {
                    println!("SPY SPELL PARTS f{} {kept:?}: {got:?}", p.frame);
                }
            }
        }
        println!("SPY SPELLS {whole} of {all}");
        assert_eq!((whole, all), INPUT_SPY_SPELLS);
    }

    /// **The spy arms without a walk** (item 4303, §37): each arm's craft
    /// on its object; the Scout line's issue falling through to the Spy
    /// arm, the second `group` the repeat; an arm that took the object and
    /// refused mints nothing and stops the dispatch; a selection neither
    /// arm counts, or a Scout line without a Commando on a unit that is
    /// not a Spy, goes on to §6's attack.
    #[test]
    fn minted_spy_arms_as_the_original() {
        use sim::ids::spell::{BRIBE, COUNTERINTEL, INFORMER, SABOTAGE, SNIPER};
        use sim::observe::SpyMarks;
        let Ok(r) = lobby() else {
            println!("skipping: no lobby");
            return;
        };
        let base = r.scene(None).buildings[0];
        drop(r);
        let terrain = TerrainScene::default();
        let c = crate::pinned_camera();
        let at = Pos {
            x: 768 * 20,
            y: 768 * 20,
        };
        let spy = SpyMarks {
            spy: true,
            ..Default::default()
        };
        let commando = SpyMarks {
            scout_line: true,
            commando: true,
            ..Default::default()
        };
        let scout = SpyMarks {
            scout_line: true,
            ..Default::default()
        };
        let target = ObjId { who: 1, o: 9 };
        let shop = ObjId { who: 1, o: 2003 };
        // `casters` are slot 0's 5 and 6; the target, a unit or a building.
        let press = |casters: [SpyMarks; 2], mark: SpyMarks, building: bool| {
            let mut units = vec![
                sim::observe::UnitScene {
                    spy: casters[0],
                    ..unit(0, 5, 0x3a, 0, 0, 0)
                },
                sim::observe::UnitScene {
                    spy: casters[1],
                    ..unit(0, 6, 0x3a, 0, 0, 0)
                },
            ];
            let mut buildings = Vec::new();
            if building {
                let mut b = base;
                (b.id, b.owner, b.pos, b.z, b.seen, b.spy) = (shop, 1, at, 0, true, mark);
                buildings.push(b);
            } else {
                units.push(sim::observe::UnitScene {
                    spy: mark,
                    pos: at,
                    ..unit(1, 9, 0x36, 0, 0, 0)
                });
            }
            let scene = Scene {
                units,
                buildings,
                ..Scene::default()
            };
            let v = view(&c, &scene, &terrain);
            let (px, py) = v.pixel(at.x, at.y, 0);
            let mut input = Input::new(0);
            input.selection = vec![ObjId { who: 0, o: 5 }, ObjId { who: 0, o: 6 }];
            input
                .event(1, Event::RightPress { x: px, y: py }, &v)
                .map(|o| o.2)
        };
        let group = Command::Group {
            who: 0,
            objects: vec![5, 6],
        };
        let repeat = Command::Group {
            who: 0,
            objects: vec![],
        };
        let none = SpyMarks::default();
        let cast = |t, id: ObjId| spell(t, i32::from(id.o), i32::from(id.who), Pos { x: 0, y: 0 });
        let one = |t, id| Some(vec![group.clone(), cast(t, id)]);
        // The Spy arm.
        assert_eq!(press([spy, none], spy, false), one(COUNTERINTEL, target));
        let citizen = SpyMarks {
            peasant: true,
            bribe: true,
            ..Default::default()
        };
        assert_eq!(press([spy, none], citizen, false), one(INFORMER, target));
        let bribable = SpyMarks {
            bribe: true,
            ..Default::default()
        };
        assert_eq!(press([spy, none], bribable, false), one(BRIBE, target));
        assert_eq!(press([spy, none], none, false), None, "taken, refused");
        let ours = SpyMarks {
            viewer_ground: true,
            ..Default::default()
        };
        assert_eq!(press([spy, none], ours, true), one(BRIBE, shop));
        assert_eq!(press([spy, none], none, true), one(INFORMER, shop));
        // The Scout line.
        assert_eq!(press([scout, none], spy, false), one(COUNTERINTEL, target));
        assert_eq!(press([commando, none], none, true), one(SABOTAGE, shop));
        let vehicle = SpyMarks {
            vehicle: true,
            sniper: true,
            ..Default::default()
        };
        assert_eq!(
            press([commando, none], vehicle, false),
            one(SABOTAGE, target)
        );
        let snipable = SpyMarks {
            sniper: true,
            ..Default::default()
        };
        assert_eq!(
            press([commando, none], snipable, false),
            one(SNIPER, target)
        );
        assert_eq!(press([commando, none], none, false), None, "taken, refused");
        // Both arms: the Scout line's issue, then the Spy's, one package.
        let both = SpyMarks {
            sniper: true,
            bribe: true,
            ..Default::default()
        };
        assert_eq!(
            press([commando, spy], both, false),
            Some(vec![
                group.clone(),
                cast(SNIPER, target),
                repeat,
                cast(BRIBE, target)
            ])
        );
        // Neither arm: §6's attack on the unit.
        let attack = Command::Attack {
            ox: 9,
            whom: 1,
            ignore: 0,
            queued: QUEUE_NEW,
        };
        assert_eq!(
            press([none, none], snipable, false),
            Some(vec![group.clone(), attack.clone()])
        );
        assert_eq!(
            press([scout, none], snipable, false),
            Some(vec![group, attack])
        );
    }

    /// **The craft buttons without a walk** (item 4264): the row a unit
    /// selection's crafts make, each once; an untargeted craft issued at
    /// once; a targeted one waiting for the pick, then naming the object
    /// under it at the ground point; a ground craft naming the point alone;
    /// nothing for a craft no member offers, a building selection, or a
    /// pick on no object; a right press drops the pick.
    #[test]
    fn minted_spell_button_edges() {
        if lobby().is_err() {
            println!("skipping: no lobby");
            return;
        }
        use sim::observe::Craft;
        let terrain = TerrainScene::default();
        let c = crate::pinned_camera();
        let march = Craft {
            t: 0x27c,
            targeted: false,
            ground: false,
        };
        let bribe = Craft {
            t: 0x275,
            targeted: true,
            ground: false,
        };
        let drop = Craft {
            t: 0x284,
            targeted: true,
            ground: true,
        };
        // The caster at x 0, another slot's unit at x 768 × 8.
        let scene = Scene {
            units: vec![
                caster(0, 5, 0, &[march, bribe, drop]),
                caster(0, 6, 768, &[march]),
                unit(1, 9, 0x32, 768 * 8, 0, 0),
            ],
            ..Scene::default()
        };
        let v = view(&c, &scene, &terrain);
        let selected = |o: &[i16]| {
            let mut input = Input::new(0);
            input.selection = o.iter().map(|&o| ObjId { who: 0, o }).collect();
            input
        };
        let row: Vec<i32> = crate::hud::spell_menu(&selected(&[5, 6]), &scene, HUD_SIZE)
            .iter()
            .map(|b| b.t)
            .collect();
        assert_eq!(row, vec![0x275, 0x27c, 0x284]);
        // Untargeted: the group and spell(t, −1, −1, 0, 0) at once.
        let mut input = selected(&[5, 6]);
        let got = input.event(7, Event::Spell(0x27c), &v).map(|o| o.2);
        let group = Command::Group {
            who: 0,
            objects: vec![5, 6],
        };
        let no_point = Pos { x: 0, y: 0 };
        assert_eq!(
            got,
            Some(vec![group.clone(), spell(0x27c, -1, -1, no_point)])
        );
        // Through the HUD: a press on the button is the same event, and
        // its group repeats the one just sent, so it goes as no objects
        // (`add_group`, item 4267).
        let b = crate::hud::spell_menu(&input, &scene, HUD_SIZE)[1];
        let (bx, by) = (b.rect.x + 1, b.rect.y + 1);
        let got = crate::hud::event(&mut input, HUD_SIZE, 7, Event::Press { x: bx, y: by }, &v)
            .map(|o| o.2);
        let repeat = Command::Group {
            who: 0,
            objects: vec![],
        };
        assert_eq!(got, Some(vec![repeat, spell(0x27c, -1, -1, no_point)]));
        // Targeted: nothing on the button, then the pick on slot 1's unit.
        let mut input = selected(&[5, 6]);
        assert_eq!(input.event(7, Event::Spell(0x275), &v), None);
        assert_eq!(input.casting, Some(0x275));
        let target = scene.units[2].pos;
        let (px, py) = v.pixel(target.x, target.y, 0);
        let at = ground_at(&v, px, py).expect("the target stands on the map");
        let got = input
            .event(8, Event::Press { x: px, y: py }, &v)
            .map(|o| o.2);
        assert_eq!(got, Some(vec![group.clone(), spell(0x275, 9, 1, at)]));
        assert_eq!(input.casting, None);
        // A pick on no object: nothing, and the pick is spent.
        let mut input = selected(&[5, 6]);
        input.event(7, Event::Spell(0x275), &v);
        let (ex, ey) = v.pixel(768 * 30, 768 * 30, 0);
        assert_eq!(input.event(8, Event::Press { x: ex, y: ey }, &v), None);
        assert_eq!(input.casting, None);
        // A ground craft: the point alone.
        let mut input = selected(&[5]);
        input.event(7, Event::Spell(0x284), &v);
        let at = ground_at(&v, ex, ey).expect("on the map");
        let got = input
            .event(8, Event::Press { x: ex, y: ey }, &v)
            .map(|o| o.2);
        let one = Command::Group {
            who: 0,
            objects: vec![5],
        };
        assert_eq!(got, Some(vec![one, spell(0x284, -1, -1, at)]));
        // A right press drops the pick.
        let mut input = selected(&[5]);
        input.event(7, Event::Spell(0x275), &v);
        input.event(8, Event::RightPress { x: ex, y: ey }, &v);
        assert_eq!(input.casting, None);
        // No member offers it; a building selection; another slot's unit.
        assert_eq!(selected(&[6]).event(7, Event::Spell(0x275), &v), None);
        let mut b = selected(&[5]);
        b.building = true;
        assert_eq!(b.event(7, Event::Spell(0x27c), &v), None);
        let mut theirs = Input::new(0);
        theirs.selection = vec![ObjId { who: 1, o: 9 }];
        assert_eq!(theirs.event(7, Event::Spell(0x27c), &v), None);
    }

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
            let mut last = LastSent::new();
            for p in &rec.packages {
                let Ok(cmds) = rondata::commands::decode(&p.data) else {
                    continue;
                };
                let kept: Vec<Command> =
                    cmds.into_iter().filter(rondata::input::is_input).collect();
                let (who, objects, before) = behind(&last, p.play, &kept);
                note_sent(&mut last, p.play, &kept);
                let Some(Command::CityGather { t, .. }) = kept
                    .iter()
                    .find(|c| matches!(c, Command::CityGather { .. }))
                else {
                    continue;
                };
                all += 1;
                let scene = Scene {
                    buildings: objects.iter().map(|&o| city_building(who, o)).collect(),
                    ..Scene::default()
                };
                let v = view(&c, &scene, &terrain);
                let mut input = Input::new(who);
                input.selection = objects.iter().map(|&o| ObjId { who, o }).collect();
                input.building = true;
                input.sent = sent_for(&before, &scene);
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
                enemies: vec![],
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
            enemies: vec![],
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
            uid: 0,
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
            captain: o,
            crafts: vec![],
            spy: sim::observe::SpyMarks::default(),
            garrison_cost: 1,
            enters: vec![],
            figures: vec![],
        }
    }

    /// **The right drag** (item 4295, `docs/CLIENT_INPUT.md` §34.3): a
    /// `move_to` to the press's ground point, `set_angle` 1, the angle
    /// `find_angle` takes from it to the release's, the minimap's queue,
    /// and ctrl not read; nothing for a building, an empty selection, or a
    /// trader whose press names an object. The angles are pinned in the
    /// order's own units, a turn being `2^32`.
    #[test]
    fn minted_a_right_drag_faces_as_the_original() {
        let terrain = TerrainScene::default();
        let c = crate::pinned_camera();
        let caravan = sim::ids::ty::CARAVAN as i32;
        let scene = Scene {
            units: vec![unit(0, 5, 2, 0, 9, 50), unit(0, 6, caravan, 4000, 0, 50)],
            ..Scene::default()
        };
        let v = view(&c, &scene, &terrain);
        let mods = |shift, ctrl, alt| Mods { shift, ctrl, alt };
        let drag = |sel: i16, m: Mods, (x0, y0): (i32, i32), (x1, y1): (i32, i32)| {
            let mut input = Input {
                selection: vec![ObjId { who: 0, o: sel }],
                mods: m,
                ..Input::new(0)
            };
            input.event(3, Event::RightDrag { x0, y0, x1, y1 }, &v)
        };
        let none = mods(false, false, false);
        let (press, at) = ((900, 500), ground_at(&v, 900, 500).unwrap());
        // The four screen directions from one press, and no move at all.
        let mut angles = vec![];
        for release in [(1000, 500), (900, 600), (800, 500), (900, 400), (900, 500)] {
            let got = drag(5, none, press, release).unwrap();
            assert_eq!(got.0, 3);
            let Command::MoveTo {
                to_x,
                to_y,
                set_angle,
                angle,
                orders,
                queued,
                form,
                width,
                disembark,
            } = got.2[1]
            else {
                panic!("{got:?}");
            };
            assert_eq!(
                (
                    to_x, to_y, set_angle, orders, queued, form, width, disembark
                ),
                (at.x, at.y, 1, 1, QUEUE_NEW as i8, 9, 50, 0),
                "the press's point and the click's fields"
            );
            let to = ground_at(&v, release.0, release.1).unwrap();
            assert_eq!(angle, sim::movement::find_angle(to.x - at.x, to.y - at.y).0);
            angles.push(angle as u32);
        }
        println!("RIGHT DRAG ANGLES {angles:#x?}");
        assert_eq!(angles, RIGHT_DRAG_MINTED);
        // The queue is the minimap's; ctrl is not read.
        let queue = |m| match drag(5, m, press, (1000, 500)).unwrap().2[1] {
            Command::MoveTo { queued, .. } => i32::from(queued),
            _ => unreachable!(),
        };
        assert_eq!(
            [
                queue(mods(true, false, false)),
                queue(mods(false, false, true)),
                queue(mods(true, false, true)),
                queue(mods(false, true, false)),
            ],
            [QUEUE_LAST, QUEUE_FIRST, QUEUE_LAST, QUEUE_NEW]
        );
        // A trader's press on an object is the trade route, not built.
        let (cx, cy) = v.pixel(4000, 0, 0);
        assert!(drag(6, none, (cx, cy), (cx + 100, cy)).is_none());
        assert!(drag(6, none, press, (1000, 500)).is_some());
        assert!(drag(5, none, (cx, cy), (cx + 100, cy)).is_some());
        let mut building = Input {
            selection: vec![ObjId { who: 0, o: 2000 }],
            building: true,
            ..Input::new(0)
        };
        let d = Event::RightDrag {
            x0: 900,
            y0: 500,
            x1: 1000,
            y1: 500,
        };
        assert_eq!(building.event(3, d, &v), None);
        assert_eq!(Input::new(0).event(3, d, &v), None);
    }

    /// **The attack-move** (item 4297, `docs/CLIENT_INPUT.md` §35.3): ctrl on
    /// a unit selection's right press mints `ATTACK_TO` to the ground
    /// point, no facing, −1/−1, the dispatch's queue, before any object
    /// under the press; with alt too, `MOVE_TO` with the formation, unless
    /// the profile's `0x400` hides alt. On the minimap ctrl mints
    /// `ATTACK_TO` at the map point with the minimap's queue, alt or not.
    /// A building's selection keeps its gather point.
    #[test]
    fn minted_an_attack_move_as_the_original() {
        let terrain = TerrainScene {
            width: 64,
            height: 64,
            ..TerrainScene::default()
        };
        let c = crate::pinned_camera();
        let bare = Scene::default();
        let probe = view(&c, &bare, &terrain);
        let at = ground_at(&probe, 900, 500).unwrap();
        // An enemy unit and one's own, both under the press.
        let mut enemy = unit(1, 7, 2, at.x, 0, 50);
        enemy.pos = at;
        let mut friend = unit(0, 6, 2, at.x, 0, 50);
        friend.pos = Pos {
            x: at.x,
            y: at.y + 1,
        };
        let scene = Scene {
            units: vec![unit(0, 5, 2, 0, 9, 50), enemy, friend],
            ..Scene::default()
        };
        let v = view(&c, &scene, &terrain);
        assert!(v.names_object(900, 500), "the press names an object");
        let mods = |shift, ctrl, alt| Mods { shift, ctrl, alt };
        let press = |m: Mods, prefs: u32, e: Event| {
            let mut input = Input {
                selection: vec![ObjId { who: 0, o: 5 }],
                mods: m,
                ..Input::new(0)
            };
            input.prefs.prefs |= prefs;
            input.event(3, e, &v).map(|o| o.2)
        };
        let ground = Event::RightPress { x: 900, y: 500 };
        let atk = |at: Pos, q: i32| Command::MoveTo {
            to_x: at.x,
            to_y: at.y,
            set_angle: 0,
            angle: 0,
            orders: 2,
            queued: q as i8,
            form: -1,
            width: -1,
            disembark: 0,
        };
        // Without ctrl the press on the enemy is the attack.
        let got = press(mods(false, false, false), 0, ground).unwrap();
        assert!(matches!(got[1], Command::Attack { .. }), "{got:?}");
        for (m, prefs, want) in [
            (mods(false, true, false), 0, atk(at, QUEUE_NEW)),
            (mods(true, true, false), 0, atk(at, QUEUE_LAST)),
            (mods(false, true, true), 0, move_to(at, 9, 50, QUEUE_NEW)),
            (mods(true, true, true), 0, move_to(at, 9, 50, QUEUE_FIRST)),
            (mods(false, true, true), 0x400, atk(at, QUEUE_NEW)),
            (mods(true, true, true), 0x400, atk(at, QUEUE_LAST)),
        ] {
            let got = press(m, prefs, ground).unwrap();
            assert_eq!(got.len(), 2, "{m:?}");
            assert!(matches!(got[0], Command::Group { .. }));
            assert_eq!(got[1], want, "{m:?} prefs {prefs:#x}");
            assert_eq!(
                rondata::commands::encode(&got[1..]),
                rondata::commands::encode(std::slice::from_ref(&want))
            );
        }
        // The minimap: `ATTACK_TO` in place of `MOVE_TO`, alt or not.
        let to = Pos { x: 9574, y: 25876 };
        let map = Event::MapRightPress { x: to.x, y: to.y };
        for (m, want) in [
            (mods(false, true, false), atk(to, QUEUE_NEW)),
            (mods(true, true, false), atk(to, QUEUE_LAST)),
            (mods(false, true, true), atk(to, QUEUE_FIRST)),
            (mods(false, false, false), move_to(to, -1, -1, QUEUE_NEW)),
        ] {
            assert_eq!(press(m, 0, map).unwrap()[1], want, "{m:?} the minimap");
        }
        // Nothing for an empty selection; a building's selection takes the
        // minimap's gather point, `add_to_end` 1 under ctrl (item 4302).
        let ctrl = mods(false, true, false);
        let mut empty = Input {
            mods: ctrl,
            ..Input::new(0)
        };
        assert_eq!(empty.event(3, ground, &v), None);
        assert_eq!(empty.event(3, map, &v), None);
        let mut building = Input {
            selection: vec![ObjId { who: 0, o: 2000 }],
            building: true,
            mods: ctrl,
            ..Input::new(0)
        };
        assert_eq!(
            building.event(3, map, &v).map(|o| o.2[1].clone()),
            Some(Command::GatherPoint {
                x: to.x,
                y: to.y,
                action: 0,
                add_to_end: 1
            })
        );
    }

    /// The angles [`minted_a_right_drag_faces_as_the_original`] mints at
    /// the pinned camera, for a drag right, down, left and up the screen
    /// from `(900, 500)`, and none: no move is `find_angle(0, 0)`, south.
    /// The camera's turn puts a drag right on the screen near north-east.
    const RIGHT_DRAG_MINTED: [u32; 5] = [
        0x204d_0000,
        0x5fbc_0000,
        0x9fc5_0000,
        0xe044_0000,
        0x8000_0000,
    ];

    /// The window's slop (§34.1): `vector_dist` of the screen legs past 6.
    #[test]
    fn a_right_press_drags_past_the_slop() {
        assert!(!right_dragged((10, 10), (10, 10)));
        assert!(!right_dragged((10, 10), (16, 10)));
        assert!(right_dragged((10, 10), (17, 10)));
        assert!(!right_dragged((10, 10), (14, 14)));
        assert!(right_dragged((10, 10), (15, 15)));
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

    /// **The rename's button** (item 4268, `docs/CLIENT_INPUT.md` §21):
    /// stage 0's pixel, the centre of [`crate::hud::rename_button`] under
    /// [`HUD_SIZE`] with the capital selected.
    const RENAME_AT: (i32, i32) = (1822, 966);

    /// The unqueued events and the rename's: the capital is still selected
    /// from the gather, so a press on its RENAME button, the name typed,
    /// and Return, all on [`script::RENAME_FRAME`].
    fn rename_events() -> Vec<Scripted> {
        let mut e = unqueue_events();
        let (x, y) = RENAME_AT;
        let f = script::RENAME_FRAME;
        e.push((f, None, Event::Press { x, y }));
        e.extend(
            script::RENAME_NAME
                .chars()
                .map(|k| (f, None, Event::Key(k))),
        );
        e.push((f, None, Event::Enter));
        e
    }

    /// **`INPUT_RENAMED`** (item 4268, `docs/CLIENT_INPUT.md` §21): the
    /// unqueued events and the rename's, through the HUD and the layer
    /// under the opening camera, mint [`script::renamed`] field for field,
    /// of 23.
    const INPUT_RENAMED: usize = 23;

    /// **The renamed count**: stage 0 on the rename's frame — the capital
    /// is slot 0's, finished and in a city, and its RENAME button stands
    /// where the event presses it; then the events mint the renamed
    /// script, and the file written from them is the renamed file.
    #[test]
    fn minted_renamed_at_the_opening_camera() {
        let Ok(mut r) = lobby() else {
            println!("skipping: no lobby");
            return;
        };
        let unqueued = script::unqueued();
        let mut next = 0;
        while r.frame() < i64::from(script::RENAME_FRAME) {
            let f = r.frame() as i32;
            while next < unqueued.len() && unqueued[next].0 == f {
                r.issue(&unqueued[next]);
                next += 1;
            }
            r.step();
        }
        {
            let scene = r.scene(Some(0));
            let id = ObjId {
                who: 0,
                o: script::CAPITAL,
            };
            let b = scene.buildings.iter().find(|b| b.id == id).unwrap();
            let mut input = Input::new(0);
            input.selection = vec![id];
            input.building = true;
            let at = crate::hud::rename_button(&input, &scene, HUD_SIZE).map(|r| r.centre());
            println!(
                "RENAME STAGE0 f{} active {} in_city {} button {at:?}",
                r.frame(),
                b.active,
                b.in_city
            );
            assert!(b.active && b.in_city);
            assert_eq!(at, Some(RENAME_AT));
        }
        drop(r);

        let Some((ours, r)) = mint_with(rename_events(), |input, frame, ev, view| {
            crate::hud::event(input, HUD_SIZE, frame, ev, view)
        }) else {
            return;
        };
        let renamed = script::renamed();
        let whole = ours.iter().zip(&renamed).filter(|(a, b)| a == b).count();
        assert_eq!(
            (ours.len(), whole),
            (renamed.len(), INPUT_RENAMED),
            "{ours:?}"
        );
        assert_eq!(ours, renamed);
        drop(r);

        let home = std::env::var_os("HOME").unwrap_or_default();
        let (Ok(repeated), Ok(file)) = (
            std::fs::read(std::path::Path::new(&home).join(REPEATED_RCX)),
            std::fs::read(std::path::Path::new(&home).join(RENAMED_RCX)),
        ) else {
            println!("skipping the bytes: no {REPEATED_RCX} or {RENAMED_RCX}");
            return;
        };
        assert_eq!(file.len(), RENAMED_RCX_BYTES);
        let written = written(&ours).unwrap();
        assert!(written == file, "the file from the renamed orders");
        assert!(written != repeated, "not the repeated file");
    }

    /// The renamed events and the hot key's: the capital is still selected
    /// from the gather, so ctrl and [`script::HOTKEY_KEY`] on
    /// [`script::HOTKEY_FRAME`] (item 4270).
    fn hotkey_events() -> Vec<Scripted> {
        let mut e = rename_events();
        e.push((
            script::HOTKEY_FRAME,
            None,
            Event::Hotkey(script::HOTKEY_KEY as u8),
        ));
        e
    }

    /// **`INPUT_HOTKEYED`** (item 4270, `docs/CLIENT_INPUT.md` §22): the
    /// renamed events and the hot key, through the HUD and the layer under
    /// the opening camera, mint [`script::hotkeyed`] field for field, of 24.
    const INPUT_HOTKEYED: usize = 24;

    /// **The hot-keyed count**: stage 0 on the key's frame — the capital,
    /// the selection the key copies, is slot 0's and standing; then the
    /// events mint the hot-keyed script, and the file written from them is
    /// the hot-keyed file.
    #[test]
    fn minted_hotkeyed_at_the_opening_camera() {
        let Ok(mut r) = lobby() else {
            println!("skipping: no lobby");
            return;
        };
        let renamed = script::renamed();
        let mut next = 0;
        while r.frame() < i64::from(script::HOTKEY_FRAME) {
            let f = r.frame() as i32;
            while next < renamed.len() && renamed[next].0 == f {
                r.issue(&renamed[next]);
                next += 1;
            }
            r.step();
        }
        {
            let scene = r.scene(Some(0));
            let id = ObjId {
                who: 0,
                o: script::CAPITAL,
            };
            let b = scene.buildings.iter().find(|b| b.id == id);
            println!(
                "HOTKEY STAGE0 f{} capital {:?}",
                r.frame(),
                b.map(|b| (b.active, b.uid))
            );
            assert!(b.is_some_and(|b| b.active));
        }
        drop(r);

        let Some((ours, r)) = mint_with(hotkey_events(), |input, frame, ev, view| {
            crate::hud::event(input, HUD_SIZE, frame, ev, view)
        }) else {
            return;
        };
        let hotkeyed = script::hotkeyed();
        let whole = ours.iter().zip(&hotkeyed).filter(|(a, b)| a == b).count();
        assert_eq!(
            (ours.len(), whole),
            (hotkeyed.len(), INPUT_HOTKEYED),
            "{ours:?}"
        );
        assert_eq!(ours, hotkeyed);
        drop(r);
        let home = std::env::var_os("HOME").unwrap_or_default();
        let (Ok(renamed), Ok(file)) = (
            std::fs::read(std::path::Path::new(&home).join(RENAMED_RCX)),
            std::fs::read(std::path::Path::new(&home).join(HOTKEYED_RCX)),
        ) else {
            println!("skipping the bytes: no {RENAMED_RCX} or {HOTKEYED_RCX}");
            return;
        };
        assert_eq!(file.len(), HOTKEYED_RCX_BYTES);
        let written = written(&ours).unwrap();
        assert!(written == file, "the file from the hot-keyed orders");
        assert!(written != renamed, "not the renamed file");
    }

    /// The hot-keyed file (`docs/CLIENT_INPUT.md` §22), written by
    /// rondata's `the_hotkeyed_minted_packages_round_trip` under
    /// `RON_RCX_OUT`; the file run1610 takes open.
    const HOTKEYED_RCX: &str = "ron-golden/recordings/item4270-hotkeyed-greatlakes-24000.rcx";
    const HOTKEYED_RCX_BYTES: usize = 59_605;

    /// **`INPUT_HOTKEYS`** (item 4270): of the hot keys the recordings on
    /// disk carry, those the package's selection and ctrl with the key the
    /// group names (`group − who × 18`) mint field for field, the `group`
    /// before it included, of all of them. Seen red pinned (0, 0).
    const INPUT_HOTKEYS: (usize, usize) = (8, 8);

    /// **Stage 0 of the garrisoned script** (item 4273,
    /// `docs/CLIENT_INPUT.md` §23): on [`script::GARRISON_FRAME`] the
    /// Citizen [`script::GARRISON_CITIZEN`]'s anchor and the capital's, at
    /// the opening camera.
    const GARRISON_ANCHORS: [AttackAnchor; 2] = [
        (
            script::GARRISON_FRAME,
            (0, script::GARRISON_CITIZEN),
            (1369, 468),
        ),
        (script::GARRISON_FRAME, (0, script::CAPITAL), (1127, 521)),
    ];

    /// **The garrison button** (item 4273): the centre of
    /// [`crate::hud::garrison_button`] under [`HUD_SIZE`] with the Citizen
    /// selected.
    const GARRISON_AT: (i32, i32) = (1822, 1010);

    /// The hot-keyed events and the garrison's three, on
    /// [`script::GARRISON_FRAME`]: a press on the Citizen, a press on the
    /// garrison button, and the pick on the capital.
    fn garrison_events() -> Vec<Scripted> {
        let mut e = hotkey_events();
        let (f, _, (ux, uy)) = GARRISON_ANCHORS[0];
        let (_, _, (cx, cy)) = GARRISON_ANCHORS[1];
        let (gx, gy) = GARRISON_AT;
        e.extend([
            (f, None, Event::Press { x: ux, y: uy }),
            (f, None, Event::Press { x: gx, y: gy }),
            (f, None, Event::Press { x: cx, y: cy }),
        ]);
        e
    }

    /// **`INPUT_GARRISONED`** (item 4273, `docs/CLIENT_INPUT.md` §23): the
    /// hot-keyed events and the garrison's, through the HUD and the layer
    /// under the opening camera, mint [`script::garrisoned`] field for
    /// field, of 25.
    const INPUT_GARRISONED: usize = 25;

    /// **The garrisoned count**: stage 0 on the garrison's frame — the
    /// Citizen is slot 0's, on the map, and admitted into the capital,
    /// which is finished with room; both anchors and the button stand where
    /// the events press; then the events mint the garrisoned script, and
    /// the file written from them is the garrisoned file.
    #[test]
    fn minted_garrisoned_at_the_opening_camera() {
        let Ok(mut r) = lobby() else {
            println!("skipping: no lobby");
            return;
        };
        let hotkeyed = script::hotkeyed();
        let c = camera(None);
        let mut next = 0;
        while r.frame() < i64::from(script::GARRISON_FRAME) {
            let f = r.frame() as i32;
            while next < hotkeyed.len() && hotkeyed[next].0 == f {
                r.issue(&hotkeyed[next]);
                next += 1;
            }
            r.step();
        }
        {
            let (scene, terrain) = (r.scene(Some(0)), r.terrain());
            let v = view(&c, &scene, &terrain);
            let uid = ObjId {
                who: 0,
                o: script::GARRISON_CITIZEN,
            };
            let cid = ObjId {
                who: 0,
                o: script::CAPITAL,
            };
            let u = scene.units.iter().find(|u| u.id == uid).unwrap();
            let b = scene.buildings.iter().find(|b| b.id == cid).unwrap();
            let mut input = Input::new(0);
            input.selection = vec![uid];
            let at = crate::hud::garrison_button(&input, &scene, HUD_SIZE).map(|r| r.centre());
            println!(
                "GARRISON STAGE0 f{} citizen ti {} pixel {:?} cost {} enters {:?} capital pixel {:?} active {} limit {} inside {} button {at:?}",
                r.frame(),
                u.type_index,
                v.pixel(u.pos.x, u.pos.y, u.z),
                u.garrison_cost,
                u.enters,
                v.pixel(b.pos.x, b.pos.y, b.z),
                b.active,
                b.garrison_limit,
                b.inside
            );
            assert_eq!(u.type_index, script::CITIZEN);
            assert_eq!(v.pixel(u.pos.x, u.pos.y, u.z), GARRISON_ANCHORS[0].2);
            assert_eq!(v.pixel(b.pos.x, b.pos.y, b.z), GARRISON_ANCHORS[1].2);
            assert!(u.enters.contains(&script::CAPITAL));
            assert!(b.active && b.inside + u.garrison_cost <= b.garrison_limit);
            assert_eq!(at, Some(GARRISON_AT));
        }
        drop(r);

        let Some((ours, r)) = mint_with(garrison_events(), |input, frame, ev, view| {
            crate::hud::event(input, HUD_SIZE, frame, ev, view)
        }) else {
            return;
        };
        let garrisoned = script::garrisoned();
        let whole = ours.iter().zip(&garrisoned).filter(|(a, b)| a == b).count();
        assert_eq!(
            (ours.len(), whole),
            (garrisoned.len(), INPUT_GARRISONED),
            "{ours:?}"
        );
        assert_eq!(ours, garrisoned);
        drop(r);
        let home = std::env::var_os("HOME").unwrap_or_default();
        let (Ok(hotkeyed), Ok(file)) = (
            std::fs::read(std::path::Path::new(&home).join(HOTKEYED_RCX)),
            std::fs::read(std::path::Path::new(&home).join(GARRISONED_RCX)),
        ) else {
            println!("skipping the bytes: no {HOTKEYED_RCX} or {GARRISONED_RCX}");
            return;
        };
        assert_eq!(file.len(), GARRISONED_RCX_BYTES);
        let written = written(&ours).unwrap();
        assert!(written == file, "the file from the garrisoned orders");
        assert!(written != hotkeyed, "not the hot-keyed file");
    }

    /// The garrisoned file (`docs/CLIENT_INPUT.md` §23), written by
    /// rondata's `the_garrisoned_minted_packages_round_trip` under
    /// `RON_RCX_OUT`; the file run1613 takes open.
    const GARRISONED_RCX: &str = "ron-golden/recordings/item4273-garrisoned-greatlakes-24000.rcx";
    const GARRISONED_RCX_BYTES: usize = 59_620;

    /// **`INPUT_GARRISONS`** (item 4273): of the garrisons the recordings
    /// on disk carry, those the garrison button and a pick on the building
    /// the order names mint field for field, the package's orders whole,
    /// from the package's selection (an empty `group` the slot's last full
    /// one, `add_group`'s repeat), each unit admitted and the building
    /// finished with room. Seen red pinned (0, 0).
    const INPUT_GARRISONS: (usize, usize) = (6, 6);

    /// A building for the garrison tests: the lobby's capital record as
    /// `who`'s `o`, finished, with `limit` and `inside`.
    fn garrison_building(who: u8, o: i16, limit: i32, inside: i32) -> sim::observe::BuildingScene {
        let mut b = city_building(who, o);
        (b.active, b.garrison_limit, b.inside) = (true, limit, inside);
        b
    }

    /// A unit of `who`'s admitted into `enters`.
    fn garrison_unit(who: u8, o: i16, enters: &[i16]) -> sim::observe::UnitScene {
        sim::observe::UnitScene {
            enters: enters.to_vec(),
            ..unit(who, o, script::CITIZEN, 0, 0, 0)
        }
    }

    /// **Every recorded garrison** (item 4273): each package carrying a
    /// `garrison` is minted from its selection, the button and a pick on
    /// the building's anchor.
    #[test]
    fn minted_garrisons_equal_the_recordings() {
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
            let mut last = LastSent::new();
            for p in &rec.packages {
                let Ok(cmds) = rondata::commands::decode(&p.data) else {
                    continue;
                };
                let kept: Vec<Command> =
                    cmds.into_iter().filter(rondata::input::is_input).collect();
                let (who, objects, before) = behind(&last, p.play, &kept);
                note_sent(&mut last, p.play, &kept);
                let Some(&Command::Garrison { ox, whom, .. }) =
                    kept.iter().find(|c| matches!(c, Command::Garrison { .. }))
                else {
                    continue;
                };
                all += 1;
                let o = ox as i16;
                let b = garrison_building(whom as u8, o, 10, 0);
                let scene = Scene {
                    units: objects
                        .iter()
                        .map(|&u| garrison_unit(who, u, &[o]))
                        .collect(),
                    buildings: vec![b],
                    ..Scene::default()
                };
                let v = view(&c, &scene, &terrain);
                let mut input = Input::new(who);
                input.selection = objects.iter().map(|&u| ObjId { who, o: u }).collect();
                input.sent = sent_for(&before, &scene);
                let (bx, by) = v.pixel(b.pos.x, b.pos.y, b.z);
                assert_eq!(input.event(p.frame, Event::Garrison, &v), None);
                let got = input.event(p.frame, Event::Press { x: bx, y: by }, &v);
                // The other orders the package carries (a camera) are not
                // the layer's: what is compared is its kept orders.
                let mine: Vec<Command> = kept
                    .iter()
                    .filter(|c| !matches!(c, Command::Camera { .. }))
                    .cloned()
                    .collect();
                if got.as_ref().map(|o| &o.2) == Some(&mine) {
                    whole += 1;
                } else {
                    println!("GARRISON PARTS f{} {kept:?}: {got:?}", p.frame);
                }
            }
        }
        println!("GARRISONS {whole} of {all}");
        assert_eq!((whole, all), INPUT_GARRISONS);
    }

    /// **The garrison's edges** (item 4273): the button only for a unit
    /// selection with a member admitted somewhere; no pick without a unit
    /// selection; a pick on no building keeps the pick; a building with no
    /// limit, an unfinished one, a full one or one that admits no member
    /// spends it and mints nothing; the first admitted member's slots
    /// decide; a right press drops the pick.
    #[test]
    fn minted_garrison_edges() {
        if lobby().is_err() {
            println!("skipping: no lobby");
            return;
        }
        let terrain = TerrainScene::default();
        let c = crate::pinned_camera();
        let mint = |scene: &Scene, ids: &[i16], at_building: bool| {
            let v = view(&c, scene, &terrain);
            let mut input = Input::new(0);
            input.selection = ids.iter().map(|&o| ObjId { who: 0, o }).collect();
            input.event(9, Event::Garrison, &v);
            let armed = input.garrisoning;
            let (x, y) = if at_building {
                let b = &scene.buildings[0];
                v.pixel(b.pos.x, b.pos.y, b.z)
            } else {
                (-500, -500)
            };
            let got = input.event(9, Event::Press { x, y }, &v).map(|o| o.2);
            (armed, got, input.garrisoning)
        };
        let scene = |units: Vec<sim::observe::UnitScene>, b: sim::observe::BuildingScene| Scene {
            units,
            buildings: vec![b],
            ..Scene::default()
        };
        let garrison = Command::Garrison {
            ox: 2000,
            whom: 0,
            queued: QUEUE_NEW,
        };
        let group = |objects: Vec<i16>| Command::Group { who: 0, objects };
        let ok = scene(
            vec![garrison_unit(0, 5, &[2000])],
            garrison_building(0, 2000, 10, 0),
        );
        assert_eq!(
            mint(&ok, &[5], true),
            (true, Some(vec![group(vec![5]), garrison.clone()]), false)
        );
        // The button.
        let mut input = Input::new(0);
        input.selection = vec![ObjId { who: 0, o: 5 }];
        assert_eq!(
            crate::hud::garrison_button(&input, &ok, HUD_SIZE).map(|r| r.centre()),
            Some(GARRISON_AT)
        );
        let none = scene(
            vec![garrison_unit(0, 5, &[])],
            garrison_building(0, 2000, 10, 0),
        );
        assert_eq!(crate::hud::garrison_button(&input, &none, HUD_SIZE), None);
        input.selection = vec![ObjId { who: 0, o: 2000 }];
        input.building = true;
        assert_eq!(crate::hud::garrison_button(&input, &ok, HUD_SIZE), None);
        let v = view(&c, &ok, &terrain);
        assert_eq!(input.event(9, Event::Garrison, &v), None);
        assert!(!input.garrisoning, "a building selection takes no pick");
        assert_eq!(Input::new(0).event(9, Event::Garrison, &v), None);
        // A pick on no building keeps the pick.
        assert_eq!(mint(&ok, &[5], false), (true, None, true));
        // No limit, unfinished, full, not admitted: spent, nothing.
        let mut unfinished = garrison_building(0, 2000, 10, 0);
        unfinished.active = false;
        for s in [
            scene(
                vec![garrison_unit(0, 5, &[2000])],
                garrison_building(0, 2000, 0, 0),
            ),
            scene(vec![garrison_unit(0, 5, &[2000])], unfinished),
            scene(
                vec![garrison_unit(0, 5, &[2000])],
                garrison_building(0, 2000, 10, 10),
            ),
            none.clone(),
        ] {
            assert_eq!(mint(&s, &[5], true), (true, None, false));
        }
        // The first admitted member decides: 4 is not admitted, 5 costs 3
        // into a building with two slots left.
        let mut costly = garrison_unit(0, 5, &[2000]);
        costly.garrison_cost = 3;
        let two_left = scene(
            vec![
                garrison_unit(0, 4, &[]),
                costly,
                garrison_unit(0, 6, &[2000]),
            ],
            garrison_building(0, 2000, 10, 8),
        );
        assert_eq!(mint(&two_left, &[4, 5, 6], true), (true, None, false));
        assert_eq!(
            mint(&two_left, &[4, 6, 5], true),
            (true, Some(vec![group(vec![4, 6, 5]), garrison]), false)
        );
        // A right press drops the pick.
        let v = view(&c, &ok, &terrain);
        let mut input = Input::new(0);
        input.selection = vec![ObjId { who: 0, o: 5 }];
        input.event(9, Event::Garrison, &v);
        assert!(input.garrisoning);
        input.event(9, Event::RightPress { x: -500, y: -500 }, &v);
        assert!(!input.garrisoning);
    }

    /// **Every recorded hot key** (item 4270): each package carrying a
    /// `hotkey` is minted from a fresh layer whose selection is the
    /// package's group, and ctrl with the key.
    #[test]
    fn minted_hotkeys_equal_the_recordings() {
        let files = recordings_on_disk();
        if files.is_empty() {
            println!("skipping: no .rcx on disk");
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
                let Some(Command::Hotkey { group, .. }) = kept
                    .iter()
                    .find(|c| matches!(c, Command::Hotkey { .. }))
                    .cloned()
                else {
                    continue;
                };
                all += 1;
                let Some(Command::Group { who, objects }) = kept.first().cloned() else {
                    println!("HOTKEY WITHOUT A GROUP f{} {kept:?}", p.frame);
                    continue;
                };
                let (sent_who, who) = (who, who as u8);
                let scene = Scene::default();
                let v = view(&c, &scene, &terrain);
                let mut input = Input::new(who);
                input.selection = objects.iter().map(|&o| ObjId { who, o }).collect();
                let key = group - i32::from(sent_who) * i32::from(HOTKEY_SLOTS);
                let got = u8::try_from(key)
                    .ok()
                    .and_then(|k| input.event(p.frame, Event::Hotkey(k), &v));
                if got.as_ref().map(|o| &o.2) == Some(&kept) {
                    whole += 1;
                } else {
                    println!("HOTKEY PARTS f{} {kept:?}: {got:?}", p.frame);
                }
            }
        }
        println!("HOTKEYS {whole} of {all}");
        assert_eq!((whole, all), INPUT_HOTKEYS);
    }

    /// **The hot key's edges** (item 4270): no selection sends nothing (the
    /// original's screen group is not sent); a key past F8 sends nothing;
    /// slot 1's keys are numbered from 18; and a key while the entry line
    /// is open sends nothing.
    #[test]
    fn minted_hotkey_edges() {
        let scene = Scene::default();
        let terrain = TerrainScene::default();
        let c = crate::pinned_camera();
        let v = view(&c, &scene, &terrain);
        let mut input = Input::new(1);
        assert_eq!(input.event(5, Event::Hotkey(3), &v), None);
        input.selection = vec![ObjId { who: 1, o: 2001 }];
        assert_eq!(input.event(5, Event::Hotkey(HOTKEY_SLOTS), &v), None);
        let got = input.event(5, Event::Hotkey(17), &v).unwrap();
        assert_eq!(
            got.2,
            vec![
                Command::Group {
                    who: 1,
                    objects: vec![2001]
                },
                Command::Hotkey {
                    group: 35,
                    clear: 0,
                    valid: 0,
                    x: HOTKEY_NO_CAMERA,
                    y: HOTKEY_NO_CAMERA,
                    zoom: 0
                }
            ]
        );
        input.line = Some(String::new());
        assert_eq!(input.event(5, Event::Hotkey(1), &v), None);
    }

    /// The renamed file (`docs/CLIENT_INPUT.md` §21), written by rondata's
    /// `the_renamed_minted_packages_round_trip` under `RON_RCX_OUT`; the
    /// file run1609 takes open.
    const RENAMED_RCX: &str = "ron-golden/recordings/item4268-renamed-greatlakes-24000.rcx";
    const RENAMED_RCX_BYTES: usize = 59_591;

    /// **`INPUT_RENAMES`** (item 4268): of the renames the recordings on
    /// disk carry, those the RENAME button, the recorded name typed and
    /// Return mint field for field, of all of them, the units past the
    /// terminator aside: all ten carry `issue_rename_city`'s stack there
    /// (`Command::RenameCity::after_nul`), which `process_rename_city`
    /// never reads and a command built here pads with zeros. Seen red
    /// pinned (0, 0).
    const INPUT_RENAMES: (usize, usize) = (10, 10);

    /// **Every recorded rename the city's name popup issues** (item 4268):
    /// each recorded `rename_city`'s building selected, finished and in a
    /// city, then its RENAME button, its name typed, and Return.
    #[test]
    fn minted_renames_equal_the_recordings() {
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
                let Some(Command::RenameCity { who, o, name, .. }) = kept
                    .iter()
                    .find(|c| matches!(c, Command::RenameCity { .. }))
                    .cloned()
                else {
                    continue;
                };
                all += 1;
                let (who, o) = (who as u8, o as i16);
                let mut b = city_building(who, o);
                b.in_city = true;
                let scene = Scene {
                    buildings: vec![b],
                    ..Scene::default()
                };
                let v = view(&c, &scene, &terrain);
                let mut input = Input::new(who);
                input.selection = vec![ObjId { who, o }];
                input.building = true;
                let mut got = input.event(p.frame, Event::Rename, &v);
                for k in name.chars() {
                    got = got.or(input.event(p.frame, Event::Key(k), &v));
                }
                got = got.or(input.event(p.frame, Event::Enter, &v));
                let read: Vec<Command> = kept
                    .iter()
                    .map(|c| match c {
                        Command::RenameCity { who, o, name, .. } => Command::RenameCity {
                            who: *who,
                            o: *o,
                            name: name.clone(),
                            after_nul: Vec::new(),
                        },
                        c => c.clone(),
                    })
                    .collect();
                if got.as_ref().map(|o| &o.2) == Some(&read) {
                    whole += 1;
                } else {
                    println!("RENAME PARTS f{} {kept:?}: {got:?}", p.frame);
                }
            }
        }
        println!("RENAMES {whole} of {all}");
        assert_eq!((whole, all), INPUT_RENAMES);
    }

    /// **The rename's edges** (item 4268): the button only for a finished
    /// building of the slot's own in a city; the box takes twenty-one
    /// units and the command carries twenty; the spaces at both ends go,
    /// a line left empty sends nothing, and Escape closes the box with
    /// nothing sent and the next Return opening chat's line.
    #[test]
    fn minted_rename_edges() {
        if lobby().is_err() {
            println!("skipping: no lobby");
            return;
        }
        let terrain = TerrainScene::default();
        let c = crate::pinned_camera();
        let mut b = city_building(0, 2000);
        b.in_city = true;
        let send = |b: sim::observe::BuildingScene, evs: &[Event]| {
            let scene = Scene {
                buildings: vec![b],
                ..Scene::default()
            };
            let v = view(&c, &scene, &terrain);
            let mut input = Input::new(0);
            input.selection = vec![b.id];
            input.building = true;
            let got: Vec<Order> = evs.iter().filter_map(|e| input.event(9, *e, &v)).collect();
            (got, input)
        };
        let typed = |text: &str| -> Vec<Event> {
            let mut e = vec![Event::Rename];
            e.extend(text.chars().map(Event::Key));
            e.push(Event::Enter);
            e
        };
        let named = |name: &str| {
            vec![(
                9,
                0,
                vec![Command::RenameCity {
                    who: 0,
                    o: 2000,
                    name: name.to_string(),
                    after_nul: Vec::new(),
                }],
            )]
        };
        assert_eq!(send(b, &typed("  Attrition ")).0, named("Attrition"));
        // Twenty-two typed: the box keeps twenty-one, the command twenty.
        let (got, _) = send(b, &typed("abcdefghijklmnopqrstuv"));
        assert_eq!(got, named("abcdefghijklmnopqrst"));
        assert_eq!(send(b, &typed("   ")).0, vec![]);
        let (got, input) = send(b, &[Event::Rename, Event::Key('x'), Event::Escape]);
        assert_eq!((got, input.line, input.renaming), (vec![], None, None));
        // Not the slot's, unfinished, in no city: no box opens.
        for edit in [
            |b: &mut sim::observe::BuildingScene| b.owner = 1,
            |b: &mut sim::observe::BuildingScene| b.active = false,
            |b: &mut sim::observe::BuildingScene| b.in_city = false,
        ] {
            let mut other = b;
            edit(&mut other);
            let (got, input) = send(other, &typed("Attrition"));
            assert!(input.renaming.is_none());
            // The Return that would have sent the name opens chat's line.
            assert!(got.is_empty(), "{got:?}");
            assert_eq!(input.line, Some(String::new()));
        }
    }

    /// **Stage 0 of the buildmasked script** (item 4276,
    /// `docs/CLIENT_INPUT.md` §24): the capital's anchor on
    /// [`script::BUILDMASK_TRAIN_FRAME`] at the opening camera.
    const BUILDMASK_CAPITAL_AT: (i32, i32) = (1127, 521);
    /// The Citizen's button, the menu's first with the capital selected,
    /// and the infinite-queue button
    /// ([`crate::hud::infinite_button`]), their centres under
    /// [`HUD_SIZE`].
    const BUILDMASK_TRAIN_AT: (i32, i32) = (128, 1054);
    const INFINITE_AT: (i32, i32) = (1634, 966);

    /// The garrisoned events and the buildmask's three: on
    /// [`script::BUILDMASK_TRAIN_FRAME`] a press on the capital and one on
    /// its Citizen button; on [`script::BUILDMASK_FRAME`] a press on the
    /// infinite-queue button.
    fn buildmask_events() -> Vec<Scripted> {
        let mut e = garrison_events();
        let ((cx, cy), (tx, ty), (ix, iy)) =
            (BUILDMASK_CAPITAL_AT, BUILDMASK_TRAIN_AT, INFINITE_AT);
        let (f, g) = (script::BUILDMASK_TRAIN_FRAME, script::BUILDMASK_FRAME);
        e.extend([
            (f, None, Event::Press { x: cx, y: cy }),
            (f, None, Event::Press { x: tx, y: ty }),
            (g, None, Event::Press { x: ix, y: iy }),
        ]);
        e
    }

    /// **`INPUT_BUILDMASKED`** (item 4276, `docs/CLIENT_INPUT.md` §24):
    /// the garrisoned events and the buildmask's, through the HUD and the
    /// layer under the opening camera, mint [`script::buildmasked`] field
    /// for field, of 27.
    const INPUT_BUILDMASKED: usize = 27;

    /// **The buildmasked count**: stage 0 on the Citizen's frame — the
    /// capital is slot 0's, finished, a training building with an empty
    /// queue, and stands where the press lands; both buttons stand where
    /// the events press; then the events mint the buildmasked script, and
    /// the file written from them is the buildmasked file.
    #[test]
    fn minted_buildmasked_at_the_opening_camera() {
        let Ok(mut r) = lobby() else {
            println!("skipping: no lobby");
            return;
        };
        let garrisoned = script::garrisoned();
        let c = camera(None);
        let mut next = 0;
        while r.frame() < i64::from(script::BUILDMASK_TRAIN_FRAME) {
            let f = r.frame() as i32;
            while next < garrisoned.len() && garrisoned[next].0 == f {
                r.issue(&garrisoned[next]);
                next += 1;
            }
            r.step();
        }
        {
            let (scene, terrain) = (r.scene(Some(0)), r.terrain());
            let v = view(&c, &scene, &terrain);
            let cid = ObjId {
                who: 0,
                o: script::CAPITAL,
            };
            let b = scene.buildings.iter().find(|b| b.id == cid).unwrap();
            let mut input = Input::new(0);
            input.selection = vec![cid];
            input.building = true;
            let train = crate::hud::menu(&input, HUD_SIZE)
                .into_iter()
                .find(|m| m.ty == script::CITIZEN)
                .map(|m| m.rect.centre());
            let infinite =
                crate::hud::infinite_button(&input, &scene, HUD_SIZE).map(|r| r.centre());
            println!(
                "BUILDMASK STAGE0 f{} capital pixel {:?} active {} trains {} queued {} train {train:?} infinite {infinite:?}",
                r.frame(),
                v.pixel(b.pos.x, b.pos.y, b.z),
                b.active,
                b.trains,
                b.queued,
            );
            assert_eq!(v.pixel(b.pos.x, b.pos.y, b.z), BUILDMASK_CAPITAL_AT);
            assert!(b.active && b.trains && b.queued == 0);
            assert_eq!(train, Some(BUILDMASK_TRAIN_AT));
            assert_eq!(infinite, Some(INFINITE_AT));
        }
        drop(r);

        let Some((ours, r)) = mint_with(buildmask_events(), |input, frame, ev, view| {
            crate::hud::event(input, HUD_SIZE, frame, ev, view)
        }) else {
            return;
        };
        let buildmasked = script::buildmasked();
        let whole = ours
            .iter()
            .zip(&buildmasked)
            .filter(|(a, b)| a == b)
            .count();
        assert_eq!(
            (ours.len(), whole),
            (buildmasked.len(), INPUT_BUILDMASKED),
            "{ours:?}"
        );
        assert_eq!(ours, buildmasked);
        drop(r);
        let home = std::env::var_os("HOME").unwrap_or_default();
        let (Ok(garrisoned), Ok(file)) = (
            std::fs::read(std::path::Path::new(&home).join(GARRISONED_RCX)),
            std::fs::read(std::path::Path::new(&home).join(BUILDMASKED_RCX)),
        ) else {
            println!("skipping the bytes: no {GARRISONED_RCX} or {BUILDMASKED_RCX}");
            return;
        };
        assert_eq!(file.len(), BUILDMASKED_RCX_BYTES);
        let written = written(&ours).unwrap();
        assert!(written == file, "the file from the buildmasked orders");
        assert!(written != garrisoned, "not the garrisoned file");
    }

    /// The buildmasked file (`docs/CLIENT_INPUT.md` §24), written by
    /// rondata's `the_buildmasked_minted_packages_round_trip` under
    /// `RON_RCX_OUT`; the file run1614 takes open.
    const BUILDMASKED_RCX: &str = "ron-golden/recordings/item4276-buildmasked-greatlakes-24000.rcx";
    const BUILDMASKED_RCX_BYTES: usize = 59_637;

    /// **`INPUT_BUILDMASKS`** (item 4276): of the buildmasks the recordings
    /// on disk carry, those the infinite-queue button over the package's
    /// selection mints field for field, the package's orders whole, from
    /// the package's selection (an empty `group` the slot's last full one,
    /// `add_group`'s repeat). Seen red pinned (0, 0).
    const INPUT_BUILDMASKS: (usize, usize) = (5, 5);

    /// **Every recorded buildmask** (item 4276): each package carrying a
    /// `buildmask` is minted from its selection of buildings and a press
    /// on the infinite-queue button.
    #[test]
    fn minted_buildmasks_equal_the_recordings() {
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
            let mut last = LastSent::new();
            for p in &rec.packages {
                let Ok(cmds) = rondata::commands::decode(&p.data) else {
                    continue;
                };
                let kept: Vec<Command> =
                    cmds.into_iter().filter(rondata::input::is_input).collect();
                let (who, objects, before) = behind(&last, p.play, &kept);
                note_sent(&mut last, p.play, &kept);
                if !kept.iter().any(|c| matches!(c, Command::Buildmask { .. })) {
                    continue;
                }
                all += 1;
                let scene = Scene {
                    buildings: objects
                        .iter()
                        .map(|&o| {
                            let mut b = city_building(who, o);
                            b.trains = true;
                            b
                        })
                        .collect(),
                    ..Scene::default()
                };
                let v = view(&c, &scene, &terrain);
                let mut input = Input::new(who);
                input.selection = objects.iter().map(|&o| ObjId { who, o }).collect();
                input.building = true;
                input.sent = sent_for(&before, &scene);
                let got = input.event(p.frame, Event::Infinite, &v);
                if got.as_ref().map(|o| &o.2) == Some(&kept) {
                    whole += 1;
                } else {
                    println!("BUILDMASK PARTS f{} {kept:?}: {got:?}", p.frame);
                }
            }
        }
        println!("BUILDMASKS {whole} of {all}");
        assert_eq!((whole, all), INPUT_BUILDMASKS);
    }

    /// **`INPUT_COME_OUTS`** (item 4280, `docs/CLIENT_INPUT.md` §26): of
    /// the come-outs the recordings on disk carry, those a press on the
    /// unit's icon mints field for field, the package's orders whole. The
    /// command names no building, so each is minted from a selection of
    /// one building of the slot's own holding the unit (ours). Seen red
    /// pinned (0, 0).
    const INPUT_COME_OUTS: (usize, usize) = (4, 4);

    /// **Every recorded come-out** (item 4280): each package carrying a
    /// `come_out` is minted from a building holding its unit and a press
    /// on that unit's icon.
    #[test]
    fn minted_come_outs_equal_the_recordings() {
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
                let Some(&Command::ComeOut { who, o, uid }) =
                    kept.iter().find(|c| matches!(c, Command::ComeOut { .. }))
                else {
                    continue;
                };
                all += 1;
                let who = who as u8;
                let scene = Scene {
                    buildings: vec![city_building(who, 2000)],
                    garrisons: vec![sim::observe::GarrisonScene {
                        building: 2000,
                        owner: who,
                        o: o as i16,
                        uid: uid as u16,
                        type_index: 50,
                    }],
                    ..Scene::default()
                };
                let v = view(&c, &scene, &terrain);
                let mut input = Input::new(who);
                input.selection = vec![ObjId { who, o: 2000 }];
                input.building = true;
                let got = input.event(p.frame, Event::ComeOut(0), &v);
                if got.as_ref().map(|o| &o.2) == Some(&kept) {
                    whole += 1;
                } else {
                    println!("COME_OUT PARTS f{} {kept:?}: {got:?}", p.frame);
                }
            }
        }
        println!("COME_OUTS {whole} of {all}");
        assert_eq!((whole, all), INPUT_COME_OUTS);
    }

    /// **The come-out's edges** (item 4280): the icons only for a
    /// selection of buildings of the slot's own, one per unit of its own
    /// inside; nothing minted for a unit selection, an icon past the last,
    /// another slot's building, or with the entry line open; the order
    /// carries no group.
    #[test]
    fn minted_come_out_edges() {
        if lobby().is_err() {
            println!("skipping: no lobby");
            return;
        }
        let terrain = TerrainScene::default();
        let c = crate::pinned_camera();
        let inside = |building: i16, owner: u8, o: i16| sim::observe::GarrisonScene {
            building,
            owner,
            o,
            uid: 7,
            type_index: 50,
        };
        let scene = Scene {
            buildings: vec![city_building(0, 2000), city_building(1, 2001)],
            garrisons: vec![inside(2000, 0, 3), inside(2000, 0, 4), inside(2001, 1, 5)],
            ..Scene::default()
        };
        let v = view(&c, &scene, &terrain);
        let sel = |who: u8, o: i16, building: bool| {
            let mut input = Input::new(0);
            input.selection = vec![ObjId { who, o }];
            input.building = building;
            input
        };
        let menu = |input: &Input| crate::hud::come_out_menu(input, &scene, HUD_SIZE).len();
        assert_eq!(menu(&sel(0, 2000, true)), 2, "two of its own inside");
        assert_eq!(menu(&sel(1, 2001, true)), 0, "another slot's building");
        assert_eq!(menu(&sel(0, 3, false)), 0, "a unit selection");
        let got = sel(0, 2000, true).event(9, Event::ComeOut(1), &v).unwrap();
        assert_eq!(
            got.2,
            vec![Command::ComeOut {
                who: 0,
                o: 4,
                uid: 7
            }],
            "the second icon, no group"
        );
        assert_eq!(sel(0, 2000, true).event(9, Event::ComeOut(2), &v), None);
        assert_eq!(sel(1, 2001, true).event(9, Event::ComeOut(0), &v), None);
        assert_eq!(sel(0, 3, false).event(9, Event::ComeOut(0), &v), None);
        let mut typing = sel(0, 2000, true);
        typing.line = Some(String::new());
        assert_eq!(typing.event(9, Event::ComeOut(0), &v), None);
    }

    /// **The buildmask's edges** (item 4276): the button only for a
    /// selection of buildings with a training building of the slot's own;
    /// the event mints nothing for a unit selection, for no selection or
    /// with the entry line open; any selection of buildings mints, whether
    /// or not a member takes the bit, which is `process_buildmask`'s; the
    /// same selection again sends its `group` as no objects.
    #[test]
    fn minted_buildmask_edges() {
        if lobby().is_err() {
            println!("skipping: no lobby");
            return;
        }
        let terrain = TerrainScene::default();
        let c = crate::pinned_camera();
        let trains = |o: i16, t: bool| {
            let mut b = city_building(0, o);
            b.trains = t;
            b
        };
        let scene = Scene {
            buildings: vec![trains(2000, true), trains(2001, false), trains(2002, true)],
            units: vec![unit(0, 3, script::CITIZEN, 0, 0, 0)],
            ..Scene::default()
        };
        let v = view(&c, &scene, &terrain);
        let sel = |ids: &[i16], building: bool| {
            let mut input = Input::new(0);
            input.selection = ids.iter().map(|&o| ObjId { who: 0, o }).collect();
            input.building = building;
            input
        };
        let button = |input: &Input| crate::hud::infinite_button(input, &scene, HUD_SIZE);
        assert!(button(&sel(&[2000], true)).is_some());
        assert!(
            button(&sel(&[2001], true)).is_none(),
            "no training building"
        );
        assert!(
            button(&sel(&[2001, 2002], true)).is_some(),
            "one of them trains"
        );
        assert!(button(&sel(&[3], false)).is_none(), "a unit selection");
        let mut other = sel(&[2000], true);
        other.player = 1;
        assert!(button(&other).is_none(), "another slot's");
        assert_eq!(sel(&[3], false).event(9, Event::Infinite, &v), None);
        assert_eq!(Input::new(0).event(9, Event::Infinite, &v), None);
        let mut typing = sel(&[2000], true);
        typing.line = Some(String::new());
        assert_eq!(typing.event(9, Event::Infinite, &v), None);
        let buildmask = Command::Buildmask {
            mask: INFINITE_MASK,
            set: 1,
        };
        let mut input = sel(&[2001], true);
        let got = input.event(9, Event::Infinite, &v).unwrap();
        assert_eq!(
            got.2,
            vec![
                Command::Group {
                    who: 0,
                    objects: vec![2001]
                },
                buildmask.clone()
            ],
            "a building that takes no bit still sends"
        );
        let again = input.event(10, Event::Infinite, &v).unwrap();
        assert_eq!(
            again.2,
            vec![
                Command::Group {
                    who: 0,
                    objects: Vec::new()
                },
                buildmask
            ],
            "the repeat"
        );
    }

    /// **The come-out's pixels** (item 4280, `docs/CLIENT_INPUT.md` §26),
    /// stage 0 on [`script::COME_OUT_FRAME`] at the opening camera: the
    /// capital, and the icon of [`script::GARRISON_CITIZEN`] inside it
    /// ([`crate::hud::come_out_menu`]).
    const COME_OUT_CAPITAL_AT: (i32, i32) = (1127, 521);
    const COME_OUT_ICON_AT: (i32, i32) = (1822, 922);

    /// The buildmasked events, then a press on the capital and on the
    /// Citizen's icon on [`script::COME_OUT_FRAME`].
    fn come_out_events() -> Vec<Scripted> {
        let mut e = buildmask_events();
        let ((cx, cy), (ix, iy)) = (COME_OUT_CAPITAL_AT, COME_OUT_ICON_AT);
        let f = script::COME_OUT_FRAME;
        e.extend([
            (f, None, Event::Press { x: cx, y: cy }),
            (f, None, Event::Press { x: ix, y: iy }),
        ]);
        e
    }

    /// **`INPUT_CAME_OUT`** (item 4280): the come-out's events, through the
    /// HUD and the layer under the opening camera, mint
    /// [`script::came_out`] field for field, of 28.
    const INPUT_CAME_OUT: usize = 28;

    /// **The came-out count**: stage 0 on the come-out's frame — the
    /// capital is slot 0's and holds the Citizen, whose `o` and `uid` are
    /// the script's, and the press and the icon stand where the events
    /// press; then the events mint the came-out script, and the file
    /// written from them is the came-out file.
    #[test]
    fn minted_came_out_at_the_opening_camera() {
        let Ok(mut r) = lobby() else {
            println!("skipping: no lobby");
            return;
        };
        let buildmasked = script::buildmasked();
        let c = camera(None);
        let mut next = 0;
        while r.frame() < i64::from(script::COME_OUT_FRAME) {
            let f = r.frame() as i32;
            while next < buildmasked.len() && buildmasked[next].0 == f {
                r.issue(&buildmasked[next]);
                next += 1;
            }
            r.step();
        }
        {
            let (scene, terrain) = (r.scene(Some(0)), r.terrain());
            let v = view(&c, &scene, &terrain);
            let cid = ObjId {
                who: 0,
                o: script::CAPITAL,
            };
            let b = scene.buildings.iter().find(|b| b.id == cid).unwrap();
            let mut input = Input::new(0);
            input.selection = vec![cid];
            input.building = true;
            let inside = garrisoned(&scene, 0, &input.selection);
            let icons = crate::hud::come_out_menu(&input, &scene, HUD_SIZE);
            println!(
                "COME_OUT STAGE0 f{} capital pixel {:?} inside {inside:?} icons {:?}",
                r.frame(),
                v.pixel(b.pos.x, b.pos.y, b.z),
                icons
                    .iter()
                    .map(|(k, r)| (*k, r.centre()))
                    .collect::<Vec<_>>(),
            );
            assert_eq!(v.pixel(b.pos.x, b.pos.y, b.z), COME_OUT_CAPITAL_AT);
            let k = inside
                .iter()
                .position(|g| g.o == script::GARRISON_CITIZEN)
                .unwrap();
            assert_eq!(inside[k].uid as i16, script::COME_OUT_UID);
            assert_eq!(icons[k].1.centre(), COME_OUT_ICON_AT);
        }
        drop(r);

        let Some((ours, r)) = mint_with(come_out_events(), |input, frame, ev, view| {
            crate::hud::event(input, HUD_SIZE, frame, ev, view)
        }) else {
            return;
        };
        let came_out = script::came_out();
        let whole = ours.iter().zip(&came_out).filter(|(a, b)| a == b).count();
        assert_eq!(
            (ours.len(), whole),
            (came_out.len(), INPUT_CAME_OUT),
            "{ours:?}"
        );
        assert_eq!(ours, came_out);
        drop(r);
        let home = std::env::var_os("HOME").unwrap_or_default();
        let (Ok(buildmasked), Ok(file)) = (
            std::fs::read(std::path::Path::new(&home).join(BUILDMASKED_RCX)),
            std::fs::read(std::path::Path::new(&home).join(CAME_OUT_RCX)),
        ) else {
            println!("skipping the bytes: no {BUILDMASKED_RCX} or {CAME_OUT_RCX}");
            return;
        };
        assert_eq!(file.len(), CAME_OUT_RCX_BYTES);
        let written = written(&ours).unwrap();
        assert!(written == file, "the file from the came-out orders");
        assert!(written != buildmasked, "not the buildmasked file");
    }

    /// The came-out file (`docs/CLIENT_INPUT.md` §26), written by
    /// rondata's `the_came_out_minted_packages_round_trip` under
    /// `RON_RCX_OUT`.
    const CAME_OUT_RCX: &str = "ron-golden/recordings/item4280-came-out-greatlakes-24000.rcx";
    const CAME_OUT_RCX_BYTES: usize = 59_649;

    /// **`INPUT_STANCES`** (item 4282, `docs/CLIENT_INPUT.md` §27): of the
    /// stances the recordings on disk carry, those a press on the stance
    /// button mints field for field, the package's orders whole, from the
    /// package's selection (an empty `group` the slot's last full one).
    /// Seen red pinned (0, 0).
    const INPUT_STANCES: (usize, usize) = (4, 4);

    /// **Every recorded stance** (item 4282): each package carrying a
    /// `stance` is minted from its selection and a press on the stance
    /// button.
    #[test]
    fn minted_stances_equal_the_recordings() {
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
            let mut last = LastSent::new();
            for p in &rec.packages {
                let Ok(cmds) = rondata::commands::decode(&p.data) else {
                    continue;
                };
                let kept: Vec<Command> =
                    cmds.into_iter().filter(rondata::input::is_input).collect();
                let (who, objects, before) = behind(&last, p.play, &kept);
                note_sent(&mut last, p.play, &kept);
                if !kept.iter().any(|c| matches!(c, Command::Stance { .. })) {
                    continue;
                }
                all += 1;
                let scene = Scene {
                    buildings: objects
                        .iter()
                        .map(|&o| {
                            let mut b = city_building(who, o);
                            b.trains = true;
                            b
                        })
                        .collect(),
                    ..Scene::default()
                };
                let v = view(&c, &scene, &terrain);
                let mut input = Input::new(who);
                input.selection = objects.iter().map(|&o| ObjId { who, o }).collect();
                input.building = objects.iter().all(|&o| o >= 2000);
                input.sent = sent_for(&before, &scene);
                let got = crate::hud::stance_button(&input, &scene, HUD_SIZE).and_then(|r| {
                    let (x, y) = r.centre();
                    crate::hud::event(&mut input, HUD_SIZE, p.frame, Event::Press { x, y }, &v)
                });
                if got.as_ref().map(|o| &o.2) == Some(&kept) {
                    whole += 1;
                } else {
                    println!("STANCE PARTS f{} {kept:?}: {got:?}", p.frame);
                }
            }
        }
        println!("STANCES {whole} of {all}");
        assert_eq!((whole, all), INPUT_STANCES);
    }

    /// **The stance button's edges** (item 4282): the button for a
    /// selection of the slot's own units, or of buildings one of which is
    /// a training building of its own; none for another slot's, a building
    /// that trains nothing, no selection, or an open entry line; the order
    /// is the selection's `group` and `stance(-1)`, and the same selection
    /// again sends its `group` as no objects.
    #[test]
    fn minted_stance_edges() {
        if lobby().is_err() {
            println!("skipping: no lobby");
            return;
        }
        let terrain = TerrainScene::default();
        let c = crate::pinned_camera();
        let mut farm = city_building(0, 2001);
        farm.trains = false;
        let mut theirs = city_building(1, 2000);
        theirs.trains = true;
        let mut capital = city_building(0, 2000);
        capital.trains = true;
        let scene = Scene {
            buildings: vec![capital, farm, theirs],
            ..Scene::default()
        };
        let v = view(&c, &scene, &terrain);
        let sel = |ids: &[(u8, i16)], building: bool| {
            let mut input = Input::new(0);
            input.selection = ids.iter().map(|&(who, o)| ObjId { who, o }).collect();
            input.building = building;
            input
        };
        let button = |input: &Input| crate::hud::stance_button(input, &scene, HUD_SIZE);
        assert!(button(&sel(&[(0, 2000)], true)).is_some(), "the capital");
        assert!(
            button(&sel(&[(0, 2001), (0, 2000)], true)).is_some(),
            "a mixed pick"
        );
        assert!(button(&sel(&[(0, 2001)], true)).is_none(), "a farm");
        assert!(button(&sel(&[(1, 2000)], true)).is_none(), "another slot's");
        assert!(
            button(&sel(&[(0, 7)], false)).is_some(),
            "a unit of its own"
        );
        assert!(
            button(&sel(&[(1, 7)], false)).is_none(),
            "another slot's unit"
        );
        assert!(button(&Input::new(0)).is_none(), "no selection");
        let mut typing = sel(&[(0, 2000)], true);
        typing.line = Some(String::new());
        assert!(button(&typing).is_none(), "the entry line open");
        assert_eq!(typing.event(9, Event::Stance, &v), None);
        assert_eq!(Input::new(0).event(9, Event::Stance, &v), None);
        let stance = Command::Stance {
            stance: STANCE_CYCLE,
        };
        let mut input = sel(&[(0, 2000)], true);
        let got = input.event(9, Event::Stance, &v).unwrap();
        assert_eq!(
            got.2,
            vec![
                Command::Group {
                    who: 0,
                    objects: vec![2000]
                },
                stance.clone()
            ]
        );
        let again = input.event(10, Event::Stance, &v).unwrap();
        assert_eq!(
            again.2,
            vec![
                Command::Group {
                    who: 0,
                    objects: vec![]
                },
                stance
            ]
        );
    }

    /// Where the stance button stands with the capital selected, under
    /// the opening camera (`minted_stanced_at_the_opening_camera`).
    const STANCE_AT: (i32, i32) = (1446, 966);

    /// The stanced script's events: the come-out's, then a press on the
    /// stance button on [`script::STANCE_FRAME`], the capital still
    /// selected.
    fn stance_events() -> Vec<Scripted> {
        let mut e = come_out_events();
        let (x, y) = STANCE_AT;
        e.push((script::STANCE_FRAME, None, Event::Press { x, y }));
        e
    }

    /// **`INPUT_STANCED`** (item 4282): the stanced script's events,
    /// through the HUD and the layer under the opening camera, mint
    /// [`script::stanced`] field for field, of 29.
    const INPUT_STANCED: usize = 29;

    /// **The stanced count**: stage 0 on the press's frame — the capital is
    /// slot 0's, a finished training building, and the stance button
    /// stands where the event presses; then the events mint the stanced
    /// script, and the file written from them is the stanced file.
    #[test]
    fn minted_stanced_at_the_opening_camera() {
        let Ok(mut r) = lobby() else {
            println!("skipping: no lobby");
            return;
        };
        let came_out = script::came_out();
        let c = camera(None);
        let mut next = 0;
        while r.frame() < i64::from(script::STANCE_FRAME) {
            let f = r.frame() as i32;
            while next < came_out.len() && came_out[next].0 == f {
                r.issue(&came_out[next]);
                next += 1;
            }
            r.step();
        }
        {
            let (scene, terrain) = (r.scene(Some(0)), r.terrain());
            let v = view(&c, &scene, &terrain);
            let cid = ObjId {
                who: 0,
                o: script::CAPITAL,
            };
            let b = scene.buildings.iter().find(|b| b.id == cid).unwrap();
            let mut input = Input::new(0);
            input.selection = vec![cid];
            input.building = true;
            let at = crate::hud::stance_button(&input, &scene, HUD_SIZE).map(|r| r.centre());
            println!(
                "STANCE STAGE0 f{} capital pixel {:?} trains {} active {} button {at:?}",
                r.frame(),
                v.pixel(b.pos.x, b.pos.y, b.z),
                b.trains,
                b.active,
            );
            assert_eq!(v.pixel(b.pos.x, b.pos.y, b.z), COME_OUT_CAPITAL_AT);
            assert!(b.trains && b.active);
            assert_eq!(at, Some(STANCE_AT));
        }
        drop(r);

        let Some((ours, r)) = mint_with(stance_events(), |input, frame, ev, view| {
            crate::hud::event(input, HUD_SIZE, frame, ev, view)
        }) else {
            return;
        };
        let stanced = script::stanced();
        let whole = ours.iter().zip(&stanced).filter(|(a, b)| a == b).count();
        assert_eq!(
            (ours.len(), whole),
            (stanced.len(), INPUT_STANCED),
            "{ours:?}"
        );
        assert_eq!(ours, stanced);
        drop(r);
        let home = std::env::var_os("HOME").unwrap_or_default();
        let (Ok(came_out), Ok(file)) = (
            std::fs::read(std::path::Path::new(&home).join(CAME_OUT_RCX)),
            std::fs::read(std::path::Path::new(&home).join(STANCED_RCX)),
        ) else {
            println!("skipping the bytes: no {CAME_OUT_RCX} or {STANCED_RCX}");
            return;
        };
        assert_eq!(file.len(), STANCED_RCX_BYTES);
        let written = written(&ours).unwrap();
        assert!(written == file, "the file from the stanced orders");
        assert!(written != came_out, "not the came-out file");
    }

    /// The stanced file (`docs/CLIENT_INPUT.md` §27), written by rondata's
    /// `the_stanced_minted_packages_round_trip` under `RON_RCX_OUT`.
    const STANCED_RCX: &str = "ron-golden/recordings/item4282-stanced-greatlakes-24000.rcx";
    const STANCED_RCX_BYTES: usize = 59_661;

    /// The console's line typed on `frame`: the key, each character, and
    /// Return (item 4283).
    fn consoled(frame: i32, line: &str) -> Vec<Scripted> {
        let mut e = vec![(frame, None, Event::Console)];
        e.extend(line.chars().map(|c| (frame, None, Event::Key(c))));
        e.push((frame, None, Event::Enter));
        e
    }

    /// **`INPUT_TOGGLES`** (item 4283, `docs/CLIENT_INPUT.md` §28): of the
    /// AI toggles the recordings on disk carry, those the console's
    /// `ai off`, typed by the package's slot with the AI on, mints field
    /// for field, the package's orders whole. Seen red with the scene's
    /// `ai_off` set: (0, 4).
    const INPUT_TOGGLES: (usize, usize) = (4, 4);

    /// **Every recorded AI toggle** (item 4283): each package carrying a
    /// `cheat_ai_toggle` is minted from the console's line. Every one is a
    /// capture driver's `ai off` early in a game whose AI was on.
    #[test]
    fn minted_toggles_equal_the_recordings() {
        let files = recordings_on_disk();
        if files.is_empty() || lobby().is_err() {
            println!("skipping: no .rcx on disk or no lobby");
            return;
        }
        let terrain = TerrainScene::default();
        let c = crate::pinned_camera();
        let scene = Scene::default();
        let v = view(&c, &scene, &terrain);
        let (mut whole, mut all) = (0, 0);
        for f in &files {
            let rec = rondata::recgame::read(&f.display().to_string()).unwrap();
            for p in &rec.packages {
                let Ok(cmds) = rondata::commands::decode(&p.data) else {
                    continue;
                };
                let kept: Vec<Command> =
                    cmds.into_iter().filter(rondata::input::is_input).collect();
                if !kept.iter().any(|c| matches!(c, Command::CheatAiToggle)) {
                    continue;
                }
                all += 1;
                let mut input = Input::new(p.play as u8);
                let mut got = None;
                for (_, _, ev) in consoled(p.frame, script::TOGGLE_TEXT) {
                    got = got.or(input.event(p.frame, ev, &v));
                }
                if got.as_ref().map(|o| &o.2) == Some(&kept) {
                    whole += 1;
                } else {
                    println!("TOGGLE PARTS f{} {kept:?}: {got:?}", p.frame);
                }
            }
        }
        println!("TOGGLES {whole} of {all}");
        assert_eq!((whole, all), INPUT_TOGGLES);
    }

    /// **The console's edges** (item 4283): `ai` alone toggles either way;
    /// `ai off` only with the AI on and `ai on` only with it off, without
    /// case; `ai debug`, an unknown argument and every other word mint
    /// nothing; Escape closes the line and sends nothing; the key inside
    /// an open line opens nothing; and a line opened by Return is still
    /// chat's.
    #[test]
    fn minted_console_edges() {
        let terrain = TerrainScene::default();
        let c = crate::pinned_camera();
        let on = Scene::default();
        let off = Scene {
            ai_off: true,
            ..Scene::default()
        };
        let send = |scene: &Scene, line: &str| {
            let v = view(&c, scene, &terrain);
            let mut input = Input::new(0);
            let mut got = None;
            for (_, _, ev) in consoled(9, line) {
                got = got.or(input.event(9, ev, &v));
            }
            assert!(input.line.is_none() && !input.console, "{line}: closed");
            got.map(|o| o.2)
        };
        let toggle = Some(vec![Command::CheatAiToggle]);
        assert_eq!(send(&on, "ai"), toggle);
        assert_eq!(send(&off, "ai"), toggle);
        assert_eq!(send(&on, "ai off"), toggle);
        assert_eq!(send(&off, "ai off"), None);
        assert_eq!(send(&off, "ai on"), toggle);
        assert_eq!(send(&on, "ai on"), None);
        assert_eq!(send(&on, "AI  OFF"), toggle);
        assert_eq!(send(&on, "ai debug"), None);
        assert_eq!(send(&on, "ai maybe"), None);
        assert_eq!(send(&on, "human 1"), None);
        assert_eq!(send(&on, ""), None);
        let v = view(&c, &on, &terrain);
        let mut input = Input::new(0);
        for ev in [Event::Console, Event::Key('a'), Event::Key('i')] {
            assert_eq!(input.event(9, ev, &v), None);
        }
        assert_eq!(input.event(9, Event::Console, &v), None, "no second line");
        assert_eq!(input.line.as_deref(), Some("ai"));
        assert_eq!(input.event(9, Event::Escape, &v), None);
        assert!(input.line.is_none() && !input.console, "Escape closes it");
        let mut chat = Input::new(0);
        for ev in [Event::Enter, Event::Key('a'), Event::Key('i')] {
            assert_eq!(chat.event(9, ev, &v), None);
        }
        let said = chat.event(9, Event::Enter, &v).unwrap();
        assert!(
            matches!(said.2[0], Command::Chat { .. }),
            "Return's line is chat's"
        );
    }

    /// The toggled script's events: the stance's, then the console's
    /// `ai off` on [`script::TOGGLE_FRAME`].
    fn toggle_events() -> Vec<Scripted> {
        let mut e = stance_events();
        e.extend(consoled(script::TOGGLE_FRAME, script::TOGGLE_TEXT));
        e
    }

    /// **`INPUT_TOGGLED`** (item 4283): the toggled script's events,
    /// through the HUD and the layer under the opening camera, mint
    /// [`script::toggled`] field for field, of 30.
    const INPUT_TOGGLED: usize = 30;

    /// **The toggled count**: stage 0 on the toggle's frame — the scene
    /// reads the AI on, which is what lets `ai off` issue — then the events
    /// mint the toggled script, and the file written from them is the
    /// toggled file.
    #[test]
    fn minted_toggled_at_the_opening_camera() {
        let Ok(mut r) = lobby() else {
            println!("skipping: no lobby");
            return;
        };
        let stanced = script::stanced();
        let mut next = 0;
        while r.frame() < i64::from(script::TOGGLE_FRAME) {
            let f = r.frame() as i32;
            while next < stanced.len() && stanced[next].0 == f {
                r.issue(&stanced[next]);
                next += 1;
            }
            r.step();
        }
        let scene = r.scene(Some(0));
        println!(
            "TOGGLE STAGE0 f{} ai_off {} leaders {}",
            r.frame(),
            scene.ai_off,
            scene.leaders.len()
        );
        assert!(!scene.ai_off, "the AI is on when the line is typed");
        drop(r);

        let Some((ours, mut r)) = mint_with(toggle_events(), |input, frame, ev, view| {
            crate::hud::event(input, HUD_SIZE, frame, ev, view)
        }) else {
            return;
        };
        r.step();
        assert!(
            r.scene(Some(0)).ai_off,
            "and off after the tick that takes it"
        );
        let toggled = script::toggled();
        let whole = ours.iter().zip(&toggled).filter(|(a, b)| a == b).count();
        assert_eq!(
            (ours.len(), whole),
            (toggled.len(), INPUT_TOGGLED),
            "{ours:?}"
        );
        assert_eq!(ours, toggled);
        drop(r);
        let home = std::env::var_os("HOME").unwrap_or_default();
        let (Ok(stanced), Ok(file)) = (
            std::fs::read(std::path::Path::new(&home).join(STANCED_RCX)),
            std::fs::read(std::path::Path::new(&home).join(TOGGLED_RCX)),
        ) else {
            println!("skipping the bytes: no {STANCED_RCX} or {TOGGLED_RCX}");
            return;
        };
        assert_eq!(file.len(), TOGGLED_RCX_BYTES);
        let written = written(&ours).unwrap();
        assert!(written == file, "the file from the toggled orders");
        assert!(written != stanced, "not the stanced file");
    }

    /// The toggled file (`docs/CLIENT_INPUT.md` §28), written by rondata's
    /// `the_toggled_minted_packages_round_trip` under `RON_RCX_OUT`.
    const TOGGLED_RCX: &str = "ron-golden/recordings/item4283-toggled-greatlakes-24000.rcx";
    const TOGGLED_RCX_BYTES: usize = 59_670;

    /// **`INPUT_DISBANDS`** (item 4287, `docs/CLIENT_INPUT.md` §29): of the
    /// disbands the recordings on disk carry, those a press on the DISBAND
    /// button mints field for field, the package's orders whole. Seen red
    /// pinned (0, 0).
    const INPUT_DISBANDS: (usize, usize) = (1, 1);

    /// **Every recorded disband** (item 4287): each package carrying a
    /// `disband` is minted from a fresh layer whose selection is the
    /// package's group (or for an empty group the slot's last full one),
    /// by a press on the DISBAND button.
    #[test]
    fn minted_disbands_equal_the_recordings() {
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
            let mut last = LastSent::new();
            for p in &rec.packages {
                let Ok(cmds) = rondata::commands::decode(&p.data) else {
                    continue;
                };
                let kept: Vec<Command> =
                    cmds.into_iter().filter(rondata::input::is_input).collect();
                let (who, objects, before) = behind(&last, p.play, &kept);
                note_sent(&mut last, p.play, &kept);
                if !kept.iter().any(|c| matches!(c, Command::Disband { .. })) {
                    continue;
                }
                all += 1;
                let building = objects.iter().all(|&o| o >= 2000);
                let scene = Scene {
                    buildings: objects
                        .iter()
                        .filter(|&&o| o >= 2000)
                        .map(|&o| city_building(who, o))
                        .collect(),
                    units: objects
                        .iter()
                        .filter(|&&o| o < 2000)
                        .map(|&o| garrison_unit(who, o, &[]))
                        .collect(),
                    ..Scene::default()
                };
                let v = view(&c, &scene, &terrain);
                let mut input = Input::new(who);
                input.selection = objects.iter().map(|&o| ObjId { who, o }).collect();
                input.building = building;
                input.sent = sent_for(&before, &scene);
                let got = crate::hud::disband_button(&input, &scene, HUD_SIZE).and_then(|r| {
                    let (x, y) = r.centre();
                    crate::hud::event(&mut input, HUD_SIZE, p.frame, Event::Press { x, y }, &v)
                });
                if got.as_ref().map(|o| &o.2) == Some(&kept) {
                    whole += 1;
                } else {
                    println!("DISBAND PARTS f{} {kept:?}: {got:?}", p.frame);
                }
            }
        }
        println!("DISBANDS {whole} of {all}");
        assert_eq!((whole, all), INPUT_DISBANDS);
    }

    /// **The disband button's edges** (item 4287): it shows for a unit or a
    /// building of the slot's own, and for a mixed pick holding one; not
    /// for another slot's, no selection, or the entry line open; the order
    /// is the selection's `group` and `disband(0)`, and the same selection
    /// again sends its `group` as no objects.
    #[test]
    fn minted_disband_edges() {
        if lobby().is_err() {
            println!("skipping: no lobby");
            return;
        }
        let terrain = TerrainScene::default();
        let c = crate::pinned_camera();
        let scene = Scene {
            buildings: vec![city_building(0, 2001), city_building(1, 2000)],
            units: vec![garrison_unit(0, 2, &[]), garrison_unit(1, 7, &[])],
            ..Scene::default()
        };
        let v = view(&c, &scene, &terrain);
        let sel = |ids: &[(u8, i16)], building: bool| {
            let mut input = Input::new(0);
            input.selection = ids.iter().map(|&(who, o)| ObjId { who, o }).collect();
            input.building = building;
            input
        };
        let button = |input: &Input| crate::hud::disband_button(input, &scene, HUD_SIZE);
        assert!(
            button(&sel(&[(0, 2)], false)).is_some(),
            "a unit of its own"
        );
        assert!(button(&sel(&[(0, 2001)], true)).is_some(), "a building");
        assert!(
            button(&sel(&[(1, 7), (0, 2)], false)).is_some(),
            "a mixed pick"
        );
        assert!(
            button(&sel(&[(1, 7)], false)).is_none(),
            "another slot's unit"
        );
        assert!(
            button(&sel(&[(1, 2000)], true)).is_none(),
            "another slot's building"
        );
        assert!(button(&Input::new(0)).is_none(), "no selection");
        let mut typing = sel(&[(0, 2)], false);
        typing.line = Some(String::new());
        assert!(button(&typing).is_none(), "the entry line open");
        assert_eq!(typing.event(9, Event::Disband, &v), None);
        assert_eq!(Input::new(0).event(9, Event::Disband, &v), None);
        let disband = Command::Disband { all: DISBAND_ONE };
        let mut input = sel(&[(0, 2)], false);
        let got = input.event(9, Event::Disband, &v).unwrap();
        assert_eq!(
            got.2,
            vec![
                Command::Group {
                    who: 0,
                    objects: vec![2]
                },
                disband.clone()
            ]
        );
        let again = input.event(10, Event::Disband, &v).unwrap();
        assert_eq!(
            again.2,
            vec![
                Command::Group {
                    who: 0,
                    objects: vec![]
                },
                disband
            ]
        );
    }

    /// Where the came-out Citizen stands on [`script::DISBAND_FRAME`], and
    /// the DISBAND button with it selected, under the opening camera
    /// (`minted_disbanded_at_the_opening_camera`).
    const DISBAND_CITIZEN_AT: (i32, i32) = (1316, 471);
    const DISBAND_AT: (i32, i32) = (1258, 966);

    /// The disbanded script's events: the toggle's, then on
    /// [`script::DISBAND_FRAME`] a press on the Citizen and one on the
    /// DISBAND button.
    fn disband_events() -> Vec<Scripted> {
        let mut e = toggle_events();
        let (ux, uy) = DISBAND_CITIZEN_AT;
        let (bx, by) = DISBAND_AT;
        e.extend([
            (script::DISBAND_FRAME, None, Event::Press { x: ux, y: uy }),
            (script::DISBAND_FRAME, None, Event::Press { x: bx, y: by }),
        ]);
        e
    }

    /// **`INPUT_DISBANDED`** (item 4287): the disbanded script's events,
    /// through the HUD and the layer under the opening camera, mint
    /// [`script::disbanded`] field for field, of 31.
    const INPUT_DISBANDED: usize = 31;

    /// **The disbanded count**: stage 0 on the disband's frame — the
    /// Citizen is slot 0's and alive, where the press lands, and the
    /// DISBAND button stands where the second press lands — then the
    /// events mint the disbanded script, and the file written from them is
    /// the disbanded file.
    #[test]
    fn minted_disbanded_at_the_opening_camera() {
        let Ok(mut r) = lobby() else {
            println!("skipping: no lobby");
            return;
        };
        let toggled = script::toggled();
        let c = camera(None);
        let mut next = 0;
        while r.frame() < i64::from(script::DISBAND_FRAME) {
            let f = r.frame() as i32;
            while next < toggled.len() && toggled[next].0 == f {
                r.issue(&toggled[next]);
                next += 1;
            }
            r.step();
        }
        {
            let (scene, terrain) = (r.scene(Some(0)), r.terrain());
            let v = view(&c, &scene, &terrain);
            let uid = ObjId {
                who: 0,
                o: script::DISBAND_UNIT,
            };
            let u = scene.units.iter().find(|u| u.id == uid).unwrap();
            let mut input = Input::new(0);
            input.selection = vec![uid];
            let at = crate::hud::disband_button(&input, &scene, HUD_SIZE).map(|r| r.centre());
            println!(
                "DISBAND STAGE0 f{} citizen pixel {:?} button {at:?}",
                r.frame(),
                v.pixel(u.pos.x, u.pos.y, u.z),
            );
            assert_eq!(v.pixel(u.pos.x, u.pos.y, u.z), DISBAND_CITIZEN_AT);
            assert_eq!(at, Some(DISBAND_AT));
        }
        drop(r);

        let Some((ours, mut r)) = mint_with(disband_events(), |input, frame, ev, view| {
            crate::hud::event(input, HUD_SIZE, frame, ev, view)
        }) else {
            return;
        };
        r.step();
        let gone = !r.scene(Some(0)).units.iter().any(|u| {
            u.id == ObjId {
                who: 0,
                o: script::DISBAND_UNIT,
            }
        });
        println!("DISBAND after the tick: the Citizen gone {gone}");
        let disbanded = script::disbanded();
        let whole = ours.iter().zip(&disbanded).filter(|(a, b)| a == b).count();
        assert_eq!(
            (ours.len(), whole),
            (disbanded.len(), INPUT_DISBANDED),
            "{ours:?}"
        );
        assert_eq!(ours, disbanded);
        drop(r);
        let home = std::env::var_os("HOME").unwrap_or_default();
        let (Ok(toggled), Ok(file)) = (
            std::fs::read(std::path::Path::new(&home).join(TOGGLED_RCX)),
            std::fs::read(std::path::Path::new(&home).join(DISBANDED_RCX)),
        ) else {
            println!("skipping the bytes: no {TOGGLED_RCX} or {DISBANDED_RCX}");
            return;
        };
        assert_eq!(file.len(), DISBANDED_RCX_BYTES);
        let written = written(&ours).unwrap();
        assert!(written == file, "the file from the disbanded orders");
        assert!(written != toggled, "not the toggled file");
    }

    /// The disbanded file (`docs/CLIENT_INPUT.md` §29), written by
    /// rondata's `the_disbanded_minted_packages_round_trip` under
    /// `RON_RCX_OUT`.
    const DISBANDED_RCX: &str = "ron-golden/recordings/item4287-disbanded-greatlakes-24000.rcx";
    const DISBANDED_RCX_BYTES: usize = 59_681;

    /// The queues the layer's issuers write (item 4293, §33.1): `QUEUE_FIRST`,
    /// `QUEUE_LAST` and `QUEUE_NEW`.
    const QUEUES: [i32; 3] = [QUEUE_FIRST, QUEUE_LAST, QUEUE_NEW];

    /// **Who reaches a missed arm** (item 4285, `docs/CLIENT_INPUT.md` §30).
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum Reach {
        /// A button, a key or a modifier of the original's issues it; the
        /// layer has not taken that arm.
        Button,
        /// Our file writer: the original's one package a turn holds every
        /// issue of the turn, and `recgame::minted_packages` writes one
        /// order a package on strictly increasing frames.
        Writer,
        /// No button: `leader_options` is issued from the player's profile
        /// by `Game::run` (§26.1).
        Profile,
    }

    /// **What the census says of one recorded package** (item 4285): every
    /// order of it takes the package's verdict, since the layer mints a
    /// package whole or not at all.
    #[derive(Clone, Debug, PartialEq, Eq)]
    enum Verdict {
        /// A pinned test mints every recorded package of this form field
        /// for field, the package whole; the pin is named.
        Shown(&'static str),
        /// Of a kind and a form the layer mints (its fixed fields equal the
        /// layer's, its type in the layer's keys), but no test mints the
        /// recorded one from a scene: its point, its object and its group
        /// are not checked.
        Form,
        /// Not minted: the arm, and who reaches it.
        Miss(&'static str, Reach),
    }

    /// The census's verdict on one package's kept orders. `paused` is the
    /// layer's pause bit, minted here: a `pause` is the one kind whose
    /// order needs no scene, so it is minted in place, byte for byte.
    fn census_verdict(
        kept: &[Command],
        frame: i32,
        play: u8,
        who: u8,
        pauser: &mut Input,
        v: &View,
        roster: &Roster,
    ) -> Verdict {
        use Reach::*;
        let heads: Vec<&Command> = kept
            .iter()
            .filter(|c| !matches!(c, Command::Group { .. }))
            .collect();
        if heads.is_empty() {
            return Verdict::Miss("a group with no order", Writer);
        }
        if heads.len() > 1 {
            // Since item 4302 the layer writes a turn's issues as one
            // package (§36): each issue takes its own verdict, and the turn
            // is shown when none is missed, since `INPUT_TURNS` mints every
            // recorded turn of two issues or more byte for byte.
            for issue in issues(kept) {
                if let Verdict::Miss(arm, reach) =
                    census_verdict(&issue, frame, play, who, pauser, v, roster)
                {
                    return Verdict::Miss(arm, reach);
                }
            }
            return Verdict::Shown("INPUT_TURNS");
        }
        let minted = |c: &Command| MINTS.contains(&c.name());
        match heads[0] {
            Command::LeaderOptions { .. } => {
                if leader_prefs(&kept[0], frame, play, who).is_some() {
                    Verdict::Shown("INPUT_LEADER_OPTIONS")
                } else {
                    Verdict::Miss("leader_options: a profile of neither", Profile)
                }
            }
            Command::Pause { .. } => {
                pauser.player = play;
                let got = pauser.event(frame, Event::Pause, v);
                if got.as_ref().map(|o| o.2.as_slice()) == Some(kept) {
                    Verdict::Shown("the census's own pause")
                } else {
                    Verdict::Miss("pause: the bit parts", Button)
                }
            }
            Command::Unqueue { .. } => Verdict::Shown("INPUT_UNQUEUES"),
            Command::CityGather { .. } => Verdict::Shown("INPUT_CITY_GATHERS"),
            Command::Sell { .. } | Command::Buy { .. } => Verdict::Shown("INPUT_TRADED"),
            Command::Chat { .. } => Verdict::Shown("INPUT_CHEATED"),
            Command::Hotkey { .. } => Verdict::Shown("INPUT_HOTKEYS"),
            Command::Garrison { .. } => Verdict::Shown("INPUT_GARRISONS"),
            Command::RenameCity { .. } => Verdict::Shown("INPUT_RENAMES"),
            Command::Buildmask { .. } => Verdict::Shown("INPUT_BUILDMASKS"),
            Command::ComeOut { .. } => Verdict::Shown("INPUT_COME_OUTS"),
            Command::Stance { .. } => Verdict::Shown("INPUT_STANCES"),
            Command::CheatAiToggle => Verdict::Shown("INPUT_TOGGLES"),
            Command::Spell { ox: -1, .. } => Verdict::Shown("INPUT_SPELLS"),
            Command::Spell {
                spell, x: 0, y: 0, ..
            } if SPY_CRAFTS.contains(spell) => Verdict::Shown("INPUT_SPY_SPELLS"),
            Command::Spell { .. } => Verdict::Miss("spell on an object: the right press's", Button),
            &Command::MoveTo {
                set_angle,
                angle,
                orders,
                queued,
                form,
                width,
                disembark,
                ..
            } => {
                if is_hand_move(heads[0]) {
                    // Since item 4318 the minimap's move, the attack-move
                    // and the right drag are minted from a hand-made
                    // selection (§30.3).
                    Verdict::Shown("INPUT_HAND_MOVES")
                } else if set_angle != 0 && (set_angle != 1 || !of_find_angle(angle)) {
                    // Since item 4295 the right drag's facing is the
                    // layer's (§34): `set_angle` 1 with an angle
                    // `find_angle` gives is of its form.
                    Verdict::Miss(
                        "move_to set_angle: an angle find_angle does not give",
                        Button,
                    )
                } else if !QUEUES.contains(&i32::from(queued)) {
                    Verdict::Miss("queued: a value no issuer writes", Button)
                } else if orders == ATTACK_TO {
                    // Since item 4297 the attack-move is the layer's
                    // (§35): no facing, −1/−1.
                    if (set_angle, angle, form, width, disembark) == (0, 0, -1, -1, 0) {
                        Verdict::Form
                    } else {
                        Verdict::Miss(
                            "move_to orders 2: a field ctrl's issuers do not write",
                            Button,
                        )
                    }
                } else if orders != 1 {
                    Verdict::Miss("move_to orders: an order no arm of ours issues", Button)
                } else if is_main_move(heads[0]) {
                    // Since item 4318 the main view's move is minted from
                    // a hand-made selection and a camera that reaches its
                    // point (§30.3).
                    Verdict::Shown("INPUT_MAIN_MOVES")
                } else if disembark == 0 && matches!((form, width), (0, 50) | (9, 50) | (-1, -1)) {
                    Verdict::Form
                } else {
                    Verdict::Miss("move_to: a formation the layer does not write", Button)
                }
            }
            // Since item 4318 each is minted from a hand-made selection
            // and a right press on its object (§30.3).
            c if is_hand_object_order(c) => Verdict::Shown("INPUT_HAND_OBJECT_ORDERS"),
            Command::Attack { .. }
            | Command::Guard { .. }
            | Command::Gather { .. }
            | Command::SwarmAround { .. } => {
                Verdict::Miss("queued: a value no issuer writes", Button)
            }
            // Since item 4304 the press's point beside the release's is
            // the layer's too (§38): `(x2, y2)` is the drag's press.
            &Command::Build { build, queued, .. } => {
                if !QUEUES.contains(&queued) {
                    Verdict::Miss("queued: a value no issuer writes", Button)
                } else if !BUILD_KEYS.iter().any(|k| k.1 == build)
                    && !roster.builds.contains(&build)
                {
                    Verdict::Miss("build: a type no key of ours binds", Button)
                } else {
                    // Since item 4318 each is minted from a hand-made
                    // selection and a click or a drag at a camera that
                    // reaches its points (§30.3).
                    Verdict::Shown("INPUT_HAND_PLACEMENTS")
                }
            }
            &Command::QueueUp { unit, num } => {
                if num != 1 {
                    Verdict::Miss("queue_up: a count past one", Button)
                } else if !TRAIN_KEYS.iter().any(|k| k.1 == unit)
                    && roster.trained_at(unit).is_empty()
                {
                    Verdict::Miss("queue_up: a type no key of ours binds", Button)
                } else {
                    Verdict::Shown("INPUT_ROSTER_QUEUES")
                }
            }
            // Since item 4301 a building's or one's own unit's gather point
            // is the layer's too (§11.4): actions 1 and 2 at a point, and
            // 3 naming `(captain, who)` of the issuer's slot.
            // Since item 4318 each is minted from a hand-made selection,
            // a press on the object or the ground (§30.3).
            c if is_hand_gather_point(c, who) => Verdict::Shown("INPUT_HAND_GATHER_POINTS"),
            Command::GatherPoint { add_to_end: 0, .. } => Verdict::Miss(
                "gather_point: a field the gather arm does not write",
                Button,
            ),
            Command::Disband { .. } => Verdict::Shown("INPUT_DISBANDS"),
            Command::Unitmask { .. } => Verdict::Shown("INPUT_UNITMASKS"),
            c if minted(c) => Verdict::Form,
            _ => Verdict::Miss("a kind the census does not read", Button),
        }
    }

    /// A package's issues (item 4302): each `group` with the orders behind
    /// it, and an order with no `group` before it on its own.
    fn issues(kept: &[Command]) -> Vec<Vec<Command>> {
        let mut out: Vec<Vec<Command>> = Vec::new();
        let mut open = false;
        for c in kept {
            match c {
                Command::Group { .. } => {
                    out.push(vec![c.clone()]);
                    open = true;
                }
                _ if open => out.last_mut().unwrap().push(c.clone()),
                _ => out.push(vec![c.clone()]),
            }
        }
        out
    }

    /// **The two profiles the recordings carry** (item 4289,
    /// `docs/CLIENT_INPUT.md` §26.1): the fresh one, every bottle capture's,
    /// and the 2017 sample's, whose `prefs` has `0x800000` set too. Of the
    /// sample's word only [`rondata::prefs::PREFS_BITS`] are known, since
    /// the order reads no other; the rest are the fresh profile's.
    const LEADER_PREFS: [(&str, rondata::prefs::Prefs); 2] = [
        ("fresh", rondata::prefs::Prefs::FRESH),
        (
            "sample",
            rondata::prefs::Prefs {
                prefs: rondata::prefs::DEFAULT_PREFS | 0x80_0000,
                ..rondata::prefs::Prefs::FRESH
            },
        ),
    ];

    /// Which of [`LEADER_PREFS`] the layer mints `c` from, byte for byte,
    /// on play `play` for the human `who`: only on frame 0, where
    /// `Game::run` issues it.
    fn leader_prefs(c: &Command, frame: i32, play: u8, who: u8) -> Option<&'static str> {
        LEADER_PREFS.iter().find_map(|(name, prefs)| {
            let layer = Input {
                prefs: *prefs,
                ..Input::new(play)
            };
            let ours = layer.leader_options(who);
            (frame == 0
                && ours == *c
                && rondata::commands::encode(&[ours])
                    == rondata::commands::encode(std::slice::from_ref(c)))
            .then_some(*name)
        })
    }

    /// The recording's human `who`, `console->who`: the active slot whose
    /// `play` the frame-0 package carries (`Slot::play`, the value packages
    /// carry). The bottle's 49 are `who` 0 on play 0; the 2017 sample's is
    /// `who` 3 on play 0 (its AIs `who` 0, 1, 2, 4 on plays 1 to 4).
    fn console_who(rec: &rondata::recgame::RecGame) -> u8 {
        let play = rec.packages[0].play as u8;
        let human: Vec<&rondata::recgame::Slot> = rec
            .slots
            .iter()
            .filter(|s| s.flags & 1 != 0 && s.play == play)
            .collect();
        assert_eq!(human.len(), 1, "one active slot on play {play}");
        human[0].who
    }

    /// **`INPUT_LEADER_OPTIONS`** (item 4289): of the `leader_options`
    /// orders the recordings on disk carry, those the layer mints byte for
    /// byte from one of [`LEADER_PREFS`]; and how many each profile mints.
    /// Seen red pinned (0, 0).
    const INPUT_LEADER_OPTIONS: (usize, usize) = (50, 50);
    const INPUT_LEADER_OPTIONS_BY_PREFS: &[(&str, usize)] = &[("fresh", 49), ("sample", 1)];

    /// Every recording's frame-0 `leader_options`, minted from the layer's
    /// prefs: the package whole, the seat its `play`.
    #[test]
    fn minted_leader_options_equal_the_recordings() {
        let files = recordings_on_disk();
        if files.is_empty() {
            println!("skipping: no .rcx on disk");
            return;
        }
        let (mut minted, mut of) = (0, 0);
        let mut by: std::collections::BTreeMap<&str, usize> = Default::default();
        for f in &files {
            let rec = rondata::recgame::read(&f.display().to_string()).unwrap();
            let who = console_who(&rec);
            for p in &rec.packages {
                let Ok(cmds) = rondata::commands::decode(&p.data) else {
                    continue;
                };
                for c in cmds
                    .iter()
                    .filter(|c| matches!(c, Command::LeaderOptions { .. }))
                {
                    of += 1;
                    let got = leader_prefs(c, p.frame, p.play as u8, who);
                    println!(
                        "LEADER {}: frame {} play {} {c:?} -> {got:?}",
                        f.file_name().unwrap().to_string_lossy(),
                        p.frame,
                        p.play
                    );
                    if let Some(name) = got {
                        assert_eq!(cmds.len(), 1, "the package is the order alone");
                        minted += 1;
                        *by.entry(name).or_default() += 1;
                    }
                }
            }
        }
        let by: Vec<(&str, usize)> = by.into_iter().collect();
        assert_eq!((minted, of), INPUT_LEADER_OPTIONS);
        assert_eq!(by, INPUT_LEADER_OPTIONS_BY_PREFS);
    }

    /// **`INPUT_WHOLE`** (item 4285, `docs/CLIENT_INPUT.md` §30): of the
    /// recordings on disk, those whose every order is [`Verdict::Shown`],
    /// of all of them. Every recording carries `leader_options`; since item
    /// 4289 the layer mints it from its prefs ([`INPUT_LEADER_OPTIONS`]),
    /// so the count was the old carried one, 42: the 38 files that carry
    /// nothing else, and the four whose other orders are all shown (a
    /// pause pair, two of chat lines, a toggle). Since item 4318 every
    /// form order is minted from a hand-made selection (§30.3): 50 of 50.
    /// Seen red pinned (0, 50).
    const INPUT_WHOLE: (usize, usize) = (50, 50);
    /// Every order on disk by verdict: (shown, of the layer's form but not
    /// minted from a scene, missed, all). A package's `group` takes its
    /// order's verdict.
    const INPUT_WHOLE_ORDERS: (usize, usize, usize, usize) = (2429, 0, 0, 2429);
    /// The missed orders by arm, most first, and who reaches each, the
    /// package's `group` counted with its order. Empty since item 4304
    /// (§38), the dragged placement's ten the last.
    const INPUT_WHOLE_MISSES: &[(&str, Reach, usize)] = &[];

    /// **A person's whole game, by recording** (item 4285): each recording
    /// on disk, its every order given its package's [`Verdict`]; the
    /// recordings whose orders are all shown, and the misses by arm. Each
    /// shown pin is held to the disk: its recorded packages are the census's
    /// count of packages it names, and it mints all of them.
    #[test]
    fn input_whole_recordings_on_disk() {
        let files = recordings_on_disk();
        if files.is_empty() || lobby().is_err() {
            println!("skipping: no .rcx on disk or no lobby");
            return;
        }
        let roster = install_roster().unwrap_or_default();
        let terrain = TerrainScene::default();
        let c = crate::pinned_camera();
        let scene = Scene::default();
        let v = view(&c, &scene, &terrain);
        let mut whole = 0;
        let (mut shown, mut form, mut missed, mut all) = (0, 0, 0, 0);
        let mut by_pin: std::collections::BTreeMap<&str, usize> = Default::default();
        let mut arms: std::collections::BTreeMap<(&str, Reach), usize> = Default::default();
        for f in &files {
            let rec = rondata::recgame::read(&f.display().to_string()).unwrap();
            let who = console_who(&rec);
            let mut pauser = Input::new(0);
            let (mut s, mut fo, mut m) = (0, 0, 0);
            let mut its: std::collections::BTreeMap<&str, usize> = Default::default();
            for p in &rec.packages {
                let Ok(cmds) = rondata::commands::decode(&p.data) else {
                    continue;
                };
                let kept: Vec<Command> =
                    cmds.into_iter().filter(rondata::input::is_input).collect();
                if kept.is_empty() {
                    continue;
                }
                let n = kept.len();
                match census_verdict(&kept, p.frame, p.play as u8, who, &mut pauser, &v, &roster) {
                    Verdict::Shown(pin) => {
                        s += n;
                        *by_pin.entry(pin).or_default() += 1;
                    }
                    Verdict::Form => fo += n,
                    Verdict::Miss(arm, reach) => {
                        m += n;
                        *arms.entry((arm, reach)).or_default() += n;
                        *its.entry(arm).or_default() += n;
                    }
                }
            }
            let name = f.file_name().unwrap().to_string_lossy();
            println!(
                "WHOLE {name}: {s} shown, {fo} form, {m} missed of {}; {its:?}",
                s + fo + m
            );
            whole += usize::from(fo == 0 && m == 0);
            (shown, form, missed) = (shown + s, form + fo, missed + m);
            all += s + fo + m;
        }
        let mut misses: Vec<(&str, Reach, usize)> =
            arms.into_iter().map(|((a, r), n)| (a, r, n)).collect();
        misses.sort_by(|a, b| b.2.cmp(&a.2).then(a.0.cmp(b.0)));
        println!(
            "WHOLE {whole} of {}; orders {shown} shown, {form} form, {missed} missed of {all}",
            files.len()
        );
        println!("WHOLE PINS {by_pin:?}");
        println!("WHOLE MISSES {misses:?}");
        for (pin, (minted, of)) in [
            ("INPUT_UNQUEUES", INPUT_UNQUEUES),
            ("INPUT_CITY_GATHERS", INPUT_CITY_GATHERS),
            ("INPUT_TRADED", INPUT_TRADED),
            ("INPUT_CHEATED", INPUT_CHEATED),
            ("INPUT_HOTKEYS", INPUT_HOTKEYS),
            ("INPUT_GARRISONS", INPUT_GARRISONS),
            ("INPUT_RENAMES", INPUT_RENAMES),
            ("INPUT_BUILDMASKS", INPUT_BUILDMASKS),
            ("INPUT_COME_OUTS", INPUT_COME_OUTS),
            ("INPUT_STANCES", INPUT_STANCES),
            ("INPUT_TOGGLES", INPUT_TOGGLES),
            ("INPUT_SPELLS", INPUT_SPELLS),
            ("INPUT_SPY_SPELLS", INPUT_SPY_SPELLS),
            ("INPUT_DISBANDS", INPUT_DISBANDS),
            ("INPUT_UNITMASKS", INPUT_UNITMASKS),
            ("INPUT_LEADER_OPTIONS", INPUT_LEADER_OPTIONS),
            ("INPUT_ROSTER_QUEUES", INPUT_ROSTER_QUEUES),
            ("INPUT_TURNS", INPUT_TURNS),
            ("INPUT_MAIN_MOVES", INPUT_MAIN_MOVES),
            ("INPUT_HAND_GATHER_POINTS", INPUT_HAND_GATHER_POINTS),
            ("INPUT_HAND_MOVES", INPUT_HAND_MOVES),
            ("INPUT_HAND_OBJECT_ORDERS", INPUT_HAND_OBJECT_ORDERS),
            ("INPUT_HAND_PLACEMENTS", INPUT_HAND_PLACEMENTS),
        ] {
            assert_eq!(minted, of, "{pin} mints every package it reads");
            assert_eq!(
                by_pin.get(pin).copied().unwrap_or(0),
                of,
                "{pin} against the census"
            );
        }
        assert_eq!((whole, files.len()), INPUT_WHOLE);
        assert_eq!((shown, form, missed, all), INPUT_WHOLE_ORDERS);
        assert_eq!(misses, INPUT_WHOLE_MISSES);
    }

    /// The kind of one form package's head, as §30.3's census of the form
    /// orders splits it (item 4318).
    fn form_kind(head: &Command) -> &'static str {
        match *head {
            Command::MoveTo {
                orders: 1,
                form: -1,
                ..
            } => "move_to minimap",
            Command::MoveTo { orders, .. } if orders == ATTACK_TO => "attack_to",
            Command::MoveTo { set_angle: 1, .. } => "move_to faced",
            Command::MoveTo { .. } => "move_to main",
            Command::Build { x, y, x2, y2, .. } if (x, y) == (x2, y2) => "build click",
            Command::Build { .. } => "build drag",
            Command::GatherPoint { action: 0, .. } => "gather_point 0",
            Command::GatherPoint { action: 1, .. } => "gather_point 1",
            Command::GatherPoint { action: 2, .. } => "gather_point 2",
            Command::GatherPoint { .. } => "gather_point 3",
            Command::Attack { .. } => "attack",
            Command::Guard { .. } => "guard",
            Command::Gather { .. } => "gather",
            Command::SwarmAround { orders, .. } if orders == BUILD_AT => "swarm build_at",
            Command::SwarmAround { .. } => "swarm repair",
            _ => "other",
        }
    }

    /// A camera at which a main-view press reaches `p`: the pinned camera
    /// moved onto `p` and nudged by eighths of a unit, the pixel `p`
    /// projects to and [`ground_at`] there `p` (item 4318).
    fn camera_reaching(
        scene: &Scene,
        terrain: &TerrainScene,
        p: Pos,
    ) -> Option<(render::Camera, (i32, i32))> {
        let c0 = crate::pinned_camera();
        let at = |lx: f32, ly: f32| {
            let mut c = c0;
            (c.loc_x, c.loc_y) = (lx, ly);
            let v = view(&c, scene, terrain);
            let px = v.pixel(p.x, p.y, v.ground_z(p.x, p.y));
            (c, px, ground_at(&v, px.0, px.1))
        };
        for k in 0..64 {
            let (mut lx, mut ly) = (
                p.x as f32 + (k % 8) as f32 / 8.0,
                p.y as f32 + (k / 8) as f32 / 8.0,
            );
            // The press's point sits a few units off the projected one:
            // move the camera by that offset, twice at most.
            for _ in 0..3 {
                let (c, px, got) = at(lx, ly);
                match got {
                    Some(g) if g == p => return Some((c, px)),
                    Some(g) => (lx, ly) = (lx - (g.x - p.x) as f32, ly - (g.y - p.y) as f32),
                    None => break,
                }
            }
        }
        None
    }

    /// **A selection made by hand** (item 4318, §30.3): the recorded
    /// group's units, `who`'s `objects`, laid out on a grid from `at`,
    /// each run of ascending objects one type, so that the box's sort by
    /// type and then object (§7.1) gives the recorded order back. Every
    /// type is a soldier's, past the civilian rule. `form` and `width` are
    /// each member's, so [`View::formation`] gives them back.
    fn hand_units(
        who: u8,
        objects: &[i16],
        form: i8,
        width: i8,
        at: Pos,
    ) -> Vec<sim::observe::UnitScene> {
        let mut run = 0;
        objects
            .iter()
            .enumerate()
            .map(|(k, &o)| {
                if k > 0 && o < objects[k - 1] {
                    run += 1;
                }
                let mut u = unit(who, o, HAND_TYPE + run, 0, form, width);
                u.pos = Pos {
                    x: at.x + (k % 12) as i32 * HAND_SPACING,
                    y: at.y + (k / 12) as i32 * HAND_SPACING,
                };
                u
            })
            .collect()
    }

    /// The first type [`hand_units`] gives (`0x60`): no worker, merchant
    /// or fleet type lies at or past it below `0x13d`, the fishermen.
    const HAND_TYPE: i32 = 0x60;
    /// The grid's step, in world units: at the pinned zoom about 45 px,
    /// past [`PICK_RADIUS`].
    const HAND_SPACING: i32 = 300;
    /// Where the hand's units stand from the order's point: off the
    /// screen of the camera that presses it.
    const HAND_AWAY: i32 = 50_000;

    /// The camera the hand selects at: the pinned one, looking at `at`.
    fn camera_at(at: Pos) -> render::Camera {
        let mut c = crate::pinned_camera();
        (c.loc_x, c.loc_y) = (at.x as f32, at.y as f32);
        c
    }

    /// **The hand's events** (item 4318): `who`'s `objects` selected by
    /// a press on the one, or by a box over them all, at a camera on the
    /// grid, each member found in `scene`; true when the layer's selection
    /// is the recorded group, in its order.
    fn select_by_hand(
        input: &mut Input,
        frame: i32,
        scene: &Scene,
        terrain: &TerrainScene,
        who: u8,
        objects: &[i16],
    ) -> bool {
        let members: Vec<&sim::observe::UnitScene> = objects
            .iter()
            .filter_map(|&o| scene.units.iter().find(|u| u.id == ObjId { who, o }))
            .collect();
        let buildings: Vec<&sim::observe::BuildingScene> = objects
            .iter()
            .filter_map(|&o| scene.buildings.iter().find(|b| b.id == ObjId { who, o }))
            .collect();
        let first = members
            .first()
            .map(|u| (u.pos, u.z))
            .or_else(|| buildings.first().map(|b| (b.pos, b.z)));
        let Some((at, _)) = first else {
            return objects.is_empty();
        };
        let c = camera_at(at);
        let v = view(&c, scene, terrain);
        let pixels: Vec<(i32, i32)> = members
            .iter()
            .map(|u| v.pixel(u.pos.x, u.pos.y, u.z))
            .chain(buildings.iter().map(|b| v.pixel(b.pos.x, b.pos.y, b.z)))
            .collect();
        if pixels.len() == 1 {
            let (x, y) = pixels[0];
            assert_eq!(input.event(frame, Event::Press { x, y }, &v), None);
        } else {
            let x0 = pixels.iter().map(|p| p.0).min().unwrap();
            let x1 = pixels.iter().map(|p| p.0).max().unwrap();
            let y0 = pixels.iter().map(|p| p.1).min().unwrap();
            let y1 = pixels.iter().map(|p| p.1).max().unwrap();
            assert_eq!(input.event(frame, Event::Drag { x0, y0, x1, y1 }, &v), None);
        }
        input.selection
            == objects
                .iter()
                .map(|&o| ObjId { who, o })
                .collect::<Vec<_>>()
    }

    /// The modifiers a dispatch's queue is held with (§33.1): shift for
    /// `QUEUE_LAST`, shift and alt for `QUEUE_FIRST`.
    fn dispatch_mods(queued: i32) -> Mods {
        Mods {
            shift: queued != QUEUE_NEW,
            alt: queued == QUEUE_FIRST,
            ctrl: false,
        }
    }

    /// One recorded package of [`recorded_issues`]: `(frame, kept,
    /// who, objects, before)`.
    type Issue = (i32, Vec<Command>, u8, Vec<i16>, Option<Group>);

    /// The recorded packages the census reads, by recording: each one's
    /// frame, its kept orders, and the selection behind its `group` with
    /// the slot's last group before it ([`behind`]).
    fn recorded_issues(rec: &rondata::recgame::RecGame) -> Vec<Issue> {
        let mut last = LastSent::new();
        let mut out = Vec::new();
        for p in &rec.packages {
            let Ok(cmds) = rondata::commands::decode(&p.data) else {
                continue;
            };
            let kept: Vec<Command> = cmds.into_iter().filter(rondata::input::is_input).collect();
            let (who, objects, before) = behind(&last, p.play, &kept);
            note_sent(&mut last, p.play, &kept);
            if !kept.is_empty() {
                out.push((p.frame, kept, who, objects, before));
            }
        }
        out
    }

    /// The one head of a package of one issue: its order behind the
    /// `group`, or `None` for two issues or none.
    fn one_head(kept: &[Command]) -> Option<&Command> {
        let heads: Vec<&Command> = kept
            .iter()
            .filter(|c| !matches!(c, Command::Group { .. }))
            .collect();
        (heads.len() == 1).then(|| heads[0])
    }

    /// Whether `c` is a main-view right press's `move_to` the layer
    /// writes (§7.1, §33.1): `orders` 1, no facing, a formation of the
    /// layer's, an issuer's queue.
    fn is_main_move(c: &Command) -> bool {
        matches!(*c, Command::MoveTo {
            orders: 1,
            set_angle: 0,
            angle: 0,
            disembark: 0,
            form,
            width,
            queued,
            ..
        } if matches!((form, width), (0, 50) | (9, 50)) && QUEUES.contains(&i32::from(queued)))
    }

    /// **`INPUT_MAIN_MOVES`** (item 4318, §30.3): of the recorded packages
    /// of one issue whose order is a main view's `move_to`, those the
    /// layer mints byte for byte from a person's events: the group
    /// selected by a press or a box ([`select_by_hand`]), the camera
    /// moved to one at which a press reaches the point
    /// ([`camera_reaching`]), and a right press there with the queue's
    /// modifiers.
    const INPUT_MAIN_MOVES: (usize, usize) = (168, 168);

    #[test]
    fn minted_main_moves_equal_the_recordings() {
        let files = recordings_on_disk();
        if files.is_empty() {
            println!("skipping: no .rcx on disk");
            return;
        }
        let terrain = TerrainScene::default();
        let (mut minted, mut all) = (0, 0);
        for f in &files {
            let rec = rondata::recgame::read(&f.display().to_string()).unwrap();
            for (frame, kept, who, objects, before) in recorded_issues(&rec) {
                let Some(&Command::MoveTo {
                    to_x,
                    to_y,
                    form,
                    width,
                    queued,
                    ..
                }) = one_head(&kept).filter(|c| is_main_move(c))
                else {
                    continue;
                };
                all += 1;
                let to = Pos { x: to_x, y: to_y };
                let away = Pos {
                    x: to.x + HAND_AWAY,
                    y: to.y + HAND_AWAY,
                };
                let scene = Scene {
                    units: hand_units(who, &objects, form, width, away),
                    ..Scene::default()
                };
                let mut input = Input::new(who);
                input.sent = sent_for(&before, &scene);
                let selected = select_by_hand(&mut input, frame, &scene, &terrain, who, &objects);
                let got = camera_reaching(&scene, &terrain, to).and_then(|(c, (x, y))| {
                    let v = view(&c, &scene, &terrain);
                    input.mods = dispatch_mods(i32::from(queued));
                    input.event(frame, Event::RightPress { x, y }, &v)
                });
                if selected && got.as_ref().map(|o| &o.2) == Some(&kept) {
                    minted += 1;
                } else {
                    println!("MAIN MOVE PARTS f{frame} selected {selected} {kept:?}: {got:?}");
                }
            }
        }
        println!("MAIN MOVES {minted} of {all}");
        assert_eq!((minted, all), INPUT_MAIN_MOVES);
    }

    /// Whether `c` is a building's gather point the layer writes (§11.2,
    /// §11.4): `add_to_end` 0, at a point for actions 0 to 2, and action
    /// 3 naming `(captain, who)` of the issuer's slot.
    fn is_hand_gather_point(c: &Command, who: u8) -> bool {
        match *c {
            Command::GatherPoint {
                action: 0..=2,
                add_to_end: 0,
                ..
            } => true,
            Command::GatherPoint {
                action: 3,
                y,
                add_to_end: 0,
                ..
            } => y == i32::from(who),
            _ => false,
        }
    }

    /// **`INPUT_HAND_GATHER_POINTS`** (item 4318, §30.3): of the recorded
    /// packages of one issue whose order is a building's gather point,
    /// those the layer mints byte for byte from a person's events: the
    /// group's buildings selected by a press ([`select_by_hand`]), then a
    /// right press on the ground at a camera that reaches the point
    /// (action 0), on a building standing at the point, the issuer's or
    /// an enemy's (1, 2), or on one of the issuer's units whose captain is
    /// the recorded `x` (3).
    const INPUT_HAND_GATHER_POINTS: (usize, usize) = (171, 171);

    #[test]
    fn minted_gather_points_by_hand() {
        let files = recordings_on_disk();
        let Ok(r) = lobby() else {
            println!("skipping: no lobby");
            return;
        };
        if files.is_empty() {
            println!("skipping: no .rcx on disk");
            return;
        }
        let template = r.scene(None).buildings[0];
        drop(r);
        let terrain = TerrainScene::default();
        let (mut minted, mut all) = (0, 0);
        for f in &files {
            let rec = rondata::recgame::read(&f.display().to_string()).unwrap();
            let me = console_who(&rec);
            for (frame, kept, who, objects, before) in recorded_issues(&rec) {
                let Some(&Command::GatherPoint { x, y, action, .. }) =
                    one_head(&kept).filter(|c| is_hand_gather_point(c, me))
                else {
                    continue;
                };
                all += 1;
                let point = Pos { x, y };
                let away = Pos {
                    x: x.rem_euclid(30_000) + HAND_AWAY,
                    y: y.rem_euclid(30_000) + HAND_AWAY,
                };
                let enemy_slot = (who + 1) % 5;
                let mut scene = Scene {
                    buildings: objects
                        .iter()
                        .enumerate()
                        .map(|(k, &o)| {
                            let mut b = template;
                            (b.id, b.owner, b.trains, b.seen) = (ObjId { who, o }, who, true, true);
                            // Each run of ascending objects one type, as
                            // [`hand_units`] does for units.
                            let run = objects[..=k].windows(2).filter(|w| w[1] < w[0]).count();
                            b.ty = Some(run);
                            b.pos = Pos {
                                x: away.x + k as i32 * HAND_SPACING,
                                y: away.y,
                            };
                            b
                        })
                        .collect(),
                    leaders: vec![sim::observe::LeaderScene {
                        who,
                        enemies: vec![enemy_slot],
                        ..leader_scene(who)
                    }],
                    ..Scene::default()
                };
                let target = match action {
                    0 => None,
                    3 => {
                        let mut u = unit(who, HAND_TARGET, 0x36, 0, 0, 0);
                        (u.captain, u.pos) = (x as i16, Pos { x: 0, y: 0 });
                        let at = (u.pos, u.z);
                        scene.units.push(u);
                        Some(at)
                    }
                    _ => {
                        let mut b = template;
                        let owner = if action == 2 { enemy_slot } else { who };
                        b.id = ObjId {
                            who: owner,
                            o: HAND_TARGET,
                        };
                        (b.owner, b.seen, b.trains, b.pos) = (owner, true, false, point);
                        scene.buildings.push(b);
                        Some((b.pos, b.z))
                    }
                };
                let mut input = Input::new(who);
                input.sent = sent_for(&before, &scene);
                let selected = select_by_hand(&mut input, frame, &scene, &terrain, who, &objects);
                let press = match target {
                    None => camera_reaching(&scene, &terrain, point),
                    Some((at, z)) => {
                        let c = camera_at(at);
                        let v = view(&c, &scene, &terrain);
                        Some((c, v.pixel(at.x, at.y, z)))
                    }
                };
                let got = press.and_then(|(c, (px, py))| {
                    let v = view(&c, &scene, &terrain);
                    input.event(frame, Event::RightPress { x: px, y: py }, &v)
                });
                if selected && got.as_ref().map(|o| &o.2) == Some(&kept) {
                    minted += 1;
                } else {
                    println!("HAND GP PARTS f{frame} selected {selected} {kept:?}: {got:?}");
                }
            }
        }
        println!("HAND GATHER POINTS {minted} of {all}");
        assert_eq!((minted, all), INPUT_HAND_GATHER_POINTS);
    }

    /// The object a hand-made scene's target is: past every recorded
    /// object of the census.
    const HAND_TARGET: i16 = 30_000;

    /// Whether `c` is a `move_to` the layer writes off the main view's
    /// plain press (item 4318): the minimap's, −1/−1 (§8); the
    /// attack-move, `ATTACK_TO` with no facing and −1/−1 (§35); or the
    /// right drag's, `set_angle` 1 with an angle of `find_angle`'s image and
    /// the press's formation (§34). Each with an issuer's queue.
    fn is_hand_move(c: &Command) -> bool {
        let &Command::MoveTo {
            set_angle,
            angle,
            orders,
            queued,
            form,
            width,
            disembark,
            ..
        } = c
        else {
            return false;
        };
        QUEUES.contains(&i32::from(queued))
            && disembark == 0
            && match (orders, set_angle) {
                (1, 0) => (angle, form, width) == (0, -1, -1),
                (1, 1) => of_find_angle(angle) && matches!((form, width), (0, 50) | (9, 50)),
                (o, 0) if o == ATTACK_TO => (angle, form, width) == (0, -1, -1),
                _ => false,
            }
    }

    /// The modifiers the minimap's queue is held with (§33.1): shift for
    /// `QUEUE_LAST`, alt for `QUEUE_FIRST`.
    fn map_mods(queued: i32) -> Mods {
        Mods {
            shift: queued == QUEUE_LAST,
            alt: queued == QUEUE_FIRST,
            ctrl: false,
        }
    }

    /// **A right drag's release** (item 4318): a pixel at least
    /// [`crate::DRAG_SLOP`] from the press at `(px, py)`, whose ground point
    /// `find_angle` takes from `at`'s to `angle`. Walked along the angle's
    /// own direction on the ground, each step's pixel and its eight
    /// neighbours, out to the screen's edge.
    fn release_for(v: &View, at: Pos, (px, py): (i32, i32), angle: i32) -> Option<(i32, i32)> {
        let th = f64::from(angle as u32) * std::f64::consts::TAU / 4_294_967_296.0;
        // `Angle(0)` is north, y decreasing; a quarter turn is east.
        let (dx, dy) = (th.sin(), -th.cos());
        let (w, h) = (v.screen.scaled_xs, 1080);
        for k in 1..4000 {
            let r = f64::from(k) * 3.0;
            let (wx, wy) = (
                (f64::from(at.x) + r * dx).round() as i32,
                (f64::from(at.y) + r * dy).round() as i32,
            );
            let (cx, cy) = v.pixel(wx, wy, v.ground_z(wx, wy));
            if !(0..w).contains(&cx) || !(0..h).contains(&cy) {
                return None;
            }
            for (ox, oy) in [
                (0, 0),
                (1, 0),
                (-1, 0),
                (0, 1),
                (0, -1),
                (1, 1),
                (1, -1),
                (-1, 1),
                (-1, -1),
            ] {
                let (x, y) = (cx + ox, cy + oy);
                if (x - px).pow(2) + (y - py).pow(2) < DRAG_SLOP_PX.pow(2) {
                    continue;
                }
                if let Some(to) = ground_at(v, x, y)
                    && sim::movement::find_angle(to.x - at.x, to.y - at.y).0 == angle
                {
                    return Some((x, y));
                }
            }
        }
        None
    }

    /// The window's drag threshold, in pixels (§7.1, ours).
    const DRAG_SLOP_PX: i32 = 4;

    /// **`INPUT_HAND_MOVES`** (item 4318, §30.3): of the recorded
    /// packages of one issue whose order is a minimap's `move_to`, an
    /// attack-move or a right drag's ([`is_hand_move`]), those the layer
    /// mints byte for byte from a person's events on a hand-made
    /// selection ([`select_by_hand`]): a minimap press at the point with
    /// its queue's modifiers; ctrl on a main-view press at a camera that
    /// reaches the point, or on the minimap for `QUEUE_FIRST`, which the
    /// main view's alt turns into a move; a right drag from a press that
    /// reaches the point to a release [`release_for`] finds.
    const INPUT_HAND_MOVES: (usize, usize) = (222, 222);

    #[test]
    fn minted_hand_moves_equal_the_recordings() {
        let files = recordings_on_disk();
        if files.is_empty() {
            println!("skipping: no .rcx on disk");
            return;
        }
        let terrain = TerrainScene {
            width: 256,
            height: 256,
            ..TerrainScene::default()
        };
        let (mut minted, mut all) = (0, 0);
        let mut by: std::collections::BTreeMap<&str, (usize, usize)> = Default::default();
        for f in &files {
            let rec = rondata::recgame::read(&f.display().to_string()).unwrap();
            for (frame, kept, who, objects, before) in recorded_issues(&rec) {
                let Some(
                    head @ &Command::MoveTo {
                        to_x,
                        to_y,
                        angle,
                        form,
                        width,
                        queued,
                        ..
                    },
                ) = one_head(&kept).filter(|c| is_hand_move(c))
                else {
                    continue;
                };
                all += 1;
                let kind = form_kind(head);
                let to = Pos { x: to_x, y: to_y };
                let away = Pos {
                    x: to.x + HAND_AWAY,
                    y: to.y + HAND_AWAY,
                };
                let (uf, uw) = if form == -1 { (0, 50) } else { (form, width) };
                let scene = Scene {
                    units: hand_units(who, &objects, uf, uw, away),
                    ..Scene::default()
                };
                let mut input = Input::new(who);
                input.sent = sent_for(&before, &scene);
                let selected = select_by_hand(&mut input, frame, &scene, &terrain, who, &objects);
                let queued = i32::from(queued);
                let v0 = view(&crate::pinned_camera(), &scene, &terrain);
                let got = match kind {
                    "move_to minimap" => {
                        input.mods = map_mods(queued);
                        input.event(frame, Event::MapRightPress { x: to.x, y: to.y }, &v0)
                    }
                    "attack_to" if queued == QUEUE_FIRST => {
                        input.mods = Mods {
                            ctrl: true,
                            ..map_mods(queued)
                        };
                        input.event(frame, Event::MapRightPress { x: to.x, y: to.y }, &v0)
                    }
                    "attack_to" => camera_reaching(&scene, &terrain, to).and_then(|(c, (x, y))| {
                        let v = view(&c, &scene, &terrain);
                        input.mods = Mods {
                            ctrl: true,
                            ..dispatch_mods(queued)
                        };
                        input.event(frame, Event::RightPress { x, y }, &v)
                    }),
                    _ => camera_reaching(&scene, &terrain, to).and_then(|(c, (x0, y0))| {
                        let v = view(&c, &scene, &terrain);
                        let (x1, y1) = release_for(&v, to, (x0, y0), angle)?;
                        input.mods = map_mods(queued);
                        input.event(frame, Event::RightDrag { x0, y0, x1, y1 }, &v)
                    }),
                };
                let e = by.entry(kind).or_default();
                e.1 += 1;
                if selected && got.as_ref().map(|o| &o.2) == Some(&kept) {
                    minted += 1;
                    e.0 += 1;
                } else {
                    println!(
                        "HAND MOVE PARTS {kind} f{frame} selected {selected} {kept:?}: {got:?}"
                    );
                }
            }
        }
        println!("HAND MOVES {minted} of {all}; {by:?}");
        assert_eq!((minted, all), INPUT_HAND_MOVES);
    }

    /// Whether `c` is an order on an object the right press's dispatch
    /// writes (§6, §10): `attack` with `ignore` 0, `guard`, `gather`, and
    /// `swarm_around` to build or repair, each with an issuer's queue.
    fn is_hand_object_order(c: &Command) -> bool {
        match *c {
            Command::Attack {
                ignore: 0, queued, ..
            }
            | Command::Guard { queued, .. }
            | Command::Gather { queued, .. } => QUEUES.contains(&queued),
            Command::SwarmAround { queued, orders, .. } => {
                QUEUES.contains(&queued) && (orders == BUILD_AT || orders == REPAIR)
            }
            _ => false,
        }
    }

    /// **`INPUT_HAND_OBJECT_ORDERS`** (item 4318, §30.3): of the recorded
    /// packages of one issue whose order is on an object
    /// ([`is_hand_object_order`]), those the layer mints byte for byte from
    /// a person's events: a hand-made selection ([`select_by_hand`]), then
    /// a right press on the object the order names, with the queue's
    /// modifiers. The object stands where the press names it: another
    /// slot's unit to attack, one of the issuer's units to guard, one of
    /// its buildings with room to gather at, unfinished to build, damaged
    /// to repair.
    const INPUT_HAND_OBJECT_ORDERS: (usize, usize) = (129, 129);

    #[test]
    fn minted_object_orders_by_hand() {
        let files = recordings_on_disk();
        let Ok(r) = lobby() else {
            println!("skipping: no lobby");
            return;
        };
        if files.is_empty() {
            println!("skipping: no .rcx on disk");
            return;
        }
        let template = r.scene(None).buildings[0];
        drop(r);
        let terrain = TerrainScene::default();
        let (mut minted, mut all) = (0, 0);
        let mut by: std::collections::BTreeMap<&str, (usize, usize)> = Default::default();
        for f in &files {
            let rec = rondata::recgame::read(&f.display().to_string()).unwrap();
            for (frame, kept, who, objects, before) in recorded_issues(&rec) {
                let Some(head) = one_head(&kept).filter(|c| is_hand_object_order(c)) else {
                    continue;
                };
                all += 1;
                let kind = form_kind(head);
                let at = Pos {
                    x: 10_000,
                    y: 10_000,
                };
                let away = Pos {
                    x: at.x + HAND_AWAY,
                    y: at.y + HAND_AWAY,
                };
                let mut scene = Scene {
                    units: hand_units(who, &objects, 0, 50, away),
                    ..Scene::default()
                };
                let building = |o: i32, active: bool| {
                    let mut b = template;
                    (b.id, b.owner, b.pos, b.seen) = (ObjId { who, o: o as i16 }, who, at, true);
                    (b.active, b.gather) = (active, None);
                    b.health = b.hits;
                    b
                };
                let (queued, z) = match *head {
                    Command::Attack {
                        ox, whom, queued, ..
                    }
                    | Command::Guard { ox, whom, queued } => {
                        let mut u = unit(whom as u8, ox as i16, HAND_TYPE, 0, 0, 50);
                        u.pos = at;
                        let z = u.z;
                        scene.units.push(u);
                        (queued, z)
                    }
                    Command::Gather { ox, queued } => {
                        let mut b = building(ox, true);
                        b.gather = Some((0, Some(5)));
                        scene.buildings.push(b);
                        (queued, b.z)
                    }
                    Command::SwarmAround {
                        ox, queued, orders, ..
                    } => {
                        let mut b = building(ox, orders != BUILD_AT);
                        if orders == REPAIR {
                            b.health = b.hits - 1;
                        }
                        scene.buildings.push(b);
                        (queued, b.z)
                    }
                    _ => unreachable!("an order on an object"),
                };
                let mut input = Input::new(who);
                input.sent = sent_for(&before, &scene);
                let selected = select_by_hand(&mut input, frame, &scene, &terrain, who, &objects);
                let c = camera_at(at);
                let v = view(&c, &scene, &terrain);
                let (x, y) = v.pixel(at.x, at.y, z);
                input.mods = dispatch_mods(queued);
                let got = input.event(frame, Event::RightPress { x, y }, &v);
                let e = by.entry(kind).or_default();
                e.1 += 1;
                if selected && got.as_ref().map(|o| &o.2) == Some(&kept) {
                    minted += 1;
                    e.0 += 1;
                } else {
                    println!(
                        "HAND OBJECT PARTS {kind} f{frame} selected {selected} {kept:?}: {got:?}"
                    );
                }
            }
        }
        println!("HAND OBJECT ORDERS {minted} of {all}; {by:?}");
        assert_eq!((minted, all), INPUT_HAND_OBJECT_ORDERS);
    }

    /// A camera at which main-view presses reach every one of `points`:
    /// [`camera_reaching`]'s search on the first, from 32 by 32 starting
    /// fractions, until the others' pixels give them back too.
    fn camera_reaching_all(
        scene: &Scene,
        terrain: &TerrainScene,
        points: &[Pos],
    ) -> Option<(render::Camera, Vec<(i32, i32)>)> {
        for k in HAND_ZOOMS {
            let zoom = k as f32 / 20.0;
            let mut c0 = crate::pinned_camera();
            c0.zoom_distance *= zoom;
            c0.parallel_distance *= zoom;
            if let Some(found) = reaching_all_at(c0, scene, terrain, points) {
                return Some(found);
            }
        }
        None
    }

    /// The zooms [`camera_reaching_all`] tries, in twentieths of the
    /// pinned camera's distance: the pinned one, then from a fifth to
    /// twice it.
    const HAND_ZOOMS: [i32; 38] = {
        let mut z = [20; 38];
        let mut k = 1;
        while k < 38 {
            z[k] = 3 + k as i32;
            k += 1;
        }
        z
    };

    /// [`camera_reaching_all`] at one zoom.
    fn reaching_all_at(
        c0: render::Camera,
        scene: &Scene,
        terrain: &TerrainScene,
        points: &[Pos],
    ) -> Option<(render::Camera, Vec<(i32, i32)>)> {
        let p = points[0];
        for k in 0..1024 {
            let (mut lx, mut ly) = (
                p.x as f32 + (k % 32) as f32 / 32.0,
                p.y as f32 + (k / 32) as f32 / 32.0,
            );
            for _ in 0..3 {
                let mut c = c0;
                (c.loc_x, c.loc_y) = (lx, ly);
                let v = view(&c, scene, terrain);
                let px = v.pixel(p.x, p.y, v.ground_z(p.x, p.y));
                match ground_at(&v, px.0, px.1) {
                    Some(g) if g == p => {
                        let pixels: Vec<(i32, i32)> = points
                            .iter()
                            .map(|q| v.pixel(q.x, q.y, v.ground_z(q.x, q.y)))
                            .collect();
                        if points
                            .iter()
                            .zip(&pixels)
                            .all(|(q, px)| ground_at(&v, px.0, px.1) == Some(*q))
                        {
                            return Some((c, pixels));
                        }
                        break;
                    }
                    Some(g) => (lx, ly) = (lx - (g.x - p.x) as f32, ly - (g.y - p.y) as f32),
                    None => break,
                }
            }
        }
        None
    }

    /// The modifiers a placement's queue is held with (§33.1): shift for
    /// `QUEUE_LAST`, alt for `QUEUE_FIRST`, as `placed_queue` reads them.
    fn placement_mods(queued: i32) -> Mods {
        map_mods(queued)
    }

    /// **`INPUT_HAND_PLACEMENTS`** (item 4318, §30.3): of the recorded
    /// packages of one issue whose order is a `build` the census calls the
    /// layer's form, those the layer mints byte for byte from a person's
    /// events: a hand-made selection ([`select_by_hand`]), the type armed
    /// (`Event::Place`, the roster's build page or a key), then at a camera
    /// that reaches both points ([`camera_reaching_all`]) a click where
    /// `(x2, y2)` is `(x, y)`, and otherwise a release away from the press,
    /// which the pick takes whether or not it passes the slop (§38.2),
    /// with the queue's modifiers.
    const INPUT_HAND_PLACEMENTS: (usize, usize) = (31, 31);

    #[test]
    fn minted_placements_by_hand() {
        let files = recordings_on_disk();
        let Some(roster) = install_roster() else {
            println!("skipping: no install");
            return;
        };
        if files.is_empty() {
            println!("skipping: no .rcx on disk");
            return;
        }
        let terrain = TerrainScene::default();
        let (mut minted, mut all) = (0, 0);
        let mut by: std::collections::BTreeMap<&str, (usize, usize)> = Default::default();
        for f in &files {
            let rec = rondata::recgame::read(&f.display().to_string()).unwrap();
            for (frame, kept, who, objects, before) in recorded_issues(&rec) {
                let Some(
                    head @ &Command::Build {
                        x,
                        y,
                        x2,
                        y2,
                        build,
                        queued,
                    },
                ) = one_head(&kept)
                else {
                    continue;
                };
                if !QUEUES.contains(&queued)
                    || !(BUILD_KEYS.iter().any(|k| k.1 == build) || roster.builds.contains(&build))
                {
                    continue;
                }
                all += 1;
                let kind = form_kind(head);
                let (at, from) = (Pos { x, y }, Pos { x: x2, y: y2 });
                let away = Pos {
                    x: at.x + HAND_AWAY,
                    y: at.y + HAND_AWAY,
                };
                let scene = Scene {
                    units: hand_units(who, &objects, 0, 50, away),
                    ..Scene::default()
                };
                let mut input = Input::new(who);
                input.sent = sent_for(&before, &scene);
                let selected = select_by_hand(&mut input, frame, &scene, &terrain, who, &objects);
                input.mods = placement_mods(queued);
                let v0 = view(&crate::pinned_camera(), &scene, &terrain);
                assert_eq!(input.event(frame, Event::Place(build), &v0), None);
                let got = camera_reaching_all(&scene, &terrain, &[at, from]).and_then(|(c, px)| {
                    let v = view(&c, &scene, &terrain);
                    let ((x1, y1), (x0, y0)) = (px[0], px[1]);
                    if at == from {
                        input.event(frame, Event::Press { x: x1, y: y1 }, &v)
                    } else {
                        input.event(frame, Event::Drag { x0, y0, x1, y1 }, &v)
                    }
                });
                let e = by.entry(kind).or_default();
                e.1 += 1;
                if selected && got.as_ref().map(|o| &o.2) == Some(&kept) {
                    minted += 1;
                    e.0 += 1;
                } else {
                    println!(
                        "HAND PLACEMENT PARTS {kind} f{frame} selected {selected} {kept:?}: {got:?}"
                    );
                }
            }
        }
        println!("HAND PLACEMENTS {minted} of {all}; {by:?}");
        assert_eq!((minted, all), INPUT_HAND_PLACEMENTS);
    }

    /// **The minimap's gather point** (item 4302, §36.2): with buildings
    /// selected, a minimap press is the gather point at the map point,
    /// `action` 0, `add_to_end` with shift or ctrl and not alt; a
    /// selection holding a patrol's launcher mints nothing; off the map,
    /// nothing. Two presses in one turn are one package, the second
    /// `group` empty (§20).
    #[test]
    fn minted_a_minimap_gather_point_as_the_original() {
        let Ok(r) = lobby() else {
            println!("skipping: no lobby");
            return;
        };
        let terrain = r.terrain();
        let mut b = r.scene(None).buildings[0];
        drop(r);
        let c = crate::pinned_camera();
        (b.id, b.owner, b.ty) = (ObjId { who: 0, o: 2000 }, 0, Some(0));
        let scene = Scene {
            buildings: vec![b],
            ..Scene::default()
        };
        let v = view(&c, &scene, &terrain);
        let press = |mods: Mods, x: i32, y: i32, scene: &Scene| {
            let v = view(&c, scene, &terrain);
            let mut input = Input {
                selection: vec![b.id],
                building: true,
                mods,
                ..Input::new(0)
            };
            input
                .event(1, Event::MapRightPress { x, y }, &v)
                .map(|o| o.2)
        };
        let gp = |add_to_end: i32| Command::GatherPoint {
            x: 9574,
            y: 25876,
            action: 0,
            add_to_end,
        };
        let none = Mods::default();
        for (m, want) in [
            (none, gp(0)),
            (
                Mods {
                    shift: true,
                    ..none
                },
                gp(1),
            ),
            (Mods { ctrl: true, ..none }, gp(1)),
            (Mods { alt: true, ..none }, gp(0)),
        ] {
            let got = press(m, 9574, 25876, &scene).unwrap();
            assert_eq!(
                got[0],
                Command::Group {
                    who: 0,
                    objects: vec![2000]
                },
                "{m:?}"
            );
            assert_eq!(got[1], want, "{m:?}");
        }
        let (w, _) = v.map_extent();
        assert_eq!(press(none, w, 25876, &scene), None, "off the map");
        for t in PATROL_LAUNCHERS {
            let mut s = scene.clone();
            s.buildings[0].ty = Some(t);
            assert_eq!(press(none, 9574, 25876, &s), None, "a launcher {t}");
        }
        // Two presses in one turn: one package, the second group empty.
        let mut input = Input {
            selection: vec![b.id],
            building: true,
            ..Input::new(0)
        };
        let ev = Event::MapRightPress { x: 9574, y: 25876 };
        let got = crate::hud::turn(&mut input, HUD_SIZE, 5, &[ev, ev], &v).unwrap();
        assert_eq!(
            got,
            (
                5,
                0,
                vec![
                    Command::Group {
                        who: 0,
                        objects: vec![2000]
                    },
                    gp(0),
                    Command::Group {
                        who: 0,
                        objects: vec![]
                    },
                    gp(0),
                ]
            )
        );
        assert_eq!(crate::hud::turn(&mut input, HUD_SIZE, 6, &[], &v), None);
    }

    /// **`INPUT_TURNS`** (item 4302, `docs/CLIENT_INPUT.md` §36): of the
    /// recorded packages holding two issues or more, those the layer mints
    /// byte for byte from one turn's events through the HUD
    /// ([`crate::hud::turn`]). Seen red pinned (0, 9).
    const INPUT_TURNS: (usize, usize) = (9, 9);
    /// The turns by `(kind, issues)`: all nine are the 2017 sample's, play
    /// 0, slot 3, one issue repeated behind §20's empty `group`.
    const TURN_KINDS: &[((&str, usize), usize)] = &[
        (("gather_point", 2), 3),
        (("move_to", 2), 3),
        (("queue_up", 2), 2),
        (("queue_up", 3), 1),
    ];
    /// The minted turns written as a file (`recgame::minted_packages` over
    /// the sample's header, the paused pair inside its recorded pause) and
    /// read back: each turn's package, its frame and its bytes, the
    /// recording's. Seen red pinned 0.
    const TURNS_WRITTEN: usize = 9;

    /// **Two issues in one turn, from the layer** (item 4302): each
    /// recorded package of two issues or more, its selection the package's
    /// group (or for an empty group the slot's last full one) and the
    /// layer's [`Sent`] the slot's last before it; then one turn of events,
    /// one per issue: a roster button's press for a `queue_up`, a minimap
    /// press for a `move_to` (the shift for `QUEUE_LAST`) and for a
    /// building's `gather_point`. The turn's one package equals the
    /// recording's byte for byte, the camera aside.
    #[test]
    fn minted_turns_equal_the_recordings() {
        let files = recordings_on_disk();
        let (Ok(r), Some(roster)) = (lobby(), install_roster()) else {
            println!("skipping: no lobby or no install");
            return;
        };
        if files.is_empty() {
            println!("skipping: no .rcx on disk");
            return;
        }
        let template = r.scene(None).buildings[0];
        drop(r);
        let roster = std::sync::Arc::new(roster);
        let terrain = TerrainScene {
            width: 256,
            height: 256,
            ..TerrainScene::default()
        };
        let c = crate::pinned_camera();
        let (mut minted, mut all) = (0, 0);
        let mut kinds: std::collections::BTreeMap<(&str, usize), usize> = Default::default();
        let mut written = 0;
        for f in &files {
            let rec = rondata::recgame::read(&f.display().to_string()).unwrap();
            let mut last = LastSent::new();
            // The minted turns, and the recorded package each stands for.
            let mut ours: Vec<(Order, Vec<u8>)> = Vec::new();
            for p in &rec.packages {
                let Ok(cmds) = rondata::commands::decode(&p.data) else {
                    continue;
                };
                let kept: Vec<Command> = cmds
                    .iter()
                    .filter(|c| rondata::input::is_input(c))
                    .cloned()
                    .collect();
                let (who, objects, before) = behind(&last, p.play, &kept);
                note_sent(&mut last, p.play, &kept);
                let heads: Vec<&Command> = kept
                    .iter()
                    .filter(|c| !matches!(c, Command::Group { .. }))
                    .collect();
                if heads.len() < 2 {
                    continue;
                }
                all += 1;
                *kinds.entry((heads[0].name(), heads.len())).or_default() += 1;
                assert!(
                    heads.iter().all(|h| h.name() == heads[0].name()),
                    "one kind a turn"
                );
                let ids: Vec<ObjId> = objects.iter().map(|&o| ObjId { who, o }).collect();
                let (scene, building, mods) = match *heads[0] {
                    Command::QueueUp { unit, .. } => {
                        let at = roster.trained_at(unit).first().copied();
                        let buildings = ids
                            .iter()
                            .map(|&id| {
                                let mut b = template;
                                (b.id, b.owner, b.ty, b.active) = (id, who, at, true);
                                b
                            })
                            .collect();
                        let s = Scene {
                            buildings,
                            ..Scene::default()
                        };
                        (s, true, Mods::default())
                    }
                    Command::GatherPoint { add_to_end, .. } => {
                        let buildings = ids
                            .iter()
                            .map(|&id| {
                                let mut b = template;
                                (b.id, b.owner, b.ty, b.active) = (id, who, None, true);
                                b
                            })
                            .collect();
                        let s = Scene {
                            buildings,
                            ..Scene::default()
                        };
                        let shift = add_to_end != 0;
                        (
                            s,
                            true,
                            Mods {
                                shift,
                                ..Mods::default()
                            },
                        )
                    }
                    Command::MoveTo { queued, .. } => {
                        let units = objects
                            .iter()
                            .map(|&o| unit(who, o, SCOUT_TYPE, 0, 0, 0))
                            .collect();
                        let s = Scene {
                            units,
                            ..Scene::default()
                        };
                        let shift = i32::from(queued) == QUEUE_LAST;
                        let alt = i32::from(queued) == QUEUE_FIRST;
                        (
                            s,
                            false,
                            Mods {
                                shift,
                                alt,
                                ..Mods::default()
                            },
                        )
                    }
                    _ => panic!("a turn of a kind this test does not press: {kept:?}"),
                };
                let v = view(&c, &scene, &terrain);
                let mut input = Input {
                    roster: roster.clone(),
                    building,
                    selection: ids.clone(),
                    mods,
                    ..Input::new(who)
                };
                input.sent = sent_for(&before, &scene);
                let events: Vec<Event> = heads
                    .iter()
                    .map(|h| match **h {
                        Command::QueueUp { unit, .. } => {
                            let b = crate::hud::roster_menu(&input, &scene, None, HUD_SIZE)
                                .into_iter()
                                .find(|b| b.ev == Event::Train(unit))
                                .expect("the roster offers the type");
                            let (x, y) = b.rect.centre();
                            Event::Press { x, y }
                        }
                        Command::GatherPoint { x, y, .. } => Event::MapRightPress { x, y },
                        Command::MoveTo { to_x, to_y, .. } => {
                            Event::MapRightPress { x: to_x, y: to_y }
                        }
                        _ => unreachable!(),
                    })
                    .collect();
                let got = crate::hud::turn(&mut input, HUD_SIZE, p.frame, &events, &v);
                let bytes = rondata::commands::encode(&kept);
                let whole = got.as_ref().is_some_and(|o| {
                    o.2 == kept
                        && rondata::commands::encode(&o.2) == bytes
                        && (kept.len() < cmds.len() || bytes == p.data)
                });
                println!(
                    "TURN {} f{} play {} {} issues of {}: minted {whole}",
                    f.file_name().unwrap().to_string_lossy(),
                    p.frame,
                    p.play,
                    heads.len(),
                    heads[0].name()
                );
                if whole {
                    minted += 1;
                    ours.push((got.unwrap(), bytes));
                }
            }
            if ours.is_empty() {
                continue;
            }
            // The file: the turns on their frames, the paused ones inside
            // the recorded pause, over the recording's own header.
            let mut script: Vec<Order> = Vec::new();
            for (o, _) in &ours {
                let held = script.last().is_some_and(|l| l.0 == o.0);
                if held {
                    let first = script.iter().rposition(|l| l.0 != o.0).map_or(0, |k| k + 1);
                    if !matches!(script[first].2[..], [Command::Pause { state: 1 }]) {
                        script.insert(first, (o.0, o.1, vec![Command::Pause { state: 1 }]));
                    }
                }
                if let Some(l) = script.last()
                    && l.0 != o.0
                    && script
                        .iter()
                        .any(|q| q.0 == l.0 && matches!(q.2[..], [Command::Pause { state: 1 }]))
                {
                    let f0 = l.0;
                    script.push((f0, o.1, vec![Command::Pause { state: 0 }]));
                }
                script.push(o.clone());
            }
            let lastf = rec.packages.last().unwrap().frame;
            let mut w = rec.clone();
            w.packages = rondata::recgame::minted_packages(
                Input::default().leader_options(0),
                &script,
                lastf,
            )
            .unwrap();
            let back = rondata::recgame::parse(&rondata::recgame::write(&w), "turns").unwrap();
            for (o, bytes) in &ours {
                let found = back
                    .packages
                    .iter()
                    .any(|q| q.frame == o.0 && q.play == o.1 && q.data == *bytes);
                written += usize::from(found);
            }
        }
        let kinds: Vec<((&str, usize), usize)> = kinds.into_iter().collect();
        println!("TURNS {minted} of {all}; {kinds:?}; written {written}");
        assert_eq!(kinds, TURN_KINDS);
        assert_eq!((minted, all), INPUT_TURNS);
        assert_eq!(written, TURNS_WRITTEN);
    }

    /// **The census of `gather_point` on an object** (item 4301,
    /// `docs/CLIENT_INPUT.md` §11.4): every recorded `gather_point` with
    /// `action` 1 to 3, by `(action, orders in its package)`. All 24 are
    /// the 2017 sample's, slot 3's on play 0, each with its `group` alone,
    /// so none is a two-issue turn's. Seen red pinned with action 3's 12
    /// as 11.
    const GATHER_POINTS_ON_AN_OBJECT: &[((i32, usize), usize)] =
        &[((1, 2), 9), ((2, 2), 3), ((3, 2), 12)];
    /// Of them, the heads the layer mints byte for byte from a scene that
    /// holds the object the head names: for actions 1 and 2 a building at
    /// the recorded point, the issuer's own or an enemy's; for action 3 one
    /// of the issuer's units whose captain is the recorded `x`, `y` being
    /// the issuer's slot. Seen red with the action-3 arm minting
    /// `(o, who)` of the unit pressed in place of its captain: 12 of 24,
    /// every action 3 parted (the unit is `o` 900, its captain the
    /// recorded `x`).
    const GATHER_POINTS_MINTED: (usize, usize) = (24, 24);

    #[test]
    fn gather_points_on_an_object_on_disk() {
        let files = recordings_on_disk();
        let Ok(r) = lobby() else {
            println!("skipping: no lobby");
            return;
        };
        if files.is_empty() {
            println!("skipping: no .rcx on disk");
            return;
        }
        let terrain = r.terrain();
        let template = r.scene(None).buildings[0];
        drop(r);
        let c = crate::pinned_camera();
        let mut by: std::collections::BTreeMap<(i32, usize), usize> = Default::default();
        let (mut minted, mut all) = (0, 0);
        for f in &files {
            let rec = rondata::recgame::read(&f.display().to_string()).unwrap();
            let who = console_who(&rec);
            for p in &rec.packages {
                let Ok(cmds) = rondata::commands::decode(&p.data) else {
                    continue;
                };
                let kept: Vec<Command> =
                    cmds.into_iter().filter(rondata::input::is_input).collect();
                for c0 in &kept {
                    let &Command::GatherPoint {
                        x,
                        y,
                        action,
                        add_to_end,
                    } = c0
                    else {
                        continue;
                    };
                    if action == 0 {
                        continue;
                    }
                    *by.entry((action, kept.len())).or_default() += 1;
                    all += 1;
                    // The selection: a building of the issuer's that trains,
                    // hidden from the looks so the press names only the
                    // object below.
                    let mut sel = template;
                    (sel.id, sel.owner) = (ObjId { who, o: 2000 }, who);
                    (sel.trains, sel.seen) = (true, false);
                    let enemy_slot = (who + 1) % 5;
                    let mut scene = Scene {
                        buildings: vec![sel],
                        leaders: vec![sim::observe::LeaderScene {
                            who,
                            enemies: vec![enemy_slot],
                            ..leader_scene(who)
                        }],
                        ..Scene::default()
                    };
                    let at = if action == 3 {
                        let mut u = unit(who, 900, 0x36, 0, 0, 0);
                        u.captain = x as i16;
                        u.pos = template.pos;
                        let at = (u.pos, u.z);
                        scene.units.push(u);
                        at
                    } else {
                        let mut b = template;
                        let owner = if action == 2 { enemy_slot } else { who };
                        (b.id, b.owner, b.seen, b.trains) = (
                            ObjId {
                                who: owner,
                                o: 2001,
                            },
                            owner,
                            true,
                            false,
                        );
                        b.pos = Pos { x, y };
                        scene.buildings.push(b);
                        (b.pos, b.z)
                    };
                    let v = view(&c, &scene, &terrain);
                    let (px, py) = v.pixel(at.0.x, at.0.y, at.1);
                    let input = Input {
                        selection: vec![sel.id],
                        building: true,
                        mods: Mods {
                            shift: add_to_end != 0,
                            ..Mods::default()
                        },
                        // Seated on the slot, as the other census mints
                        // are: the layer's `player` is the slot its looks
                        // read, and the head carries no play.
                        ..Input::new(who)
                    };
                    let got = input
                        .clone()
                        .event(p.frame, Event::RightPress { x: px, y: py }, &v)
                        .map(|o| o.2);
                    let whole = got.as_ref().is_some_and(|g| {
                        g.len() == 2
                            && g[1] == *c0
                            && rondata::commands::encode(&g[1..])
                                == rondata::commands::encode(std::slice::from_ref(c0))
                    });
                    println!(
                        "GP {} f{} play {} who {who} ({x}, {y}) action {action}; {} in the package; minted {whole}",
                        f.file_name().unwrap().to_string_lossy(),
                        p.frame,
                        p.play,
                        kept.len()
                    );
                    if action == 3 {
                        assert_eq!(y, i32::from(who), "action 3 names the issuer's slot");
                    }
                    minted += usize::from(whole);
                }
            }
        }
        let by: Vec<((i32, usize), usize)> = by.into_iter().collect();
        println!("GP BY {by:?}; minted {minted} of {all}");
        assert_eq!(by, GATHER_POINTS_ON_AN_OBJECT);
        assert_eq!((minted, all), GATHER_POINTS_MINTED);
    }

    /// A leader record with nothing in it but its slot.
    fn leader_scene(who: u8) -> sim::observe::LeaderScene {
        sim::observe::LeaderScene {
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
            enemies: vec![],
        }
    }

    /// **The gather arm on an object** (item 4301, `docs/CLIENT_INPUT.md`
    /// §11.4): with a building selected that trains, a right press on
    /// one's own building is `action` 1 at its position, on an enemy's 2,
    /// on another slot's at peace 1; on one's own unit 3 with its captain
    /// and the slot; on an enemy unit the ground, `action` 0, since the
    /// second look is the slot's own. Refused: the selected building
    /// itself (unless the profile's `prefs & 0x202`), and an active
    /// University of one's own for a city (an unfinished one is named).
    #[test]
    fn minted_a_gather_point_on_an_object_as_the_original() {
        let Ok(r) = lobby() else {
            println!("skipping: no lobby");
            return;
        };
        let terrain = r.terrain();
        let template = r.scene(None).buildings[0];
        drop(r);
        let c = crate::pinned_camera();
        let mut sel = template;
        (sel.id, sel.owner, sel.trains, sel.village) = (ObjId { who: 0, o: 2000 }, 0, true, true);
        let far = Pos {
            x: template.pos.x + 6 * 768,
            y: template.pos.y,
        };
        let mut other = template;
        (other.trains, other.village, other.pos) = (false, false, far);
        let building = |who: u8, o: i16| {
            let mut b = other;
            (b.id, b.owner) = (ObjId { who, o }, who);
            b
        };
        let leaders = vec![sim::observe::LeaderScene {
            enemies: vec![1],
            ..leader_scene(0)
        }];
        let press = |scene: &Scene, prefs: u32, mods: Mods, at: (Pos, i32)| {
            let v = view(&c, scene, &terrain);
            let (px, py) = v.pixel(at.0.x, at.0.y, at.1);
            let input = Input {
                selection: vec![sel.id],
                building: true,
                mods,
                prefs: rondata::prefs::Prefs {
                    prefs,
                    ..rondata::prefs::Prefs::FRESH
                },
                ..Input::new(0)
            };
            input
                .clone()
                .event(1, Event::RightPress { x: px, y: py }, &v)
                .map(|o| o.2[1].clone())
        };
        let fresh = rondata::prefs::DEFAULT_PREFS;
        let none = Mods::default();
        let gp = |x: i32, y: i32, action: i32, add_to_end: i32| Command::GatherPoint {
            x,
            y,
            action,
            add_to_end,
        };
        let with = |extra: Vec<sim::observe::BuildingScene>,
                    units: Vec<sim::observe::UnitScene>| {
            let mut buildings = vec![sel];
            buildings.extend(extra);
            Scene {
                buildings,
                units,
                leaders: leaders.clone(),
                ..Scene::default()
            }
        };
        let on = |b: &sim::observe::BuildingScene| (b.pos, b.z);
        // A building of one's own, an enemy's, and one at peace.
        for (who, action) in [(0, 1), (1, 2), (2, 1)] {
            let b = building(who, 2001);
            let s = with(vec![b], vec![]);
            assert_eq!(
                press(&s, fresh, none, on(&b)),
                Some(gp(far.x, far.y, action, 0)),
                "slot {who}"
            );
        }
        // Shift or ctrl: `add_to_end` 1 on every arm.
        let b = building(1, 2001);
        let s = with(vec![b], vec![]);
        for mods in [
            Mods {
                shift: true,
                ..none
            },
            Mods { ctrl: true, ..none },
        ] {
            assert_eq!(press(&s, fresh, mods, on(&b)), Some(gp(far.x, far.y, 2, 1)));
        }
        // One's own unit: its captain and the slot. An enemy's: the ground.
        let mut u = unit(0, 31, 0x36, far.x, 0, 0);
        (u.pos, u.captain) = (far, 27);
        let s = with(vec![], vec![u.clone()]);
        assert_eq!(press(&s, fresh, none, (u.pos, u.z)), Some(gp(27, 0, 3, 0)));
        let mut e = u.clone();
        (e.id, e.owner) = (ObjId { who: 1, o: 31 }, 1);
        let at = (e.pos, e.z);
        let s = with(vec![], vec![e]);
        let got = press(&s, fresh, none, at);
        assert!(
            matches!(got, Some(Command::GatherPoint { action: 0, .. })),
            "{got:?}"
        );
        // A building beats a unit under the same press.
        let b = building(0, 2001);
        let s = with(vec![b], vec![u]);
        assert_eq!(press(&s, fresh, none, on(&b)), Some(gp(far.x, far.y, 1, 0)));
        // The selected building itself: refused, unless `prefs & 0x202`.
        let s = with(vec![], vec![]);
        assert_eq!(press(&s, fresh, none, on(&sel)), None);
        for bit in [0x2, 0x200] {
            assert_eq!(
                press(&s, fresh | bit, none, on(&sel)),
                Some(gp(sel.pos.x, sel.pos.y, 1, 0)),
                "prefs {bit:#x}"
            );
        }
        // One's own active University, for a city: refused; unfinished, or
        // with no city selected, named.
        let mut uni = building(0, 2001);
        uni.university = true;
        let s = with(vec![uni], vec![]);
        assert_eq!(press(&s, fresh, none, on(&uni)), None);
        uni.active = false;
        let s = with(vec![uni], vec![]);
        assert_eq!(
            press(&s, fresh, none, on(&uni)),
            Some(gp(far.x, far.y, 1, 0))
        );
        uni.active = true;
        let mut s = with(vec![uni], vec![]);
        s.buildings[0].village = false;
        assert_eq!(
            press(&s, fresh, none, on(&uni)),
            Some(gp(far.x, far.y, 1, 0))
        );
    }

    /// A command's queue position, `QueuePos`, for the kinds that carry
    /// one: `QUEUE_FIRST` 0, `QUEUE_LAST` 1, `QUEUE_NEW` 2.
    fn queue_of(c: &Command) -> Option<i32> {
        Some(match *c {
            Command::Form { queued, .. }
            | Command::Attack { queued, .. }
            | Command::SiegeAttack { queued, .. }
            | Command::SwarmAround { queued, .. }
            | Command::LaunchPatrol { queued, .. }
            | Command::BoardShip { queued, .. }
            | Command::Repair { queued, .. }
            | Command::Trade { queued, .. }
            | Command::CityGather { queued, .. }
            | Command::Gather { queued, .. }
            | Command::Garrison { queued, .. }
            | Command::Build { queued, .. }
            | Command::Follow { queued, .. }
            | Command::Guard { queued, .. } => queued,
            Command::MoveTo { queued, .. }
            | Command::MoveNear { queued, .. }
            | Command::AttackGround { queued, .. }
            | Command::Patrol { queued, .. } => i32::from(queued),
            _ => return None,
        })
    }

    /// **The modifiers' queue** (item 4293, `docs/CLIENT_INPUT.md` §33):
    /// every arm that takes a queue, under each modifier the issuers read.
    /// The dispatch's arms (the five objects and the ground): `QUEUE_NEW`,
    /// `QUEUE_LAST` with shift, `QUEUE_FIRST` with shift and alt, and alt
    /// ignored under `prefs & 0x400`; the minimap's: `QUEUE_FIRST` with alt
    /// alone; ctrl on a unit's right press mints nothing. The placement:
    /// shift or ctrl keeps it armed except for a wonder. The ground's
    /// gather point: `add_to_end` with shift or ctrl.
    #[test]
    fn minted_shift_edges() {
        let Ok(r) = lobby() else {
            println!("skipping: no lobby");
            return;
        };
        // The lobby's ground, and no object on it but the gather point's
        // building below.
        let scene = Scene::default();
        let terrain = r.terrain();
        let c = crate::pinned_camera();
        let v = view(&c, &scene, &terrain);
        let none = Mods::default();
        let shift = Mods {
            shift: true,
            ..none
        };
        let alt = Mods { alt: true, ..none };
        let ctrl = Mods { ctrl: true, ..none };
        let shift_alt = Mods { alt: true, ..shift };
        let shift_ctrl = Mods {
            ctrl: true,
            ..shift
        };
        let ctrl_alt = Mods { alt: true, ..ctrl };
        let unit = |mods: Mods| Input {
            selection: vec![ObjId { who: 0, o: 5 }],
            mods,
            ..Input::new(0)
        };
        let (enemy, own) = (ObjId { who: 1, o: 7 }, ObjId { who: 0, o: 2000 });
        let hits = [
            (
                Hit::Enemy {
                    id: enemy,
                    unit: true,
                },
                true,
            ),
            (
                Hit::Enemy {
                    id: enemy,
                    unit: true,
                },
                false,
            ),
            (Hit::Friend(ObjId { who: 0, o: 6 }), true),
            (
                Hit::Own {
                    id: own,
                    active: false,
                    damaged: false,
                    room: None,
                },
                true,
            ),
            (
                Hit::Own {
                    id: own,
                    active: true,
                    damaged: false,
                    room: Some(true),
                },
                true,
            ),
            (
                Hit::Own {
                    id: own,
                    active: true,
                    damaged: true,
                    room: Some(false),
                },
                true,
            ),
        ];
        let ground = Event::RightPress { x: 900, y: 500 };
        let map = Event::MapRightPress { x: 100, y: 100 };
        for (mods, q, mapq) in [
            (none, QUEUE_NEW, QUEUE_NEW),
            (shift, QUEUE_LAST, QUEUE_LAST),
            (shift_alt, QUEUE_FIRST, QUEUE_LAST),
            (alt, QUEUE_NEW, QUEUE_FIRST),
        ] {
            let input = unit(mods);
            for (h, attacks) in hits {
                let cmd = input.on_object(h, attacks).unwrap();
                assert_eq!(queue_of(&cmd), Some(q), "{mods:?} {cmd:?}");
            }
            let got = unit(mods).event(1, ground, &v).unwrap().2;
            assert_eq!(queue_of(&got[1]), Some(q), "{mods:?} the ground");
            let got = unit(mods).event(1, map, &v).unwrap().2;
            assert_eq!(queue_of(&got[1]), Some(mapq), "{mods:?} the minimap");
        }
        // Alt is not read under the profile's `0x400`.
        let mut input = unit(shift_alt);
        input.prefs.prefs |= 0x400;
        let got = input.event(1, ground, &v).unwrap().2;
        assert_eq!(queue_of(&got[1]), Some(QUEUE_LAST));
        // Ctrl is the attack-move (item 4297, §35): the dispatch's queue
        // on the ground, the minimap's on the map; with alt on the ground
        // it is `MOVE_TO` with the formation.
        for (mods, q, mapq, orders) in [
            (ctrl, QUEUE_NEW, QUEUE_NEW, ATTACK_TO),
            (shift_ctrl, QUEUE_LAST, QUEUE_LAST, ATTACK_TO),
            (ctrl_alt, QUEUE_NEW, QUEUE_FIRST, 1),
        ] {
            let got = unit(mods).event(1, ground, &v).unwrap().2;
            assert_eq!(queue_of(&got[1]), Some(q), "{mods:?} the ground");
            assert!(
                matches!(got[1], Command::MoveTo { orders: o, .. } if o == orders),
                "{mods:?} {:?}",
                got[1]
            );
            let got = unit(mods).event(1, map, &v).unwrap().2;
            assert_eq!(queue_of(&got[1]), Some(mapq), "{mods:?} the minimap");
            assert!(
                matches!(
                    got[1],
                    Command::MoveTo {
                        orders: ATTACK_TO,
                        ..
                    }
                ),
                "{mods:?} {:?}",
                got[1]
            );
        }
        // The placement: the queue, and whether the type stays armed.
        let farm = BUILD_KEYS[0].1;
        let wonder = 0x20e;
        assert!(!sim::core::type_index::is_wonder_type(farm));
        for (ty, mods, q, armed) in [
            (farm, none, QUEUE_NEW, false),
            (farm, alt, QUEUE_FIRST, false),
            (farm, shift, QUEUE_LAST, true),
            (farm, shift_alt, QUEUE_LAST, true),
            (farm, ctrl, QUEUE_FIRST, true),
            (farm, shift_ctrl, QUEUE_LAST, true),
            (wonder, none, QUEUE_NEW, false),
            (wonder, ctrl, QUEUE_NEW, false),
            (wonder, ctrl_alt, QUEUE_FIRST, false),
            (wonder, shift, QUEUE_LAST, false),
        ] {
            let mut input = unit(mods);
            input.event(1, Event::Place(ty), &v);
            assert_eq!(input.placing, Some(ty));
            let got = input
                .event(1, Event::Press { x: 900, y: 500 }, &v)
                .unwrap()
                .2;
            assert!(matches!(got[1], Command::Build { build, .. } if build == ty));
            assert_eq!(queue_of(&got[1]), Some(q), "{ty} {mods:?}");
            assert_eq!(input.placing.is_some(), armed, "{ty} {mods:?}");
        }
        // The ground's gather point: `add_to_end` with shift or ctrl. The
        // building is the lobby's, trained at, its anchor far from the
        // press.
        drop(r);
        let mut b = queue_building(0, 2000, 0, 0);
        b.trains = true;
        let trains = Scene {
            buildings: vec![b],
            ..Scene::default()
        };
        let (bx, by) = view(&c, &trains, &terrain).pixel(b.pos.x, b.pos.y, b.z);
        assert!((bx - 900).abs() + (by - 500).abs() > 4 * PICK_RADIUS);
        for (mods, add) in [(none, 0), (alt, 0), (shift, 1), (ctrl, 1)] {
            let input = Input {
                selection: vec![own],
                building: true,
                mods,
                ..Input::new(0)
            };
            let add_to_end = |s: &Scene| {
                let v = view(&c, s, &terrain);
                input.clone().event(1, ground, &v).map(|o| o.2)
            };
            let Some(got) = add_to_end(&trains) else {
                panic!("no gather point minted under {mods:?}");
            };
            assert!(
                matches!(got[1], Command::GatherPoint { add_to_end, .. } if add_to_end == add),
                "{mods:?} {got:?}"
            );
        }
    }

    /// **`QUEUED_ORDERS`** (item 4293, `docs/CLIENT_INPUT.md` §33): every
    /// package on disk whose head carries a queue other than `QUEUE_NEW`,
    /// its orders (the `group` with them) by the head's kind, its queue and
    /// the census's verdict. Before the item every one but the two-issue
    /// turns was the miss "queued 1: the shift held", 200 orders; the
    /// shift's queue is now the layer's, and four `build` orders fall to
    /// the dragged line, the layer's form since item 4304 (§38). No
    /// recording carries `QUEUE_FIRST`. Seen red pinned with the miss: the
    /// build parted it.
    const QUEUED_ORDERS: &[(&str, i32, &str, usize)] = &[
        ("build", 1, "INPUT_HAND_PLACEMENTS", 6),
        ("gather", 1, "INPUT_HAND_OBJECT_ORDERS", 4),
        ("move_to", 1, "INPUT_HAND_MOVES", 170),
        ("move_to", 1, "INPUT_MAIN_MOVES", 16),
        ("move_to", 1, "INPUT_TURNS", 12),
        ("swarm_around", 1, "INPUT_HAND_OBJECT_ORDERS", 4),
    ];

    /// The census of the shifted orders (item 4293's first act).
    #[test]
    fn the_queued_orders_of_the_recordings_on_disk() {
        let files = recordings_on_disk();
        if files.is_empty() || lobby().is_err() {
            println!("skipping: no .rcx on disk or no lobby");
            return;
        }
        let roster = install_roster().unwrap_or_default();
        let terrain = TerrainScene::default();
        let c = crate::pinned_camera();
        let scene = Scene::default();
        let v = view(&c, &scene, &terrain);
        let mut by: std::collections::BTreeMap<(&str, i32, &str), usize> = Default::default();
        for f in &files {
            let rec = rondata::recgame::read(&f.display().to_string()).unwrap();
            let who = console_who(&rec);
            let mut pauser = Input::new(0);
            for p in &rec.packages {
                let Ok(cmds) = rondata::commands::decode(&p.data) else {
                    continue;
                };
                let kept: Vec<Command> =
                    cmds.into_iter().filter(rondata::input::is_input).collect();
                let Some((head, q)) = kept
                    .iter()
                    .find_map(|c| queue_of(c).filter(|&q| q != QUEUE_NEW).map(|q| (c, q)))
                else {
                    continue;
                };
                let arm = match census_verdict(
                    &kept,
                    p.frame,
                    p.play as u8,
                    who,
                    &mut pauser,
                    &v,
                    &roster,
                ) {
                    Verdict::Shown(pin) => pin,
                    Verdict::Form => "form",
                    Verdict::Miss(arm, _) => arm,
                };
                *by.entry((head.name(), q, arm)).or_default() += kept.len();
            }
        }
        let by: Vec<(&str, i32, &str, usize)> =
            by.into_iter().map(|((k, q, a), n)| (k, q, a, n)).collect();
        println!("QUEUED {by:?}");
        assert_eq!(by, QUEUED_ORDERS);
    }

    /// Whether `angle` is one `find_angle@0092d130` gives for some leg
    /// pair (item 4295, `docs/CLIENT_INPUT.md` §34.2): every value of the
    /// ratio `t` (0 to `0x4000`) in every octant, which a long leg of
    /// `0x4000` and a short one of 0 to `0x4000` reach exactly.
    fn of_find_angle(angle: i32) -> bool {
        static IMAGE: std::sync::OnceLock<std::collections::HashSet<i32>> =
            std::sync::OnceLock::new();
        IMAGE
            .get_or_init(|| {
                const L: i32 = 0x4000;
                (-L..=L)
                    .flat_map(|d| [(L, d), (-L, d), (d, L), (d, -L)])
                    .chain([(0, 0)])
                    .map(|(dx, dy)| sim::movement::find_angle(dx, dy).0)
                    .collect()
            })
            .contains(&angle)
    }

    /// A right drag's fields past its point and angle: `(form, width,
    /// queued, orders, disembark)`.
    type DragFields = (i8, i8, i8, i8, i8);
    /// **`RIGHT_DRAG_FIELDS`** (item 4295, `docs/CLIENT_INPUT.md` §34.2): every
    /// recorded `move_to` that carries `set_angle` 1, by its other fields
    /// `(form, width, queued, orders, disembark)`. All 70 packages are the
    /// 2017 sample's; the 140 orders the census of §30 missed are these
    /// with their `group`s. Seen red pinned with `queued` 1.
    const RIGHT_DRAG_FIELDS: &[(DragFields, usize)] =
        &[((0, 50, 2, 1, 0), 58), ((9, 50, 2, 1, 0), 12)];
    /// The angles: (of `find_angle`'s image, exactly south `0x80000000`,
    /// all). The low sixteen bits of every one are 0, as `find_angle`'s
    /// `& 0xffffc000` and `<< 2` leave them.
    const RIGHT_DRAG_ANGLES: (usize, usize, usize) = (70, 13, 70);
    /// The angles by octant, `angle as u32 >> 29`: 0 is north to
    /// north-east, 4 south to south-west.
    const RIGHT_DRAG_OCTANTS: &[(u32, usize)] =
        &[(0, 6), (2, 2), (3, 7), (4, 35), (5, 12), (6, 7), (7, 1)];
    /// The selections, by the objects the package's `group` lists; 0 is
    /// the wire's `num = 0`, the last selection sent again (§20).
    const RIGHT_DRAG_GROUPS: &[(usize, usize)] = &[
        (0, 28),
        (1, 19),
        (2, 8),
        (3, 4),
        (7, 2),
        (11, 3),
        (21, 1),
        (28, 1),
        (33, 1),
        (35, 1),
        (36, 1),
        (38, 1),
    ];

    /// The census of the right drags (item 4295's first act).
    #[test]
    fn the_right_drags_of_the_recordings_on_disk() {
        let files = recordings_on_disk();
        if files.is_empty() {
            println!("skipping: no .rcx on disk");
            return;
        }
        let mut fields: std::collections::BTreeMap<DragFields, usize> = Default::default();
        let mut octants: std::collections::BTreeMap<u32, usize> = Default::default();
        let mut groups: std::collections::BTreeMap<usize, usize> = Default::default();
        let (mut image, mut south, mut all) = (0, 0, 0);
        let mut files_with = std::collections::BTreeSet::new();
        for f in &files {
            let rec = rondata::recgame::read(&f.display().to_string()).unwrap();
            for p in &rec.packages {
                let Ok(cmds) = rondata::commands::decode(&p.data) else {
                    continue;
                };
                let kept: Vec<Command> =
                    cmds.into_iter().filter(rondata::input::is_input).collect();
                let mut listed = 0;
                for c in &kept {
                    match *c {
                        Command::Group { ref objects, .. } => listed = objects.len(),
                        Command::MoveTo {
                            set_angle,
                            angle,
                            orders,
                            queued,
                            form,
                            width,
                            disembark,
                            ..
                        } if set_angle != 0 => {
                            assert_eq!(set_angle, 1, "a set_angle other than 1");
                            *fields
                                .entry((form, width, queued, orders, disembark))
                                .or_default() += 1;
                            *octants.entry(angle as u32 >> 29).or_default() += 1;
                            *groups.entry(listed).or_default() += 1;
                            image += usize::from(of_find_angle(angle));
                            south += usize::from(angle == i32::MIN);
                            all += 1;
                            files_with.insert(f.file_name().unwrap().to_string_lossy().to_string());
                        }
                        _ => {}
                    }
                }
            }
        }
        println!("RIGHT DRAGS in {files_with:?}: {fields:?} {octants:?} {groups:?}");
        assert_eq!(files_with.len(), 1, "one recording carries the drags");
        let fields: Vec<_> = fields.into_iter().collect();
        assert_eq!(fields, RIGHT_DRAG_FIELDS);
        assert_eq!((image, south, all), RIGHT_DRAG_ANGLES);
        let octants: Vec<_> = octants.into_iter().collect();
        assert_eq!(octants, RIGHT_DRAG_OCTANTS);
        let groups: Vec<_> = groups.into_iter().collect();
        assert_eq!(groups, RIGHT_DRAG_GROUPS);
    }

    /// An attack-move's fields past its point: `(set_angle, angle, form,
    /// width, queued, disembark)`.
    type AttackMoveFields = (i32, i32, i8, i8, i8, i8);
    /// **`ATTACK_MOVE_FIELDS`** (item 4297, `docs/CLIENT_INPUT.md` §35.2):
    /// every recorded `move_to` whose `orders` is not `MOVE_TO`, by its
    /// `orders` and its other fields. Every issuer of `ATTACK_TO` writes
    /// `(0, 0, −1, −1, queue, 0)`: `Console::execute_at_cursor@007c6630`
    /// under ctrl, `WorldMap::on_right_up@008c7050` under ctrl, and the
    /// ATTACK_TO option's spot through the dispatch.
    /// All 27 packages are the 2017 sample's; with their `group`s they
    /// are the 54 orders §30.2 missed. Seen red pinned with `queued` 1.
    const ATTACK_MOVE_FIELDS: &[((i8, AttackMoveFields), usize)] =
        &[((2, (0, 0, -1, -1, 2, 0)), 27)];
    /// Of the 27 recorded heads, those the minimap mints under ctrl at the
    /// recorded point, byte for byte. Seen red with the layer's ctrl
    /// minting nothing (item 4293's arm): 0.
    const ATTACK_MOVE_MINTED: usize = 27;
    /// The selections, by the objects the package's `group` lists; 0 is
    /// the wire's `num = 0`, the last selection sent again (§20).
    const ATTACK_MOVE_GROUPS: &[(usize, usize)] = &[
        (0, 4),
        (1, 3),
        (2, 1),
        (5, 1),
        (7, 1),
        (8, 3),
        (10, 1),
        (11, 1),
        (17, 1),
        (20, 2),
        (21, 3),
        (23, 1),
        (32, 3),
        (37, 2),
    ];

    /// The census of the attack-moves (item 4297's first act).
    #[test]
    fn the_attack_moves_of_the_recordings_on_disk() {
        let files = recordings_on_disk();
        if files.is_empty() {
            println!("skipping: no .rcx on disk");
            return;
        }
        // The minimap reaches any world point, so each recorded head is
        // minted there under ctrl, byte for byte; the map is sized past the
        // sample's points (its own size is not on disk).
        let terrain = TerrainScene {
            width: 256,
            height: 256,
            ..TerrainScene::default()
        };
        let c = crate::pinned_camera();
        let scene = Scene::default();
        let v = view(&c, &scene, &terrain);
        let mut minted = 0;
        let mut fields: std::collections::BTreeMap<(i8, AttackMoveFields), usize> =
            Default::default();
        let mut groups: std::collections::BTreeMap<usize, usize> = Default::default();
        let mut files_with = std::collections::BTreeSet::new();
        for f in &files {
            let rec = rondata::recgame::read(&f.display().to_string()).unwrap();
            for p in &rec.packages {
                let Ok(cmds) = rondata::commands::decode(&p.data) else {
                    continue;
                };
                let kept: Vec<Command> =
                    cmds.into_iter().filter(rondata::input::is_input).collect();
                let mut listed = 0;
                for c in &kept {
                    match *c {
                        Command::Group { ref objects, .. } => listed = objects.len(),
                        Command::MoveTo {
                            to_x,
                            to_y,
                            set_angle,
                            angle,
                            orders,
                            queued,
                            form,
                            width,
                            disembark,
                        } if orders != 1 => {
                            println!(
                                "ATTACK MOVE {}: frame {} play {} heads {} to ({to_x}, {to_y}) \
                                 listed {listed} {c:?}",
                                f.file_name().unwrap().to_string_lossy(),
                                p.frame,
                                p.play,
                                kept.len()
                            );
                            *fields
                                .entry((orders, (set_angle, angle, form, width, queued, disembark)))
                                .or_default() += 1;
                            *groups.entry(listed).or_default() += 1;
                            let mut input = Input {
                                selection: vec![ObjId { who: 0, o: 5 }],
                                mods: Mods {
                                    ctrl: true,
                                    ..Mods::default()
                                },
                                ..Input::new(0)
                            };
                            let ours = input
                                .event(p.frame, Event::MapRightPress { x: to_x, y: to_y }, &v)
                                .map(|o| o.2[1].clone());
                            minted += usize::from(
                                ours.as_ref() == Some(c)
                                    && rondata::commands::encode(&[ours.unwrap()])
                                        == rondata::commands::encode(std::slice::from_ref(c)),
                            );
                            files_with.insert(f.file_name().unwrap().to_string_lossy().to_string());
                        }
                        _ => {}
                    }
                }
            }
        }
        println!("ATTACK MOVES in {files_with:?}: {fields:?} {groups:?}; minted {minted}");
        assert_eq!(
            files_with.len(),
            1,
            "one recording carries the attack-moves"
        );
        assert_eq!(
            minted, ATTACK_MOVE_MINTED,
            "each head, minted on the minimap"
        );
        let fields: Vec<_> = fields.into_iter().collect();
        assert_eq!(fields, ATTACK_MOVE_FIELDS);
        let groups: Vec<_> = groups.into_iter().collect();
        assert_eq!(groups, ATTACK_MOVE_GROUPS);
    }

    /// `find_angle`'s image, read by [`of_find_angle`]: every multiple of
    /// `0x10000` in the turn, and nothing else. One step of the ratio `t`
    /// moves the scaled value by at most about `0x2b4d`, under the mask's
    /// `0x4000`, so no multiple is skipped; the image is what the low
    /// sixteen bits being 0 says. Seen red pinned with a gap in the first
    /// octant: none.
    #[test]
    fn of_find_angle_reads_the_image() {
        for a in [0, 0x4000_0000, i32::MIN, -0x4000_0000] {
            assert!(of_find_angle(a), "{a:#x}");
        }
        assert!(!of_find_angle(1), "a low bit");
        assert!(!of_find_angle(0x8000), "a low bit");
        let reached = (0..0x10000_i64)
            .map(|k| (k << 16) as u32 as i32)
            .filter(|&a| of_find_angle(a))
            .count();
        assert_eq!(reached, 0x10000, "every multiple of 0x10000");
    }

    /// The install's [`Roster`], or `None` with no install.
    pub(crate) fn install_roster() -> Option<Roster> {
        let root = std::env::var_os("RON_INSTALL")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| "../../game".into());
        let install = rondata::Install::new(&root);
        if !install.looks_valid() {
            return None;
        }
        Some(Roster::new(&rondata::load::load(&install).ok()?))
    }

    /// **`PANEL_ROSTER`** (item 4290): of the types the census misses as
    /// "a type no key of ours binds", (`queue_up` types on a panel, of
    /// them; `build` types on the citizen's pages, of them). Seen red
    /// pinned (0, 0, 0, 0).
    const PANEL_ROSTER: (usize, usize, usize, usize) = (82, 82, 16, 16);
    /// And their orders, the package's `group` counted with its order:
    /// (`queue_up` orders on a panel, of them; `build`, of them). Item 4293
    /// adds the one shifted `build` package of a type no key binds, which
    /// the queue's miss hid until then: 42 → 44, its type already counted.
    /// Item 4302 adds the two turns of two `queue_up`s of type 69 (19159,
    /// 19175), which the writer's miss hid: 390 → 398, the type counted.
    /// Item 4304 adds the five dragged placements (§38), all of types no
    /// key binds, which the dragged line's miss hid: builds 44 → 54, and
    /// the Temple (437), placed only by a drag, a sixteenth type, 15 → 16.
    const PANEL_ROSTER_ORDERS: (usize, usize, usize, usize) = (398, 398, 54, 54);
    /// The types on no panel, by `TypeIndex`.
    const PANEL_ROSTER_OFF: &[i32] = &[];

    /// **The census of the panel's roster** (item 4290): every package the
    /// whole census misses for an unbound type, its type set against the
    /// panels the original offers it on.
    #[test]
    fn the_panel_roster_of_the_recordings_on_disk() {
        let files = recordings_on_disk();
        let root = std::env::var_os("RON_INSTALL")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| "../../game".into());
        let install = rondata::Install::new(&root);
        if files.is_empty() || lobby().is_err() || !install.looks_valid() {
            println!("skipping: no .rcx on disk, no lobby or no install");
            return;
        }
        let loaded = rondata::load::load(&install).unwrap();
        let names = crate::hud::Names::new(&loaded);
        let roster = Roster::new(&loaded);
        let terrain = TerrainScene::default();
        let c = crate::pinned_camera();
        let scene = Scene::default();
        let v = view(&c, &scene, &terrain);
        let mut by: std::collections::BTreeMap<(&str, i32), usize> = Default::default();
        for f in &files {
            let rec = rondata::recgame::read(&f.display().to_string()).unwrap();
            let who = console_who(&rec);
            let mut pauser = Input::new(0);
            for p in &rec.packages {
                let Ok(cmds) = rondata::commands::decode(&p.data) else {
                    continue;
                };
                let kept: Vec<Command> =
                    cmds.into_iter().filter(rondata::input::is_input).collect();
                if kept.is_empty() {
                    continue;
                }
                let keys = Roster::default();
                let verdict =
                    census_verdict(&kept, p.frame, p.play as u8, who, &mut pauser, &v, &keys);
                let ty = kept.iter().find_map(|c| match c {
                    Command::QueueUp { unit, .. } => Some(("queue_up", *unit)),
                    Command::Build { build, .. } => Some(("build", *build)),
                    _ => None,
                });
                if let (Verdict::Miss(arm, _), Some(k)) = (verdict, ty)
                    && arm.ends_with("a type no key of ours binds")
                {
                    *by.entry(k).or_default() += kept.len();
                }
            }
        }
        let (mut types, mut orders) = ((0, 0, 0, 0), (0, 0, 0, 0));
        let mut off = Vec::new();
        for (&(kind, ti), &n) in &by {
            let (on, panel) = if kind == "queue_up" {
                let at: Vec<String> = roster
                    .trained_at(ti)
                    .iter()
                    .map(|&b| names.builds[b].clone())
                    .collect();
                (!at.is_empty(), format!("{at:?}"))
            } else {
                let on = roster.builds.contains(&ti);
                (on, format!("the citizen's: {on}"))
            };
            println!(
                "ROSTER {kind} {ti} {:?}: {n} orders, on {panel}",
                names.of_type_index(ti)
            );
            let (t, o) = if kind == "queue_up" {
                ((&mut types.0, &mut types.1), (&mut orders.0, &mut orders.1))
            } else {
                ((&mut types.2, &mut types.3), (&mut orders.2, &mut orders.3))
            };
            *t.1 += 1;
            *o.1 += n;
            if on {
                *t.0 += 1;
                *o.0 += n;
            } else {
                off.push(ti);
            }
        }
        println!("ROSTER types {types:?}, orders {orders:?}, off {off:?}");
        assert_eq!(types, PANEL_ROSTER);
        assert_eq!(orders, PANEL_ROSTER_ORDERS);
        assert_eq!(off, PANEL_ROSTER_OFF);
    }

    /// `OPTION_TRAIN` and `OPTION_RESEARCH`, the two kinds of button the
    /// roster stands for (`OptionIndex` 47 and 48: the train arm's adds at
    /// `0x62a742`, and the Library's 35 research buttons on run1591).
    const PANEL_KINDS: [i32; 2] = [OPTION_TRAIN, OPTION_RESEARCH];

    /// **`PANEL_LEADER`** (item 4305): the panel the original offers a
    /// selected building against ours, over every building of every packet
    /// `tools/recomp/panels.py` has run `BuildOut::make_options@00629ed0`
    /// on. (buildings whose train and research types equal ours, buildings
    /// either side offers one on; types both offer, the original's, ours),
    /// each building's types counted once a building. Minted at its first
    /// count on the static roster of §32, which offers every type its
    /// building may ever queue: no building's panel equal, every type the
    /// original offers on ours, and ours seven times as many. Seen red
    /// pinned (0, 0, 0, 0, 0). The packets: run1591 at 6537 (the minted
    /// GreatLakes lobby our walk holds, 9 buildings) and run1640 at 6079
    /// (run1575's lobby at `DIFFICULTY 4`, tribes 10 and 13, 68). Item 4315
    /// re-minted it over the minted game's sixteen packets (frames 1 to
    /// 7195, 110 buildings), run1640 counted apart as
    /// [`PANEL_LEADER_UNHELD`].
    const PANEL_LEADER: (usize, usize, usize, usize, usize) = (0, 110, 1238, 1238, 1945);

    /// The packets of a lobby our walk does not hold, counted apart from
    /// [`PANEL_LEADER`] (item 4315): run1640, run1575's lobby at
    /// `DIFFICULTY 4`, whose panels can be set against ours only through
    /// state read off its packet.
    const PANEL_UNHELD: [&str; 1] = ["run1640"];
    /// **`PANEL_LEADER_UNHELD`** (item 4315): [`PANEL_LEADER`]'s count over
    /// the [`PANEL_UNHELD`] packets alone.
    const PANEL_LEADER_UNHELD: (usize, usize, usize, usize, usize) = (0, 68, 218, 218, 1938);

    /// The packets' panels, `~/ron-data/panels/*.tsv` (or
    /// `RON_PANELS_DIR`): per (file, slot, object), the building's type and
    /// the train and research types the original offered it, sorted.
    fn panels_on_disk() -> Vec<(String, i32, i32, i32, Vec<i32>)> {
        buttons_on_disk()
            .into_iter()
            .map(|(f, who, object, b, bs)| {
                let mut ts: Vec<i32> = bs.iter().map(|x| x.ty).collect();
                ts.dedup();
                (f, who, object, b, ts)
            })
            .collect()
    }

    /// **The `*.tsv` files of a shared oracle directory this tree counts**
    /// (item 4343; `rondata::corpus`, DECISIONS 132 §3): `~/ron-data/panels`,
    /// `pages`, `pages-standard` and `pages-turns` are shared by every
    /// worktree on the machine, so a file whose run (`run1652.tsv`) this
    /// tree's `docs/RUNS.md` heads no section for is skipped and printed,
    /// never counted: another lane's capture cannot move this tree's pins
    /// before its ledger and re-pins land. Sorted; `None` with no directory.
    fn tsvs_in_tree(dir: &std::path::Path) -> Option<Vec<std::path::PathBuf>> {
        let listed = rondata::corpus::listed(dir, rondata::corpus::Ledger::of_tree(), |p| {
            p.extension().is_some_and(|x| x == "tsv")
        })
        .ok()?;
        if !listed.unfiled.is_empty() {
            println!(
                "{}: skipping unfiled {}",
                dir.display(),
                listed.unfiled.join(", ")
            );
        }
        Some(listed.kept.iter().map(|n| dir.join(n)).collect())
    }

    /// [`panels_on_disk`] with each button whole (item 4326): its type,
    /// option and `disable`, the TSV's columns 8, 5 and 9, sorted.
    fn buttons_on_disk() -> Vec<(String, i32, i32, i32, Vec<Button>)> {
        let dir = std::env::var_os("RON_PANELS_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::Path::new(&std::env::var_os("HOME").unwrap_or_default())
                    .join("ron-data/panels")
            });
        let Some(files) = tsvs_in_tree(&dir) else {
            return Vec::new();
        };
        let mut out: std::collections::BTreeMap<(String, i32, i32), (i32, Vec<Button>)> =
            std::collections::BTreeMap::new();
        for f in files {
            let name = f.file_stem().unwrap().to_string_lossy().into_owned();
            for line in std::fs::read_to_string(&f).unwrap().lines() {
                if line.starts_with('#') || line.is_empty() {
                    continue;
                }
                let c: Vec<i32> = line
                    .split('\t')
                    .take(10)
                    .map(|v| v.parse().expect("a panels.py row"))
                    .collect();
                let (who, object, building, option, ty, disable) =
                    (c[1], c[2], c[3], c[4], c[7], c[8]);
                assert!(option != -1, "{name}: a run that did not return: {line}");
                let e = out
                    .entry((name.clone(), who, object))
                    .or_insert((building, Vec::new()));
                if PANEL_KINDS.contains(&option) {
                    e.1.push(Button {
                        ty,
                        option,
                        disable,
                    });
                }
            }
        }
        out.into_iter()
            .map(|((f, who, object), (b, mut bs))| {
                bs.sort_unstable();
                bs.dedup();
                (f, who, object, b, bs)
            })
            .collect()
    }

    /// **The panel the original offers** (item 4305): every building the
    /// packets' runs of `make_options` cover, its train and research types
    /// against the ones our panel offers it.
    #[test]
    fn the_panels_of_the_packets_on_disk() {
        let panels = panels_on_disk();
        let Some(roster) = install_roster() else {
            println!("skipping: no install");
            return;
        };
        if panels.is_empty() {
            println!("skipping: no panels on disk (RON_PANELS_DIR, ~/ron-data/panels)");
            return;
        }
        let mut count = (0, 0, 0, 0, 0);
        let mut unheld = (0, 0, 0, 0, 0);
        for (file, who, object, building, theirs) in &panels {
            let count = if PANEL_UNHELD.iter().any(|r| file.starts_with(r)) {
                &mut unheld
            } else {
                &mut count
            };
            let ours = roster
                .trains
                .get((building - rondata::load::BASE_BUILDTYPES) as usize)
                .cloned()
                .unwrap_or_default();
            if theirs.is_empty() && ours.is_empty() {
                continue;
            }
            count.1 += 1;
            if *theirs == ours {
                count.0 += 1;
            }
            count.2 += theirs.iter().filter(|t| ours.contains(t)).count();
            count.3 += theirs.len();
            count.4 += ours.len();
            println!(
                "PANEL {file} who {who} object {object} building {building}: theirs {} {theirs:?}, ours {}, \
                 theirs not ours {:?}, ours not theirs {:?}",
                theirs.len(),
                ours.len(),
                theirs
                    .iter()
                    .filter(|t| !ours.contains(t))
                    .collect::<Vec<_>>(),
                ours.iter()
                    .filter(|t| !theirs.contains(t))
                    .collect::<Vec<_>>()
            );
        }
        println!("PANEL_LEADER {count:?}, unheld {unheld:?}");
        assert_eq!(count, PANEL_LEADER);
        assert_eq!(unheld, PANEL_LEADER_UNHELD);
    }

    /// **`PANEL_LEADER_WALKED`** (item 4315): [`PANEL_LEADER`]'s count over
    /// the held packets (every packet but [`PANEL_UNHELD`]'s, all of the
    /// minted GreatLakes game), ours [`Roster::offered`] on our own walk of
    /// the minted script at the packet's frame, where [`PANEL_LEADER`]'s
    /// ours is the static roster.
    ///
    /// First count, `check_predecessor` alone: (0, 110, 1238, 1238, 1717),
    /// ours 1,945 → 1,717 against the static roster on the same packets,
    /// every type the original offers still on ours. Seen red pinned
    /// (0, 0, 0, 0, 0).
    ///
    /// Item 4321, `can_make`'s arms and the age gate: (69, 110, 1238, 1238,
    /// 1401), the City's 51, the Markets' 60 and 62 and the Barracks' 55
    /// gone; then the `is_active` block: **(69, 103, 1238, 1238, 1373)**,
    /// seven unfinished buildings offering nothing on either side. Every
    /// unit agrees; the 34 parted buildings part on technologies alone (the
    /// research loop, not read). Seen red at the old pin.
    ///
    /// Item 4323, the research loop over `BASE_AGETYPES..0x275` and the
    /// Library's tail: **(103, 103, 1238, 1238, 1238)**, every building's
    /// panel equal, the Libraries' 579 to 582, the 439's 597 to 599 and the
    /// 420's 616 to 619 gone. Seen red at the old pin.
    const PANEL_LEADER_WALKED: (usize, usize, usize, usize, usize) = (103, 103, 1238, 1238, 1238);

    /// **`PANEL_DISABLE_WALKED`** (item 4326, `docs/CLIENT_INPUT.md`
    /// §32.4): [`PANEL_LEADER_WALKED`]'s buildings with each button whole,
    /// its type, option and `disable` (the TSV's columns 8, 5 and 9) against
    /// [`Roster::offered`]'s: (buildings whose buttons equal, buildings
    /// either side offers one on, buttons both offer whole, the
    /// original's, ours). Seen red pinned (0, 0, 0, 0, 0).
    const PANEL_DISABLE_WALKED: (usize, usize, usize, usize, usize) = (103, 103, 1238, 1238, 1238);

    /// **`HUD_PANEL_WALKED`** (item 4334, `docs/CLIENT_INPUT.md` §32.5):
    /// [`PANEL_DISABLE_WALKED`]'s buildings, each the selection alone, and
    /// the buttons [`crate::hud::roster_menu`] draws for it on the walk's
    /// own scene, each whole (type, option, `disable`) against the
    /// original's: (buildings whose buttons equal, buildings either side
    /// offers one on, buttons both offer whole, the original's, the HUD's).
    /// Seen red pinned (0, 0, 0, 0, 0), where the static HUD counted
    /// (0, 110, 103, 1238, 1945): every type its building may queue, a
    /// clear `OPTION_TRAIN`, on the seven unfinished buildings too.
    const HUD_PANEL_WALKED: (usize, usize, usize, usize, usize) = (103, 103, 1238, 1238, 1238);
    /// **`HUD_PANEL_PRESSED`** (item 4334): of the drawn buttons, each
    /// pressed at its centre through [`crate::hud::event`], (those that
    /// order as their `disable` says, pressed): a greyed one nothing, a
    /// clear one its type's `queue_up` of one. Ours: `Options::exec`'s
    /// arms are not read (`docs/CLIENT_INPUT.md` §32.5).
    const HUD_PANEL_PRESSED: (usize, usize) = (1238, 1238);

    /// **The panel through the leader's predicates, on our walk** (item
    /// 4315): every held packet's buildings, the original's train and
    /// research types against [`Roster::offered`] on our walk of the minted
    /// script at that frame, the building found by slot and object id.
    #[test]
    fn the_panels_of_the_packets_on_our_walk() {
        let panels: Vec<_> = buttons_on_disk()
            .into_iter()
            .filter(|(f, ..)| !PANEL_UNHELD.iter().any(|r| f.starts_with(r)))
            .collect();
        let Some(roster) = install_roster() else {
            println!("skipping: no install");
            return;
        };
        if panels.is_empty() {
            println!("skipping: no panels on disk (RON_PANELS_DIR, ~/ron-data/panels)");
            return;
        }
        let mut r = match lobby() {
            Ok(r) => r,
            Err(e) => {
                println!("skipping: {e}");
                return;
            }
        };
        let frame_of = |f: &str| -> i64 { f.rsplit('-').next().unwrap().parse().unwrap() };
        let mut panels = panels;
        panels.sort_by_key(|(f, ..)| frame_of(f));
        let orders = script::script();
        let mut next = 0;
        let mut count = (0, 0, 0, 0, 0);
        let mut disabled = (0, 0, 0, 0, 0);
        let mut hud = (0, 0, 0, 0, 0);
        let mut presses = (0, 0);
        for (file, who, object, building, buttons) in &panels {
            let frame = frame_of(file);
            while r.frame() < frame {
                while next < orders.len() && i64::from(orders[next].0) == r.frame() {
                    r.issue(&orders[next]);
                    next += 1;
                }
                r.step();
            }
            let sim = r.sim();
            let b = sim
                .buildings
                .iter()
                .find(|b| b.alive && b.owner as i32 == *who && i32::from(b.index) == *object)
                .unwrap_or_else(|| panic!("{file}: no building {who}/{object} on our walk"));
            assert_eq!(
                b.ty.map(|t| t as i32 + rondata::load::BASE_BUILDTYPES),
                Some(*building),
                "{file}: {who}/{object}'s type"
            );
            let ours_buttons = roster.offered(sim, *who as usize, *building, b.active);
            // The HUD's row (item 4334): the buttons the HUD draws with
            // this building alone selected, on the walk's own scene.
            let scene = sim.observe(Some(*who as sim::Player));
            let mut input = Input::new(*who as u8);
            input.roster = std::sync::Arc::new(roster.clone());
            input.selection = vec![ObjId {
                who: *who as sim::Player,
                o: *object as i16,
            }];
            input.building = true;
            let row = crate::hud::roster_menu(&input, &scene, Some(sim), HUD_SIZE);
            let mut drawn: Vec<Button> = row.iter().map(|x| x.button).collect();
            drawn.sort_unstable();
            // A press through the HUD: a greyed button orders nothing, a
            // clear one its `queue_up` behind the selection's group.
            let terrain = TerrainScene::default();
            let c = crate::pinned_camera();
            let v = View {
                sim: Some(sim),
                ..view(&c, &scene, &terrain)
            };
            for b in &row {
                let (x, y) = b.rect.centre();
                let got =
                    crate::hud::event(&mut input.clone(), HUD_SIZE, 1, Event::Press { x, y }, &v);
                let queued = got.as_ref().is_some_and(|o| {
                    o.2.iter().any(|k| {
                        *k == Command::QueueUp {
                            unit: b.ty(),
                            num: 1,
                        }
                    })
                });
                presses.1 += 1;
                if (got.is_none() && b.greyed()) || (queued && !b.greyed()) {
                    presses.0 += 1;
                }
            }
            if !(buttons.is_empty() && drawn.is_empty()) {
                hud.1 += 1;
                if *buttons == drawn {
                    hud.0 += 1;
                }
                hud.2 += buttons.iter().filter(|x| drawn.contains(x)).count();
                hud.3 += buttons.len();
                hud.4 += drawn.len();
                if *buttons != drawn {
                    println!(
                        "HUD {file} who {who} object {object} building {building}: \
                         theirs not drawn {:?}, drawn not theirs {:?}",
                        buttons
                            .iter()
                            .filter(|x| !drawn.contains(x))
                            .collect::<Vec<_>>(),
                        drawn
                            .iter()
                            .filter(|x| !buttons.contains(x))
                            .collect::<Vec<_>>()
                    );
                }
            }
            if !(buttons.is_empty() && ours_buttons.is_empty()) {
                disabled.1 += 1;
                if *buttons == ours_buttons {
                    disabled.0 += 1;
                }
                disabled.2 += buttons.iter().filter(|x| ours_buttons.contains(x)).count();
                disabled.3 += buttons.len();
                disabled.4 += ours_buttons.len();
                let parted: Vec<_> = buttons
                    .iter()
                    .filter_map(|x| {
                        let o = ours_buttons.iter().find(|o| o.ty == x.ty)?;
                        (o != x).then_some((x.ty, (x.option, x.disable), (o.option, o.disable)))
                    })
                    .collect();
                if !parted.is_empty() {
                    println!(
                        "DISABLE {file} who {who} object {object} building {building}: \
                         (type, theirs (option, disable), ours) {parted:?}"
                    );
                }
            }
            let theirs: Vec<i32> = buttons.iter().map(|x| x.ty).collect();
            let theirs = &theirs;
            let ours: Vec<i32> = ours_buttons.iter().map(|x| x.ty).collect();
            if theirs.is_empty() && ours.is_empty() {
                continue;
            }
            count.1 += 1;
            if *theirs == ours {
                count.0 += 1;
            }
            count.2 += theirs.iter().filter(|t| ours.contains(t)).count();
            count.3 += theirs.len();
            count.4 += ours.len();
            println!(
                "WALKED {file} who {who} object {object} building {building}: theirs {theirs:?}, \
                 theirs not ours {:?}, ours not theirs {:?}",
                theirs
                    .iter()
                    .filter(|t| !ours.contains(t))
                    .collect::<Vec<_>>(),
                ours.iter()
                    .filter(|t| !theirs.contains(t))
                    .collect::<Vec<_>>()
            );
        }
        println!("PANEL_LEADER_WALKED {count:?}");
        println!("PANEL_DISABLE_WALKED {disabled:?}");
        println!("HUD_PANEL_WALKED {hud:?}");
        println!("HUD_PANEL_PRESSED {presses:?}");
        assert_eq!(count, PANEL_LEADER_WALKED);
        assert_eq!(disabled, PANEL_DISABLE_WALKED);
        assert_eq!(hud, HUD_PANEL_WALKED);
        assert_eq!(presses, HUD_PANEL_PRESSED);
    }

    /// The citizens' pages (item 4337), `~/ron-data/pages/*.tsv` (or
    /// `RON_PAGES_DIR`), `panels.py --citizens`' rows: per (file, frame,
    /// slot, citizen), the buttons of its three pages whole (type, option,
    /// `disable`: columns 9, 6 and 10), sorted, the page's `OPTION_BACK`,
    /// `OPTION_NEXT` and `OPTION_CANCEL` aside.
    fn pages_on_disk() -> Vec<(String, i64, i32, i32, Vec<Button>)> {
        page_rows("RON_PAGES_DIR", "ron-data/pages")
            .into_iter()
            .map(|(f, frame, who, object, rows)| {
                let mut bs: Vec<Button> = rows
                    .into_iter()
                    .filter(|(_, b)| (0..=2).contains(&b.option))
                    .map(|(_, b)| b)
                    .collect();
                bs.sort_unstable();
                (f, frame, who, object, bs)
            })
            .collect()
    }

    /// One citizen's adds, each with the page its run poked (item 4339).
    type PageAdds = Vec<(i32, Button)>;

    /// Every row of `panels.py --citizens` TSVs under `var` (or
    /// `~/<default>`), per (file, frame, slot, citizen): each add whole
    /// (type, option, `disable`) with the page its run poked (−1 for
    /// `--standard`'s run), in the file's order.
    fn page_rows(var: &str, default: &str) -> Vec<(String, i64, i32, i32, PageAdds)> {
        let dir = std::env::var_os(var)
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::Path::new(&std::env::var_os("HOME").unwrap_or_default()).join(default)
            });
        let Some(files) = tsvs_in_tree(&dir) else {
            return Vec::new();
        };
        let mut out: std::collections::BTreeMap<(String, i64, i32, i32), PageAdds> =
            std::collections::BTreeMap::new();
        for f in files {
            let name = f.file_stem().unwrap().to_string_lossy().into_owned();
            for line in std::fs::read_to_string(&f).unwrap().lines() {
                if line.starts_with('#') || line.is_empty() {
                    continue;
                }
                let c: Vec<i64> = line
                    .split('\t')
                    .take(11)
                    .map(|v| v.parse().expect("a panels.py --citizens row"))
                    .collect();
                let (frame, who, object, page, option, ty, disable) = (
                    c[0],
                    c[1] as i32,
                    c[2] as i32,
                    c[4] as i32,
                    c[5] as i32,
                    c[8] as i32,
                    c[9] as i32,
                );
                assert!(option != -1, "{name}: a run that did not return: {line}");
                out.entry((name.clone(), frame, who, object))
                    .or_default()
                    .push((
                        page,
                        Button {
                            ty,
                            option,
                            disable,
                        },
                    ));
            }
        }
        out.into_iter()
            .map(|((f, frame, who, object), rows)| (f, frame, who, object, rows))
            .collect()
    }

    /// **`HUD_PAGES_WALKED`** (item 4337, `docs/CLIENT_INPUT.md` §32.6):
    /// each held packet's citizens (one a slot; every citizen of a slot
    /// shares its pages, run1600's nineteen of slot 1 measured), the
    /// selection alone on the walk's own scene, and the buttons
    /// [`crate::hud::roster_menu`] draws for it, each whole (type, option,
    /// `disable`), against the original's three pages: (citizens whose
    /// buttons equal, citizens, buttons both draw whole, the original's,
    /// the HUD's). Seen red pinned (0, 0, 0, 0, 0). Widened by item 4343
    /// over run1652 and run1653, which reach the limit, site and
    /// `already_built` arms: each arm perturbed turns it red (§32.8).
    const HUD_PAGES_WALKED: (usize, usize, usize, usize, usize) = (36, 36, 1473, 1473, 1473);
    /// **`HUD_PAGES_STATIC`** (item 4337): the same citizens against the
    /// HUD without the walk, the static [`Roster::builds`], each a clear
    /// `OPTION_BUILD`: the gap [`HUD_PAGES_WALKED`] closed.
    const HUD_PAGES_STATIC: (usize, usize, usize, usize, usize) = (0, 36, 175, 1473, 1908);
    /// **`HUD_PAGES_PRESSED`** (item 4337): of the drawn page buttons,
    /// each pressed at its centre through [`crate::hud::event`], (those
    /// that arm as their `disable` says, pressed): a greyed one arms
    /// nothing, a clear one the placement of its type. Ours:
    /// `Options::exec`'s build arm is not read here. Seen red pinned (0, 0).
    const HUD_PAGES_PRESSED: (usize, usize) = (1473, 1473);
    /// **`HUD_PAGES_PAGED`** (item 4339, `docs/CLIENT_INPUT.md` §32.7):
    /// each held packet's citizen paged through the HUD as the original
    /// pages, one page at a time: the standard panel's page buttons
    /// (`panels.py --citizens --standard`), each pressed, and from each
    /// page so reached `OPTION_NEXT`, `OPTION_BACK` and `OPTION_CANCEL`
    /// pressed: thirteen states a citizen, each the page `Options::exec`'s
    /// arm sets and the buttons drawn on it whole (type, option, `disable`)
    /// against the original's adds in that mode: (citizens every state
    /// equal, citizens, states equal, states). Seen red pinned (0, 0, 0, 0).
    const HUD_PAGES_PAGED: (usize, usize, usize, usize) = (36, 36, 468, 468);
    /// **`HUD_STANDARD_WALKED`** (item 4346, `docs/CLIENT_INPUT.md` §32.9):
    /// each held citizen's standard panel as [`crate::hud::roster_menu`]
    /// draws it with the walk and no page, against the original's
    /// `STANDARD_OPTIONS` adds whole (`panels.py --citizens --standard`;
    /// type, option, `disable`): (citizens equal, citizens, buttons both
    /// whole, the original's, the HUD's). Minted at the base's
    /// (0, 36, 108, 398, 108), the page buttons alone.
    const HUD_STANDARD_WALKED: (usize, usize, usize, usize, usize) = (36, 36, 398, 398, 398);
    /// **`HUD_STANDARD_PRESSED`** (item 4346): of the standard panel's
    /// buttons other than the page buttons, each pressed at its centre
    /// through [`crate::hud::event`], (those that order as the original's
    /// press does, pressed): `OPTION_HALT` the selection's `group` and
    /// `halt`, `OPTION_ALARM_PEASANT` its `group` and `alarm`,
    /// `OPTION_AUTO_EXPLORE` and `OPTION_SPELL` their events' orders,
    /// `OPTION_GARRISON` the armed pick and no order; a greyed one, and
    /// [`Event::Unbuilt`]'s, nothing.
    const HUD_STANDARD_PRESSED: (usize, usize) = (290, 290);

    /// **The citizen's build pages, on our walk** (item 4337): every held
    /// packet's citizens, the original's pages (`UnitOut::make_options@00615fc0`
    /// in `BUILD_OPTIONS` mode, run by `tools/recomp/panels.py --citizens`)
    /// against the HUD's on our walk of the minted script at that frame,
    /// the citizen found by slot and object id.
    #[test]
    fn the_pages_of_the_packets_on_our_walk() {
        let pages = pages_on_disk();
        let Some(roster) = install_roster() else {
            println!("skipping: no install");
            return;
        };
        if pages.is_empty() {
            println!("skipping: no pages on disk (RON_PAGES_DIR, ~/ron-data/pages)");
            return;
        }
        let mut r = match lobby() {
            Ok(r) => r,
            Err(e) => {
                println!("skipping: {e}");
                return;
            }
        };
        let mut pages = pages;
        pages.sort_by_key(|(_, frame, ..)| *frame);
        let orders = script::script();
        let mut next = 0;
        let mut hud = (0, 0, 0, 0, 0);
        let mut statics = (0, 0, 0, 0, 0);
        let mut presses = (0, 0);
        let mut paged = (0, 0, 0, 0);
        let mut standard_row = (0, 0, 0, 0, 0);
        let mut standard_pressed = (0, 0);
        let keyed =
            |var, default| -> std::collections::BTreeMap<(i64, i32, i32), Vec<(i32, Button)>> {
                page_rows(var, default)
                    .into_iter()
                    .map(|(_, frame, who, object, rows)| ((frame, who, object), rows))
                    .collect()
            };
        let all = keyed("RON_PAGES_DIR", "ron-data/pages");
        let standard = keyed("RON_PAGES_STANDARD_DIR", "ron-data/pages-standard");
        let tally =
            |row: &mut (usize, usize, usize, usize, usize), theirs: &[Button], ours: &[Button]| {
                row.1 += 1;
                if theirs == ours {
                    row.0 += 1;
                }
                row.2 += theirs.iter().filter(|x| ours.contains(x)).count();
                row.3 += theirs.len();
                row.4 += ours.len();
            };
        for (file, frame, who, object, buttons) in &pages {
            while r.frame() < *frame {
                while next < orders.len() && i64::from(orders[next].0) == r.frame() {
                    r.issue(&orders[next]);
                    next += 1;
                }
                r.step();
            }
            let sim = r.sim();
            let u = sim
                .units
                .iter()
                .find(|u| u.alive() && u.owner as i32 == *who && i32::from(u.index) == *object)
                .unwrap_or_else(|| panic!("{file}: no unit {who}/{object} on our walk"));
            assert!(
                u.ty.is_some_and(|t| CITIZENS.contains(&(t as i32 + rondata::load::BASE_UNITTYPES))),
                "{file}: {who}/{object} is not a citizen on our walk"
            );
            let scene = sim.observe(Some(*who as sim::Player));
            let mut input = Input::new(*who as u8);
            input.roster = std::sync::Arc::new(roster.clone());
            input.selection = vec![ObjId {
                who: *who as sim::Player,
                o: *object as i16,
            }];
            let drawn = |input: &Input, sim| {
                let mut d: Vec<Button> = crate::hud::roster_menu(input, &scene, sim, HUD_SIZE)
                    .iter()
                    .map(|x| x.button)
                    .collect();
                d.sort_unstable();
                d
            };
            // The three pages, one at a time (item 4339): their buildings,
            // each page's own three buttons aside.
            let mut walked = Vec::new();
            for k in 0..3 {
                let paged = Input {
                    page: Some(k),
                    ..input.clone()
                };
                walked.extend(drawn(&paged, Some(sim)).into_iter().filter(|b| b.ty >= 0));
            }
            walked.sort_unstable();
            let statik = drawn(&input, None);
            let terrain = TerrainScene::default();
            let c = crate::pinned_camera();
            let v = View {
                sim: Some(sim),
                ..view(&c, &scene, &terrain)
            };
            for k in 0..3 {
                let paged = Input {
                    page: Some(k),
                    ..input.clone()
                };
                for b in crate::hud::roster_menu(&paged, &scene, Some(sim), HUD_SIZE) {
                    if b.ty() < 0 {
                        continue;
                    }
                    let (x, y) = b.rect.centre();
                    let mut pressed = paged.clone();
                    let got =
                        crate::hud::event(&mut pressed, HUD_SIZE, 1, Event::Press { x, y }, &v);
                    presses.1 += 1;
                    let armed = pressed.placing == Some(b.ty());
                    if got.is_none() && armed != b.greyed() && (armed || pressed.placing.is_none())
                    {
                        presses.0 += 1;
                    }
                }
            }
            // The paging (item 4339): theirs in each mode, the standard
            // panel's page buttons and each page's adds whole.
            let rows = |page: i32| -> Vec<Button> {
                let src = if page < 0 { &standard } else { &all };
                let mut t: Vec<Button> = src
                    .get(&(*frame, *who, *object))
                    .into_iter()
                    .flatten()
                    .filter(|(p, b)| {
                        *p == page
                            && if page < 0 {
                                // The standard panel whole (item 4346).
                                true
                            } else {
                                b.option == page
                                    || [OPTION_NEXT, OPTION_BACK, OPTION_CANCEL].contains(&b.option)
                            }
                    })
                    .map(|(_, b)| *b)
                    .collect();
                t.sort_unstable();
                t
            };
            let state = |inp: &Input, want: Option<i32>| {
                inp.page == want && drawn(inp, Some(sim)) == rows(want.unwrap_or(-1))
            };
            let press = |inp: &Input, option: i32| -> Option<Input> {
                let b = crate::hud::roster_menu(inp, &scene, Some(sim), HUD_SIZE)
                    .into_iter()
                    .find(|b| b.ty() < 0 && b.button.option == option)?;
                let (x, y) = b.rect.centre();
                let mut next = inp.clone();
                let got = crate::hud::event(&mut next, HUD_SIZE, 1, Event::Press { x, y }, &v);
                got.is_none().then_some(next)
            };
            // The standard panel (item 4346): drawn whole, and each of
            // its other buttons pressed.
            let ours = drawn(&input, Some(sim));
            let theirs = rows(-1);
            tally(&mut standard_row, &theirs, &ours);
            if theirs != ours {
                println!(
                    "STANDARD {file} frame {frame} who {who} citizen {object}: \
                     theirs not drawn {:?}, drawn not theirs {:?}",
                    theirs
                        .iter()
                        .filter(|x| !ours.contains(x))
                        .collect::<Vec<_>>(),
                    ours.iter()
                        .filter(|x| !theirs.contains(x))
                        .collect::<Vec<_>>()
                );
            }
            for b in crate::hud::roster_menu(&input, &scene, Some(sim), HUD_SIZE) {
                if b.ty() < 0 && (OPTION_BUILD..=OPTION_BUILD_WONDER).contains(&b.button.option) {
                    continue;
                }
                let (x, y) = b.rect.centre();
                let mut pressed = input.clone();
                let got = crate::hud::event(&mut pressed, HUD_SIZE, 1, Event::Press { x, y }, &v);
                standard_pressed.1 += 1;
                let last = got.as_ref().and_then(|o| o.2.last());
                let ok = match (b.greyed(), b.button.option) {
                    (true, _) => {
                        got.is_none()
                            && pressed.page.is_none()
                            && pressed.placing.is_none()
                            && !pressed.garrisoning
                    }
                    (_, OPTION_HALT) => last == Some(&Command::Halt),
                    (_, OPTION_ALARM_PEASANT) => last == Some(&Command::Alarm),
                    (_, OPTION_GARRISON) => got.is_none() && pressed.garrisoning,
                    (_, OPTION_AUTO_EXPLORE | OPTION_SPELL) => {
                        got.is_some() && got == input.clone().event(1, b.ev, &v)
                    }
                    _ => {
                        got.is_none()
                            && pressed.page.is_none()
                            && pressed.placing.is_none()
                            && !pressed.garrisoning
                    }
                };
                if ok {
                    standard_pressed.0 += 1;
                } else {
                    println!(
                        "STANDARD PRESS {file} frame {frame} who {who}: {:?} gave {got:?}",
                        b.button
                    );
                }
            }
            let mut good = (usize::from(state(&input, None)), 1);
            for k in [OPTION_BUILD, OPTION_BUILD_MILITARY, OPTION_BUILD_WONDER] {
                good.1 += 4;
                let Some(on) = press(&input, k) else {
                    continue;
                };
                good.0 += usize::from(state(&on, Some(k)));
                for (turn, want) in [
                    (OPTION_NEXT, Some((k + 1) % 3)),
                    (OPTION_BACK, Some((k + 2) % 3)),
                    (OPTION_CANCEL, None),
                ] {
                    good.0 += usize::from(press(&on, turn).is_some_and(|i| state(&i, want)));
                }
            }
            paged.1 += 1;
            paged.2 += good.0;
            paged.3 += good.1;
            if good.0 == good.1 {
                paged.0 += 1;
            } else {
                println!(
                    "PAGED {file} frame {frame} who {who} citizen {object}: {} of {} states",
                    good.0, good.1
                );
            }
            tally(&mut hud, buttons, &walked);
            tally(&mut statics, buttons, &statik);
            if *buttons != walked {
                println!(
                    "PAGES {file} frame {frame} who {who} citizen {object}: \
                     theirs not drawn {:?}, drawn not theirs {:?}",
                    buttons
                        .iter()
                        .filter(|x| !walked.contains(x))
                        .collect::<Vec<_>>(),
                    walked
                        .iter()
                        .filter(|x| !buttons.contains(x))
                        .collect::<Vec<_>>()
                );
            }
        }
        println!("HUD_PAGES_WALKED {hud:?}");
        println!("HUD_PAGES_STATIC {statics:?}");
        println!("HUD_PAGES_PRESSED {presses:?}");
        println!("HUD_PAGES_PAGED {paged:?}");
        println!("HUD_STANDARD_WALKED {standard_row:?}");
        println!("HUD_STANDARD_PRESSED {standard_pressed:?}");
        assert_eq!(hud, HUD_PAGES_WALKED);
        assert_eq!(statics, HUD_PAGES_STATIC);
        assert_eq!(presses, HUD_PAGES_PRESSED);
        assert_eq!(paged, HUD_PAGES_PAGED);
        assert_eq!(standard_row, HUD_STANDARD_WALKED);
        assert_eq!(standard_pressed, HUD_STANDARD_PRESSED);
    }

    /// **`PAGE_TURNS_RUN`** (item 4343, `docs/CLIENT_INPUT.md` §32.8):
    /// `Options::exec`'s page turns run on packets (`panels.py --turns`,
    /// `~/ron-data/pages-turns/*.tsv` or `RON_PAGE_TURNS_DIR`): each row a
    /// start (the standard panel or a page), a press (a page button,
    /// `OPTION_NEXT`, `OPTION_BACK`, `OPTION_CANCEL`) under three or two
    /// pages, and the mode the original's run leaves; against [`turn`]:
    /// (rows equal, rows). Seen red pinned (0, 0).
    const PAGE_TURNS_RUN: (usize, usize) = (63, 63);

    /// **The page turns, against the original's own runs** (item 4343):
    /// each row's start and press through [`turn`], the page it leaves
    /// against the mode and `mode_data` `exec` left on the packet.
    #[test]
    fn the_page_turns_of_the_packets() {
        let dir = std::env::var_os("RON_PAGE_TURNS_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::Path::new(&std::env::var_os("HOME").unwrap_or_default())
                    .join("ron-data/pages-turns")
            });
        let Some(files) = tsvs_in_tree(&dir) else {
            println!("skipping: no turns on disk (RON_PAGE_TURNS_DIR, ~/ron-data/pages-turns)");
            return;
        };
        // `OptionMode`: `STANDARD_OPTIONS` 0 is no page, `BUILD_OPTIONS` 1
        // is page `mode_data`; any other mode is no state of ours.
        let page = |mode: i64, data: i64| match mode {
            0 => Ok(None),
            1 => Ok(Some(data as i32)),
            m => Err(m),
        };
        let mut run = (0, 0);
        for f in &files {
            for line in std::fs::read_to_string(f).unwrap().lines() {
                if line.starts_with('#') || line.is_empty() {
                    continue;
                }
                let c: Vec<i64> = line
                    .split('\t')
                    .take(9)
                    .map(|v| v.parse().expect("a panels.py --turns row"))
                    .collect();
                assert!(c[7] != -1, "{f:?}: a run that did not return: {line}");
                let (n, option) = (c[3] as i32, c[6] as i32);
                let from = page(c[4], c[5]).expect("a start the harness poked");
                let theirs = page(c[7], c[8]);
                let ours = turn(option, from, n);
                run.1 += 1;
                if theirs == Ok(ours) {
                    run.0 += 1;
                } else {
                    println!(
                        "TURN {f:?}: pages {n} from {from:?} option {option:#x}: theirs {theirs:?}, ours {ours:?}"
                    );
                }
            }
        }
        println!("PAGE_TURNS_RUN {run:?}");
        assert_eq!(run, PAGE_TURNS_RUN);
    }

    /// **`PAGE_ARMS_WALKED`** (item 4339, `docs/CLIENT_INPUT.md` §32.7): on
    /// our walk of the minted script, each frame where a slot's state for
    /// the page loop's four idle arms changes, as (frame, slot,
    /// `at_city_limit`, `wonders_built`, the wonder types it has sited),
    /// over the whole 24,000 frames. Slot 1 is under its limit on 383..577
    /// and from 7431; it sites the Pyramids (526) on 12783 and finishes it
    /// on 14312. No `obs` arm fires on any frame ([`PAGE_ARMS_OBS`]).
    const PAGE_ARMS_WALKED: [(i64, usize, bool, u32, &[i32]); 8] = [
        (1, 0, true, 0, &[]),
        (1, 1, true, 0, &[]),
        (383, 1, false, 0, &[]),
        (577, 1, true, 0, &[]),
        (7431, 1, false, 0, &[]),
        (12783, 1, false, 0, &[526]),
        (14312, 0, true, 1, &[]),
        (14312, 1, false, 1, &[]),
    ];
    /// The frames of our walk on which a page type is obsolete for a
    /// slot (`obs_flags` or `has_tech(obs)`): none.
    const PAGE_ARMS_OBS: usize = 0;
    /// **`PAGE_ARMS_HELD`** (item 4339): of the held packets' citizens
    /// ([`pages_on_disk`]), those whose slot stands, at the packet's frame
    /// on our walk, under its city limit, or with a wonder built or sited:
    /// three since item 4343 (`docs/CLIENT_INPUT.md` §32.8), run1652's
    /// slot 1 at 13000 (the Pyramids sited, under its limit) and run1653's
    /// two slots at 14400 (`already_built`; slot 1 under its limit). Item
    /// 4339's held packets reach none. Seen red at 0, the pin before them.
    const PAGE_ARMS_HELD: usize = 3;

    /// **The page loop's idle arms on our walk** (item 4339): where on the
    /// minted game a packet would reach them, and that no held packet does.
    #[test]
    fn the_page_arms_on_our_walk() {
        let Some(roster) = install_roster() else {
            println!("skipping: no install");
            return;
        };
        let mut r = match lobby() {
            Ok(r) => r,
            Err(e) => {
                println!("skipping: {e}");
                return;
            }
        };
        let held: std::collections::BTreeSet<(i64, i32)> = pages_on_disk()
            .into_iter()
            .map(|(_, frame, who, ..)| (frame, who))
            .collect();
        let orders = script::script();
        let mut next = 0;
        let mut last: Vec<Option<(bool, u32, Vec<i32>)>> = vec![None; 8];
        let (mut changes, mut obs, mut armed) = (Vec::new(), 0, 0);
        while r.frame() < 24_000 {
            while next < orders.len() && i64::from(orders[next].0) == r.frame() {
                r.issue(&orders[next]);
                next += 1;
            }
            r.step();
            let (sim, frame) = (r.sim(), r.frame());
            let (tree, setup) = (&sim.tables.tech_tree, &sim.tables.setup);
            for (who, p) in sim.tech.iter().enumerate().take(8) {
                let mut sites = Vec::new();
                for &(ti, ..) in &roster.pages {
                    let Some(rec) = usize::try_from(ti - rondata::load::BASE_BUILDTYPES).ok()
                    else {
                        continue;
                    };
                    let Some(t) = sim.tables.build_types.get(rec).and_then(|b| b.tree) else {
                        continue;
                    };
                    if p.obs[t] || tree.has_tech_p(setup, p, tree.types[t].obs) {
                        obs += 1;
                    }
                    if WONDERS.contains(&ti) && sim.num_sites_of(who as sim::Player, rec) > 0 {
                        sites.push(ti);
                    }
                }
                let limit = sim.at_city_limit(who as sim::Player);
                let now = (limit, sim.cities.wonders_built, sites);
                if held.contains(&(frame, who as i32))
                    && (!now.0 || now.1 != 0 || !now.2.is_empty())
                {
                    armed += 1;
                }
                if last[who].as_ref() != Some(&now) {
                    println!("PAGE_ARMS frame {frame} who {who}: {now:?}");
                    changes.push((frame, who, now.0, now.1, now.2.clone()));
                    last[who] = Some(now);
                }
            }
        }
        let pinned: Vec<_> = PAGE_ARMS_WALKED
            .iter()
            .map(|&(f, w, l, b, s)| (f, w, l, b, s.to_vec()))
            .collect();
        println!("PAGE_ARMS_OBS {obs} PAGE_ARMS_HELD {armed}");
        assert_eq!(changes, pinned);
        assert_eq!(obs, PAGE_ARMS_OBS);
        assert_eq!(armed, PAGE_ARMS_HELD);
    }

    /// **`INPUT_ROSTER_QUEUES`** (item 4290, `docs/CLIENT_INPUT.md` §32):
    /// of the packages the recordings on disk carry whose order is a
    /// `queue_up` of one, those a press on the roster's button for its type
    /// mints field for field, the package's orders whole, with the
    /// package's group selected as buildings of the first type whose panel
    /// queues it. Seen red pinned (0, 0), and at (256, 257) while the
    /// roster's rows ran under the disband button.
    const INPUT_ROSTER_QUEUES: (usize, usize) = (257, 257);

    /// **Every recorded `queue_up`, from the roster** (item 4290): each
    /// package's group (or for an empty group the slot's last full one)
    /// made buildings of a type whose panel offers the order's type, then
    /// that type's button pressed.
    #[test]
    fn minted_roster_queue_ups_equal_the_recordings() {
        let files = recordings_on_disk();
        let (Ok(r), Some(roster)) = (lobby(), install_roster()) else {
            println!("skipping: no lobby or no install");
            return;
        };
        if files.is_empty() {
            println!("skipping: no .rcx on disk");
            return;
        }
        let template = r.scene(None).buildings[0];
        drop(r);
        let roster = std::sync::Arc::new(roster);
        let terrain = TerrainScene::default();
        let c = crate::pinned_camera();
        let (mut whole, mut all) = (0, 0);
        let mut parted: std::collections::BTreeMap<i32, usize> = Default::default();
        for f in &files {
            let rec = rondata::recgame::read(&f.display().to_string()).unwrap();
            let mut last = LastSent::new();
            for p in &rec.packages {
                let Ok(cmds) = rondata::commands::decode(&p.data) else {
                    continue;
                };
                let kept: Vec<Command> =
                    cmds.into_iter().filter(rondata::input::is_input).collect();
                let (who, objects, before) = behind(&last, p.play, &kept);
                note_sent(&mut last, p.play, &kept);
                let heads: Vec<&Command> = kept
                    .iter()
                    .filter(|c| !matches!(c, Command::Group { .. }))
                    .collect();
                let [&Command::QueueUp { unit, num: 1 }] = heads[..] else {
                    continue;
                };
                all += 1;
                let at = roster.trained_at(unit).first().copied();
                let scene = Scene {
                    buildings: objects
                        .iter()
                        .map(|&o| {
                            let mut b = template;
                            (b.id, b.owner, b.ty, b.active) = (ObjId { who, o }, who, at, true);
                            b
                        })
                        .collect(),
                    ..Scene::default()
                };
                let v = view(&c, &scene, &terrain);
                let mut input = Input {
                    roster: roster.clone(),
                    building: true,
                    selection: objects.iter().map(|&o| ObjId { who, o }).collect(),
                    ..Input::new(who)
                };
                input.sent = sent_for(&before, &scene);
                let got = crate::hud::roster_menu(&input, &scene, None, HUD_SIZE)
                    .into_iter()
                    .find(|b| b.ev == Event::Train(unit))
                    .and_then(|b| {
                        let (x, y) = b.rect.centre();
                        crate::hud::event(&mut input, HUD_SIZE, p.frame, Event::Press { x, y }, &v)
                    });
                if got.as_ref().map(|o| &o.2) == Some(&kept) {
                    whole += 1;
                } else {
                    *parted.entry(unit).or_default() += 1;
                    println!("ROSTER QUEUE parted f{} {kept:?}: {got:?}", p.frame);
                }
            }
        }
        println!("ROSTER QUEUES {whole} of {all}; parted by type {parted:?}");
        assert_eq!((whole, all), INPUT_ROSTER_QUEUES);
    }

    /// **`INPUT_UNITMASKS`** (item 4288, `docs/CLIENT_INPUT.md` §31): of the
    /// unitmasks the recordings on disk carry, those a press on the
    /// auto-explore button mints field for field, the package's orders
    /// whole. Seen red pinned (0, 0).
    const INPUT_UNITMASKS: (usize, usize) = (1, 1);

    /// **Every recorded unitmask** (item 4288): each package carrying a
    /// `unitmask` is minted from a fresh layer whose selection is the
    /// package's group (or for an empty group the slot's last full one),
    /// by a press on the auto-explore button.
    #[test]
    fn minted_unitmasks_equal_the_recordings() {
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
            let mut last = LastSent::new();
            for p in &rec.packages {
                let Ok(cmds) = rondata::commands::decode(&p.data) else {
                    continue;
                };
                let kept: Vec<Command> =
                    cmds.into_iter().filter(rondata::input::is_input).collect();
                let (who, objects, before) = behind(&last, p.play, &kept);
                note_sent(&mut last, p.play, &kept);
                if !kept.iter().any(|c| matches!(c, Command::Unitmask { .. })) {
                    continue;
                }
                all += 1;
                let scene = Scene {
                    units: objects
                        .iter()
                        .map(|&o| unit(who, o, SCOUT_TYPE, 0, 0, 0))
                        .collect(),
                    ..Scene::default()
                };
                let v = view(&c, &scene, &terrain);
                let mut input = Input::new(who);
                input.selection = objects.iter().map(|&o| ObjId { who, o }).collect();
                input.sent = sent_for(&before, &scene);
                let got = crate::hud::auto_explore_button(&input, &scene, HUD_SIZE).and_then(|r| {
                    let (x, y) = r.centre();
                    crate::hud::event(&mut input, HUD_SIZE, p.frame, Event::Press { x, y }, &v)
                });
                println!("UNITMASK f{} {kept:?}: {got:?}", p.frame);
                if got.as_ref().map(|o| &o.2) == Some(&kept) {
                    whole += 1;
                }
            }
        }
        println!("UNITMASKS {whole} of {all}");
        assert_eq!((whole, all), INPUT_UNITMASKS);
    }

    /// **The auto-explore button's edges** (item 4288): it shows for a unit
    /// of the slot's own and a mixed pick holding one; not for another
    /// slot's unit, a building, no selection, or the entry line open; the
    /// order is the selection's `group` and `unitmask(0x100, −1)`, and the
    /// same selection again sends its `group` as no objects.
    #[test]
    fn minted_auto_explore_edges() {
        if lobby().is_err() {
            println!("skipping: no lobby");
            return;
        }
        let terrain = TerrainScene::default();
        let c = crate::pinned_camera();
        let scene = Scene {
            buildings: vec![city_building(0, 2001)],
            units: vec![garrison_unit(0, 0, &[]), garrison_unit(1, 7, &[])],
            ..Scene::default()
        };
        let v = view(&c, &scene, &terrain);
        let sel = |ids: &[(u8, i16)], building: bool| {
            let mut input = Input::new(0);
            input.selection = ids.iter().map(|&(who, o)| ObjId { who, o }).collect();
            input.building = building;
            input
        };
        let button = |input: &Input| crate::hud::auto_explore_button(input, &scene, HUD_SIZE);
        assert!(
            button(&sel(&[(0, 0)], false)).is_some(),
            "a unit of its own"
        );
        assert!(
            button(&sel(&[(1, 7), (0, 0)], false)).is_some(),
            "a mixed pick"
        );
        assert!(
            button(&sel(&[(1, 7)], false)).is_none(),
            "another slot's unit"
        );
        assert!(button(&sel(&[(0, 2001)], true)).is_none(), "a building");
        assert!(button(&Input::new(0)).is_none(), "no selection");
        let mut typing = sel(&[(0, 0)], false);
        typing.line = Some(String::new());
        assert!(button(&typing).is_none(), "the entry line open");
        assert_eq!(typing.event(9, Event::AutoExplore, &v), None);
        assert_eq!(Input::new(0).event(9, Event::AutoExplore, &v), None);
        assert_eq!(
            sel(&[(0, 2001)], true).event(9, Event::AutoExplore, &v),
            None
        );
        let mask = Command::Unitmask {
            mask: AUTO_EXPLORE_MASK,
            set: AUTO_EXPLORE_SET,
        };
        let mut input = sel(&[(0, 0)], false);
        let got = input.event(9, Event::AutoExplore, &v).unwrap();
        assert_eq!(
            got.2,
            vec![
                Command::Group {
                    who: 0,
                    objects: vec![0]
                },
                mask.clone()
            ]
        );
        let again = input.event(10, Event::AutoExplore, &v).unwrap();
        assert_eq!(
            again.2,
            vec![
                Command::Group {
                    who: 0,
                    objects: vec![]
                },
                mask
            ]
        );
    }

    /// Where the Scout stands on [`script::AUTO_EXPLORE_FRAME`], and the
    /// auto-explore button with it selected, under the opening camera
    /// (`minted_auto_explored_at_the_opening_camera`).
    /// The Scout's `TypeIndex`, 69 (`sim::ids::ty::SCOUT`), the type of
    /// [`script::AUTO_EXPLORE_UNIT`] on stage 0.
    const SCOUT_TYPE: i32 = sim::ids::ty::SCOUT as i32;
    const AUTO_EXPLORE_SCOUT_AT: (i32, i32) = (1134, 699);
    const AUTO_EXPLORE_AT: (i32, i32) = (1070, 966);

    /// The auto-explored script's events: the disband's, then on
    /// [`script::AUTO_EXPLORE_FRAME`] a press on the Scout and one on the
    /// auto-explore button.
    fn auto_explore_events() -> Vec<Scripted> {
        let mut e = disband_events();
        let (ux, uy) = AUTO_EXPLORE_SCOUT_AT;
        let (bx, by) = AUTO_EXPLORE_AT;
        e.extend([
            (
                script::AUTO_EXPLORE_FRAME,
                None,
                Event::Press { x: ux, y: uy },
            ),
            (
                script::AUTO_EXPLORE_FRAME,
                None,
                Event::Press { x: bx, y: by },
            ),
        ]);
        e
    }

    /// **`INPUT_AUTO_EXPLORED`** (item 4288): the auto-explored script's
    /// events, through the HUD and the layer under the opening camera, mint
    /// [`script::auto_explored`] field for field, of 32.
    const INPUT_AUTO_EXPLORED: usize = 32;

    /// **The auto-explored count**: stage 0 on the order's frame — the
    /// Scout is slot 0's and alive, where the press lands, and the button
    /// stands where the second press lands — then the events mint the
    /// auto-explored script, and the file written from them is the
    /// auto-explored file.
    #[test]
    fn minted_auto_explored_at_the_opening_camera() {
        let Ok(mut r) = lobby() else {
            println!("skipping: no lobby");
            return;
        };
        let disbanded = script::disbanded();
        let c = camera(None);
        let mut next = 0;
        while r.frame() < i64::from(script::AUTO_EXPLORE_FRAME) {
            let f = r.frame() as i32;
            while next < disbanded.len() && disbanded[next].0 == f {
                r.issue(&disbanded[next]);
                next += 1;
            }
            r.step();
        }
        {
            let (scene, terrain) = (r.scene(Some(0)), r.terrain());
            let v = view(&c, &scene, &terrain);
            let uid = ObjId {
                who: 0,
                o: script::AUTO_EXPLORE_UNIT,
            };
            let u = scene.units.iter().find(|u| u.id == uid).unwrap();
            let mut input = Input::new(0);
            input.selection = vec![uid];
            let at = crate::hud::auto_explore_button(&input, &scene, HUD_SIZE).map(|r| r.centre());
            println!(
                "AUTO_EXPLORE STAGE0 f{} scout type {} at {:?} pixel {:?} button {at:?}",
                r.frame(),
                u.type_index,
                u.pos,
                v.pixel(u.pos.x, u.pos.y, u.z),
            );
            assert_eq!(u.type_index, SCOUT_TYPE, "the Scout");
            assert_eq!(v.pixel(u.pos.x, u.pos.y, u.z), AUTO_EXPLORE_SCOUT_AT);
            assert_eq!(at, Some(AUTO_EXPLORE_AT));
        }
        drop(r);

        let Some((ours, r)) = mint_with(auto_explore_events(), |input, frame, ev, view| {
            crate::hud::event(input, HUD_SIZE, frame, ev, view)
        }) else {
            return;
        };
        let auto_explored = script::auto_explored();
        let whole = ours
            .iter()
            .zip(&auto_explored)
            .filter(|(a, b)| a == b)
            .count();
        assert_eq!(
            (ours.len(), whole),
            (auto_explored.len(), INPUT_AUTO_EXPLORED),
            "{ours:?}"
        );
        assert_eq!(ours, auto_explored);
        drop(r);
        let home = std::env::var_os("HOME").unwrap_or_default();
        let (Ok(disbanded), Ok(file)) = (
            std::fs::read(std::path::Path::new(&home).join(DISBANDED_RCX)),
            std::fs::read(std::path::Path::new(&home).join(AUTO_EXPLORED_RCX)),
        ) else {
            println!("skipping the bytes: no {DISBANDED_RCX} or {AUTO_EXPLORED_RCX}");
            return;
        };
        assert_eq!(file.len(), AUTO_EXPLORED_RCX_BYTES);
        let written = written(&ours).unwrap();
        assert!(written == file, "the file from the auto-explored orders");
        assert!(written != disbanded, "not the disbanded file");
    }

    /// The auto-explored file (`docs/CLIENT_INPUT.md` §31), written by
    /// rondata's `the_auto_explored_minted_packages_round_trip` under
    /// `RON_RCX_OUT`.
    const AUTO_EXPLORED_RCX: &str =
        "ron-golden/recordings/item4288-auto-explored-greatlakes-24000.rcx";
    const AUTO_EXPLORED_RCX_BYTES: usize = 59_692;

    /// **`INPUT_KINDS`** (item 4249, `docs/CLIENT_INPUT.md` §11): of the
    /// order kinds the recordings on disk carry
    /// (`rondata::input::KINDS_ON_DISK`), those the layer mints, and the
    /// share of the recordings' orders they make.
    const INPUT_KINDS: (usize, usize) = (26, 26);
    /// The orders those kinds make, of every order the recordings carry.
    const INPUT_KINDS_ORDERS: (usize, usize) = (2429, 2429);
    /// The kinds on disk the layer does not mint, most first: the next
    /// kind by share is the first.
    const NOT_MINTED: &[&str] = &[];

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
        // A kind the layer builds through rondata's issuer, named by the
        // call (item 4289: the order is the profile's, and one builder
        // serves the layer and rondata's own minted files).
        let (name, call) = ("leader_options", ".issue_leader_option(");
        if body.contains(call) && !built.iter().any(|b| b == name) {
            built.push(name.to_string());
        }
        let mut want: Vec<String> = MINTS.iter().map(|s| s.to_string()).collect();
        built.sort();
        want.sort();
        assert_eq!(built, want);
    }

    /// **`DRAGGED_PLACEMENTS`** (item 4304, `docs/CLIENT_INPUT.md` §38):
    /// every recorded package whose `build` carries a press's point apart
    /// from its release's, `(frame, type, queue, the census's verdict)`.
    /// All five are the 2017 sample's slot 3, ten orders with their
    /// groups, the census's last misses before the item. Seen red with
    /// 6300's row pinned at the old miss: it parted at "form".
    const DRAGGED_PLACEMENTS: &[(i32, i32, i32, &str)] = &[
        (6300, 428, 1, "INPUT_HAND_PLACEMENTS"),
        (7259, 419, 2, "INPUT_HAND_PLACEMENTS"),
        (7950, 437, 2, "INPUT_HAND_PLACEMENTS"),
        (10057, 419, 2, "INPUT_HAND_PLACEMENTS"),
        (12022, 424, 1, "INPUT_HAND_PLACEMENTS"),
    ];

    /// The census of the dragged placements (item 4304's first act).
    #[test]
    fn the_dragged_placements_of_the_recordings_on_disk() {
        let files = recordings_on_disk();
        if files.is_empty() || lobby().is_err() {
            println!("skipping: no .rcx on disk or no lobby");
            return;
        }
        let roster = install_roster().unwrap_or_default();
        let terrain = TerrainScene::default();
        let c = crate::pinned_camera();
        let scene = Scene::default();
        let v = view(&c, &scene, &terrain);
        let mut got = Vec::new();
        for f in &files {
            let rec = rondata::recgame::read(&f.display().to_string()).unwrap();
            let who = console_who(&rec);
            let mut pauser = Input::new(0);
            for p in &rec.packages {
                let Ok(cmds) = rondata::commands::decode(&p.data) else {
                    continue;
                };
                let kept: Vec<Command> =
                    cmds.into_iter().filter(rondata::input::is_input).collect();
                let Some(&Command::Build {
                    x,
                    y,
                    x2,
                    y2,
                    build,
                    queued,
                }) = kept.iter().find(|c| matches!(c, Command::Build { .. }))
                else {
                    continue;
                };
                if (x, y) == (x2, y2) {
                    continue;
                }
                let verdict = match census_verdict(
                    &kept,
                    p.frame,
                    p.play as u8,
                    who,
                    &mut pauser,
                    &v,
                    &roster,
                ) {
                    Verdict::Shown(pin) => pin,
                    Verdict::Form => "form",
                    Verdict::Miss(arm, _) => arm,
                };
                println!(
                    "DRAGGED {} f{} {kept:?} {verdict}",
                    f.file_name().unwrap().to_string_lossy(),
                    p.frame
                );
                got.push((p.frame, build, queued, verdict));
            }
        }
        assert_eq!(got, DRAGGED_PLACEMENTS);
    }

    /// **The placement on a drag, as the original's** (item 4304, §38):
    /// with a type armed, a left drag is `picked_spot` at the release with
    /// the press beside it, `build(x, y, x2, y2)` the release's ground
    /// point and the press's. It does not box: the selection stands. The
    /// queue and the armed state are the press's (§33). With no pick armed
    /// the drag boxes; a garrison pick with no building under the release
    /// stands. Seen red with the two points swapped.
    #[test]
    fn minted_dragged_placements_as_the_original() {
        let Ok(r) = lobby() else {
            println!("skipping: no lobby");
            return;
        };
        let scene = Scene::default();
        let terrain = r.terrain();
        let c = crate::pinned_camera();
        let v = view(&c, &scene, &terrain);
        let unit = |mods: Mods| Input {
            selection: vec![ObjId { who: 0, o: 5 }],
            mods,
            ..Input::new(0)
        };
        let shift = Mods {
            shift: true,
            ..Mods::default()
        };
        let farm = BUILD_KEYS[0].1;
        let (press, release) = ((900, 500), (960, 530));
        let drag = Event::Drag {
            x0: press.0,
            y0: press.1,
            x1: release.0,
            y1: release.1,
        };
        let from = ground_at(&v, press.0, press.1).unwrap();
        let at = ground_at(&v, release.0, release.1).unwrap();
        assert_ne!(from, at);
        for (mods, queued, armed) in [
            (Mods::default(), QUEUE_NEW, false),
            (shift, QUEUE_LAST, true),
        ] {
            let mut input = unit(mods);
            input.event(1, Event::Place(farm), &v);
            let got = input.event(1, drag, &v).unwrap();
            assert_eq!((got.0, got.1, got.2.len()), (1, 0, 2), "{mods:?}");
            assert_eq!(
                got.2[0],
                Command::Group {
                    who: 0,
                    objects: vec![5]
                },
                "{mods:?}"
            );
            assert_eq!(
                got.2[1],
                Command::Build {
                    x: at.x,
                    y: at.y,
                    x2: from.x,
                    y2: from.y,
                    build: farm,
                    queued,
                },
                "{mods:?}"
            );
            assert_eq!(input.selection, vec![ObjId { who: 0, o: 5 }]);
            assert_eq!(input.placing.is_some(), armed, "{mods:?}");
        }
        // A click places at its one point, both pairs equal.
        let mut input = unit(Mods::default());
        input.event(1, Event::Place(farm), &v);
        let got = input
            .event(
                1,
                Event::Press {
                    x: press.0,
                    y: press.1,
                },
                &v,
            )
            .unwrap();
        assert!(matches!(
            got.2[1],
            Command::Build { x, y, x2, y2, .. } if (x, y) == (from.x, from.y) && (x2, y2) == (x, y)
        ));
        // No pick: the drag boxes, and the empty scene's box takes nothing.
        let mut input = unit(Mods::default());
        assert_eq!(input.event(1, drag, &v), None);
        assert!(input.selection.is_empty());
        // A garrison pick with no building under the release stands.
        let mut input = unit(Mods::default());
        input.event(1, Event::Garrison, &v);
        assert_eq!(input.event(1, drag, &v), None);
        assert!(input.garrisoning);
        assert_eq!(input.selection, vec![ObjId { who: 0, o: 5 }]);
    }
}
