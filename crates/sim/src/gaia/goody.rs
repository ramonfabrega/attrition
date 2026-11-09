//! The goody box — `Unit::explore_goody@005f9780`, `docs/GOODY.md`.
//!
//! A map generator scatters a handful of ruins across the world and marks
//! each with one bit on the cell: `WData.flags & 0x8000`, [`GOODY`]. The
//! first land unit that is not gaia to step into such a cell takes it, and
//! what it takes is **a pile of one good, chosen by a lottery over the
//! goods the player can currently gather**. Every candidate good costs one
//! sync draw, which is why a mechanic worth a page of prose is worth a
//! module: on East Indies it is three draws on frame 867 and the whole
//! stream after it.
//!
//! The pick is not "the good you have least of" and not a uniform choice
//! either. Each candidate scores `draw % 25 + bucket[good]` and the lowest
//! score wins — a strict *less-than*, so ties go to the earlier good — so
//! the jitter only decides between goods within 24 of each other and the
//! poorest good wins outright whenever it is more than 24 behind.
//!
//! # The other half: the search
//!
//! `Unit::find_goody_box@005f2540` is the *approach* — a land unit's
//! 49-cell sweep for a box it has seen and not yet taken, and the
//! `EXPLORE_TO` order `Unit::get_goody_box@005f7690` gives it. It runs from
//! two places: the head of `Unit::think_scout` and, every fifteenth frame,
//! the tail of `Unit::do_explore_to`. Neither spends a draw. `docs/GOODY.md`
//! §7 is the whole of it.
//!
//! `World::reveal_fog@006b3d30` reaches `get_goody_box` directly for a unit
//! carrying `unit_masks & 0x100`; nothing here sets that bit, so that third
//! caller is still a seam.

use crate::economy::RESOURCES;
use crate::orders::{MoveKind, QueuePos};
use crate::tech::Line;
use crate::world::cell::GOODY;
use crate::world::{Cell, MOVE_49, Pos, UNITS_PER_CELL};
use crate::{Player, Sim};

/// The lottery's draw site, under the original's own offset. One draw per
/// candidate good, in good order.
pub const SITE_PICK: &str = "Unit::explore_goody+0x27c";

/// `KNOWLEDGE`, the one good the lottery refuses outright — no draw, no
/// candidacy — however available it is.
const KNOWLEDGE: usize = 3;

/// `WEALTH`, the fallback when no good is a candidate. The original starts
/// the search with `best = -1` and finishes with `cmovns`, so a player who
/// can gather nothing at all still gets a pile of gold.
const FALLBACK: usize = 2;

/// The jitter's width: `Random::get(game_random, 0, 0xffff) % 0x19`.
const JITTER: i32 = 25;

impl Sim {
    /// `Unit::explore_goody@005f9780`: the unit `u` has just entered a cell
    /// carrying [`GOODY`]. Clear the bit and pay the finder.
    ///
    /// The caller is [`Sim::set_new_location`], which applies the four
    /// guards (§2); this function assumes them.
    pub(crate) fn explore_goody(&mut self, u: usize) {
        let who = self.units[u].owner;
        explore_goody_steps(&mut SimGoody { sim: self, u, who });
    }
}

/// What `Unit::explore_goody@005f9780` asks of the world, in the order the
/// original asks it. [`Sim`] answers through [`SimGoody`]; the sweep
/// (`sweep/unit_explore_goody.rs`) answers from the emulated original's
/// windows, so the body below is the crate's own on both sides.
pub(crate) trait GoodyView {
    /// `WData.flags &= ~GOODY` on the finder's cell.
    fn clear_box(&mut self);
    /// `game->frame != 0`.
    fn started(&self) -> bool;
    /// `LeaderData::type_avail(good, 1) != 0`.
    fn candidate(&mut self, good: usize) -> bool;
    /// The finder's stock of the good (`bucket[good]`, decrypted).
    fn held(&self, good: usize) -> i32;
    /// `Random::get(game_random, 0, 0xffff)`.
    fn roll(&mut self) -> i32;
    /// The pile: the Spanish pair when `has_tribe_bonus(9)`.
    fn pile(&mut self) -> i32;
    /// `bucket[good] += amount` and `goody_box_resources += amount`.
    fn pay(&mut self, good: usize, amount: i32);
}

/// The body of `Unit::explore_goody` past the UI: the bit, the `frame`
/// gate, the lottery, the pile, the payment.
pub(crate) fn explore_goody_steps(v: &mut impl GoodyView) {
    // The bit goes first and unconditionally — before the `frame != 0`
    // gate, so a goody a unit is *placed* on at setup is consumed
    // without paying (§4).
    v.clear_box();
    if !v.started() {
        return;
    }
    let good = pick_goody_steps(v);
    let amount = v.pile();
    v.pay(good, amount);
}

/// The lottery — one draw a candidate, the lowest score winning. The score
/// is the original's `int` sum, so it wraps like one.
///
/// A candidate is a good whose `LeaderData::type_avail(good, 1)` is
/// `AVAILABLE` and which is not [`KNOWLEDGE`]. The original tests
/// availability *first*, so an unavailable knowledge costs nothing
/// either way and the draw count is the number of available non-
/// knowledge goods.
fn pick_goody_steps(v: &mut impl GoodyView) -> usize {
    let mut best: i32 = 99_999_999;
    let mut pick: Option<usize> = None;
    for g in 0..RESOURCES {
        if !v.candidate(g) || g == KNOWLEDGE {
            continue;
        }
        let held = v.held(g);
        let score = (v.roll() % JITTER).wrapping_add(held);
        if score < best {
            best = score;
            pick = Some(g);
        }
    }
    pick.unwrap_or(FALLBACK)
}

/// [`Sim`]'s answers to [`GoodyView`] for the unit `u` of player `who`.
struct SimGoody<'a> {
    sim: &'a mut Sim,
    u: usize,
    who: Player,
}

impl GoodyView for SimGoody<'_> {
    fn clear_box(&mut self) {
        let cell = self.sim.units[self.u].pos.cell();
        let mut d = self.sim.world.cell_data(cell);
        d.flags &= !GOODY;
        self.sim.world.set_cell_data(cell, d);
    }
    fn started(&self) -> bool {
        self.sim.frame != 0
    }
    fn candidate(&mut self, good: usize) -> bool {
        self.sim.type_available(self.who, good)
    }
    fn held(&self, good: usize) -> i32 {
        self.sim.economy.ledgers[self.who as usize].bucket[good]
    }
    fn roll(&mut self) -> i32 {
        self.sim.mark(SITE_PICK);
        self.sim.rng.roll()
    }
    fn pile(&mut self) -> i32 {
        self.sim.goody_amount(self.who)
    }
    fn pay(&mut self, good: usize, amount: i32) {
        let l = &mut self.sim.economy.ledgers[self.who as usize];
        l.bucket[good] = l.bucket[good].wrapping_add(amount);
        l.goody_box_resources = l.goody_box_resources.wrapping_add(amount);
    }
}

/// One goody box: `Item`'s position and live bit, and its `ever_seen`
/// (`ItemData +0x20`). `Item::init@006770e0` zeroes the byte and marks the
/// cell [`GOODY`]; `World::reveal_fog@006b3d30` is the byte's only other
/// writer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GoodyItem {
    pub pos: Pos,
    pub alive: bool,
    pub ever_seen: u8,
}

impl Sim {
    /// A box from the dump's `ITEM` record (item 3335).
    pub fn add_goody_item(&mut self, pos: Pos, alive: bool, ever_seen: u8) {
        self.gaia.goody_items.push(GoodyItem {
            pos,
            alive,
            ever_seen,
        });
    }

    /// `ObjectsData::find_goody_at@0065c040` as far as this crate keeps
    /// items: the live box standing on `c` while the cell still carries
    /// [`GOODY`] — the flag `get_goody` clears when the box is taken.
    pub(crate) fn goody_item_at(&self, c: Cell) -> Option<usize> {
        if self.world.cell_data(c).flags & GOODY == 0 {
            return None;
        }
        self.gaia
            .goody_items
            .iter()
            .position(|it| it.alive && it.pos.cell() == c)
    }

    /// `Unit::find_goody_box@005f2540` (`docs/GOODY.md` §7): the 49-cell
    /// sweep for a box worth walking to. Returns whether an order was
    /// issued; spends no draw either way.
    ///
    /// A candidate cell must be **in the sweeper's own region**, carry
    /// [`GOODY`], and have been seen — `WorldData::was_seen` at any one of
    /// the cell's four half-cells, so a cell glimpsed at one corner
    /// counts. The first candidate in `move_x`/`move_y` order wins, and a
    /// candidate the unit is *already* walking to (the cell of
    /// `orders_x`/`orders_y`) ends the sweep with no order rather than
    /// re-issuing the one it has.
    pub(crate) fn find_goody_box(&mut self, u: usize) -> bool {
        let Some(t) = self.units[u].ty else {
            return false;
        };
        if self.tables.unit_types[t].kind.domain != crate::territory::attrition::Domain::Land {
            return false;
        }
        let who = self.units[u].owner;
        let here = self.units[u].pos.cell();
        // The original reads `WData.region` of its own cell whatever it is
        // — including the `-1` of a cell in no region — and compares the
        // raw short, so an off-region unit looks for off-region boxes.
        let region = self.world.region_of(here);
        for (dx, dy) in MOVE_49 {
            let c = Cell::new(here.x + dx, here.y + dy);
            if !self.world.contains(c) {
                continue;
            }
            if self.world.region_of(c) != region {
                continue;
            }
            let d = self.world.cell_data(c);
            if d.flags & GOODY == 0 {
                continue;
            }
            if !self.goody_was_seen(c, who) {
                continue;
            }
            // …and then the **item** on the cell: `find_goody_at`'s chain
            // walk, and `ItemData::is_seen` on what it finds. That is a
            // second, *stricter* fog read — `ever_seen`, the bare
            // accumulation of line of sight, with none of `was_seen`'s
            // ally-territory shortcut — and on East Indies it is the gate
            // that matters, because the box sits inside its finder's own
            // borders and the shortcut above answers yes from frame 0.
            // `docs/GOODY.md` §7.2.
            if !self.goody_item_is_seen(c, who) {
                continue;
            }
            if c == self.units[u].orders_pos.cell() {
                return false;
            }
            self.get_goody_box(u, c);
            return true;
        }
        false
    }

    /// The sweep's fog read: `WorldData::was_seen` at all four half-cells
    /// of `c`, in the original's own order — `(2x+1, 2y+1)`, `(2x, 2y+1)`,
    /// `(2x+1, 2y)`, `(2x, 2y)` — and true if **any** of them answers yes.
    /// The listing tests them in sequence and only falls through to the
    /// next cell when all four are clear.
    fn goody_was_seen(&self, c: Cell, who: Player) -> bool {
        let (x, y) = (2 * c.x, 2 * c.y);
        self.was_seen_fog(x + 1, y + 1, who)
            || self.was_seen_fog(x, y + 1, who)
            || self.was_seen_fog(x + 1, y, who)
            || self.was_seen_fog(x, y, who)
    }

    /// `ItemData::is_seen@00677850` (vtable `+0x48`) on the box's own
    /// item, as far as a simulation with no item chain can answer it.
    ///
    /// ```text
    /// if ever_seen & ally_mask:                      return 1
    /// if who >= 0 && has_tribe_bonus(9) && <the item's cell is who's>:
    ///                                                return was_seen(item half-cell)
    /// return WorldData::is_seen(item half-cell, who)
    /// ```
    ///
    /// The first arm is `ever_seen & ally_mask` — a per-item byte that
    /// `World::reveal_fog@006b3d30` ORs `1 << who` into when a half-cell
    /// of the item's cell is first revealed, the same monotone
    /// accumulation `World::seen2` already is. So it reads
    /// `was_really_seen` — the fog with none of `was_seen`'s
    /// ally-territory shortcut — over the cell's four half-cells.
    ///
    /// **The fall-through is the current line of sight**, and its
    /// territory arm is what the coverage pair's word turned on (item
    /// 1558, `docs/AI.md` §153): [`Sim::world_is_seen_fog`] at the item's
    /// half-cell, so a leader holding the territory-reveal bonus
    /// (Computerization, `leader_flags & 0x2000`) sees a box on its own
    /// ground with no unit of its ever having looked. run682's packet holds
    /// the one box this crate has read whole: item 0 at the **cell centre**
    /// (29568, 26496), half-cell `(2x + 1, 2y + 1)`, `ever_seen` 0 and
    /// `seen2` clear on all four half-cells, the cell `who` 1. The centre
    /// is assumed for every box.
    ///
    /// **The Spanish arm** (item 3318, `docs/GOODY.md` §7.2): a leader
    /// with `has_tribe_bonus(9)` whose own cell the box is on — `WData
    /// +0xf`, `who`, read at the item's cell (`00677850`: the item's
    /// coordinates `>> 8` through `div_3_table`, the byte at `wdata +
    /// cell × 0x1c + 0xf` against `who`) — answers
    /// `WorldData::was_seen` at the item's half-cell, territory shortcut
    /// and all, and returns it **without** falling through to the line of
    /// sight. On Great Sahara the Spanish Scout sweeps a box inside its own
    /// borders on 524 from three cells away (run896b's `astar_path` to the
    /// box's cell that frame, `docs/AI.md` §264).
    ///
    /// SEAM: the whole chain walk that finds the item in the first place
    /// (`docs/QUEUE.md` item 48); the items are loaded from a start dump,
    /// and a sim without them reads the fog in the item's place. A box on
    /// a cell nothing has ever seen and nobody's ground answers no here
    /// either way.
    fn goody_item_is_seen(&self, c: Cell, who: Player) -> bool {
        let (x, y) = (2 * c.x, 2 * c.y);
        // The item's own byte when the boxes are loaded (item 3335): the
        // fog is not it. A box a building's light reached before any unit
        // looked is in `seen2` and in no leader's `ever_seen` —
        // run962's item 1 at (45, 49), `ever_seen` 0 on every block to 36
        // under `seen2` 2 at (91, 98) and (91, 99).
        let item_seen = if self.gaia.goody_items.is_empty() {
            self.was_really_seen_fog(x + 1, y + 1, who)
                || self.was_really_seen_fog(x, y + 1, who)
                || self.was_really_seen_fog(x + 1, y, who)
                || self.was_really_seen_fog(x, y, who)
        } else {
            let mask = self
                .ai
                .ai
                .get(who as usize)
                .map_or(0, |a| a.census.ally_mask)
                | (1 << who);
            self.goody_item_at(c)
                .is_some_and(|i| u32::from(self.gaia.goody_items[i].ever_seen) & mask != 0)
        };
        if item_seen {
            return true;
        }
        if self
            .tables
            .tech_tree
            .has_tribe_bonus(&self.tables.setup, &self.tech[who as usize], 9)
            && self.world.owner(c) == crate::world::Owner::Player(who)
        {
            return self.was_seen_fog(x + 1, y + 1, who);
        }
        self.world_is_seen_fog(x + 1, y + 1, who)
    }

    /// `Unit::get_goody_box@005f7690`: a one-member group, and an
    /// `EXPLORE_TO` to the box's **cell centre** at `QUEUE_FIRST`.
    ///
    /// The queue position is the group's, not the unit's, and the two are
    /// different things (`docs/GROUPS.md` §17): the group halts its
    /// members and re-issues the move as `QUEUE_NEW`, so the walk the unit
    /// was on is dropped rather than stacked behind.
    fn get_goody_box(&mut self, u: usize, c: Cell) {
        let who = self.units[u].owner;
        let o = goody_box_order(c);
        let mut g = crate::groups::group::Group::stack(who);
        self.group_add(&mut g, u);
        if !self.push_group(&mut g, o.force) {
            return;
        }
        self.group_action_move_to(&g, o.to, o.queue, o.set_angle, o.angle, o.kind, o.action);
    }

    /// The pile: `epoch * GOODY_BOX_AGE + GOODY_BOX`, and the Spanish pair
    /// in place of both when the finder has `has_tribe_bonus(9)`.
    ///
    /// **The `epoch` is not the age.** The original reads
    /// `LeaderDataEncrypt +0xf4`, which is `epoch[3]` — the Science line's
    /// level, `get_epoch_base(3) == BASE_EPOCHTYPES` — and not `ages` at
    /// `+0xdc`, whatever `GOODY_BOX_AGE`'s name suggests. `docs/GOODY.md`
    /// §3 has the listing that settles it.
    fn goody_amount(&self, who: Player) -> i32 {
        let level = self.tech[who as usize].epoch[Line::Science.index()];
        let t = &self.tables.tuning;
        let spanish =
            self.tables
                .tech_tree
                .has_tribe_bonus(&self.tables.setup, &self.tech[who as usize], 9);
        goody_pile(
            level,
            spanish,
            [
                t.goody_box_age,
                t.goody_box,
                t.spanish_ruins,
                t.spanish_ruins_base,
            ],
        )
    }
}

/// The pile's arithmetic, `int` wraparound and all: `level * k[0] + k[1]`
/// (`GOODY_BOX_AGE`, `GOODY_BOX`), the Spanish pair `k[2]`, `k[3]` in place
/// of both when the finder has the tribe bonus.
pub(crate) fn goody_pile(level: i32, spanish: bool, k: [i32; 4]) -> i32 {
    if spanish {
        level.wrapping_mul(k[2]).wrapping_add(k[3])
    } else {
        level.wrapping_mul(k[0]).wrapping_add(k[1])
    }
}

/// What `Unit::get_goody_box@005f7690` hands its one-member group:
/// `push_group`'s `force`, and `Group::action_move_to`'s arguments.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GoodyBoxOrder {
    /// `push_group(who, g, 1)`: a group of one is installed.
    pub force: bool,
    /// The box's **cell centre**, `cell * 0x300 + 0x180` on each axis.
    pub to: Pos,
    pub queue: QueuePos,
    pub set_angle: bool,
    pub angle: crate::movement::Angle,
    pub kind: MoveKind,
    pub action: bool,
}

/// [`GoodyBoxOrder`] for the box on cell `c` — the whole of
/// `get_goody_box` but the group it is handed to, so the emulated
/// original can be held to it (`sweep::unit_get_goody_box`, item 2314).
pub(crate) fn goody_box_order(c: Cell) -> GoodyBoxOrder {
    let half = UNITS_PER_CELL / 2;
    GoodyBoxOrder {
        force: true,
        to: Pos::new(c.x * UNITS_PER_CELL + half, c.y * UNITS_PER_CELL + half),
        queue: QueuePos::First,
        set_angle: false,
        angle: crate::movement::Angle(0),
        kind: MoveKind::ExploreTo,
        action: false,
    }
}

#[cfg(test)]
mod tests;
