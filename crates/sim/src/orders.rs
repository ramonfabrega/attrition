//! Orders: the per-unit order list, the three enqueue modes, one order stepped
//! per frame, and the lifecycle every other mechanic hangs off.
//! `docs/ORDERS.md`.
//!
//! The original keeps a circular ring of `UnitOrder`s per unit whose current
//! order is `head->prev`, the oldest; `QUEUE_LAST` appends, `QUEUE_FIRST`
//! rotates the new order to the front, `QUEUE_NEW` kills everything first.
//! Here that is a [`VecDeque`] whose **front is the current order**. The step
//! is `Unit::work` → `do_job`: one dispatch on the front order per unit per
//! frame, after attrition and before the body follows (`docs/ORDERS.md` §2).
//! An order a `do_*` inserts in front (`QUEUE_FIRST`) runs from the next
//! frame; an order applied from outside the step runs in the same frame.
//!
//! What is modelled and what is an input is §13 of the document. The
//! pathfinder is no longer a stub: [`Sim::find_path`] is the straight-line
//! verifier, and the three grid planners live in `path.rs`
//! (`docs/PATHFINDER.md`).

use crate::anim;
use crate::build::{self, Ident, flags as bflags};
use crate::combat::{self, Obj};
use crate::economy;
use crate::garrison::GarrisonRefused;
use crate::movement::{self, Angle, find_angle};
use crate::world::{tile, vector_dist};
use crate::{Player, Pos, Sim, Unit, farms};

/// `OrderIndex` — the value the gamelog's `type` line carries.
pub mod index {
    pub const NONE: u8 = 0;
    pub const MOVE_TO: u8 = 1;
    pub const ATTACK_TO: u8 = 2;
    pub const EXPLORE_TO: u8 = 3;
    pub const FLEE_TO: u8 = 4;
    pub const BUILD_AT: u8 = 6;
    pub const GATHER: u8 = 7;
    pub const ATTACK: u8 = 10;
    /// Never constructed here; read by `UnitData::get_speed`'s order scale
    /// (`docs/MOVEMENT.md`, "The effective speed here, now").
    pub const GUARD: u8 = 12;
    pub const REPAIR: u8 = 13;
    pub const CAST_SPELL: u8 = 14;
    pub const TRADE_ROUTE: u8 = 15;
    pub const GARRISON: u8 = 26;
    pub const THINK: u8 = 27;

    /// The three kinds of the **move family** this crate does not spell as
    /// a plain `MoveKind`. `CHANGE_FORM` is `FormOrder`, which this crate
    /// has no order for at all; the other two are what the original calls
    /// a move whose `MoveOrder` carries a [`super::GroupMove`], and
    /// [`super::Order::index`] returns them because `get_type()` does
    /// (`GroupMoveOrder::get_type@00485a10` → `GROUP_MOVE`,
    /// `GroupAttackToOrder::get_type@004825b0` → `GROUP_ATTACK_TO`).
    pub const CHANGE_FORM: u8 = 18;
    pub const GROUP_MOVE: u8 = 19;
    pub const GROUP_ATTACK_TO: u8 = 21;

    /// **The move family** — the seven kinds whose class derives from
    /// `MoveOrder`, which `kill_current_order`, `work`, `repath`,
    /// `resolve_unit_collision` and `detect_unit_collision` all treat
    /// together (`docs/ORDERS.md` §1.2). Written out here once so a caller
    /// names the set rather than the four it happens to construct:
    /// `Unit::resolve_unit_collision@005f9d30:261` and `:381` are the two
    /// sites that enumerate it in the original, and both list all seven.
    ///
    /// SEAM: [`CHANGE_FORM`] is in the original's set and can never be
    /// this crate's answer, so every caller of this is six-sevenths of
    /// the original's test.
    pub const fn is_move_family(kind: u8) -> bool {
        matches!(
            kind,
            MOVE_TO | ATTACK_TO | EXPLORE_TO | FLEE_TO | CHANGE_FORM | GROUP_MOVE | GROUP_ATTACK_TO
        )
    }

    /// Whether this crate can ever *produce* `kind` — the domain of
    /// [`super::Order::index`]. The complement is the set of
    /// `OrderIndex` values a dump can hold and the simulation cannot,
    /// which `crate::diff` reports as a structural gap of its own
    /// (`OrderMismatch::Unspellable`) rather than as a state divergence
    /// (`docs/ORDERS.md` §1.7).
    pub const fn is_modelled(kind: u8) -> bool {
        matches!(
            kind,
            NONE | MOVE_TO
                | ATTACK_TO
                | EXPLORE_TO
                | FLEE_TO
                | BUILD_AT
                | GATHER
                | ATTACK
                | REPAIR
                | CAST_SPELL
                | TRADE_ROUTE
                | GROUP_MOVE
                | GROUP_ATTACK_TO
                | GARRISON
                | THINK
        )
    }
}

/// The `TypeIndex` of a spell, the value a `CastOrder` carries at `+0x20`
/// and `SpellType::cast` switches on. The spell rows are `0x275..=0x2ab`,
/// in `craftrules.xml`'s own order (`docs/DATALAYER.md`).
pub mod spell {
    /// The first row of `craftrules.xml`, `Bribe` — the base `SpellType`
    /// index and the offset [`crate::Sim::spell`] subtracts.
    pub const FIRST: i32 = 0x275;
    /// The last, the eighth `Cancel Alliance`. `TypeData::is_spell_type`
    /// is `FIRST <= i <= LAST` and nothing else (`Unit::do_cast`'s head).
    pub const LAST: i32 = 0x2ab;

    /// `TRANSPORT` — the shore conversion (`docs/TRANSPORT.md` §6).
    pub const TRANSPORT: i32 = 0x28a;

    /// The four **pack** rows and the four **unpack** ones, in the pairs
    /// `add_cast_order` rewrites `PACK`/`UNPACK` into: the siege engine's
    /// (the generic pair, and the one the file names `Catapult`), the
    /// machine gun's, the merchants' and the fishing boat's
    /// (`docs/ORDERS.md` §6.9).
    pub const PACK: i32 = 0x28b;
    pub const UNPACK: i32 = 0x28c;
    pub const PACK_MACHINEGUN: i32 = 0x28d;
    pub const UNPACK_MACHINEGUN: i32 = 0x28e;
    pub const PACK_MERCHANT: i32 = 0x28f;
    pub const UNPACK_MERCHANT: i32 = 0x290;
    pub const PACK_FISHERMEN: i32 = 0x291;
    pub const UNPACK_FISHERMEN: i32 = 0x292;

    /// `TypeData::is_pack` — the vtable slot `+0x50`, which `Unit::do_cast`
    /// inlines as these four identities and nothing else.
    pub fn is_pack(spell: i32) -> bool {
        matches!(
            spell,
            PACK | PACK_MACHINEGUN | PACK_MERCHANT | PACK_FISHERMEN
        )
    }

    /// `TypeData::is_unpack` — the slot `+0x54`, the mirror four.
    pub fn is_unpack(spell: i32) -> bool {
        matches!(
            spell,
            UNPACK | UNPACK_MACHINEGUN | UNPACK_MERCHANT | UNPACK_FISHERMEN
        )
    }
}

/// One row of `craftrules.xml` — `SpellTypeData`, as much of it as
/// `Unit::do_cast` reads (`docs/ORDERS.md` §6.9). The rows are `TypeIndex`
/// [`spell::FIRST`]`..=`[`spell::LAST`] in file order, which is how
/// [`crate::Sim::spells`] is keyed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SpellType {
    /// `JOB_TIME`, in frames: how many `do_cast` steps the cast waits
    /// before `SpellType::cast` runs. `SpellTypeData::get_job_time@00675800`
    /// hands this back untouched for every row but the nine its national
    /// and general arms name (§6.9).
    pub job_time: i16,
    /// `FLAGS`, the letters `a`..`m` of the file's own legend as bits
    /// `0..12`. `do_cast` splits on `& 0xe` — `b` units, `c` buildings,
    /// `d` an area — which is what makes a craft *targeted*.
    pub flags: u32,
}

impl SpellType {
    /// `spell_flags & 0xe`: this craft takes a target, so `do_cast` walks
    /// its other half.
    pub fn targeted(&self) -> bool {
        self.flags & 0xe != 0
    }
}

/// How far above the commerce cap `find_gather_spot` believes the Dutch
/// interest bonus (`has_tribe_bonus(0x16)`) can carry a good's income, in
/// sixteenths — `0x640` at `find_gather_spot@005f5170:191`.
///
/// `Leader::do_gather` spells the same headroom `dutch_interest_cap × 16`
/// out of `GameAccess::constants`; the search hard-codes it, so it is a
/// literal here too rather than a [`crate::tuning::Tuning`] slot. Nothing in
/// this simulation pays the bonus yet, so this term only ever moves the
/// *choice* of building, never the income.
const DUTCH_INTEREST_HEADROOM: i32 = 0x640;

/// `UnitOrder::flags` bits (`docs/ORDERS.md` §1.3).
pub mod flag {
    /// The top segment of the unit's path stack is this move's.
    pub const PATHED: u8 = 0x1;
    pub const FLEEING: u8 = 0x2;
    /// This order is an intent, not a transit leg: `get_action` stops on it.
    pub const ACTION: u8 = 0x4;
    /// A DEFENSIVE unit's "this move is my post".
    pub const POST: u8 = 0x8;
    /// An attack order's "re-target requested".
    pub const RETARGET: u8 = 0x10;
}

/// `QueuePos` — where an order goes (§1.5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueuePos {
    /// Ctrl-click: the new order runs now, the old current resumes after it.
    First,
    /// Shift-click: after everything queued.
    Last,
    /// Plain click: everything is killed first.
    New,
}

/// `MoveOrder`'s kind: which `do_job` case steps it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoveKind {
    MoveTo,
    AttackTo,
    ExploreTo,
    FleeTo,
}

/// The fields of `MoveOrder` that are live on open ground (§4.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MoveOrder {
    pub kind: MoveKind,
    /// The destination, snapped to its 48-unit cell centre.
    pub dest: Pos,
    /// The facing to apply on arrival — the direction from the unit to the
    /// target at order time. A group move overrides it with the formation's
    /// own bearing plus the slot's packed byte (`docs/GROUPS.md` §6.6 step
    /// 6).
    pub angle: Angle,
    /// `MoveOrder +0x28 facing` — the mirror the **formation** this order
    /// belongs to was laid out with, or `None` for the −1 every plain move
    /// carries (`Unit::add_move_order@00616ed0` passes it literally).
    ///
    /// It is not read while the order runs. It is read when the order
    /// *dies*: `Unit::kill_current_order` writes it back onto the group as
    /// `GroupData::facing` when the unit is the group's leader, so the next
    /// formation starts from the mirror the last one used
    /// (`docs/GROUPS.md` §6.3).
    pub facing: Option<bool>,
    /// "I have a current waypoint."
    pub has_waypoint: bool,
    /// The current waypoint — what the step walks toward.
    pub waypoint: Pos,
    /// Where the last straight-line plan was made.
    pub last: Option<Pos>,
    pub pause: i32,
    pub timer: i32,
    /// `MoveOrder +0x3c/+0x40 coll_x/coll_y` — the point the last
    /// collision refused, which `resolve_unit_collision` sidesteps from
    /// (`docs/COLLISION.md` §4.3, §6 step 4). The dump prints the pair.
    pub coll: Option<Pos>,
    /// `Some` when this is a **`GroupMoveOrder`** rather than a plain
    /// `MoveOrder` — the order `Group::action_move_near`'s step 6 hands a
    /// land formation of two or more, and the one `Unit::do_group_move`
    /// steps (`docs/ORDERS.md` §8.3).
    pub group: Option<GroupMove>,
}

/// The fields a `GroupMoveOrder` carries beyond its `MoveOrder` base —
/// the `GROUPORDER` block's four plus its own `in_group` (`docs/GROUPS.md`
/// §12.1, `docs/ORDERS.md` §8.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GroupMove {
    /// `+0x54`/`+0x58 oxx`/`whose` — the **leader** the formation was laid
    /// out around, which is `find_leader`'s answer and not `list[0]`.
    pub leader: usize,
    /// `+0x5c id`, shared by every member's order:
    /// `(group.id + frame × 10) × 100 + group.order_num`.
    pub id: i64,
    /// `+0x60 form_id` — my index into the group's own arrays. Rewritten
    /// from `list` on every frame the follower arm runs.
    pub form_id: usize,
    /// `+0x64 group_angle` — the formation's bearing plus this slot's
    /// packed byte, the same value the `MoveOrder`'s own angle carries.
    pub group_angle: Angle,
    /// `+0x68 in_group` — "I am walking to my slot rather than to a
    /// waypoint of the leader's".
    pub in_group: bool,
}

/// One entry of the unit's path stack.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PathData {
    pub to: Pos,
    pub tolerance: i32,
    pub flags: u8,
}

/// `PathData::flags`.
pub mod path_flag {
    /// The final waypoint of an order's segment — the goal.
    pub const FINAL: u8 = 0x1;
    pub const SIDESTEP: u8 = 0x2;
    /// Turn in place before walking to this one.
    pub const TURN_FIRST: u8 = 0x4;
    /// The same bit, as the pathfinder writes it: this waypoint boards a
    /// transport (`docs/PATHFINDER.md` §7). One bit, two producers.
    pub const TRANSPORT: u8 = 0x4;
    /// A `go_around_building` mid-detour point — the turn-in corner between
    /// the lane the unit is in and the one it is stepping into. It
    /// suppresses the collision test entirely (`docs/ORDERS.md` §4.1).
    pub const DETOUR: u8 = 0x8;
    /// **A road was laid on this waypoint's tile.** Only
    /// `Caravan::build_road@0073db10` writes it, on every node of a trade
    /// route's plan that is not open water, and nothing in the executable
    /// reads it back — the road stack carries it into the unit's own path
    /// when a leg starts (`docs/CARAVAN.md` §5.3).
    pub const ROAD: u8 = 0x20;
}

/// The fields of `GatherOrder` (§1.1, §6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GatherOrder {
    pub building: usize,
    /// The resource tile a woodcutter or miner is working, in tiles.
    pub tile: Option<Pos>,
    pub wait: i32,
    /// Non-flat: heading to or at the camp (true) vs out at the tile.
    pub goto_build: bool,
    pub dist_mod: i32,
    /// The unit has arrived; the economy counts only these.
    pub been_there: bool,
}

/// The fields of `AttackOrder` the order keeps; the target and `mandatory`
/// stay in [`combat::State`], which `fight.rs` reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttackOrder {
    pub defensive: bool,
    pub def: Option<Pos>,
    pub in_range: bool,
    pub ever_in_range: bool,
    /// "Has not struck yet" — 1 at creation, cleared by the first strike.
    pub new_ord: bool,
}

/// The fields of `CastOrder` this crate keeps (§1.2's value 14).
///
/// The target half — `+0x8`/`+0xc` the object and its owner, `+0x10` its
/// `uid`, `+0x14`/`+0x18` the ground point — is absent because the one
/// spell modelled is untargeted: `set_new_location` queues it with
/// `(-1, -1, -1, -1)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CastOrder {
    /// `+0x20` — the spell's `TypeIndex` (see [`spell`]).
    pub spell: i32,
    /// `+0x1c` — "the cost has been taken", so a cast that waits out a
    /// job time pays once rather than once a frame.
    pub paid: bool,
}

/// The fields of `TradeOrder` the road half of a trade route reads
/// (`docs/CARAVAN.md` §3). The **legs** — `+0x20 loaded`, the walk between
/// the two cities, the wealth it pays — are not modelled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TradeOrder {
    /// `+0x8`/`+0xc` — the city the order was issued against: the nearest
    /// allied city that could take another route (`think_caravan`).
    pub home: usize,
    /// `+0x14`/`+0x18` — the far city, `-1` until `do_trade` picks one.
    pub dest: Option<usize>,
    /// `+0x1c` — the route has been established, so the selection loop is
    /// not run again.
    pub started: bool,
    /// `+0x20 loaded` — the caravan is carrying, so the leg it is on runs
    /// to the **home** city; cleared there, and set again at the far one
    /// (`docs/CARAVAN.md` §7.1).
    pub loaded: bool,
}

/// The order kinds this crate implements.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Body {
    Move(MoveOrder),
    Trade(TradeOrder),
    Build(usize),
    Repair(usize),
    Garrison { building: usize, search: bool },
    Gather(GatherOrder),
    Attack(AttackOrder),
    Cast(CastOrder),
    Think,
}

/// One order in a unit's list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Order {
    pub flags: u8,
    pub body: Body,
}

impl Order {
    /// `get_type()` — the `OrderIndex` value.
    pub const fn index(&self) -> u8 {
        match self.body {
            // **A grouped move is its own class, and so its own
            // `OrderIndex`** (`docs/ORDERS.md` §1.2). The original has no
            // "move with a group beside it": `Group::action_move_near`
            // asks `get_new_order` for a `GroupMoveOrder` or a
            // `GroupAttackToOrder`, whose `get_type` return `GROUP_MOVE`
            // and `GROUP_ATTACK_TO`, and `ungroup_move_order` swaps the
            // object for a plain `MoveOrder`/`AttackToOrder` — which is
            // why the dump's header changes on the ungroup frame. This
            // crate keeps one class and a [`MoveOrder::group`], so the
            // `Option` is what stands in for the original's class, and
            // this is the only place that matters. There is no grouped
            // explore or flee: `Group::action_move_near`'s own gate is
            // `MOVE_TO`/`ATTACK_TO` (`crate::Sim::group_move_near`).
            Body::Move(m) => match (m.kind, m.group.is_some()) {
                (MoveKind::MoveTo, false) => index::MOVE_TO,
                (MoveKind::MoveTo, true) => index::GROUP_MOVE,
                (MoveKind::AttackTo, false) => index::ATTACK_TO,
                (MoveKind::AttackTo, true) => index::GROUP_ATTACK_TO,
                (MoveKind::ExploreTo, _) => index::EXPLORE_TO,
                (MoveKind::FleeTo, _) => index::FLEE_TO,
            },
            Body::Trade(_) => index::TRADE_ROUTE,
            Body::Build(_) => index::BUILD_AT,
            Body::Repair(_) => index::REPAIR,
            Body::Garrison { .. } => index::GARRISON,
            Body::Gather(_) => index::GATHER,
            Body::Attack(_) => index::ATTACK,
            Body::Cast(_) => index::CAST_SPELL,
            Body::Think => index::THINK,
        }
    }

    pub const fn is_move(&self) -> bool {
        matches!(self.body, Body::Move(_))
    }

    /// A move order's destination — the `+0x4`/`+0x8` pair
    /// `Group::action_attack` reads off the current order (§10).
    pub const fn move_dest(&self) -> Option<Pos> {
        match self.body {
            Body::Move(m) => Some(m.dest),
            _ => None,
        }
    }

    pub const fn has(&self, f: u8) -> bool {
        self.flags & f != 0
    }

    /// A plain transit move: a move without the action bit — what
    /// `get_action` walks past.
    pub const fn is_transit(&self) -> bool {
        self.is_move() && !self.has(flag::ACTION)
    }

    pub(crate) const fn move_mut(&mut self) -> Option<&mut MoveOrder> {
        match &mut self.body {
            Body::Move(m) => Some(m),
            _ => None,
        }
    }
}

/// What `Unit::do_move` reports back to `work`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Did {
    Nothing,
    Something,
}

/// `do_group_move`'s angle window (`5e815d`–`5e816d`): the wrapped
/// difference folded into half a turn and compared against `0x55555555` —
/// **a third of a turn**, 120°, not the quarter `reversing` uses.
pub(crate) fn within_third(a: Angle, b: Angle) -> bool {
    let d = (a.0 as u32).wrapping_sub(b.0 as u32);
    let d = if d > 0x8000_0000 { !d } else { d };
    d < 0x5555_5555
}

/// `ACCEL_CONSTRUCT`-independent literals of the mechanic, position units.
const SNAP: i32 = 0x30;
const SNAP_CENTRE: i32 = 0x18;
const HALF_TILE: i32 = 0x60;
const TILE: i32 = 0xc0;
const CELL: i32 = 0x300;
/// `Game::init_data@0058dca0`: `num_def_builds = 200`, the per-player
/// building slot count, and the threshold `Objects::find_builds` compares
/// its circle against.
const NUM_DEF_BUILDS: usize = 200;
/// The 48-unit snap, `div_3_table[v >> 4] * 0x30 + 0x18`.
///
/// Two callers in the original and they are the same arithmetic: every move
/// order's destination (`Unit::add_move_facing_order`, §4.3) and **every
/// unit's starting position** — `Unit::init@00612100:69` snaps both
/// coordinates before it hands them to `Object::init` and to
/// `set_new_location`, so an object is never born off the grid however
/// unrounded the point its maker computed. `div_3_table` is `floor(i / 3)`
/// and the coordinates are non-negative, so `div_euclid` is the same table.
pub(crate) const fn snapped(p: Pos) -> Pos {
    Pos::new(
        p.x.div_euclid(SNAP) * SNAP + SNAP_CENTRE,
        p.y.div_euclid(SNAP) * SNAP + SNAP_CENTRE,
    )
}

/// `find_path`'s "far and reachable: the pathfinder's job" rule, world cells.
const STRAIGHT_LINE_CELLS: i32 = 4;
/// The working-the-tile radius of a woodcutter or miner.
const AT_TILE: i32 = 0x140;
/// The oil-well stance offset.
const OILWELL_OFFSET: i32 = 0x120;
/// Adjacency: `attack_dist < 0x60`.
const ADJACENT: i32 = 0x60;
/// The herder's phase — `Unit::do_gather@005ef2a0:5efd8c`. A citizen
/// gathering at a **pasture** re-picks its spot on the frames where
/// `(o · 7 + frame + who) % 256` is zero, and does nothing else at all
/// (§6.5). Both numbers are the listing's.
const PASTURE_PHASE: i64 = 0x100;
const PASTURE_STRIDE: i64 = 7;

/// `unit_masks & 0x78000000` — the four carrying-walk bits
/// ([`crate::Unit::carry`]), named as `Guy::set_anim`'s walk arm tests
/// them and written only by `Unit::do_non_flat_gather` (§6.4).
pub(crate) const CARRY_WITH_WOOD: u32 = 0x0800_0000;
pub(crate) const CARRY_TO_WOOD: u32 = 0x1000_0000;
pub(crate) const CARRY_WITH_ORE: u32 = 0x2000_0000;
pub(crate) const CARRY_TO_ORE: u32 = 0x4000_0000;
/// The nibble `& 0x87ffffff` clears whole.
const CARRY_ANY: u32 = CARRY_WITH_WOOD | CARRY_TO_WOOD | CARRY_WITH_ORE | CARRY_TO_ORE;

/// The wood machine's **three** direct draw sites, under the original's own
/// offsets from `Unit::do_non_flat_gather@005f0170`, §6.4. [`Sim::mark`]
/// writes them into [`Sim::phase_marks`], which is what keeps them from
/// being read as the stand that precedes them (`docs/SYNC.md` §5).
///
/// The middle one is the trap. The two waits are **not** the same branch:
/// `+0xcc3` is the chopping guy's `% 100 + 300`, reached only through the
/// `cur_anim == CHAR_CHOP_WOOD` test that stands in front of the tile
/// arithmetic, and `+0xdad` is the arrival frame's `% 50 + 100`, reached
/// only when the guy is *not* yet chopping. A woodcutter takes the first
/// on every reroll after its first frame at the tile and the second
/// essentially never — so a merged branch that rolls `% 50 + 100` sends it
/// back to the camp two hundred frames early (`docs/JOURNAL.md`,
/// 2026-08-31).
pub const SITE_TILE_WAIT: &str = "Unit::do_non_flat_gather+0x54b";
pub const SITE_WORK_WAIT: &str = "Unit::do_non_flat_gather+0xcc3";
pub const SITE_ARRIVE_WAIT: &str = "Unit::do_non_flat_gather+0xdad";

/// The farmer's cell re-pick — `Unit::do_gather@005ef2a0`'s two
/// `GameAccess::rnd(4)` calls, the pair that follows the cell-state switch
/// when the state and the animation disagree (§6.5).
///
/// Both draws are the **same** address: `GameAccess::rnd` is a frameless
/// helper whose `Random::get` call returns to `+0x20` whoever asked, so the
/// trace names the pair once and the `ebp` walk skips straight past both it
/// and `do_gather` to `Unit::do_job+0x67`. One mark therefore carries the
/// two draws (`docs/SYNC.md` §3.10).
pub const SITE_FARM_CELL: &str = "GameAccess::rnd+0x20 < Unit::do_job+0x67";

/// `Unit::do_move@005f7b30:599`'s grid draw — the call is at `005f89af`,
/// so the site is `+0xe84` (§4.4, `docs/SYNC.md` §6's frame-3 item). The
/// sim reaches it where the original does not, and naming it is what turns
/// that from an unattributed unit-loop draw into a row.
///
/// It is also Great Lakes' parting frame, and there in the other
/// direction: on 6994 the original spends two of these under
/// `Unit::do_attack_to` and this crate none, because the units that spend
/// them are a squad the original has just given an order to and this crate
/// leaves standing — `docs/ARMY.md` §16.7, not a pathfinder question.
pub const SITE_MOVE_GRID: &str = "Unit::do_move+0xe84";

/// The 31 bearings of one ring of `find_nearby_spot`, as multiples of a
/// sixteenth of a turn from the base angle; `|k| >= 8` adds a thirty-second.
const BEARINGS: [i32; 31] = [
    0, 1, -1, 2, -2, 3, -3, 4, -4, 5, -5, 6, -6, 7, -7, 8, -8, 9, -9, 10, -10, 11, -11, 12, -12,
    13, -13, 14, -14, 15, -15,
];

/// Which of `find_nearby_spot`'s two collision halves a call site asks for
/// (`docs/ORDERS.md` §10).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Coll {
    /// `FILTER_NOT_ME` / `FILTER_CAN_COLLIDE` with a real `(o, who)` and no
    /// squad — every build, repair, gather, garrison, idle and stable site:
    /// `Objects::find_collision` and then `find_ordered_collision`.
    ///
    /// A **squad** placement wears the same name and takes the other path:
    /// [`Sim::find_nearby_spot_squad`] passes `uber = 1`, and the
    /// original's `bVar17` is written only in the arm `uber_unit != 0`
    /// jumps over (`61df3e`).
    Pairwise,
    /// Accept any passable candidate. The original's `nocoll != 0`, and —
    /// as a stated seam — its `FILTER_ALL` general path, whose
    /// `big_radius + r_coll` circle is modelled only for the two forms
    /// that reach it here (the `(-1, -1)` cast and the squad).
    None,
}

/// Who is asking `find_nearby_spot` — `this` is always the unit **type**,
/// but every call site but one hands it a `(not_o, not_who)` pair as well,
/// and the pair is what the collision half exempts. `cast_transport` passes
/// `(-1, -1)` (`docs/TRANSPORT.md` §6), and that is [`Seeker::Type`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Seeker {
    Unit(usize),
    Type(usize),
}

/// Which worker kind a unit type is, for the gather chain and `think_peasant`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Worker {
    #[default]
    None,
    Citizen,
    Scholar,
}

impl Unit {
    /// Discard waypoints through the current segment's final waypoint.
    ///
    /// This is the path-only operation used by order teardown (ORDERS §3.2).
    /// It can also be replayed on a unit without constructing a world or sim.
    pub fn discard_current_path_segment(&mut self) {
        while let Some(p) = self.path.pop() {
            if p.flags & path_flag::FINAL != 0 {
                break;
            }
        }
    }
}

impl Sim {
    // ------------------------------------------------------------------
    // The list
    // ------------------------------------------------------------------

    /// The current order — `Unit::update_order`, `head->prev`.
    pub fn current_order(&self, u: usize) -> Option<&Order> {
        self.units[u].orders.front()
    }

    /// `get_speed(x, y, 0)` — the virtual at slot `+0x17c` that both the
    /// unit step (`do_move@005f7b30`) and the pull-back
    /// (`find_path@005fb910`) call for the step length.
    ///
    /// For a player's unit this is `UnitData::get_speed@00608720`, whose
    /// three layers `docs/MOVEMENT.md`, "The speed pipeline", describes and
    /// which this crate takes as the cached `Movement::speed`.
    ///
    /// **An animal replaces it whole.** `AnimalData::get_speed@005d8380`
    /// occupies the same slot on `Animal`'s and `AnimalData`'s vtables, and
    /// it does not call `UnitData::get_speed` at all — it calls
    /// `UnitData::speed` (the aura layer) and then, on a land or sea
    /// animal:
    ///
    /// - if the current order `is_move` (slot `+0x14`) and the animal is
    ///   more than `0x180` from that order's **goal** — `get_move_order`
    ///   (slot `+0xb8`), then `MoveOrder +0x4/+0x8`, the ordered point and
    ///   not the current waypoint — the speed becomes `speed * 3 / 2`,
    ///   truncated toward zero;
    /// - and the result is floored at 3.
    ///
    /// An **air** animal — a bird, `type +0x218 == 2` — returns before
    /// both: no hurry and no floor.
    ///
    /// So none of `UnitData::get_speed`'s own layers reach an animal: not
    /// the order scale, not the `unit_masks & 0x10` halving, not the
    /// `0x800` tile, not the group cap. The two things `do_move` applies
    /// *after* the virtual — `ai_speed` and the modern-infantry `5/4` —
    /// still do.
    ///
    /// The `0x180` is the same threshold `Animal::do_idle` measures its
    /// wander with ([`crate::anim::WANDER_NEAR`] is `0x181`, the strict
    /// `<`), but it is a **different distance**: `do_idle` measures from
    /// the herd's centre, this from the order's goal. `docs/MOVEMENT.md`,
    /// "The animal's own `get_speed`".
    ///
    /// SEAM: the original decides this by class — a unit is an `Animal` or
    /// it is not — and this crate has no class, so it asks
    /// [`crate::Unit::is_gaia`], the same stand-in
    /// [`Sim::do_idle`](Self::do_idle) uses to reach `Animal::do_idle`.
    pub fn get_speed(&self, u: usize) -> i32 {
        let unit = &self.units[u];
        let speed = unit.movement.speed;
        if unit.is_gaia() {
            if unit.kind.domain == crate::attrition::Domain::Air {
                return speed;
            }
            let far = self
                .current_order(u)
                .and_then(Order::move_dest)
                .is_some_and(|to| vector_dist(unit.pos.x - to.x, unit.pos.y - to.y) > 0x180);
            let speed = if far { speed * 3 / 2 } else { speed };
            return speed.max(movement::SPEED_FLOOR);
        }
        // The **action**'s own scale, `00608743`–`00608777`. `get_action`
        // is the first order that is neither a transit move nor a
        // `CHANGE_FORM`; [`Sim::action_of`] is that walk without the
        // `CHANGE_FORM` half, which no capture reaches.
        let action = self
            .action_of(u)
            .and_then(|i| unit.orders.get(i))
            .map_or(index::NONE, Order::index);
        let mut speed = match action {
            index::ATTACK => speed * 9 / 8,
            index::GUARD if self.ai_driven(unit.owner) => speed * 10 / 8,
            index::GUARD => speed * 9 / 8,
            _ => speed,
        };
        if unit.kind.domain == crate::attrition::Domain::Land {
            // SEAM: `unit_masks & 0x10` halves it first —
            // `Unit::target_opportunity`'s "moving in contact with a
            // target", set during the frame and cleared by `work` at the
            // end of it, so no dump can ever print it. This crate's
            // [`Sim::target_opportunity`](crate::Sim::target_opportunity)
            // is the retaliation alone and sets no such bit.
            if self.on_river(unit.pos) {
                speed /= 2;
            }
            // SEAM: `has_general(0, 0x162)` doubles a siege type's speed;
            // no capture has a general.
        }
        // SEAM: the group cap ([`movement::group_capped`]) — this crate's
        // `Group` carries no speed.
        speed.max(movement::SPEED_FLOOR)
    }

    /// `UnitData::get_speed`'s slow-ground test, `0060879b`–`006087d8`: the
    /// **tile** the unit stands on carries `0x800`, and the unit's own
    /// `z_internal` is not above zero.
    ///
    /// Both halves are the position the unit is standing at when the step
    /// is computed. `z_internal` is written by
    /// `SubObject::set_new_location@00662680` — `TerrainOut::find_tcoord_z`
    /// of the tile it has just moved to — which is
    /// [`World::tile_z`](crate::world::World::tile_z) here, so the stored
    /// field and the tile read are the same tile and this needs no field of
    /// its own. A world with no height grid reads zero everywhere, which is
    /// the flat harness world and passes the test.
    ///
    /// The `^ 0x63637` all over the decompilation of this and of
    /// `SubObjectData::log_data` is not a predicate: **the three
    /// coordinates are stored XORed with `0x63637`** and every reader
    /// decodes them, the gamelog's own printer included. So
    /// `(z ^ 0x63637) > 0` skipping the halving is `z > 0` skipping it.
    pub(crate) fn on_river(&self, pos: Pos) -> bool {
        let t = pos.tile();
        self.world.tile_z(t) <= 0 && self.world.tile_mask(t) & tile::RIVER != 0
    }

    /// `UnitData::order_type`: the current order's `OrderIndex`, `NONE` for
    /// an empty list.
    pub fn order_type(&self, u: usize) -> u8 {
        self.current_order(u).map_or(index::NONE, Order::index)
    }

    /// `UnitData::get_action`: the position in the list of the first order
    /// that is not a transit move — the intent under the pathing legs.
    pub fn action_of(&self, u: usize) -> Option<usize> {
        self.units[u].orders.iter().position(|o| !o.is_transit())
    }

    /// `Unit::update_action`: `orders_x/y` = the final destination of the
    /// leading run of transit moves, `dest_angle` = the last one's angle;
    /// returns the action's position (§3.3).
    pub fn update_action(&mut self, u: usize) -> Option<usize> {
        let unit = &mut self.units[u];
        unit.orders_pos = unit.pos;
        unit.movement.des_angle = unit.movement.heading;
        let mut action = None;
        for (i, o) in unit.orders.iter().enumerate() {
            match o.body {
                Body::Move(m) if !o.has(flag::ACTION) => {
                    unit.orders_pos = m.dest;
                    unit.movement.des_angle = m.angle;
                }
                Body::Move(m) => {
                    // A move that is itself the action: `update_action`
                    // copies it once more and stops.
                    unit.orders_pos = m.dest;
                    unit.movement.des_angle = m.angle;
                    action = Some(i);
                    break;
                }
                _ => {
                    action = Some(i);
                    break;
                }
            }
        }
        action
    }

    /// The generic enqueue (§3.1): `QUEUE_NEW` clears first, then the order
    /// is appended (`Last`, `New`) or rotated to the front (`First`).
    pub(crate) fn enqueue_order(&mut self, u: usize, order: Order, pos: QueuePos) {
        self.enqueue(u, order, pos);
    }

    fn enqueue(&mut self, u: usize, order: Order, pos: QueuePos) {
        match pos {
            QueuePos::New => {
                self.units[u].path.clear();
                self.close_orders(u);
                self.clear_partial_path(u);
                self.update_action(u);
                self.units[u].orders.push_back(order);
            }
            QueuePos::Last => self.units[u].orders.push_back(order),
            QueuePos::First => {
                self.clear_partial_path(u);
                self.units[u].orders.push_front(order);
            }
        }
        self.update_action(u);
    }

    /// `Unit::clear_partial_path@005e3920` — the drop of a suspended
    /// pathfinder search, and nothing else.
    ///
    /// The original's body is the suspended pathfinder search and nothing
    /// else: `UnitData +0x104` and `+0x10c`, the two `Tree<PathNode *>`s
    /// it hands back to `PathFinder::kill_tree` and the recycler, and the
    /// `Tree<CollBlock *>` beside them. It does **not** touch
    /// `unit_masks` — grep the function for `0x68` and there is no hit —
    /// so an order teardown, a `close_orders`, an animal's give-up in
    /// `resolve_unit_collision` step 0 and every other caller leave the
    /// verified-line bit exactly as `do_move` last wrote it.
    ///
    /// This crate used to clear [`crate::Unit::line_ok`] here, and that
    /// one line was most of `line_ok`'s wrong lifecycle: a unit that
    /// finished a walk kept the bit **set** over there and lost it here,
    /// which is 335 of run65's 450 unit-frames (`docs/MOVEMENT.md`,
    /// "The verified line's lifecycle", 2026-09-05).
    ///
    /// ~~SEAM: the suspended search itself is not modelled, so there is
    /// nothing left to free.~~ It is, since item 301: the stash is
    /// [`crate::Unit::search`] and this is the function that drops it.
    pub(crate) fn clear_partial_path(&mut self, u: usize) {
        self.units[u].search = None;
    }

    /// `Unit::kill_current_order(0)` (§3.2): the per-kind teardown, then the
    /// pop, the path segment, `update_action`.
    pub fn kill_current_order(&mut self, u: usize) {
        let Some(order) = self.units[u].orders.front().copied() else {
            return;
        };
        // Before the teardown: a dying **move** carries the mirror its
        // formation was laid out with, and the group's leader hands it back
        // (`docs/GROUPS.md` §6.3). This is the write that makes a group's
        // second right-click start from the first click's mirror instead of
        // from whatever the leader's turning left behind.
        if let Body::Move(m) = order.body
            && let Some(f) = m.facing
        {
            self.hand_back_facing(u, f, m.angle);
        }
        match order.body {
            Body::Gather(g) => {
                let who = self.units[u].owner;
                self.ledgers[who as usize].dirty = true;
                if self.buildings.get(g.building).is_some_and(|b| b.alive)
                    && self.buildings[g.building].owner == who
                {
                    self.remove_gatherer(g.building, u);
                }
                // `kill_current_order:107`, inside the same arm and after
                // the chain: the carrying walk goes with the job.
                self.units[u].carry &= !CARRY_ANY;
            }
            Body::Attack(_) => {
                // The order is the target's home; dropping it drops the target.
                let unit = &mut self.units[u];
                unit.combat.target = None;
                unit.combat.mandatory = false;
            }
            _ => {}
        }
        self.units[u].orders.pop_front();
        if order.is_move() && order.has(flag::PATHED) {
            self.kill_current_path(u);
        }
        self.units[u].movement.dest = None;
        self.clear_partial_path(u);
        self.update_action(u);
    }

    /// Remove the order the cursor sits on, `i` places behind the current
    /// one. Only `check_build_order` walks a cursor, and only onto a
    /// `BUILD_AT`, which carries no per-kind teardown — so this is
    /// [`Sim::kill_current_order`]'s generic tail applied at an index.
    fn remove_order_at(&mut self, u: usize, i: usize) {
        if self.units[u].orders.remove(i).is_none() {
            return;
        }
        self.units[u].movement.dest = None;
        self.clear_partial_path(u);
        self.update_action(u);
    }

    /// `Unit::kill_current_path@005e31d0`: pop the stack until an entry
    /// with the final flag is popped — **and then free the suspended
    /// search**.
    ///
    /// The `clear_partial_path` call sits inside the function's own `0 <
    /// length` guard, so an already-empty stack frees nothing; that guard
    /// is the whole of the difference between this and
    /// [`Sim::kill_current_order`], which clears unconditionally.
    ///
    /// It is what ends a suspended search that no longer has an order to
    /// serve. `Unit::ungroup_move_order@005fd140` calls this on every
    /// member that is not the leader (`docs/GROUPS.md`, and
    /// `docs/PATHFINDER.md` §18.4), so the frame a formation degrades is
    /// the frame each follower's stash goes — and the follower re-plans on
    /// the next frame instead of standing in `do_move`'s suspended block
    /// for the rest of the capture.
    pub(crate) fn kill_current_path(&mut self, u: usize) {
        if self.units[u].path.is_empty() {
            return;
        }
        self.units[u].discard_current_path_segment();
        self.clear_partial_path(u);
    }

    /// `Unit::close_orders`: kill everything, oldest first.
    pub fn close_orders(&mut self, u: usize) {
        while !self.units[u].orders.is_empty() {
            self.kill_current_order(u);
        }
    }

    /// `Unit::clear_orders`: the `QUEUE_NEW` clear without the add.
    pub fn clear_orders(&mut self, u: usize) {
        self.units[u].path.clear();
        self.close_orders(u);
        self.clear_partial_path(u);
        self.update_action(u);
    }

    // ------------------------------------------------------------------
    // The adders
    // ------------------------------------------------------------------

    /// `Unit::add_move_order` → `add_move_facing_order` (§4.3): the 48-unit
    /// snap, the angle at order time, the action bit from the caller.
    ///
    /// **The angle is taken to the caller's point, not to the snapped one**
    /// (§4.3). The listing at `00616ed0` computes the two `find_angle`
    /// arguments in registers from the *arguments* — `ecx = x − (this->
    /// field_0x10 ^ 0x63637)`, `edx = y − (field_0x14 ^ 0x63637)` — and only
    /// then indexes `div_3_table` for the pair it pushes. So a caller that
    /// hands over an unsnapped point gets the bearing to that point and a
    /// destination 48-snapped away from it, and the two differ by up to
    /// twenty-four units on each axis. A farmer's re-picked cell is
    /// `t · 0xc0 + 0x60` — the centre of a 192-unit tile, which is never a
    /// 48-unit cell centre — so every farm walk in the game ends facing
    /// somewhere the snapped bearing does not name (`docs/ORDERS.md` §4.3,
    /// `docs/SYNC.md` §3.12).
    pub fn add_move_order(
        &mut self,
        u: usize,
        to: Pos,
        kind: MoveKind,
        pos: QueuePos,
        action: bool,
    ) {
        let here = self.units[u].pos;
        let angle = find_angle(to.x - here.x, to.y - here.y);
        self.add_move_facing_order(u, to, kind, pos, action, angle, None, false);
    }

    /// `Unit::add_move_facing_order@005e55c0` (§4.3), and
    /// `add_group_move_order@005e4710` where they agree: the same snap and
    /// the same record, with the **caller's** angle and the formation's own
    /// `facing` instead of the bearing `add_move_order` derives.
    ///
    /// `add_move_order` is that call with `angle = find_angle(dest − here)`
    /// and `facing = −1`, which is exactly what the listing at `616ed0`
    /// pushes. The two arguments only ever come from a group move
    /// (`docs/GROUPS.md` §6.6 step 6), which is why the ordinary adder does
    /// not take them.
    ///
    /// `pathed` is the original's `param_5`, and it writes [`flag::PATHED`]
    /// straight into the order at birth (`005e55c0`: `*pbVar1 |= 1`). Only
    /// the group move passes it: `action_move_near` plans the whole chain
    /// itself a few lines later (`docs/GROUPS.md` §6.7), so the order it
    /// hands out must not send `do_move` off to plan again.
    #[allow(clippy::too_many_arguments)] // the original's twelve, minus the seven this does not model
    pub fn add_move_facing_order(
        &mut self,
        u: usize,
        to: Pos,
        kind: MoveKind,
        pos: QueuePos,
        action: bool,
        angle: Angle,
        facing: Option<bool>,
        pathed: bool,
    ) {
        self.add_move_facing_order_grouped(u, to, kind, pos, action, angle, facing, pathed, None);
    }

    /// `Unit::add_group_move_order@005e4710` and `add_move_facing_order`
    /// in one — the two write the **same** record, and the group adder
    /// adds the `GroupOrder` base's five fields on top (`docs/GROUPS.md`
    /// §6.6 step 6). It writes the angle a second time, into
    /// `group_angle`, which is why the two are equal on every record run31
    /// holds; `pathed` is 1 from both.
    #[allow(clippy::too_many_arguments)]
    pub fn add_move_facing_order_grouped(
        &mut self,
        u: usize,
        to: Pos,
        kind: MoveKind,
        pos: QueuePos,
        action: bool,
        angle: Angle,
        facing: Option<bool>,
        pathed: bool,
        group: Option<GroupMove>,
    ) {
        let dest = snapped(to);
        let order = Order {
            flags: if action { flag::ACTION } else { 0 } | if pathed { flag::PATHED } else { 0 },
            body: Body::Move(MoveOrder {
                kind,
                dest,
                angle,
                facing,
                has_waypoint: false,
                waypoint: dest,
                last: None,
                pause: 0,
                timer: 0,
                coll: None,
                group,
            }),
        };
        self.enqueue(u, order, pos);
    }

    /// `Unit::add_build_order`.
    pub fn add_build_order(&mut self, u: usize, b: usize, pos: QueuePos, action: bool) {
        self.units[u].was_builder = true;
        let order = Order {
            flags: if action { flag::ACTION } else { 0 },
            body: Body::Build(b),
        };
        self.enqueue(u, order, pos);
    }

    /// `Unit::add_repair_order`.
    pub fn add_repair_order(&mut self, u: usize, b: usize, pos: QueuePos, action: bool) {
        self.units[u].was_builder = true;
        let order = Order {
            flags: if action { flag::ACTION } else { 0 },
            body: Body::Repair(b),
        };
        // The one adder that is **not** §3.1's common shape: `add_repair_
        // order@005e4ff0` has no `QUEUE_FIRST` tail at all — after the list
        // add at `0x5e51f0` come only `update_action` and `ret 0x10`, where
        // `add_build_order` has an explicit `cmp [ebp+0x10],0` →
        // `clear_partial_path; head = head->next`. No caller passes
        // `QUEUE_FIRST`, so this is fidelity, not a live bug.
        // (`docs/audit/2026-08-21-orders.md` R3 P1.)
        debug_assert!(
            pos != QueuePos::First,
            "add_repair_order has no QUEUE_FIRST branch in the original"
        );
        let pos = if pos == QueuePos::First {
            QueuePos::Last
        } else {
            pos
        };
        self.enqueue(u, order, pos);
    }

    /// `Unit::add_garrison_order(o, who, search, pos, action)`.
    pub fn add_garrison_order(
        &mut self,
        u: usize,
        b: usize,
        search: bool,
        pos: QueuePos,
        action: bool,
    ) {
        let order = Order {
            flags: if action { flag::ACTION } else { 0 },
            body: Body::Garrison {
                building: b,
                search,
            },
        };
        self.enqueue(u, order, pos);
    }

    /// `Unit::add_gather_order(o, pos, action)` (§6.2): the unit joins the
    /// building's chain **now**, before it has moved, if the type is a
    /// gatherer; the non-flat fields are set for wood and ore.
    pub fn add_gather_order(&mut self, u: usize, b: usize, pos: QueuePos, action: bool) {
        if pos == QueuePos::New {
            self.units[u].path.clear();
            self.close_orders(u);
            self.clear_partial_path(u);
            self.update_action(u);
        }
        let mut dist_mod = 0;
        if self.is_gather_type(b) {
            self.add_gatherer(b, u);
            if !self.is_flat(b) && self.building_ident(b) != Ident::University {
                dist_mod = if self.building_ident(b) == Ident::Mine {
                    10
                } else {
                    4
                };
            }
        }
        let order = Order {
            flags: if action { flag::ACTION } else { 0 },
            body: Body::Gather(GatherOrder {
                building: b,
                tile: None,
                wait: 0,
                goto_build: true,
                dist_mod,
                been_there: false,
            }),
        };
        self.units[u].orders.push_back(order);
        if pos == QueuePos::First {
            let o = self.units[u].orders.pop_back().expect("just pushed");
            self.clear_partial_path(u);
            self.units[u].orders.push_front(o);
        }
        self.update_action(u);
    }

    /// `Unit::add_attack_order(o, who, pos, mandatory, action)` (§7.1). The
    /// target and `mandatory` live in [`combat::State`]; the order carries
    /// the rest.
    pub fn add_attack_order(
        &mut self,
        u: usize,
        target: Obj,
        pos: QueuePos,
        mandatory: bool,
        action: bool,
    ) {
        if pos == QueuePos::New {
            self.units[u].path.clear();
            self.close_orders(u);
            self.clear_partial_path(u);
            self.update_action(u);
        }
        let unit = &self.units[u];
        let defensive = unit.combat.stance == combat::Stance::Defensive && !action;
        let def = if defensive {
            Some(self.find_def_pos(u))
        } else {
            None
        };
        self.bump_targeted_pub(target, 1);
        let unit = &mut self.units[u];
        unit.combat.target = Some(target);
        unit.combat.mandatory = mandatory;
        let order = Order {
            flags: if action { flag::ACTION } else { 0 },
            body: Body::Attack(AttackOrder {
                defensive,
                def,
                in_range: false,
                ever_in_range: false,
                new_ord: true,
            }),
        };
        match pos {
            QueuePos::First => {
                self.clear_partial_path(u);
                self.units[u].orders.push_front(order);
            }
            _ => self.units[u].orders.push_back(order),
        }
        self.update_action(u);
    }

    /// `UnitData::find_def_pos`: an existing DEFENSIVE attack's post, else a
    /// head move carrying the post flag, else the unit's own quarter-tile
    /// centre.
    fn find_def_pos(&self, u: usize) -> Pos {
        for o in &self.units[u].orders {
            match o.body {
                Body::Attack(a) if a.defensive => {
                    if let Some(p) = a.def {
                        return p;
                    }
                }
                Body::Move(m) if o.has(flag::POST) => return m.dest,
                _ => {}
            }
        }
        let p = self.units[u].pos;
        Pos::new(
            p.x.div_euclid(SNAP) * SNAP + SNAP_CENTRE,
            p.y.div_euclid(SNAP) * SNAP + SNAP_CENTRE,
        )
    }

    /// `Unit::add_cast_order(o, who, x, y, spell, QUEUE_FIRST, 0)@005e4a60`,
    /// in the one shape this crate issues it: `set_new_location`'s
    /// transport conversion, untargeted, `QUEUE_FIRST`, and **without** the
    /// action bit — `param_7` is 0, which clears [`flag::ACTION`] rather
    /// than setting it (`docs/TRANSPORT.md` §6).
    ///
    /// **The pack/unpack rewrite at the head of the original is here**, and
    /// `Unit::think_fish` is what reaches it: `add_cast_order@005e4a60`
    /// re-aims the two generic spells at the caster's own before it builds
    /// the order, so a Fisherman asking for `UNPACK` (`0x28c`) gets
    /// `0x292` — which is the `spell 658` run58's block 4949 prints
    /// (`docs/ORDERS.md` §6.8).
    ///
    /// SEAM: only the `FISHERMEN` and merchant arms are modelled. The
    /// third — a `MACHINEGUN` lineage's `0x28d`/`0x28e` — has no caller
    /// here, and `0x28b` (pack) none at all.
    pub fn add_cast_order(&mut self, u: usize, spell: i32) {
        self.add_cast_order_at(u, spell, QueuePos::First);
    }

    /// The same with the original's `param_6`. `Unit::unpack_merchant`
    /// is the one caller that passes `QUEUE_NEW`
    /// ([`crate::merchant`] §3); every other one this crate reaches
    /// passes `QUEUE_FIRST`.
    pub fn add_cast_order_at(&mut self, u: usize, spell: i32, pos: QueuePos) {
        // The rewrite at `add_cast_order@005e4a60`'s head, in the
        // original's own order: the machine-gun lineage (`is(0x7b, 0)`)
        // first, then the three **exact** merchant ids, then the
        // `FISHERMEN` lineage. The first of the three is the one still
        // unmodelled — no caller here reaches a machine gun's pack.
        let spell = if spell != crate::fish::UNPACK {
            spell
        } else if self.is_merchant(u) {
            spell::UNPACK_MERCHANT
        } else if self.unit_line_is(u, crate::fish::FISHERMEN) {
            crate::fish::UNPACK_FISHERMEN
        } else {
            spell
        };
        let order = Order {
            flags: 0,
            body: Body::Cast(CastOrder { spell, paid: false }),
        };
        self.enqueue(u, order, pos);
    }

    /// `Unit::add_think_order`: the argument is ignored — it is always
    /// rotated to the front, with the action bit.
    pub fn add_think_order(&mut self, u: usize) {
        let order = Order {
            flags: flag::ACTION,
            body: Body::Think,
        };
        self.units[u].orders.push_front(order);
        self.clear_partial_path(u);
        self.update_action(u);
    }

    // ------------------------------------------------------------------
    // The step — `Unit::work` → `do_job`
    // ------------------------------------------------------------------

    /// `Unit::work` (§2.3): the liveness test on the action's target, the
    /// idle reset, then one `do_job` on the front order.
    pub(crate) fn work(&mut self, u: usize, frame: i64) {
        // The recharging-melee gate: a recharging melee unit steps only an
        // order carrying the action bit, and never an ATTACK through here
        // (ATTACK's own entry gate is `fight`'s).
        if let Some(o) = self.current_order(u).copied()
            && o.index() != index::ATTACK
            && self.units[u].combat.recharging != 0
            && self.max_range_of(Obj::Unit(u)) == 0
            && !o.has(flag::ACTION)
        {
            return;
        }
        // Target liveness: a build, repair, garrison or gather whose target
        // died (the `uid` test — indices are never reused here, so it is the
        // alive bit) dies before its `do_*` ever runs.
        if let Some(a) = self.action_of(u) {
            let dead = match self.units[u].orders[a].body {
                Body::Build(b) | Body::Repair(b) | Body::Garrison { building: b, .. } => {
                    !self.buildings.get(b).is_some_and(|bd| bd.alive)
                }
                Body::Gather(g) => !self.buildings.get(g.building).is_some_and(|bd| bd.alive),
                _ => false,
            };
            if dead {
                self.repath(u);
                self.kill_current_order(u);
                return;
            }
        }
        // `Unit::work@0060d180:378`, before the dispatch: the collision
        // cooldown a failed 48-grid search bought counts down
        // (`docs/COLLISION.md` §4.1).
        if self.units[u].safe != 0 {
            self.units[u].safe -= 1;
        }
        if !self.units[u].orders.is_empty() {
            self.units[u].idle = 0;
        }
        // `Unit::work@0060d180:268`: a unit that has an order and is still
        // carrying the idle latch drops it, and a **fisherman or merchant**
        // marks its owner's economy dirty on the way — it has just left
        // `Leader::calc_gather` step 6's walk ([`crate::rares`]).
        if self.order_type(u) != index::NONE && self.units[u].idle_latch {
            self.units[u].idle_latch = false;
            if self.unit_line_is(u, crate::fish::FISHERMEN) || self.is_merchant(u) {
                self.economy_changed(self.units[u].owner);
            }
        }
        // `Unit::work@0060d180:97` — the caravan block, ahead of the
        // dispatch: a linked trade route runs another frame of its road
        // plan, or is ended when the order under it is no longer one
        // (`crate::caravan` §6).
        self.caravan_work(u);
        match self.current_order(u).map(|o| o.body) {
            None => self.do_idle(u, frame),
            Some(Body::Move(m)) => {
                // §8.3: a `GroupMoveOrder` is stepped by `do_group_move`,
                // which runs `do_move` for the **leader** alone and steers
                // every follower off the leader's own position.
                if m.group.is_some() {
                    self.do_group_move(u, frame);
                } else {
                    self.do_move(u, frame);
                }
                if m.kind == MoveKind::ExploreTo {
                    self.do_explore_to_tail(u, frame, m.dest);
                }
            }
            Some(Body::Trade(_)) => self.do_trade(u),
            Some(Body::Build(_)) => self.do_build(u, frame),
            Some(Body::Repair(_)) => self.do_repair(u, frame),
            Some(Body::Garrison { .. }) => self.do_garrison_order(u),
            Some(Body::Gather(_)) => self.do_gather(u, frame),
            Some(Body::Attack(_)) => self.do_attack(u, frame),
            Some(Body::Cast(c)) => self.do_cast(u, c),
            Some(Body::Think) => self.do_think_order(u, frame),
        }
    }

    /// `Unit::do_explore_to@005f24a0`'s tail — everything the case does
    /// beyond `do_move`.
    ///
    /// **One frame in fifteen, phased by `o`**, a unit still walking the
    /// same `EXPLORE_TO` looks around for a goody box and re-targets onto
    /// it (`docs/GOODY.md` §7). The phase is `(o + frame) % 15`, and the
    /// original's `pUVar2 == param_1` — the order list's head is still the
    /// order this call was dispatched for — is what `dest` stands in for:
    /// an arrival that popped the order, or a collision that replaced it,
    /// skips the look.
    ///
    /// `is_captain` (`o_up < 0`) is the second guard: a figure marching
    /// under someone else's formation does not go off on its own.
    fn do_explore_to_tail(&mut self, u: usize, frame: i64, dest: Pos) {
        if (frame + i64::from(self.units[u].index)).rem_euclid(15) != 0 {
            return;
        }
        let same = matches!(
            self.current_order(u).map(|o| o.body),
            Some(Body::Move(m)) if m.kind == MoveKind::ExploreTo && m.dest == dest
        );
        if !same || !self.is_captain(u) {
            return;
        }
        self.find_goody_box(u);
    }

    /// `Unit::repath`: pop the leading transit legs so the target order
    /// re-issues them.
    fn repath(&mut self, u: usize) {
        while self.current_order(u).is_some_and(Order::is_transit) {
            self.kill_current_order(u);
        }
    }

    // ------------------------------------------------------------------
    // Idle — `do_idle`, `check_idle`, `think`
    // ------------------------------------------------------------------

    /// `Unit::do_idle`: `set_anim(CHAR_DEFAULT, 0, 1)`, `collide = 0`,
    /// `check_idle`, `think`. An animal's is `Animal::do_idle`, which
    /// replaces the whole of it (`anim.rs`).
    pub(crate) fn do_idle(&mut self, u: usize, frame: i64) {
        if self.units[u].is_gaia() {
            // A gaia bird is never idle in the original: it carries an air
            // order from birth, and `do_job` dispatches on that order
            // rather than reaching `do_idle` at all. Both orders end in
            // `Unit::do_air_physics`, which ends in `set_anim(CHAR_WALK,
            // 0, 1)` on every frame the bird is not a flock's
            // (`field_0xae`, only ever set for `FLOCKBIRD`). SEAM: this
            // crate has no air order, so the bird stands here instead and
            // the *flight* is unmodelled — but what reaches the stream is
            // here (`docs/SYNC.md` §3.9, `docs/TRANSPORT.md` §5.2).
            let t = self.units[u].type_index;
            if crate::anim::is_air_gaia(t) {
                // `Animal::think_bird`'s `0x192` arm, under
                // `Unit::do_air_patrol+0x28`. A **gull** is on a
                // `StrafeOrder`, so `Unit::do_strafe` calls the same
                // `+0x180` virtual — and `think_bird`'s `0x194` arm
                // returns after an `order_type` call and draws nothing.
                if t == crate::anim::BIRD_TYPE {
                    self.think_bird(u, frame);
                }
                self.set_anim(u, crate::anim::WALK, false, true);
                // **The flight**, `Unit::do_air_patrol`'s second half —
                // [`crate::air`]'s bank, step and edge coin
                // (`docs/SYNC.md` §3.9). run61 proxies the original's own
                // `do_air_physics` and puts 45,712 of its air frames
                // beside this module's: ten birds, every position and
                // every bank zero-crossing, birth to frame 5,400, exact.
                // The thirty-three-frame error this used to carry was
                // never the flight — it was the birth, and
                // `Gaia::spawn_bird` now takes `Unit::init`'s tile snap.
                if t == crate::anim::BIRD_TYPE {
                    let goal = self.bird_goal(u);
                    self.do_air_physics(u, goal, frame);
                }
                //
                // `do_air_patrol`'s own tail, after `do_air_physics`
                // returns 1: the caller branches on `vtable+0x30`,
                // `SubObjectData::is_animal`, and the animal arm is
                // `else if (spell_time == 0) spell_time = 1`. The
                // counter is only ever 0 there on a **landing** frame —
                // `think_bird` steps it on every other — so this is one
                // extra step per landing, and it is what run39's frame
                // 1256 turned on (`docs/SYNC.md` §3.9). The tail is
                // `do_air_patrol`'s; `do_strafe` has none, so the gull
                // does not take it.
                if t == crate::anim::BIRD_TYPE && self.units[u].spell_time == 0 {
                    self.units[u].spell_time = 1;
                }
                return;
            }
            self.animal_idle(u);
            return;
        }
        self.mark(crate::anim::SITE_IDLE_UNIT);
        self.set_default_anim(u);
        // `Unit::do_idle`'s own `collide = 0` (`docs/COLLISION.md` §6).
        self.units[u].collide = 0;
        self.check_idle(u, frame);
        self.think(u, frame);
    }

    /// The carrying walk a gatherer plays — `unit_masks & 0x78000000`, in
    /// `Guy::set_anim`'s own order (`005db61f`–`005db665`): `WALK_TO_WOOD`
    /// first, then `WALK_WITH_WOOD`, `WALK_TO_ORE`, `WALK_WITH_ORE`.
    ///
    /// **It is the mask, not the order's `goto_build`.** Reading the order
    /// made every walk of a gather a carrying walk, including the first
    /// walk to the camp — which `do_non_flat_gather` has never run for, so
    /// the original plays it as the plain `CHAR_WALK` and takes the
    /// arrival stand `Guy::move+0x19f` at the end of it. run33's `1/7`
    /// carries `unit_masks 262146` on the frame it arrives, and this
    /// crate had it on `WALK_WITH_WOOD` (`docs/ANIM.md` §4.4).
    pub(crate) fn gather_walk(&self, u: usize) -> Option<i8> {
        let m = self.units[u].carry;
        if m & CARRY_ANY == 0 {
            return None;
        }
        Some(if m & CARRY_TO_WOOD != 0 {
            crate::anim::WALK_TO_WOOD
        } else if m & CARRY_WITH_WOOD != 0 {
            crate::anim::WALK_WITH_WOOD
        } else if m & CARRY_TO_ORE != 0 {
            crate::anim::WALK_TO_ORE
        } else {
            crate::anim::WALK_WITH_ORE
        })
    }

    /// `Unit::check_idle`: 0 → 1 → 2 on consecutive frames, then +1 every
    /// 16 frames phased by `o`, wrapping to 1.
    fn check_idle(&mut self, u: usize, frame: i64) {
        let unit = &mut self.units[u];
        if unit.idle == 0xff {
            unit.idle = 1;
        } else if unit.idle <= 1 || (frame + i64::from(unit.index)) & 15 == 0 {
            unit.idle += 1;
        }
        // `check_idle@006032c0`'s **tail**, which runs whatever the counter
        // did: the first frame a unit is idle it takes the latch, and a
        // fisherman or a merchant marks its owner's economy dirty — it has
        // just joined `Leader::calc_gather` step 6's walk. That is what
        // drops the recompute period from 512 frames to 8 and puts a
        // newly-arrived boat's deposit on the books within the same
        // handful of frames ([`crate::rares`]).
        if !self.units[u].idle_latch {
            self.units[u].idle_latch = true;
            if self.unit_line_is(u, crate::fish::FISHERMEN) || self.is_merchant(u) {
                self.economy_changed(self.units[u].owner);
            }
        }
    }

    /// `Unit::think` (§2.4): the auto-attack on the first idle frame and
    /// every 32 after, phased by `o`; a worker's `think_peasant(0)`.
    fn think(&mut self, u: usize, frame: i64) {
        // `think:82` — a **citizen** (`TypeIndex` 0x32 or 0x33) drops its
        // carrying walk here, and the leader takes `0x80000`. In the
        // original this sits after the auto-attack arm and before the
        // cadence gate; here the gate stands first, so a citizen that
        // takes an auto-attack keeps the nibble the original would have
        // cleared. SEAM: the two orderings differ only for an armed
        // citizen, and no capture has one.
        if matches!(self.units[u].type_index, 0x32 | 0x33) {
            self.units[u].carry &= !CARRY_ANY;
        }
        let unit = &self.units[u];
        let phase = frame + i64::from(unit.index);
        // The global cadence gate (`Unit::think@005f6e40:87`, found by the
        // second reading — `docs/audit/2026-08-21-orders.md` R1): once a unit
        // has been idle for more than two frames it thinks only one frame in
        // sixteen, phased by `o`. The "could not reach" bit exempts it, so a
        // unit that has just failed to reach something keeps searching every
        // frame.
        if !unit.cant_reach && unit.idle > 2 && phase % 16 != 0 {
            return;
        }
        let me = Obj::Unit(u);
        // Step 3's own cadence — the first idle frame and then one in
        // thirty-two, phased by `o` (`think@005f6e40`, the arm at
        // `5f70fd`). Past it the original enters `Unit::think_attack`,
        // whose **head** joins an army before its target search ever runs
        // (`docs/ARMY.md` §4.2): that is how the AI's first soldier gets
        // into one, and this crate had only the search.
        if self.attack_of(me) != 0 && (unit.idle == 1 || phase & 0x1f == 0) {
            self.think_attack_join_army(u);
            let unit = &self.units[u];
            if unit.combat.stance != combat::Stance::HoldFire
                && unit.combat.target.is_none()
                && let Some(t) = self.find_melee_target(u, -1)
            {
                self.add_attack_order(u, t, QueuePos::New, false, false);
                return;
            }
        }
        // `005f7195`: a worker whose `think_peasant` **found something**
        // ends the think there — the listing's `goto LAB_005f761a`, which
        // is the function's own exit. Ignoring the return value put a
        // citizen that had just been given a gather job through the tail
        // below on the same frame.
        if self.worker_of(u) != Worker::None && self.think_peasant(u, false) {
            return;
        }
        // Step 5's first arm, and it sits **above** the tail's gate: a
        // caravan takes `think_caravan`, whose own idle threshold is its
        // only cadence (`crate::caravan` §3).
        if self.units[u].caravan.is_some() && self.think_caravan(u) {
            return;
        }
        // **The tail's own cadence gate** (§2.4 step 5), the second of the
        // two in this function and the one the code was missing: after the
        // human block's `unit_masks & 0x40000` exit, `think@005f6e40`
        // returns when
        //
        //     idle != 1 && ((o + frame) & 31) != 0
        //
        // — so everything from `think_fish` down, the tail included, runs
        // on a unit's **first** idle frame and then once in thirty-two,
        // phased by `o`. The mod-16 gate above only lets the function be
        // entered; this one is what decides the tail.
        //
        // run56's scout is the diff: it arrives on 2664 with `idle == 1`,
        // thinks, and then — with `o == 0` — thinks again on 2688, 2720,
        // 2752, … exactly. Without this gate the simulation ran a whole
        // second `think_scout` on 2665, where `idle == 2` still passes the
        // mod-16 gate, and that was East Indies' word.
        if self.units[u].idle != 1 && phase & 31 != 0 {
            return;
        }
        // `is(0x13d, 0)` — the `FISHERMEN` lineage — takes `think_fish`,
        // and a `1` back ends the think (`docs/ORDERS.md` §6.8). It sits
        // above the scout/army tail and below everything else, which is
        // why a fishing boat never joins an army.
        if self.unit_line_is(u, crate::fish::FISHERMEN) && self.think_fish(u, frame) {
            return;
        }
        // `is_merchant` — the three exact ids — takes `think_merchant`
        // (`crate::merchant`), and it has a **cadence of its own** on top
        // of the tail's: `think@005f7515` runs it on the first idle frame
        // and then one frame in a hundred and twenty-eight, phased by
        // `o`, where everything either side of it runs one in thirty-two.
        if self.is_merchant(u)
            && (self.units[u].idle == 1 || phase & 127 == 0)
            && self.think_merchant(u)
        {
            return;
        }
        // The tail (`docs/SCOUT.md` §2): a scout or a spy not in an army
        // takes `think_scout`; a supply wagon or a hero takes
        // `add_to_army` (`docs/ARMY.md` §4). The two are exclusive in the
        // original and are kept so — and nothing else joins anything.
        if self.scout_thinks(u) {
            self.think_scout(u);
            return;
        }
        self.think_join_army(u);
    }

    /// `Unit::think_peasant(forced)` (§5.9): the idle gate, the colonist
    /// arm, then the job search — here `find_gather_spot`;
    /// `find_build_spot`/`find_repair_spot` need the object searches and are
    /// not yet modelled.
    ///
    /// **The gate's threshold is 1 for an AI-driven worker**, whatever the
    /// owner's idle-citizen option says: `think_peasant@005f5760:16` reads
    /// the option's switch only when `unit_masks & 0x40000` is clear, and
    /// takes 1 when it is set. So a computer player's new citizen finds a
    /// job on the *first* frame it is idle, and a human's on the second.
    /// That one frame is what run33's trained citizen spends: it comes out
    /// at 99, is first visited at 100, and its walk — and the collision
    /// that ends it at 122 — hangs off that frame (`docs/SYNC.md` §3.16).
    pub(crate) fn think_peasant(&mut self, u: usize, forced: bool) -> bool {
        let unit = &self.units[u];
        let ai = self.ai_driven(unit.owner);
        if !forced {
            let t = if ai {
                1
            } else {
                i32::from(unit.idle_threshold)
            };
            let idle = i32::from(unit.idle);
            if idle < t {
                return false;
            }
            if idle != t && (idle - 2).rem_euclid(5) != 0 {
                return false;
            }
        }
        // **The colonist arm** (`think_peasant@005f5760:53`,
        // `docs/TRANSPORT.md` §7): an AI-driven **citizen** — the base type
        // itself, `0x32` or `0x33`, not the worker category, so a scholar
        // never asks — offers itself for the boat *before* it looks for a
        // job, and a `1` back ends the think. It is the only caller with
        // `colonise = 1`, and so the only writer of the census's
        // `xport_peasants` throttle between sweeps.
        if ai
            && matches!(self.units[u].type_index, 0x32 | 0x33)
            && self.think_civilian_transport(u, true)
        {
            return true;
        }
        let unit = &self.units[u];
        let stance = unit.stance;
        // The same call's other half of the AI split: an AI-driven worker
        // searches without a range limit (§5.9's `find_gather_spot(AI ? −1
        // : UNIT_GATHER_RESPOND_RANGE × 192)`), as a scholar does.
        let range = if ai {
            -1
        } else {
            self.tuning.unit_gather_respond_range * TILE
        };
        // §5.9's **build arm**, ahead of the gather search: `not a scholar
        // and (unit_masks & 0x400 or worker_stance ∈ {1, 2}) and
        // find_build_spot()`. It waited on `stance` (item 190) and could
        // not land while this crate wrote a flat 1 — with that, every
        // citizen asked, and run69's `1/6` left the original's point on
        // **103**. `Unit::init` and `Build::train` between them make an
        // AI's trained citizen **0**, so the arm now asks who the original
        // asks (`crate::stance`, §5.10).
        if !matches!(self.units[u].type_index, 0x34 | 0x35)
            && (self.units[u].was_builder || stance == 1 || stance == 2)
            && self.find_build_spot(u)
        {
            return true;
        }
        if stance <= 1 && self.find_gather_spot(u, range) {
            return true;
        }

        // **The tail, `LAB_005f5920`** — everything below the job search,
        // and the arm East Indies' word at 4313 was (`docs/SCOUT.md`
        // §11.1). A **scholar** (`0x34`/`0x35`) skips the whole of it,
        // `unit_masks & 0x400` included, so it is the one worker that
        // keeps "has been a builder" across a failed search.
        if matches!(self.units[u].type_index, 0x34 | 0x35) {
            return false;
        }
        // SEAM: a **human's** arm here is `(was_builder or stance ∈
        // {1, 2}) and find_repair_spot()`, and `find_repair_spot` is not
        // modelled (`docs/ORDERS.md` §5.9).
        self.units[u].was_builder = false;
        if !ai {
            return false;
        }

        // The AI's: a worker standing in a region where its leader has no
        // city, and which none of the leader's ten sites claims, gives up
        // on the region and **explores** — `think_scout(0)` from here, the
        // second of that function's three call sites and the only one a
        // citizen ever reaches (`docs/SCOUT.md` §11.1). A region a site
        // does claim is worth waiting in, but only for six idle frames.
        //
        // The region is `get_tregion@006b52e0`, the coastal-refined one
        // (`World::tregion_alt`), as the listing calls it.
        let who = self.units[u].owner as usize;
        let t = self.units[u].pos.tile();
        let Some(region) = self.world.tregion_alt(t) else {
            return false;
        };
        if crate::ai::Census::reg(&self.ai[who].census.reg_cities, region) == 0 {
            let claimed = self.ai[who]
                .sites
                .iter()
                .any(|s| s.val != 0 && s.reg == i32::from(region));
            if !claimed || self.units[u].idle > 6 {
                return self.think_scout(u);
            }
        }
        // SEAM: `find_repair_spot` again, the AI's own.
        false
    }

    /// `Unit::do_think_order`: consume the THINK; if nothing follows, a
    /// worker runs the forced search.
    fn do_think_order(&mut self, u: usize, _frame: i64) {
        self.kill_current_order(u);
        if self.order_type(u) != index::NONE {
            return;
        }
        if self.worker_of(u) != Worker::None {
            self.think_peasant(u, true);
        }
    }

    // ------------------------------------------------------------------
    // The move order — `Unit::do_move`
    // ------------------------------------------------------------------

    /// `Unit::do_move` (§4.4) on open ground: the planning half, the
    /// waypoint take with both arrival tests, the straight-line check, the
    /// step.
    fn do_move(&mut self, u: usize, frame: i64) -> Did {
        let Some(Order {
            body: Body::Move(mut mo),
            flags,
        }) = self.current_order(u).copied()
        else {
            return Did::Nothing;
        };
        let mut flags = flags;

        // **The suspended-search block is read, built and not wired**
        // (`docs/DECISIONS.md` entry 30, item 301). §4.4 step 2 is
        // `do_move`'s first block after the cavalry-archer fire: while
        // [`crate::Unit::search`] is `Some`, **no step happens** — every
        // arm returns — the "has the blocker gone" probe fires on the 5th,
        // 7th, 9th … frame after `collide_frame`, `repaths` ticks on every
        // fourth `o + frame`, `collide` counts up every frame and
        // [`Sim::find_upath_restore`] resumes the search.
        //
        // **Its Great Lakes cost is paid** (item 304,
        // `docs/PATHFINDER.md` §18.5). Wired, the block used to cost
        // Great Lakes' long word 7679 → 6862, and the cause was upstream
        // of every line of it: run76's `1/28` degrades out of formation on
        // 6860 on the original's own frame, but this crate's
        // [`Sim::kill_current_path`] popped the path segment without the
        // `clear_partial_path` under it, so the follower kept a suspended
        // search that no order was left to serve and stood in this block
        // for the rest of the capture. With that one call in place the
        // wired measurement is East Indies 7812 → **8193** and Great Lakes
        // **7679**, unmoved.
        //
        // What remains is the wiring itself, one commit on the branch
        // `worktree-loop-301-suspend`. `docs/PATHFINDER.md` §18.

        // `timer`: a self-destruct.
        if mo.timer > 0 {
            if mo.timer == 1 {
                self.kill_current_order(u);
                self.work(u, frame);
                return Did::Nothing;
            }
            mo.timer -= 1;
            self.store_move(u, mo, flags);
        }

        // The action under the move: an ATTACK whose target is in range has
        // no further use for the chase; one whose target is gone re-paths.
        if let Some(a) = self.action_of(u)
            && let Body::Attack(_) = self.units[u].orders[a].body
        {
            let me = Obj::Unit(u);
            match self.units[u].combat.target {
                Some(t) if self.valid_target(me, t) => {
                    if self.is_in_range(me, t) {
                        self.kill_current_order(u);
                        return Did::Something;
                    }
                }
                _ => {
                    self.repath(u);
                    return Did::Nothing;
                }
            }
        }

        // Planning: the first time, or with an empty stack. Every fresh
        // move calls `find_wpath` exactly once; a refusal re-pushes the
        // goal and falls back to the tile grid (§4.4).
        if flags & flag::PATHED == 0 || self.units[u].path.is_empty() {
            let goal = PathData {
                to: mo.dest,
                tolerance: 0,
                flags: path_flag::FINAL,
            };
            self.units[u].path.push(goal);
            let r = self.find_wpath(u);
            if r < 1 {
                if self.units[u].path.is_empty() {
                    self.units[u].path.push(goal);
                }
                let r2 = self.find_tpath(u);
                if r2 < 1 && self.units[u].path.is_empty() {
                    self.units[u].path.push(goal);
                }
            }
            flags |= flag::PATHED;
            self.store_move(u, mo, flags);
            if self.units[u].path.len() > 10 {
                return Did::Something;
            }
        }

        // Taking a waypoint.
        if !mo.has_waypoint {
            let Some(top) = self.units[u].path.last().copied() else {
                // Nothing to walk: the goal was already here.
                self.kill_current_order(u);
                return Did::Nothing;
            };
            mo.has_waypoint = true;
            mo.waypoint = top.to;
            self.units[u].line_ok = false;
            self.units[u].tolerance = top.tolerance;
            self.store_move(u, mo, flags);

            // **The waypoint's own collision test** (§4.4), the one call of
            // `detect_unit_collision` that is not `move_step`'s or
            // `resolve_unit_collision`'s. It runs once per leg — only on
            // the frame the waypoint is taken — and asks whether somebody
            // is already standing where this leg ends.
            //
            // A **final** waypoint under a `GATHER`, `ATTACK` or
            // `BUILD_AT` action is then not worth walking at all: the walk
            // dies here and the action picks somewhere else next frame.
            // That is how a farmer whose re-picked cell a sibling already
            // works stays where it is (`docs/COLLISION.md` §8).
            //
            // Otherwise a **parked** collider — one whose own current order
            // is not a move, so it is not going to get out of the way —
            // widens the tolerance to three of its `big_radius`
            // (`ObjectType +0x244`): give up short of it rather than walk
            // into it.
            //
            // SEAM: the original also spells `TRADE_ROUTE` in the kill's
            // action set and runs a region check just above (a
            // turn-in-place before a leg that ends in another terrain
            // region); neither is modelled — `docs/ORDERS.md` §4.4.
            if let Some(other) = self.detect_unit_collision(u, mo.waypoint) {
                let action = self.action_of(u).map(|a| self.units[u].orders[a].index());
                if top.flags & path_flag::FINAL != 0
                    && matches!(
                        action,
                        Some(index::GATHER | index::ATTACK | index::BUILD_AT)
                    )
                {
                    self.kill_current_order(u);
                    return Did::Nothing;
                }
                if !self.current_order(other).is_some_and(Order::is_move) {
                    let t = self.profile(Obj::Unit(other)).big_radius * 3;
                    if self.units[u].tolerance < t {
                        self.units[u].tolerance = t;
                        self.units[u].path.pop();
                        self.units[u].path.push(PathData {
                            tolerance: t,
                            ..top
                        });
                    }
                }
            }
            // `detect_unit_collision` writes `coll_x`/`coll_y` into the
            // order in place; take the copy back so the stores below keep
            // them.
            if let Some(m) = self.current_move(u) {
                mo = m;
            }

            let here = self.units[u].pos;
            // Already there: the Euclidean test before the step.
            if vector_dist(mo.waypoint.x - here.x, mo.waypoint.y - here.y)
                <= self.units[u].tolerance
            {
                mo.has_waypoint = false;
                let popped = self.units[u].path.pop();
                self.store_move(u, mo, flags);
                if popped.is_none_or(|p| p.flags & path_flag::FINAL == 0) {
                    return Did::Something;
                }
                // `false`: `do_move`'s own arrival (`005f8844`) tests
                // `orderlist.length == 1` and nothing else — the gather
                // clause belongs to `move_step`'s snap arm alone.
                self.arrive(u, mo, false);
                return Did::Nothing;
            }
            self.store_move(u, mo, flags);
        }

        // The speed, and the straight-line check. `do_move` takes it from
        // the `+0x17c` virtual, which an animal overrides
        // ([`Sim::get_speed`]).
        let speed = self.get_speed(u);
        // `if (masks & 8) goto STEP` (§4.4): the jump the straight-line
        // check takes when it succeeds lands **past** the pause check, so a
        // unit that verifies its line this frame steps this frame even with
        // a collision pause running — and the pause does not tick. Only the
        // re-plan's own `TAKE` comes back through `STEP_IF_MOVING`, and so
        // does a unit that already had the bit at entry.
        //
        // run10's `1/2` is the case, and it is worth a line: the collision
        // at frame 572 sets `pause = 3` and clears the bit; 573 re-verifies
        // and **steps with `pause` still 3**; 574–576 are the three still
        // frames. Ticking on 573 put this simulation a frame ahead for the
        // rest of the walk (2026-08-31, item 69).
        let mut straight_to_step = false;
        if !self.units[u].line_ok {
            self.units[u].path_recursion = 0;
            let goal = mo.waypoint;
            let r = self.find_path(u, &mut mo, goal);
            if r == 0 {
                let top = self.units[u].path.last().copied();
                match top {
                    Some(t) if t.to != self.units[u].pos => {
                        self.units[u].line_ok = true;
                        mo.has_waypoint = true;
                        mo.waypoint = t.to;
                        self.units[u].tolerance = t.tolerance;
                        self.store_move(u, mo, flags);
                    }
                    _ => self.units[u].line_ok = false,
                }
            }
            straight_to_step = self.units[u].line_ok;
            if !self.units[u].line_ok {
                // The straight line is not enough: the pathfinder's job
                // (§4.4). An unreachable goal with more orders queued kills
                // this one and the next.
                let here = self.units[u].pos;
                if self.invalid_loc(u, mo.dest.tile(), true, false, false, false, true) != 0
                    && self.units[u].orders.len() > 1
                {
                    self.kill_current_order(u);
                    self.kill_current_order(u);
                    return Did::Something;
                }
                // The draw that chooses the grid — one `Random::get` off
                // the sync stream on the first `do_move` of any move
                // `find_path` refuses.
                self.mark(SITE_MOVE_GRID);
                let n = self.rng.roll();
                let thr = match n % 5 {
                    2 => 2 * CELL,
                    0 => 8 * CELL,
                    _ => 5 * CELL,
                };
                let far = (mo.waypoint.x - here.x).abs() + (mo.waypoint.y - here.y).abs() > thr;
                // `wflag` (which grid planned) only matters to the
                // resolve-block branch below, which is a seam; kept for the
                // shape.
                //
                // The length a positive return is compared against is read
                // **per arm**, and in the near arm *after* every pop: the
                // original captures `field_0xc0` beside each planner call,
                // and re-reads it once the unwind loop has run. Reading it
                // before the pops made "unchanged" nearly unreachable.
                let (r, len_before, _wflag) = if far {
                    let len_before = self.units[u].path.len();
                    (self.find_wpath(u), len_before, true)
                } else {
                    // A non-final top equal to `last` is stale: pop it.
                    if let Some(t) = self.units[u].path.last().copied()
                        && t.flags & path_flag::FINAL == 0
                        && Some(t.to) == mo.last
                    {
                        self.units[u].path.pop();
                    }
                    // **`collide`, and the two grids it chooses between**
                    // (`docs/ORDERS.md` §4.4, `Unit::do_move@005f7b30`'s
                    // `field_0x88` test). A unit that has *not* been
                    // colliding drops its loose near waypoints and plans on
                    // tiles; one that has keeps them and plans on the
                    // 48-grid, which is the finer one and the only one that
                    // can get around the unit in the way.
                    //
                    // Taking the tile arm unconditionally is what put
                    // run53's `1/7` in a two-frame livelock at 5502: the
                    // sidestep `move_step` had just pushed was popped the
                    // next frame, the unit walked back into the same
                    // collider, and the pair repeated for the rest of the
                    // capture (item 204).
                    if self.units[u].collide != 0 {
                        let len_before = self.units[u].path.len();
                        (self.find_upath(u, false), len_before, false)
                    } else {
                        while let Some(t) = self.units[u].path.last().copied() {
                            if t.flags & (path_flag::FINAL | 0x20) == 0 && t.tolerance < 0x60 {
                                self.units[u].path.pop();
                            } else {
                                break;
                            }
                        }
                        let len_before = self.units[u].path.len();
                        (self.find_tpath(u), len_before, false)
                    }
                };
                // A positive return with the stack length unchanged counts
                // as a refusal.
                let r = if r > 0 && self.units[u].path.len() == len_before {
                    0
                } else {
                    r
                };
                if r == 0 && !self.units[u].path.is_empty() {
                    let top = self.units[u].path.last().copied().expect("non-empty");
                    if top.flags & path_flag::FINAL == 0 {
                        self.units[u].path.pop();
                        mo.has_waypoint = false;
                        self.store_move(u, mo, flags);
                        return Did::Something;
                    }
                    self.units[u].path.pop();
                    flags &= !flag::PATHED;
                    self.store_move(u, mo, flags);
                    self.kill_current_order(u);
                    if self
                        .current_order(u)
                        .is_some_and(|o| matches!(o.body, Body::Attack(_) | Body::Build(_)))
                    {
                        self.kill_current_order(u);
                    }
                    return Did::Something;
                } else if r == -1 {
                    self.kill_current_order(u);
                    if self
                        .current_order(u)
                        .is_some_and(|o| matches!(o.body, Body::Attack(_) | Body::Build(_)))
                    {
                        self.kill_current_order(u);
                    }
                    return Did::Something;
                }
                // TAKE: the new top is the target, and the straight line to
                // it is verified the same frame.
                let Some(top) = self.units[u].path.last().copied() else {
                    self.kill_current_order(u);
                    return Did::Something;
                };
                mo.last = None;
                mo.has_waypoint = true;
                mo.waypoint = top.to;
                self.units[u].tolerance = top.tolerance;
                self.units[u].path_recursion = 0;
                let goal = mo.waypoint;
                let r2 = self.find_path(u, &mut mo, goal);
                if r2 == 0 {
                    self.units[u].line_ok = true;
                }
                if !self.units[u].line_ok {
                    // The line to the new top failed too: with a world-grid
                    // plan, wait for the next frame; the tile-grid case
                    // falls into collision resolution (SEAM: `do_move`'s own
                    // arm, `docs/COLLISION.md` §9 — the step's is modelled).
                    self.store_move(u, mo, flags);
                    return Did::Something;
                }
                self.store_move(u, mo, flags);
            }
        }

        // Stepping — `STEP_IF_MOVING`, which `goto STEP` jumps past.
        if !straight_to_step && mo.pause != 0 {
            mo.pause -= 1;
            self.store_move(u, mo, flags);
            return Did::Something;
        }
        self.unit_step(u, mo, speed)
    }

    /// `Unit::do_group_move@005e79a0` (`docs/ORDERS.md` §8.3) — the
    /// per-frame half of a `GroupMoveOrder`, and the reason a marching
    /// formation costs the original **one** `do_move` and not one per
    /// member.
    ///
    /// The leader steps a plain move and then rotates the group's slot
    /// table by its own heading; every follower is steered off the
    /// leader's *current* position plus its rotated offset, and takes
    /// `move_step` directly. That difference is the whole of it: three
    /// archers walking as a squad spend one `Unit::do_move+0xe84` between
    /// them, where N independent moves spend three.
    fn do_group_move(&mut self, u: usize, frame: i64) {
        let Some(Order {
            body: Body::Move(mo),
            flags,
        }) = self.current_order(u).copied()
        else {
            return;
        };
        let Some(gm) = mo.group else { return };
        // `if (this->group == -1) ungroup` (`5e79f0`). This crate's group
        // membership **is** the army's (`docs/GROUPS.md` §1): a unit with
        // no army is in no group, and its group order degrades on the
        // spot.
        let Some(g) = self.group_of(u) else {
            self.ungroup_move_order(u, gm.id);
            return;
        };
        if gm.leader == u {
            self.group_move_leader(u, frame, &g, gm);
        } else {
            self.group_move_follower(u, frame, &g, gm, mo, flags);
        }
    }

    /// `do_group_move`'s leader arm (`5e7a10`–`5e7c6c`).
    fn group_move_leader(&mut self, u: usize, frame: i64, g: &crate::group::Group, gm: GroupMove) {
        // Every 32nd frame, phased by `o`, an attacking leader whose
        // target has come within `0x900` drops the whole group move — the
        // formation stops marching and starts fighting.
        if let Some(a) = self.action_of(u)
            && matches!(self.units[u].orders[a].body, Body::Attack(_))
            && (frame + i64::from(self.units[u].index)).rem_euclid(32) == 0
            && let Some(t) = self.units[u].combat.target
        {
            let p = self.pos_of(t);
            let here = self.units[u].pos;
            if vector_dist(p.x - here.x, p.y - here.y) < 0x900 {
                self.kill_group_move(g, gm.id);
                return;
            }
        }
        if self.do_move(u, frame) == Did::Something {
            // `update_order(this) != param_1` — the move I just stepped is
            // still the one at the head.
            if !self.still_group_move(u, gm.id) {
                return;
            }
            // SEAM: the group's `speed`/`new_speed` pair and the `march`
            // flag `has_general` sets. `UnitData::get_speed`'s group cap is
            // already a stated seam here ([`Sim::get_speed`]), so the pair
            // has no reader and is not carried.
            self.group_update_positions(g, u);
            return;
        }
        // `do_move` gave up. The attack hand-off is a seam
        // (`Group::distribute_attack`, `Group::action_attack` — no capture
        // reaches either from here); what is left is the ungroup, which is
        // what a move that simply ended does.
        if self.still_group_move(u, gm.id) {
            self.ungroup_move_order(u, gm.id);
        }
    }

    /// `do_group_move`'s follower arm (`5e7c8c`–`5e8660`).
    fn group_move_follower(
        &mut self,
        u: usize,
        frame: i64,
        g: &crate::group::Group,
        mut gm: GroupMove,
        mut mo: MoveOrder,
        flags: u8,
    ) {
        let _ = frame;
        let l = gm.leader;
        // 1. The leader has to still be usable: alive, on the map, in my
        //    group, and holding a group order with **my** `id` — or a
        //    `CHANGE_FORM`, which this crate does not model.
        let lost = !(self.units[l].alive()
            && self.units[l].on_map
            && self.army_of(l) == g.army
            && self.still_group_move(l, gm.id));
        // 2. `form_id` is rewritten from the list on every frame that
        //    reaches here (`5e7ea0`). A member the list no longer holds
        //    loses its group outright.
        let Some(i) = g.list.iter().position(|&m| m == u) else {
            self.ungroup_move_order(u, gm.id);
            return;
        };
        if gm.form_id != i {
            gm.form_id = i;
            mo.group = Some(gm);
            self.store_move(u, mo, flags);
        }
        if lost {
            // `LAB_005e7ee2`: more than `0x5ff` from **the order's own
            // destination** — my slot, not the leader's cell — and I take
            // the formation over; otherwise the order degrades.
            let here = self.units[u].pos;
            if vector_dist(mo.dest.x - here.x, mo.dest.y - here.y) > 0x5ff {
                self.group_refresh_order(g, u);
                self.group_rewrite_leader(g, u, gm.id);
                return;
            }
            self.ungroup_move_order(u, gm.id);
            return;
        }
        // 3. An attacking follower whose target is already in range stops
        //    marching (`5e8611`).
        if let Some(a) = self.action_of(u)
            && matches!(self.units[u].orders[a].body, Body::Attack(_))
            && let Some(t) = self.units[u].combat.target
            && self.is_in_range(Obj::Unit(u), t)
        {
            self.kill_current_order(u);
            return;
        }
        // 4. My slot this frame: the leader's **current** position plus my
        //    rotated offset.
        let Some(slot) = self.group_slot_point(g, l, i) else {
            self.ungroup_move_order(u, gm.id);
            return;
        };
        // 5. Unwind my path to its goal (`5e8034`): pop entries until one
        //    carries `FINAL`, keep it, and push it back. An empty stack is
        //    the original's uninitialised read; the order's own
        //    destination stands in for it.
        let mut goal = PathData {
            to: mo.dest,
            tolerance: 0,
            flags: path_flag::FINAL,
        };
        while let Some(top) = self.units[u].path.pop() {
            goal = top;
            if top.flags & path_flag::FINAL != 0 {
                break;
            }
            if self.units[u].path.is_empty() {
                goal.flags |= path_flag::FINAL;
                break;
            }
        }
        self.units[u].path.push(goal);

        let here = self.units[u].pos;
        let d_slot = vector_dist(slot.x - here.x, slot.y - here.y);
        let d_goal = vector_dist(goal.to.x - here.x, goal.to.y - here.y);
        // 6. **The formation is over** when my slot buys me no more than
        //    `0x60`, or the goal is within `0x180`: the order becomes a
        //    plain move at the goal (`5e80fc`, `5e8105`).
        if d_goal - d_slot <= 0x60 || d_goal <= 0x180 {
            gm.in_group = false;
            mo.group = Some(gm);
            mo.has_waypoint = true;
            mo.waypoint = goal.to;
            self.store_move(u, mo, flags);
            self.ungroup_move_order(u, gm.id);
            return;
        }
        // 7. Where to walk. Straight to my slot when the leader is my own
        //    captain, or when the bearing to the slot is within 120° of
        //    the bearing to the goal (`5e8167`'s `0x55555555`); otherwise
        //    off the leader's next waypoint, or the midpoint.
        let to_slot = find_angle(slot.x - here.x, slot.y - here.y);
        let to_goal = find_angle(goal.to.x - here.x, goal.to.y - here.y);
        let straight = self.squad_captain(u) == l || within_third(to_slot, to_goal);
        let mut disagrees = false;
        if straight {
            self.units[u].path.push(PathData {
                to: slot,
                tolerance: 0,
                flags: 0,
            });
            mo.waypoint = slot;
            gm.in_group = true;
        } else {
            // `5e8173`: a slot this close is not worth chasing at all.
            if d_slot < 0x30 {
                return;
            }
            let lead = self.units[l].path.last().copied();
            match lead {
                Some(w) if gm.in_group => {
                    let off = self.armies[g.who as usize].list[g.army.unwrap_or(0)]
                        .group
                        .curr[i];
                    let to = Pos::new(w.to.x + off.x, w.to.y + off.y);
                    self.units[u].path.push(PathData {
                        to,
                        tolerance: 0,
                        flags: 0,
                    });
                    mo.waypoint = to;
                    disagrees = !within_third(to_slot, self.units[u].movement.heading);
                    if vector_dist(to.x - here.x, to.y - here.y) < 0x60 {
                        gm.in_group = false;
                    }
                }
                _ => {
                    mo.waypoint = Pos::new((slot.x + goal.to.x) / 2, (slot.y + goal.to.y) / 2);
                    disagrees = true;
                }
            }
        }
        mo.has_waypoint = true;
        mo.group = Some(gm);
        self.store_move(u, mo, flags);
        if mo.waypoint == here {
            return;
        }
        self.units[u].tolerance = 0;
        // 8. The speed. A follower walking with the formation is allowed a
        //    third more, capped at nine; one walking across it takes half.
        let v = if disagrees {
            self.get_speed(u) / 2
        } else {
            let v = self.get_speed(u);
            v + (v / 3).min(9)
        };
        // 9. **A slot the world refuses is not walked into at all**
        //    (`5e838a`): the formation gives up and the order degrades,
        //    which is what keeps a follower from marching into its
        //    neighbour rather than standing blocked beside it.
        //
        //    SEAM: the flock of birds an invalid slot within `0x300`
        //    Manhattan of an **ocean** cell adds — one sync-stream draw —
        //    and the `cavarch_fight` call below it.
        if self.invalid_loc(u, mo.waypoint.tile(), false, false, false, false, false) != 0 {
            self.ungroup_move_order(u, gm.id);
            return;
        }
        // 10. **The step's own answer is the formation's end condition**
        //     (`5e856d`–`5e8660`). `move_step` returns 0 from exactly
        //     three places — blocked and still owing a turn, blocked and
        //     handed to `resolve_unit_collision`, and a tile the world
        //     refused ([`Sim::unit_step`]) — and every one of them drops
        //     the follower out of formation: the head order is re-read,
        //     and unless the attack hand-off takes it the group move
        //     degrades into N independent moves. It is the same tail the
        //     leader takes off `do_move`'s 0 in
        //     [`Sim::group_move_leader`], and `docs/ORDERS.md` §8.3 read
        //     it as "arrived", which is the one thing a 0 never means:
        //     arrival returns 1.
        //
        //     SEAM: the attack-context hand-off above it —
        //     `Group::action_attack` for a `GROUP_ATTACK_TO` whose
        //     `collide_o` is a valid target, `kill_current_order`
        //     otherwise — is the leader arm's own seam and is not
        //     modelled; no capture reaches it, because every collider in
        //     the captures on disk belongs to the colliding unit's own
        //     player.
        if self.unit_step(u, mo, v) == Did::Nothing && self.still_group_move(u, gm.id) {
            self.ungroup_move_order(u, gm.id);
        }
    }

    /// `ObjectData::get_captain` — the head of `u`'s own squad, up the
    /// `o_up` chain. `Unit::ungroup_move_order` starts there and walks
    /// back down, so ungrouping any figure ungroups the whole squad.
    fn squad_captain(&self, u: usize) -> usize {
        let mut at = u;
        while !self.units[at].captain {
            match self.units[at].o_up {
                Some(c) if c != at && self.units[c].alive() => at = c,
                _ => break,
            }
        }
        at
    }

    /// Is `u`'s head order still a group move carrying `id`?
    fn still_group_move(&self, u: usize, id: i64) -> bool {
        self.current_order(u).is_some_and(|o| match o.body {
            Body::Move(m) => m.group.is_some_and(|x| x.id == id),
            _ => false,
        })
    }

    /// `Unit::ungroup_move_order@005fd140` — replace the group order
    /// carrying `id` with the plain move it stands on, up the captain
    /// chain and then down every subordinate.
    ///
    /// `GROUP_ATTACK_TO → ATTACK_TO` and `GROUP_MOVE → MOVE_TO`; here the
    /// kind is already the plain one and dropping [`MoveOrder::group`] is
    /// the whole conversion. `dest = 0` clears the waypoint, and a member
    /// that is **not** the leader also loses [`flag::PATHED`] and its
    /// path — so the group's shared plan dies with the formation and each
    /// unit re-plans for itself.
    pub(crate) fn ungroup_move_order(&mut self, u: usize, id: i64) {
        let mut at = self.squad_captain(u);
        loop {
            self.ungroup_one(at, id);
            match self.units[at].o_down {
                Some(d) if self.units[d].alive() => at = d,
                _ => return,
            }
        }
    }

    fn ungroup_one(&mut self, u: usize, id: i64) {
        let Some(i) = self.units[u].orders.iter().position(|o| match o.body {
            Body::Move(m) => m.group.is_some_and(|x| x.id == id),
            _ => false,
        }) else {
            return;
        };
        let leader = match self.units[u].orders[i].body {
            Body::Move(m) => m.group.is_some_and(|x| x.leader == u),
            _ => false,
        };
        let order = &mut self.units[u].orders[i];
        if !leader {
            order.flags &= !flag::PATHED;
        }
        if let Some(m) = order.move_mut() {
            m.group = None;
            m.has_waypoint = false;
        }
        if !leader {
            self.kill_current_path(u);
        }
    }

    /// `Unit::kill_group_move@005e3400` over the group
    /// (`Group::kill_group_move@007123f0`): every member's orders lose the
    /// group plan, and every one carrying `id` that is **not** an
    /// attack-move is killed. A `GROUP_ATTACK_TO` survives both arms.
    pub(crate) fn kill_group_move(&mut self, g: &crate::group::Group, id: i64) {
        for i in 0..g.list.len() {
            let u = g.list[i];
            if !(self.units[u].alive() && self.units[u].on_map) {
                continue;
            }
            let mut kill = false;
            for j in 0..self.units[u].orders.len() {
                let Body::Move(m) = self.units[u].orders[j].body else {
                    continue;
                };
                if m.kind == MoveKind::AttackTo {
                    continue;
                }
                self.units[u].orders[j].flags &= !flag::PATHED;
                self.kill_current_path(u);
                if m.group.is_some_and(|x| x.id == id) {
                    kill = true;
                }
            }
            if kill {
                self.kill_current_order(u);
            }
        }
    }

    /// Writes a move order's fields back to the front of the list.
    fn store_move(&mut self, u: usize, mo: MoveOrder, flags: u8) {
        // The order being stepped is the front one — **unless**
        // `set_new_location` has just pushed a transport cast in front of
        // it (`docs/TRANSPORT.md` §6). The original writes through a
        // pointer to the move and so does not care; here the write has to
        // find it again, one place back.
        let i = usize::from(matches!(
            self.units[u].orders.front().map(|o| o.body),
            Some(Body::Cast(_))
        ));
        if let Some(order) = self.units[u].orders.get_mut(i) {
            order.flags = flags;
            if let Some(m) = order.move_mut() {
                *m = mo;
            }
        }
    }

    /// `Unit::find_path` (§4.6): 0 = walk straight, 1 = plan. The written
    /// `2` ("abort") is dead code in the original — nothing produces it at
    /// the base case — so it is not modelled.
    ///
    /// The march, per the settled third reading (`docs/PATHFINDER.md` §9):
    /// the angle is recomputed from the current remainder every iteration,
    /// each component is clamped to its axis' remainder, and the exits are
    /// `manh <= speed` (the loop condition), a grown remainder, and a
    /// remainder within one *actual clamped step* on both axes (`<=`). A
    /// no-progress step is unreachable with `speed >= 3`, so the interim
    /// guard of 2026-08-22 is retired; the soak that found the hang stands
    /// guard over this rewrite.
    fn find_path(&mut self, u: usize, mo: &mut MoveOrder, goal: Pos) -> u8 {
        let here = self.units[u].pos;
        let mut goal = goal;
        if goal == here {
            return 0;
        }
        let cells = (goal.cell().x - here.cell().x).abs() + (goal.cell().y - here.cell().y).abs();
        if cells > STRAIGHT_LINE_CELLS
            && self.invalid_loc(u, goal.tile(), true, true, false, false, false) == 0
        {
            return 1;
        }
        self.units[u].path_recursion = self.units[u].path_recursion.saturating_add(1);
        // `find_path`'s own `(*+0x17c)(x, y, 0)` and its `< 4 → 3` floor,
        // the same virtual the step takes ([`Sim::get_speed`]).
        let speed = self.get_speed(u).max(3);
        // THE PULL-BACK (§4.6, `005fbaa6`-`005fbb56`), and it is what item 28
        // was: a goal whose own tile refuses is walked *back toward us* one
        // step at a time until it does not, and the waypoint and the path's
        // top follow it wherever they are the goal. So a woodcutter sent at
        // a forest tile is really sent at the last open point short of it,
        // and the march that follows never enters the tile that would have
        // asked `go_around_building` for a detour. The sim had no pull-back,
        // marched into the forest, and paid `do_move`'s grid draw for a
        // detour the original never needed.
        //
        // The step is `sinx/cosx` of the bearing to the goal, taken once and
        // then only ever clamped down — never re-aimed, unlike the march's.
        let ang = find_angle(goal.x - here.x, goal.y - here.y);
        let mut back_x = movement::sin_component(ang, speed);
        let mut back_y = movement::cos_component(ang, speed);
        while self.invalid_loc(u, goal.tile(), false, false, false, false, false) != 0 {
            let (dx, dy) = (goal.x - here.x, goal.y - here.y);
            if dx.abs() < back_x.abs() {
                back_x = dx;
            }
            if dy.abs() < back_y.abs() {
                back_y = -dy;
            }
            if mo.waypoint == goal {
                mo.waypoint = Pos::new(mo.waypoint.x - back_x, mo.waypoint.y + back_y);
            }
            if let Some(top) = self.units[u].path.last_mut()
                && top.to == goal
            {
                top.to = Pos::new(top.to.x - back_x, top.to.y + back_y);
            }
            goal = Pos::new(goal.x - back_x, goal.y + back_y);
            // `sinx` and `cosx` of a bearing are never both zero at speed
            // >= 3, so this only fires once both have been clamped to a
            // remainder of zero — the goal has arrived at us.
            if back_x == 0 && back_y == 0 {
                break;
            }
        }
        // The goal walked all the way back onto us: there is nowhere to go,
        // and the cell is marked so the pathfinder does not offer it again.
        if goal == here {
            self.units[u].avoid = Some(goal);
            return 1;
        }
        let mut at = here;
        let (mut dx, mut dy) = (goal.x - at.x, goal.y - at.y);
        let mut manh = dx.abs() + dy.abs();
        let mut prev = manh;
        while speed < manh {
            if prev < manh {
                return 0;
            }
            prev = manh;
            let ang = find_angle(dx, dy);
            let mut sx = movement::sin_component(ang, speed);
            let mut cy = movement::cos_component(ang, speed);
            if dx.abs() < sx.abs() {
                sx = dx;
            }
            if dy.abs() < cy.abs() {
                cy = -dy;
            }
            let next = Pos::new(at.x + sx, at.y - cy);
            if next.tile() != at.tile()
                && self.invalid_loc(u, next.tile(), false, false, false, false, false) != 0
            {
                // Blocked. `go_around_building` walks the blocking tile's
                // edge and pushes a detour; a recursive `find_path` on its
                // first point is what decides whether the detour counts.
                // Ten deep, the original stops asking.
                if self.units[u].path_recursion < 10 {
                    let n0 = self.units[u].path.len();
                    self.go_around_building(u, mo, at, ang, speed);
                    if self.units[u].path_recursion != 10 && self.detour_verified(u, mo, goal, n0) {
                        return 0;
                    }
                }
                return 1;
            }
            at = next;
            dx = goal.x - at.x;
            dy = goal.y - at.y;
            if dx.abs() <= sx.abs() && dy.abs() <= cy.abs() {
                return 0;
            }
            manh = dx.abs() + dy.abs();
        }
        0
    }

    /// `UnitData::invalid_loc` with the five flags `find_path` and
    /// `go_around_building` both pass clear — the plain "may this unit
    /// stand on this tile" question.
    fn tile_refuses(&self, u: usize, t: Pos) -> bool {
        self.invalid_loc(u, t, false, false, false, false, false) != 0
    }

    /// `Unit::go_around_building@005fc350` (§4.6) — the answer to a
    /// straight line that clips a *building*, which the pull-back cannot
    /// help with: the goal's own tile is clear, so there is nothing to pull
    /// back, and the obstacle sits somewhere in the middle of the march.
    ///
    /// It is a **tile-edge walk**, and its shape is the same in both axes.
    /// The step that failed crossed one tile edge, so the unit has a *lane*
    /// (the row or column it is still standing in) and a *blocked lane*
    /// (the one it tried to enter). The walk runs perpendicular to the
    /// crossing, one tile at a time in both directions from the blocked
    /// point, and stops when either the blocked lane opens (success, at
    /// that offset) or the unit's own lane closes (success only if the
    /// blocked lane happens to be open there). Off the map is a failure for
    /// that direction; both directions failing gives up.
    ///
    /// Of the two candidates the closer to `mo.waypoint` wins, with
    /// forward as the tie-break, and either is discarded if it resolves to
    /// the unit's own tile centre. A candidate more than `0x300` from the
    /// blocked point in either axis gives up — the detour would be longer
    /// than the pathfinder's own leg.
    ///
    /// Then one to three `PathData`s go on the stack, bottom first:
    /// **the turn-in point** in the blocked lane and a **`DETOUR`
    /// midpoint** between the lanes, both only when the unit's own lane is
    /// closed at the found offset; and always **the target**, in the unit's
    /// own lane, one tile back along the walk when the turn-in pair was
    /// pushed. Every one of them is placed `off % 0xc0 / 2` off its tile's
    /// low corner rather than at the centre, from the order's own
    /// `off_x/off_y` — so a formation's units do not all aim at the same
    /// point.
    ///
    /// Giving up is `mo.dest = 0; masks &= ~8; path_recursion = 10`, and
    /// the recursion counter is how the caller tells the two apart.
    fn go_around_building(
        &mut self,
        u: usize,
        mo: &mut MoveOrder,
        last: Pos,
        ang: Angle,
        spd: i32,
    ) {
        let lt = last.tile();
        // The blocked point, recomputed here from an *unclamped* step: the
        // march clamps each component to its remainder, this does not, so
        // the two need not agree. That is the original's, not a slip.
        let sx = movement::sin_component(ang, spd);
        let sy = -movement::cos_component(ang, spd);
        let blocked = Pos::new(last.x + sx, last.y + sy);
        let bt = blocked.tile();

        // Which lane is which. `cmp` is the column the diagonal case keeps;
        // `settled` is the original's `goto LAB_005fc4c0`.
        let mut cmp = bt.x;
        let mut settled = lt.x == bt.x;
        if !settled && lt.y != bt.y {
            // A diagonal entry. More than two tiles at once is not a step
            // this can reason about.
            if (lt.y - bt.y).abs() + (lt.x - bt.x).abs() > 2 {
                return self.detour_gave_up(u, mo);
            }
            if self.tile_refuses(u, Pos::new(lt.x, bt.y)) {
                if self.tile_refuses(u, Pos::new(bt.x, lt.y)) {
                    // Both corners of the diagonal refuse: shorten the step
                    // and ask again, which may land orthogonally instead.
                    cmp = bt.x;
                    if spd > 1 {
                        return self.go_around_building(u, mo, last, ang, spd - 1);
                    }
                } else {
                    cmp = lt.x;
                }
                settled = true;
            }
        }
        let vertical = !settled || lt.y == bt.y || (lt.x != cmp && sx.abs() < sy.abs());
        let step = if vertical {
            Pos::new(0, if sy > 0 { 1 } else { -1 })
        } else {
            Pos::new(if sx > 0 { 1 } else { -1 }, 0)
        };
        let back = Pos::new(-step.x, -step.y);
        let (fwd_ok, fwd) = self.edge_walk(u, blocked, step, lt);
        let (bwd_ok, bwd) = self.edge_walk(u, blocked, back, lt);

        let here = self.units[u].pos;
        let centre = |t: i32| t * 0xc0 + 0x60;
        let at_unit = |p: Pos| {
            let t = p.tile();
            here.x == centre(t.x) && here.y == centre(t.y)
        };
        let (pick, pstep) = if !fwd_ok {
            if !bwd_ok {
                return self.detour_gave_up(u, mo);
            }
            (bwd, back)
        } else if !bwd_ok {
            (fwd, step)
        } else if at_unit(fwd) {
            (bwd, back)
        } else if at_unit(bwd) {
            (fwd, step)
        } else {
            let reach = |p: Pos| (p.x - mo.waypoint.x).abs() + (p.y - mo.waypoint.y).abs();
            if reach(bwd) < reach(fwd) {
                (bwd, back)
            } else {
                (fwd, step)
            }
        };

        if (pick.x - last.x).abs() >= 0x301 || (pick.y - last.y).abs() >= 0x301 {
            return self.detour_gave_up(u, mo);
        }

        // The waypoints. `off` is the order's own destination offset inside
        // its world cell (§4.1); the halved tile part of it is what places
        // every detour point off its tile's centre.
        let off = Pos::new(mo.dest.x.rem_euclid(0x300), mo.dest.y.rem_euclid(0x300));
        let skew = |v: i32| (v % 0xc0) / 2;
        let ct = pick.tile();
        let mut pushes: Vec<PathData> = Vec::new();
        let target = if pstep.x == 0 {
            let mut ty = ct.y;
            if self.tile_refuses(u, Pos::new(lt.x, ty)) {
                let (cx, cy) = (centre(ct.x), centre(ty));
                pushes.push(PathData {
                    to: Pos::new(skew(off.x) - 0x30 + cx, skew(off.y) - 0x30 + cy),
                    tolerance: 0,
                    flags: 0,
                });
                ty -= pstep.y;
                let (lane_cx, back_cy) = (centre(lt.x), centre(ty));
                pushes.push(PathData {
                    to: Self::detour_mid(cx, cy, lane_cx, back_cy),
                    tolerance: 0,
                    flags: path_flag::DETOUR,
                });
            }
            Pos::new(lt.x, ty)
        } else {
            let mut tx = ct.x;
            if self.tile_refuses(u, Pos::new(tx, lt.y)) {
                let (cx, cy) = (centre(tx), centre(ct.y));
                pushes.push(PathData {
                    to: Pos::new(skew(off.x) - 0x30 + cx, skew(off.y) - 0x30 + cy),
                    tolerance: 0,
                    flags: 0,
                });
                tx -= pstep.x;
                let (back_cx, lane_cy) = (centre(tx), centre(lt.y));
                pushes.push(PathData {
                    to: Self::detour_mid(cx, cy, back_cx, lane_cy),
                    tolerance: 0,
                    flags: path_flag::DETOUR,
                });
            }
            Pos::new(tx, lt.y)
        };
        pushes.push(PathData {
            to: Pos::new(
                skew(off.x) + target.x * 0xc0 + 0x30,
                skew(off.y) + target.y * 0xc0 + 0x30,
            ),
            tolerance: 0,
            flags: 0,
        });
        for p in pushes {
            self.units[u].path.push(p);
        }
    }

    /// The `DETOUR` midpoint between the two lanes: the mean of the two
    /// tile centres on each axis, biased one unit *down* whenever the
    /// turn-in centre is the larger, so the point falls inside the tile the
    /// unit is leaving rather than on the edge between them.
    fn detour_mid(cx: i32, cy: i32, other_x: i32, other_y: i32) -> Pos {
        let mut m = Pos::new((other_x + cx) / 2, (other_y + cy) / 2);
        if cx != other_x && cx - other_x >= 0 {
            m.x -= 1;
        }
        if cy != other_y && cy - other_y >= 0 {
            m.y -= 1;
        }
        m
    }

    /// One direction of `go_around_building`'s edge walk. Returns whether
    /// the blocked lane was found open, and the point the walk stopped at.
    fn edge_walk(&self, u: usize, from: Pos, step: Pos, lt: Pos) -> (bool, Pos) {
        let w = self.world.width() * crate::world::TILES_PER_CELL * 0xc0;
        let h = self.world.height() * crate::world::TILES_PER_CELL * 0xc0;
        let mut at = from;
        loop {
            let next = Pos::new(at.x + step.x * 0xc0, at.y + step.y * 0xc0);
            if next.x < 0 || next.y < 0 || next.x >= w || next.y >= h {
                return (false, at);
            }
            at = next;
            let cur = at.tile();
            // The unit's own lane keeps the coordinate the walk does not
            // move; the blocked lane is the tile the walk is standing on.
            let lane = if step.x == 0 {
                Pos::new(lt.x, cur.y)
            } else {
                Pos::new(cur.x, lt.y)
            };
            if self.tile_refuses(u, lane) {
                return (!self.tile_refuses(u, cur), at);
            }
            if !self.tile_refuses(u, cur) {
                return (true, at);
            }
        }
    }

    /// `go_around_building`'s refusal: the waypoint is dropped, the
    /// verified-line bit is cleared, and `path_recursion` is pinned at ten
    /// so that neither this call nor any caller asks again.
    fn detour_gave_up(&mut self, u: usize, mo: &mut MoveOrder) {
        mo.has_waypoint = false;
        self.units[u].line_ok = false;
        self.units[u].path_recursion = 10;
    }

    /// `find_path`'s acceptance of a detour (§4.6), and the half of it that
    /// is not `go_around_building`'s.
    ///
    /// The pushed waypoints are lifted off one at a time — any that lands
    /// exactly on the unit is dropped and the next one down takes its place
    /// as the candidate — and the candidate is then verified by a
    /// **recursive `find_path`**. Six degenerate cases refuse before the
    /// recursion: the candidate or the entry below it being the unit's own
    /// position or the position the last plan was made at, a unit already
    /// standing where it last planned, and a candidate equal to the goal we
    /// could not reach.
    ///
    /// On a verified line the detour goes back on the stack in its original
    /// order, each entry's "turn in place" bit recomputed against the one
    /// that will sit above it — set when exactly one of the two is on
    /// water, so that a leg crossing the shore turns before it walks.
    fn detour_verified(&mut self, u: usize, mo: &mut MoveOrder, goal: Pos, n0: usize) -> bool {
        let here = self.units[u].pos;
        let Some(top) = self.units[u].path.last().copied() else {
            return false;
        };
        let n1 = self.units[u].path.len();
        // `w` is the candidate to verify; `below` the entry under it.
        let mut w = top.to;
        let mut below = top.to;
        let mut saved: Vec<PathData> = Vec::new();
        let mut i = n0;
        while i < n1 {
            let Some(e) = self.units[u].path.pop() else {
                break;
            };
            below = e.to;
            if w == here && i + 1 < n1 {
                if let Some(p) = self.units[u].path.last().copied() {
                    w = p.to;
                    below = p.to;
                }
            } else {
                saved.push(e);
            }
            i += 1;
        }
        let planned_at = mo.last.unwrap_or(Pos::new(-1, -1));
        if w == here
            || w == planned_at
            || below == here
            || below == planned_at
            || here == planned_at
            || goal == w
        {
            return false;
        }
        // The `2` arm here is the original's dead code (§4.6): nothing
        // produces it at the base case, so a non-zero return is "plan".
        if self.find_path(u, mo, w) != 0 {
            return false;
        }
        if !self.units[u].line_ok {
            while let Some(e) = saved.pop() {
                if let Some(f) = self.units[u].path.pop() {
                    let f = self.shore_flagged(f, e.to);
                    self.units[u].path.push(f);
                }
                self.units[u].path.push(e);
            }
            mo.last = Some(here);
            if let Some(t) = self.units[u].path.pop() {
                let t = self.shore_flagged(t, here);
                self.units[u].path.push(t);
            }
            self.units[u].line_ok = true;
        }
        true
    }

    /// `TURN_FIRST` set on `e` exactly when it and `other` are on opposite
    /// sides of the shore — the `(mask & 0x30) == 0x20` test, water against
    /// everything else.
    fn shore_flagged(&self, mut e: PathData, other: Pos) -> PathData {
        let wet = |p: Pos| {
            self.world.tile_mask(p.tile()) & crate::world::tile::SURFACE
                == crate::world::tile::SURFACE_OCEAN
        };
        if wet(e.to) == wet(other) {
            e.flags &= !path_flag::TURN_FIRST;
        } else {
            e.flags |= path_flag::TURN_FIRST;
        }
        e
    }

    /// The unit step and its arrival (§4.5): `move_step` through
    /// `docs/MOVEMENT.md`, then the Manhattan test against the waypoint's
    /// tolerance, the pop, the facing on the final waypoint, the kill.
    fn unit_step(&mut self, u: usize, mut mo: MoveOrder, speed: i32) -> Did {
        // STEP (§4.4): `avoid_x/y = −1,−1` before every step.
        self.units[u].avoid = None;
        let unit = &self.units[u];
        let m = unit.movement;
        let from = unit.pos;
        // `GuyData::turn_speed@005de340` reads `unit_masks & 0x80000` on the
        // **unit**, not a copy taken when the type was assigned, so the
        // pack bonus follows the boat rather than the record
        // (`docs/ORDERS.md` §6.8).
        let turning = movement::Turning {
            packed: unit.combat.packed,
            ..m.turning
        };
        let rate = movement::turn_speed(
            &self.tuning,
            &turning,
            m.body.last_speed,
            m.body.avg_speed,
            movement::TurnMode::Unit,
        );
        // `local_18 & 4` at `005fb1a5`: the **current waypoint's** own
        // `TURN_FIRST`/`TRANSPORT` bit, which `shore_flagged` above writes
        // and until 2026-09-01 nothing read. A waypoint that crosses the
        // shore line is turned to before it is walked to, whatever the
        // distance.
        let turn_first = self.units[u]
            .path
            .last()
            .is_some_and(|e| e.flags & path_flag::TURN_FIRST != 0);
        // `unit_masks & 0x100000`, the one-shot half step the **soft**
        // collision arm left on this unit last frame
        // (`docs/COLLISION.md` §7). `move_step` spends it and says so;
        // the bit is the unit's, so the clearing is here.
        let half = self.units[u].half_step;
        let step = movement::move_step(
            from,
            m.facing,
            mo.waypoint,
            speed,
            &m.turning,
            rate,
            turn_first,
            half,
        );
        if step.half_step_used {
            self.units[u].half_step = false;
        }
        // `Unit::move_step`'s own `set_angle`, which is the one call of the
        // eighteen this simulation makes — and it is where a marching
        // leader's turn-around flips its group's mirror flag
        // (`docs/GROUPS.md` §4.1, §6.3). It is passed the **heading**, before
        // and regardless of the turn: `move_step` calls it at the top, on the
        // bearing `find_angle` just returned. The facing that the step is
        // actually taken along is guy 0's, and only `Guy::do_turn` moves it.
        self.unit_set_angle(u, step.heading);
        // **The turning stand** (`docs/ANIM.md` §4.8). Both turn-in-place
        // arms hand `Guy::do_turn` a non-zero fifth argument, so a guy
        // with `guy_flags & 8` is asked for a turn animation — and a
        // packing type that has none takes the idle roll instead. The two
        // arms are two `do_turn` call sites and so two `ebp` chains.
        match step.turned_in_place {
            Some(movement::TurnArm::Near) => self.mark(crate::anim::SITE_TURN_NEAR),
            Some(movement::TurnArm::Far) => self.mark(crate::anim::SITE_TURN_FAR),
            None => {}
        }
        if step.turned_in_place.is_some() {
            self.do_turn_anim(u, m.facing, step.facing, step.heading);
        }
        let unit = &mut self.units[u];
        unit.movement.facing = step.facing;

        // **And there the frame ends** (`docs/COLLISION.md` §8.8). Both
        // turn-in-place arms are `call Guy::do_turn(…, 0, 1); mov eax,1;
        // ret 8` on the listing — the far one returning at `005fb2b9`,
        // the near one at `005fb2e6` — so a frame spent turning reaches
        // **none** of what follows: not the four world-bounds tests, not
        // the collision probe, not `invalid_loc`, not `set_new_location`
        // and its reveal, and not the tail's arrival test.
        //
        // The probe is the one that cost a frame. `detect_unit_collision`
        // does its clearing on **every** path out (§4.1), so running it on
        // a turning unit wipes the `collide_o`/`collide_who` that
        // `do_move`'s own waypoint probe wrote three statements earlier
        // and ages `collide` to zero — the original keeps both, because it
        // never asks. run90 block 7809 is the case: `1/7` turns 44° in
        // place with `1/6`'s block over the cell its new leg ends in, and
        // the original carries `collide 1 / collide_o 6 / collide_who 1`
        // into the dump where this crate carried the fields clear.
        //
        // The store is still owed — this crate steps on a *copy* of the
        // order, and `do_move`'s waypoint block writes `coll_x`/`coll_y`
        // into the real one (§4.3's "the store is into the order").
        if step.turned_in_place.is_some() {
            let flags = self.current_order(u).map_or(0, |o| o.flags);
            self.store_move(u, mo, flags);
            return Did::Something;
        }

        // **The collision block** (`docs/COLLISION.md` §5). The proposed
        // point is tested against the occupancy index; a blocked step
        // either snaps through onto a sidestep waypoint, waits out the
        // turn it still owes, gives up and calls the waypoint reached, or
        // goes to `resolve_unit_collision`.
        let top = self.units[u].path.last().copied();
        let mut target = step.pos;
        let hit = self.detect_unit_collision(u, target);
        // `detect_unit_collision` writes `coll_x`/`coll_y` **into the
        // order**, and the original walks `move_step` on a pointer to it,
        // so every later write of the move's own fields keeps the pair.
        // This crate steps on a copy, so take the pair back before any
        // `store_move` below puts the stale one back — the same reason
        // `do_move`'s waypoint test does it, and the arm that showed it is
        // the one that stores and returns while a turn is still owed.
        if let Some(m) = self.current_move(u) {
            mo.coll = m.coll;
        }
        if let Some(other) = hit {
            let (dx, dy) = (mo.waypoint.x - from.x, mo.waypoint.y - from.y);
            let through = top.is_some_and(|t| t.flags & path_flag::SIDESTEP != 0)
                && !self.detect_quick(u, top.expect("tested").to, false)
                && dx.abs() < 0x61
                && dy.abs() < 0x61;
            if through {
                // The final snap through a collision: walk onto the
                // waypoint itself and let the arrival test take it.
                target = mo.waypoint;
            } else {
                // The blocked stand — `move_step:281`, the call at
                // `005fb74e` and so the site `+0x823`
                // ([`crate::anim::SITE_BLOCKED`], `docs/COLLISION.md` §5).
                // It sits **before** all three give-up tests, so a unit
                // still owed a turn has re-rolled its idle by the time
                // `move_step` returns.
                self.mark(crate::anim::SITE_BLOCKED);
                self.set_default_anim(u);
                // **The two arms `move_step` answers 0 from**
                // (`005fb689` and `005fb6df`, the only `return 0`s the
                // function has besides the tile refusal below). The value
                // is not decoration: `do_group_move` reads it on **both**
                // sides of the formation — the leader's through
                // `do_move`, the follower's straight off `move_step` —
                // and a zero ungroups the whole squad
                // (`docs/ORDERS.md` §8.3). Answering `Did::Something`
                // here kept run76's Archer squad in formation for the
                // rest of its march where the original degrades it into
                // three independent moves on the frame the leading
                // Archer is squeezed onto its cell centre (item 236).
                if step.owed != 0 {
                    let flags = self.current_order(u).map_or(0, |o| o.flags);
                    self.store_move(u, mo, flags);
                    return Did::Nothing;
                }
                let manh = dx.abs() + dy.abs();
                let reach = self.profile(Obj::Unit(u)).big_radius
                    + self.profile(Obj::Unit(other)).big_radius;
                let give_up = reach * 3 <= manh
                    || self.units[u].collide < 0x1a
                    || (top.is_none_or(|t| t.tolerance == 0 || t.flags & path_flag::SIDESTEP != 0)
                        && top.is_none_or(|t| t.flags & path_flag::FINAL == 0));
                if give_up {
                    self.resolve_unit_collision(u);
                    let flags = self.current_order(u).map_or(0, |o| o.flags);
                    if self.current_order(u).is_some_and(Order::is_move) {
                        let mo = self.current_move(u).expect("a move order");
                        self.store_move(u, mo, flags);
                    }
                    return Did::Nothing;
                }
                // Give up on reaching it exactly: the waypoint is close
                // enough now.
                self.units[u].tolerance = manh * 2;
            }
        }

        let mut arrived = false;
        let mut snapped_in = false;
        if self.world.accepts(target) {
            // **The step that changes tile is asked permission**
            // (`move_step@005faf30`, `005fb7c1`–`005fb7fd`). The original
            // compares the tile of the proposed point against the tile it
            // is standing on, and where they differ calls
            // `UnitData::invalid_loc` with all five flags clear — the same
            // call `find_path` and `go_around_building` make. A refusal
            // drops the step whole: no `set_anim`, no `set_new_location`,
            // no reveal, and the order keeps its waypoint, so the unit
            // stands where it is, turns another frame's worth, and tries
            // the step again next frame from a bearing it has turned
            // further round. It is the last of the four things
            // `docs/MOVEMENT.md`'s `move_step` section listed as not
            // modelled.
            //
            // **And it clears the verified line on the way out**
            // (`005fb7c1`-`005fb7fd`: the `& 0xfffffff7` and the
            // `return 0` are the same two lines), which is what makes the
            // next frame re-verify rather than step straight again. This
            // crate's one reader is `do_move`'s `if !line_ok` above, so
            // leaving the bit standing cost run65 a `Unit::find_path` the
            // original enters on 6207 and this crate did not.
            if target.tile() != from.tile()
                && self.invalid_loc(u, target.tile(), false, false, false, false, false) != 0
            {
                self.units[u].line_ok = false;
                return Did::Nothing;
            }
            // **`move_step`'s own `set_anim`, and it is a clock and not
            // a picture** (`docs/ANIM.md` §4.9). Both arms of an accepted
            // step — the partial one at `move_step:304` and the Manhattan
            // snap at `:355` — call `Unit::set_anim(CHAR_WALK, 0, 1)` on
            // **every guy** immediately before `set_new_location`, so a
            // walking unit asks for its walk *twice* a frame: here, and
            // again in `Guy::move`'s own half ([`Sim::guys_follow`]).
            //
            // For guy 0 the pair is idempotent — a walk still inside its
            // length keeps its clock — and for a **crew figure whose
            // packet is empty** it is the whole mechanic.
            // `Guy::set_anim`'s walk arm splits on whether the resolved
            // slot is the one already playing: a *change* rescales, and
            // the rescale passes the old slot to both `get_anim_time`
            // calls, so it is `cur_time · t / t` and keeps the clock; only
            // the *same* slot takes an overrun length off it. A crew
            // figure comes off the mirror on `CHAR_SLOG` carrying guy 0's
            // clock, so this call is the change — `SLOG → WALK`, clock
            // kept — and `Guy::move`'s is then the same slot and
            // subtracts. With only the second call the figure kept the
            // mirrored clock, wrapped a frame early and paid an idle roll
            // the original does not: run73's window, Great Lakes 5571.
            //
            // **Past the turn-in-place arms**, which `return 1` at
            // `005fb2e1` and `005fb2b4`, so a frame spent turning never
            // reaches the call. This crate walks on to the collision block
            // and the store on such a frame — it steps on a *copy* of the
            // order, so the store is owed — but the animation is not.
            //
            // SEAM: the snap arm asks `CHAR_DEFAULT` instead when the
            // waypoint offsets were both zero at the top of `move_step`
            // and `UnitData+0xd8 < 2`. `do_move`'s own "already there"
            // test takes that case a step earlier here, so the arm is
            // unreachable; it would be a draw if it were not.
            if step.turned_in_place.is_none() {
                self.set_anim(u, crate::anim::WALK, false, true);
            }
            let flags = self.current_order(u).map_or(0, |o| o.flags);
            if !self.set_new_location(u, target, false) {
                // The step crossed the waterline and was converted rather
                // than taken (`docs/TRANSPORT.md` §6): `move_step` returns
                // 1 on the spot, so the waypoint, the arrival test and the
                // reveal all wait for the boat. The flags are the move's
                // own, read before the cast went in front of it.
                self.store_move(u, mo, flags);
                return Did::Something;
            }
            // `Unit::set_new_location`'s half-cell test and the reveal
            // behind it (`docs/VISION.md` §6). `move_step` is the caller
            // that passes `param_3 = 0`, so this is the **ring** pass.
            self.moved_to(u, from, true);
            if step.arrived && target == step.pos {
                arrived = true;
                snapped_in = step.snapped;
            } else {
                let (dx, dy) = (mo.waypoint.x - target.x, mo.waypoint.y - target.y);
                if dx.abs() + dy.abs() <= self.units[u].tolerance {
                    arrived = true;
                }
            }
        } else {
            // A step outside the world. The original's four bounds tests
            // each `return 1` on their own and none of them touches
            // `unit_masks`, so the line stays verified and the unit tries
            // the same step again next frame; answering `Did::Nothing`
            // here ungrouped a marching formation whose leader reached
            // the edge (`group_move_leader`, 2026-09-05's R7).
            self.units[u].movement.dest = None;
            return Did::Something;
        }
        self.units[u].movement.dest = Some(mo.waypoint);
        if !arrived {
            let flags = self.current_order(u).map_or(0, |o| o.flags);
            self.store_move(u, mo, flags);
            return Did::Something;
        }
        mo.has_waypoint = false;
        let popped = self.units[u].path.pop();
        let flags = self.current_order(u).map_or(0, |o| o.flags);
        self.store_move(u, mo, flags);
        self.units[u].movement.dest = None;
        if popped.is_none_or(|p| p.flags & path_flag::FINAL == 0) {
            // Next frame takes the next waypoint and re-checks the line.
            // The clear belongs to *that* frame: the original writes
            // `MoveOrder::dest = 0` here and nothing else, and it is
            // `do_move`'s `dest == 0` arm (`005f7b30:428`) that clears
            // the bit when it lifts the next entry off the stack —
            // which is `mo.has_waypoint = false` and the clear above.
            return Did::Something;
        }
        self.arrive(u, mo, snapped_in);
        Did::Something
    }

    /// The final waypoint: `set_angle(mo->angle)` when the move is the only
    /// order — **or, on the Manhattan-snap arm only, when the action beneath
    /// is a gather**; clear the pathed bit; kill.
    ///
    /// The two arms of `move_step@005faf30` end with two different tests, and
    /// the difference is one clause: the snap's is `orderlist.length == 1 ||
    /// get_action()->get_type() == GATHER` (`005fb562`), the partial step's
    /// is `orderlist.length == 1` alone (`005fb4a8`). A farmer that lands on
    /// its cell by a partial step — its last leg still *longer* in Manhattan
    /// than one step, so the snap does not fire, while the sine and cosine
    /// components reach it exactly — keeps the bearing of that step and never
    /// faces the order's angle. run10's AI farmers do
    /// exactly that on frame 116 and the human's do not on 110
    /// (`docs/ORDERS.md` §4.5, `docs/SYNC.md` §3.12).
    fn arrive(&mut self, u: usize, mo: MoveOrder, snapped_in: bool) {
        let only = self.units[u].orders.len() == 1;
        let gather_beneath = snapped_in
            && self
                .action_of(u)
                .is_some_and(|a| matches!(self.units[u].orders[a].body, Body::Gather(_)));
        if only || gather_beneath {
            // `set_angle(mo->angle, …, 0)` — the heading, and guy 0's
            // `des_angle` with it. The facing does not snap: the body turns
            // toward the order's angle while it stands there.
            self.unit_set_angle(u, mo.angle);
        }
        if let Some(front) = self.units[u].orders.front_mut() {
            front.flags &= !flag::PATHED;
        }
        self.kill_current_order(u);
    }

    // ------------------------------------------------------------------
    // Build, repair, garrison — §5
    // ------------------------------------------------------------------

    /// `Object::adjacent_to`: `attack_dist < 0x60`, the footprint's extent
    /// for the building and `block_radius + 0x18` for the unit.
    pub fn adjacent_to(&self, u: usize, b: usize) -> bool {
        let bd = &self.buildings[b];
        let t_ext = match bd.ty {
            Some(ty) => (
                self.build_types[ty].x_size * HALF_TILE,
                self.build_types[ty].y_size * HALF_TILE,
            ),
            None => combat::extent(&bd.combat.unwrap_or_default(), true),
        };
        let a_ext = combat::extent(&self.profile(Obj::Unit(u)), false);
        combat::attack_dist(self.units[u].pos, bd.pos, a_ext, t_ext, false) < ADJACENT
    }

    /// `WallData::covers_tile`: the tile is inside the footprint.
    pub fn covers_tile(&self, b: usize, tile: Pos) -> bool {
        let bd = &self.buildings[b];
        let Some(ty) = bd.ty else {
            return bd.pos.tile() == tile;
        };
        let corner = self.tile_corner(ty, bd.pos);
        let (xs, ys) = (self.build_types[ty].x_size, self.build_types[ty].y_size);
        tile.x >= corner.x && tile.x < corner.x + xs && tile.y >= corner.y && tile.y < corner.y + ys
    }

    /// `UnitType::find_nearby_spot(x, y, min, max, step, angle, …)`
    /// (`docs/ORDERS.md` §10) under the **pairwise** filter — the
    /// `FILTER_NOT_ME` every build, repair, gather, garrison and stable
    /// call site passes.
    ///
    /// Rings of radius `min, min + step, …, max` around the centre, each
    /// swept over 31 bearings in the order `0, ±1, ±2, … ±7` sixteenths of
    /// a turn from `angle`, then the odd thirty-seconds (`17/32` twice,
    /// `1/32` never, the direct opposite never); each projected point
    /// snapped to its quarter-tile centre; the first candidate on the map,
    /// not on a blocked tile, not on `footprint_of`'s footprint (a farm
    /// excepted for a citizen), of the unit's terrain class and **free of
    /// other units** wins. `max <= 0` and `step <= 0` take the defaults.
    /// No RNG.
    #[allow(clippy::too_many_arguments)]
    pub fn find_nearby_spot(
        &self,
        u: usize,
        centre: Pos,
        min: i32,
        max: i32,
        step: i32,
        angle: Angle,
        footprint_of: Option<usize>,
    ) -> Option<Pos> {
        self.find_nearby_spot_coll(
            u,
            centre,
            min,
            max,
            step,
            angle,
            footprint_of,
            Coll::Pairwise,
        )
    }

    /// The same sweep with the collision half named. `Coll::None` is the
    /// original's `nocoll != 0` — and, as a stated seam, its whole
    /// general path (`docs/ORDERS.md` §10, §14).
    #[allow(clippy::too_many_arguments)]
    pub fn find_nearby_spot_coll(
        &self,
        u: usize,
        centre: Pos,
        min: i32,
        max: i32,
        step: i32,
        angle: Angle,
        footprint_of: Option<usize>,
        coll: Coll,
    ) -> Option<Pos> {
        self.spot_sweep(
            Seeker::Unit(u),
            centre,
            min,
            max,
            step,
            angle,
            footprint_of,
            coll,
            false,
        )
    }

    /// The same sweep with the original's **`uber_unit`** argument set —
    /// `Unit::go_to@005f7a50`'s call, and the only one in this crate
    /// (`docs/ARMY.md` §4.3).
    ///
    /// A squad placement changes two things at once (`docs/ORDERS.md`
    /// §10): the collision block grows by half the formation's span, and
    /// the pairwise pair is not asked at all — the general radius query
    /// and its ordered twin are.
    pub fn find_nearby_spot_squad(
        &self,
        u: usize,
        centre: Pos,
        min: i32,
        max: i32,
        step: i32,
        angle: Angle,
    ) -> Option<Pos> {
        self.spot_sweep(
            Seeker::Unit(u),
            centre,
            min,
            max,
            step,
            angle,
            None,
            Coll::Pairwise,
            true,
        )
    }

    /// The same sweep asked by a **type** rather than by a unit — the
    /// original's `not_o`/`not_who` of `(-1, -1)`, which is what
    /// `Unit::do_cast` and `SpellType::cast_transport` pass when they look
    /// for the water a barge is born on (`docs/TRANSPORT.md` §6).
    ///
    /// Nothing is exempt from the collision half, and the block, the domain
    /// and the radius defaults are the type's.
    pub fn find_nearby_spot_type(
        &self,
        ty: usize,
        centre: Pos,
        min: i32,
        max: i32,
        step: i32,
        angle: Angle,
    ) -> Option<Pos> {
        self.spot_sweep(
            Seeker::Type(ty),
            centre,
            min,
            max,
            step,
            angle,
            None,
            Coll::Pairwise,
            false,
        )
    }

    #[allow(clippy::too_many_arguments)] // as `find_nearby_spot_coll`
    fn spot_sweep(
        &self,
        who: Seeker,
        centre: Pos,
        min: i32,
        max: i32,
        step: i32,
        angle: Angle,
        footprint_of: Option<usize>,
        coll: Coll,
        uber: bool,
    ) -> Option<Pos> {
        let p = match who {
            Seeker::Unit(u) => self.profile(Obj::Unit(u)),
            Seeker::Type(t) => self.unit_types[t].combat,
        };
        let mut max = max;
        if (min > 0 && max == 0) || max < 0 {
            // `find_nearby_spot@0061de70:45`: the default is
            // `min + 4 × big_radius`, overridden to `min + 0x240` only when
            // `big_radius == 0` **and** `!(unit_flags & 0x10)` — the
            // transport-capable flag, which this crate does not model, so the
            // second condition is assumed false. It can only differ for a
            // transport-capable type whose `big_radius` is 0, and no shipped
            // row is both. Stated in §14. The squad branch
            // (`min + 0xc0 + 4 × (((uber−1) × guy_spacing)/2 + big_radius)`)
            // is not modelled either — nothing here places a squad.
            // (`docs/audit/2026-08-21-orders.md` R7 N3.)
            max = if p.big_radius == 0 {
                min + 0x240
            } else {
                min + 4 * p.big_radius
            };
        }
        if max < min {
            max = min;
        }
        let step = if step <= 0 {
            ((max - min) / 8).max(1)
        } else {
            step
        };
        // **The squad's own collision radius** (`61df42`–`61df58`): with
        // `uber_unit != 0` the block the general query is asked with is
        // `block_radius + ((uber_size − 1) × guy_spacing) / 2 + 0x30` —
        // half the formation's own span plus a quarter-tile, so a squad
        // asks for the room it will actually stand in.
        let uber_r = p.block_radius + ((p.uber_size - 1) * p.guy_spacing) / 2 + 0x30;
        let farm_ok = footprint_of.is_some_and(|b| self.building_ident(b) == Ident::Farm)
            && matches!(who, Seeker::Unit(u) if self.worker_of(u) == Worker::Citizen);
        let air = matches!(p.domain, crate::attrition::Domain::Air);
        let mut r = min;
        loop {
            let ks: &[i32] = if r == 0 { &[0] } else { &BEARINGS };
            for &k in ks {
                let cand = if r == 0 {
                    centre
                } else {
                    let half = if !(-7..=7).contains(&k) {
                        0x800_0000
                    } else {
                        0
                    };
                    let a = Angle(
                        angle
                            .0
                            .wrapping_add(k.wrapping_mul(0x1000_0000))
                            .wrapping_add(half),
                    );
                    Pos::new(
                        centre.x + movement::sin_component(a, r),
                        centre.y - movement::cos_component(a, r),
                    )
                };
                let c = Pos::new(
                    cand.x.div_euclid(SNAP) * SNAP + SNAP_CENTRE,
                    cand.y.div_euclid(SNAP) * SNAP + SNAP_CENTRE,
                );
                if !self.world.tile_in_bounds(c.tile()) {
                    continue;
                }
                if !air && self.world.tile_mask(c.tile()) & crate::world::tile::BLOCKED != 0 {
                    continue;
                }
                if let Some(b) = footprint_of
                    && !farm_ok
                    && self.covers_tile(b, c.tile())
                {
                    continue;
                }
                // **The terrain class, by domain** (`0061e3a1`..`0061e39e`):
                // an air type takes anything; a **sea** type needs the tile's
                // surface field to be `SURFACE_OCEAN` exactly; a **land**
                // type needs it not to be. This crate asked
                // `World::accepts` here — a cell-bounds test the loop's own
                // `tile_in_bounds` has already made — so a citizen was free
                // to stand in the sea, which is what put the AI's dock
                // builder on the wrong point of the swarm ring
                // (`docs/ORDERS.md` §10, "The ocean the ring could stand
                // in").
                //
                // SEAM: the warship clause. A sea type with `+0x1e8` or
                // `is(0x15f, 1)` also needs `!(mask & 0x2400)`; neither
                // input is loaded here, so it is taken as false — a warship
                // may stand on a bad-path ocean tile that the original
                // would refuse.
                if !air {
                    let ocean =
                        self.world.tile_mask(c.tile()) & tile::SURFACE == tile::SURFACE_OCEAN;
                    if ocean != matches!(p.domain, crate::attrition::Domain::Sea) {
                        continue;
                    }
                }
                // The collision half, last of all: `Objects::find_collision`
                // then `Objects::find_ordered_collision`, both against
                // `(u)` as "me" (`docs/COLLISION.md` §5.2). A spot another
                // unit is standing on — or has already been sent to — is
                // taken.
                let hit = coll == Coll::Pairwise
                    && match (uber, who) {
                        // **A squad placement never takes the pairwise
                        // pair.** `bVar17` — the flag that selects
                        // `find_collision` plus `find_ordered_collision` —
                        // is written only in the arm `uber_unit != 0`
                        // jumps over (`61df3e`, the write at
                        // `61df5f`–`61df7c`), so a squad goes down the general
                        // path whatever its filter says: one radius query
                        // against every player's positions, and, because
                        // `not_who` is a real player here, the *ordered*
                        // one beside it.
                        (true, _) => {
                            let exempt = match who {
                                Seeker::Unit(u) => Some(u),
                                Seeker::Type(_) => None,
                            };
                            self.find_unit_with_radius(uber_r, c, exempt)
                                || match who {
                                    Seeker::Unit(u) => self.find_unit_ordered_with_radius(
                                        uber_r,
                                        c,
                                        self.units[u].owner,
                                        exempt,
                                    ),
                                    Seeker::Type(_) => false,
                                }
                        }
                        (false, Seeker::Unit(u)) => {
                            self.find_collision(u, c) || self.find_ordered_collision(u, c)
                        }
                        (false, Seeker::Type(_)) => {
                            self.find_unit_with_radius(p.block_radius, c, None)
                        }
                    };
                if hit {
                    continue;
                }
                return Some(c);
            }
            if max <= r {
                return None;
            }
            r = (r + step).min(max);
        }
    }

    /// The swarm ring (§5.4): `min(x_size, y_size) × 0x60 + 0x30`, halved
    /// for a farm under a build, nudged `0x30` outward.
    ///
    /// Returns the spot **and the move order's angle**, which is
    /// `find_angle(site − spot)` — the bearing from the spot back to the
    /// site, so a builder arrives facing what it is about to build. It is
    /// taken from the ring's own answer, *before* the `BUILD_AT` nudge:
    /// the listing at `7103f3`–`710406` reads the two out-parameters of
    /// `find_nearby_spot` straight into the subtraction and only then, at
    /// `710415`, runs `leal 0x30(%esi)` on the same register. The
    /// decompiler prints both of the function's `find_angle` calls with
    /// the same two locals because the pair travels in `ecx`/`edx`, and
    /// they are not the same pair: the first is `find_angle(unit − site)`
    /// (`71021a`), the ring's sweep bearing, and this is the second.
    fn swarm_spot(&self, u: usize, b: usize, building: bool) -> Option<(Pos, Angle)> {
        let bd = &self.buildings[b];
        let (xs, ys) = bd.ty.map_or((1, 1), |t| {
            (self.build_types[t].x_size, self.build_types[t].y_size)
        });
        let mut r = xs.min(ys) * HALF_TILE + SNAP;
        if building && self.building_ident(b) == Ident::Farm {
            r /= 2;
        }
        let here = self.units[u].pos;
        let angle = find_angle(here.x - bd.pos.x, here.y - bd.pos.y);
        let spot = self.find_nearby_spot(u, bd.pos, r, 0, -1, angle, Some(b))?;
        let facing = find_angle(bd.pos.x - spot.x, bd.pos.y - spot.y);
        if !building {
            return Some((spot, facing));
        }
        // The `+0x30` nudge away from the site on each axis, re-validated
        // as a one-candidate ring.
        let nudge = |v: i32, c: i32| {
            if v > c {
                v + SNAP
            } else if v < c {
                v - SNAP
            } else {
                v
            }
        };
        let nudged = Pos::new(nudge(spot.x, bd.pos.x), nudge(spot.y, bd.pos.y));
        Some((
            self.find_nearby_spot(u, nudged, 0, 0, 0, angle, Some(b))
                .unwrap_or(spot),
            facing,
        ))
    }

    /// `Group::action_swarm_around` for one unit: the approach move in
    /// front, then the order re-queued with the same action bit.
    pub(crate) fn swarm_around(&mut self, u: usize, b: usize, body: Body, action: bool) {
        let building = matches!(body, Body::Build(_));
        if let Some((spot, facing)) = self.swarm_spot(u, b, building) {
            self.add_move_facing_order(
                u,
                spot,
                MoveKind::ExploreTo,
                QueuePos::First,
                false,
                facing,
                None,
                false,
            );
        }
        let order = Order {
            flags: if action { flag::ACTION } else { 0 },
            body,
        };
        // The build order goes behind the approach move and ahead of
        // everything that was queued.
        let at = usize::from(self.current_order(u).is_some_and(Order::is_transit));
        self.units[u].orders.insert(at, order);
        self.update_action(u);
    }

    /// `Unit::do_build` (§5.2).
    fn do_build(&mut self, u: usize, frame: i64) {
        let Some(Order {
            body: Body::Build(b),
            flags,
        }) = self.current_order(u).copied()
        else {
            return;
        };
        let who = self.units[u].owner;
        let action = flags & flag::ACTION != 0;
        if !self.buildings.get(b).is_some_and(|bd| bd.alive) {
            self.kill_current_order(u);
            if self.order_type(u) != index::NONE {
                return;
            }
            self.build_done(u, Some(b));
            return;
        }
        if self.buildings[b].active {
            self.kill_current_order(u);
            if self.order_type(u) != index::NONE {
                self.check_build_order(u);
                return;
            }
            // §5.2 step 2, and the same predicate as step 6: only an
            // **oil platform** overrides it, and the exemption is
            // "not AI-driven *and* a gathering stance". An AI builder
            // that finds its site already finished never adopts it — it
            // goes back through `build_done`'s own arm.
            if self.building_ident(b) != Ident::OilPlatform
                && (self.ai_driven(who) || self.units[u].stance > 1)
            {
                self.build_done(u, Some(b));
                return;
            }
            if self.buildings[b].owner != who
                || !self.is_gather_type(b)
                || self.building_ident(b) == Ident::University
            {
                self.build_done(u, Some(b));
                return;
            }
            self.add_gather_order(u, b, QueuePos::New, false);
            return;
        }
        let inside =
            self.covers_tile(b, self.units[u].pos.tile()) && self.building_ident(b) != Ident::Farm;
        if !self.adjacent_to(u, b) || inside {
            self.kill_current_order(u);
            self.swarm_around(u, b, Body::Build(b), action);
            return;
        }
        // §5.2 step 4's first half, and it is a draw the arrival stand
        // would otherwise spend: the builder is put on its **work**
        // animation before it is turned, so `Guy::move`'s arrival test
        // (`cur_anim == CHAR_WALK`) never sees a walk again and the idle
        // roll at `Guy::set_anim+0x97a < Guy::move+0x19f` never fires
        // (`docs/ANIM.md` §4.6). A farm sows; everything else builds.
        let work = if self.building_ident(b) == Ident::Farm {
            anim::SOW
        } else {
            anim::BUILD
        };
        self.set_anim(u, work, false, true);
        let bpos = self.buildings[b].pos;
        let here = self.units[u].pos;
        self.units[u]
            .movement
            .set_heading(find_angle(bpos.x - here.x, bpos.y - here.y));
        let amount = build::builder_amount(
            &self.tuning,
            self.buildings[b].is_under_attack(),
            self.nation[who as usize].koreans,
        );
        if !self.do_construct(b, amount) {
            return;
        }
        let _ = frame;
        self.kill_current_order(u);
        let rest = self.order_type(u);
        if rest != index::NONE {
            self.check_build_order(u);
            return;
        }
        // §5.2 step 6 — **the builder keeps the building it just finished
        // only if it is not the AI's.** `(OILPLATFORM or (not
        // `unit_masks & 0x40000` and stance ∈ {0,1}))`; an AI citizen goes
        // to `build_done`, whose own arm searches afresh and may well pick
        // a different building.
        let gather = (self.building_ident(b) == Ident::OilPlatform
            || (!self.ai_driven(who) && self.units[u].stance <= 1))
            && self.buildings[b].owner == who
            && self.is_gather_type(b)
            && self.building_ident(b) != Ident::University;
        if gather {
            self.add_gather_order(u, b, QueuePos::New, false);
            return;
        }
        self.build_done(u, Some(b));
    }

    /// `Unit::check_build_order` (§5.3): pop the queued build orders, re-add
    /// the surviving sites least-crowded first, all with the action bit.
    fn check_build_order(&mut self, u: usize) {
        let mut sites = Vec::new();
        // A **cursor** walk, not a run of kills (the second reading's R3 C2 —
        // the first reading had this hedged as a stale decompile local; the
        // listing at `0x6036fd` is a plain `current_node = current_node->prev`
        // with no call between the flag test and the advance). Three
        // consequences the earlier code got wrong:
        //
        //  * an intervening transit move is **stepped over, not removed**;
        //  * only a `MOVE_TO` is stepped over — the loop's own gate is
        //    `get_type() == BUILD_AT || get_type() == MOVE_TO`, so a swarm's
        //    `EXPLORE_TO` ends the scan;
        //  * the site the scan **stops** on keeps its order (R3 C5, a finding
        //    neither reading made): the `break` jumps out before the
        //    `repath; kill`, so that site ends the pass with a duplicate,
        //    which the next `do_build` disposes of through its
        //    "already finished" branch.
        //
        // After a kill the original re-tails the list and resumes from the
        // front, which is what resetting the cursor to 0 reproduces.
        let mut i = 0usize;
        while let Some(o) = self.units[u].orders.get(i).copied() {
            match o.body {
                Body::Build(b) => {
                    let live = self
                        .buildings
                        .get(b)
                        .is_some_and(|bd| bd.alive && !bd.active);
                    if live && self.building_is_city(b) {
                        sites.push(b);
                        break;
                    }
                    self.repath(u);
                    self.remove_order_at(u, i);
                    if live {
                        sites.push(b);
                    }
                    i = 0;
                }
                Body::Move(m) if m.kind == MoveKind::MoveTo && !o.has(flag::ACTION) => {
                    if i + 1 >= self.units[u].orders.len() {
                        break;
                    }
                    i += 1;
                }
                _ => break,
            }
        }
        if sites.is_empty() {
            return;
        }
        if sites.len() > 1 {
            let who = self.units[u].owner;
            // The count skips the unit doing the counting — `check_build_
            // order@00603470:132` guards the whole body on
            // `this_00->o != this->o`. (`find_build_spot`'s otherwise
            // identical count does *not*; see the audit's R3 F2.)
            let me = self.units[u].index;
            let crowd = |sim: &Sim, b: usize| {
                sim.units
                    .iter()
                    .filter(|x| x.owner == who && x.alive() && x.index != me)
                    .filter(|x| {
                        x.orders
                            .iter()
                            .find(|o| !o.is_transit())
                            .is_some_and(|o| matches!(o.body, Body::Build(s) if s == b))
                    })
                    .count()
            };
            let least = (0..sites.len())
                .min_by_key(|&i| crowd(self, sites[i]))
                .expect("non-empty");
            sites.swap(0, least);
        }
        for &b in sites.iter().rev() {
            self.add_build_order(u, b, QueuePos::First, true);
        }
    }

    /// `Unit::build_done` (§5.5): what a builder does next.
    ///
    /// **The AI's arm is its own and stops there.** `unit_masks & 0x40000`
    /// takes `find_build_spot` → `find_repair_spot` → `find_gather_spot`
    /// with **no stance gate** and, crucially, **without the tail** that
    /// hands a human builder the site it has just finished. The two object
    /// searches are the unmodelled seams; the gather search is not.
    fn build_done(&mut self, u: usize, site: Option<usize>) {
        if self.units[u].orders.len() > 1 {
            return;
        }
        let who = self.units[u].owner;
        if self.ai_driven(who) {
            // **The AI's first question is where to build next**, and it is
            // what an AI citizen that has just finished a site does instead
            // of adopting it (§5.2 step 6). Great Lakes' `1/1` finishes
            // building `2010` on frame 2803 and walks to `2011` on 2804;
            // this crate sent it to gather until [`Self::find_build_spot`]
            // landed. `find_repair_spot` is still a seam, and it only ever
            // runs when the build search comes back empty.
            if self.find_build_spot(u) {
                return;
            }
            if !self.lobby.resources_unlimited() {
                self.find_gather_spot(u, self.tuning.unit_gather_respond_range * TILE);
            }
            return;
        }
        let stance = self.units[u].stance;
        if (stance == 1 || stance == 2) && self.find_build_spot(u) {
            return;
        }
        if stance <= 1 && self.find_gather_spot(u, self.tuning.unit_gather_respond_range * TILE) {
            return;
        }
        if let Some(b) = site
            && self
                .buildings
                .get(b)
                .is_some_and(|bd| bd.alive && bd.owner == who)
            && self.is_gather_type(b)
            && !matches!(
                self.building_ident(b),
                Ident::University | Ident::OilPlatform
            )
            && self.gather_room(b)
        {
            self.add_gather_order(u, b, QueuePos::New, false);
        }
    }

    /// `Objects::find_builds`/`Objects::find_units`' ring index:
    /// `(range + 0x2ff) / 0x300`, the range in **cells** rounded up, and a
    /// cell is four tiles. Both searches take their cheap circle walk only
    /// while that ring holds no more points than the thing being counted
    /// (`num_def_builds`, a flat **200** from `Game::init_data@0058dca0`,
    /// and `total_units`, the live unit count); past it they walk the
    /// object lists instead.
    fn ring_index(range: i32) -> usize {
        ((range.max(0) + 0x2ff) / 0x300).min(0x40) as usize
    }

    /// `Objects::find_builds(SEARCH_FRIENDLY, who, range, 0x200,
    /// FILTER_CONSTRUCT)` as `find_build_spot` uses it, with that caller's
    /// own two extra tests folded in: **my** sites, and not under attack
    /// (`build_masks & 0x20`).
    ///
    /// `FILTER_CONSTRUCT` is arm 5 of `Search::valid_filter@0067dbb0`'s
    /// jump table — the table is at `0067e57c` and the arm at `0067dd54`,
    /// read from the PE because the decompiler prints the dispatch as an
    /// indirect jump. It is `vtable+0xc` (the object exists) and then a
    /// **negated** `vtable+0x4c` on what `vtable+0x40` hands back: an
    /// object that is not active, which for a building is a site still
    /// under construction. Arm 6 next door is `FILTER_DAMAGED` and is the
    /// same pair un-negated plus `+0x24 damage != 0`, which is how the
    /// polarity is settled.
    ///
    /// The `0x200` flag is the region gate: only cells whose region is the
    /// searcher's own. Order is the circle's, then — because
    /// `Object::add_to_world` pushes onto the head of the cell's chain —
    /// the **newest** object of a cell first. Nothing threads a building
    /// into `chain_heads` here (`docs/QUEUE.md` 48), so the within-cell
    /// half is descending slot order rather than a chain walk.
    fn find_construct_sites(&self, u: usize, range: i32) -> Vec<usize> {
        let who = self.units[u].owner;
        let here = self.units[u].pos;
        let Some(region) = self.world.region_of(here.cell()) else {
            return Vec::new();
        };
        let mine: Vec<usize> = (0..self.buildings.len())
            .filter(|&b| {
                let bd = &self.buildings[b];
                bd.alive && !bd.active && bd.owner == who && !bd.is_under_attack()
            })
            .collect();
        if mine.is_empty() {
            return Vec::new();
        }
        let circle = crate::ai_place::circle();
        let ring = Self::ring_index(range);
        // The list path, when the circle is dearer than the object arrays.
        if circle.radius[ring] > NUM_DEF_BUILDS {
            return mine
                .into_iter()
                .filter(|&b| {
                    let p = self.buildings[b].pos;
                    self.world.region_of(p.cell()) == Some(region)
                        && vector_dist(p.x - here.x, p.y - here.y) <= range
                })
                .collect();
        }
        let c0 = here.cell();
        let mut out = Vec::new();
        for i in 0..circle.radius[ring] {
            let c = crate::world::Cell::new(c0.x + circle.x[i], c0.y + circle.y[i]);
            if !self.world.contains(c) || self.world.region_of(c) != Some(region) {
                continue;
            }
            out.extend(
                mine.iter()
                    .rev()
                    .filter(|&&b| self.buildings[b].pos.cell() == c)
                    .copied(),
            );
        }
        out
    }

    /// The builder tally `find_build_spot` takes over
    /// `Objects::find_units(SEARCH_FRIENDLY, who, range, 0x200,
    /// FILTER_BUILDREPAIR)`: for every friendly unit the search returns
    /// whose **action** is a `BUILD_AT` on a site of mine, one on that
    /// site's slot — and the scan stops at the first slot it matches, so a
    /// unit is counted once.
    ///
    /// The filter itself is subsumed: a unit with a `BUILD_AT` action is
    /// one `FILTER_BUILDREPAIR` keeps. What is *not* subsumed is the
    /// search's own reach, so the two paths are both here — and the list
    /// path's region test is the searcher's cell region against the
    /// candidate's, where the original indexes its cell grid with **tile**
    /// coordinates (`div_3_table[pos >> 6]`, a `>> 8` everywhere else).
    /// That arithmetic is not reproduced; it is a count that breaks ties.
    fn build_crowd(&self, u: usize, range: i32, sites: &[usize]) -> Vec<i32> {
        let who = self.units[u].owner;
        let here = self.units[u].pos;
        let mut counts = vec![0i32; sites.len()];
        let circle = crate::ai_place::circle();
        let ring = Self::ring_index(range);
        let live = self.units.iter().filter(|x| x.alive()).count();
        let tally = |sim: &Self, o: usize, counts: &mut Vec<i32>| {
            let unit = &sim.units[o];
            if !unit.alive() || (unit.owner != who && !sim.is_ally(who, unit.owner)) {
                return;
            }
            let Some(i) = sim.action_of(o) else { return };
            let Body::Build(b) = sim.units[o].orders[i].body else {
                return;
            };
            if sim.buildings.get(b).is_none_or(|bd| bd.owner != who) {
                return;
            }
            if let Some(k) = sites.iter().position(|&s| s == b) {
                counts[k] += 1;
            }
        };
        if circle.radius[ring] <= live {
            let c0 = here.cell();
            let region = self.world.region_of(c0);
            for i in 0..circle.radius[ring] {
                let c = crate::world::Cell::new(c0.x + circle.x[i], c0.y + circle.y[i]);
                if !self.world.contains(c) || self.world.region_of(c) != region {
                    continue;
                }
                let slot = (c.y as usize) * (self.world.width() as usize) + (c.x as usize);
                let mut next = self.chain_heads[slot];
                while let Some(o) = next {
                    next = self.units[o].down;
                    tally(self, o, &mut counts);
                }
            }
        } else {
            for o in 0..self.units.len() {
                let p = self.units[o].pos;
                if vector_dist(p.x - here.x, p.y - here.y) > range {
                    continue;
                }
                tally(self, o, &mut counts);
            }
        }
        counts
    }

    /// `Unit::find_build_spot@00603e20` (§5.5) — the search an AI builder
    /// takes the moment a site is finished, and the one an idle builder
    /// takes ahead of the gather search.
    ///
    /// Range is `UNIT_BUILD_RESPOND_RANGE × 0xc0`, **doubled** on worker
    /// stance 1 or 2. The candidates are [`Self::find_construct_sites`];
    /// the choice is the fewest builders already on one, and the first of
    /// a tie, because the original's min-search is a strict `<` walking up
    /// from index 0. The winner is a `swarm_around` with `BUILD_AT` and no
    /// action bit.
    ///
    /// The original wraps the swarm in a one-member `Group`; every other
    /// swarm call site here is the same single-unit shape (§10).
    pub(crate) fn find_build_spot(&mut self, u: usize) -> bool {
        let stance = self.units[u].stance;
        let wide = stance == 1 || stance == 2;
        let range = self.tuning.unit_build_respond_range * if wide { TILE * 2 } else { TILE };
        let sites = self.find_construct_sites(u, range);
        if sites.is_empty() {
            return false;
        }
        let counts = self.build_crowd(u, range, &sites);
        let mut best = 0;
        for i in 1..sites.len() {
            if counts[i] < counts[best] {
                best = i;
            }
        }
        let site = sites[best];
        self.swarm_around(u, site, Body::Build(site), false);
        true
    }

    /// `Unit::do_repair` (§5.6), on the existing repair step.
    fn do_repair(&mut self, u: usize, frame: i64) {
        let Some(Order {
            body: Body::Repair(b),
            flags,
        }) = self.current_order(u).copied()
        else {
            return;
        };
        let who = self.units[u].owner;
        let action = flags & flag::ACTION != 0;
        // §5.6's first line, and unlike `do_build`'s it is **ahead of every
        // gate**: a repairer is put on `CHAR_REPAIR` even on the frame the
        // order dies. Same reason as `do_build`'s (`docs/ANIM.md` §4.6).
        self.set_anim(u, anim::REPAIR, false, true);
        // §5.6: an AI repairer takes `find_repair_spot` (a seam) instead of
        // adopting the building it has just mended — the same `unit_masks &
        // 0x40000` split as `do_build`'s two.
        let gather_after = |sim: &mut Sim| {
            if !sim.ai_driven(who)
                && sim.order_type(u) == index::NONE
                && sim.buildings[b].owner == who
                && sim.units[u].stance < 2
                && sim.is_gather_type(b)
                && sim.building_ident(b) != Ident::University
            {
                sim.add_gather_order(u, b, QueuePos::New, false);
            }
        };
        let bd = &self.buildings[b];
        if bd.damage == 0 {
            self.kill_current_order(u);
            gather_after(self);
            return;
        }
        if bd.owner != who && !self.is_ally(who, bd.owner) {
            self.kill_current_order(u);
            gather_after(self);
            return;
        }
        if !bd.active || (bd.is_under_attack() && !action) {
            self.kill_current_order(u);
            return;
        }
        let bpos = bd.pos;
        if matches!(self.world.owner_at(bpos), crate::Owner::Player(p) if self.is_enemy(who, p)) {
            self.kill_current_order(u);
            return;
        }
        if !self.adjacent_to(u, b) {
            self.kill_current_order(u);
            self.swarm_around(u, b, Body::Repair(b), action);
            return;
        }
        self.repair_step(u, b, frame);
    }

    /// `Unit::do_garrison`'s order side (§5.7): the gates are the garrison
    /// mechanic's; not adjacent → a move **in front**, the order stays.
    fn do_garrison_order(&mut self, u: usize) {
        let Some(Order {
            body:
                Body::Garrison {
                    building: b,
                    search,
                },
            // The redirected order's action bit is a constant 0, so the
            // original order's flags are not read here (R3 G6).
            flags: _,
        }) = self.current_order(u).copied()
        else {
            return;
        };
        match self.garrison(u, b) {
            Ok(()) => self.kill_garrison_order(u),
            Err(GarrisonRefused::NotAdjacent) => {
                let bd = &self.buildings[b];
                let (xs, ys) = bd.ty.map_or((1, 1), |t| {
                    (self.build_types[t].x_size, self.build_types[t].y_size)
                });
                let r = xs.min(ys) * HALF_TILE + SNAP;
                let here = self.units[u].pos;
                let angle = find_angle(here.x - bd.pos.x, here.y - bd.pos.y);
                let bpos = bd.pos;
                match self.find_nearby_spot(u, bpos, r, -1, 0, angle, None) {
                    Some(spot) => {
                        self.add_move_order(u, spot, MoveKind::MoveTo, QueuePos::First, false)
                    }
                    None => self.kill_current_order(u),
                }
            }
            Err(GarrisonRefused::Full | GarrisonRefused::NoCapacity) if search => {
                let who = self.units[u].owner;
                let city = self.buildings[b].city;
                let next = self.find_garrison_build(u, city, who);
                self.kill_current_order(u);
                if let Some(n) = next {
                    // The action bit of the redirected order is **not** the
                    // original's: `do_garrison@005e6b80:288` reads it through
                    // vtable slot `+0x2c` on the order's `UnitOrder`
                    // sub-object, and `GarrisonOrder`'s entry there
                    // (`0xb48ee8+0x2c` → `0x41bff0`) is the folded
                    // `xor eax,eax; ret`. So it is always 0.
                    // (`docs/audit/2026-08-21-orders.md` R3 G6.)
                    self.add_garrison_order(u, n, true, QueuePos::First, false);
                }
            }
            Err(_) => self.kill_current_order(u),
        }
    }

    /// `Unit::kill_garrison_order`: every GARRISON action down the squad is
    /// killed; here the unit's own.
    fn kill_garrison_order(&mut self, u: usize) {
        while let Some(a) = self.action_of(u) {
            if !matches!(self.units[u].orders[a].body, Body::Garrison { .. }) {
                break;
            }
            self.repath(u);
            self.kill_current_order(u);
        }
    }

    /// `Unit::find_garrison_build`: the city's nearest active building with
    /// room that the unit may enter.
    fn find_garrison_build(&self, u: usize, city: Option<usize>, who: Player) -> Option<usize> {
        let city = city?;
        let here = self.units[u].pos;
        let uty = self.units[u].ty?;
        self.buildings
            .iter()
            .enumerate()
            .filter(|(_, b)| b.alive && b.active && b.owner == who && b.city == Some(city))
            .filter(|(i, b)| {
                b.ty.is_some_and(|t| self.can_garrison(uty, t))
                    && self.num_inside(*i) < self.garrison_limit(*i)
            })
            .min_by_key(|(_, b)| vector_dist(b.pos.x - here.x, b.pos.y - here.y))
            .map(|(i, _)| i)
    }

    // ------------------------------------------------------------------
    // The gather chain — §6.1
    // ------------------------------------------------------------------

    /// `BuildType` vslot `0x90`: `build_flags & 0x40`.
    pub fn is_gather_type(&self, b: usize) -> bool {
        self.buildings[b]
            .ty
            .is_some_and(|t| self.build_types[t].has(bflags::GATHER))
    }

    /// `build_flags & FLAT`.
    pub fn is_flat(&self, b: usize) -> bool {
        self.buildings[b]
            .ty
            .is_some_and(|t| self.build_types[t].has(bflags::FLAT))
    }

    /// The unit type's worker kind.
    pub fn worker_of(&self, u: usize) -> Worker {
        self.units[u]
            .ty
            .map_or(Worker::None, |t| self.unit_types[t].worker)
    }

    /// `BuildData::gather_max` against the chain: room for one more.
    fn gather_room(&self, b: usize) -> bool {
        self.buildings[b]
            .gather_max
            .is_none_or(|max| self.num_gatherers(b, false, false) < max)
    }

    /// `Build::add_gatherer` (§6.1): same owner, room, a worker on the map,
    /// not already in the chain → prune, then push-front.
    pub fn add_gatherer(&mut self, b: usize, u: usize) -> bool {
        let unit = &self.units[u];
        if self.buildings[b].owner != unit.owner
            || !self.gather_room(b)
            || !unit.alive()
            || !unit.on_map
            || self.worker_of(u) == Worker::None
            || self.buildings[b].gatherers.contains(&u)
        {
            return false;
        }
        self.check_gatherers(b);
        self.buildings[b].gatherers.insert(0, u);
        true
    }

    /// `Build::remove_gatherer`.
    pub fn remove_gatherer(&mut self, b: usize, u: usize) {
        self.buildings[b].gatherers.retain(|&x| x != u);
    }

    /// `Build::check_gatherers`: prune the dead, the re-tasked, the off-map.
    pub fn check_gatherers(&mut self, b: usize) {
        let keep: Vec<usize> = self.buildings[b]
            .gatherers
            .iter()
            .copied()
            .filter(|&u| {
                self.units[u].alive() && self.units[u].on_map && self.is_gathering_at(u, b, false)
            })
            .collect();
        self.buildings[b].gatherers = keep;
    }

    /// `BuildData::is_gathered_by`.
    pub fn is_gathered_by(&self, b: usize, u: usize) -> bool {
        self.buildings[b].gatherers.contains(&u)
    }

    /// `UnitData::is_gathering_at(o, who, strict)`: a worker whose **action**
    /// is a GATHER on this building — and, strictly, arrived.
    ///
    /// The match is on [`Sim::action_of`], not the front of the list
    /// (`is_gathering_at@00608880`: `pUVar3 = get_action(this)`, then
    /// `get_type() == 7`). That is load-bearing, not pedantry: `do_gather`
    /// and `do_non_flat_gather` insert their walks as `QUEUE_FIRST` moves
    /// *without* the action bit, so on every walk-out and walk-back leg the
    /// front order is a move and only the action walk still finds the
    /// GATHER. Reading the front instead made a woodcutter stop counting as
    /// a gatherer the moment it set off — and be pruned out of its own chain
    /// by `check_gatherers`. (`docs/audit/2026-08-21-orders.md` R4 G16.)
    ///
    /// The inside-the-building arm is **scholar-only**: the original gates it
    /// on `ptype[4] ∈ {SCHOLARS, SCHOLARSKOREAN}` (G17), so a garrisoned
    /// citizen matches nothing. A platform's peasants are counted by
    /// `num_gatherers` through `count_inside` instead.
    pub fn is_gathering_at(&self, u: usize, b: usize, strict: bool) -> bool {
        let unit = &self.units[u];
        if !unit.on_map {
            return self.worker_of(u) == Worker::Scholar && unit.inside == Some(b);
        }
        if self.worker_of(u) == Worker::None {
            return false;
        }
        match self
            .action_of(u)
            .and_then(|i| unit.orders.get(i))
            .map(|o| o.body)
        {
            Some(Body::Gather(g)) if g.building == b => !strict || g.been_there,
            _ => false,
        }
    }

    /// `BuildData::num_gatherers(arrived, skip_decoys)`: the garrisoned
    /// gatherers of a university or platform, plus the chain members that
    /// are gathering here. `calc_gather` calls it `(1, 1)`.
    pub fn num_gatherers(&self, b: usize, arrived: bool, skip_decoys: bool) -> i32 {
        let ident = self.building_ident(b);
        let mut n = 0;
        if matches!(ident, Ident::University | Ident::OilPlatform) {
            let want = if ident == Ident::University {
                Worker::Scholar
            } else {
                Worker::Citizen
            };
            n += self.buildings[b]
                .garrison
                .iter()
                .filter(|&&u| self.worker_of(u) == want)
                .count() as i32;
        }
        n += self.buildings[b]
            .gatherers
            .iter()
            .filter(|&&u| self.is_gathering_at(u, b, arrived))
            .filter(|&&u| !(skip_decoys && self.units[u].decoy))
            .count() as i32;
        n
    }

    /// `Build::all_gathering`: every chain member is out at its tile.
    fn all_gathering(&self, b: usize) -> bool {
        self.buildings[b].gatherers.iter().all(|&u| {
            matches!(self.units[u].orders.front().map(|o| o.body),
                Some(Body::Gather(g)) if !g.goto_build && g.wait >= 0)
        })
    }

    // ------------------------------------------------------------------
    // `do_gather` — §6.3
    // ------------------------------------------------------------------

    /// `Unit::do_gather`: the walk, the arrival, the stand.
    fn do_gather(&mut self, u: usize, frame: i64) {
        let Some(Order {
            body: Body::Gather(mut g),
            ..
        }) = self.current_order(u).copied()
        else {
            return;
        };
        let b = g.building;
        let who = self.units[u].owner;
        let bd = &self.buildings[b];
        if bd.owner != who || !bd.alive || !bd.active {
            self.kill_current_order(u);
            return;
        }
        let ident = self.building_ident(b);
        let fail = |sim: &mut Sim| {
            sim.kill_current_order(u);
            sim.add_think_order(u);
            if sim.is_flat(b) && !sim.covers_tile(b, sim.units[u].pos.tile()) {
                let p = sim.buildings[b].pos;
                sim.add_move_order(u, p, MoveKind::MoveTo, QueuePos::First, false);
            }
        };
        if !g.been_there && !self.is_gathered_by(b, u) {
            // The retry for a unit refused at issue. An oil platform, and any
            // non-land unit, skip the chain entirely and go straight on to
            // the arrival path — `do_gather@005ef2a0:98`
            // `if (build_type == 0x1a6 || ptype[0x218] != 0) goto ARRIVED`.
            // Registering them gave the platform chain entries the original
            // never creates. (`docs/audit/2026-08-21-orders.md` R4 G19.)
            let skips_chain = ident == Ident::OilPlatform
                || self.units[u].kind.domain != crate::attrition::Domain::Land;
            if !skips_chain && (!self.gather_room(b) || !self.add_gatherer(b, u)) {
                fail(self);
                return;
            }
        }
        if ident == Ident::Farm && self.buildings[b].city.is_none() {
            fail(self);
            return;
        }
        if matches!(ident, Ident::Mine | Ident::Woodcutter) {
            let phase = frame + i64::from(self.units[u].index) * 4;
            if phase & 127 == 0
                && self.buildings[b]
                    .gather_max
                    .is_some_and(|max| self.num_gatherers(b, false, false) > max)
            {
                self.kill_current_order(u);
                self.add_think_order(u);
                return;
            }
            self.do_non_flat_gather(u, g);
            return;
        }
        let here = self.units[u].pos;
        let bpos = self.buildings[b].pos;
        if ident != Ident::Farm {
            if !self.adjacent_to(u, b) {
                let (xs, ys) = self.buildings[b].ty.map_or((1, 1), |t| {
                    (self.build_types[t].x_size, self.build_types[t].y_size)
                });
                let d = xs.min(ys) * HALF_TILE + SNAP;
                let angle = find_angle(here.x - bpos.x, here.y - bpos.y);
                let Some(spot) = self.find_nearby_spot(u, bpos, d, -1, 0, angle, None) else {
                    self.kill_current_order(u);
                    self.units[u].cant_reach = true;
                    return;
                };
                if !self.is_gathered_by(b, u) && ident != Ident::OilPlatform {
                    g.been_there = false;
                    self.store_gather(u, g);
                    return;
                }
                self.add_move_order(u, spot, MoveKind::MoveTo, QueuePos::First, false);
                return;
            }
        } else if !self.covers_tile(b, here.tile()) {
            if self.buildings[b]
                .gather_max
                .is_some_and(|max| self.num_gatherers(b, false, false) > max)
            {
                self.kill_current_order(u);
                self.units[u].cant_reach = true;
                return;
            }
            if !self.is_gathered_by(b, u) {
                g.been_there = false;
                self.store_gather(u, g);
                return;
            }
            self.add_move_order(u, bpos, MoveKind::MoveTo, QueuePos::First, false);
            return;
        }
        // Arrived.
        if !g.been_there {
            if self.buildings[b]
                .gather_max
                .is_some_and(|max| self.num_gatherers(b, false, false) > max)
            {
                self.kill_current_order(u);
                self.units[u].cant_reach = true;
                return;
            }
            g.been_there = true;
            self.store_gather(u, g);
            self.ledgers[who as usize].dirty = true;
            if self.worker_of(u) == Worker::Scholar || ident == Ident::OilPlatform {
                self.go_inside(u, b);
                self.check_gatherers(b);
                self.kill_current_order(u);
                return;
            }
            self.check_gatherers(b);
        }
        if ident != Ident::Farm {
            // The oil well: one worker a frame bumps the building's counter;
            // stand at `b.x + 0x120`.
            if !self.buildings[b].gather_bumped {
                self.buildings[b].recharging += 1;
                self.buildings[b].gather_bumped = true;
            }
            self.units[u].movement.set_heading(Angle(0x4000_0000));
            let stand = Pos::new(bpos.x + OILWELL_OFFSET, bpos.y);
            if self.world.accepts(stand) {
                let from = self.units[u].pos;
                self.set_new_location(u, stand, true);
                // `do_gather`'s `set_new_location(…, 1, 1)`: the whole disc,
                // not the ring (`docs/VISION.md` §6).
                self.moved_to(u, from, false);
            }
            return;
        }
        self.do_farm(u, b, g, frame);
    }

    /// Writes a gather order's fields back — **to the gather order, not to
    /// the front of the list**.
    ///
    /// The original holds a pointer to the order object (`go`) for the whole
    /// of `do_gather` and `do_non_flat_gather`, and every walk those
    /// functions issue goes in *front* of it: `add_move_order(…, 1, …)`
    /// allocates a new order and links it at the head, leaving `go` valid.
    /// So a branch that issues a walk and then writes `go->goto_build` is
    /// writing the same object it was handed.
    ///
    /// Writing the front instead dropped every field of the branches that
    /// walk — most visibly the return to camp, which sets `goto_build = 1`
    /// and `wait = 32` *after* `add_move_order` (`docs/ORDERS.md` §6.4).
    /// Without them a woodcutter that had finished its shift never left the
    /// `goto_build == 0, wait < 0` arm: it walked to the camp, arrived, and
    /// re-issued the same walk and the same `CHAR_DEFAULT` stand for the
    /// rest of the game, so it never unloaded and never took another tile.
    fn store_gather(&mut self, u: usize, g: GatherOrder) {
        if let Some(x) = self.units[u]
            .orders
            .iter_mut()
            .find_map(|o| match &mut o.body {
                Body::Gather(x) => Some(x),
                _ => None,
            })
        {
            *x = g;
        }
    }

    /// `Unit::do_non_flat_gather` (§6.4): wood and ore, given the building's
    /// resource tiles. Without a tile list the worker stays at the camp with
    /// `been_there` set — an input.
    fn do_non_flat_gather(&mut self, u: usize, mut g: GatherOrder) {
        let b = g.building;
        let who = self.units[u].owner;
        let wood = self.building_ident(b) == Ident::Woodcutter;
        if g.been_there && !self.buildings[b].gather_bumped {
            self.buildings[b].recharging += 1;
            self.buildings[b].gather_bumped = true;
        }
        let bpos = self.buildings[b].pos;
        let here = self.units[u].pos;
        let (xs, ys) = self.buildings[b].ty.map_or((1, 1), |t| {
            (self.build_types[t].x_size, self.build_types[t].y_size)
        });
        let d = xs.min(ys) * HALF_TILE + SNAP;
        if !g.goto_build {
            // `do_non_flat_gather:96`, the first statement of this half:
            // the carrying walk is dropped whole, and one of the four is
            // put back by whichever walk this frame issues.
            self.units[u].carry &= !CARRY_ANY;
            if !g.been_there {
                g.been_there = true;
                self.ledgers[who as usize].dirty = true;
            }
            if g.wait < 0 {
                // Return to the camp.
                let angle = find_angle(here.x - bpos.x, here.y - bpos.y);
                match self.find_nearby_spot(u, bpos, d, -1, 0, angle, None) {
                    None => {
                        // No spot: keep chopping (`do_non_flat_gather:130`).
                        let work = if wood {
                            crate::anim::CHOP_WOOD
                        } else {
                            crate::anim::MINE_ORE
                        };
                        self.set_anim(u, work, false, true);
                        g.wait = 20;
                    }
                    Some(spot) => {
                        self.mark(crate::anim::SITE_STAND_RETURN);
                        self.set_default_anim(u);
                        self.add_move_order(u, spot, MoveKind::MoveTo, QueuePos::First, false);
                        g.goto_build = true;
                        g.wait = 32;
                        // `:150` / `:154` — the walk back is a *carrying*
                        // one, by the camp's resource.
                        self.units[u].carry |= if wood {
                            CARRY_WITH_WOOD
                        } else {
                            CARRY_WITH_ORE
                        };
                    }
                }
                self.store_gather(u, g);
                return;
            }
            // **The guy's own animation is read before the tile is**
            // (`do_non_flat_gather:255`, `005f0d0f`): a worker already in
            // its resource loop never reaches the tile arithmetic below,
            // and the branch it takes instead is where the steady-state
            // wait lives.
            //
            // - `CHAR_MINE_ORE` — a miner mid-swing does *nothing*: no
            //   decrement, no facing, no animation. Its `wait` is the
            //   1,000,000 the tile choice gave it and it stays out.
            // - `CHAR_CHOP_WOOD` — the decrement, and on zero the reroll,
            //   and that is the whole frame. No facing, no `set_anim`.
            //
            // The reroll here is `% 100 + 300`, **not** the `% 50 + 100`
            // of the arrival frame below. The two used to be one branch
            // and the count could not see it: both sites draw once, so
            // the word stayed matched for six hundred frames while the
            // worker's clock ran at a third of the original's and sent it
            // home two hundred frames early (§6.4, `docs/SYNC.md` §3.18).
            let anim = self.units[u].guys.first().map_or(0, |g| g.anim);
            if anim == crate::anim::MINE_ORE {
                self.store_gather(u, g);
                return;
            }
            if anim == crate::anim::CHOP_WOOD {
                g.wait -= 1;
                if g.wait != 0 {
                    self.store_gather(u, g);
                    return;
                }
                g.wait = if self.all_gathering(b) {
                    -1
                } else {
                    self.mark(SITE_WORK_WAIT);
                    300 + self.rng.roll() % 100
                };
                self.store_gather(u, g);
                return;
            }
            let Some(t) = g.tile else {
                // A tile was never chosen: nothing to walk to.
                self.store_gather(u, g);
                return;
            };
            let centre = Pos::new(t.x * TILE + HALF_TILE, t.y * TILE + HALF_TILE);
            if vector_dist(centre.x - here.x, centre.y - here.y) < AT_TILE {
                // The **arrival** frame: at the tile and not yet in the
                // loop. The decrement and its reroll come first
                // (`do_non_flat_gather:267`), then the facing, and the
                // work animation is the function's last statement — which
                // is what puts every later frame on the branch above.
                g.wait -= 1;
                if g.wait == 0 {
                    if self.all_gathering(b) {
                        // `LAB_005f0ef1`: the return is immediate, so this
                        // frame sets neither the facing nor the animation.
                        g.wait = -1;
                        self.store_gather(u, g);
                        return;
                    }
                    self.mark(SITE_ARRIVE_WAIT);
                    g.wait = 100 + self.rng.roll() % 50;
                }
                self.units[u]
                    .movement
                    .set_heading(find_angle(centre.x - here.x, centre.y - here.y));
                let work = if wood {
                    crate::anim::CHOP_WOOD
                } else {
                    crate::anim::MINE_ORE
                };
                self.set_anim(u, work, false, true);
                self.store_gather(u, g);
                return;
            }
            // The stand before the approach — `do_non_flat_gather:275`,
            // the call at `005f113f` and so the site `+0xfd4`. §6.4's
            // pseudocode has always carried it and the implementation did
            // not; the whole-frame sequence check is what made the gap
            // countable (`docs/SYNC.md` §5.1).
            self.mark(crate::anim::SITE_STAND_TILE);
            self.set_default_anim(u);
            let angle = find_angle(here.x - centre.x, here.y - centre.y);
            match self.find_nearby_spot(u, centre, TILE, 0x100, 2, angle, None) {
                Some(spot) if spot != here => {
                    if !self.is_gathered_by(b, u) {
                        g.been_there = false;
                        self.store_gather(u, g);
                        return;
                    }
                    self.add_move_order(u, spot, MoveKind::MoveTo, QueuePos::First, false);
                    // `:376` and `LAB_005f13da` — the approach is a walk
                    // *to* the resource.
                    self.units[u].carry |= if wood { CARRY_TO_WOOD } else { CARRY_TO_ORE };
                }
                _ => {
                    g.tile = None;
                    g.goto_build = true;
                    g.wait = -1;
                    if g.dist_mod > 0 {
                        g.dist_mod -= 1;
                    }
                }
            }
            self.store_gather(u, g);
            return;
        }
        // Heading to, or at, the camp.
        if g.wait >= 0 {
            if self.adjacent_to(u, b) {
                // At the camp, and the unload — `do_non_flat_gather:398`
                // through `:416`, in the original's own order: the wait is
                // decremented, `been_there` set, and a wait that has run
                // out returns *before* the facing and the animation. There
                // is **no idle stand here**: the branch is a two-way
                // `CHAR_DUMP_WOOD` / `CHAR_DUMP_ORE` on the camp's
                // resource, `been_there` is written and never read, and
                // the listing at `5f0b5e`–`5f0b89` shows the two `set_anim`
                // calls and no third.
                //
                // A first arrival used to stand idle here, which is where
                // the sim's two extra frame-0 draws came from: run20's
                // trace has no `Unit::do_non_flat_gather` draw at frame 0
                // at all, and the sequence comparison in `rondata::diff`
                // named this call as the divergence (`docs/SYNC.md` §4.2,
                // §6 — the stand half of the stand/wrap swap).
                g.wait -= 1;
                if !g.been_there {
                    g.been_there = true;
                    self.ledgers[who as usize].dirty = true;
                }
                if g.wait >= 0 {
                    self.units[u]
                        .movement
                        .set_heading(find_angle(bpos.x - here.x, bpos.y - here.y));
                    let dump = if wood {
                        crate::anim::DUMP_WOOD
                    } else {
                        crate::anim::DUMP_ORE
                    };
                    self.set_anim(u, dump, false, true);
                }
                self.store_gather(u, g);
                return;
            }
            if !self.is_gathered_by(b, u) {
                g.been_there = false;
                self.store_gather(u, g);
                return;
            }
            let angle = find_angle(here.x - bpos.x, here.y - bpos.y);
            if let Some(spot) = self.find_nearby_spot(u, bpos, d, -1, 0, angle, None) {
                self.add_move_order(u, spot, MoveKind::MoveTo, QueuePos::First, false);
                self.store_gather(u, g);
                return;
            }
            if !g.been_there && vector_dist(bpos.x - here.x, bpos.y - here.y) > 0x600 {
                match self.find_nearby_spot(u, bpos, 0x600, -1, 0, angle, None) {
                    Some(spot) => {
                        self.add_move_order(u, spot, MoveKind::MoveTo, QueuePos::First, false)
                    }
                    None => self.kill_current_order(u),
                }
                return;
            }
            g.wait = -1;
            self.store_gather(u, g);
            return;
        }
        // Choose a tile. The stand first (`do_non_flat_gather:512`, whose
        // call is at `005f027a` and whose return address is therefore
        // `+0x10f` — the listing's three `CHAR_DEFAULT` stands are
        // `+0x10f`, `+0xb99` and `+0xfd4`, and the decompile's own
        // `pTVar19 = (TCoord *)0x5f027f` names this one) — the
        // woodcutters' frame-0 draws on run12 are this call (`docs/ANIM.md`
        // §4).
        if !g.been_there {
            g.been_there = true;
            self.ledgers[who as usize].dirty = true;
        }
        self.mark(crate::anim::SITE_STAND_GATHER);
        self.set_default_anim(u);
        // `tx < 0 || ty < 0 || !has_gather_access(tx, ty, who, 1, 0)` — the
        // held tile is given up and chosen again when the walk into it has
        // closed (`005f0655`, the `else` arm's `goto LAB_005f02b2`).
        let stale = match g.tile {
            None => true,
            Some(t) => !self.has_gather_access(t, who),
        };
        if stale {
            g.tile = None;
            let btile = bpos.tile();
            let dm = g.dist_mod;
            let mut best: Option<(i32, usize)> = None;
            for i in 0..self.buildings[b].gather_from.len() {
                let t = self.buildings[b].gather_from[i];
                // **The candidate is filtered before it is scored**
                // (`005f0575`): the tile still carries `mask & 0x4000`, and
                // `has_gather_access` still holds for it. A tile ringed by
                // its own kind has no orthogonal neighbour to stand on, so
                // it is never chosen however near the camp it is — which is
                // exactly what separates player 1's `(213, 92)` from the
                // nearer `(214, 93)` on run10 (`docs/ORDERS.md` §6.4).
                if self.world.tile_mask(t) & tile::BLOCKED == 0 || !self.has_gather_access(t, who) {
                    continue;
                }
                let score = vector_dist((t.x - btile.x).abs(), (t.y - btile.y).abs()).max(3) * dm
                    + (i as i32 >> 2);
                if best.is_none_or(|(s, _)| score < s) {
                    best = Some((score, i));
                }
            }
            // Nothing eligible — an empty list, or every tile walled in.
            // The original returns here (`005f05b0`), before the wait is
            // rolled, so the worker keeps the camp and takes no draw.
            let Some((_, at)) = best else {
                self.store_gather(u, g);
                return;
            };
            let pick = self.buildings[b].gather_from.remove(at);
            self.buildings[b].gather_from.push(pick);
            g.tile = Some(pick);
        }
        self.mark(SITE_TILE_WAIT);
        g.wait = 400 + self.rng.roll() % 200;
        g.goto_build = false;
        if !wood {
            g.wait = 1_000_000;
        }
        // `:694` / `:700`, the function's last statement either way.
        self.units[u].carry |= if wood { CARRY_TO_WOOD } else { CARRY_TO_ORE };
        self.store_gather(u, g);
    }

    /// The farm stand (§6.5), `Unit::do_gather@005ef2a0`'s wheat branch on
    /// the cell under the farmer, by the cell's state and the guy's
    /// animation (`'#'` is the sow anim — index 35, the one every farmer
    /// shows — and `'$'` the reap):
    ///
    /// ```text
    /// pasture: set_anim(SOW); off phase → return; else → inner tile
    /// 0: not reaping → as 1;                    else → new tile
    /// 1: set_anim(SOW); Farms::grow
    /// 2: not sowing  → set_anim(REAP); snip;    else → new tile
    /// 3: set_anim(REAP)
    /// ```
    ///
    /// The clock is `farms.rs`'s: the farmer's `grow` and `inc_time`'s add
    /// each frame ripen the cell on frame 100's `grow` — the farmer is still
    /// sowing, so the next frame is the "new tile": two draws
    /// (`GameAccess::rnd` on the type's `x_size`, then its `y_size`) and a
    /// move to that tile's centre, in front of the gather order. That is
    /// the original's re-target on the log's frame 102 — sim-frame 101,
    /// where run13 shows all six farmers' `orders_x/y` change and the walk
    /// start on 102 (`docs/SYNC.md` §4). A **pasture** has no such clock
    /// and no switch: see the arm below.
    fn do_farm(&mut self, u: usize, b: usize, _g: GatherOrder, frame: i64) {
        let Some(ty) = self.buildings[b].ty else {
            return;
        };
        let corner = self.tile_corner(ty, self.buildings[b].pos);
        // **The re-target's modulus is the type's own footprint**, and the
        // decompiler lost it because it travels in `ecx`: `005efff9` loads
        // the crop's from `ObjectType::x_size` (`+0x234`) and `005f0004`
        // its `y_size` (`+0x238`); the pasture's two, at `005efdd8` and
        // `005efde5`, take half of each. Both are 4 for the farm, which is
        // what run13 measured before the register was read — all six
        // farmers' twelve draws on sim-frame 101 land on the dump's new
        // tiles under `% 4` and under no other modulus (`docs/SYNC.md` §4).
        let (xs, ys) = (self.build_types[ty].x_size, self.build_types[ty].y_size);
        // **The pasture arm, `005efd77`.** `FarmsData::get_farm_type`
        // answers `farm_type & 1`, and a `1` takes the whole of the
        // function before the cell arithmetic below has run. A herder sows
        // nothing: it shows the sow animation, and on its own 256-frame
        // phase it walks to a tile of the **inner** 2 × 2 —
        // `corner + 1 + rnd(size / 2)` an axis, against the crop's
        // `corner + rnd(size)` over all sixteen. `Farms::inc_time` skips a
        // pasture, so no cell under it ever ripens on the clock, and the
        // switch below would instead send the herder walking on the
        // farmer's own 201st add — a hundred frames late, and on a frame
        // the original spends nothing (`docs/SYNC.md` §3.15).
        if self.buildings[b].farm.farm_type & farms::ANIMAL_FARM != 0 {
            self.set_anim(u, crate::anim::SOW, false, true);
            let phase = i64::from(self.units[u].index) * PASTURE_STRIDE
                + frame
                + i64::from(self.units[u].owner);
            if phase % PASTURE_PHASE != 0 {
                return;
            }
            self.mark(SITE_FARM_CELL);
            let rx = self.rnd(xs / 2);
            let ry = self.rnd(ys / 2);
            let dest = Pos::new(
                (corner.x + 1 + rx) * TILE + HALF_TILE,
                (corner.y + 1 + ry) * TILE + HALF_TILE,
            );
            self.add_move_order(u, dest, MoveKind::MoveTo, QueuePos::First, false);
            return;
        }
        let here = self.units[u].pos.tile();
        let (mut dx, mut dy) = (here.x - corner.x, here.y - corner.y);
        if dx < 0 || dy < 0 || dx >= xs || dy >= ys {
            dx = 1;
            dy = 1;
        }
        // **`dx * 4 + dy`, not `dy * 4 + dx`.** `FarmStruct::status` is a
        // `uchar[4][4]` and the farmer's cell is `status[dx][dy]`: both
        // `do_gather`'s own switch (`005eff54`, `(dx + farm·0x30)·4 + 0xac +
        // dy`) and `Farms::grow@008d91c0`'s write (`(farm·0x30 + param_3)·4
        // + 0xac + param_2`, called `(dy, dx)`) land on the same byte, and
        // it is the transpose of what this had. Invisible for a hundred
        // frames — every starting farmer stands on `(2, 2)`, which is its
        // own transpose — and then wrong the moment one re-picks a cell
        // (`docs/ORDERS.md` §6.5).
        let idx = (dx * 4 + dy) as usize;
        // **`guy[0].cur_anim`, live — not a flag the farm branch keeps.**
        // The switch reads `*(char *)(**(int **)&this->field_0xf4 + 0x9c)`
        // (`do_gather@005ef2a0:454`, `GuyData +0x9c cur_anim`), so anything
        // that plays an animation between two farm frames — a walk, a
        // blocked stand, an idle — is what the test sees. A sticky byte
        // written only here and cleared only by a step is not the same
        // thing (`docs/ORDERS.md` §6.5).
        let anim = self.units[u]
            .guys
            .first()
            .map_or(crate::anim::DEFAULT, |g| g.anim);
        let state = self.buildings[b].farm.state[idx];
        match state {
            s if s == farms::GROWING || (s == farms::EMPTY && anim != crate::anim::REAP) => {
                self.set_anim(u, crate::anim::SOW, false, true);
                self.buildings[b].farm.grow(idx);
                return;
            }
            farms::RIPE if anim != crate::anim::SOW => {
                self.set_anim(u, crate::anim::REAP, false, true);
                self.buildings[b].farm.snip(idx);
                return;
            }
            farms::CUT => {
                self.set_anim(u, crate::anim::REAP, false, true);
                return;
            }
            // Empty under a reaper, or ripe under a sower: a new tile.
            _ => {}
        }
        self.mark(SITE_FARM_CELL);
        let rx = self.rnd(xs);
        let ry = self.rnd(ys);
        let dest = Pos::new(
            (corner.x + rx) * TILE + HALF_TILE,
            (corner.y + ry) * TILE + HALF_TILE,
        );
        self.add_move_order(u, dest, MoveKind::MoveTo, QueuePos::First, false);
    }

    /// `Unit::find_gather_spot` (§6.6): the best gather building of the
    /// owner's with room, scored on the **headroom under the commerce cap**
    /// of the goods it gathers, divided by the distance.
    pub(crate) fn find_gather_spot(&mut self, u: usize, range: i32) -> bool {
        let who = self.units[u].owner;
        let here = self.units[u].pos;
        let scholar = self.worker_of(u) == Worker::Scholar;
        // A scholar searches without a range limit: `find_gather_spot@
        // 005f5170:44` keeps the caller's range only `if (!bVar2)`, where
        // `bVar2` is the `SCHOLARS`/`SCHOLARSKOREAN` test — otherwise it
        // stays −1. (`docs/audit/2026-08-21-orders.md` R4 G42.)
        let range = if scholar { -1 } else { range };
        let region = self.world.tregion(here.tile());
        // `find_city_at(x, y, who, −1, 0)` on the citizen's own position: the
        // city it counts as standing in, and the input to the crossing rule.
        let my_city = self.find_city_at(who, here, None);
        // `local_20 = 0` and a strict `<`: the first maximum wins, and a
        // building whose score is not **above** zero is never taken.
        let mut best: Option<(i32, usize)> = None;
        let mut best_score = 0;
        for b in 0..self.buildings.len() {
            let bd = &self.buildings[b];
            if !bd.alive || !bd.active || bd.owner != who || !self.is_gather_type(b) {
                continue;
            }
            if (self.building_ident(b) == Ident::University) != scholar {
                continue;
            }
            if !self.gather_room(b) && !self.is_gathering_at(u, b, false) {
                continue;
            }
            if self.world.tregion(self.buildings[b].pos.tile()) != region {
                continue;
            }
            if !scholar
                && let Some(mine) = my_city
                && !self.crosses_to(who, mine, self.buildings[b].city)
            {
                continue;
            }
            let dist = vector_dist(
                self.buildings[b].pos.x - here.x,
                self.buildings[b].pos.y - here.y,
            );
            if range > 0 && dist > range {
                continue;
            }
            let score = self.gather_spot_value(who, b) * 500 / (dist / TILE + 2);
            if score > best_score {
                best_score = score;
                best = Some((score, b));
            }
        }
        let Some((_, b)) = best else {
            return false;
        };
        if !self.is_gathering_at(u, b, false) {
            self.add_gather_order(u, b, QueuePos::Last, false);
        }
        true
    }

    /// `find_gather_spot@005f5170:108` — whether a citizen standing in city
    /// `mine` will walk to a building belonging to city `theirs`.
    ///
    /// It always will inside its own city, and never when its own city holds
    /// fewer than two citizens (`CityData +0x5a free` plus `+0x5c gatherers`,
    /// the census counters). Otherwise it crosses only to a building of no
    /// city at all, or to a city that is more than two citizens *less*
    /// crowded than its own.
    fn crosses_to(&self, who: Player, mine: usize, theirs: Option<usize>) -> bool {
        if theirs == Some(mine) {
            return true;
        }
        // `city_ai` only exists as far as the last census reached, and a
        // human leader never runs one — so an absent record is zero, which
        // is what the original's `CityData` holds for the same reason.
        let pop = |c: usize| {
            self.ai[who as usize]
                .city_ai
                .get(c)
                .map_or(0, |r| i32::from(r.free) + r.gatherers)
        };
        let mypop = pop(mine);
        if mypop < 2 {
            return false;
        }
        match theirs {
            Some(c) => mypop > pop(c) + 2,
            None => true,
        }
    }

    /// The numerator of `find_gather_spot`'s score: over the six goods, the
    /// ones this leader has and this building gathers, each contributing the
    /// **unused** part of its commerce cap — `resource_cap[g] − income[g]`,
    /// both in sixteenths — and skipped entirely while the good is already
    /// over its cap. `has_tribe_bonus(0x16)` — the Dutch interest — adds
    /// [`DUTCH_INTEREST_HEADROOM`] to every good but knowledge, because
    /// `Leader::do_gather` lets that bonus carry income that far above the
    /// commerce cap.
    ///
    /// This is what sends a citizen to a farm rather than to a nearer
    /// woodcutter's camp once the camp's timber is closer to the ceiling.
    fn gather_spot_value(&self, who: Player, b: usize) -> i32 {
        let Some(good) = crate::ai_place::gather_good(self.building_ident(b)) else {
            return 0;
        };
        let w = who as usize;
        if !self.holdings[w].available[good] {
            return 0;
        }
        let ledger = &self.ledgers[w];
        if ledger.over_cap[good] != economy::OverCap::Under {
            return 0;
        }
        let mut v = ledger.cap[good] - ledger.income[good];
        if good != economy::Resource::Knowledge.index()
            && self
                .tech_tree
                .has_tribe_bonus(&self.setup, &self.tech[w], 0x16)
        {
            v += DUTCH_INTEREST_HEADROOM;
        }
        v
    }

    // ------------------------------------------------------------------
    // The attack order — §7.2
    // ------------------------------------------------------------------

    /// `Unit::do_attack` → `fight`'s order-management half, on the existing
    /// attack step: the target check, the reload gate with `new_ord`, the
    /// 1/5 re-search, range → strike, out of range → the chase inserted in
    /// front.
    fn do_attack(&mut self, u: usize, frame: i64) {
        let Some(Order {
            body: Body::Attack(mut a),
            flags,
        }) = self.current_order(u).copied()
        else {
            return;
        };
        let me = Obj::Unit(u);
        let Some(target) = self.units[u].combat.target else {
            self.kill_current_order(u);
            return;
        };
        if self.attack_of(me) == 0 {
            self.kill_current_order(u);
            return;
        }
        let state = self.units[u].combat;
        if !self.valid_target(me, target) || flags & flag::RETARGET != 0 {
            // `find_new_target`: the idle search with the order dropped.
            self.kill_current_order(u);
            if let Some(t) = self.find_melee_target(u, -1) {
                self.add_attack_order(u, t, QueuePos::First, false, false);
            }
            return;
        }
        if state.stance == combat::Stance::HoldFire {
            return;
        }
        // The reload gate: a recharging unit returns at once unless this is
        // the first frame of a fresh order, which turns and then returns.
        if state.recharging != 0 {
            if a.new_ord {
                let (from, to) = (self.units[u].pos, self.pos_of(target));
                self.units[u]
                    .movement
                    .set_heading(find_angle(to.x - from.x, to.y - from.y));
            }
            return;
        }
        // The one-in-five re-search (`docs/COMBAT.md` §8.2 step 0).
        if !state.mandatory
            && state.captain == i32::from(self.units[u].index)
            && let Obj::Unit(t) = target
        {
            let roll = self.rng.roll();
            if !self.profile(Obj::Unit(t)).combat_role
                && roll % 5 != 0
                && let Some(f) = self.find_melee_target(u, -1)
                && f != target
            {
                self.retarget_attack(u, f);
                return;
            }
        }
        let in_range = self.is_in_range(me, target);
        a.in_range = in_range;
        if in_range {
            a.ever_in_range = true;
            a.new_ord = false;
            self.store_attack(u, a);
            self.fight_pub(u, target, frame);
            return;
        }
        self.store_attack(u, a);
        if state.stance == combat::Stance::StandGround {
            return;
        }
        // The chase: `find_attack_pos` is the target's position here; the
        // move goes in front and runs from the next frame.
        let dest = self.pos_of(target);
        self.add_move_order(u, dest, MoveKind::MoveTo, QueuePos::First, false);
    }

    fn store_attack(&mut self, u: usize, a: AttackOrder) {
        if let Some(front) = self.units[u].orders.front_mut()
            && let Body::Attack(x) = &mut front.body
        {
            *x = a;
        }
    }

    /// A found better target rewrites the order's target in place.
    fn retarget_attack(&mut self, u: usize, t: Obj) {
        self.bump_targeted_pub(t, 1);
        let unit = &mut self.units[u];
        unit.combat.target = Some(t);
        unit.combat.mandatory = false;
    }
}
