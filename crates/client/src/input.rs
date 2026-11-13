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
    Key(char),
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
    /// enemy arm (`docs/CLIENT_INPUT.md` §6).
    Enemy(ObjId),
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
    /// building (the attack's search, units and buildings). Each look takes
    /// the nearest anchor within [`PICK_RADIUS`], as a press does (ours:
    /// the original's ±5 px box and bounding volumes are not read). The
    /// issuing slot's own units are no target: the press falls to the
    /// ground.
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
                .filter(|u| u.id.who != who)
                .filter_map(|u| Some((near(u.pos, u.z)?, u.id))),
        );
        if let Some((_, id)) = enemy_unit {
            return Some(Hit::Enemy(id));
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
                .filter(|b| b.id.who != who)
                .filter_map(|b| Some((near(b.pos, b.z)?, b.id))),
        );
        enemy_building.map(|(_, id)| Hit::Enemy(id))
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
    /// - another slot's unit or building: `attack(ox, whom, ignore 0,
    ///   QUEUE_NEW)` (`GroupOut::issue_attack`);
    /// - one's own unfinished building: `swarm_around(ox, whom, QUEUE_NEW,
    ///   BUILD_AT)`, citizens sent to help build it;
    /// - one's own finished gather building with room: `gather(ox,
    ///   QUEUE_NEW)`;
    /// - one's own finished building that is damaged, and full or not a
    ///   gather type: `swarm_around(…, REPAIR)`;
    /// - one's own finished building that is undamaged and not a gather
    ///   building with room (the city, a full camp): nothing.
    fn on_object(&self, hit: Hit) -> Option<Command> {
        match hit {
            Hit::Enemy(id) => Some(Command::Attack {
                ox: i32::from(id.o),
                whom: i32::from(id.who),
                ignore: 0,
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

    /// One event on `frame` (the recording frame the package is consumed
    /// on): the order it makes, if any.
    pub fn event(&mut self, frame: i32, ev: Event, view: &View) -> Option<Order> {
        let play = i32::from(self.player);
        match ev {
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
                if self.selection.is_empty() || self.building {
                    return None;
                }
                if let Some(hit) = view.target(self.player, x, y) {
                    let cmd = self.on_object(hit)?;
                    return Some((frame, play, vec![self.group(), cmd]));
                }
                let at = ground_at(view, x, y)?;
                // One unit keeps the scripts' −1/−1, the fields every file
                // played so far carries (ours: the original's main view
                // mints its formation for one unit too, parked 4180a).
                let (form, width) = if self.selection.len() > 1 {
                    view.formation(&self.selection)
                } else {
                    (-1, -1)
                };
                let cmd = Command::MoveTo {
                    to_x: at.x,
                    to_y: at.y,
                    set_angle: 0,
                    angle: 0,
                    orders: 1,
                    queued: 2,
                    form,
                    width,
                    disembark: 0,
                };
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

    pub(crate) fn events(closer: bool) -> Vec<Scripted> {
        let o = |k: usize| OBJECTS[k].3;
        let (t0, t1) = (TARGETS[0], TARGETS[1]);
        let tp = |t: Target| {
            if closer { t.7 } else { t.3 }
        };
        let cam = |t: Target| closer.then_some((Pos { x: t.1.0, y: t.1.1 }, t.2));
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
                Event::RightPress {
                    x: tp(t0).0,
                    y: tp(t0).1,
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
            let (scene, terrain) = (r.scene(None), r.terrain());
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

    /// **`INPUT_CLICKED`** (`docs/CLIENT_INPUT.md` §5, item 4149): the same
    /// seven events, every target clicked under the opening camera at the
    /// pixel the script's own target projects to, mint
    /// [`script::clicked`] field for field, of 3. Was 0 (no clicked script).
    const INPUT_CLICKED: usize = 3;
    /// The clicked file (`docs/CLIENT_INPUT.md` §5): the minted file's
    /// lobby, `leader_options` and closing frame with the clicked orders,
    /// written by rondata's `the_clicked_minted_packages_round_trip` under
    /// `RON_RCX_OUT`; the file the original's playback takes open (run
    /// 1563).
    pub(crate) const CLICKED_RCX: &str =
        "ron-golden/recordings/item4149-clicked-greatlakes-24000.rcx";
    pub(crate) const CLICKED_RCX_BYTES: usize = 59_280;

    /// **The clicked count**: the events at the opening camera mint the
    /// clicked script, every field; the walk that took them stands on frame
    /// 299 with the Scout on its clicked point and the Farm's site `0/2007`
    /// where the first script's stands; and the file written from them is
    /// the clicked file, byte for byte, not the first script's.
    #[test]
    fn minted_clicked_at_the_opening_camera() {
        let Some((ours, r)) = mint(false) else { return };
        let clicked = script::clicked();
        let whole = ours.iter().zip(&clicked).filter(|(a, b)| a == b).count();
        assert_eq!(
            (ours.len(), whole),
            (clicked.len(), INPUT_CLICKED),
            "{ours:?}"
        );
        assert_eq!(ours, clicked);
        let scene = r.scene(None);
        let scout = object(&scene, script::SCOUT_OBJECT).0;
        assert_eq!(
            (scout.x, scout.y),
            (script::CLICKED_SCOUT_TO.x, script::CLICKED_SCOUT_TO.y)
        );
        let site = object(&scene, 2007).0;
        assert_eq!((site.x, site.y), (3456, 29760));

        let home = std::env::var_os("HOME").unwrap_or_default();
        let (Ok(first), Ok(file)) = (
            std::fs::read(std::path::Path::new(&home).join(MINTED_RCX)),
            std::fs::read(std::path::Path::new(&home).join(CLICKED_RCX)),
        ) else {
            println!("skipping the bytes: no {MINTED_RCX} or {CLICKED_RCX}");
            return;
        };
        assert_eq!(file.len(), CLICKED_RCX_BYTES);
        let path = std::path::Path::new(&home).join(MINTED_RCX);
        let rec = rondata::recgame::read(&path.to_string_lossy()).unwrap();
        let lo = rondata::commands::decode(&rec.packages[0].data).unwrap();
        let last = rec.packages.last().unwrap().frame;
        let mut written = rec.clone();
        written.packages = rondata::recgame::minted_packages(lo[0].clone(), &ours, last).unwrap();
        let written = rondata::recgame::write_gz(&written);
        assert!(written == file, "the file from the clicked orders");
        assert!(
            written != first,
            "the clicked file is not the first script's"
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
                let (scene, terrain) = (r.scene(None), r.terrain());
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
                        Some(Hit::Enemy(ObjId { who, o })),
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
}
