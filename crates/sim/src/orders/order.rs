//! The order types: the per-kind bodies, `Order` itself, the flag and index tables, and the move-order constructor (STRUCTURE §5, move 7).

use super::*;

/// `OrderIndex` — the value the gamelog's `type` line carries.
pub mod index {
    pub use crate::ids::order::*;

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

    /// **The air family**, `is_air@0046f000`: `STRAFE`, `AIR_PATROL` and
    /// `AIR_ATTACK_GROUND` (`0x10`, `0x11`, `0x18`). `Ammo::init`'s lead
    /// asks it beside [`is_move_family`] (`docs/COMBAT.md` §74).
    pub const fn is_air_family(kind: u8) -> bool {
        matches!(kind, STRAFE | AIR_PATROL | AIR_ATTACK_GROUND)
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
    pub use crate::ids::spell::*;

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
    /// `SpellTypeData::duration` and `duration_upgrade` (`+0x1d4`,
    /// `+0x1d8`), in frames: the row's `DURATION` and `DURATION_UPGRADE`
    /// seconds × 15, as `SpellType::init@00674a80` stores them.
    pub duration: i32,
    pub duration_upgrade: i32,
    /// `COST2`, the cast's price per good as the file writes it
    /// (`SpellType::init@00674a80`'s second `Type::load_cost`, into
    /// `+0x1e0`, unscaled): `TypeData::get_cost@00664090`'s spell arm
    /// multiplies it by `SPELL_COST_FACTOR`. The declaration's crafts are
    /// `8g` each (`crate::leaders::declare`).
    pub cost2: [i32; crate::economy::RESOURCES],
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
/// literal here too rather than a [`crate::core::tuning::Tuning`] slot. Nothing in
/// this simulation pays the bonus yet, so this term only ever moves the
/// *choice* of building, never the income.
pub(super) const DUTCH_INTEREST_HEADROOM: i32 = 0x640;

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
    /// `add_move_facing_order@005e55c0`'s `param_11` and
    /// `add_group_move_order`'s `param_17` (`docs/ORDERS.md` §4.3: the
    /// group move's "disembark" argument). Nothing in this crate sets it.
    pub const DISEMBARK: u8 = 0x20;
    /// **An attack-ground order has fired** (`docs/ORDERS.md` §1.3).
    /// `Unit::do_attack_ground@005f1410` sets it on the shot, and while
    /// the reload runs a set bit holds the unit still rather than
    /// letting it roll the idle; `add_attack_ground_order` and `fight`'s
    /// siege arm clear it on a new order. The dump prints it as the
    /// signed byte `flags -128`.
    pub const FIRED: u8 = 0x80;
}

/// Which arm of `Unit::work`'s pack test a unit took
/// ([`Sim::pack_before_move`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PackArm {
    /// Not an unpacked packer under a move: the sixteen-frame review runs.
    Not,
    /// Arm 1, the point inside the block radius: the move is re-aimed at
    /// the unit's own position and runs.
    Held,
    /// Arm 2, already in the point's cell: the order died and `work` ends.
    Killed,
    /// Arm 3: a pack cast went on top, and it is what the dispatch runs.
    Packing,
}

/// What `find_melee_target`'s squad head answers ([`Sim::melee_squad_head`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SquadHead {
    /// A captain, or a follower whose captain's target has gone: search.
    Search,
    /// A follower whose captain is not attacking: `−1`, nothing.
    Nothing,
    /// A follower taking its captain's target, and its `mandatory`.
    Captain(Obj, bool),
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
    /// unit-grid search** (`crate::movement::path`, `docs/PATHFINDER.md` §21).
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
    pub const BOARDS_TRANSPORT: u8 = 0x4;
    /// A `go_around_building` mid-detour point — the turn-in corner between
    /// the lane the unit is in and the one it is stepping into. It
    /// suppresses the collision test entirely (`docs/ORDERS.md` §4.1).
    pub const DETOUR: u8 = 0x8;
    /// **This waypoint crosses a building's footprint** — `astar_path`'s
    /// tile grid sets it on a node over a blocked building tile, which
    /// only an armed walker's search admits, and only for a building not
    /// its owner's (`docs/PATHFINDER.md` §7). `do_move` asks
    /// `Unit::resolve_block` on it (`docs/ORDERS.md` §4.4).
    pub const BLOCK: u8 = 0x10;
    /// **A road was laid on this waypoint's tile.** Only
    /// `Caravan::build_road@0073db10` writes it, on every node of a trade
    /// route's plan that is not open water, and the road stack carries it
    /// into the unit's own path when a leg starts (`docs/CARAVAN.md` §5.3).
    /// ~~Nothing in the executable reads it back.~~ `Unit::do_move@005f7b30`
    /// does, on the waypoint a caravan takes: a road tile that has been
    /// built over verifies the route (`docs/CARAVAN.md` §10, item 695).
    pub const ROAD_LAID: u8 = 0x20;
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
/// a seam (`crate::groups::group`), so no building is ever guarded here, and the
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

/// The fields of `AirAttackGroundOrder` (item 1050): the
/// `ATTACKGROUNDORDER` base's point, and the `AIRORDER` base as
/// `Unit::add_air_attack_ground_order@005e41c0` fills it — `+0x18/+0x1c`
/// the home base (`oxx/whose`), `+0x20` `cruising_alt` 0x640, and
/// `sharp_turn`, `old` and `returning` (`+0x2c`) 0. The base's `accuracy`
/// and `attack_unit` are written 0 and never read on this arm; the
/// class's own `total_time`, `sx`, `sy` are not written by the adder and
/// print 0 (run371, 2672..2701).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AirAttackGroundOrder {
    /// `att_x`/`att_y`: the target's point when the order was laid.
    pub at: Pos,
    /// `AirOrder::oxx/whose`: the silo it waits in.
    pub home: Option<usize>,
    /// `AirOrder::cruising_alt`, [`CRUISING_ALT`] when added.
    pub cruising_alt: i32,
    /// `AirOrder::sharp_turn`, the edge coin's ±1.
    pub sharp_turn: i32,
    /// `AirOrder::returning` (`+0x2c` of the base `get_attack_ground`
    /// answers): `do_air_attack_ground` fires nothing while it is set.
    pub returning: bool,
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
    AirAttackGround(AirAttackGroundOrder),
    Think,
}

/// One order in a unit's list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Order {
    pub flags: u8,
    pub body: Body,
}

/// `UnitData::is_fleeing@0046efa0` on the current order's `get_type()` (vslot `+0x10`): `FLEE_TO`; no order
/// is not fleeing (item 2090's sweep holds the ask and the answer).
pub fn is_fleeing_type(front: Option<i32>) -> bool {
    front == Some(i32::from(index::FLEE_TO))
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
            Body::AirAttackGround(_) => index::AIR_ATTACK_GROUND,
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

/// The three flag arguments of `Unit::add_move_facing_order@005e55c0`:
/// `param_5` (`PATHED`, `1`), `param_7` (`ACTION`, `4`) and `param_11`
/// (`0x20`, [`flag::DISEMBARK`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MoveBits {
    pub pathed: bool,
    pub action: bool,
    pub disembark: bool,
}

/// What `add_move_facing_order` writes into the order it made: the cell's
/// centre (`u · 0x30 + 0x18`, twice — `x`/`y` and `dest_x`/`dest_y`) and
/// the three flag bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MoveFields {
    pub at: Pos,
    pub bits: MoveBits,
}

/// The order `OrdersMemManager::get_obj(idx)` hands out for a move of
/// `OrderIndex` `idx` (1..=4), with the caller's angle, the formation's
/// `facing`, the click `orig` and a group move's own block; the cell and
/// the flags are [`fill_move_order`]'s.
pub(crate) fn new_move_order(
    idx: i32,
    angle: Angle,
    facing: Option<bool>,
    orig: Option<Pos>,
    group: Option<GroupMove>,
) -> Order {
    let kind = match idx {
        2 => MoveKind::AttackTo,
        3 => MoveKind::ExploreTo,
        4 => MoveKind::FleeTo,
        _ => MoveKind::MoveTo,
    };
    let at = Pos::new(0, 0);
    Order {
        flags: 0,
        body: Body::Move(MoveOrder {
            kind,
            dest: at,
            angle,
            facing,
            has_waypoint: false,
            waypoint: at,
            last: None,
            pause: 0,
            retry: 0,
            attempts: 0,
            timer: 0,
            coll: None,
            orig,
            group,
        }),
    }
}

/// `add_move_facing_order`'s writes into the made order: `x`/`y` and
/// `dest_x`/`dest_y` the cell's centre, and the bits `1`, `4`, `0x20`
/// each set or cleared by its argument.
pub(crate) fn fill_move_order(o: &mut Order, f: &MoveFields) {
    if let Body::Move(m) = &mut o.body {
        m.dest = f.at;
        m.waypoint = f.at;
    }
    for (on, bit) in [
        (f.bits.pathed, flag::PATHED),
        (f.bits.action, flag::ACTION),
        (f.bits.disembark, flag::DISEMBARK),
    ] {
        if on {
            o.flags |= bit;
        } else {
            o.flags &= !bit;
        }
    }
}
