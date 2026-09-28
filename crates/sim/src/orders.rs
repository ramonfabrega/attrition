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

use crate::ai_load::uflags2;
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
    /// `FollowOrder` — [`super::Body::Follow`], issued by
    /// `Group::action_follow@006fd510` from a player's follow command
    /// (`docs/ORDERS.md` §28).
    pub const FOLLOW: u8 = 11;
    /// `GuardOrder` — [`super::Body::Guard`], issued by
    /// `Group::action_guard` (`docs/ORDERS.md` §7.5, §24) and read by
    /// `UnitData::get_speed`'s order scale (`docs/MOVEMENT.md`, "The
    /// effective speed here, now").
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
    /// `GroupPatrolOrder` — [`super::Body::Patrol`]. `PATROL` (5) is a
    /// class that is never constructed alone: `Unit::add_patrol_order@
    /// 005e4560` asks `OrdersMemManager::get_obj` for `GROUP_PATROL` and
    /// nothing else (`docs/ORDERS.md` §7.7, §27).
    pub const GROUP_PATROL: u8 = 22;
    /// `AttackGroundOrder` — [`super::Body::AttackGround`], which this
    /// crate issues from one place: `Unit::fight`'s siege arm, an
    /// unpacked packer's shot at a unit (`docs/COMBAT.md` §57).
    pub const ATTACK_GROUND: u8 = 23;
    /// `StrafeOrder` — [`super::Body::Strafe`], the one class
    /// `CommandManager::issue_flight@00941d40` makes, through
    /// `Unit::add_strafe_order@005e48c0` (`docs/ORDERS.md` §32). `AirOrder`
    /// has no index of its own: it is this class's second base.
    pub const STRAFE: u8 = 16;
    /// `AirPatrolOrder` — [`super::Body::AirPatrol`], which this crate
    /// makes in one place: `Unit::do_strafe`'s arm for a strike whose
    /// target it may not take, `add_air_patrol_order` over the strike's
    /// point (`docs/ORDERS.md` §34).
    pub const AIR_PATROL: u8 = 17;

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
                | GUARD
                | REPAIR
                | CAST_SPELL
                | TRADE_ROUTE
                | GROUP_MOVE
                | GROUP_ATTACK_TO
                | GROUP_PATROL
                | ATTACK_GROUND
                | STRAFE
                | AIR_PATROL
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

    /// The Spy's three crafts (`FROM Spy`), each targeted: `BRIBE` (`fcbhm`,
    /// `MANA 1000`), `COUNTERINTEL` (`febchm`, `MANA 500`; the one a human
    /// Spy's `think_spellcaster` casts by itself) and `INFORMER` (`fbcml`,
    /// `MANA 500`), which `SpellType::cast` hands to `cast_double_agent`
    /// and `GroupOut::validate_spell` names `DOUBLE_AGENT`
    /// (`docs/GOLDEN.md` §27).
    pub const BRIBE: i32 = 0x275;
    pub const COUNTERINTEL: i32 = 0x277;
    pub const INFORMER: i32 = 0x27f;

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
    /// `SpellTypeData::spell_range`, **in internal units**: the row's
    /// `SPELL_RANGE` tiles × 192, which is what `get_range@00676a80`
    /// hands back before its craft arms (the Informer's 10 is 1,920, and
    /// run246's packet answers 960 for it on a building, the halving).
    pub range: i32,
    /// `SpellTypeData::mana` (`+0x1d0`): what `pay_cast_costs` adds to the
    /// caster's `mana_burn`, and what `action_spell` asks it to have left.
    pub mana: i32,
    /// `FROM`/`FROM2`, the caster lineages `is_castable`'s head tests.
    pub from: [Option<crate::tech::TypeId>; 2],
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
    /// **`Unit::fight`'s re-entrancy latch**, on the *action* order —
    /// `005fe3b7`-`005fe3e7`: with the bit clear the chase tail sets it
    /// and calls `Unit::work` again (vtable `+0x188`) so the move it just
    /// pushed runs in the same frame; a nested `fight` finds it set,
    /// clears it and returns without recursing (`docs/ORDERS.md` §7.10).
    ///
    /// It was `RETARGET` until item 329, read as "re-target requested"
    /// and never written by anything, so the one arm that read it could
    /// not fire. `Unit::do_attack@005f1b80` reads no order flag at all;
    /// where `fight` reads this one on the *current* order it **narrows**
    /// the search — `005fde93`'s one-in-five suppression.
    pub const FIGHT_REENTRY: u8 = 0x10;
    /// **An attack-ground order has fired** (`docs/ORDERS.md` §1.3).
    /// `Unit::do_attack_ground@005f1410` sets it on the shot, and while
    /// the reload runs a set bit holds the unit still rather than
    /// letting it roll the idle; `add_attack_ground_order` and `fight`'s
    /// siege arm clear it on a new order. The dump prints it as the
    /// signed byte `flags -128`.
    pub const FIRED: u8 = 0x80;
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
    /// **`MoveOrder +0x1c retry`** — the type record's own name, and not
    /// `pause` beside it: the frames a move sits still after a **failed
    /// unit-grid search** (`crate::path`, `docs/PATHFINDER.md` §21).
    /// `do_move@005f7b30:348` counts it down and does nothing else on
    /// every frame it is non-zero; the frame it reaches zero,
    /// [`MoveOrder::attempts`] gains 3.
    pub retry: i32,
    /// **`MoveOrder +0x20 attempts`** — the counter that decides whether a
    /// *further* failure is allowed to buy another [`MoveOrder::retry`].
    /// `do_move` decays it by one on every frame that reaches the planner,
    /// a spent `retry` adds 3, and `astar_path`'s open-list-exhausted tail
    /// rolls a new delay only while it is **under 13**
    /// (`astar_path@00683770:949`). The work-cap tail has no such gate.
    pub attempts: i32,
    pub timer: i32,
    /// `MoveOrder +0x3c/+0x40 coll_x/coll_y` — the point the last
    /// collision refused, which `resolve_unit_collision` sidesteps from
    /// (`docs/COLLISION.md` §4.3, §6 step 4). The dump prints the pair.
    pub coll: Option<Pos>,
    /// **`MoveOrder +0x44`/`+0x48 orig_x`/`orig_y`** on a plain move — the
    /// point `Group::action_move_near@00704990` was handed, the click,
    /// which `add_move_facing_order@005e55c0` stores beside the member's
    /// snapped slot; `None` for the −1, −1 of `Unit::add_move_order`.
    /// `Group::finish_insert@0070e620`'s cases 1–4 replay a copied move
    /// **to this point**, not to its slot (`docs/GOLDEN.md` §24, run219:
    /// an explorer's walk behind its goody-box leg). A group move's is
    /// [`GroupMove::orig`].
    pub orig: Option<Pos>,
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
    /// **`MoveOrder +0x44`/`+0x48 orig_x`/`orig_y`** — the group's own
    /// point, the one `action_move_near` was handed, which
    /// `Unit::add_group_move_order@005e4710` stores beside the member's
    /// slot. Carried on the group half because the one reader this crate
    /// has is a group's: `Group::finish_insert@0070e620`'s case `0x13`
    /// replays a copied group move **to `orig`**, not to the leader's
    /// slot (`docs/GOLDEN.md` §22, run210's 742).
    pub orig: Pos,
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
    /// route's plan that is not open water, and the road stack carries it
    /// into the unit's own path when a leg starts (`docs/CARAVAN.md` §5.3).
    /// ~~Nothing in the executable reads it back.~~ `Unit::do_move@005f7b30`
    /// does, on the waypoint a caravan takes: a road tile that has been
    /// built over verifies the route (`docs/CARAVAN.md` §10, item 695).
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
/// `uid`, `+0x14`/`+0x18` the ground point — is `(-1, -1, -1, -1)` for
/// every untargeted craft (`set_new_location`, `think_fish`, the unpacks)
/// and a player's pick for a targeted one (`Group::action_spell`,
/// `docs/GOLDEN.md` §27). The uid is the target's own, read when needed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CastOrder {
    /// `+0x20` — the spell's `TypeIndex` (see [`spell`]).
    pub spell: i32,
    /// `+0x1c` — "the cost has been taken", so a cast that waits out a
    /// job time pays once rather than once a frame.
    pub paid: bool,
    /// `+0x8`/`+0xc` — the target object, `None` for `(-1, -1)`.
    pub target: Option<crate::combat::Obj>,
    /// `+0x14`/`+0x18` — the point the pick was made at, `(-1, -1)` for
    /// an untargeted craft.
    pub at: Pos,
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

/// The fields of `GroupPatrolOrder` (`docs/ORDERS.md` §27): the
/// `PATROLORDER` base's two point arrays and `waypoint`, then the
/// `GROUPORDER` row's leader, id and `form_id` — `add_patrol_order`'s
/// `+0x44`, `+0x4c` and `+0x50`.
///
/// SEAM: the arrays are two points. `add_patrol_order` sets both lengths
/// to exactly 2, and only a shift-click's `QUEUE_LAST` append in
/// `Group::action_patrol` (and `redo_patrol_order`'s copy of it) makes a
/// third; no capture issues one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PatrolOrder {
    /// `x_pos[i]`/`y_pos[i]`, each snapped to the 48-unit grid.
    pub points: [Pos; 2],
    /// `+0x3c` — the point the current leg walks to.
    pub waypoint: usize,
    /// `+0x44`/`+0x48` — the leader `Group::action_patrol` found; the one
    /// member whose `do_patrol` issues the legs.
    pub leader: usize,
    /// `+0x4c` — `(group.id + frame × 10) × 100 + order_num`.
    pub id: i64,
    /// `+0x50` — the member's index when `action_patrol` issued it, the
    /// **leader's** once `redo_patrol_order` has rebuilt it.
    pub form_id: usize,
}

/// The fields of `GuardOrder` (`docs/ORDERS.md` §7.5, §24): the
/// `TARGETORDER` base's object, then `dx dy guard_x guard_y idle retry` as
/// the dump prints them.
///
/// The target is a **unit** only. `Group::action_guard`'s building arm is
/// a seam (`crate::group`), so no building is ever guarded here, and the
/// `uid` is the unit's index because this crate never reuses one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FollowOrder {
    /// `+0x8`/`+0xc` — the leader, as the command named it
    /// (`Unit::add_follow_order@005e3f60`; `docs/ORDERS.md` §28). The dump
    /// prints it as `ox/whom/uid`.
    ///
    /// SEAM: `+0x14`/`+0x18`/`+0x1c`, `oxx/whose/uid2`, the leader again
    /// or the container it is inside, and `do_follow`'s swap onto the
    /// container and back. This crate puts no unit inside another that a
    /// follow can name, and the dump does not print the three.
    pub target: usize,
}

/// `GuardOrder` (`docs/ORDERS.md` §7.5, §24).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GuardOrder {
    /// `+0x8`/`+0xc` — the escorted unit, always its squad's captain
    /// (`action_guard` takes `get_captain` of what it was given).
    pub target: usize,
    /// `+0x14`/`+0x18` — the post's offset from the target, in position
    /// units, in the target's own frame: `Form::compute`'s `off_x/off_y`
    /// for this member, or `(-1, -1)` for a player's click.
    pub dx: i32,
    pub dy: i32,
    /// `+0x1c`/`+0x20` — the post, a 48-unit cell centre; the target's
    /// position at creation, then rewritten by every `do_guard` that
    /// reaches the positioning arm.
    pub guard: Pos,
    /// `+0x24` — frames on the post (or sixteenths of frames beside an
    /// idle target); zeroed by every reposition.
    pub idle: i32,
    /// `+0x28` — a countdown during which `do_guard` does nothing:
    /// `Random::get % 3 + 6` after a reposition that ended at once.
    pub retry: i32,
}

/// `AirOrder::cruising_alt` as `Unit::add_strafe_order@005e48c0` writes
/// it (`+0x30`, 0x640): the altitude a flight climbs to before
/// `do_air_physics` redraws it (`docs/ORDERS.md` §32).
pub const CRUISING_ALT: i32 = 0x640;

/// The fields of `StrafeOrder : AttackOrder, AirOrder` (0x54,
/// `docs/ORDERS.md` §1.1, §32), as `Unit::add_strafe_order@005e48c0`
/// writes them and the dump prints them: the `TARGETORDER` row, the
/// `ATTACKORDER`'s `mandatory`, the `AIRORDER` row and the strafe's own
/// `xx yy`. `old` is the constructor's 0 on every order this crate makes,
/// so it is not carried; `sharp_turn` is the flight's edge turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StrafeOrder {
    /// `TargetOrder::ox/whom` (`+0x8/+0xc`): what it strikes, `None` (−1)
    /// for a flight home.
    pub target: Option<Obj>,
    /// `AttackOrder::mandatory` (`+0x1c`).
    pub mandatory: bool,
    /// `AirOrder::oxx/whose` (`+0x28/+0x2c`): the home base.
    pub home: Option<usize>,
    /// `AirOrder::cruising_alt` (`+0x30`), [`CRUISING_ALT`] when added;
    /// a non-bomber redraws it every eighth frame of its flight.
    pub cruising_alt: i32,
    /// `AirOrder::sharp_turn` (`+0x34`): which way the flight is turning
    /// away from the world's edge, `±1`, and 0 when it is not — the
    /// edge coin's, as `crate::air::Flight::turn` is a bird's.
    pub sharp_turn: i32,
    /// `AirOrder::returning` (`+0x3c`): 1 for a flight with no target.
    pub returning: bool,
    /// `StrafeOrder::xx/yy` (`+0x40/+0x44`): the target's point, `None`
    /// (−1) without one.
    pub at: Option<Pos>,
}

/// The fields of `AirPatrolOrder : PatrolOrder, AirOrder` (type 17,
/// `docs/ORDERS.md` §34), as `Unit::add_air_patrol_order@005e4350` writes
/// them and the dump prints them: the `PATROLORDER` base's point arrays
/// and `waypoint`, and the `AIRORDER` row. `old` is the adder's 0 and is
/// not carried.
///
/// **The arrays hold the patrol's points in order** (item 947,
/// `docs/PRODUCTION.md` "The gather point"): `add_air_patrol_order` sets
/// both lengths to 1, and an Airbase's gather list appends the rest
/// (`Build::add_gather_point@00622e70`'s hangar loop, `Build::train@
/// 0062f9b0`'s `CARRY_AIR` arm). `do_air_patrol` flies at
/// `points[waypoint]` and steps the waypoint on within `0x240` of it.
///
/// SEAM: the original's arrays grow without bound; this crate holds
/// [`PATROL_POINTS`] and passes over a point past them.
/// `Group::action_air_patrol`'s `QUEUE_LAST` append is not entered, and a
/// home that is a unit (a carrier) stores the point relative to it, which
/// no capture reaches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AirPatrolOrder {
    /// `x_pos`/`y_pos` (`+0x4..+0x14` and `+0x20..+0x30`): the points
    /// patrolled, the first [`Self::len`] of them live.
    pub points: [Pos; PATROL_POINTS],
    /// `x_pos.length` (`+0x8`), which `y_pos.length` (`+0x24`) equals on
    /// every path this crate takes.
    pub len: u8,
    /// `PatrolOrder::waypoint` (`+0x3c`).
    pub waypoint: usize,
    /// `AirOrder::oxx/whose` (`+0x44/+0x48`): the home base.
    pub home: Option<usize>,
    /// `AirOrder::cruising_alt` (`+0x4c`), [`CRUISING_ALT`] when added.
    pub cruising_alt: i32,
    /// `AirOrder::sharp_turn` (`+0x50`), the edge coin's ±1.
    pub sharp_turn: i32,
    /// `AirOrder::returning` (`+0x58`): 0 when added; `check_fuel` sets it
    /// on an empty tank, which this crate does not carry (§32 piece 4).
    pub returning: bool,
}

/// The points an [`AirPatrolOrder`] carries here (its SEAM).
pub const PATROL_POINTS: usize = 8;

impl AirPatrolOrder {
    /// One point: what `add_air_patrol_order` lays.
    pub fn over(point: Pos, home: Option<usize>) -> Self {
        let mut points = [Pos::new(0, 0); PATROL_POINTS];
        points[0] = point;
        AirPatrolOrder {
            points,
            len: 1,
            waypoint: 0,
            home,
            cruising_alt: CRUISING_ALT,
            sharp_turn: 0,
            returning: false,
        }
    }

    /// The live points, in order.
    pub fn live(&self) -> &[Pos] {
        &self.points[..usize::from(self.len)]
    }

    /// `x_pos[waypoint]`/`y_pos[waypoint]`, the point flown at.
    pub fn current(&self) -> Pos {
        self.points[self.waypoint.min(usize::from(self.len).max(1) - 1)]
    }

    /// `x_pos[length − 1]`/`y_pos[length − 1]`: the point a strike's
    /// search is asked round (`do_air_patrol`, `do_strafe`).
    pub fn last(&self) -> Pos {
        self.points[usize::from(self.len).max(1) - 1]
    }

    /// `SimpleArray::make_valid(length)` and the store behind it: the point
    /// appended to both arrays. False past [`PATROL_POINTS`].
    pub fn push(&mut self, p: Pos) -> bool {
        let n = usize::from(self.len);
        if n >= PATROL_POINTS {
            return false;
        }
        self.points[n] = p;
        self.len += 1;
        true
    }
}

/// The fields of `AttackGroundOrder` (`docs/ORDERS.md` §1.2, §26):
/// `+0x4 att_x, +0x8 att_y, +0xc accuracy, +0x10 attack_unit`, as the
/// dump prints them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttackGroundOrder {
    /// `+0x4`/`+0x8` — the point fired at. `fight`'s siege arm writes the
    /// target's **own position** on the frame it pushes the order, not
    /// its cell (run44 `1/6`, `docs/COMBAT.md` §57.1), and nothing
    /// rewrites it: the round released frames later lands here.
    pub at: Pos,
    /// `+0xc` — "the target was at sea" (`domain == 1`), which makes the
    /// round exact: `Ammo::init` takes no scatter for it.
    pub sea: bool,
    /// `+0x10` — the shots left to count: 2 on `fight`'s push, 1 once
    /// `do_attack_ground` has fired, and the ready frame at 1 ends the
    /// order. A player's ground click would carry 0, which this crate has
    /// no command for.
    pub attack_unit: u8,
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
    Guard(GuardOrder),
    Follow(FollowOrder),
    AttackGround(AttackGroundOrder),
    Patrol(PatrolOrder),
    Strafe(StrafeOrder),
    AirPatrol(AirPatrolOrder),
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
            Body::Guard(_) => index::GUARD,
            Body::Follow(_) => index::FOLLOW,
            Body::AttackGround(_) => index::ATTACK_GROUND,
            Body::Patrol(_) => index::GROUP_PATROL,
            Body::Strafe(_) => index::STRAFE,
            Body::AirPatrol(_) => index::AIR_PATROL,
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
/// `TypeIndex` `MERCHANTDUTCH` — `think_attack`'s packed arm skips its
/// lineage (`is(0x3e, 1)`).
const MERCHANTDUTCH: crate::tech::TypeId = 0x3e;
/// `TypeIndex` `MACHINEGUN` — the lineage whose packed arm unpacks at
/// `idle 3` and whose unpack `add_cast_order` re-aims at `0x28e`.
const MACHINEGUN: crate::tech::TypeId = 0x7b;
/// `PackerStanceIndex::PACKER_AUTO` (the PDB): a packer that unpacks by
/// itself once idle.
const PACKER_AUTO: u8 = 0;

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

/// `Unit::do_guard@005e5c70+0x8fb` — the `retry` roll after a reposition
/// whose leg ended on the frame it was issued (`docs/ORDERS.md` §24).
pub const SITE_GUARD_RETRY: &str = "Unit::do_guard+0x8fb";

/// `do_guard`'s three stands, each `set_anim(CHAR_DEFAULT, 0, 1)` through
/// `Unit::set_anim`, one draw a figure, named by their return addresses in
/// the listing (item 569): `5e6464` the **idle stand on the post**,
/// `5e6550` the stand before the `retry` roll when a reposition leg ended
/// the frame it was issued, and `5e6596` the stand before a dead target's
/// order is killed. Item 567 built all three unmarked, and golden chapter
/// four's escort spends the first from 1464.
pub const SITE_GUARD_IDLE: &str = "Guy::set_anim+0x97a < Unit::do_guard+0x7f4";
pub const SITE_GUARD_STAND: &str = "Guy::set_anim+0x97a < Unit::do_guard+0x8e0";
pub const SITE_GUARD_DEAD: &str = "Guy::set_anim+0x97a < Unit::do_guard+0x926";

/// `Unit::do_follow@005e65d0`'s stand within the standoff,
/// `set_anim(CHAR_DEFAULT, 0, 1)` at `5e68f5`, named by its return
/// address (`docs/ORDERS.md` §28).
pub const SITE_FOLLOW_STAND: &str = "Guy::set_anim+0x97a < Unit::do_follow+0x32a";

/// The 31 bearings of one ring of `find_nearby_spot`, as multiples of a
/// sixteenth of a turn from the base angle; `|k| >= 8` adds a thirty-second.
/// The tile bits a warship's spot may not carry
/// (`find_nearby_spot@0061de70`, `uVar7 & 0x2400`): [`tile::BAD_PATH`]
/// and the `0x400` this crate has no name for. `UnitData::invalid_loc`
/// answers 3 on the same pair (`docs/ORDERS.md` §10).
pub const WARSHIP_REFUSES: u16 = tile::BAD_PATH | 0x400;

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
    /// as a stated seam — its `FILTER_ALL` general path where a call site
    /// here has not yet asked for [`Coll::All`] (`come_out`'s
    /// `block_radius == 0` arm, which no capture's trained type reaches).
    None,
    /// `FILTER_ALL` with the unit's own `(o, who)`: the **general** path
    /// (`61e37e` and `61e39c`, `docs/COLLISION.md` §5.2.1) —
    /// `find_unit_with_radius` over every player's units at `big_radius +
    /// r_coll`, `r_coll` the type's `block_radius` (`0x180` when that is
    /// 0), then its ordered twin, since `not_who >= 0`. `come_out`'s
    /// re-seat at a building point is the call site (`619360`,
    /// `docs/PRODUCTION.md` "The gather point").
    ///
    /// **The seeker counts** (item 955, run338): `FILTER_ALL` skips
    /// `Search::valid_filter` (`659a1a`), the only reader of `(not_o,
    /// not_who)`, and the search argument is never read
    /// (`Search::valid_search(·, 0, …)` answers 1, `6599d4`), so every
    /// player's live, on-map unit is a candidate, the one asking too.
    /// run338's three Citizens, each re-seated with its first candidate
    /// the point it stands on, stand one candidate on (`docs/GOLDEN.md`
    /// §40). The ordered twin keeps the exemption: the seeker has no order
    /// when a re-seat asks.
    All,
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

/// A waypoint `do_move`'s tile arm unwinds before `find_tpath`: neither
/// final nor `0x20`, with a tolerance of **1 to `0x60`**, unsigned —
/// `lea eax,[ecx-1]; cmp eax,0x5f; ja` at `0x5f8ad2`. A tolerance-0
/// waypoint is an exact point and is kept: a group move's formation
/// waypoint is one, and popping it re-aims the tile search at the final
/// goal (`docs/ORDERS.md` §4.4, item 669).
fn is_loose(t: &PathData) -> bool {
    t.flags & (path_flag::FINAL | 0x20) == 0 && (t.tolerance as u32).wrapping_sub(1) <= 0x5f
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
    ///
    /// `flag` is the original's third argument and it selects **one**
    /// thing: the group cap at the end. `do_move`, `find_path`,
    /// `Object::poor_target` and `do_group_move`'s cross-formation arm
    /// pass 0 and are capped; `GuyData::get_speed`, the animal's air
    /// step and `do_group_move`'s in-formation arm pass 1 and are not.
    pub fn get_speed(&self, u: usize, flag: i32) -> i32 {
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
        // **The group cap**, `00608789`-`006087a7`: with flag 0, a unit
        // that is in a group and has **no action order** — `get_action`
        // answering null, which is `local_8 == 0` and the same value the
        // order scale above keyed on — walks no faster than its group's
        // own speed (`docs/GROUPS.md` §14). `action` is `index::NONE`
        // exactly when `get_action` is null, since no order has index 0.
        if flag == 0 && action == index::NONE {
            speed = movement::group_capped(speed, self.group_speed_of(u));
        }
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

    pub(crate) fn enqueue(&mut self, u: usize, order: Order, pos: QueuePos) {
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
        // `kill_current_order@005e2cb0:30`: `unit_masks &= ~0x20000`, a
        // targeted cast's "started" bit, for whatever order dies.
        self.units[u].casting = false;
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
        self.add_move_facing_order_grouped(
            u, to, kind, pos, action, angle, facing, pathed, None, None,
        );
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
        orig: Option<Pos>,
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
                retry: 0,
                attempts: 0,
                timer: 0,
                coll: None,
                orig,
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
        // **No `targeted` bump.** `Unit::add_attack_order` never writes
        // `ObjectData +0x3d`; only `Object::find_nearby_target` does, on
        // the winner it returns (`docs/COMBAT.md` §33). Bumping here
        // counted a squad's mirror copies as separate attackers.
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

    /// `Unit::add_strafe_order(ox, whom, home_o, home_who, mandatory,
    /// queue, action)@005e48c0` (`docs/ORDERS.md` §32): a `StrafeOrder`
    /// with the target, its point in `xx/yy`, `returning` set exactly when
    /// there is no target, the home base, `cruising_alt` 0x640, and the
    /// action bit as asked. The `QUEUE_NEW` head and the `QUEUE_FIRST`
    /// rotation are [`Self::enqueue`]'s, as for `add_guard_order`.
    ///
    /// SEAM: the missile arm at its head — a type with `unit_flags &
    /// 0x8000000` and a valid target becomes `add_air_attack_ground_order`
    /// at the target's point — is not taken: no missile reaches a flight
    /// here.
    pub fn add_strafe_order(
        &mut self,
        u: usize,
        target: Option<Obj>,
        home: Option<usize>,
        mandatory: bool,
        pos: QueuePos,
        action: bool,
    ) {
        let at = target.map(|t| self.pos_of(t));
        let order = Order {
            flags: if action { flag::ACTION } else { 0 },
            body: Body::Strafe(StrafeOrder {
                target,
                mandatory,
                home,
                cruising_alt: CRUISING_ALT,
                sharp_turn: 0,
                returning: target.is_none(),
                at,
            }),
        };
        // **`QUEUE_FIRST` takes no `update_action`** (`0x5e4a2b`): the
        // adder's tail is `clear_partial_path` and the list's rotate, so
        // `orders_x/orders_y` and `dest_angle` keep the frame's own —
        // run223's 777, where the patrol's search pushes the strike after
        // the step and the dump still names the point before it
        // (`docs/ORDERS.md` §34.5). The generic enqueue's closing
        // `update_action` is the other adders'.
        if pos == QueuePos::First {
            self.clear_partial_path(u);
            self.units[u].orders.push_front(order);
            return;
        }
        self.enqueue(u, order, pos);
    }

    /// **`Unit::add_air_patrol_order(x, y, home_o, home_who, action,
    /// queue)@005e4350`** (`docs/ORDERS.md` §34.1): for a plane, the old
    /// orders go (`field_0xc0 = 0`, `close_orders`, `clear_partial_path`,
    /// `update_action`) and one `AirPatrolOrder` is appended over the
    /// point — `waypoint 0`, the home, `cruising_alt` 0x640, `sharp_turn`
    /// and `returning` 0, the action bit as asked. The queue argument is
    /// not read on this arm.
    ///
    /// SEAM: a helicopter (`unit_flags & 0x20`) is given a move instead,
    /// and a home that is a unit stores the point relative to it; neither
    /// is reached.
    pub(crate) fn add_air_patrol_order(
        &mut self,
        u: usize,
        point: Pos,
        home: Option<usize>,
        action: bool,
    ) {
        self.units[u].path.clear();
        self.close_orders(u);
        self.clear_partial_path(u);
        self.update_action(u);
        self.units[u].orders.push_back(Order {
            flags: if action { flag::ACTION } else { 0 },
            body: Body::AirPatrol(AirPatrolOrder::over(point, home)),
        });
    }

    /// `Unit::do_strafe@005eab00`, the air order a flight is
    /// (`docs/ORDERS.md` §33, §34).
    ///
    /// **A flight home** (`returning 1`, `+0x3c`) goes straight to
    /// `do_air_physics(order, −1, −1)` (`0x5eb0ae`) — [`crate::air`]'s
    /// plane arm — and, when that returns 1, a non-bomber with no live
    /// target sets `CHAR_WALK` a second time (`0x5eb0cd`), and the order
    /// count test at `0x5eb62b` returns for a single order.
    ///
    /// **A strike** (`returning 0`, §34.2) first asks `valid_target` of
    /// its target. One it may not take — dead, or unseen, as run223's
    /// Barracks is on 665 — ends the strike: with another order behind it
    /// the strafe is killed and `work` runs again; alone, it becomes an
    /// `AirPatrolOrder` over the strike's point (`xx/yy`), with no action
    /// bit (the `UnitOrder` slot `+0x2c` it passes is `xor eax, eax`), and
    /// `work` runs the patrol in the same frame. One it may take is flown
    /// at, and bombed once in range and within 15° of the heading
    /// ([`Sim::strafe_attack`]).
    ///
    /// SEAM: a target that is an ally (the escort's sixteen-frame
    /// re-target), a missile, a helicopter, a flying target's lead point,
    /// and a strike with no point left (`is_valid(xx, yy)` false turns it
    /// for home).
    fn do_strafe(&mut self, u: usize, frame: i64) {
        let Some(Body::Strafe(sf)) = self.current_order(u).map(|o| o.body) else {
            return;
        };
        let mut goal = None;
        if !sf.returning {
            let Some(target) = sf.target else {
                return;
            };
            let me = crate::combat::Obj::Unit(u);
            let whom = self.owner_of(target);
            if whom < crate::world::PLAYER_SLOTS && self.is_ally(self.units[u].owner, whom) {
                return;
            }
            if !self.valid_target(me, target) {
                if self.units[u].orders.len() > 1 {
                    self.kill_current_order(u);
                    self.set_anim(u, crate::anim::WALK, true, true);
                    self.work(u, frame);
                    return;
                }
                let Some(at) = sf.at else {
                    return;
                };
                self.kill_current_order(u);
                self.add_air_patrol_order(u, at, sf.home, false);
                self.work(u, frame);
                return;
            }
            goal = Some(self.pos_of(target));
        }
        if self.plane_air_physics(u, goal, frame) == crate::air::Flew::Done {
            return;
        }
        let target = sf.target;
        let me = crate::combat::Obj::Unit(u);
        let live = target.filter(|&t| self.valid_target(me, t));
        match live {
            Some(t) if self.units[u].combat.recharging == 0 => {
                if !self.strafe_attack(u, t, sf) {
                    return;
                }
            }
            _ => {
                if let Some(t) = target
                    && self.strafe_on_line(u, t)
                {
                    // `0x5eb62b` with the heading on the target.
                } else if !self.is_bomber(u) {
                    self.set_anim(u, crate::anim::WALK, false, true);
                }
            }
        }
        self.strafe_retarget(u, frame);
    }

    /// `do_strafe`'s arm for a strike it may take and is not reloading
    /// (`0x5eb0f4`–`0x5eb62b`, `docs/ORDERS.md` §34.3). Returns false when
    /// the strike was killed and the frame is over.
    ///
    /// - **The leash.** A strike that is not `mandatory`, on a target that
    ///   does not fly, with an `AirPatrolOrder` behind it dies when the
    ///   target is more than `AIRCRAFT_RESPOND_RANGE × 0x100` from the
    ///   patrol's last point — the *target's* distance (`0x5eb22a`).
    /// - **The release.** In range (`ObjectData::is_in_range@00648d70`,
    ///   from the plane's own point) and within 15° of the heading — 60°
    ///   for the `0x127` line — a Bomber plays `CHAR_ATTACK2` (the draw
    ///   at `do_strafe+0x9d0`, chapter seventeen's 805) and reloads
    ///   `recharge() + 1`.
    ///
    /// SEAM: the other order behind a strike (`+0x100`'s arm), the
    /// `0x400000` type arm, a non-bomber's `fire_ammo`, the missile's
    /// death, and `BOMBING_MANA_COST` (0 in the shipped rules, and the
    /// tank is not carried).
    fn strafe_attack(&mut self, u: usize, t: crate::combat::Obj, sf: StrafeOrder) -> bool {
        let air_target = matches!(self.profile(t).domain, crate::attrition::Domain::Air);
        if !air_target
            && self.units[u].orders.len() > 1
            && !sf.mandatory
            && let Some(Body::AirPatrol(p)) = self.units[u].orders.get(1).map(|o| o.body)
        {
            let at = self.pos_of(t);
            let last = p.last();
            let d = crate::world::vector_dist(at.x - last.x, at.y - last.y);
            if d > self.tuning.aircraft_respond_range * 0x100 {
                self.kill_current_order(u);
                return false;
            }
        }
        let off = self.strafe_off_line(u, t);
        if !self.is_in_range(crate::combat::Obj::Unit(u), t) {
            return true;
        }
        if off >= 0x0aaa_aaaa && (!self.is_fighter_line(u) || off > 0x2aaa_aaaa) {
            return true;
        }
        self.mark(crate::air::SITE_STRAFE_BOMB);
        self.set_anim(u, crate::anim::ATTACK2, false, true);
        self.units[u].combat.recharging = self.reload_frames(u) + 1;
        true
    }

    /// `fold(heading − find_angle(target − plane))`, the angle
    /// `do_strafe` measures its target off the nose with (`0x5eb3ef`).
    fn strafe_off_line(&self, u: usize, t: crate::combat::Obj) -> u32 {
        let (at, to) = (self.units[u].pos, self.pos_of(t));
        let bearing = crate::movement::find_angle(to.x - at.x, to.y - at.y);
        let d = (self.units[u].movement.heading.0 as u32).wrapping_sub(bearing.0 as u32);
        if d > 0x8000_0000 { !d } else { d }
    }

    /// The reloading arm's test (`0x5eb54c`): the target within 30° of
    /// the heading, 90° for the `0x127` line, skips the `CHAR_WALK`.
    fn strafe_on_line(&self, u: usize, t: crate::combat::Obj) -> bool {
        let lim = if self.is_fighter_line(u) {
            0x4000_0000
        } else {
            0x1555_5555
        };
        self.strafe_off_line(u, t) <= lim
    }

    /// **`do_strafe`'s tail, `0x5eb62b`** (`docs/ORDERS.md` §34.4): a
    /// strike that is not alone and not `mandatory`, on every frame
    /// `(frame + 2·o) & 31 == 0`, asks the order behind it for a target —
    /// an `AirPatrolOrder`'s search at its last point — and is re-pointed
    /// at a valid one, or killed.
    ///
    /// SEAM: the fuel test at its head (`type +0x2ec` and `mana_left`),
    /// and an order behind that is not a patrol (`+0x100`'s arm, which
    /// kills and walks).
    fn strafe_retarget(&mut self, u: usize, frame: i64) {
        if self.units[u].orders.len() < 2 {
            return;
        }
        let Some(Body::Strafe(sf)) = self.current_order(u).map(|o| o.body) else {
            return;
        };
        if sf.mandatory || sf.returning || sf.target.is_none() {
            return;
        }
        if (frame + 2 * i64::from(self.units[u].index)) & 31 != 0 {
            return;
        }
        let Some(Body::AirPatrol(p)) = self.units[u].orders.get(1).map(|o| o.body) else {
            return;
        };
        let found = if self.is_bomber(u) {
            self.find_new_bomber_target(u, p.last())
        } else {
            None
        };
        let me = crate::combat::Obj::Unit(u);
        match found.filter(|&t| self.valid_target(me, t)) {
            Some(t) => {
                // `ox/whom/uid` and `returning 0`; `xx/yy` stand.
                if let Some(Order {
                    body: Body::Strafe(sf),
                    ..
                }) = self.units[u].orders.front_mut()
                {
                    sf.target = Some(t);
                    sf.returning = false;
                }
            }
            None => self.kill_current_order(u),
        }
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
    /// The machine gun's arm (`is(0x7b, 0)` → `0x28e`) is modelled for
    /// the unpack, which `think_attack`'s packed arm reaches. SEAM: the
    /// pack rewrite (`0x28b` → `0x28d`/`0x28f`/`0x291`) has no caller.
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
        // `FISHERMEN` lineage. The machine gun's unpack is reached by
        // `think_attack`'s packed arm ([`Self::think_attack_packed`]);
        // no caller here issues a pack (`0x28b`) at all.
        let id = if spell != crate::fish::UNPACK {
            spell
        } else if self.unit_line_is(u, MACHINEGUN) {
            spell::UNPACK_MACHINEGUN
        } else if self.is_merchant(u) {
            spell::UNPACK_MERCHANT
        } else if self.unit_line_is(u, crate::fish::FISHERMEN) {
            crate::fish::UNPACK_FISHERMEN
        } else {
            spell
        };
        // One field to a line: `rondata::writers`' line parser does not
        // see the shorthand `spell` inside a one-line literal, and this is
        // the field's one writer (item 590).
        let order = Order {
            flags: 0,
            body: Body::Cast(CastOrder {
                spell: id,
                paid: false,
                target: None,
                at: Pos::new(-1, -1),
            }),
        };
        self.enqueue(u, order, pos);
    }

    /// `Unit::add_guard_order(o, who, dx, dy, queue, ·)@005e3e40`
    /// (`docs/ORDERS.md` §7.5, §24): the target, the offset asked, the
    /// post at the target's own position, `idle = retry = 0`, and the
    /// action bit set unconditionally.
    ///
    /// The `QUEUE_NEW` head is the generic one: the original clears
    /// `unit_masks & 0x4000000` (the deferred `EXPLORE_TO` conversion,
    /// which this crate does not carry), zeroes `UnitData +0xc0` — the
    /// path stack's `length`, so the stack is emptied — and then
    /// `close_orders`, `clear_partial_path`, `update_action`, which is
    /// [`Self::enqueue`]'s `New` arm exactly. `QUEUE_FIRST` rotates the
    /// list head onto the new order after a `clear_partial_path`, which is
    /// its `First` arm.
    pub fn add_guard_order(&mut self, u: usize, target: usize, dx: i32, dy: i32, pos: QueuePos) {
        let order = Order {
            flags: flag::ACTION,
            body: Body::Guard(GuardOrder {
                target,
                dx,
                dy,
                guard: self.units[target].pos,
                idle: 0,
                retry: 0,
            }),
        };
        self.enqueue(u, order, pos);
    }

    /// `Unit::add_follow_order(ox, whom, queue, ·)@005e3f60`
    /// (`docs/ORDERS.md` §28): the leader, and the action bit set
    /// unconditionally. The `QUEUE_NEW` head and the `QUEUE_FIRST`
    /// rotation are [`Self::enqueue`]'s, as for the guard.
    pub fn add_follow_order(&mut self, u: usize, target: usize, pos: QueuePos) {
        let order = Order {
            flags: flag::ACTION,
            body: Body::Follow(FollowOrder { target }),
        };
        self.enqueue(u, order, pos);
    }

    /// `Unit::add_patrol_order@005e4560(x1, y1, x2, y2, id, form_id,
    /// leader, who, queue)` (`docs/ORDERS.md` §27): a `GroupPatrolOrder`
    /// with the two points — each passed as `div_3_table[v >> 4]` and
    /// stored `× 0x30 + 0x18`, the 48-unit grid's centre — `waypoint 0`,
    /// and the action bit set unconditionally. The `QUEUE_NEW` head and
    /// the `QUEUE_FIRST` rotation are [`Self::enqueue`]'s, as for the
    /// guard.
    pub fn add_patrol_order(
        &mut self,
        u: usize,
        (from, to): (Pos, Pos),
        id: i64,
        form_id: usize,
        leader: usize,
        pos: QueuePos,
    ) {
        let snap = |v: i32| (v >> 4) / 3 * 0x30 + 0x18;
        let order = Order {
            flags: flag::ACTION,
            body: Body::Patrol(PatrolOrder {
                points: [
                    Pos::new(snap(from.x), snap(from.y)),
                    Pos::new(snap(to.x), snap(to.y)),
                ],
                waypoint: 0,
                leader,
                id,
                form_id,
            }),
        };
        self.enqueue(u, order, pos);
    }

    /// `Unit::update_patrol_order(id)@005e35e0` — the first
    /// `GROUP_PATROL` in the list, front to back, with this `id` (`None`
    /// for any): its position, for the caller to rewrite in place.
    pub fn update_patrol_order(&self, u: usize, id: Option<i64>) -> Option<usize> {
        self.units[u]
            .orders
            .iter()
            .position(|o| matches!(o.body, Body::Patrol(p) if id.is_none_or(|i| p.id == i)))
    }

    /// `Unit::update_guard_order(o, who)@005e3220` — the first `GUARD` in
    /// the list, front to back, whose target is `target`: its position in
    /// the list, for the caller to rewrite in place.
    pub fn update_guard_order(&self, u: usize, target: usize) -> Option<usize> {
        self.units[u]
            .orders
            .iter()
            .position(|o| matches!(o.body, Body::Guard(g) if g.target == target))
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
        // **`ObjectData::visible`'s clock**, and it is the first thing
        // `Unit::work@0060d180:63` does — above every gate below, so a
        // unit that returns early still keeps it
        // (`docs/VISION.md` §7).
        //
        // One frame in 32, phased by `o`, a unit that is **not** carrying
        // the attack latch forgets everyone it has made itself visible to.
        // The latch is dropped on any frame the front order is not
        // `ATTACK`, and the test above reads it *before* that drop — so
        // the frame a unit stops attacking still counts as attacking, and
        // the visibility outlives the order by up to 32 frames.
        if self.units[u].phase(frame).rem_euclid(32) == 0 {
            if !self.units[u].attacking {
                self.units[u].visible = 0;
            }
            // And the in-danger latch beside it, on the same tick and
            // with no latch of its own (`0060d19d`): `unit_masks &= ~4`
            // sits inside the same `if` as the `visible` clear, one line
            // below it. [`sim::Unit::in_danger`].
            self.units[u].in_danger = false;
        }
        if self.order_type(u) != index::ATTACK {
            self.units[u].attacking = false;
        }
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
        // Target liveness: a build, repair, garrison or gather whose target's
        // **slot was reused** dies before its `do_*` ever runs. It is the
        // `uid` test (`Unit::work@0060d180:319`, the action's target uid
        // against the object's `+0x30`), and a closed building keeps its
        // uid until `Objects::find_free` hands its number to the next one.
        // So a dead target alone is not enough: the order stands, the walk
        // under it goes on, and `do_build`/`do_gather`'s own `exists`
        // tests end it on arrival. run157's `1/7` keeps its `BUILDORDER`
        // on the Woodcutter's Camp the script placed and destroyed on 1176
        // to the end of the capture (item 644). This crate's handles are
        // never reused, so "reused" is a later building carrying the same
        // owner and number.
        if let Some(a) = self.action_of(u) {
            let dead = match self.units[u].orders[a].body {
                Body::Build(b) | Body::Repair(b) | Body::Garrison { building: b, .. } => {
                    self.building_slot_reused(b)
                }
                Body::Gather(g) => self.building_slot_reused(g.building),
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
        // `Unit::work@0060d180:440` — **the chase's sixteen-frame review**,
        // and it sits above the dispatch, so a chase it ends is answered by
        // the order underneath in the *same* frame (`docs/COMBAT.md` §36).
        self.check_target_path_review(u, frame);
        // `Unit::work@0060d180:283-313`, after the review and before the
        // dispatch: an action that is an `ATTACK` with a target marks the
        // attacker's squad in danger, and the target's when it is a unit.
        // It runs ahead of the target's `uid` test, so a stale target is
        // marked too (`docs/ORDERS.md` §22).
        //
        // And `update_action` is **unconditional** there (`:283`): every
        // frame's `Unit::work` rewrites `orders_x/y` and `dest_angle`
        // before the dispatch, whatever the order.
        if self
            .update_action(u)
            .is_some_and(|a| self.units[u].orders[a].index() == index::ATTACK)
            && let Some(t) = self.units[u].combat.target
        {
            self.set_in_danger(u);
            if let Obj::Unit(v) = t {
                self.set_in_danger(v);
            }
        }
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
                if m.kind == MoveKind::AttackTo && m.group.is_none() {
                    self.do_attack_to_tail(u, frame, m.dest);
                }
            }
            Some(Body::Trade(_)) => self.do_trade(u),
            Some(Body::Build(_)) => self.do_build(u, frame),
            Some(Body::Repair(_)) => self.do_repair(u, frame),
            Some(Body::Garrison { .. }) => self.do_garrison_order(u),
            Some(Body::Gather(_)) => self.do_gather(u, frame),
            Some(Body::Attack(_)) => self.do_attack(u, frame),
            Some(Body::Cast(c)) => self.do_cast(u, c),
            Some(Body::Guard(_)) => self.do_guard(u, frame),
            Some(Body::Follow(f)) => self.do_follow(u, f, frame),
            Some(Body::AttackGround(_)) => self.do_attack_ground(u, frame),
            Some(Body::Patrol(p)) => self.do_patrol(u, p),
            Some(Body::Strafe(_)) => self.do_strafe(u, frame),
            Some(Body::AirPatrol(_)) => self.do_air_patrol(u, frame),
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

    /// `Unit::do_attack_to@005f2320`'s tail — the attack-move's look
    /// around, after `do_move` (`docs/ORDERS.md` §7.4,
    /// `docs/ORDERS.md` §22).
    ///
    /// **One frame in fifteen, phased by `o`**, while the head is still
    /// this order: an armed unit that is not a supply wagon runs
    /// `find_melee_target(−1, NULL, 0, 1, 0)`, which adds what it finds
    /// `QUEUE_FIRST` above the attack-move. The attack-move resumes when
    /// that attack dies. One gate comes first, read off the listing at
    /// `005f23ca`–`005f243f`. An AI-driven unit (`unit_masks & 0x40000`)
    /// whose army is hurrying (`ArmyData +0x2c`) skips the look when it is
    /// more than six cells from the army's muster, by `vector_dist`.
    ///
    /// `find_melee_target`'s head is the squad's. A **follower** takes
    /// its captain's target when the captain's action is an `ATTACK`
    /// (`005ff9c0:32-91`), and looks for nothing when it is not. The
    /// search is only reached when the captain's target is no longer a
    /// valid one.
    ///
    /// **The unarmed arm is [`Self::do_attack_to_pause`]** (item 569):
    /// the listing's test is the type's attack **and** the raw
    /// `is_supply`, `unit_flags2 & 0x40`, so a Supply Wagon and every
    /// unarmed unit wait for their group instead of looking.
    ///
    /// The search's `flags` word, which `find_melee_target` derives from
    /// the type's vslots `+0x10c`/`+0x110` for an attack-move, is
    /// [`Sim::melee_search_flags`] (item 997, `docs/COMBAT.md` §64): it
    /// passes over an unarmed building.
    ///
    /// SEAM: `find_nearby_target`'s naval refusal (`+0x218 == 2`), and the
    /// siege-on-a-city `mandatory` arm. Neither is carried by
    /// [`Sim::find_nearby_target`].
    pub(crate) fn do_attack_to_tail(&mut self, u: usize, frame: i64, dest: Pos) {
        if (frame + i64::from(self.units[u].index)).rem_euclid(15) != 0 {
            return;
        }
        let same = matches!(
            self.current_order(u).map(|o| o.body),
            Some(Body::Move(m)) if m.kind == MoveKind::AttackTo && m.group.is_none() && m.dest == dest
        );
        if !same {
            return;
        }
        let me = Obj::Unit(u);
        let supply = self.units[u]
            .ty
            .is_some_and(|t| self.unit_types[t].cols.flag2(uflags2::SUPPLY_OR_HERO));
        if self.profile(me).attack == 0 || supply {
            self.do_attack_to_pause(u);
            return;
        }
        let who = self.units[u].owner;
        if self.ai_driven(who)
            && let Some(slot) = self.army_of(u)
            && let Some(a) = self.armies[who as usize].list.get(slot)
            && a.hurry != 0
        {
            let (c, m) = (self.units[u].pos.cell(), a.muster);
            if crate::world::vector_dist((c.x - m.x).abs(), (c.y - m.y).abs()) > 6 {
                return;
            }
        }
        if !self.units[u].captain {
            let cap = self.squad_captain(u);
            let Some(k) = self.action_of(cap) else { return };
            if !matches!(self.units[cap].orders[k].body, Body::Attack(_)) {
                return;
            }
            if let Some(t) = self.units[cap].combat.target
                && self.valid_target(me, t)
                && ((self.units[u].combat.stance != combat::Stance::StandGround
                    && !self.units[u].combat.entrenched
                    && !self.units[cap].combat.entrenched)
                    || self.is_in_range(me, t))
            {
                let mandatory = self.units[cap].combat.mandatory;
                self.add_attack_order(u, t, QueuePos::First, mandatory, false);
                return;
            }
        }
        if let Some(t) = self.find_melee_target(u, -1) {
            self.add_attack_order(u, t, QueuePos::First, false, false);
        }
    }

    /// `Unit::do_attack_to_pause@005f22a0` (`docs/ORDERS.md` §24.9): an
    /// unarmed unit on an attack-move **waits for its group** when the
    /// group is fighting beside it — or, as it is written, when no more
    /// than half of the armed captains near it are still walking.
    ///
    /// The unit's group (`+0x80`) must hold it, active
    /// (`GroupData::member(o, who, 1)@0070f8f0`); then
    /// `Group::is_attacking_near(x, y)@00710e40`, read off the listing at
    /// `710e40`–`711061` because the decompile prints `unaff_EDI/ESI` for
    /// both the `vector_dist` pair and the order type:
    ///
    /// - `normalize` the group first;
    /// - over every member that is active, on the map, **a captain**,
    ///   armed (type `+0x1e8`), and neither `is_supply` (`& 0x40`) nor
    ///   `is_hero` (`& 0x20`) in `unit_flags2`: `near += 1` when
    ///   `vector_dist(|x − ux|, |y − uy|) ≤ 0x600`, and then `attacking
    ///   += 1` when its head order's type is not a move (`is_move@0046f050`:
    ///   1–4, `0x12`, `0x13`, `0x15`), not `NONE` and not `GUARD`;
    /// - yes when `near ≠ 0 && attacking ≥ near / 2`.
    ///
    /// **One armed captain near and none fighting is a yes** — `1 / 2` is
    /// 0 — so a Supply Wagon with its escort's captain inside `0x600`
    /// stops. The order's `pause` becomes 15, and `do_move` stands it out.
    fn do_attack_to_pause(&mut self, u: usize) {
        let Some(seat) = self.seat_of(u) else { return };
        if !self.units[u].alive() || !self.seat_list(seat).contains(&u) {
            return;
        }
        self.seat_normalize(seat);
        let at = self.units[u].pos;
        let (mut near, mut attacking) = (0, 0);
        for m in self.seat_list(seat).clone() {
            let unit = &self.units[m];
            if !unit.alive() || !unit.on_map || !unit.captain {
                continue;
            }
            if self.profile(Obj::Unit(m)).attack == 0 {
                continue;
            }
            let special = unit.ty.is_some_and(|t| {
                let c = self.unit_types[t].cols;
                c.flag2(uflags2::SUPPLY_OR_HERO) || c.flag2(uflags2::GENERAL)
            });
            if special {
                continue;
            }
            let (dx, dy) = ((at.x - unit.pos.x).abs(), (at.y - unit.pos.y).abs());
            if vector_dist(dx, dy) > 0x600 {
                continue;
            }
            near += 1;
            let t = self.current_order(m).map_or(index::NONE, Order::index);
            let moving = matches!(t, 1..=4 | 0x12 | 0x13 | 0x15);
            if !moving && t != index::NONE && t != index::GUARD {
                attacking += 1;
            }
        }
        if near != 0
            && attacking >= near / 2
            && let Some(front) = self.units[u].orders.front_mut()
            && let Some(m) = front.move_mut()
        {
            m.pause = 15;
        }
    }

    /// `Unit::work@0060d180:440`'s gate on [`Self::check_target_path`] —
    /// **one frame in sixteen, phased by `o`** (`docs/COMBAT.md` §36).
    ///
    /// It runs for a head order of the **move family** only, and only when
    /// the order underneath it is an action the review knows how to ask
    /// about. `Unit::work` re-reads the head afterwards
    /// (`0060d710: update_order`), which is what this crate's dispatch
    /// `match` does anyway by reading `current_order` after the call — so a
    /// chase ended here is answered by `do_attack` on the same frame, and
    /// that is exactly what separates it from `do_move`'s own kill
    /// (§36.2).
    ///
    /// **The `GUARD` arm** (`action type == 0xc`, `0060d4a0`–`0060d4d3`)
    /// is the review's other branch, and it asks no question of the
    /// target at all: on the sixty-four-frame phase `(frame + o) % 64 ==
    /// 0` it `repath`s — the reposition leg goes — and jumps to the head
    /// re-read, so `do_guard` answers on the same frame with a fresh post
    /// (`docs/ORDERS.md` §24). Off that phase it does nothing.
    ///
    /// SEAM — the guard above it that no capture has reached:
    /// `ptype +0x2b8 & 4` with `unit_masks & 0x80000` (the packable
    /// lineage, which takes an `add_cast_order` branch instead).
    ///
    /// SEAM — the head-order conjunct `head->vt+0x2c() == 0 ||
    /// head->vt+0x94()` names *me*: a group move's head is reviewed only
    /// by the member the order belongs to. This crate asks only
    /// `group.is_none()`, which is the same answer for every capture on
    /// file — all four of run112's reviews fire on a plain `MOVE_TO`.
    fn check_target_path_review(&mut self, u: usize, frame: i64) {
        let Some(Body::Move(m)) = self.current_order(u).map(|o| o.body) else {
            return;
        };
        if m.group.is_some() || self.units[u].phase(frame).rem_euclid(16) != 0 {
            return;
        }
        // `action->vt+0x20() != 0` is "this order can be an action", which
        // [`Self::action_of`] has already answered; `!= 9` excludes
        // `AWAIT_BOARD`, which this crate never puts under a move.
        let Some(a) = self.action_of(u) else { return };
        match self.units[u].orders[a].body {
            Body::Attack(_) => {
                self.check_target_path(u);
            }
            Body::Guard(_) if self.units[u].phase(frame).rem_euclid(64) == 0 => {
                self.repath(u);
            }
            // **The `GARRISON` arm** (`check_target_path@005e22d0`, action
            // type `0x1a`): a building target that is on the map and
            // `Object::adjacent_to` the unit (vslot `+0x170`) ends the walk
            // — `repath` pops the leg — and the head re-read runs
            // `do_garrison` on the same frame, which takes the squad in
            // (`docs/ORDERS.md` §29). run208's chariot goes in on 699
            // without its last step, ~140 short of its leg's point.
            Body::Garrison { building, .. } if self.adjacent_to(u, building) => {
                self.repath(u);
            }
            _ => {}
        }
    }

    /// `Unit::change_target@005e36c0` — one decision, written down the
    /// **whole squad** in place (`docs/COMBAT.md` §37.4).
    ///
    /// The function is a walk of `o_down` from the unit it is called on,
    /// and at each link it does two things: rewrite the action order's
    /// target from `old` to `new` *if it still holds `old`*, and then pop
    /// head orders until the head is the targeted one — which is what
    /// drops the chase the retarget has just made pointless. The walk
    /// stops at `o_down < 0` or at the first link whose `flags & 1` is
    /// clear.
    ///
    /// **The rewrite is in place, and that is measurable.** run112's
    /// `0/10` and `0/11` take `1/6` on block 645 with `in_range 1`,
    /// `ever_in_range 1` and `new_ord 0` — the flags they were already
    /// carrying against `1/8`. A fresh `add_attack_order` through the
    /// captain mirror would have reset all three, so the dump
    /// distinguishes this from every order-creating path, and it is what
    /// rules out `Unit::think`'s own `near_o` arm (§37.5) as the
    /// mechanism even before one notices that `think` is reached from
    /// `do_idle` alone.
    ///
    /// SEAM: the head-popping test is `UnitOrder::is_targeted` (`vt+0x20`
    /// by the type record — every class deriving `TargetOrder` answers
    /// it), and this crate asks for [`Body::Attack`] instead. They agree
    /// wherever the action is an attack, which is every capture on file
    /// that reaches this at all; a `GATHER` or a `BUILD_AT` under a move
    /// would part.
    fn change_target(&mut self, u: usize, old: Obj, new: Obj) {
        let mut v = u;
        loop {
            if let Some(a) = self.update_action(v)
                && matches!(self.units[v].orders[a].body, Body::Attack(_))
                && self.units[v].combat.target == Some(old)
            {
                self.units[v].combat.target = Some(new);
                while self.units[v]
                    .orders
                    .front()
                    .is_some_and(|o| !matches!(o.body, Body::Attack(_)))
                {
                    self.kill_current_order(v);
                }
            }
            let Some(next) = self.units[v].o_down else {
                return;
            };
            if !self.active(Obj::Unit(next)) {
                return;
            }
            v = next;
        }
    }

    /// `Unit::check_target_path@005e22d0`'s first arm: **the chase that has
    /// arrived**. The target is in reach on the plain radius — no
    /// `mandatory` margin, unlike `do_move`'s own kill (§35.3) — so the
    /// transit legs are dropped and the attack underneath takes over.
    ///
    /// The flank triple ahead of the range test is `do_move@005f7fbe`'s,
    /// with one difference that matters: the angle it compares the
    /// target's facing against is the **bearing from the attacker to the
    /// target** (`find_angle(t.x − my.x, t.y − my.y)`) rather than the
    /// attacker's own heading. A target facing within 120° of that bearing
    /// is running away from me, and a running target that is *moving* is
    /// chased rather than shot at.
    ///
    /// Returns whether the review changed the order list.
    ///
    /// SEAM: everything past the range test. The original falls through to
    /// `find_attack_pos`, `add_move_order`, `find_new_target` and
    /// `Group::action_attack` when the target is **inactive**, or when the
    /// flank triple holds (a flanked, moving target). Neither is reached
    /// by any capture on file — run112's four reviews all have a
    /// stationary target — and the arms are `docs/COMBAT.md` §36.4.
    fn check_target_path(&mut self, u: usize) -> bool {
        let me = Obj::Unit(u);
        let Some(t) = self.units[u].combat.target else {
            return false;
        };
        if !self.target_is_seen(me, t) || !self.active(t) {
            return false;
        }
        let Obj::Unit(tu) = t else { return false };
        // `5e2434`-`5e24b3`: the target's facing against the bearing to it.
        let (tp, mp) = (self.pos_of(t), self.units[u].pos);
        let bearing = crate::movement::find_angle(tp.x - mp.x, tp.y - mp.y);
        let e = (self.units[tu].movement.heading.0 as u32)
            .wrapping_sub(bearing.0 as u32)
            .wrapping_add(0x8000_0000);
        // `5e2493`: the negative half of the window is tested inline and
        // jumps past the call, so `flanking` alone is not the predicate.
        if e >= 0x2aaa_aaaa && combat::flanking(e) != 0 && self.is_moving(tu) {
            return false;
        }
        if !self.is_in_range(me, t) {
            return false;
        }
        self.repath(u);
        true
    }

    /// `Unit::repath`: pop the leading transit legs so the target order
    /// re-issues them.
    pub(crate) fn repath(&mut self, u: usize) {
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
        // **A non-captain's whole think is the captain mirror**, and it is
        // `Unit::think@005f6e40`'s *first* statement — above the citizen
        // mask-clear, above both cadence gates, above everything
        // (`docs/COMBAT.md` §21). The listing is five tests and a return:
        //
        //     if (!is_captain(this)) {
        //         a = get_action(units[who][get_captain()]);
        //         if (a == 0 || a->get_type() != 10) return;   // ATTACK
        //         if (!valid_target(this, a->o, a->who)) return;
        //         add_attack_order(this, a->o, a->who, QUEUE_NEW,
        //                          a->mandatory, 0);
        //         return;
        //     }
        //
        // — so a squad member never searches, never picks a target of its
        // own, and never reaches `think_attack` at all: it copies whatever
        // its captain is attacking. That is what the golden record's
        // `near_o` witnesses (`docs/COMBAT.md` §18): over 901 frames only
        // `1/6` and `0/6`, the two captains, ever carry one.
        if !self.units[u].captain {
            self.captain_mirror(u);
            return;
        }
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
        //
        // **The military bit belongs to the whole arm, not only to the
        // join** (`docs/COMBAT.md` §33). `think@005f6e40:150` reads
        //
        //     (is(0x3e, 1) && (merchant arm || (!packing && think_attack())))
        //     || (type->attack != 0 && (type->role & 0x10000) != 0 && think_attack())
        //
        // — so a type with an attack column and no `role & 0x10000` never
        // enters `think_attack` at all. [`Sim::think_attack_join_army`]
        // has carried that pair since item 350 and the search beside it
        // carried only the attack, so **a citizen** — `attack 40`,
        // `obj_masks & CIVILIAN`, no military bit — took an attack order
        // on its first idle frame. The base column is the original's
        // (`+0x1e8`), not the runtime stat.
        //
        // SEAM: the merchant arm (`is(0x3e, 1)`), which no capture on
        // disk reaches; `docs/MERCHANT.md` owns it.
        let p = self.profile(me);
        if p.attack != 0
            && p.combat_role
            && (unit.idle == 1 || phase & 0x1f == 0)
            && !self.think_attack_packed(u)
        {
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
        // **The human rare collector's deploy** (`think@005f6e40:163`–`199`,
        // `docs/ORDERS.md` §23), between the caravan and the computer
        // block, and so **above** `ai off`'s exit. A packed merchant or
        // fishing boat whose leader is human (`leader_flags & 4`) asks
        // `do_gather(…, 1, 1, −1, −1)` — `calc_gather` with `param_7` set,
        // [`Sim::calc_gather`] — where it stands, and if that answers
        // `unpack_merchant(this, 4)`: the ring-4 spot search and the
        // `[MOVE_TO, CAST]` pair. A computer's boat never enters it; its
        // deploy is `think_fish`'s, in the tail below. run127's fisher
        // `0/7` is the diff: born on 620 under a human leader, it walks
        // to (11736, 35352) and casts `0x292` for forty frames, and the
        // deploy lands on block 665 — golden chapter five's word 664.
        //
        // The cadence is the tail's (`idle == 1`, else one in thirty-two
        // phased by `o`). A search that does not deploy bumps `idle` a
        // second time this frame (`:196`); the console's message and
        // `S_INVALID_ORDER` beside it are interface.
        //
        // SEAM: `(unit_masks & 0x100) == 0 || is_merchant` — the "ordered
        // recently" bit (`docs/MERCHANT.md` §7), which this crate does not
        // keep, so a boat is always taken as not recently ordered.
        if self.is_rare_collector(u)
            && self.nation[self.units[u].owner as usize].human
            && self.units[u].combat.packed
            && (self.units[u].idle == 1 || phase & 31 == 0)
        {
            if self.calc_gather(u) && self.unpack_merchant(u, 4) {
                return;
            }
            self.units[u].idle = self.units[u].idle.wrapping_add(1);
        }
        // **The computer block, and the second place `ai off` is read**
        // (`docs/INPUT.md` §11). `think@005f6e40:205` opens
        // `if ((leader_flags & 4) != 0 || ai_off != 0)`, and its only
        // unconditional statement is the exit below it:
        // `if ((unit_masks & 0x40000) == 0) goto LAB_005f761a` — the
        // function's return. The arms inside are the computer leader's
        // alone (`uVar4 != 0` guards them), so for a human leader with the
        // cheat on the block is exactly one thing: **a unit that is not
        // AI-driven loses the whole tail** — `think_fish`,
        // `think_merchant`, `think_scout`, `think_carry`, the army join.
        // Everything above this line — the auto-attack arm,
        // `think_peasant`, `think_caravan` — is untouched, which is what
        // `docs/RUNS.md` run101–run105 measured as "auto-engage survives
        // AI-off".
        //
        // **Why the predicate is one term here and two there.** This crate
        // has a single stand-in, `ai_driven(owner)`, for both of the
        // original's words — `leader_flags & 4` on the leader and
        // `unit_masks & 0x40000` on the unit. Substituting it for both,
        // `(x || ai_off) && !x` is `ai_off && !x`, which is what stands.
        // SEAM: the two are not the same word in the original — a human
        // who leaves the auto-manage option on has citizens carrying
        // `0x40000` under a leader that is not computer-controlled — so
        // when a per-unit bit lands, this reverts to the two-term form.
        if self.ai_off && !self.ai_driven(self.units[u].owner) {
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

    /// `Unit::think_attack@005f5a80`'s **packed-unit arm**
    /// (`docs/COMBAT.md` §51), between the function's stance head and its
    /// army join — so above both the join and the target search. `true`
    /// is the function's `return 0` before either: the think goes on
    /// past the auto-attack arm exactly as if nothing had been found.
    ///
    /// The listing (`llvm-objdump 0x5f5b0a..0x5f5c00`):
    ///
    /// ```text
    /// if type.unit_flags2 & 4 and unit_masks & 0x80000:      # packs, packed
    ///     if !is(MERCHANTDUTCH 0x3e, 1):
    ///         if get_packer_stance() == PACKER_AUTO:          # vslot +0x100
    ///             thr = is(MACHINEGUN 0x7b, 0) ? 3
    ///                 : leader_flags & 4 ? 7 : 0x15           # human : computer
    ///             if idle >= thr:
    ///                 add_cast_order(-1, -1, -1, -1, 0x28c, QUEUE_FIRST, 0)
    ///                 return 0
    ///         if manual: return 0                             # 5f5bf6
    /// ```
    ///
    /// `manual` is the head's: not AI-driven (`unit_masks & 0x40000`
    /// clear), or AI-driven under a leader without `leader_flags & 2`, or
    /// `leader_flags2 & 8` — this crate's [`Sim::think_attack_join_army`]
    /// reads the same word as `!ai_driven || defeated`, and so does this.
    /// `get_packer_stance@00610970` is the unit's `stance` byte when the
    /// type's stance type is `STANCE_PACKER` (3), else `PACKER_NEVER` (1);
    /// `PACKER_AUTO` is 0 (the PDB's `LF_ENUMERATE`s).
    ///
    /// So **a human's packed siege engine never searches**: it waits,
    /// packed and orderless, for the first auto-attack frame with `idle ≥
    /// 7`, and its first order is the unpack. run145's catapult `0/9` is
    /// the diff — born on 621 at `idle 1`, cast on 696 at `idle 7`,
    /// unpacked on 776 — and run146's `0/6` is the cadence's: `idle 7` on
    /// 683, which is a 16-phase frame and not a 32-phase one, and the cast
    /// on 699 at `idle 8`. A computer's packed unit (`manual` clear) falls
    /// through to the join and the search while it waits out its 21.
    ///
    /// SEAM: `is(0x3e, 1)` is read as the strict lineage of the Dutch
    /// merchant, and a merchant never reaches this arm through the
    /// military one.
    fn think_attack_packed(&mut self, u: usize) -> bool {
        let Some(t) = self.units[u].ty else {
            return false;
        };
        if !self.unit_types[t].combat.packs || !self.units[u].combat.packed {
            return false;
        }
        if self
            .unit_tree(u)
            .is_some_and(|ti| self.tech_tree.is(ti, MERCHANTDUTCH, true))
        {
            return false;
        }
        let who = self.units[u].owner;
        if self.unit_stance_type(u) == crate::group::StanceType::Packer
            && self.units[u].stance == PACKER_AUTO
        {
            let threshold = if self.unit_line_is(u, MACHINEGUN) {
                3
            } else if self.nation[who as usize].human {
                7
            } else {
                0x15
            };
            if self.units[u].idle >= threshold {
                self.add_cast_order(u, spell::UNPACK);
                return true;
            }
        }
        !self.ai_driven(who) || self.defeated[who as usize]
    }

    /// `Unit::think@005f6e40`'s opening arm — the **captain mirror**
    /// (`docs/COMBAT.md` §21). A unit with a captain takes that captain's
    /// standing ATTACK order and nothing else; the think ends here whether
    /// the mirror fires or not.
    ///
    /// Four things it is not. It is not gated by the stance, by
    /// `unit_masks & 0x100`, or by either cadence — an idle member mirrors
    /// on the first frame it is idle. It reads the captain's **action**
    /// (`UnitData::get_action`, the intent under the pathing legs), so a
    /// captain walking to a chase point still hands its target down. It
    /// tests `Object::valid_target` on the **member**, not on the captain.
    /// And the order it adds is `QUEUE_NEW` with the captain's own
    /// `mandatory` byte — `add_attack_order`'s `action` argument is 0, so
    /// a DEFENSIVE member still takes a post.
    ///
    /// **The frame it explains**, twice over (`docs/COMBAT.md` §21): who=1's
    /// three hoplites all carry `type 10 ox 7 whom 0` at the end of frame
    /// 615, the frame they are born, and only the captain `1/6` carries a
    /// `near_o`; and who=0's `0/7`/`0/8` take their retaliation ATTACKORDER
    /// on **617**, one frame after the captain `0/6` took it at 616, because
    /// the captain's order is written after they have already been processed.
    pub(crate) fn captain_mirror(&mut self, u: usize) {
        let cap = self.squad_captain(u);
        if cap == u {
            return;
        }
        // `get_action`'s order must be an ATTACK — `get_type() == 10`.
        let Some(i) = self.action_of(cap) else {
            return;
        };
        if !matches!(self.units[cap].orders[i].body, Body::Attack(_)) {
            return;
        }
        let Some(t) = self.units[cap].combat.target else {
            return;
        };
        if !self.valid_target(Obj::Unit(u), t) {
            return;
        }
        let mandatory = self.units[cap].combat.mandatory;
        self.add_attack_order(u, t, QueuePos::New, mandatory, false);
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
    /// job on the *first* frame it is idle. That one frame is what run33's
    /// trained citizen spends: it comes out at 99, is first visited at 100,
    /// and its walk — and the collision that ends it at 122 — hangs off that
    /// frame (`docs/SYNC.md` §3.16).
    ///
    /// **A human's is [`crate::stance::LeaderOptions::idle_wait`], and it is
    /// 12** — the option's switch, not the option itself (item 494,
    /// `docs/ORDERS.md` §21). Reading `peasants_wait` as the wait put the
    /// human's citizen back to work ten frames early, which is Great Lakes'
    /// word at 10294: `0/5` takes its flight to `(792, 31800)` with a
    /// `GATHERORDER` queued under it, so it never has the single order
    /// [`Self::arrive`] needs to face the order's angle, and the frame the
    /// original spends in `do_idle` it spends walking away.
    pub(crate) fn think_peasant(&mut self, u: usize, forced: bool) -> bool {
        let unit = &self.units[u];
        let ai = self.ai_driven(unit.owner);
        if !forced {
            let t = if ai {
                1
            } else {
                i32::from(self.leader_options(unit.owner).idle_wait())
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
            // **A human's found gather drops its group** (§5.9; the
            // listing at `5f5900`–`5f590c`: `testl $0x40000, 0x68(%esi)`,
            // and for a human `orl $-1` into `movw %ax, 0x80(%esi)`). The
            // pool keeps listing the unit; only the back-pointer goes, so
            // `find_ordered_collision`'s own-group arm stops seeing the
            // swarm it came from. Without it, chapter twenty-one's `0/7`
            // was refused the camp spot `0/8` was already sent to and
            // walked a tile west (item 824, `docs/GOLDEN.md` §29).
            //
            // SEAM: the `deselect` ahead of it (`5f58d2`–`5f58fd`, the
            // console player's select list, vslot `+0x10`, when it holds
            // more than one) is the UI's selection, which this crate does
            // not model.
            if !ai {
                self.units[u].group_ptr = None;
            }
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

    /// `Unit::do_guard@005e5c70` (`docs/ORDERS.md` §7.5, §24) — one frame
    /// of an escort.
    ///
    /// A dead target ends the order with an idle stand. A pending `retry`
    /// counts down and does nothing else. Beside a target that is not
    /// moving, one frame in sixteen looks for a fight and another counts
    /// `idle`. Otherwise the post is recomputed — the target's position
    /// plus `(dx, dy)` turned into the target's own frame, snapped to its
    /// 48-unit cell — and a unit off that cell is given an `ATTACK_TO`
    /// leg to it at the head of its list, stepped **this same frame**,
    /// with a `timer` of thirty frames a world cell of Manhattan distance.
    /// A unit on its post turns to the facing and stands.
    ///
    /// The four fastcall pairs are the listing's, not the decompiler's
    /// (`tools/ghidra/README.md`): the post is `x + sinx(a, dy) + cosx(a,
    /// dx)`, `y − cosx(a, dy) + sinx(a, dx)` with `a` the target's heading
    /// (`5e6022`–`5e6061`), and the idle facing is `find_angle(post −
    /// target)` — outward, away from what is guarded (`5e626f`).
    ///
    /// The target's own `unit_masks & 2` ([`crate::Movement::mirror`])
    /// negates `dx` before the turn (`5e5fed`–`5e5ff6`): run196's Despot
    /// carries it on 15095, and its escort's flanks stand on the other
    /// side for it (`docs/GROUPS.md` §25).
    ///
    /// SEAM, none of them reached by a capture on file: the building-target
    /// arm (`action_guard`'s building arm
    /// is itself a seam, so no building is ever a target here); the
    /// packer's unpack after `0x1e`/`0x46` frames on the post; and the
    /// sixteen-frame engagement, which asks [`Self::find_melee_target`]'s
    /// idle radius rather than `find_melee_target(−1, 0, 0, 1, 0)`'s own
    /// guard arm.
    /// `Unit::do_patrol@005f1910` (`docs/ORDERS.md` §27) — a patrol at the
    /// head of the list.
    ///
    /// - **The leader** (`order.leader` is this unit): step `waypoint`
    ///   modulo the point count and hand the group the next leg,
    ///   `Group::action_move_to(group, point, QUEUE_FIRST, 0, 0,
    ///   ATTACK_TO, action 0, −1, −1, 0)`. A group's `QUEUE_FIRST` halts
    ///   every member and re-issues the leader's action orders behind the
    ///   leg — the patrol among them, through `redo_patrol_order`
    ///   ([`Self::group_redo_patrol_order`]) — so the step is carried to
    ///   every member's rebuilt patrol.
    /// - **A follower**: `set_anim(CHAR_DEFAULT, 0, 1)`, and nothing else;
    ///   its legs are the leader's group moves.
    ///
    /// SEAMS: the arm for a unit in no group (`+0x80 < 0`), which steps the
    /// waypoint and pushes a bare `ATTACK_TO` of its own — no command
    /// reaches it, because `process_group` forces the push even for one
    /// unit; and the tail's `inside_down >= 0` scramble, a transport's.
    pub(crate) fn do_patrol(&mut self, u: usize, p: PatrolOrder) {
        let Some(g) = self.group_of(u) else {
            return;
        };
        if p.leader != u {
            self.set_default_anim(u);
            return;
        }
        let waypoint = (p.waypoint + 1) % p.points.len();
        if let Some(Body::Patrol(x)) = self.units[u].orders.front_mut().map(|o| &mut o.body) {
            x.waypoint = waypoint;
        }
        self.group_action_move_to(
            &g,
            p.points[waypoint],
            QueuePos::First,
            false,
            Angle(0),
            MoveKind::AttackTo,
            false,
        );
    }

    fn do_guard(&mut self, u: usize, frame: i64) {
        let Some(Order {
            body: Body::Guard(mut g),
            ..
        }) = self.current_order(u).copied()
        else {
            return;
        };
        let store = |sim: &mut Sim, at: usize, g: GuardOrder| {
            if let Some(Body::Guard(x)) = sim.units[u].orders.get_mut(at).map(|o| &mut o.body) {
                *x = g;
            }
        };
        let t = g.target;
        // `ObjectData::flags & 1` — the target's slot is still an object.
        if !self.units[t].alive() {
            self.mark(SITE_GUARD_DEAD);
            self.set_anim(u, anim::DEFAULT, false, true);
            self.kill_current_order(u);
            return;
        }
        if g.retry != 0 {
            g.retry -= 1;
            store(self, 0, g);
            return;
        }
        // `is_active() && is_on_map() && is_moving()` (vslots `+0x8`,
        // `+0xbc`, `+0xd8`): `iStack_9e4`.
        let moving = self.units[t].on_map && self.is_moving(t);
        if !moving {
            let ph = self.units[u].phase(frame);
            if (ph + 8).rem_euclid(16) == 0 {
                if let Some(v) = self.find_melee_target(u, -1) {
                    self.add_attack_order(u, v, QueuePos::First, false, false);
                }
                return;
            }
            if ph.rem_euclid(16) == 0 {
                g.idle += 1;
                store(self, 0, g);
                return;
            }
        }
        let a = self.units[t].movement.heading;
        let tp = self.units[t].pos;
        // `5e5fed`: the target's own `unit_masks & 2` mirrors the offset
        // (`docs/GROUPS.md` §25).
        let dx = if self.units[t].movement.mirror {
            -g.dx
        } else {
            g.dx
        };
        let gx = tp.x + movement::sin_component(a, g.dy) + movement::cos_component(a, dx);
        let gy = tp.y - movement::cos_component(a, g.dy) + movement::sin_component(a, dx);
        // `div_3_table[v >> 4]`: the 48-unit cell, a floor divide.
        let q = |v: i32| (v >> 4).div_euclid(3);
        let (mut qx, mut qy) = (q(gx), q(gy));
        if self.invalid_loc(
            u,
            Pos::new(qx >> 2, qy >> 2),
            false,
            false,
            false,
            false,
            false,
        ) != 0
        {
            let centre = Pos::new(qx * SNAP + SNAP_CENTRE, qy * SNAP + SNAP_CENTRE);
            let transport = self.units[u].ty.is_some_and(|ty| {
                self.unit_types[ty]
                    .cols
                    .flag(crate::ai_load::uflags::TRANSPORT)
            });
            let water = self.world.tile_mask(Pos::new(qx >> 2, qy >> 2)) & tile::SURFACE
                == tile::SURFACE_OCEAN;
            let spot = if !transport || water {
                let bearing = Angle(0x5555_5555);
                self.find_nearby_spot(u, centre, 0xc0, 0x180, 0x60, bearing, None)
                    .or_else(|| {
                        let d = vector_dist(g.dx, g.dy);
                        self.find_nearby_spot(u, tp, d, d + 0x180, 0xc0, bearing, None)
                    })
                    .unwrap_or(self.units[u].pos)
            } else {
                centre
            };
            (qx, qy) = (q(spot.x), q(spot.y));
        }
        g.guard = Pos::new(qx * SNAP + SNAP_CENTRE, qy * SNAP + SNAP_CENTRE);
        let mut facing = if moving {
            a
        } else {
            find_angle(g.guard.x - tp.x, g.guard.y - tp.y)
        };
        // `5e6297`: an AI-driven siege engine, wagon or hero faces where
        // its charge faces, moving or not.
        if self.ai_driven(self.units[u].owner)
            && (self.is_siege_unit(u) || self.is_supply_unit(u) || self.is_hero_unit(u))
        {
            facing = a;
        }
        let here = self.units[u].pos;
        if q(here.x) != qx || q(here.y) != qy {
            g.idle = 0;
            store(self, 0, g);
            self.add_move_facing_order(
                u,
                g.guard,
                MoveKind::AttackTo,
                QueuePos::First,
                false,
                facing,
                None,
                false,
            );
            // `div_3_table[v >> 8]`: the world cell, 768 units.
            let cell = |v: i32| (v >> 8).div_euclid(3);
            let d = (cell(here.x) - cell(g.guard.x)).abs() + (cell(here.y) - cell(g.guard.y)).abs();
            if let Some(m) = self.units[u].orders.front_mut().and_then(Order::move_mut) {
                m.timer = 0x1e * d.max(1);
            }
            self.do_move(u, frame);
            if self.order_type(u) != index::GUARD {
                return;
            }
            self.mark(SITE_GUARD_STAND);
            self.set_anim(u, anim::DEFAULT, false, true);
            self.mark(SITE_GUARD_RETRY);
            let r = self.rng.roll();
            if let Some(Body::Guard(x)) = self.units[u].orders.front_mut().map(|o| &mut o.body) {
                x.retry = r % 3 + 6;
            }
            return;
        }
        if self.units[u].movement.heading != facing && !moving {
            self.unit_set_angle(u, facing);
        }
        g.idle += 1;
        store(self, 0, g);
        self.mark(SITE_GUARD_IDLE);
        self.set_anim(u, anim::DEFAULT, false, true);
    }

    /// `Unit::do_follow@005e65d0` (`docs/ORDERS.md` §28), with the
    /// `FOLLOW` at the head.
    ///
    /// A leader no longer active and on the map, or not seen by the
    /// follower's player (`UnitData::is_seen@00607a60(who, 0)`, which
    /// answers 1 for the owner), kills the order. Otherwise the standoff:
    ///
    /// - `k = los × 0x60` when the follower is the faster
    ///   (`UnitData::speed@0060aae0`), else `los × 0x300 / 5`, truncated;
    /// - `k` doubles while the leader `is_moving`;
    /// - `s = clamp(los × 0x180 − k, 0x180, 0x600)`, `los` the follower's.
    ///
    /// Within `s + 0xc0` of the leader (`vector_dist`), the follower stands:
    /// `set_anim(CHAR_DEFAULT, 0, 1)`. Farther, the point `s` from the
    /// leader toward the follower, placed by `find_nearby_spot`; failing
    /// that the point `s` behind the leader's heading; then a ring
    /// `s .. s + 0xc0` round the leader on its heading; then the leader's
    /// own place. A `MOVE_TO` leg goes on at `QUEUE_FIRST` without the
    /// action bit, facing the leader's heading, and `do_move` runs this
    /// frame (`5e6b1c`–`5e6b29`). The leg has no timer.
    ///
    /// SEAM: the container swap at the head (`5e6643`–`5e66fd`) and its
    /// tail (`5e6b5b`, the swap back and `Unit::work`), with
    /// [`FollowOrder`]'s `oxx/whose/uid2`.
    fn do_follow(&mut self, u: usize, f: FollowOrder, frame: i64) {
        let t = f.target;
        let who = self.units[u].owner;
        let seen = self.units[t].owner == who || self.target_is_seen(Obj::Unit(u), Obj::Unit(t));
        if !(self.units[t].alive() && self.units[t].on_map) || !seen {
            self.kill_current_order(u);
            return;
        }
        let me = self.units[u].pos;
        let tp = self.units[t].pos;
        let d = vector_dist((tp.x - me.x).abs(), (tp.y - me.y).abs());
        let los = self.unit_los(u);
        let mut k = if self.units[t].movement.speed < self.units[u].movement.speed {
            los * 0x60
        } else {
            los * 0x300 / 5
        };
        if self.is_moving(t) {
            k *= 2;
        }
        let s = (los * 0x180 - k).clamp(0x180, 0x600);
        if d <= s + 0xc0 {
            self.mark(SITE_FOLLOW_STAND);
            self.set_anim(u, anim::DEFAULT, false, true);
            return;
        }
        let heading = self.units[t].movement.heading;
        let bearing = Angle(0x5555_5555);
        let toward = crate::army::step_along(tp, find_angle(me.x - tp.x, me.y - tp.y), s);
        let behind = crate::army::step_along(tp, Angle(heading.0.wrapping_sub(i32::MIN)), s);
        let spot = self
            .find_nearby_spot(u, toward, 0, -1, 0, bearing, None)
            .or_else(|| self.find_nearby_spot(u, behind, 0, -1, 0, bearing, None))
            .or_else(|| self.find_nearby_spot(u, tp, s, s + 0xc0, 0x30, heading, None))
            .unwrap_or(tp);
        self.add_move_facing_order(
            u,
            spot,
            MoveKind::MoveTo,
            QueuePos::First,
            false,
            heading,
            None,
            false,
        );
        self.do_move(u, frame);
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

        // **A suspended search** (§4.4 step 2, item 301). `do_move`'s
        // first block after the cavalry-archer fire, and **no step happens
        // while one is pending**: every arm below returns.
        //
        // **Its Great Lakes cost is paid** (item 304,
        // `docs/PATHFINDER.md` §18.5). Wired, this block used to cost
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
        // The clock is `frame − collide_frame` — `UnitData +0x48`, which is
        // `collide_frame` by the type record, not a second stamp — so the
        // "has the blocker gone" probe fires on the 5th, 7th, 9th … frame
        // after the collision that suspended the search, and the `repaths`
        // tick on every fourth `o + frame`.
        //
        // SEAM: the `ATTACK` retarget (`find_new_target` every 4 frames)
        // is the arm above this one and is dormant; no capture has reached
        // it.
        if self.units[u].search.is_some() {
            let elapsed = frame - self.units[u].collide_frame;
            // **The `GATHER` park** (`docs/COLLISION.md` §15, item 698):
            // a gatherer whose search is suspended, on the frame
            // `(elapsed + 2) & 7 == 0`, within `vector_dist < 0x120` of the
            // move's own `x/y`, gives the walk up where it stands.
            // `avoid` takes the move's point (`005f7ce6`, not the unit's),
            // the gather order forgets its tile (`tx`/`ty`/`wait` −1,
            // `goto_build` 1) and the move dies, so the next frame's
            // `do_non_flat_gather` picks a tile afresh. `collide` is not
            // counted: the arm returns above the increment. Run178's `1/43`
            // on 14649, six frames after it met the standing `1/18`. The
            // `hold_doobers` cleanup beside it is presentation.
            let gathering = self
                .action_of(u)
                .is_some_and(|i| self.units[u].orders[i].index() == index::GATHER);
            if gathering
                && elapsed > 0
                && (elapsed + 2) % 8 == 0
                && vector_dist(
                    mo.dest.x - self.units[u].pos.x,
                    mo.dest.y - self.units[u].pos.y,
                ) < 0x120
            {
                self.units[u].avoid = Some(mo.dest);
                if let Some(mut g) = self.units[u].orders.iter().find_map(|o| match o.body {
                    Body::Gather(g) => Some(g),
                    _ => None,
                }) {
                    g.tile = None;
                    g.wait = -1;
                    g.goto_build = true;
                    self.store_gather(u, g);
                }
                self.kill_current_order(u);
                return Did::Nothing;
            }
            // The probe is the **quick** form (`docs/COLLISION.md` §18,
            // item 857): an occupied cell is the blocker still there, with
            // no corner rule and no write to `collide_o`. East Indies'
            // `1/71` on 19498 met `1/75`'s corner at `coll`, which the full
            // form let pass and the original counts.
            let coll = mo.coll.unwrap_or(self.units[u].pos);
            if elapsed > 3 && (elapsed - 1) % 2 == 0 && !self.blocker_still_there(u, coll) {
                // The blocker has gone: drop the search and re-plan next
                // frame off the stack as it stands.
                mo.has_waypoint = false;
                self.clear_partial_path(u);
                flags |= flag::PATHED;
                self.units[u].collide = 0;
                self.store_move(u, mo, flags);
                return Did::Nothing;
            }
            if (i64::from(self.units[u].index) + frame) % 4 == 0 {
                let who = self.units[u].owner as usize;
                self.repaths[who] += 1;
            }
            self.units[u].collide += 1;
            mo.has_waypoint = false;
            self.store_move(u, mo, flags);
            self.find_upath_restore(u);
            if self.units[u].search.is_some() {
                return Did::Nothing;
            }
            if frame - 5 <= self.units[u].collide_frame {
                return Did::Nothing;
            }
            self.units[u].collide = 0;
            return Did::Nothing;
        }

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
        //
        // **And the whole block is a RANGED attacker's** (item 405). The
        // original's test at `do_move@005f7b30:212` is
        // `if (*(int *)(*(int *)&this->field_0x18 + 0x1fc) != 0)` —
        // `SubObjectData +0x18 ptype`, `ObjectTypeData +0x1fc max_range`
        // by the type record — and **everything** the ATTACK action does
        // under a move sits inside it: the unit-target kill, the building
        // -target kill, and the melee retarget beneath them. A melee type
        // never abandons its chase because the target came into reach; it
        // walks the leg it was given and `do_attack` takes over when the
        // move ends. `docs/ORDERS.md` §4.4 has said "a ranged type" since
        // the second reading and this crate asked every type.
        //
        // It is the golden record's frames 619 and 620. `1/8` stands at
        // `(1332, 8121)` with `attack_dist` **246** to `0/7` — exactly
        // `0xf6`, the HOPLITES reach, in range by a single unit — and
        // this crate killed the move 171 short of the `(1176, 8088)` the
        // dump walks it to; `1/7` did the same a frame later at 240. The
        // original's own record says the collision half was never it:
        // `tolerance` is 0 and `collide_o`/`collide_frame` are clear on
        // every block 616..629, on both sides (`docs/RUNS.md` run110).
        //
        // SEAM: two further conjuncts of the unit-target kill are not
        // modelled, and both only make it **rarer** — the flank clause
        // (`angle` window, `flanking`, `max_range` again) and
        // `Objects::find_collision(my own spot, o, who, 1) == 0`.
        // ~~The `vector_dist < 0x481` guard on the re-path arm is the
        // third.~~ **Item 487 landed it**, below.
        if let Some(a) = self.action_of(u)
            && let Body::Attack(_) = self.units[u].orders[a].body
        {
            let me = Obj::Unit(u);
            match self.units[u].combat.target {
                Some(t) if self.valid_target(me, t) => {
                    // **The gate is a block, not a conjunct** (item 481).
                    // `do_move@005f7b30:207` opens
                    // `if (ptype->max_range != 0) { … }` and the brace
                    // closes past the captain retarget below, so *both*
                    // arms are a ranged attacker's. Writing it as a
                    // conjunct of the kill alone let a melee captain
                    // retarget: run112's hoplite captain `1/6` carries
                    // `near_o 10` from block 621 to the end of the
                    // window and this crate switched its whole squad off
                    // `0/11` at 671, where the dump has all three
                    // hoplites on `0/11` for every block of the capture.
                    // `docs/COMBAT.md` §38.
                    if self.profile(me).max_range != 0 {
                        // **`is_in_range`'s sixth argument**, and this is
                        // the executable's only caller that sets it: a
                        // non-`mandatory` chase is dropped `0x90` inside
                        // the attacker's reach rather than at its edge
                        // (`docs/COMBAT.md` §35.3).
                        let margin = !self.units[u].combat.mandatory;
                        let at = self.units[u].pos;
                        if self.is_in_range_at_margin(me, at, t, margin) {
                            self.kill_current_order(u);
                            return Did::Something;
                        }
                        // **The captain's retarget**, `005f803f`-`005f8216`
                        // (`docs/COMBAT.md` §37.2). It runs only when the
                        // kill did *not* fire — so a ranged captain
                        // walking to a target it cannot yet reach asks,
                        // every frame, whether the incumbent its last
                        // search left in [`crate::Unit::near`] is one it
                        // can.
                        //
                        // The two range tests are the same function with
                        // a different sixth argument, and that is the
                        // whole of why both can be true on one frame: the
                        // kill uses the reach less `0x90`, this uses the
                        // plain reach.
                        //
                        // `mandatory` gates it — an ordered attack is
                        // never retargeted under the player — and so does
                        // `is_captain`: a squad member's target comes
                        // down the chain from here and it decides nothing
                        // itself.
                        if !self.units[u].combat.mandatory
                            && self.units[u].captain
                            && let Some(c) = self.units[u].near
                            && c != t
                            && self.active(c)
                            && self.is_in_range(me, c)
                            && !self.poor_target(me, c)
                            && matches!(c, Obj::Unit(_))
                            && self.profile(c).combat_role
                        {
                            // `005f820a`, and it is the fingerprint the
                            // item was found by: `in_range` is written
                            // **here**, on the deciding unit alone and a
                            // frame before `Unit::fight` writes
                            // `ever_in_range`. run112's `0/9` prints
                            // `in_range 1` with `ever 0` on block 645 and
                            // nothing else in the executable can produce
                            // that pair.
                            if let Body::Attack(x) = &mut self.units[u].orders[a].body {
                                x.in_range = true;
                            }
                            self.change_target(u, t, c);
                            return Did::Something;
                        }
                    }
                }
                // **The dead target's arm, and it is a *near* one**
                // (item 487, `docs/ORDERS.md` §20). The original reaches
                // here when the action's target is gone — a negative
                // `o`/`who`, an object whose `flags & 1` is down, or a
                // slot whose `uid` no longer matches — and it does **not**
                // re-path unconditionally. `do_move@005f7b30`,
                // `005f8221`-`005f825d`:
                //
                // ```
                // 5f8221: eax = this->y ^ 0x63637; eax -= [edi+0x8]
                // 5f822f: eax = this->x ^ 0x63637; eax -= [edi+0x4]
                // 5f8247: call vector_dist            ; (|dx|, |dy|)
                // 5f824c: cmp  eax, 0x480
                // 5f8251: jg   0x5f82c1                ; → the planner
                // 5f8256: test $0x10, 0x2b4(ptype)     ; a sea transport
                // 5f825d: jne  0x5f82c1                ; → the planner
                // 5f8261: call Unit::repath
                // ```
                //
                // `edi` is the `MoveOrder` the function opened with
                // (`5f7b4b`'s vfunc `+0x40`, stashed at `[ebp-0x18]`), and
                // `+0x4`/`+0x8` are `x`/`y` by the type record — the
                // order's own destination. So the octagonal distance from
                // the unit to **where it was walking** decides it: within
                // `0x480` the walk is pointless and the transit legs are
                // popped; beyond it the unit keeps the order and falls
                // through to the planner, dead target and all.
                //
                // run100's `1/29` is the case and it is 28,000 units out:
                // an AI raider whose farm died on sim-frame 10230, whose
                // reload keeps `do_attack` from ever reaching `fight`'s
                // own validity kill, and whose move home the original
                // plans on 10240 — 43 nodes — while this crate popped it
                // and left the unit standing in `1/27`'s way.
                _ => {
                    let d = crate::world::vector_dist(
                        (self.units[u].pos.x - mo.dest.x).abs(),
                        (self.units[u].pos.y - mo.dest.y).abs(),
                    );
                    let transport = self.units[u].ty.is_some_and(|t| {
                        self.unit_types[t].cols.unit_flags & crate::ai_load::uflags::TRANSPORT != 0
                    });
                    if d <= 0x480 && !transport {
                        self.repath(u);
                        return Did::Nothing;
                    }
                }
            }
        }

        // **The retry delay a failed unit-grid search bought**
        // (`do_move@005f7b30:348`-`355`, `docs/PATHFINDER.md` §21). It sits
        // between the action tests and the planner, and while it stands the
        // move does nothing at all — no plan, no waypoint, no step. The
        // frame it runs out, `attempts` gains 3, which is what stops a unit
        // wedged against a neighbour buying delay after delay for ever.
        if mo.retry != 0 {
            mo.retry -= 1;
            if mo.retry == 0 {
                mo.attempts += 3;
            }
            self.store_move(u, mo, flags);
            return Did::Something;
        }
        // And the decay, on every frame that reaches the planner
        // (`005f7b30:369`). SEAM: the modern-infantry unpack between the
        // two — `is_modern_infantry && !has_general(0x8000)` on a
        // `(o * 0x11 + frame) & 0x7f == 0` phase, which sets `retry` from
        // the type's `+0x78` and `attempts` to −3 — is not modelled; no
        // capture has a packed type in it.
        if mo.attempts != 0 {
            mo.attempts -= 1;
            self.store_move(u, mo, flags);
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
            // `do_move@005f7b30:437`: under a `TRADE_ROUTE` action, a
            // waypoint the road was laid on asks whether the road is still
            // there (`docs/CARAVAN.md` §10).
            if top.flags & path_flag::ROAD != 0
                && self
                    .action_of(u)
                    .is_some_and(|i| self.units[u].orders[i].index() == index::TRADE_ROUTE)
            {
                self.caravan_road_step(u, top.to);
            }

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
            //
            // **And a ship never asks.** The call is `(x, y, 0, 0, 0, 0, 0)`
            // on the listing (`5f86f9`–`5f8707`); with `boats` zero the
            // second arm returns 0 at `61782b`, before any scan and past the
            // exit bookkeeping (`docs/COLLISION.md` §13). East Indies' Bark
            // `1/34` took a soft hit here from Trireme `1/32` beside the
            // navy's muster and walked its first step at half speed.
            let hit = if self.takes_boat_arm(u) {
                None
            } else {
                self.detect_unit_collision(u, mo.waypoint)
            };
            if let Some(other) = hit {
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
        let speed = self.get_speed(u, 0);
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
                            if is_loose(&t) {
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
                // **The verified line reads the stack again** (`5f8c3d`–
                // `5f8c5d`, `docs/GROUPS.md` §32): `peek`, then the order's
                // `dest_x`/`dest_y` and the unit's `+0x60` tolerance from
                // the top. A detour `find_path` just pushed and verified
                // (`go_around_building`, [`Sim::detour_verified`]) is the
                // top now, so it is what the step walks at — not the world
                // entry under it. Great Lakes' `1/40` walked past its own
                // detour for seventeen frames without this (item 795).
                if let Some(t) = self.units[u].path.last().copied() {
                    mo.waypoint = t.to;
                    self.units[u].tolerance = t.tolerance;
                }
                self.store_move(u, mo, flags);
            }
        }

        // Stepping — `STEP_IF_MOVING`, which `goto STEP` jumps past.
        if !straight_to_step && mo.pause != 0 {
            mo.pause -= 1;
            self.store_move(u, mo, flags);
            // **An unarmed attack-move stands out its pause**
            // (`do_move@005f7b30:734`–`741`): with the order's own type
            // `ATTACK_TO` or `GROUP_ATTACK_TO` and the type's attack 0, the
            // waiting frame is a `set_anim(CHAR_DEFAULT, 0, 1)` — one stand
            // per guy, each a draw (`Guy::set_anim+0x97a < Unit::set_anim <
            // Unit::do_move+0x11cf`). The pause it stands out is
            // `do_attack_to_pause`'s fifteen (`docs/ORDERS.md` §24.9).
            let t = Order {
                flags,
                body: Body::Move(mo),
            }
            .index();
            if matches!(t, index::ATTACK_TO | index::GROUP_ATTACK_TO)
                && self.profile(crate::combat::Obj::Unit(u)).attack == 0
            {
                self.mark(anim::SITE_PAUSE_STAND);
                self.set_default_anim(u);
            }
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
        // `if (this->group == -1) ungroup` (`5e79f0`): the back-pointer
        // ([`crate::Unit::group_ptr`], `docs/GROUPS.md` §23) names no seat,
        // and the group order degrades on the spot.
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
            // **The leader publishes the pass** (`5e7a9a`, which is
            // `Group::leader_report_speed@007137f0` inlined): its own
            // **uncapped** speed becomes the accumulator and what the
            // accumulator held becomes the cap every member without an
            // action order walks at this frame (`docs/GROUPS.md` §18).
            // It runs only on a frame the leader's own step succeeded,
            // and after it, which is why a group's cap is one pass stale.
            //
            // SEAM: the `march` arm under it —
            // `LeaderData & 0x8000 && has_general(0x8000, -1) >= 0` — and
            // no capture has a general, so `march` is only ever cleared.
            let mine = self.get_speed(u, 1);
            self.group_leader_report_speed(g, mine);
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
        //    "In my group" is the two back-pointers agreeing
        //    (`5e7c8c`: `leader.+0x80 != this.+0x80`), not a list.
        let lost = !(self.units[l].alive()
            && self.units[l].on_map
            && self.units[l].group_ptr == self.units[u].group_ptr
            && self.still_group_move(l, gm.id));
        // 2. `form_id` is rewritten from the list on every frame that
        //    reaches here (`5e7ea0`). A member the list no longer holds
        //    loses its group outright — `+0x80 = −1` (`5e7ed8`), then the
        //    ungroup (`docs/GROUPS.md` §23).
        let Some(i) = g.list.iter().position(|&m| m == u) else {
            self.units[u].group_ptr = None;
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
                    // The group's own formation — an army's, or a pushed
                    // group's ([`Sim::gstate`]). This read an army's list
                    // with `unwrap_or(0)`, which a pushed group (no army)
                    // indexed out of bounds: Great Lakes 18333, reached
                    // once item 695 moved the word past 14529. A slot the
                    // state does not hold stands at the leader's point.
                    let off = self
                        .gstate(g)
                        .and_then(|st| st.curr.get(i).copied())
                        .unwrap_or_default();
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
            // `5e83f6`: flag **0**, so a follower cutting across the
            // formation is capped by the group like any plain mover.
            self.get_speed(u, 0) / 2
        } else {
            // **And a follower in formation reports** (`5e8336`, which is
            // `Group::report_speed@00713bb0` inlined): its own
            // `UnitData::speed` — layer two, not `get_speed`, so neither
            // the ground it stands on nor the order it carries enters the
            // group's cap — drives the pair down and never up. This is
            // what puts a slow squad's speed on a fast leader, and a
            // follower that stops reporting lets the cap drift back to
            // the leader's own over the next two frames.
            if !self.gstate(g).is_some_and(|st| st.march) {
                let own = self.units[u].movement.speed;
                self.group_report_speed(g, own);
            }
            // `5e8355`: flag **1**. A follower keeping formation is
            // allowed its own speed and a third on top, uncapped — the
            // catch-up, and the reason the cap never stops the block
            // closing up.
            let v = self.get_speed(u, 1);
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
    pub(crate) fn squad_captain(&self, u: usize) -> usize {
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
            // `orig_x`/`orig_y` take the order's own `x`/`y` here
            // (`MoveOrder +0x44`/`+0x48`): `1/29`'s go `(39133, 21131)` →
            // `(38952, 21048)` across run100's ungroup. `finish_insert`
            // replays a copy to this point (item 738).
            m.orig = Some(m.dest);
        }
        // **The plain move is a *new* order at the head of the list, not
        // the old one rewritten where it stood** (item 487,
        // `docs/ORDERS.md` §20). `005fd2e2`-`005fd3a6`:
        // `OrdersMemManager::get_obj(MOVE_TO)`, `MoveOrder::operator=` off
        // the group order, then `remove_current` on the list, `give_obj`
        // on the node just removed, and
        // `LinkListBase<UnitOrder *, …>::add@0046d5a0` — which **prepends**
        // (`head_node = new`) and leaves the cursor on it. So the
        // conversion also *promotes*: whatever was current before the
        // ungroup is now one place back.
        //
        // In place and at the head are the same thing for a unit whose
        // group move is already the head order, which is every unit any
        // capture on disk had reached until run100's `1/29`. That one has
        // an `ATTACK` above its group move — on a farm that died on
        // sim-frame 10230, kept alive by `do_attack`'s reload gate — so
        // the two readings come apart: the original walks home from
        // block 10241 and this crate stood still.
        //
        let order = self.units[u]
            .orders
            .remove(i)
            .expect("the order just matched");
        self.units[u].orders.push_front(order);
        if !leader {
            self.kill_current_path(u);
        }
    }

    /// `Unit::kill_group_move@005e3400` over the group
    /// (`Group::kill_group_move@007123f0`): every member's orders lose the
    /// group plan, and every one carrying `id` that is **not** an
    /// attack-move is killed. A `GROUP_ATTACK_TO` survives both arms.
    pub(crate) fn kill_group_move(&mut self, g: &crate::group::Group, id: i64) {
        // `007123f9`: `normalize(this)` before anything else, which is
        // where the group's speed goes back to its leader's
        // (`docs/GROUPS.md` §18) after a march's own reports have walked
        // it up to the leader's uncapped speed.
        self.group_set_speed(g);
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
        let speed = self.get_speed(u, 0).max(3);
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
        // `move_step`'s probe is `(x, y, 0, 1, 0, 0, 0)`: a ship takes the
        // second arm, and a push that handles the point answers no
        // collision before the land scan (`docs/COLLISION.md` §13.2).
        let hit = if self.takes_boat_arm(u) && self.detect_boat_collision(u, target, true) {
            None
        } else {
            self.detect_unit_collision(u, target)
        };
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
        if let Some(_other) = hit
            && step.snapped
        {
            // **The snap arm's own collision block** (`005fb3bd`,
            // `docs/COLLISION.md` §5.4). `move_step` splits on
            // `param_2 < local_28` and has a collision block on *each*
            // side; this crate had only the partial step's, and spent it
            // for both. The snap's does none of what §5 lists — no
            // sidestep snap-through, no wait on an owed turn, no
            // `resolve_unit_collision`, no widened tolerance. It clears
            // the collider the probe just named (`field_0x8a = 0xffff`,
            // `field_0xb3 = 0xff` — `collide_o` and `collide_who`),
            // consumes the waypoint where the unit stands, and falls into
            // the same tail the accepted step uses.
            //
            // Great Lakes 9134 is the frame: `1/32` walks its formation
            // slot in ~24-unit hops, arrives on each within one step, and
            // on 9134 the hop it snaps to is blocked. The original stands
            // for the frame and does everything else one frame later;
            // this crate ran the give-up chain, snapped the unit back to
            // `(42792, 22824)` and ungrouped it a frame early.
            self.units[u].collide_o = -1;
            self.units[u].collide_who = -1;
            mo.has_waypoint = false;
            let popped = self.units[u].path.pop();
            self.mark(crate::anim::SITE_SNAP_BLOCKED);
            self.set_default_anim(u);
            let flags = self.current_order(u).map_or(0, |o| o.flags);
            self.store_move(u, mo, flags);
            self.units[u].movement.dest = None;
            if popped.is_none_or(|p| p.flags & path_flag::FINAL == 0) {
                return Did::Something;
            }
            self.arrive(u, mo, true);
            return Did::Something;
        }
        let mut gave_up = false;
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
                gave_up = true;
            }
        }

        let mut arrived = false;
        let mut snapped_in = false;
        if gave_up && self.world.accepts(target) {
            // **The give-up does not step** (`docs/COLLISION.md` §17).
            // `005fb7b5`–`005fb7bb` write `tolerance = local_28 * 2` and
            // `jmp 005fb82c`, past the step's `invalid_loc`,
            // `set_anim(walk)` and `set_new_location` (`005fb7d1`–
            // `005fb826`), onto the tail's arrival test itself:
            // `|dest_y − y| + |dest_x − x|` from where the unit stands,
            // `jg` over `tolerance` returns 1 (`005fb82c`–`005fb85c`), and
            // otherwise `dest = 0` and the pop (`005fb862`). This crate
            // took the step first, and run218's `1/23`, blocked 26 frames
            // by The Despot, walked 26 on 16459 where the original pops
            // its waypoint in place (Great Lakes 16460).
            let (dx, dy) = (mo.waypoint.x - from.x, mo.waypoint.y - from.y);
            arrived = dx.abs() + dy.abs() <= self.units[u].tolerance;
        } else if self.world.accepts(target) {
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

    /// A unit the `0x2400` tiles refuse: sea domain, and an `attack`
    /// (`ObjectTypeData +0x1e8`) or strictly `is(AIRCRAFTCARRIER)` — the
    /// test `find_nearby_spot@0061de70` and `UnitData::invalid_loc@
    /// 00607c30` both make (`docs/ORDERS.md` §25).
    pub(crate) fn is_warship(&self, u: usize) -> bool {
        let p = self.profile(Obj::Unit(u));
        matches!(p.domain, crate::attrition::Domain::Sea)
            && (p.attack != 0
                || self.units[u]
                    .ty
                    .is_some_and(|t| self.is_aircraft_carrier(t)))
    }

    /// `is(AIRCRAFTCARRIER, 1)` on a unit type — strict, so the type
    /// itself or its graft, never a lineage by `from`. A type the tree
    /// does not carry answers no.
    pub(crate) fn is_aircraft_carrier(&self, ty: usize) -> bool {
        let Some(t) = self.unit_types.get(ty).and_then(|u| u.tree) else {
            return false;
        };
        self.tech_tree
            .types
            .iter()
            .position(|d| d.kind.is_unit() && d.name.eq_ignore_ascii_case("Aircraft Carrier"))
            .is_some_and(|root| self.tech_tree.is(t, root, true))
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
            // row is both. Stated in §14. The squad branch (`uber_unit !=
            // 0`, the decompile's lines 52–54) is `min + 0xc0 + 4 ×
            // (((uber − 1) × guy_spacing) / 2 + big_radius)`: `come_out`'s
            // routing asks it for a squad's target with `max` −1 (item
            // 955, run338's `0/13` on 858); `go_to` passes its own `max`.
            // (`docs/audit/2026-08-21-orders.md` R7 N3.)
            max = if uber {
                min + 0xc0 + 4 * (((p.uber_size - 1) * p.guy_spacing) / 2 + p.big_radius)
            } else if p.big_radius == 0 {
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
        // **A warship keeps off the `0x2400` tiles** (`61df8b`–`61dfaf`,
        // the test at `61e3a1`): a sea type whose `attack` (`ObjectTypeData
        // +0x1e8`, the type record's name) is non-zero, or that strictly
        // `is(AIRCRAFTCARRIER)` (`0x15f`, `TypeIndex`), refuses an ocean
        // tile carrying either bit. East Indies' Trireme `1/32` is born
        // due east of Dock `1/2010` because the channel south of it is
        // `0x0420` (`docs/ORDERS.md` §25).
        let warship = matches!(p.domain, crate::attrition::Domain::Sea)
            && (p.attack != 0 || {
                let ty = match who {
                    Seeker::Unit(u) => self.units[u].ty,
                    Seeker::Type(t) => Some(t),
                };
                ty.is_some_and(|t| self.is_aircraft_carrier(t))
            });
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
                // And the warship clause: a sea type that can fight also
                // needs `!(mask & 0x2400)` — `BAD_PATH` and the unnamed
                // `0x400` (`warship` above).
                if !air {
                    let mask = self.world.tile_mask(c.tile());
                    let ocean = mask & tile::SURFACE == tile::SURFACE_OCEAN;
                    if ocean != matches!(p.domain, crate::attrition::Domain::Sea) {
                        continue;
                    }
                    if warship && mask & WARSHIP_REFUSES != 0 {
                        continue;
                    }
                }
                // The collision half, last of all: `Objects::find_collision`
                // then `Objects::find_ordered_collision`, both against
                // `(u)` as "me" (`docs/COLLISION.md` §5.2). A spot another
                // unit is standing on — or has already been sent to — is
                // taken.
                let hit = coll != Coll::None
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
                        (false, Seeker::Unit(u)) if coll == Coll::All => {
                            let r = if p.block_radius == 0 {
                                0x180
                            } else {
                                p.block_radius
                            };
                            // The seeker counts (item 955): no exemption
                            // on the position query under `FILTER_ALL`.
                            self.find_unit_with_radius(r, c, None)
                                || self.find_unit_ordered_with_radius(
                                    r,
                                    c,
                                    self.units[u].owner,
                                    Some(u),
                                )
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
    ///
    /// The approach's class is `add_move_facing_order`'s switch on
    /// `local_40`, which is 1 (`MOVE_TO`) unless the swarm is `BUILD_AT`
    /// for a computer, when it is 3 (`EXPLORE_TO`): `~(leader_flags >> 1)
    /// & 2 | 1`, `leader_flags & 4` being the human bit. So a human's
    /// builder walks under a `MOVEORDER` — run241's `0/8` taking
    /// `find_build_spot`'s help on 1097 (`docs/GOLDEN.md` §26).
    ///
    /// SEAM: a computer's `REPAIR` swarm is `MOVE_TO` in the original and
    /// an `EXPLORE_TO` here; the AI repairs on both long captures, so the
    /// change is held until both words are measured with it.
    pub(crate) fn swarm_around(&mut self, u: usize, b: usize, body: Body, action: bool) {
        let building = matches!(body, Body::Build(_));
        let kind = if self.nation[self.units[u].owner as usize].human {
            MoveKind::MoveTo
        } else {
            MoveKind::ExploreTo
        };
        if let Some((spot, facing)) = self.swarm_spot(u, b, building) {
            self.add_move_facing_order(u, spot, kind, QueuePos::First, false, facing, None, false);
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

    /// `Group::action_swarm_around`'s member arm at **`QUEUE_LAST`**
    /// (`0070fbe0`, `docs/GROUPS.md` §24): the approach move appended, and
    /// the build or repair order appended behind it, so both run after
    /// whatever the member already holds.
    ///
    /// Three things differ from [`Self::swarm_around`]'s `QUEUE_FIRST`
    /// shape. **Nothing is queued when the ring finds no spot**: the move
    /// and the order are both inside the `find_nearby_spot == 0` arm. The
    /// move's kind is the caller's (`local_40`). And the move is
    /// **`QUEUE_NEW`** when the member's action is a gather and the
    /// order carries the action bit (`00710487`), which clears the list
    /// before the approach goes in.
    pub(crate) fn swarm_around_last(
        &mut self,
        u: usize,
        b: usize,
        body: Body,
        action: bool,
        kind: MoveKind,
        pos: QueuePos,
    ) {
        let building = matches!(body, Body::Build(_));
        let Some((spot, facing)) = self.swarm_spot(u, b, building) else {
            return;
        };
        let gathering = self
            .action_of(u)
            .is_some_and(|i| self.units[u].orders[i].index() == index::GATHER);
        let pos = if pos == QueuePos::Last && gathering && action {
            QueuePos::New
        } else {
            pos
        };
        self.add_move_facing_order(u, spot, kind, pos, false, facing, None, false);
        // SEAM: `is_castable(0x293)` and its `add_cast_order` ahead of the
        // order, and `BUILD_AT`'s clear of the site's `+0x60 & 0x2000`.
        // The spell is castable by no unit a capture on file stages.
        match body {
            Body::Repair(_) => self.add_repair_order(u, b, QueuePos::Last, action),
            _ => self.add_build_order(u, b, QueuePos::Last, action),
        }
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

    /// `Unit::kill_garrison_order(captain, 0)@005e2bd0`: from the squad's
    /// captain down its `o_down` chain, each unit whose action is a
    /// GARRISON is `repath`ed — its leading moves go — and has its head
    /// killed, once. The walk stops at the first link that is not active.
    /// run208's squad goes in on 761 through `0/7`, and `0/8` and `0/9`
    /// go in with their walks and their GARRISONs gone too.
    fn kill_garrison_order(&mut self, u: usize) {
        let mut v = self.captain_of(u);
        for _ in 0..self.units.len() {
            if self
                .action_of(v)
                .is_some_and(|a| matches!(self.units[v].orders[a].body, Body::Garrison { .. }))
            {
                self.repath(v);
                self.kill_current_order(v);
            }
            match self.units[v].o_down {
                Some(next) if self.units[next].alive() => v = next,
                _ => return,
            }
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
        // **Every call drops the gatherer's group**, a computer's too
        // (`5f0237` `orl $-1`, `5f023e` `movw %ax, 0x80(%ebx)`: both
        // paths from the entry join there, before the `goto_build`
        // branch). The pool keeps listing the unit; only the back-pointer
        // goes, as in `think_peasant`'s human arm. On East Indies' block
        // 8369 it is `1/11`'s `group` 65 → −1, the first pool parting
        // once the AI's building groups number the pool
        // (`docs/GROUPS.md` §29).
        self.units[u].group_ptr = None;
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
            // **A tiny mountain caps the distance weight at 3**
            // (`005f0170`, the `local_30` clamp before the loop): when the
            // list is a range's (`MiningList::mtn ≥ 0`, a Mine) and holds
            // fewer tiles than `MTN_TINY_SIZE`, a Mine's 10 scores as 3,
            // so the list's order (`i >> 2`) weighs against the distance.
            // East Indies' `1/11` on 10958 is the case: 46 tiles, and the
            // cap sends it to (170, 182) where 10 sent it onto `1/6`'s
            // (173, 183) — item 620, `docs/ORDERS.md` §6.4.
            let tiny = crate::gather::MOUNTAIN_GATHER[0].0;
            let dm = if self.building_ident(b) == Ident::Mine
                && (self.buildings[b].gather_from.len() as i32) < tiny
            {
                g.dist_mod.min(3)
            } else {
                g.dist_mod
            };
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
        // **`do_attack`'s own gate is the type's `attack`, and it is the
        // only aliveness test the function has** (item 463).
        // `Unit::do_attack@005f1b80` tests the target object's `flags & 1`
        // in exactly two places — the strafe arm and the grouped arm —
        // and *both* sit under `*(int *)(ptype + 0x1e8) == 0`, which the
        // type record names `attack`. A unit whose type can attack falls
        // through both to `LAB_005f2224`'s unconditional `fight(...)`
        // without ever asking whether its target still exists. So the
        // validity kill below is `fight`'s, not this function's, and it
        // sits where `fight` puts it: **behind the reload gate**.
        if self.attack_of(me) == 0 {
            self.kill_current_order(u);
            return;
        }
        let state = self.units[u].combat;
        // **A mandatory order ignores the stance.** `Unit::do_attack@
        // 005f1b80:225`-`244`: the order's `+0x1c` (mandatory) jumps
        // straight to `LAB_005f2224`, which is the unconditional
        // `fight(...)` — every stance test in the function sits under
        // `mandatory == 0`. The probe of `docs/COMBAT.md` §17 sets
        // `action_stance(5)` — HOLD_FIRE — on the six units it then
        // gives a **mandatory** attack order to, and run19's block 8187
        // has all six chasing, so this arm is what lets them.
        if state.stance == combat::Stance::HoldFire && !state.mandatory {
            return;
        }
        // **The cell-centre snap at `fight`'s own entry** (§7.11).
        // `Unit::fight@005fd4d0`, `LAB_005fd648`:
        // `set_new_location(this, div_3_table[x >> 4] * 0x30 + 0x18, …, 1,
        // 0)` — the unit is put on its 48-unit cell centre before the
        // function does anything else. §7.11's table has four rows and
        // this is the first: the snap is skipped **only** by a recharging
        // unit that is neither §7.10's re-entry latch on an invalid target
        // nor a cavalry archer (`unit_flags & 0x400`), and those two rows
        // are unmodelled seams, not coverage — the reload gate below
        // returns for *every* recharging unit. Nothing on disk has a
        // recharging unit enter `fight`, which is what a capture would
        // have to carry to refuse them (item 336).
        if state.recharging == 0 {
            let snap = crate::collide::ucell_centre(crate::collide::ucell(self.units[u].pos));
            self.set_new_location(u, snap, true);
        }
        // **The reload gate**, `Unit::fight@005fd4d0:100-127`. A recharging
        // unit returns, and it does not turn. On a **fresh** order
        // (`new_ord`, the order's `+0x20`) whose target guy 0 is not
        // already aimed at (`GuyData +0x8e`/`+0x9f`), and whose guy 0 is
        // not on a slot below 4, it first asks for the idle:
        // `Unit::set_anim(CHAR_DEFAULT, 1, 1)`, which is the idle roll
        // (`Guy::set_anim+0x97a`).
        //
        // ~~Turns toward the target and returns~~ — the first reading's
        // (the orders landing), and never diffed until item 530. Golden
        // chapter one's `1/7` is the witness. It takes an attack on `0/8`
        // from its attack-move's look on 773 while recharging 13. On 774
        // it stands with its heading unchanged and rolls the idle, and
        // that roll is the chapter's word (`docs/ORDERS.md` §22).
        if state.recharging != 0 {
            if a.new_ord
                && let Some(target) = self.units[u].combat.target
                && let Some(g0) = self.units[u].guys.first()
                && g0.aim != Some(target)
                && !(0..4).contains(&g0.anim)
            {
                self.mark(anim::SITE_RELOAD_IDLE);
                self.set_anim(u, anim::DEFAULT, true, true);
            }
            return;
        }
        // **`fight`'s own target test, and it is downstream of the gate
        // above** — `Unit::fight@005fd4d0:196`'s `Object::valid_target`,
        // which the recharging arm at `:102` returns before reaching.
        // `Unit::do_attack@005f1b80` reads **no** order flag — grep it —
        // so the `flags & 0x10` half this test used to carry was nobody's
        // reading and, since nothing wrote the bit, a clause that could
        // not fire. Item 329 gave the bit its real writer
        // ([`flag::FIGHT_REENTRY`]) and its real readers, both in `fight`.
        //
        // SEAM: `fight`'s head does reach the validity question while
        // recharging, through `local_1c` — the re-entry latch, which
        // needs `UnitType +0x2b8 & 1` *and* an invalid target — and that
        // row of §7.11's table is unmodelled. It only ever makes the kill
        // **earlier**, and no capture on disk has a unit in it.
        // **The follower takes its captain's target here, before the
        // validity test — and that is how two of three bowmen strike on
        // the frame the third spends searching** (item 502,
        // `docs/COMBAT.md` §43.2).
        //
        // `Unit::fight@005fd4d0`, the block between the cell-centre snap
        // and `:196`'s `Object::valid_target`. It runs for a unit that is
        // **not** a captain (`vtable+0xe8`, `o_up >> 15`) whose
        // `collide_frame` is older than the frame before this one, reads
        // the captain's **action** through `UnitData::get_captain`
        // (`+0xe4`, which recurses to the head of the chain) and
        // `get_action`, and takes its `(ox, whom, uid)` when all of:
        //
        // - the captain's action is an ATTACK;
        // - its target is not the one this unit already names;
        // - `Object::valid_target` passes **on this unit**;
        // - the unit's combat stance is below `STAND_GROUND`, or the
        //   captain's target is already in range.
        //
        // It writes the pair into the **existing** order rather than
        // adding one, and the dump says so: run112's `0/7` and `0/8`
        // carry `ox 6 whom 1 uid 12` at block 696 with `new_ord 0` and
        // `ever_in_range 1` — the same order object, retargeted — where
        // their captain `0/6` carries the same target with `new_ord 1`
        // and `ever_in_range 0`, which is what a fresh `add_attack_order`
        // leaves. Those two fields are the value diff that tells the two
        // mechanisms apart, and they were on disk before this landing.
        //
        // This is a **second** captain mirror, and it is not
        // [`Self::captain_mirror`]: that one is `Unit::think`'s opening
        // arm (§21), runs only on an idle unit and adds a `QUEUE_NEW`
        // order. This one runs on the attack step of a unit that already
        // has an order, and it is why a squad does not lose a frame each
        // time its captain retargets.
        if !self.units[u].captain && self.units[u].collide_frame < frame - 1 {
            let cap = self.squad_captain(u);
            if cap != u
                && let Some(k) = self.action_of(cap)
                && matches!(self.units[cap].orders[k].body, Body::Attack(_))
                && let Some(t) = self.units[cap].combat.target
                && Some(t) != self.units[u].combat.target
                && self.valid_target(me, t)
                && (matches!(
                    state.stance,
                    combat::Stance::Aggressive | combat::Stance::Defensive
                ) || self.is_in_range(me, t))
            {
                self.units[u].combat.target = Some(t);
            }
        }
        let Some(target) = self.units[u].combat.target else {
            self.kill_current_order(u);
            return;
        };
        if !self.valid_target(me, target) {
            // `find_new_target`: the idle search with the order dropped.
            self.kill_current_order(u);
            if let Some(t) = self.find_melee_target(u, -1) {
                self.add_attack_order(u, t, QueuePos::First, false, false);
            }
            // **The frozen frame's mark** (item 502, `docs/COMBAT.md`
            // §43.2). `Unit::fight@005fd4d0`'s invalid-target branch ends
            // at `LAB_005fdb9e`, and its tail is
            //
            //     find_new_target(this, NULL, 0)
            //     leaders[who].searches += 1
            //     if (order_type() != ATTACK)                 return 0
            //     if (recharging != 0 && !reentry_latch)       return 0
            //     unit_masks2 |= 0x10
            //
            // — so a unit whose target went invalid does **not** strike
            // this frame, and it carries [`combat::umask2::NOT_FIRING`]
            // out of the order step, which stops its figures' animation
            // clocks in phase 7 (`docs/ANIM.md` §5). `recharging` is
            // zero here by the reload gate above, so the only condition
            // left is that the search put an attack order back in front:
            // a unit that finds nothing has no order to be "still
            // ordered" under and takes no mark.
            //
            // run112's chapter-two word is this arm: `0/6` retargets to
            // `1/6`, freezes, and spends **no** attack-end wrap on 695
            // where this crate spent one — 21 draws against 20.
            //
            // The count is [`Sim::retargets`], `LeaderData +0x9f4`
            // (item 668): `Leader::process@006b88b0` zeroes it at the
            // head of each leader's frame, and `resolve_unit_collision`'s
            // enemy ladder reads `< 10` (`docs/COLLISION.md` §14).
            //
            // SEAM: the **search budget** it feeds here. `LAB_005fdb9e`'s
            // first arm is `waiting < 5 && retargets > 10`, which sets
            // the same bit, adds 2 to `UnitData::waiting` and returns
            // *without* searching or counting. It is not modelled, and
            // run112 says it did not fire there: `0/6`'s `waiting` is 0
            // on block 696, where the throttle would have left 2.
            let who = usize::from(self.units[u].owner);
            self.retargets[who] += 1;
            if matches!(
                self.current_order(u).map(|o| &o.body),
                Some(Body::Attack(_))
            ) {
                self.units[u].unit_masks2 |= combat::umask2::NOT_FIRING;
            }
            return;
        }
        // **A guard's attack is leashed to its post** (`docs/COMBAT.md`
        // §63). `Unit::fight@005fd4d0`, `005fdc91`–`005fdd4f`, straight
        // after `valid_target` passes and ahead of both captain arms below:
        // when the unit's **activity** is a `GUARD` (`get_activity`, type
        // `0xc`), it asks `check_target(o, who, 1, NULL, 1, use_poor, 0)`,
        // whose guarding arm refuses a target — or a guard — further than
        // `unit_guard_respond_range × 2 × 0x60` from the post. Refused, the
        // attack is killed and `find_melee_target(−1, NULL, 0, 1, 0)` runs
        // the guard's own search, which adds what it finds at QUEUE_FIRST;
        // a unit left under an `ATTACK` and not recharging carries the
        // frozen mark. A captain with `unit_masks & 0x40000` first rolls
        // (`fight+0x824`) and drops the attack on an odd draw, and asks
        // with `use_poor`.
        //
        // Chapter eleven is the witness (run190): the chariot `0/6` shot
        // `1/6` from its post on 1011, `1/6` walked off on its army's
        // order, and on tick 1036, its first unrecharged `fight`, `1/6`
        // stood ≈1,780 from the post against a leash of 1,536. The guard
        // drops the attack with no draw, and its search, leashed too,
        // writes `near` −1.
        if let Some(g) = self.guard_activity(u) {
            let mut use_poor = false;
            let mut drop = false;
            if self.units[u].captain && self.ai_driven(self.units[u].owner) {
                use_poor = true;
                self.mark(crate::fight::SITE_FIGHT_GUARD_ROLL);
                drop = self.rng.roll() & 1 != 0;
            }
            if drop || !self.guard_check_target(u, &g, target, use_poor) {
                self.kill_current_order(u);
                if let Some(t) = self.find_melee_target(u, -1) {
                    self.add_attack_order(u, t, QueuePos::First, false, false);
                }
                if self.order_type(u) == index::ATTACK {
                    self.units[u].unit_masks2 |= combat::umask2::NOT_FIRING;
                }
                return;
            }
        }
        // **A captain's attack on a building re-searches every frame**
        // (`docs/COMBAT.md` §62). The same captain arm as the one-in-five
        // below, `Unit::fight@005fd4d0`, `LAB_005fddf7`: when the target
        // is not a unit (vtable `+0x18`, `005fdeb1`-`005fdeb6` → `005fdf50`)
        // there is no roll and no `poor_target`, only `find_new_target(
        // this, &who, 0)` at `005fdeea` — the order killed and the idle
        // search run again. Nothing found, and the attack is gone
        // (`LAB_005fe001`); another target, and the search's own order
        // stands with the frozen mark (`005fdf85`-`005fdfea`); the same
        // one, and `fight` goes on with the order the search added, fresh.
        // run177's packet: the Fighter `0/6` on tick 632 of chapter six-b.
        //
        // SEAM: the retarget arm's early return when the search names
        // `fight`'s own entry arguments (`local_20`/`local_24`) after the
        // target was changed above; nothing here changes it in between.
        if !state.mandatory
            && state.captain == i32::from(self.units[u].index)
            && let Obj::Building(_) = target
        {
            match self.find_new_target(u, false) {
                None => return,
                Some(f) if f != target => {
                    if matches!(
                        self.current_order(u).map(|o| &o.body),
                        Some(Body::Attack(_))
                    ) {
                        self.units[u].unit_masks2 |= combat::umask2::NOT_FIRING;
                    }
                    return;
                }
                Some(_) => {
                    let Some(Order {
                        body: Body::Attack(fresh),
                        ..
                    }) = self.current_order(u).copied()
                    else {
                        return;
                    };
                    a = fresh;
                }
            }
        }
        // The one-in-five re-search (`docs/COMBAT.md` §8.2 step 0).
        if !state.mandatory
            && state.captain == i32::from(self.units[u].index)
            && let Obj::Unit(t) = target
        {
            self.mark(crate::fight::SITE_FIGHT_RESEARCH);
            let roll = self.rng.roll();
            // `Unit::fight@005fd4d0`, `005fde80`-`005fde99`: the draw is
            // spent first and **then** the two suppressions are read —
            // `roll % 5 == 0`, or the current order carrying the chase
            // latch. So a latched order costs the draw and skips the
            // search, which is what keeps the frame's count right.
            if !self.profile(Obj::Unit(t)).combat_role
                && roll % 5 != 0
                && flags & flag::FIGHT_REENTRY == 0
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
            // **Siege fire at a unit is ground fire** (`docs/COMBAT.md`
            // §57.2): the arm returns before the strike's tail, so the
            // attack keeps `new_ord 1` under the order it pushes, as
            // run146's `0/6` does from 781 to 863.
            if self.siege_ground_arm(u, target, a, frame) {
                return;
            }
            a.new_ord = false;
            self.store_attack(u, a);
            self.fight_pub(u, target, frame);
            return;
        }
        self.store_attack(u, a);
        // **An unpacked packer that has been in range dies out of it**
        // (`Unit::fight@005fd4d0:496`, `LAB_005fe0e5`): `unit_flags2 & 4`,
        // not packed, and the order's `ever_in_range` → kill, before the
        // stance and the chase. It is how run146's catapult loses its
        // attack on 864, the frame its ground order ends: the hoplites
        // have walked inside the three-tile minimum (§57.3).
        let packs = self.units[u]
            .ty
            .is_some_and(|t| self.unit_types[t].combat.packs);
        if packs && !self.units[u].combat.packed && a.ever_in_range {
            self.kill_current_order(u);
            return;
        }
        // **A packer re-searches before it chases** (`fight:1051`,
        // `docs/COMBAT.md` §60.3): `find_new_target(this, &who, 0)`
        // kills the attack and runs the idle search, which adds the
        // order it finds, and the tail goes on only with a target in
        // hand. An unpacked packer's search refuses what it cannot
        // reach (`find_nearby_target`'s `local_24`), so a catapult whose
        // attacker stands inside its minimum drops the attack and stays
        // put: run146's `0/6` on 868, 870 and 871.
        let target = if packs {
            self.kill_current_order(u);
            let Some(t) = self.find_melee_target(u, -1) else {
                return;
            };
            let pos = if state.stance == combat::Stance::Defensive {
                QueuePos::First
            } else {
                QueuePos::New
            };
            self.add_attack_order(u, t, pos, false, false);
            t
        } else {
            target
        };
        if state.stance == combat::Stance::StandGround {
            return;
        }
        // **The chase** (`docs/COMBAT.md` §17): the unit asks where to
        // stand — `Unit::fight@005fd4d0+0xcb4`'s arm, through the
        // seven-argument thunk that fills the last two arguments with the
        // caller's own position — and the move goes in front, to run from
        // the next frame. A ring walk that finds nothing, and every
        // target the ring does not cover, falls back to the target's own
        // position, which is what this crate did for all of them until
        // item 328.
        let here = self.units[u].pos;
        let dest = match self.find_attack_pos(u, target, here, crate::fight::SITE_ATTACK_POS_FIGHT)
        {
            Some(p) => p,
            None => self.pos_of(target),
        };
        self.add_move_order(u, dest, MoveKind::MoveTo, QueuePos::First, false);
        self.chase_reentry(u, here, dest, frame);
    }

    /// **The chase runs in the frame it is ordered** — `Unit::fight@
    /// 005fd4d0`'s tail, `005fe395`-`005fe3e7`, and `docs/ORDERS.md`
    /// §7.10.
    ///
    /// `add_move_order` has just put the chase in front of the attack.
    /// The original does not then leave the unit standing: it re-enters
    /// **`Unit::work`** through vtable `+0x188`, so the move is dispatched,
    /// planned and — when the plan is short enough — walked on this same
    /// tick. `docs/ORDERS.md` §7.10 has the value diff: run19's block 8187
    /// carries all six of the probe's units with a **full path stack**
    /// (43, 43, 43, 94, 47, 46 entries) at the end of sim-frame 8186, the
    /// frame the chase was ordered, and this crate planned nothing until
    /// 8187.
    ///
    /// Two gates, both from the listing:
    ///
    /// - `005fe395`-`005fe3b1`: two `je`s out, one per coordinate. The
    ///   re-entry happens only when the chosen spot differs from the
    ///   unit's own position in **both** x and y — matching either one
    ///   returns.
    /// - `005fe3b7`-`005fe3e7`: the action order's [`flag::FIGHT_REENTRY`].
    ///   Clear, it is set and `work` is called; set, it is cleared and
    ///   nothing recurses. That is the whole recursion guard, and it
    ///   leaves the latch **standing** on the ordinary path, because the
    ///   nested call dispatches the new move rather than `fight`.
    fn chase_reentry(&mut self, u: usize, here: Pos, dest: Pos, frame: i64) {
        if dest.x == here.x || dest.y == here.y {
            return;
        }
        let Some(a) = self.update_action(u) else {
            return;
        };
        if self.units[u].orders[a].flags & flag::FIGHT_REENTRY == 0 {
            self.units[u].orders[a].flags |= flag::FIGHT_REENTRY;
            self.work(u, frame);
        } else {
            self.units[u].orders[a].flags &= !flag::FIGHT_REENTRY;
        }
    }

    /// **`Unit::fight@005fd4d0`'s siege arm** (`fight:496`–`572`,
    /// `docs/COMBAT.md` §57.2), in the attack's in-range branch and
    /// ahead of the strike. For an attacker whose type packs
    /// (`unit_flags2 & 4`) and is not the Dutch merchant:
    ///
    /// - packed, the original unpacks here or moves to a better spot.
    ///   SEAM: not carried. No human's packed engine holds an attack
    ///   (its think returns before the search, §51.1), and no capture on
    ///   disk has a computer's packed engine in range of one; this crate
    ///   falls through to the strike as it always has.
    /// - unpacked, a **unit** target (the target's vslot `+0x18`,
    ///   `is_unit`) and a **siege** type (vslot `+0x10c`, `is_siege`):
    ///   `set_attacking`, then an `ATTACK_GROUND` order at the head
    ///   holding the target's own position, `accuracy = (domain == sea)`
    ///   and `attack_unit = 2`, with the fired and action bits clear;
    ///   `clear_partial_path`, the head rotated onto it, `update_action`,
    ///   and **`Unit::work`** (vslot `+0x188`), so the new order fires on
    ///   this same frame.
    ///
    /// The attack's `in_range` and `ever_in_range` are written before it
    /// (`fight:491`–`493`) and its `new_ord` is left standing: the strike
    /// tail that clears it (`fight:847`) is never reached.
    fn siege_ground_arm(&mut self, u: usize, target: Obj, a: AttackOrder, frame: i64) -> bool {
        let Some(t) = self.units[u].ty else {
            return false;
        };
        if !self.unit_types[t].combat.packs
            || self.units[u].combat.packed
            || self
                .unit_tree(u)
                .is_some_and(|ti| self.tech_tree.is(ti, MERCHANTDUTCH, true))
        {
            return false;
        }
        let Obj::Unit(v) = target else {
            return false;
        };
        if !self.profile(Obj::Unit(u)).siege {
            return false;
        }
        self.set_attacking(u, self.owner_of(target));
        self.store_attack(u, a);
        let at = self.units[v].pos;
        let sea = matches!(self.profile(target).domain, crate::attrition::Domain::Sea);
        let order = Order {
            flags: 0,
            body: Body::AttackGround(AttackGroundOrder {
                at,
                sea,
                attack_unit: 2,
            }),
        };
        self.enqueue(u, order, QueuePos::First);
        self.work(u, frame);
        true
    }

    /// **`Unit::do_attack_ground@005f1410`** — one frame of an
    /// `ATTACK_GROUND` order (`docs/COMBAT.md` §57.3, `docs/ORDERS.md`
    /// §26), in the original's order:
    ///
    /// 1. `can_attack_ground`, else kill. A player's order (`attack_unit
    ///    0`) is also killed on a point at peace with the owner, and walks
    ///    into range; SEAM: no command here issues one, so both arms are
    ///    the kill.
    /// 2. The facing, `find_angle` to the point, a quarter turn off for a
    ///    broadside type (`unit_flags & 0x40`).
    /// 3. **Reloading**: a fired order holds the unit still (`flags &
    ///    0x80`, the byte read signed); otherwise a figure off slots 0–3
    ///    rolls the idle, as `fight`'s own reload gate does.
    /// 4. **`attack_unit`**: 1 is the ready frame after the shot, and it
    ///    kills the order and re-enters `Unit::work` (vslot `+0x188`), so
    ///    the attack beneath runs on this frame; 2 becomes 1 and fires.
    /// 5. `set_attack(−1, −1)`: the aimed figures forget their target. A
    ///    packed packer unpacks instead of firing.
    /// 6. `unit_masks |= 0x11000` (SEAM: this crate keeps neither bit),
    ///    the turn, `flags |= 0x80`, the swing (`fight`'s own four-arm
    ///    choice, two arms of which exist here), the shot, and **`recharging
    ///    = recharge() + 1`**: run146's catapult reads 83 on the push where
    ///    its direct strike at a building reads 82 (run44 `0/15`, 408).
    pub(crate) fn do_attack_ground(&mut self, u: usize, frame: i64) {
        let Some(Order {
            body: Body::AttackGround(mut g),
            flags,
        }) = self.current_order(u).copied()
        else {
            return;
        };
        if !self.can_attack_ground(u) || g.attack_unit == 0 {
            self.kill_current_order(u);
            return;
        }
        let from = self.units[u].pos;
        let direct = crate::movement::find_angle(g.at.x - from.x, g.at.y - from.y);
        let angle = self.broadside_angle(u, direct);
        if self.units[u].combat.recharging != 0 {
            if flags & flag::FIRED != 0 {
                return;
            }
            if self.units[u]
                .guys
                .first()
                .is_some_and(|g0| (0..4).contains(&g0.anim))
            {
                return;
            }
            self.mark(anim::SITE_RELOAD_IDLE);
            self.set_anim(u, anim::DEFAULT, true, true);
            return;
        }
        if g.attack_unit == 1 {
            self.kill_current_order(u);
            self.work(u, frame);
            return;
        }
        g.attack_unit = 1;
        if let Some(front) = self.units[u].orders.front_mut() {
            front.body = Body::AttackGround(g);
        }
        self.clear_attack(u);
        if self.units[u]
            .ty
            .is_some_and(|t| self.unit_types[t].combat.packs)
            && self.units[u].combat.packed
        {
            self.add_cast_order_at(u, spell::UNPACK, QueuePos::First);
            return;
        }
        if angle != self.units[u].movement.heading {
            self.unit_set_angle(u, angle);
        }
        if let Some(front) = self.units[u].orders.front_mut() {
            front.flags |= flag::FIRED;
        }
        self.swing_anim_pub(u, direct);
        self.fire_ground(u, g, direct, frame);
        let r = self.reload_frames(u);
        self.units[u].combat.recharging = r.wrapping_add(1);
    }

    /// **`UnitTypeData::can_attack_ground@0061db20`**, from the listing
    /// (`61db20`–`61dc13`): the rush-rules age gate (SEAM: this crate
    /// plays without rush rules), a `max_range` (`+0x1fc`), neither of
    /// `unit_flags & 0x102000`, and then any of `obj_masks` EXPLOSIVE,
    /// SIEGE, NAVAL or BOMBARD, or the machine-gun lineage.
    pub(crate) fn can_attack_ground(&self, u: usize) -> bool {
        let Some(t) = self.units[u].ty else {
            return false;
        };
        let ty = &self.unit_types[t];
        let p = ty.combat;
        if p.max_range == 0 || ty.cols.flag(0x10_2000) {
            return false;
        }
        use crate::combat::mask;
        p.has(mask::EXPLOSIVE | mask::SIEGE | mask::NAVAL | mask::BOMBARD)
            || self.unit_line_is(u, MACHINEGUN)
    }

    fn store_attack(&mut self, u: usize, a: AttackOrder) {
        if let Some(front) = self.units[u].orders.front_mut()
            && let Body::Attack(x) = &mut front.body
        {
            *x = a;
        }
    }

    /// `Unit::find_new_target(this, NULL, stand)@005ff6a0`, in the one
    /// shape this crate calls it: from `resolve_unit_collision`'s enemy
    /// ladder, `stand` 1 (`docs/COLLISION.md` §14, the listing at
    /// `005f9ffb`–`005fa001` pushes `1, 0`).
    ///
    /// 1. `repath`: the leading transit legs go.
    /// 2. The current order goes too: `kill_current_order`, or for a
    ///    group order `Group::kill_group_order`.
    /// 3. With `stand`, the unit's stance (`+0xb1`) reads **2,
    ///    `STAND_GROUND`**, across `find_melee_target(-1, NULL, 0, 1, 0)`
    ///    and is put back after. That is the whole of what `stand` does,
    ///    and it does two things inside the search: `find_nearby_target`'s
    ///    `local_24` makes every candidate pass `is_in_range` from where
    ///    the unit stands, and the order it adds is `QUEUE_NEW`, because
    ///    the stance it reads is not `DEFENSIVE` (`00649b6d`).
    ///
    /// So a captain bumped by an enemy that is not its target drops the
    /// walk and the old attack where it stands, and takes whatever it can
    /// strike from there. run171's `1/6` on 658: `0/7`.
    ///
    /// SEAM: the defensive-post arm (`bVar3`, a `DEFENSIVE` unit whose
    /// type has stances and whose `find_def_pos` answers), which writes
    /// the post into the found attack or walks back to it; and the group
    /// order's `kill_group_order`, which this crate has no group attack
    /// order for. Neither is reached by `1/6`, `stance 0` and not grouped
    /// in its order.
    pub(crate) fn find_new_target(&mut self, u: usize, stand: bool) -> Option<Obj> {
        self.repath(u);
        self.kill_current_order(u);
        let stance = self.units[u].combat.stance;
        if stand {
            self.units[u].combat.stance = combat::Stance::StandGround;
        }
        let found = self.find_melee_target(u, -1);
        if let Some(t) = found {
            // `find_nearby_target`'s add (`00649b3a`–`00649bc0`): an
            // attack-move in front keeps its place under a `QUEUE_FIRST`
            // attack; otherwise `QUEUE_FIRST` only for a `DEFENSIVE`
            // stance, and `QUEUE_NEW` for every other. SEAM: a unit whose
            // activity is a `GUARD` (`local_2c`) kills an attack-move in
            // front and adds `QUEUE_FIRST`, and a group's attack-move
            // may hand the target to `Group::action_attack` instead.
            let pos = if matches!(
                self.order_type(u),
                index::ATTACK_TO | index::GROUP_ATTACK_TO
            ) || self.units[u].combat.stance == combat::Stance::Defensive
            {
                QueuePos::First
            } else {
                QueuePos::New
            };
            self.add_attack_order(u, t, pos, false, false);
        }
        self.units[u].combat.stance = stance;
        found
    }

    /// A found better target rewrites the order's target in place.
    fn retarget_attack(&mut self, u: usize, t: Obj) {
        let unit = &mut self.units[u];
        unit.combat.target = Some(t);
        unit.combat.mandatory = false;
    }
}

#[cfg(test)]
mod loose_tests {
    use super::*;

    fn at(tolerance: i32, flags: u8) -> PathData {
        PathData {
            to: Pos::new(0, 0),
            tolerance,
            flags,
        }
    }

    /// The unwind's test, both ends of the range: a tolerance-0 point is
    /// kept, `0x60` is unwound, and `0x61` and the world grid's `0x180`
    /// are kept.
    #[test]
    fn a_tolerance_0_waypoint_is_not_loose_and_0x60_is() {
        assert!(!is_loose(&at(0, 0)), "an exact point is kept");
        assert!(is_loose(&at(1, 0)));
        assert!(is_loose(&at(0x60, 0)), "half a tile is loose");
        assert!(!is_loose(&at(0x61, 0)));
        assert!(!is_loose(&at(0x180, 0)));
        assert!(!is_loose(&at(0x60, path_flag::FINAL)), "a final is kept");
        assert!(!is_loose(&at(0x60, 0x20)));
        assert!(!is_loose(&at(-1, 0)), "unsigned: a negative is kept");
    }
}
