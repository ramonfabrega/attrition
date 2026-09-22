//! The simulation: headless, deterministic, and dependent on no engine.
//!
//! This crate knows nothing about pixels, windows, input devices, clocks or
//! threads. Everything in it runs in a `#[test]` with no display attached,
//! which is the property that makes determinism testable at all.
//!
//! Ten mechanics run here, and they run together. Borders produce territory,
//! territory produces damage, supply cancels it, units walk in and out of it
//! under orders, the ground they hold pays its owner, that income buys the next
//! unit at a price that climbs with every one already built, the unit takes
//! time to arrive — whether it may be bought at all, and what the first one
//! owns, is the tech tree's to say; units fight, and buildings are placed,
//! built, garrisoned and captured, and the cities among them grow and fall.
//! Each has a specification written from the original — `docs/ATTRITION.md`,
//! `docs/SUPPLY.md`, `docs/MOVEMENT.md`, `docs/ECONOMY.md`, `docs/COSTS.md`,
//! `docs/PRODUCTION.md`, `docs/TECH.md`, `docs/COMBAT.md`, `docs/CITIES.md` —
//! and each says how much of itself is established rather than guessed.
//!
//! [`Sim::tick`] is where they meet, and the order it does them in is the
//! original's, three times over. `Game::do_frame` pays every player before it
//! processes any object, so income comes first. Buildings and units are both
//! objects and interleave by index in `Objects::process_all`; here the
//! buildings go first, which is what the original does whenever the building
//! predates the unit it just made. And inside a unit, attrition comes before
//! movement, so a unit stepping over a border is not standing there when that
//! frame's attrition looks.
//!
//! # Arithmetic
//!
//! No floating point, ever — and so far, no fixed point either. Every quantity
//! the original computes in these mechanics is an integer with a defined
//! truncation. The one value it keeps in an `f32` is exactly a rational and is
//! carried as one; the one table it builds with doubles is built once at
//! startup, so its integers are pinned rather than recomputed. `Fx` stays
//! unearned until something genuinely needs a fraction; anticipating it would
//! only add rounding the original does not have.

pub mod ai;
pub mod ai_build;
pub mod ai_census;
pub mod ai_drive;
pub mod ai_host;
pub mod ai_load;
pub mod ai_make;
pub mod ai_place;
pub mod ai_research;
pub mod ai_sites;
pub mod ai_types;
pub mod ai_units;
pub mod air;
pub mod anim;
pub mod army;
pub mod attack_pos;
pub mod attrition;
pub mod balance;
pub mod bhs;
pub mod build;
pub mod calc_gather;
pub mod caravan;
pub mod city;
pub mod collide;
pub mod combat;
pub mod cost;
pub mod danger;
pub mod economy;
pub mod farms;
pub mod fight;
pub mod fish;
pub mod form;
pub mod gaia;
pub mod garrison;
pub mod gather;
pub mod goody;
pub mod group;
pub mod grouppath;
pub mod holdings;
pub mod launch;
pub mod market;
pub mod merchant;
pub mod mesh;
pub mod movement;
pub mod nations;
pub mod orders;
pub mod path;
pub mod place;
pub mod production;
pub mod rares;
pub mod roads;
pub mod scout;
pub mod single;
pub mod stance;
pub mod supply;
pub mod tech;
pub mod terrain;
pub mod territory;
pub mod transport;
pub mod tuning;
pub mod vision;
pub mod world;

pub use tuning::Tuning;
pub use world::{Cell, Owner, Player, Pos, Terrain, World};

/// Frames per second. The original's whole clock, and the unit every attrition
/// period is quoted in.
pub const FRAMES_PER_SECOND: i32 = 15;

/// How often a unit's periodic work runs at all. `Unit::process` gates a whole
/// block of per-unit upkeep on this.
pub const UNIT_UPKEEP_FRAMES: i64 = 16;

/// How often a unit's attrition period is recomputed.
///
/// Not every frame. This is the single most consequential fact about the
/// cadence: a unit that walks out of hostile territory keeps its stale period
/// until the next refresh, and one that walks in takes nothing until then.
pub const ATTRITION_REFRESH_FRAMES: i64 = 32;

/// How often `ObjectData::targeted` is quartered —
/// `Unit::process@00610bc0`'s `(frame + o) % 16` slot, the one
/// [`ATTRITION_REFRESH_FRAMES`]' is nested inside. `docs/COMBAT.md` §33.
pub const TARGETED_DECAY_FRAMES: i64 = 16;

/// A unit, as attrition sees one — which is **one figure**, not one squad.
///
/// Rise of Nations units are squads of one to four figures, and each figure is
/// its own `UnitData` in the owner's unit list with its own health and its own
/// cadence; the squad is what the player selects, the figure is what bleeds.
/// This struct is the figure. [`Unit::squad_size`] is how many figures its
/// squad currently has, which is what sets its share of the damage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unit {
    pub owner: Player,
    /// The unit's index in its owner's object list — `SubObjectData::o`.
    ///
    /// Every periodic thing a unit does is phased by this, so that a hundred
    /// units do not all do their upkeep on the same frame. It is part of the
    /// simulation rather than an optimisation: it decides *which* frames a
    /// given unit bleeds on.
    pub index: i16,
    pub pos: Pos,
    /// Whole hit points left.
    pub health: i32,
    /// Sixteenths of a hit point taken but not yet carried into
    /// [`Unit::health`] — `ObjectData::damage_frac`. Attrition deals its damage
    /// in sixteenths, and nothing is lost to truncation between ticks.
    pub damage_frac: i32,
    /// Figures currently in this figure's squad, one to four — the original's
    /// `curr_uber_size`. Damage per tick is set by this and nothing else.
    pub squad_size: i32,
    pub kind: attrition::UnitKind,
    /// Whether the unit is on the map, rather than garrisoned in a building or
    /// riding in a transport. Off the map, none of this runs.
    pub on_map: bool,
    /// Which slot of its owner's supply list this unit registered as, if its
    /// type is a supply source. The original caches the same index in the
    /// unit and gives it back when the unit dies.
    pub supply_slot: Option<usize>,
    /// `UnitData +0x86` for a **caravan**: its slot in the owner's
    /// [`caravan::Caravans`] list. The same field carries a hero's or a
    /// special's slot in the original; this is the caravan half alone.
    pub caravan: Option<usize>,
    /// The period the last refresh wrote, in frames. Zero means not bleeding.
    /// Public because it is observable state, not a private counter — the
    /// original keeps it in `UnitData::attrition` and the interface shows it.
    pub attrition: i32,
    /// Whether the current bleed goes through supply.
    pub ignores_supply: bool,
    /// Whether supply sheltered this unit from a tick that was otherwise due.
    /// The original tracks the same thing in a display flag.
    pub sheltered: bool,
    /// Where it is going and how fast it gets there.
    pub movement: Movement,
    /// Its type, as an index into [`Sim::unit_types`], if it has one. A unit
    /// without a type has no combat profile and neither attacks nor is worth
    /// anything to a target search; attrition and movement do not need it.
    pub ty: Option<usize>,
    /// What combat keeps on the unit: the reload counter, the target, the
    /// overkill record. See `docs/COMBAT.md`.
    pub combat: combat::State,
    /// The building this unit is garrisoned in — `UnitData::inside_up` when
    /// it points at a building. `Some` implies `on_map == false`. See
    /// `docs/CITIES.md` §6.
    pub inside: Option<usize>,
    /// The **boat** this unit is riding — `UnitData::inside_up` when it
    /// points at a unit rather than a building. `Some` implies
    /// `on_map == false`, and it is the whole reason a passenger's orders
    /// stop being stepped (`docs/TRANSPORT.md` §6).
    pub inside_unit: Option<usize>,
    /// The hit points a whole figure carries, what the garrison heal repairs
    /// up to.
    pub max_health: i32,
    /// The order list — front is the current order. `docs/ORDERS.md` §1.4.
    pub orders: std::collections::VecDeque<orders::Order>,
    /// The path stack; the top is the current waypoint (§4.2).
    pub path: Vec<orders::PathData>,
    /// **A 48-grid search this unit suspended** — `UnitData +0x104..0x148`,
    /// the five containers and the nine scalars `astar_path` hands over
    /// when it runs out of `limit` short of the goal
    /// (`docs/PATHFINDER.md` §4.3 step 3, §18).
    ///
    /// While it is `Some`, `do_move` does not step: its first block resumes
    /// the search instead (`docs/ORDERS.md` §4.4 step 2).
    /// [`Sim::clear_partial_path`] is what drops it.
    pub search: Option<Box<path::Search>>,
    /// **`UnitData::start_dist` (`+0x130`) — the dumped witness that this
    /// unit suspended a 48-grid search** (`docs/PATHFINDER.md` §18.6).
    ///
    /// It is a *stamp on the unit*, not a field of the stash: the original
    /// keeps it on `UnitData`, so it outlives the search that wrote it and
    /// still reads back long after `clear_partial_path` has freed the five
    /// containers. Three writers in the whole executable, and only one
    /// writes a value — `astar_path@00683770:517`, the suspend, which
    /// stores the search's start-to-goal Manhattan; the other two zero it
    /// at a unit's birth (`UnitData::UnitData@00606670:74` and
    /// `Unit::init@00612100:567`), which is why a recycled `o` slot reads
    /// 0 again and a live unit never does.
    pub start_dist: i32,
    /// `UnitData::tolerance`: the current waypoint's arrival radius.
    pub tolerance: i32,
    /// `unit_masks & 8`: a straight line to the waypoint has been verified.
    pub line_ok: bool,
    /// `UnitData::path_recursion`.
    pub path_recursion: u8,
    /// `UnitData::orders_x/orders_y`: the final destination of the leading
    /// run of transit moves.
    pub orders_pos: Pos,
    /// `UnitData::idle` (§2.4).
    pub idle: u8,
    /// `UnitData::stance`, the worker stance: 1 is the normal citizen (builds
    /// and gathers), 0 gathers only, 2 builds only.
    pub stance: u8,
    /// `UnitData::spell_time` (`+0x98`) — a caster's clock, and the field a
    /// **bird** reuses as its patrol counter: `Animal::think_bird` steps it
    /// every frame it runs and again on each eighth, and the number it has
    /// reached is the modulus of the landing roll (`crate::gaia`,
    /// `docs/SYNC.md` §3.9). `Unit::init` clears it with `mana_burn`, so a
    /// bird starts at 0.
    pub spell_time: i16,
    /// `unit_masks & 0x400`: has been given a build or repair order.
    pub was_builder: bool,
    /// `unit_masks & 0x78000000`: the carrying walk a gatherer plays.
    ///
    /// Four bits, and `Guy::set_anim`'s walk arm reads them in its own
    /// order — `0x10000000` `WALK_TO_WOOD`, then `0x8000000`
    /// `WALK_WITH_WOOD`, then `0x40000000` `WALK_TO_ORE`, then
    /// `0x20000000` `WALK_WITH_ORE` — so they are kept as the nibble
    /// rather than as a slot. `Unit::do_non_flat_gather` is the only
    /// writer: it clears them whole on the frame a worker reaches its
    /// tile and sets one on each walk it issues. `Unit::think` clears
    /// them for a citizen, `kill_current_order` for anybody.
    ///
    /// They are why a walk this crate had been playing as the plain
    /// `CHAR_WALK` — the carrying slots' lengths were unknown, so the
    /// packet fallback swallowed them — is a carrying walk, and why an
    /// arrival stand fires for one and not the other (`docs/ANIM.md`
    /// §4.4).
    pub carry: u32,
    /// `SubObjectData::flags & 0x10`: could not reach its target.
    pub cant_reach: bool,
    /// `unit_masks & 1`: a decoy; not counted as a gatherer.
    pub decoy: bool,
    /// `UnitData::avoid_x/avoid_y`: the point `find_path` recorded as
    /// unreachable, which `valid_wcoord` refuses. Cleared before each step
    /// (`docs/ORDERS.md` §4.5).
    pub avoid: Option<Pos>,
    /// The figures' animation clocks — `UnitData::guys`, one `Guy` per
    /// member (`docs/ANIM.md`). Empty for a unit stood up without art,
    /// which then plays nothing and draws nothing for it.
    pub guys: Vec<anim::Guy>,
    /// `UnitData::is_captain` — `o_up < 0`, a unit that heads its own
    /// squad; every standalone unit is one.
    pub captain: bool,
    /// `UnitData +0x8e` — this figure's captain, the object `Group::add`
    /// substitutes for it when `keep_captain` is 0 (`docs/GROUPS.md` §4.1).
    /// `None` for a captain, which is what [`Self::captain`] says: the
    /// original keeps one short and reads its sign bit.
    pub o_up: Option<usize>,
    /// `UnitData +0x90` — the next figure down the squad chain, which
    /// `Group::add` pulls in behind its captain with `keep_captain = 1`.
    /// A three-figure squad is a captain, its `o_down`, and that one's.
    pub o_down: Option<usize>,
    /// `ObjectData::near_o`/`near_who` (+0x34/+0x36) — the **nearest**
    /// candidate the last search saw, which is not the one it chose
    /// (`docs/COMBAT.md` §37.1).
    ///
    /// `Object::find_nearby_target@00648da0` writes it for every candidate
    /// that clears `check_target` and is nearer than the best so far,
    /// **above and independent of** the `max_dist` gate and of the score,
    /// and clears it to `-1` when the nearest one it saw is beyond
    /// `0xf00`. So it is the search's footprint rather than its answer, it
    /// survives untouched between searches, and two arms read it as a
    /// cached incumbent: `do_move`'s captain retarget (§37.2, which this
    /// crate has) and `Unit::think`'s 31-in-32 fast path (§37.5, which it
    /// does not).
    pub near: Option<combat::Obj>,
    /// `unit_masks & 0x800000`: may auto-transport (`docs/TRANSPORT.md`
    /// §3). Granted at birth under the leader's level and by
    /// `check_transport`; the player's toggle writes it too.
    pub auto_transport: bool,
    /// `unit_masks2 & 0x2000`: a scenario's per-unit veto (§3.2).
    pub never_transport: bool,
    /// `guy_flags & 0x20`, which collapses the idle roll to two variants;
    /// unread — off.
    pub guy_flag_0x20: bool,
    /// An animal's herd, an index into [`gaia::Gaia::herds`] — the
    /// `UnitData+0x86` union for an `Animal`. `None` for everything else
    /// and for a herdless animal.
    pub herd: Option<usize>,
    /// A pasture's animal — the farm it belongs to and its place in the
    /// five (`docs/SYNC.md` §3.6). `None` for everything else, which is
    /// what sends a herdless animal down `think_farm_animal`'s dead end.
    pub farm_animal: Option<farms::FarmAnimal>,
    /// The type's `TypeIndex`, when the loader named it; −1 otherwise. The
    /// birds' walk coin reads it.
    pub type_index: i32,
    /// `UnitData +0xaa`: the formation index the last group move applied,
    /// −1 none. `GroupData::get_form` reads it back (`docs/GROUPS.md`
    /// §4.4, §6.6).
    pub form: i8,
    /// `UnitData +0xab`: the twin of [`Self::form`] — the formation
    /// *width* the last group move applied, −1 none.
    /// `GroupData::get_form_mod_option` averages it over the members and
    /// `Form::compute` scales the column count by the average, so the two
    /// bytes together are how a formation remembers its own shape
    /// (`docs/GROUPS.md` §6.4, §6.6).
    pub form_width: i8,
    /// `UnitData::collide` — how many frames running this unit has been
    /// blocked. `Unit::do_idle` zeroes it; `detect_unit_collision` ages it
    /// out five frames after the last one (`docs/COLLISION.md` §4, §6).
    pub collide: i16,
    /// `UnitData::collide_frame`.
    pub collide_frame: i64,
    /// `UnitData::collide_o` / `collide_who` / `collide_guy`: what is in
    /// the way, `-1` for nothing.
    pub collide_o: i16,
    pub collide_who: i8,
    pub collide_guy: i16,
    /// `UnitData::safe` (`+0xb2`): frames left of the collision cooldown a
    /// failed 48-grid search buys. `Unit::work` counts it down and
    /// `detect_unit_collision` refuses to test while it is set.
    pub safe: i32,
    /// `unit_masks & 0x40`: waiting for the unit in front to move.
    pub waiting_on: bool,
    /// `unit_masks & 0x100000`: the one-shot half step a soft collision
    /// asks for (`docs/MOVEMENT.md`, `docs/COLLISION.md` §4.3).
    pub half_step: bool,
    /// `ObjectData::down` / `up`: this unit's place in its world cell's
    /// object chain, newest first (`docs/COLLISION.md` §3).
    pub down: Option<usize>,
    pub up: Option<usize>,
    /// `UnitData::rare` (`+0x54`, the union with `air_alt` and
    /// `former_type`): the `TypeIndex` of the good `UnitData::calc_gather`
    /// last found for this unit, `-1` when it looked and found none.
    /// `Unit::init@00612100:90` starts it at **0**, so a unit that has
    /// never asked is not the same as one that asked and failed
    /// (`docs/ORDERS.md` §6.10).
    pub rare: i32,
    /// `UnitData::good_obj` (`+0x94`): the `circle_x`/`circle_y` index, in
    /// tiles from the unit's own, that `calc_gather` found that good at —
    /// the one index its next call retries before walking the spiral
    /// again. `Unit::init:281` starts it at `-1`.
    pub good_obj: i16,
    /// `unit_masks & 0x20`: `calc_gather`'s "and I may go on gathering
    /// here", which `Unit::do_gather@005fce20` records and `think_fish`
    /// clears. Both of the original's gameplay callers pass `param_7 = 1`,
    /// which is the arm that never sets it, so nothing here has ever seen
    /// it true — and no `unit_masks` in any capture on disk carries the bit
    /// (`docs/ORDERS.md` §6.10).
    pub gather_here: bool,
    /// `ObjectData + 0x8 & 8`: "this unit has gone idle and nothing has
    /// given it an order since". `Unit::check_idle` sets it and
    /// `Unit::work` clears it, and for a **fisherman or a merchant** each
    /// of those two transitions marks the owner's economy dirty — because
    /// each of them changes who `Leader::calc_gather` step 6 will walk
    /// ([`crate::rares`]). It is the reason a boat that finishes its
    /// journey has its deposit counted within eight frames rather than at
    /// the next 512-frame refresh.
    pub idle_latch: bool,
    /// **`ObjectData::visible` (`+0x40`)** — the byte of players this
    /// object has made itself visible to *by attacking them*, one bit per
    /// `who`. `docs/VISION.md` §7.
    ///
    /// It is the fallback arm of `UnitData::is_seen`: a unit standing in a
    /// player's fog is still a legal target for that player while its bit
    /// is set, which is how a shooter that nobody can see still gets shot
    /// back at. [`Sim::set_attacking`] is the only writer here;
    /// [`Sim::work`] clears it.
    pub visible: u8,
    /// **`UnitData::unit_masks & 4`** — the "in danger" latch
    /// `Unit::set_in_danger@005fcfb0` raises and `Unit::work@0060d180`
    /// drops on the unit's own 32-frame tick (`0060d19d`, above every
    /// gate — the same tick that clears [`Unit::visible`]).
    ///
    /// Its one reader here is `Group::action_move_near`'s §6.6 step 6:
    /// a member carrying it takes a **plain** move where the rest of the
    /// group takes a `GroupMoveOrder`. `Unit::go_to_unit@005f78c0` sets
    /// it over the whole squad it is about to walk, one line before the
    /// walk, which is why no `go_to` group has ever marched in formation.
    ///
    /// SEAM: the other two writers. `Object::take_damage@00652020` marks
    /// a unit that is hit, and `Unit::work` marks an attacker and its
    /// target; neither is carried, so a group ordered within 32 frames of
    /// a fight marches in formation here where the original's would not.
    /// Nothing else in the engine reads the bit.
    pub in_danger: bool,
    /// **`SubObjectData::flags & 0x80`** — "attacked this frame, or is
    /// still on an `ATTACK` order".
    ///
    /// `Unit::set_attacking@005ff5b0` raises it on every strike and
    /// `Unit::work@0060d180` drops it on any frame the front order is not
    /// `ATTACK`. Its only job is to hold [`Unit::visible`] against the
    /// 32-frame clear at the head of [`Sim::work`], which is why the bit
    /// survives about a third of a second past the last arrow and the
    /// visibility survives up to 32 frames past that.
    pub attacking: bool,
}

/// What a unit needs in order to move.
///
/// `speed` and `turning` are **inputs**, not computed here. The speed is the
/// output of `UnitData::get_speed`'s three-layer pipeline, which reads tech,
/// nation, hero and terrain state the simulation does not model yet, and the
/// turning facts come from the unit type. Taking them as inputs is the same
/// choice supply made for a general's aura radius: the mechanic is complete,
/// and the number feeding it arrives when the layer that produces it does.
/// `docs/MOVEMENT.md` documents the pipeline in full.
///
/// The rest is state the original keeps too: the unit's position is on
/// [`Unit`], and the body — the figure that chases it, whose `last_speed` and
/// `avg_speed` the unit's turn rate reads — is here.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Movement {
    /// Which way the unit faces — **`GuyData::angle` (`+0x18`)**, guy 0's own
    /// facing, which the unit step turns by the rate and steps along. North is
    /// zero.
    pub facing: movement::Angle,
    /// The facing as it stood at the start of this frame — what the body
    /// follow compares against after the order step has turned the unit
    /// (`guy_flags & 2`).
    pub frame_facing: movement::Angle,
    /// Where it *wants* to face: **`UnitData::angle` (`+0x50`)**, which is
    /// also **`GuyData::des_angle` (`+0x64`)** — `Unit::set_angle` writes the
    /// two together, and `Unit::move_step` calls it with the bearing to the
    /// destination on every frame, before turning. So this is the heading, not
    /// the facing: it is what the dump's `UNITDATA angle` line carries, what
    /// `Group::update_positions` rotates a slot table by, and what an idle
    /// body turns toward.
    pub heading: movement::Angle,
    /// **`UnitData::dest_angle` (`+0x58`)**, which is a different field and a
    /// different thing: the angle the *order* wants the unit to end up facing.
    /// `Unit::update_action` seeds it from [`Movement::heading`] and then
    /// overwrites it with the angle of the last transit move it walks past
    /// (`docs/ORDERS.md` §3.3). Nothing in the step reads it.
    pub des_angle: movement::Angle,
    /// Where it is headed. `None` means it is not going anywhere.
    pub dest: Option<Pos>,
    /// Effective speed, in position units per frame. Used for both the unit
    /// step and the body's chase; the body's own `+9` and ungrouped speed are
    /// not modelled.
    pub speed: i32,
    /// The type-level facts behind the turn rate and turn limits.
    pub turning: movement::Turning,
    /// The body, standing on the unit until the unit moves.
    pub body: movement::Body,
}

impl Movement {
    /// A unit standing at `pos`, with its body on it and stopped — which, for
    /// a foot or mounted type, is what lets the first order turn it
    /// instantly. All three angles are `Unit::init`'s own
    /// [`movement::Angle::INITIAL`], not north.
    pub const fn at(pos: Pos) -> Movement {
        Movement {
            facing: movement::Angle::INITIAL,
            frame_facing: movement::Angle::INITIAL,
            heading: movement::Angle::INITIAL,
            des_angle: movement::Angle::INITIAL,
            dest: None,
            speed: 0,
            turning: movement::Turning {
                type_turn_speed: 0,
                packed: false,
                instant_from_stop: false,
                wide_limit: false,
            },
            body: movement::Body::at(pos),
        }
    }

    /// Points the unit and its body a given way at once — `Guy::set_angle`
    /// with the snap flag, the one caller that moves the facing and the
    /// heading together. `Unit::move_step` never uses it: its `set_angle`
    /// passes zero for that flag, so a step moves the heading and leaves the
    /// facing to the turn rate.
    pub const fn set_facing(&mut self, facing: movement::Angle) {
        self.facing = facing;
        self.heading = facing;
        self.des_angle = facing;
    }

    /// Points the unit a given way and **leaves the body to turn** —
    /// `Unit::set_angle(a, a, 0)`, which is what every order in the
    /// ordinary path calls. `Unit::set_angle@00605400` writes the unit's
    /// own `+0x50` and then `Guy::set_angle@005d9010`, whose snap flag is
    /// that third argument: with it zero the guy takes `des_angle`
    /// (`+0x64`) alone and its `angle` (`+0x18`) stands.
    ///
    /// One frame, and it is visible: a body owed a turn is a body
    /// `Guy::move` finds at its destination with `des_angle != angle`,
    /// which is the **turn arm** — `set_anim(CHAR_WALK, 0, 1)`, on top of
    /// whatever the order just asked for. So the work animation an order
    /// sets on the frame it also turns the unit is overwritten by the walk
    /// for that frame and takes hold on the next. Run44's citizen `0/2` is
    /// the record: at the camp on frame 542 its `wait` steps 32 → 31 and
    /// its guy plays `26 1/15`, the carrying walk restarted; `27 1/32`,
    /// the wood dump, is frame 543 (`docs/ANIM.md` §4.7).
    ///
    /// The snapping [`Movement::set_facing`] is the other arm — the flag
    /// set — and its callers are `do_move`'s re-face, `go_inside`,
    /// `do_spec_anim` and the scenario loader, none of which this crate
    /// reaches yet.
    pub const fn set_heading(&mut self, heading: movement::Angle) {
        self.heading = heading;
    }
}

/// The type-level facts the turn rate and the turn-in-place limits read —
/// `movement::Turning` as `Guy::init_real` and `Unit::move_step` derive them.
///
/// **`instant_from_stop` is `guy_flags & 0x10`**, and `Guy::init_real` builds
/// it out of four type tests: not a packing type (`unit_flags2 & 4`), carrying
/// objmask `FOOT` or `MOUNTED` or the program-set `unit_flags & 0x10`, and
/// **not** `unit_flags & 2` — the `FLAGS b` whose legend is "Unit is a
/// horse-drawn cart type thing". A cart does not pivot. That is the whole of
/// `unitrules.xml`'s own note, "Foot & Mounted units turn instantly from a
/// stopped position", and with a stopped body — which is every frame the unit
/// did not move — it makes the next step's turn free.
///
/// `wide_limit` is `move_step`'s 80° branch: a non-land type, or a `VEHICLE`,
/// keeps walking while it still owes up to 80° — but only two tiles out or
/// more.
///
/// `packed` is not here: it is `unit_masks & 0x80000`, a runtime state.
pub fn turning_of(t: &UnitType) -> movement::Turning {
    let objmask = t.combat.obj_masks;
    let foot_or_mounted = objmask & (combat::mask::FOOT | combat::mask::MOUNTED) != 0;
    movement::Turning {
        type_turn_speed: t.turn_speed,
        packed: false,
        instant_from_stop: !t.cols.flag2(ai_load::uflags2::PACKS)
            && (foot_or_mounted || t.cols.flag(ai_load::uflags::TRANSPORT))
            && !t.cols.flag(ai_load::uflags::CART),
        wide_limit: t.combat.domain != attrition::Domain::Land
            || objmask & combat::mask::VEHICLE != 0,
    }
}

/// A unit type the simulation knows how to build.
///
/// The price is the data's, the kind is what attrition and supply read, and
/// the group is which production building's output the ramp counts against —
/// `None` for a type whose progression counts by type, which is every civilian
/// in the shipped data.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UnitType {
    pub price: cost::Price,
    pub kind: attrition::UnitKind,
    pub group: Option<usize>,
    /// The `HITS` column: what a unit of this type is built with.
    pub hits: i32,
    /// The time half of the same record — `JOB_TIME`, `JOB_EXTRA_TIME` and
    /// `RESEARCH_PREMIUM_TIME`. See `docs/PRODUCTION.md`.
    pub times: production::Times,
    /// The combat columns and derived bits — `docs/COMBAT.md` §2.1. A
    /// default profile has zero attack and is neither target nor threat.
    pub combat: combat::Profile,
    /// This type's entry in [`Sim::tech_tree`], if it has one. With it,
    /// queueing is gated by `type_avail` and the first completion goes
    /// through `gain_tech` — predecessors become owned and obsolete, free
    /// units and buildings cascade — instead of only setting the bit. Without
    /// it the type is outside the tree, as every type was before the tree
    /// existed, and [`Muster::researched`] is the whole story.
    pub tree: Option<tech::TypeId>,
    /// What the type needs in order to garrison — `docs/CITIES.md` §6.4.
    pub garrison: garrison::UnitTraits,
    /// `MOVES`: the type's base speed in position units per frame — the
    /// number `UnitData::get_speed` starts from, before the nation, tech and
    /// terrain layers, and already in the unit movement uses
    /// (`docs/MOVEMENT.md` §1: `UNIT_MOVE_SPEED` is the identity converter).
    pub moves: i32,
    /// `TURN_SPEED`, through `degrees_to_angle` as `UnitType::init` stores
    /// it — the `type_turn_speed` of [`movement::Turning`].
    pub turn_speed: i32,
    /// `LOS`, `ObjectTypeData +0x21c` — line of sight **in tiles**, which
    /// `Object::update_seen` halves to get a fog radius (`docs/VISION.md`
    /// §3). Zero means the type reveals nothing at all.
    pub los: i32,
    /// `SCIENCE_LOS`, `+0x220` — added once per science level.
    pub science_los: i32,
    /// The type's own `TypeIndex` — `0x32 + record`, the number the dump's
    /// `GUY.type` carries and several predicates test by identity.
    pub type_index: i32,
    /// Which worker kind this is — a citizen or scholar may gather and is
    /// what `think_peasant` runs for (`docs/ORDERS.md` §6.1).
    pub worker: orders::Worker,
    /// The words no column carries — `unit_flags`, `unit_flags2`, `CAT`,
    /// `CARRY` and the `role` the loader derives from them. Every producer
    /// reads them; `docs/DATALAYER.md`, "The derived words no column
    /// carries", is the derivation and [`ai_load`] the code.
    /// The two lineage roots `ObjectData::train_time`'s **British** arm
    /// tests, precomputed the way [`ai_load::RoleFacts`]' are: `is(0xaa)`
    /// — the Bowmen root, which is the Archers line — and `is(0x119)`, the
    /// Anti-Aircraft Gun. `docs/PRODUCTION.md`, "The tail".
    pub archer: bool,
    pub anti_air: bool,
    pub cols: ai_load::UnitCols,
    /// One of the twelve gaia types (`BASE_GAIATYPES..END_GAIATYPES`, record
    /// `352..`). The animals are unit types with a `WHERE` of Large City, and
    /// several of the original's loops stop before them.
    pub gaia: bool,
}

/// What a player has built, as the price and the population cap see it.
///
/// The original keeps three per-type arrays on the leader and this holds the
/// two that production reads: `num_units` at `+0x5762`, which is what exists,
/// and `num_queued` at `+0x5a22`, which is what has been ordered. The
/// distinction is not cosmetic — **the price ramp counts both and the time
/// ramp counts only the first**, so ordering five hoplites at once raises the
/// price of each but not the time of any. See `docs/PRODUCTION.md`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Muster {
    /// Units of each type, indexed by [`Sim::unit_types`]. `num_units`.
    pub by_type: Vec<i32>,
    /// Units of each type ordered and not yet delivered. `num_queued`.
    pub queued_by_type: Vec<i32>,
    /// Whether each type has been researched — the availability bit at
    /// `leader + 0x6c18`. It decides two things at once and must decide them
    /// together: the *first* of a unit type is a research job, at research
    /// pace and research time, and every one after is a train job.
    pub researched: Vec<bool>,
    /// How many of the player's cities hold a library —
    /// `LeaderData::get_building_cities`. This is how many slots of the
    /// **first library's** queue advance at once — and of no other queue —
    /// and since every research job is forwarded to that library, it is how
    /// many technologies can progress simultaneously.
    pub library_cities: usize,
    /// The same, by production group.
    pub by_group: Vec<i32>,
    /// And the queued half of it — `LeaderData`'s `barracks_queued`,
    /// `stable_queued`, `factory_queued`, `dock_queued`, `air_queued`.
    ///
    /// `LeaderData::get_support_count@006da110` returns
    /// `<group>_queued + <group>_units` for a type whose `PROGRESSION` is
    /// by-group, exactly as it returns `num_queued + num_units` for one
    /// that is by-type. Without this half the ramp does not advance
    /// *within* a batch: `Leader::produce_unit` queues its units one at a
    /// time and each is priced from the count, so two Longbowmen ordered
    /// together both paid the first one's price (`docs/COSTS.md`, "The
    /// count").
    pub queued_by_group: Vec<i32>,
    /// Population occupied — `LeaderData::control`.
    pub control: i32,
    /// The cap it is measured against. Recomputed from scratch by
    /// [`Sim::recompute_pop_caps`] rather than maintained incrementally,
    /// because `Leader::calc_pop_cap` recomputes it from scratch too.
    pub cap: i32,
    /// The player's Military library level, `epoch[0]` — how many of the seven
    /// Military techs they hold, from The Art of War up. It indexes `POP_CAP`,
    /// and it is not the age. An input until there is a tech layer to produce
    /// it.
    pub military_level: usize,
    /// The lobby's population setting. The shipped choices are 50, 75, 100,
    /// 125, 150 and 200, and the largest is exactly `POP_CAP[7]`.
    pub limit: i32,
    /// The wonders, nations and scenario overrides that move the cap.
    pub bonuses: cost::PopBonuses,
}

/// The largest population the lobby offers, and this simulation's default.
pub const DEFAULT_POP_LIMIT: i32 = 200;

impl Muster {
    fn new(t: &Tuning) -> Muster {
        let mut m = Muster {
            limit: DEFAULT_POP_LIMIT,
            ..Muster::default()
        };
        m.cap = cost::pop_cap(t, m.military_level, m.limit, &m.bonuses);
        m
    }
}

/// Why a unit was not built.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refused {
    /// The population cap would be exceeded — `check_population`.
    Population,
    /// Something in the price could not be paid — `Leader::can_pay`.
    Cost,
}

impl Unit {
    pub fn new(owner: Player, index: i16, pos: Pos, health: i32) -> Unit {
        Unit {
            owner,
            index,
            pos,
            health,
            search: None,
            // `Unit::init@00612100:567` zeroes the stamp beside the five
            // container pointers: a unit's search state starts empty.
            start_dist: 0,
            damage_frac: 0,
            squad_size: 1,
            kind: attrition::UnitKind::default(),
            on_map: true,
            supply_slot: None,
            caravan: None,
            attrition: 0,
            ignores_supply: false,
            sheltered: false,
            movement: Movement::at(pos),
            ty: None,
            combat: combat::State {
                captain: i32::from(index),
                ..combat::State::default()
            },
            inside: None,
            inside_unit: None,
            max_health: health,
            orders: std::collections::VecDeque::new(),
            path: Vec::new(),
            tolerance: 0,
            line_ok: false,
            path_recursion: 0,
            orders_pos: pos,
            idle: 0,
            stance: 1,
            spell_time: 0,
            was_builder: false,
            carry: 0,
            cant_reach: false,
            decoy: false,
            avoid: None,
            guys: Vec::new(),
            captain: true,
            o_up: None,
            o_down: None,
            near: None,
            auto_transport: false,
            never_transport: false,
            guy_flag_0x20: false,
            herd: None,
            farm_animal: None,
            type_index: -1,
            form: -1,
            form_width: -1,
            collide: 0,
            collide_frame: 0,
            collide_o: -1,
            collide_who: -1,
            collide_guy: -1,
            safe: 0,
            waiting_on: false,
            half_step: false,
            down: None,
            up: None,
            // `Unit::init@00612100` lines 90 and 281: `rare` is **0** and
            // `good_obj` is `-1`, and the two starts are not the same
            // sentinel.
            rare: 0,
            good_obj: -1,
            gather_here: false,
            idle_latch: false,
            // `Object::init@00647750:25` zeroes `visible` and
            // `launch_frames` together as one short.
            visible: 0,
            in_danger: false,
            attacking: false,
        }
    }

    pub fn alive(&self) -> bool {
        self.health > 0
    }

    /// The frame counter this unit's periodic work is phased against.
    pub const fn phase(&self, frame: i64) -> i64 {
        frame + self.index as i64
    }
}

/// A headless world with players and units, enough to run attrition end to
/// end and no more.
///
/// This is a harness, not an architecture. It exists so that the mechanic can
/// be exercised as the original exercises it — per unit, per tick, against a
/// recomputed territory map — rather than only through its parts.
#[derive(Clone, Debug)]
pub struct Sim {
    pub tuning: Tuning,
    pub world: World,
    pub players: Vec<attrition::PlayerState>,
    pub sources: Vec<territory::Source>,
    pub units: Vec<Unit>,
    /// One supply network per player. Never shared: an ally's supply wagon
    /// does nothing for your units, which is a rule of the original and not a
    /// simplification here.
    pub supply: Vec<supply::Network>,
    /// Diplomacy, as a full matrix. `at_war[a][b]` is symmetric in practice
    /// but stored both ways, because the original reads it both ways.
    pub at_war: Vec<Vec<bool>>,
    /// One income ledger per player, and what feeds it. Split the way the
    /// original splits them: `holdings` is what a player *has*, and changes
    /// when they build something; `ledgers` is what that is worth and what has
    /// accrued, and changes every frame.
    pub holdings: Vec<economy::Holdings>,
    pub ledgers: Vec<economy::Ledger>,
    /// The unit types this simulation can build. A type's index is its id, the
    /// way the original's tables are index-keyed; see `docs/DECISIONS.md`
    /// entry 9.
    pub unit_types: Vec<UnitType>,
    /// The fifty `resourcerules.xml` goods' payouts, record index as id —
    /// the six basics first and then the forty-four rares
    /// (`crates/rondata/src/load.rs`). Only `LeaderData::calc_rare` reads
    /// them, so an empty table simply means no deposit pays anything.
    pub good_types: Vec<economy::GoodType>,
    /// One per player: whether `Leader::calc_unit_stats` is owed — the
    /// `LeaderData` flag `0x4000000`, which `Leader::process` acts on
    /// **the frame it is raised**, before any unit moves. The one writer
    /// modelled here is a change in the player's rare mask
    /// (`Sim::calc_unit_stats`).
    pub unit_stats_dirty: Vec<bool>,
    /// One per player: what they have built, and the population it occupies.
    pub muster: Vec<Muster>,
    /// Where an unavailable resource's price is charged instead.
    pub redirects: cost::Redirects,
    /// The production buildings, each with its own queue.
    pub buildings: Vec<Building>,
    /// `Farms`' list: every farm in the order it was activated
    /// (`Farms::add` runs from `Build::activate`), which is the order
    /// `Farms::inc_time` walks and so the order the farm draws are spent in
    /// (`docs/SYNC.md` §3.3, §4.1). It is not the buildings' order: the
    /// starting positions are built from the higher start slot down, so
    /// the AI's farms come before the human's in every `DUMP_ALL` dump.
    pub farm_order: Vec<usize>,
    /// The tech tree — what may be bought and what owning it changes; see
    /// `docs/TECH.md`. Empty until [`Sim::set_tech_tree`], and a unit type
    /// joins it through [`UnitType::tree`].
    pub tech_tree: tech::TechTree,
    /// The lobby settings the tree reads: starting and ending ages, the
    /// game rules, the no-powers flags.
    pub setup: tech::Setup,
    /// One per player: the tech bits and counters — `LeaderData::tech`,
    /// `ages`, `epochs`, `epoch[4]`.
    pub tech: Vec<tech::PlayerTech>,
    /// The combat table — `docs/COMBAT.md` §5 — over [`Sim::unit_types`]
    /// ids. [`combat::Table::uniform`] until something builds one.
    pub table: combat::Table,
    /// The craft table — `craftrules.xml`'s 55 rows in file order, which
    /// is `TypeIndex` [`orders::spell::FIRST`]`..=`[`orders::spell::LAST`]
    /// (`docs/ORDERS.md` §6.9). Empty until the loader fills it, and
    /// [`Sim::spell_job_time`] then answers 0 — the transport craft's own
    /// number, which is what every fixture that casts wants.
    pub spells: Vec<orders::SpellType>,
    /// The game's random stream, `game_random`. Combat draws from it for
    /// projectile scatter and the one-in-five retarget roll.
    pub rng: combat::Rng,
    /// `UnitData +0x80` → `Groups::list[gid]`, the group pool — the
    /// slots [`Sim::push_group`] installs a stack group into, each with
    /// the [`group::GroupState`] record an army's group also carries.
    ///
    /// **One slot until item 465**, which is what kept Great Lakes' probe
    /// from ever running `do_group_move`: the six raiders it pushes out
    /// of the army on frame 8186 are `group 65` in the dump for the next
    /// two thousand frames, and here the scout's next `go_to` overwrote
    /// them eleven times before the word (`docs/ORDERS.md` §16). Slots
    /// are recycled the way `Groups::get_open_slot` recycles them, so the
    /// pool stays as small as the live groups.
    pub(crate) pushed: Vec<group::Pushed>,
    /// Draw marks spent by **staged input** at `Game::do_frame`'s entry,
    /// before the frame's first phase — the cheat channel's, which
    /// `rontrace.dll` hands to `ConsoleWin::parse_cmd` there
    /// (`docs/INPUT.md` §11). [`Sim::tick`] clears [`Sim::phase_marks`] at
    /// its head, so a staged draw marked before the call would vanish from
    /// the fold while still moving the stream; these are moved to the head
    /// of the frame's own marks instead, which is where the original
    /// spends them. `crate::golden::Script::stage` is the only writer.
    pub staged_marks: Vec<(String, u32)>,
    /// The engine's global `GameAccess::ai_off`, which the `ai off` cheat
    /// toggles (`Game::action_cheat_ai_toggle@005930c0` is the whole of
    /// it: one flag, flipped, in a single-player game). Three readers in
    /// the simulation's own territory — `Leader::production_ai`'s head,
    /// `Leader::diplomacy`'s head, and the computer block of
    /// `Unit::think@005f6e40:205` — and they are what `docs/INPUT.md` §11
    /// models. Off by default; only the golden record's interpreter sets
    /// it.
    pub ai_off: bool,
    /// Whether [`Sim::tick`] records [`Sim::phase_marks`]. Off by default:
    /// the harness turns it on, the soak pays nothing.
    pub trace_phases: bool,
    /// Whether the search records [`Sim::cost_marks`]. Off by default, and
    /// the reason it exists is that the original can now answer the same
    /// question: `rontrace.cfg`'s `callwin` proxies
    /// `PathFinder::calc_cost` and logs its arguments **and its answer**
    /// (`docs/PATHFINDER.md` §10).
    pub trace_costs: bool,
    /// Every step the search priced, in order, while [`Sim::trace_costs`]
    /// is set. Nothing in the simulation reads it.
    pub cost_marks: Vec<path::CostMark>,
    /// The same for the **road** search, filled under the same flag:
    /// `PathFinder::calc_road_cost`'s arguments and answer, node for node
    /// (`docs/ROADS.md` §7.2). Nothing in the simulation reads it.
    pub road_marks: Vec<roads::RoadCostMark>,
    /// The road **mesh** — `crate::mesh`, `docs/ROADS.md` §9. One
    /// `RoadElementCandidate` per road tile, and the pass that lays road of
    /// its own to close a corner two roads leave open.
    pub mesh: mesh::RoadMesh,
    /// Whether a building replans the road to its city — `crate::roads` §5.
    /// **On**, as of run32: the search's expansion count is the original's
    /// exactly on every capture that feeds it the map's *own* terraformed
    /// heights, and the road it lays is the original's tile for tile
    /// (`docs/ROADS.md` §7). It was off while the count was six per cent
    /// short, which turned out to be the harness reading a different
    /// game's height grid rather than anything in the search.
    pub plan_roads: bool,
    /// The stream's word at each phase boundary of [`Sim::tick`], filled
    /// only while [`Sim::trace_phases`] is set. It is what
    /// `tools/trace/report.py sites` is for the original — a frame's draws
    /// attributed, here to a phase and a unit rather than a return address
    /// — and comparing the two folds is how the frame-0 gap was taken
    /// apart (`docs/SYNC.md` §4.2).
    pub phase_marks: Vec<(String, u32)>,
    /// Ammo in flight.
    pub projectiles: Vec<combat::Projectile>,
    /// One per player: the nation, wonder and patriot layer of the damage
    /// formula, as inputs.
    pub mods: Vec<combat::Modifiers>,
    /// Every delivery of damage so far, newest last. Tests read it; nothing
    /// in the simulation does.
    pub hits: Vec<combat::Hit>,
    /// The building types — `docs/CITIES.md` §1.5. A building with
    /// [`Building::ty`] set obeys the placement, construction, city and
    /// garrison rules; one without is the bare production-and-combat object
    /// the earlier mechanics used.
    pub build_types: Vec<build::BuildType>,
    /// The city records — `docs/CITIES.md` §5.
    pub cities: Vec<city::City>,
    /// One per player: the nation, wonder and tech inputs this mechanic reads.
    pub nation: Vec<city::Nation>,
    /// One per player: the border bonuses a city or fort of theirs projects,
    /// `docs/ATTRITION.md`. Zero until something sets them.
    pub borders: Vec<territory::PlayerBorders>,
    /// Alliance, as a matrix; `is_ally` needs it both ways.
    pub allied: Vec<Vec<bool>>,
    /// `LeaderData::treaties` (`+0x94`), `int[8]` — the standing agreements
    /// between two leaders, of which exactly one bit is modelled here:
    /// **bit 0, the met bit**, "these two have made contact".
    /// [`Sim::treaty_on`] is its only writer and [`Sim::has_met`] its only
    /// reader; `docs/VISION.md` §6.2 and `docs/AI.md` §46.
    pub treaties: Vec<Vec<i32>>,
    /// Whether each player has been defeated — `leader_flags & 2` clear.
    pub defeated: Vec<bool>,
    /// `LeaderData::lost_city_stamp`: the frame each player last lost a city.
    pub lost_city_stamp: Vec<Option<i64>>,
    /// `LeaderData::cities_built`, `cities_captured`, `cities_lost`.
    pub city_tally: Vec<city::Tally>,
    /// `LeaderData::high_buildings` (`+0x5660`): per player, the most
    /// buildings of a lineage this player has ever held finished at once,
    /// indexed by the lineage's **root** record. Only `Build::activate`
    /// raises it, and what it gates is the nation's free units
    /// (`crate::nations`). Grown on demand, because the build types are
    /// loaded after the simulation is stood up.
    pub building_high: Vec<Vec<i32>>,
    /// Buildings disbanded or died this frame, for tests.
    pub removed: Vec<usize>,
    /// `leader_flags & 0x8000000` per player: the wall stats are stale.
    /// `Leader::process` answers it with `calc_wall_stats` — every unfinished
    /// building's clock re-baked and every building's hit points refreshed —
    /// before any object is processed. `docs/CITIES.md` §3.2.
    pub wall_stats_dirty: Vec<bool>,
    /// One per player: the object-number marks [`Sim::find_free`] allocates
    /// against.
    pub marks: Vec<Marks>,
    /// The lobby's option block, as the AI reads it — `docs/AI.md` §12.1.
    pub lobby: ai::Lobby,
    /// One per player: `GameAccess::leader_options->list[who]`, the four
    /// per-player option words `Unit::init` reads to give a new unit its
    /// stance — `docs/ORDERS.md` §5.10.
    pub leader_options: Vec<stance::LeaderOptions>,
    /// `ScenarioData::find_counters[]` and `ScriptTimers` — the global
    /// state the script host functions keep between calls.
    pub script_env: ai_host::ScriptEnv,
    /// The loaded opening scripts and their statics, once
    /// [`Sim::load_scripts`] has run; `None` is a game with no script.
    pub scripts: Option<ai_host::Scripts>,
    /// One per player: the production AI's state — the step machine, the
    /// personality, the make list, the goods picture (`docs/AI.md` §2).
    /// A human's is never stepped.
    pub ai: Vec<ai::Leader>,
    /// `ai_speed`: 1, plus one per `ai speed increase` cheat.
    pub ai_speed: i32,
    /// One per player: the transport level bits, the lock and the scouts
    /// option (`docs/TRANSPORT.md` §2).
    pub transport: Vec<transport::LeaderTransport>,
    /// One per player: the docks registry (`docs/TRANSPORT.md` §5).
    pub docks: Vec<transport::Docks>,
    /// One `Caravans` list a player — the trade routes (`crate::caravan`).
    pub caravans: Vec<caravan::Caravans>,
    /// One per player: the sixteen army slots (`docs/ARMY.md`).
    pub armies: Vec<army::Armies>,
    /// The market's price cycle (`market.rs`).
    pub market: market::Market,
    /// The herds and the birds' sampling (`gaia.rs`).
    pub gaia: gaia::Gaia,
    /// The animation art the clocks read — lengths and pieces, an input
    /// (`docs/ANIM.md` §3).
    pub art: anim::Art,
    /// `GameInfo::seed`, the lobby's map seed, which picks a gaia guy's
    /// piece by `(seed + o) % 3`.
    pub game_seed: i32,
    /// The collision occupancy index — `CollBlock`, flattened
    /// (`docs/COLLISION.md` §2).
    pub coll: collide::CollGrid,
    /// **The run116 comparison's recorder** (`docs/COLLISION.md` §9.6),
    /// and `None` in every run but a test's.
    ///
    /// A sweep's four steps are only comparable **where the original's
    /// own bracket sits** — mid-frame, after the units that step earlier
    /// have moved the occupancy index under it. Read at a frame boundary
    /// instead, this crate's probe on Great Lakes 10161 answers a
    /// different cell and names a different mechanism (§9.2), so the
    /// comparison cannot be a replay-then-ask. This is the shape
    /// [`ai::Leader::unit_offers`] takes for `docs/AI.md` §52's offers:
    /// the simulation fills a recorder as it runs and the test reads it
    /// afterwards.
    pub sweep_watch: Option<collide::SweepWatch>,
    /// `WData::down` per world cell: the head of the object chain (§3).
    pub chain_heads: Vec<Option<usize>>,
    /// `GameDaemon::repaths[who]`: how many 48-grid recoveries this player
    /// has asked for, the throttle collision recovery reads (§6 step 6).
    pub repaths: Vec<i32>,
    pub frame: i64,
}

/// The high-water marks of a player's object array — `Objects::mark[who]`
/// for each of the three bands `find_free` allocates from. Object numbers
/// (`SubObjectData::o`) are per player: units from 0, buildings from 2000,
/// walls from 3000 (`docs/CITIES.md` §1, `docs/INPUT.md` §2). A mark only
/// ever grows; a freed slot below it is reused by [`Sim::find_free`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Marks {
    pub unit: i16,
    pub build: i16,
}

impl Default for Marks {
    fn default() -> Marks {
        Marks {
            unit: UNIT_BASE,
            build: BUILD_BASE,
        }
    }
}

/// `find_free(who, 0, 2000, …)`: the unit band.
pub const UNIT_BASE: i16 = 0;
/// `find_free(who, 2000, 3000, …)`: the building band.
pub const BUILD_BASE: i16 = 2000;
/// Where the building band ends and the wall band begins.
pub const WALL_BASE: i16 = 3000;

/// A building: a production queue, a combat profile, and — when it has a
/// type — a footprint, a construction clock, a city and a garrison.
/// `docs/CITIES.md` §1.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Building {
    pub owner: Player,
    /// Its number in its owner's object array — `SubObjectData::o`, in
    /// `[2000, 3000)`. The scripts and the AI's producers hold buildings by
    /// this number, and a dump's `b.o` is the same figure.
    pub index: i16,
    /// Its snapped centre. Where a finished unit appears: `Build::train`
    /// places it at the building's own position and then puts it inside, so
    /// every unit is born garrisoned and leaves by the ejection path.
    pub pos: Pos,
    pub queue: production::Queue,
    /// Whether this is a library — the original's `is(LIBRARY, 0)`. Two rules
    /// key on it and on nothing else: orders and cancels given to any library
    /// are forwarded to the player's *first* one, and only that first
    /// library's queue fans out across `Muster::library_cities` slots. There
    /// is no building-type system here yet; this flag is the smallest honest
    /// stand-in for the one test production needs.
    pub is_library: bool,
    /// Its combat columns — `None` for a building that neither shoots nor is
    /// shot at. A building with a profile has hit points and can die.
    pub combat: Option<combat::Profile>,
    /// The hit points it was built with and what it has left, whole and in
    /// sixteenths, the way a unit carries them.
    pub hits: i32,
    pub health: i32,
    pub damage_frac: i32,
    /// `WallData::is_active` — finished; an unfinished building takes
    /// quadruple from other buildings and loses progress when hit.
    pub active: bool,
    /// The garrison's contribution to `get_garrison_arrows` — the sum of
    /// `attack / 10` over the foot soldiers inside, an input until there is a
    /// garrison.
    pub garrison_attack: i32,
    /// `BuildData::recharging`, `attack_ox/attack_whom`, the explicit-order
    /// flag.
    pub recharging: i32,
    pub target: Option<combat::Obj>,
    pub ordered: bool,
    /// `ObjectData::targeted`.
    pub targeted: i32,
    /// Its type, as an index into [`Sim::build_types`]; `None` for the bare
    /// object.
    pub ty: Option<usize>,
    /// The type it was placed as — `BuildData::orig_type`.
    pub orig_ty: Option<usize>,
    /// `flags & 1`: in use. A dead building keeps its slot (and ejects its
    /// garrison) until `hold_frames` runs out.
    pub alive: bool,
    /// `flags & 2`: `Wall::start` has run — the footprint is committed.
    pub started: bool,
    /// `build_masks & 0x1000`: has been activated at some point.
    pub activated: bool,
    /// `build_masks & 0x100`: this building's roads want replanning, and
    /// `Build::process` will replan them on the frame `(frame + o) % 16`
    /// picks out. Set by `City::regen_roads` — `crate::roads` §1.
    pub regen_roads: bool,
    /// `ObjectData::damage`, whole hits taken. [`Building::health`] is kept
    /// equal to `hits_now − damage` and is what combat reads as the share.
    pub damage: i32,
    /// `WallData::job_counter`, `job_counter_2`, `constr_time`,
    /// `construct_hits`; `docs/CITIES.md` §3.
    pub job_counter: i32,
    pub job_counter_2: i32,
    pub constr_time: i32,
    pub construct_hits: i32,
    /// `WallData::helpers`: builders or repairers that contributed this frame.
    pub helpers: i32,
    /// `build_masks & 0x10 | 0x20`: the two-stage under-attack latch.
    pub under_attack: u8,
    /// `build_masks & 0x4000`: eject one squad a frame.
    pub eject_pending: bool,
    /// `ObjectData::hold_frames` on a dead building.
    pub hold_frames: i32,
    /// The city it belongs to, as an index into [`Sim::cities`].
    pub city: Option<usize>,
    /// The garrison chain — squad captains, in the order they entered.
    pub garrison: Vec<usize>,
    /// `BuildData::founder`.
    pub founder: Player,
    /// The four per-call construction-clock clauses, as inputs.
    pub clock: build::ClockMods,
    /// Frame of the last hit by another player, for the repair gate.
    pub hit_frame: Option<i64>,
    /// A fort's entry in [`Sim::sources`], while it projects territory.
    pub fort_source: Option<usize>,
    /// `BuildData::gather_down`'s chain: the units registered as gathering
    /// here, newest first. `docs/ORDERS.md` §6.1.
    pub gatherers: Vec<usize>,
    /// `BuildData::gather_max`: the slot count; `None` is uncapped (the
    /// non-flat count is `docs/ECONOMY.md`'s open item). A flat type gets 1.
    pub gather_max: Option<i32>,
    /// `BuildData::gather_from`: a woodcutter's or mine's resource tiles, in
    /// tiles, in the shuffled order the original keeps. An input.
    pub gather_from: Vec<Pos>,
    /// `build_masks & 0x800`: a gatherer bumped `recharging` this frame.
    pub gather_bumped: bool,
    /// The farm's tile states and growth counts (§6.5).
    pub farm: Farm,
    /// `BuildData +0x78`: this dock's slot in its owner's registry
    /// (`docs/TRANSPORT.md` §5.1), while it is active.
    pub dock_slot: Option<usize>,
    /// `WallData::ever_seen` (`Wall +0x62`) and `ever_seen_completed`
    /// (`+0x63`) — the dump prints both under those names. One bit per
    /// player: who has ever had this building's footprint in **current**
    /// line of sight, and who has had it there while it was finished.
    /// `Wall::check_ever_seen@0063ce70` grows them, and a newly arrived
    /// foreign bit is what makes the building light itself into that
    /// player's fog (`docs/VISION.md` §6.1).
    pub ever_seen: u8,
    pub ever_seen_completed: u8,
}

pub use farms::Farm;

impl Building {
    /// `hits(0)`: the site's growing figure while not active, the full one
    /// after.
    pub const fn hits_now(&self) -> i32 {
        if self.active {
            self.hits
        } else {
            self.construct_hits
        }
    }

    /// Keeps `health` — combat's share — equal to `hits_now − damage`.
    pub(crate) const fn sync_health(&mut self) {
        self.health = self.hits_now() - self.damage;
    }

    /// `WallData::is_under_attack` = `build_masks & 0x20`.
    pub const fn is_under_attack(&self) -> bool {
        self.under_attack & 0x2 != 0
    }

    /// The frame counter this building's periodic work is phased against —
    /// **its object number, not its handle** ([`Unit::phase`]'s twin).
    ///
    /// `Build::process@0061edf0:728` reads `(frame + o)`, and every period
    /// the building keeps hangs off it: the under-attack decay and the
    /// building's own attrition every 32 (`docs/CITIES.md` §1.3, §9.5), the
    /// road replan every 16 (`docs/ROADS.md` §1), the capture re-test every
    /// 64 (§7.1), the city heal every `CITY_HEAL_RATE` (§8.2) and the
    /// tower's target search (`docs/COMBAT.md` §8.6).
    ///
    /// [`Building::index`] is that `o`; the handle is this crate's `Vec`
    /// slot, which agrees with it in no game at all — `o` is per player and
    /// starts at [`BUILD_BASE`] (2000, which is 16 mod 32), the handle is
    /// global and starts at 0, and `o` is recycled by [`Sim::find_free`]
    /// while the `Vec` only grows.
    pub const fn phase(&self, frame: i64) -> i64 {
        frame + self.index as i64
    }
}

/// A unit that came out of a queue this frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Produced {
    /// Index into [`Sim::units`].
    pub unit: usize,
    pub ty: usize,
    /// Index into [`Sim::buildings`].
    pub at: usize,
}

/// What one call of `do_queue` on one slot did — the sign `Build::finished`
/// hands back, plus "not done yet".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Advanced {
    /// The counter moved and the entry is not done.
    Pending,
    /// A research entry completed: the bit is set, the entry is gone, no unit.
    Researched,
    /// A train entry completed and a unit was placed.
    Trained(Produced),
    /// Done, offered, refused — zero or negative from `finished`. The entry
    /// stays at full progress. At slot 0 this is what triggers the redirect.
    Blocked,
}

impl Sim {
    pub fn new(tuning: Tuning, world: World, players: usize) -> Sim {
        let tech_tree = tech::TechTree::new().with_tuning(&tuning);
        // `Regions::set_coastals`: the coast masks follow from the cells.
        let mut world = world;
        world.rebuild_coasts();
        let mut mesh = mesh::RoadMesh::default();
        mesh.seed(&world);
        Sim {
            pushed: Vec::new(),
            transport: vec![transport::LeaderTransport::default(); players],
            docks: vec![transport::Docks::default(); players],
            caravans: vec![caravan::Caravans::default(); players],
            armies: (0..players)
                .map(|w| army::Armies::new(w as Player))
                .collect(),
            tech: (0..players)
                .map(|_| tech::PlayerTech::new(&tech_tree))
                .collect(),
            tech_tree,
            setup: tech::Setup::STANDARD,
            staged_marks: Vec::new(),
            ai_off: false,
            trace_phases: false,
            mesh,
            plan_roads: true,
            phase_marks: Vec::new(),
            trace_costs: false,
            cost_marks: Vec::new(),
            road_marks: Vec::new(),
            players: vec![attrition::PlayerState::default(); players],
            sources: Vec::new(),
            units: Vec::new(),
            supply: vec![supply::Network::default(); players],
            at_war: vec![vec![false; players]; players],
            holdings: vec![economy::Holdings::new(); players],
            ledgers: vec![economy::Ledger::starting(&tuning); players],
            unit_types: Vec::new(),
            good_types: Vec::new(),
            unit_stats_dirty: vec![false; players],
            muster: vec![Muster::new(&tuning); players],
            redirects: cost::Redirects::RON,
            buildings: Vec::new(),
            farm_order: Vec::new(),
            table: combat::Table::uniform(0),
            spells: Vec::new(),
            rng: combat::Rng::new(0),
            projectiles: Vec::new(),
            market: market::Market::default(),
            gaia: gaia::Gaia::default(),
            art: anim::Art::default(),
            game_seed: 0,
            mods: vec![combat::Modifiers::default(); players],
            hits: Vec::new(),
            build_types: Vec::new(),
            cities: Vec::new(),
            nation: vec![city::Nation::default(); players],
            borders: vec![territory::PlayerBorders::plain(&tuning); players],
            allied: vec![vec![false; players]; players],
            treaties: vec![vec![0; players]; players],
            defeated: vec![false; players],
            lost_city_stamp: vec![None; players],
            city_tally: vec![city::Tally::default(); players],
            building_high: vec![Vec::new(); players],
            removed: Vec::new(),
            wall_stats_dirty: vec![false; players],
            // Ten slots, not `players`: gaia's animals and birds are units
            // of owners 8 and 9 and take numbers from their own bands.
            marks: vec![Marks::default(); players.max(10)],
            lobby: ai::Lobby::default(),
            leader_options: vec![stance::LeaderOptions::default(); players.max(10)],
            script_env: ai_host::ScriptEnv::default(),
            scripts: None,
            ai: vec![ai::Leader::new(); players],
            ai_speed: 1,
            coll: collide::CollGrid::new(world.width(), world.height()),
            sweep_watch: None,
            chain_heads: vec![None; (world.width() * world.height()) as usize],
            repaths: vec![0; players.max(10)],
            tuning,
            world,
            frame: 0,
        }
    }

    /// Whether `who` is at war with `p` — false for an owner outside the
    /// player table (gaia's units path through territory too).
    pub fn at_war_with(&self, who: Player, p: Player) -> bool {
        self.at_war
            .get(who as usize)
            .and_then(|row| row.get(p as usize))
            .copied()
            .unwrap_or(false)
    }

    /// Adds a player and returns their index.
    ///
    /// Seven vectors are kept in step by this. Growing one of them by hand
    /// leaves the others short, and the failure shows up as an index panic in
    /// whichever pass reaches the longest one first.
    pub fn add_player(&mut self) -> Player {
        let who = self.players.len();
        self.players.push(attrition::PlayerState::default());
        self.tech.push(tech::PlayerTech::new(&self.tech_tree));
        self.supply.push(supply::Network::default());
        self.holdings.push(economy::Holdings::new());
        self.unit_stats_dirty.push(false);
        self.ledgers.push(economy::Ledger::starting(&self.tuning));
        self.muster.push(Muster::new(&self.tuning));
        self.mods.push(combat::Modifiers::default());
        self.nation.push(city::Nation::default());
        self.borders
            .push(territory::PlayerBorders::plain(&self.tuning));
        self.defeated.push(false);
        self.lost_city_stamp.push(None);
        self.city_tally.push(city::Tally::default());
        self.building_high.push(Vec::new());
        self.wall_stats_dirty.push(false);
        self.marks.push(Marks::default());
        self.repaths.push(0);
        self.ai.push(ai::Leader::new());
        self.transport.push(transport::LeaderTransport::default());
        self.docks.push(transport::Docks::default());
        self.armies.push(army::Armies::new(
            u8::try_from(who).expect("too many players"),
        ));
        for row in &mut self.at_war {
            row.push(false);
        }
        self.at_war.push(vec![false; who + 1]);
        for row in &mut self.allied {
            row.push(false);
        }
        self.allied.push(vec![false; who + 1]);
        for row in &mut self.treaties {
            row.push(0);
        }
        self.treaties.push(vec![0; who + 1]);
        u8::try_from(who).expect("too many players")
    }

    /// Sets two players as mutual allies — both `diplos` entries at 2.
    pub fn make_allies(&mut self, a: Player, b: Player) {
        self.allied[a as usize][b as usize] = true;
        self.allied[b as usize][a as usize] = true;
    }

    /// `Leader::treaty_on@006e1190` — or `param_2` into **both** leaders'
    /// `treaties` slot for the other. The original takes the bits as an
    /// argument and every live caller passes 1, the met bit, so that is
    /// what this takes.
    ///
    /// `Leader::treaty_off@006d0370` is its opposite and clears bit 0
    /// whatever it is passed; its only caller in the whole executable is
    /// the debug console's `run_cmd`, so **nothing in a game ever clears
    /// the met bit** and there is no `treaty_off` here.
    pub fn treaty_on(&mut self, a: Player, b: Player, bits: i32) {
        let (a, b) = (a as usize, b as usize);
        if let Some(x) = self.treaties.get_mut(a).and_then(|r| r.get_mut(b)) {
            *x |= bits;
        }
        if let Some(x) = self.treaties.get_mut(b).and_then(|r| r.get_mut(a)) {
            *x |= bits;
        }
    }

    /// `Leader::meet@006e1250` — first contact. `treaty_on(other, 1)`, and
    /// then `LeaderOut::say_meet` for whichever side the console belongs to,
    /// which is the message in the corner of the screen and is not modelled.
    ///
    /// Its callers are `Wall::check_ever_seen@0063ce70` — the live one, and
    /// the reason this hangs off the fog (`docs/VISION.md` §6.2) — and
    /// `Unit::process_attrition@005e11a0`, at **three** sites — its war
    /// arm, its assassin arm, and the generic one under
    /// `get_attrition() != 0`, each after every exemption. That path is
    /// dead in every capture this crate is diffed against — 0 non-exempt
    /// outcomes over run53's 24,000 frames, measured by item 382 — and is
    /// not wired here; the seam is stated in `docs/VISION.md` §6.2.
    ///
    /// `RON_DEBUG_MEET` prints the frame each contact lands on, which is
    /// the only thing about this mechanic no capture on this disk pins:
    /// nothing dumps a leader between 7600 and 8174, so Great Lakes'
    /// original flip is bracketed to (7616, 8174] and this crate's 7944 is
    /// checked against the bracket rather than against a frame.
    pub fn meet(&mut self, a: Player, b: Player) {
        if std::env::var("RON_DEBUG_MEET").is_ok() {
            eprintln!("MEET frame {} {a} {b}", self.frame);
        }
        self.treaty_on(a, b, 1);
    }

    /// Whether `a` has allied `b` — false for an owner outside the player
    /// table, the same reading [`Sim::at_war_with`] gives.
    pub fn allied_with(&self, a: Player, b: Player) -> bool {
        self.allied
            .get(a as usize)
            .and_then(|row| row.get(b as usize))
            .copied()
            .unwrap_or(false)
    }

    /// `LeaderData::is_ally`: the same player, or allied both ways.
    ///
    /// Total in both arguments, like [`Sim::at_war_with`]. The original's
    /// `LeaderData::diplos` is `int[8]`, so `is_ally(8)` reads `treaties[0]`
    /// past the end of it — there is no faithful answer for a leader outside
    /// the table, and no caller asks for one ([`world::PLAYER_SLOTS`]). This
    /// is a guard, not a model: it keeps a gaia unit that reached a scan it
    /// should never have reached from taking the process down.
    pub fn is_ally(&self, a: Player, b: Player) -> bool {
        a == b || (self.allied_with(a, b) && self.allied_with(b, a))
    }

    /// `LeaderData::is_enemy`: different players with war declared either way.
    /// Total in both arguments; see [`Sim::is_ally`].
    pub fn is_enemy(&self, a: Player, b: Player) -> bool {
        a != b && (self.at_war_with(a, b) || self.at_war_with(b, a))
    }

    /// Registers a building type and returns its id.
    pub fn add_build_type(&mut self, ty: build::BuildType) -> usize {
        self.build_types.push(ty);
        if self.table.builds() < self.build_types.len() {
            self.table = self
                .table
                .grown(self.unit_types.len(), self.build_types.len());
        }
        self.build_types.len() - 1
    }

    /// Marks a player's economy as changed, so the next reassembly happens
    /// within eight frames rather than at the lazy 512-frame cadence.
    ///
    /// The original sets this flag from wherever the change happened — a
    /// building finished, a city captured. Anything that edits
    /// [`Sim::holdings`] should say so here, or the change will not show up
    /// for up to half a minute.
    pub fn economy_changed(&mut self, who: Player) {
        self.ledgers[who as usize].dirty = true;
    }

    /// Adds a unit, registering it as a supply source if its type is one.
    ///
    /// This is `Unit::init`'s half of the supply bookkeeping and the reason to
    /// prefer it over pushing onto `units` directly: a supply wagon that never
    /// registered supplies nobody, silently.
    pub fn add_unit(&mut self, unit: Unit) -> usize {
        let i = self.units.len();
        let owner = unit.owner as usize;
        let source = unit.kind.supply_unit;
        // A unit handed in with its number already chosen (the harness reads
        // `o` from the dump) moves the mark past it, so a unit allocated next
        // does not collide with one the original numbered higher.
        let mark = &mut self.marks[owner].unit;
        *mark = (*mark).max(unit.index.saturating_add(1));
        self.units.push(unit);
        // `Unit::init`'s transport clause (`docs/TRANSPORT.md` §3.4).
        self.transport_init_unit(i);
        // `Unit::init@00612100:376` — a type that packs is **born packed**:
        // `unit_flags2 & 4` sets `unit_masks |= 0x80000` there and nothing
        // else in the function touches the bit. It is what makes a freshly
        // trained Fisherman skip `think_fish`'s 1,024-frame head and search
        // on its first idle frame (`docs/ORDERS.md` §6.8).
        if self.units[i]
            .ty
            .is_some_and(|t| self.unit_types[t].combat.packs)
        {
            self.units[i].combat.packed = true;
        }
        if source {
            self.units[i].supply_slot = Some(self.supply[owner].list.register(i));
        }
        // `Unit::init@00612100:436` — a **land** caravan takes a slot in its
        // leader's `Caravans` list. `is_caravan` is `unit_flags2 & 8` and
        // the domain test is `UnitTypeData +0x218 == 0`, which is what
        // keeps the sea-domain Merchant Fleet out (`crate::caravan` §2).
        if self.units[i].ty.is_some_and(|ty| {
            self.unit_types[ty]
                .cols
                .flag2(crate::ai_load::uflags2::CARAVAN)
                && self.unit_types[ty].combat.domain == attrition::Domain::Land
        }) {
            let who = self.units[i].owner;
            self.units[i].caravan = self.init_caravan(who, i);
        }
        // `Object::add_to_world`: both collision indices
        // (`docs/COLLISION.md` §2, §3) **and the vision disc**.
        self.coll_add(i);
        if self.units[i].alive() && self.units[i].on_map {
            self.chain_add(i);
            // The third thing `add_to_world` does, and the one that was
            // missing here: `update_seen(0)`, the whole disc rather than
            // the ring (`docs/VISION.md` §6). [`Sim::come_out_place`] has
            // made this call since garrisoning landed; a unit *born* on the
            // map made none, so its owner's line of sight did not exist
            // until the unit first crossed a half-cell or the hundredth-
            // frame resync came round. Nothing read the plane closely
            // enough to notice until [`Sim::target_is_seen`]
            // (`docs/COMBAT.md` §31.4).
            self.update_seen(i, false);
        }
        i
    }

    /// `Objects::find_free(who, base, limit, &mark, −1)`: the object number a
    /// new object of `who` takes. The band `[base, mark)` is scanned first for
    /// a number whose object is dead (`flags & 1` clear) and holds nothing
    /// (`+0x32 == 0`) — the first such is reused; failing that the mark
    /// itself, unless it has reached `limit`, in which case there is no room
    /// and the original's callers give up. A number below the mark that no
    /// object of ours carries at all counts as free too: the only way the sim
    /// gets one is a dump that omitted it, and a dump omits the dead.
    pub fn find_free(&mut self, who: Player, base: i16, limit: i16) -> Option<i16> {
        let w = who as usize;
        let mark = if base == BUILD_BASE {
            self.marks[w].build
        } else {
            self.marks[w].unit
        };
        let mut taken = vec![false; (mark - base).max(0) as usize];
        if base == BUILD_BASE {
            for b in &self.buildings {
                let i = b.index - base;
                if b.owner == who && (base..mark).contains(&b.index) {
                    taken[i as usize] |= b.alive || !b.garrison.is_empty();
                }
            }
        } else {
            for u in &self.units {
                let i = u.index - base;
                if u.owner == who && (base..mark).contains(&u.index) {
                    taken[i as usize] |= u.alive();
                }
            }
        }
        if let Some(i) = taken.iter().position(|&t| !t) {
            return Some(base + i as i16);
        }
        if mark >= limit {
            return None;
        }
        if base == BUILD_BASE {
            self.marks[w].build += 1;
        } else {
            self.marks[w].unit += 1;
        }
        Some(mark)
    }

    /// The live unit of `who` numbered `o`, if any — the object a script or
    /// a producer holds by handle.
    pub fn unit_by_o(&self, who: Player, o: i16) -> Option<usize> {
        self.units
            .iter()
            .position(|u| u.owner == who && u.index == o && u.alive())
    }

    /// The live building of `who` numbered `o`, if any.
    pub fn building_by_o(&self, who: Player, o: i16) -> Option<usize> {
        self.buildings
            .iter()
            .position(|b| b.owner == who && b.index == o && b.alive)
    }

    /// Registers a unit type and returns its id.
    ///
    /// Every player's muster grows with it, because a type nobody has built is
    /// still a type whose count the ramp reads — as zero, which is what makes
    /// the first one cheap.
    pub fn add_unit_type(&mut self, ty: UnitType) -> usize {
        let id = self.unit_types.len();
        let groups = ty.group.map_or(0, |g| g + 1);
        let tree = ty.tree;
        self.unit_types.push(ty);
        // The combat table grows with the type space, at 100 until a
        // builder fills the new row and column.
        if self.table.width() < self.unit_types.len() {
            self.table = self
                .table
                .grown(self.unit_types.len(), self.build_types.len());
        }
        for (who, m) in self.muster.iter_mut().enumerate() {
            m.by_type.push(0);
            m.queued_by_type.push(0);
            // A type in the tree starts with the tree's bit — a free unit
            // the starting position already owns is a train job from the
            // first frame.
            m.researched
                .push(tree.is_some_and(|id| self.tech[who].tech[id]));
            if m.by_group.len() < groups {
                m.by_group.resize(groups, 0);
                m.queued_by_group.resize(groups, 0);
            }
        }
        id
    }

    /// Recomputes every player's population cap from scratch —
    /// `Leader::calc_pop_cap`, which the original also calls wholesale
    /// whenever anything that feeds it changes.
    pub fn recompute_pop_caps(&mut self) {
        for m in &mut self.muster {
            m.cap = cost::pop_cap(&self.tuning, m.military_level, m.limit, &m.bonuses);
        }
    }

    /// What a player would be charged for one of a type, per resource.
    ///
    /// The discount tail is `docs/COSTS.md`'s forty predicates and arrives as
    /// [`cost::Modifiers`] when the nation, wonder and government layers exist
    /// to produce it. Until then it is the undiscounted price, which is
    /// exactly what a stock game with no bonuses charges.
    pub fn price_of(&self, who: Player, ty: usize) -> [i32; economy::RESOURCES] {
        self.price_with(who, ty, &cost::Modifiers::default())
    }

    /// [`Sim::price_of`], with the discount tail supplied.
    pub fn price_with(
        &self,
        who: Player,
        ty: usize,
        m: &cost::Modifiers,
    ) -> [i32; economy::RESOURCES] {
        let muster = &self.muster[who as usize];
        let holdings = &self.holdings[who as usize];
        let unit = &self.unit_types[ty];
        // Built *and* ordered. `LeaderData::get_support_count` sums
        // `num_units` and `num_queued`, so five hoplites ordered at once each
        // pay a ramp step for the ones ahead of them in the queue. The time
        // ramp reads only the first of the two arrays; see
        // `docs/PRODUCTION.md`.
        let counts = cost::Counts {
            of_type: muster.by_type[ty] + muster.queued_by_type[ty],
            of_group: unit
                .group
                .map_or(0, |g| muster.by_group[g] + muster.queued_by_group[g]),
        };
        cost::charges(
            &self.tuning,
            &unit.price,
            counts,
            m,
            &holdings.available,
            &holdings.discovered,
            &self.redirects,
        )
    }

    /// Adds a production building and returns its index.
    pub fn add_building(&mut self, owner: Player, pos: Pos, capacity: usize) -> usize {
        let index = self
            .find_free(owner, BUILD_BASE, WALL_BASE)
            .expect("a player's building band is full");
        self.buildings.push(Building {
            owner,
            index,
            pos,
            queue: production::Queue::new(capacity),
            is_library: false,
            combat: None,
            hits: 0,
            health: 0,
            damage_frac: 0,
            active: true,
            regen_roads: false,
            garrison_attack: 0,
            recharging: 0,
            target: None,
            ordered: false,
            targeted: 0,
            ty: None,
            orig_ty: None,
            alive: true,
            started: true,
            activated: true,
            damage: 0,
            job_counter: 0,
            job_counter_2: 0,
            constr_time: 0,
            construct_hits: 0,
            helpers: 0,
            under_attack: 0,
            eject_pending: false,
            hold_frames: 0,
            city: None,
            garrison: Vec::new(),
            founder: owner,
            clock: build::ClockMods::default(),
            hit_frame: None,
            fort_source: None,
            gatherers: Vec::new(),
            gather_max: None,
            gather_from: Vec::new(),
            gather_bumped: false,
            dock_slot: None,
            farm: Farm::default(),
            ever_seen: 0,
            ever_seen_completed: 0,
        });
        self.buildings.len() - 1
    }

    /// Adds a library and returns its index. See [`Building::is_library`].
    pub fn add_library(&mut self, owner: Player, pos: Pos, capacity: usize) -> usize {
        let at = self.add_building(owner, pos, capacity);
        self.buildings[at].is_library = true;
        at
    }

    /// The player's first library — `LeaderData::get_first_library`, which
    /// walks the owner's buildings from the lowest index and returns the first
    /// library. (The original also asks that it be active, in a city and not
    /// unassimilated; none of those states exist here.)
    pub fn first_library(&self, who: Player) -> Option<usize> {
        self.buildings
            .iter()
            .position(|b| b.owner == who && b.is_library)
    }

    /// Where an order given at `at` actually lands. A library that is not the
    /// player's first forwards to the first — `queue_up`, `unqueue` and
    /// `do_queue` all perform this redirection, which is what puts every
    /// research job in the game on one queue.
    fn queue_home(&self, at: usize) -> usize {
        if !self.buildings[at].is_library {
            return at;
        }
        self.first_library(self.buildings[at].owner).unwrap_or(at)
    }

    /// Orders one unit of a type at a building — `Build::queue_up`.
    ///
    /// **The price is charged here**, and what was charged is written into the
    /// queue entry, which is what makes a later cancellation exact. Note what
    /// is *not* checked: the population cap. A player at the cap may queue
    /// freely, and finds out at the far end; see [`Sim::process_queues`].
    ///
    /// The two gates are the original's and in its order — affordability
    /// first, then room in the queue.
    pub fn queue_up(&mut self, at: usize, ty: usize) -> Result<usize, production::QueueFail> {
        let at = self.queue_home(at);
        let who = self.buildings[at].owner;
        // `BuildData::can_make`: not active, or in an unassimilated city, makes
        // nothing.
        if !self.buildings[at].active
            || !self.buildings[at].alive
            || self.building_unassimilated(at)
        {
            return Err(production::QueueFail::CantTrain);
        }
        // `BuildData::can_make` comes before the price: a type the tree says
        // is not available is refused before anything is charged. Only a type
        // that is in the tree can be refused by it.
        if let Some(id) = self.unit_types[ty].tree
            && self
                .tech_tree
                .type_avail(&self.setup, &self.tech[who as usize], id, true)
                == tech::NOT_AVAILABLE
        {
            return Err(production::QueueFail::CantTrain);
        }
        let charges = self.price_of(who, ty);
        let available = self.holdings[who as usize].available;
        if !cost::can_pay(&charges, &self.ledgers[who as usize], &available, 1) {
            return Err(production::QueueFail::Cost);
        }
        if !self.buildings[at].queue.has_room() {
            return Err(production::QueueFail::Full);
        }
        cost::pay(&charges, &mut self.ledgers[who as usize], &available, false);
        let slot = self.buildings[at].queue.push(ty, &charges);
        self.muster[who as usize].queued_by_type[ty] += 1;
        self.track_tree_queued(who, ty, 1);
        self.economy_changed(who);
        Ok(slot)
    }

    /// A technology's price for `who` — `TypeData::get_cost` over a tech
    /// record: `COST × TECH_COST_FACTOR`, the **science discount**, no ramp,
    /// the redirect for a good the player does not have (`docs/COSTS.md`
    /// §"The redirect"). The rest of the discount tail — being behind in
    /// ages, the lobby's tech-cost setting, the final-tech ramp — is
    /// [`cost::Modifiers`]'s and arrives as the undiscounted price until the
    /// layers that produce it exist, as [`Sim::price_of`] does for a unit.
    pub fn tech_price(&self, who: Player, t: tech::TypeId) -> [i32; economy::RESOURCES] {
        let holdings = &self.holdings[who as usize];
        let price = cost::Price {
            kind: cost::Kind::Tech,
            base: self.tech_tree.types[t].cost,
            ..cost::Price::free()
        };
        cost::charges(
            &self.tuning,
            &price,
            cost::Counts::default(),
            &cost::Modifiers {
                science_ahead: self.science_ahead(who, t),
                ..cost::Modifiers::default()
            },
            &holdings.available,
            &holdings.discovered,
            &self.redirects,
        )
    }

    /// How many Science levels `who` is ahead of technology `t` —
    /// `LeaderData::calc_science_discount`'s level term, on its **price**
    /// side.
    ///
    /// `epoch[3]` less the tech's own `AGE`, and the plus-one is the whole
    /// subtlety: the original adds one to `AGE` unless the type is an age
    /// (`0x220..0x226`) or a library epoch (`0x227..0x242`), which is
    /// [`tech::Kind::is_plain_tech`]. A non-tech gets zero, because the
    /// original guards the function on `is_tech_type` and returns the price
    /// untouched.
    ///
    /// The answer is signed. `docs/COSTS.md` §"The discounts": a player whose
    /// Science line is behind the tech's age pays *more*, out of the same
    /// expression.
    fn science_ahead(&self, who: Player, t: tech::TypeId) -> i32 {
        let kind = self.tech_tree.kind(t);
        if !kind.is_tech() {
            return 0;
        }
        let level = self.tech_tree.types[t].age + i32::from(kind.is_plain_tech());
        self.tech[who as usize].epoch[tech::Line::Science.index()] - level
    }

    /// `TypeData::research_time` for a tech, in hundredths of a frame:
    /// `JOB_TIME × 100 × RESEARCH_TICK_PREMIUM >> 8`, then the one step of
    /// `train_time`'s research tail this simulation has — the science
    /// speedup — then the floor at one. The unit-only
    /// `RESEARCH_PREMIUM_TIME` does not apply (`docs/PRODUCTION.md` §"The
    /// base, and the research step").
    ///
    /// The speedup reads the tech's `AGE` **raw**, not the price side's
    /// `AGE + 1`: see [`production::science_speedup`]. Because the target is
    /// recomputed every frame rather than stored, a Science epoch landing
    /// mid-research shortens the job already in progress — which is what
    /// puts the AI's second library entry on the original's frame rather
    /// than twenty later.
    pub fn tech_time(&self, who: Player, t: tech::TypeId) -> i32 {
        let time = self.tech_tree.types[t].job_time * production::TIME_SCALE;
        let time = (time * self.tuning.research_tick_premium) >> 8;
        let science = self.tech[who as usize].epoch[tech::Line::Science.index()];
        let tail = production::science_speedup(&self.tuning, science, self.tech_tree.types[t].age);
        production::adjusted(time, tail.as_slice()).max(production::MIN_TIME)
    }

    /// `LeaderData::researching(t)` as the scripts' `researching_tech` reads
    /// it: queued anywhere by this player.
    pub fn researching(&self, who: Player, t: tech::TypeId) -> bool {
        self.tech[who as usize].queued[t] > 0
    }

    /// Orders a technology at a building — `Build::queue_up` for a tech type.
    ///
    /// The same gates as [`Sim::queue_up`], in the same order: the building
    /// must make it (`can_make` → `queue_here` on the building's tree entry,
    /// and the tree must call it researchable), then the price, then room.
    /// `BuildQueue` holds units and techs in one list, and the library's
    /// fan-out and the stuck-head redirect treat a tech entry as the research
    /// job it is.
    pub fn queue_tech(
        &mut self,
        at: usize,
        t: tech::TypeId,
    ) -> Result<usize, production::QueueFail> {
        let at = self.queue_home(at);
        let who = self.buildings[at].owner;
        if !self.buildings[at].active
            || !self.buildings[at].alive
            || self.building_unassimilated(at)
        {
            return Err(production::QueueFail::CantTrain);
        }
        let here = self.buildings[at]
            .ty
            .and_then(|b| self.build_types[b].tree)
            .is_some_and(|b| self.tech_tree.queue_here(b, t));
        if !here
            || self
                .tech_tree
                .type_avail(&self.setup, &self.tech[who as usize], t, true)
                != tech::AVAILABLE
        {
            return Err(production::QueueFail::CantTrain);
        }
        let charges = self.tech_price(who, t);
        let available = self.holdings[who as usize].available;
        if !cost::can_pay(&charges, &self.ledgers[who as usize], &available, 1) {
            return Err(production::QueueFail::Cost);
        }
        if !self.buildings[at].queue.has_room() {
            return Err(production::QueueFail::Full);
        }
        cost::pay(&charges, &mut self.ledgers[who as usize], &available, false);
        let slot = self.buildings[at].queue.push_tech(t, &charges);
        self.tech[who as usize].queued[t] += 1;
        self.economy_changed(who);
        Ok(slot)
    }

    /// Cancels a queued order — `Build::unqueue` with a refund.
    ///
    /// The slot removed is not necessarily the one named: a cancel walks
    /// forward past entries of the same type, so cancelling one of a run
    /// removes the last of it and the one in progress keeps its progress.
    pub fn cancel(&mut self, at: usize, slot: usize) -> Option<production::Item> {
        let at = self.queue_home(at);
        let who = self.buildings[at].owner;
        let item = {
            let ledger = &mut self.ledgers[who as usize];
            self.buildings[at].queue.unqueue(slot, true, ledger)?
        };
        if let Some(t) = item.tech {
            self.tech[who as usize].queued[t] -= 1;
        } else {
            self.muster[who as usize].queued_by_type[item.ty] -= 1;
            self.track_tree_queued(who, item.ty, -1);
        }
        self.economy_changed(who);
        Some(item)
    }

    /// How long the item in `slot` at building `at` takes, right now.
    ///
    /// Recomputed every frame rather than stored, because the original
    /// recomputes it every frame: a unit of the same type completing
    /// elsewhere lengthens this one mid-build.
    pub fn queue_target(&self, at: usize, slot: usize) -> i32 {
        let b = &self.buildings[at];
        let item = &b.queue.items[slot];
        let who = b.owner;
        if let Some(t) = item.tech {
            return self.tech_time(who, t);
        }
        let muster = &self.muster[who as usize];
        let researched = muster.researched[item.ty];
        // **The tail is partitioned and the two halves never mix**
        // (`docs/PRODUCTION.md`, "The tail"): a train job takes the national
        // block and jumps over the science speedup, a research job the
        // reverse.
        //
        // A unit type whose availability bit is *clear* is a research job, and
        // it reaches `train_time`'s research block — so it takes the science
        // speedup and a trained unit does not. Its level is its first
        // prerequisite's `AGE` (`TypeData +0x30`, then `TechTypeData +0x1c8`);
        // a prerequisite that is not a technology has no `AGE` column, and
        // reads as zero here.
        let tail = if researched {
            self.train_tail(who, item.ty)
        } else {
            let science = self.tech[who as usize].epoch[tech::Line::Science.index()];
            let level = match self.unit_types[item.ty]
                .tree
                .map(|id| self.tech_tree.types[id].preq[0])
            {
                Some(tech::Preq::Of(p)) if self.tech_tree.kind(p).is_tech() => {
                    self.tech_tree.types[p].age
                }
                _ => 0,
            };
            production::science_speedup(&self.tuning, science, level)
                .into_iter()
                .collect()
        };
        production::train_time(
            &self.tuning,
            &self.unit_types[item.ty].times,
            researched,
            muster.by_type[item.ty],
            &tail,
        )
    }

    /// The national block of `ObjectData::train_time`'s tail, in the
    /// original's own order — as far as it is built.
    ///
    /// **Only the British arm is here**, and that is a scope claim rather
    /// than an oversight: the block's nine other arms are inert in every
    /// capture on disk, so each would be a predicate no diff could check,
    /// and the audit's standing lesson is that predicates are exactly where
    /// a reading goes wrong. The British arm is different — player 1 is
    /// British on both East Indies captures, and `BRITISH_SHIP_SPEED` is
    /// what the AI's Dock clock is measured against.
    ///
    /// The arms that come *before* it in the original — the lobby handicap,
    /// The President, the Mongol stable, the Japanese barracks and carrier,
    /// the Chinese citizen — are all absent from this game: the handicap is
    /// zero on both players and the rest are other nations' powers, so
    /// starting the tail here is exact rather than approximate. The ones
    /// after it are the queue's own item.
    fn train_tail(&self, who: Player, ty: usize) -> Vec<production::Adjust> {
        let mut tail = Vec::new();
        // `has_tribe_bonus(0xb)`, cached — the lobby's "No Nation Powers"
        // and the no-city gate apply through it (`crate::nations`).
        if self.nation[who as usize].british {
            let t = &self.unit_types[ty];
            if t.combat.domain == attrition::Domain::Sea {
                tail.push(production::Adjust::Faster(self.tuning.british_ship_speed));
            }
            if t.archer {
                tail.push(production::Adjust::Faster(self.tuning.british_archer_speed));
            }
            if t.anti_air {
                tail.push(production::Adjust::Faster(self.tuning.british_aa_speed));
            }
        }
        tail
    }

    /// Advances every building's queue by one frame — `Build::do_queue`.
    ///
    /// Three things about the cadence are worth stating because they would be
    /// easy to get plausibly wrong, and one of them was. It runs on **every**
    /// frame, with no phase offset by building index — unlike the per-unit
    /// upkeep in `docs/ATTRITION.md`, which is phased. **Only the first
    /// library fans out**: its queue advances `Muster::library_cities` slots at
    /// once (`LeaderData::get_building_cities`, inside `do_queue`'s library
    /// branch), a non-first library's queue never advances at all, and every
    /// other building advances slot 0 and slot 0 only. (An earlier draft
    /// applied the fan-out to every building.) And **a stuck head is not a
    /// stuck queue**: when slot 0 is done and refused, the first research
    /// entry behind it advances in its place — see [`Sim::advance_slot`].
    fn process_queues(&mut self) -> Vec<Produced> {
        let mut out = Vec::new();
        for at in 0..self.buildings.len() {
            let who = self.buildings[at].owner;
            let queued = self.buildings[at].queue.items.len();
            if queued == 0 {
                continue;
            }
            if self.buildings[at].is_library {
                if self.first_library(who) != Some(at) {
                    // `do_queue`'s first gate: a non-first library returns.
                    continue;
                }
                // The library branch recurses into `i + 1` before handling
                // its own slot, so deeper slots complete first. Walking the
                // slots in reverse is that order, and it also keeps the lower
                // indices stable when a deeper entry is removed.
                let slots =
                    production::parallel_slots(self.muster[who as usize].library_cities, queued)
                        .max(1);
                for slot in (0..slots).rev() {
                    if let Advanced::Trained(p) = self.advance_slot(at, slot) {
                        out.push(p);
                    }
                }
                continue;
            }
            match self.advance_slot(at, 0) {
                Advanced::Trained(p) => out.push(p),
                Advanced::Blocked => {
                    // The head is done and refused. `do_queue` then asks
                    // `get_next_non_unit` for the first research entry
                    // behind it and advances that one this frame instead. A
                    // train entry behind the head stays put: the cap blocks
                    // every train job, and only train jobs.
                    //
                    // The original's second fallback — on a *negative*
                    // answer only, a non-caravan or helicopter entry — is not
                    // modelled, because nothing here can be refused with a
                    // negative answer yet (no caravans, no aircraft).
                    let researched = &self.muster[who as usize].researched;
                    let next = self.buildings[at].queue.next_research(|ty| !researched[ty]);
                    if let Some(slot) = next
                        && let Advanced::Trained(p) = self.advance_slot(at, slot)
                    {
                        out.push(p);
                    }
                }
                Advanced::Pending | Advanced::Researched => {}
            }
        }
        out
    }

    /// One queue slot, for one frame.
    ///
    /// A slot that is done is offered to [`production::finished`]. A research
    /// entry — a unit type whose availability bit is clear — completes through
    /// `Leader::gain_tech`: the bit is set, the entry is removed with no
    /// refund, and **no unit is placed**; the player queues again for the
    /// first trained one. (An earlier draft spawned a unit here too.) A train
    /// entry that is refused keeps its full progress and is retried next
    /// frame, which is where a population cap actually bites.
    fn advance_slot(&mut self, at: usize, slot: usize) -> Advanced {
        let who = self.buildings[at].owner;
        if let Some(t) = self.buildings[at].queue.items[slot].tech {
            // A technology entry: research pace, and on completion
            // `Leader::gain_tech` and nothing else — no unit, no refund, no
            // population test.
            let target = self.tech_time(who, t);
            let accel = production::accel(&self.tuning, production::Job::Research, 1);
            let done =
                production::advance(&mut self.buildings[at].queue.items[slot], target, accel);
            if !done {
                return Advanced::Pending;
            }
            let mut ledger = economy::Ledger::default();
            self.buildings[at].queue.unqueue(slot, false, &mut ledger);
            self.tech[who as usize].queued[t] -= 1;
            self.gain_tech(who, t);
            self.economy_changed(who);
            return Advanced::Researched;
        }
        let ty = self.buildings[at].queue.items[slot].ty;
        let researched = self.muster[who as usize].researched[ty];
        let target = self.queue_target(at, slot);
        let accel = production::accel(&self.tuning, production::Job::for_unit(researched), 1);

        let done = production::advance(&mut self.buildings[at].queue.items[slot], target, accel);
        if !done {
            return Advanced::Pending;
        }

        let muster = &self.muster[who as usize];
        let pop = self.unit_types[ty].price.pop;
        match production::finished(researched, muster.cap, muster.control, pop, false) {
            production::Handover::Population | production::Handover::Limit => Advanced::Blocked,
            production::Handover::Researched => {
                // `gain_tech`: the bit, and nothing else. No refund, no
                // skip-forward, no unit, no population.
                let mut ledger = economy::Ledger::default();
                self.buildings[at].queue.unqueue(slot, false, &mut ledger);
                let muster = &mut self.muster[who as usize];
                muster.queued_by_type[ty] -= 1;
                muster.researched[ty] = true;
                self.track_tree_queued(who, ty, -1);
                if let Some(id) = self.unit_types[ty].tree {
                    self.gain_tech(who, id);
                }
                self.economy_changed(who);
                Advanced::Researched
            }
            production::Handover::Trained => {
                // No refund on completion, and no skip-forward: the original
                // passes false for both, and they are the same argument.
                let mut ledger = economy::Ledger::default();
                self.buildings[at].queue.unqueue(slot, false, &mut ledger);
                self.muster[who as usize].queued_by_type[ty] -= 1;
                self.track_tree_queued(who, ty, -1);
                Advanced::Trained(self.build_train(at, ty))
            }
        }
    }

    /// `Build::train@0062f9b0` — a unit born at a building.
    ///
    /// The queue is not in it: `do_queue` has already taken the entry off
    /// and dropped `num_queued` before it calls this, and the two callers
    /// that are *not* a queue — `Build::activate`'s free units
    /// (`crate::nations`) and `Build::finished`'s government hero — pay no
    /// queue at all.
    ///
    /// **The trained unit is born inside its trainer and walks out.**
    /// `Build::train` creates it at the building's own position, calls
    /// `Unit::go_inside`, and then `Unit::come_out` — which is what puts it
    /// on the exit ring rather than on the building's centre tile. The arms
    /// `train` takes before that last call — a gather-inside building that
    /// keeps its worker, a dock's boat count, the player's own text bubble
    /// — are not modelled; every trainer here lets its unit straight out.
    pub(crate) fn build_train(&mut self, at: usize, ty: usize) -> Produced {
        let who = self.buildings[at].owner;
        let unit = self.init_unit(who, ty, self.buildings[at].pos);
        // `Build::train@0062f9b0:86–101`: when the trainer's stance kind is
        // the type's, the building's own byte overrides the one
        // `Unit::init` was born with — for the whole squad, which is one
        // object as far as a stance is concerned.
        let stance = self.trained_stance(who, ty, at);
        for f in self.squad_members(unit) {
            self.units[f].stance = stance;
        }
        self.go_inside(unit, at);
        // **A scholar trained at a university stays in it** — the `else`
        // arm of `Build::train@0062f9b0`'s exit block, reached when the
        // *trainer* answers `ObjectData::is(0x1a4)`. It re-reads the
        // **trained type's** `is_scholar` (`UnitTypeData +0x4` in
        // `0x34`/`0x35`, `ObjectData::is_scholar@0046d330`), and only then
        // compares `BuildData::gather_max` (`+0x80`, a `char`, settled by
        // the type record) against `ObjectData::num_inside(1)` — the count
        // *after* the unit is in. Over the limit it ejects like anyone
        // else; at or under it calls `check_gatherers` and the scholar
        // never reaches the exit ring.
        //
        // Great Lakes 8285 is what this is: `1/44`, the game's first
        // scholar, was let out on 8272 by this crate and walked back in
        // thirteen frames later, spending
        // [`SITE_GO_INSIDE`](crate::anim::SITE_GO_INSIDE) a second time
        // where the original spends it once (`docs/CITIES.md` §6.5.2).
        // `num_inside(1)` is [`Sim::squads_inside`] here: a scholar's
        // `uber_size` is 1, so the chain count and the captain count are
        // the same number on every path this arm can take.
        let inside = self.squads_inside(at);
        let stays = self.building_ident(at) == crate::build::Ident::University
            && self.worker_of(unit) == crate::orders::Worker::Scholar
            && self.buildings[at].gather_max.is_none_or(|m| inside <= m);
        if stays {
            self.check_gatherers(at);
        } else {
            self.come_out(unit);
        }
        self.economy_changed(who);
        Produced { unit, ty, at }
    }

    /// `Objects::init_unit@0065e0c0` — **a squad is `uber_size` units, not
    /// one unit with three figures.**
    ///
    /// The loop runs `UnitTypeData::uber_size` times, and each pass is a
    /// whole `Unit::init` with its own `Guy::init_real` draw. The figures
    /// each unit gets are `crew_size + squad_size`, and `squad_size` is
    /// **written 1 by `UnitType::init` and never written again** — so the
    /// three figures of a Bowmen are three *objects*, threaded
    /// `o_up`/`o_down` as a list (the head's `o_up` is −1 and the tail's
    /// `o_down` is; a member's `o_up` is the member before it, not the
    /// captain — run17's frame 1301 has `6 → 7 → 8` exactly so).
    ///
    /// Only the head is counted: every member with an `o_up` is handed
    /// straight back to `track_unit_type(·, −1, ·)`, so `num_units`,
    /// `control` and the two running totals see one unit for the squad.
    ///
    /// SEAM: the original seats each member with `find_nearby_spot` around
    /// the captain before it returns; here they share the captain's
    /// position until [`Sim::come_out`] or a formation moves them. The
    /// search takes no draw, so the stream does not know the difference.
    pub fn init_unit(&mut self, who: Player, ty: usize, pos: Pos) -> usize {
        let n = self.unit_types[ty].combat.uber_size.max(1);
        let mut head = None;
        let mut prev = None;
        for _ in 0..n {
            // `find_free(who, 0, 2000, …)`; a full band (2,000 live units
            // of one player) is not modelled as a refusal here, so the
            // number saturates instead.
            let index = self
                .find_free(who, UNIT_BASE, BUILD_BASE)
                .unwrap_or(i16::MAX);
            let mut unit = Unit::new(who, index, pos, self.unit_types[ty].hits);
            unit.kind = self.unit_types[ty].kind;
            unit.ty = Some(ty);
            unit.type_index = self.unit_types[ty].type_index;
            // The stance is a switch on the *type's* stance kind and then,
            // when the trainer shares that kind, the trainer's own byte —
            // `Unit::init@00612100:282–309` and `Build::train@0062f9b0:86`
            // (`crate::stance`, `docs/ORDERS.md` §5.10). A trainer is what
            // [`Sim::build_train`] passes; a free-standing spawn has none.
            unit.stance = self.init_stance(who, ty);
            unit.movement.speed = self.type_speed(who, ty);
            unit.movement.turning = self.turning_for(ty);
            let u = self.add_unit(unit);
            // `Unit::init` → `Guy::init_real`: the figure's one draw.
            // (The unit's `ty` stays unset here, as it always has; the
            // piece lookup takes the type directly.)
            self.init_guys(u, Some(ty));
            match (head, prev) {
                (None, _) => {
                    head = Some(u);
                    self.track_unit_type(who, ty, 1);
                }
                (Some(h), Some(p)) => {
                    let captain = self.units[h].index;
                    self.units[u].captain = false;
                    self.units[u].combat.captain = i32::from(captain);
                    self.units[u].o_up = Some(p);
                    self.units[p].o_down = Some(u);
                    // **The squad is seated around its captain**
                    // (`Objects::init_unit@0065e0c0`, the block ending at
                    // `Unit::set_new_location(spot, 1, 1)`). Every member
                    // past the first is born on the captain's point and
                    // then moved to a free spot near it — the ring
                    // `[size · 0x30, size · 0x60 + 0xc0]`, step −1 (so an
                    // eighth of the span), the bias angle the unit's own
                    // (`UnitData +0x50`, `0x55555555` at birth), and
                    // `FILTER_NOT_ME` with this unit's own `o`/`who`.
                    //
                    // **The search takes no draw**, so the stream never
                    // knew the difference; the positions did, and they
                    // are what the golden record pins. `add hoplite
                    // who=0 4,40` puts the three at `(888, 7800)`,
                    // `(1032, 7800)`, `(936, 7944)` and `who=1 5,40` at
                    // `(1368, 7992)`, `(1512, 7992)`, `(1416, 8136)` —
                    // six coordinates this reproduces exactly, and the
                    // second captain's is the test that matters: with
                    // the squad stacked on one point the near spots stay
                    // free and the second `add` lands 18 tiles from
                    // where the original puts it (`docs/ANIM.md` §6.3,
                    // `docs/INPUT.md` §11.5).
                    //
                    // The original guards the move with a cell test —
                    // the requested point and the captain's own must
                    // share a `0x30` cell — which cannot fail here,
                    // because this crate's caller places the captain on
                    // the requested point itself. And it runs the same
                    // block for **member 0**, whose `get_captain` is its
                    // own index; the dump says the captain does not
                    // move, so the arm is taken as members 1.. only and
                    // the first member's outcome is an open question
                    // (`docs/ANIM.md` §6.4).
                    let size = self.coll_size(u);
                    let centre = self.units[h].pos;
                    let angle = self.units[u].movement.facing;
                    if let Some(spot) = self.find_nearby_spot(
                        u,
                        centre,
                        size * 0x30,
                        size * 0x60 + 0xc0,
                        -1,
                        angle,
                        None,
                    ) {
                        self.set_new_location(u, spot, true);
                    }
                }
                (Some(_), None) => unreachable!("a head is set with its own index"),
            }
            prev = Some(u);
        }
        head.expect("uber_size is at least one")
    }

    /// `Leader::gain_tech@006dcb60`'s unit-conversion loop, the **object**
    /// half of `docs/TECH.md` §7 (`6dd9bd`–`6ddbd1`).
    ///
    /// Gaining a unit type converts what you already have. The loop walks
    /// the player's object slots in order and takes every live unit whose
    /// own type is `get_graft(t.from)`, or whose `jump` chain reaches `t`,
    /// and calls `Unit::set_type(t, 0)` on it. That is how run53's three
    /// Bowmen objects become Archers on frame 6736, the frame the AI's
    /// Classical Age hands its British owner a free archer upgrade
    /// (§13's `BRITISH_ARCHER_UPGRADES` block).
    ///
    /// **The squad-size shrink** (`6ddaf0`): when the new type's
    /// `uber_size` is smaller than the old one's, a new size of **1** kills
    /// every member that is not the captain (`Unit::die`, vslot `+0x158`)
    /// and gives the captain the squad's `total_damage`; any other
    /// shrink is an error box in the original — "No support for decreasing
    /// number of guys in a squad to anything other than 2" — so it cannot
    /// be reproduced and is left alone here.
    ///
    /// SEAM: the queue arm above it (`6dd9ec`), which re-targets a
    /// **queued** entry of the old type instead of converting a standing
    /// unit — `types[t].is(0x134, 0) && u.is(0x15f, 0)` and two
    /// `track_queued` calls; no capture has a queue of the old type when
    /// its successor arrives.
    fn upgrade_units_to(&mut self, who: Player, t: tech::TypeId) {
        let Some(rec) = self.unit_record(t) else {
            return;
        };
        let from = self.tech_tree.get_graft(
            &self.setup,
            &self.tech[who as usize],
            self.tech_tree.types[t].from,
        );
        let mut list: Vec<usize> = (0..self.units.len())
            .filter(|&u| self.units[u].alive() && self.units[u].owner == who)
            .collect();
        list.sort_by_key(|&u| self.units[u].index);
        let new_uber = self.unit_types[rec].combat.uber_size.max(1);
        for u in list {
            let Some(ty) = self.unit_tree(u) else {
                continue;
            };
            if ty == t {
                continue;
            }
            let hit = Some(ty) == from
                || self
                    .tech_tree
                    .jumps_to(&self.setup, &self.tech[who as usize], ty, t);
            if !hit {
                continue;
            }
            let old_uber = self.units[u]
                .ty
                .map_or(1, |o| self.unit_types[o].combat.uber_size.max(1));
            if new_uber < old_uber {
                if new_uber != 1 {
                    // The original's own error box: not reproducible.
                    continue;
                }
                if !self.units[u].captain {
                    self.units[u].health = 0;
                    continue;
                }
                let damage: i32 = self
                    .squad_members(u)
                    .iter()
                    .map(|&f| self.units[f].max_health - self.units[f].health)
                    .sum();
                self.units[u].health = (self.units[u].max_health - damage).max(1);
            }
            self.unit_set_type(u, rec);
        }
    }

    /// `Unit::set_type@00612fa0(t, SET_TYPE_NORMAL)`, as far as this
    /// simulation carries state for it.
    ///
    /// The order is the original's: the leader's counters come **off** for
    /// the old type before the swap (`6130d1`'s `track_unit_type(·, −1,
    /// ·)` and the two running totals), the record and the derived stats
    /// change, the guys are re-initialised ([`Sim::reinit_guys`]), and the
    /// counters go **on** for the new one. Only a captain is counted, the
    /// same rule `Objects::init_unit` follows for a squad.
    ///
    /// **Damage carries.** `set_type` never writes `myhits`; the object's
    /// `damage` (`ObjectData +0x24`) is untouched by the swap, so a unit
    /// converted at half health is at the *new* type's hits minus the same
    /// damage. This crate stores the complement, so the subtraction is
    /// explicit.
    ///
    /// SEAM: the `is(0x165, 1)` CEO bit and its `update_ceo_position`, the
    /// `is(0x77, 0)` flag, `update_gpiece`, and the two vslots `+0x15c`
    /// and `+0x160` the tail calls before `update_armor`/`update_speed` —
    /// none has state here.
    pub(crate) fn unit_set_type(&mut self, u: usize, rec: usize) {
        let Some(old) = self.units[u].ty else { return };
        if old == rec {
            return;
        }
        let who = self.units[u].owner;
        let captain = self.units[u].captain;
        if captain {
            self.track_unit_type(who, old, -1);
        }
        let damage = self.units[u].max_health - self.units[u].health;
        let hits = self.unit_types[rec].hits;
        {
            let unit = &mut self.units[u];
            unit.ty = Some(rec);
            unit.max_health = hits;
            unit.health = (hits - damage).max(1);
        }
        self.units[u].kind = self.unit_types[rec].kind;
        self.units[u].type_index = self.unit_types[rec].type_index;
        self.units[u].movement.speed = self.type_speed(who, rec);
        self.units[u].movement.turning = self.turning_for(rec);
        self.reinit_guys(u, rec);
        if captain {
            self.track_unit_type(who, rec, 1);
        }
    }

    /// The squad a captain heads, captain first — `o_down` walked.
    pub fn squad_members(&self, captain: usize) -> Vec<usize> {
        let mut out = vec![captain];
        let mut cur = self.units[captain].o_down;
        let mut guard = 0;
        while let Some(u) = cur {
            out.push(u);
            cur = self.units[u].o_down;
            guard += 1;
            if guard > 16 {
                break;
            }
        }
        out
    }

    /// Installs a tech tree. Every player's tech state is reset to empty
    /// against it; call [`Sim::start_techs`] for each to lay down the starting
    /// position the way `Leader::init` does.
    pub fn set_tech_tree(&mut self, tree: tech::TechTree) {
        self.tech_tree = tree;
        for t in &mut self.tech {
            let (tribe, power, team) = (t.tribe, t.power, t.team);
            *t = tech::PlayerTech::new(&self.tech_tree);
            t.tribe = tribe;
            t.power = power;
            t.team = team;
        }
        self.sync_researched();
        // Availability is a fact about the tree, so it is wrong until there
        // is one: every player's two arrays are laid down here and then
        // maintained by [`Sim::apply_gained`].
        for who in 0..self.holdings.len().min(self.tech.len()) {
            self.sync_goods_available(u8::try_from(who).expect("too many players"));
        }
    }

    /// `Leader::init`'s tech block for one player: the ages below the
    /// starting age, the building techs that need only ages, every building
    /// and unit type whose prerequisites hold, and the nation's free starting
    /// epoch — see `docs/TECH.md`, "The starting position".
    pub fn start_techs(&mut self, who: Player) -> Vec<tech::Gained> {
        let events = self
            .tech_tree
            .start(&self.setup, &self.tuning, &mut self.tech[who as usize]);
        self.apply_gained(who);
        self.lay_starting_goods(who);
        events
    }

    /// `Leader::init@006e3930`'s last loop: the opening stockpile.
    ///
    /// The loop **zeroes all six buckets and then pays only the goods that
    /// are already available** — `type_avail(g, 1) != 0`, which for a good
    /// is `has_preq` alone (`docs/TECH.md`, "Two questions this answers").
    /// In the Ancient age that is food, timber and wealth; knowledge and
    /// metal wait for the Classical age and oil for the Industrial one, and
    /// each is paid the moment its age arrives — [`Sim::gain_tech`]'s half
    /// of the same rule. `docs/COSTS.md`, "The starting grant arrives with
    /// the good".
    ///
    /// Two nation terms ride in the same loop: the **Persians** scale the
    /// food grant by `(PERSIANS_BONUS_FOOD + 100) / 100`, and the
    /// **Greeks** — who hold knowledge from frame 0 through
    /// `GREEK_KNOWLEDGE_EARLY`, and so are paid here — have it taken
    /// straight back off them by `GREEK_DELAY_KNOWLEDGE` and re-granted at
    /// the Classical age.
    ///
    /// It **assigns**, so re-laying it is what the original's single call
    /// does: a leader whose nation or lobby is only known after the first
    /// `start_techs` gets the opening bucket laid down again, exactly as
    /// the harness re-lays the opening tech set.
    pub fn lay_starting_goods(&mut self, who: Player) {
        let w = who as usize;
        let greek = self
            .tech_tree
            .has_tribe_bonus(&self.setup, &self.tech[w], 5)
            && self.tuning.greek_delay_knowledge != 0;
        let persian = self
            .tech_tree
            .has_tribe_bonus(&self.setup, &self.tech[w], 23);
        for g in 0..economy::RESOURCES {
            let amount = match self.good_tree_type(g) {
                // Without a tree there is no availability to test, and the
                // convention the rest of the simulation keeps
                // ([`economy::Holdings::new`]) is that everything is
                // available.
                None => self.starting_good(who, g),
                Some(t) if self.type_avail(who, t) != tech::NOT_AVAILABLE => {
                    self.starting_good(who, g)
                }
                Some(_) => 0,
            };
            let amount = match g {
                0 if persian => amount * (self.tuning.persians_bonus_food + 100) / 100,
                3 if greek => 0,
                _ => amount,
            };
            self.ledgers[w].bucket[g] = amount;
        }
        self.ledgers[w].dirty = true;
    }

    /// `game->starting[g]` as one leader is paid it — the amount both
    /// `Leader::init@006e3930` and `Leader::gain_tech@006dcb60` hand to
    /// `bucket_add`, under the same three-armed scale.
    ///
    /// `Game::init_starting_resources@0058a500` fills the array before the
    /// first frame: the lobby's `STARTING_RESOURCES` row gives a `lo`/`hi`
    /// pair, `lo == 0` halves `STARTING_GOODS`, and a row with a spread
    /// draws `rand % (span x base)` per good from the sync stream
    /// (`docs/ORDERS.md` §9.4). **Only row 1 is modelled**, and it is the
    /// row every capture on disk plays — `STARTING_RESOURCES 1` in every
    /// one of the 350 `GAMEINFO` blocks across the kept dumps — where the
    /// constant is
    /// paid unscaled; run40's forty frames of food, timber and wealth are
    /// what confirm the row pays `base`. The `lo`/`hi` table itself is
    /// unread. Row 8, the unlimited lobby, is the one other arm that is
    /// read, and both callers *assign* 99,999 after the add.
    ///
    /// Conquer the World's `ctw_nomad_starting_res_x` is the third arm and
    /// is cut from v1.
    fn starting_good(&self, who: Player, g: usize) -> i32 {
        if self.lobby.resources_unlimited() {
            return economy::UNLIMITED_GOODS;
        }
        let base = self.tuning.starting_goods[g];
        // Barbarians at the Gates pays the defending team the *other* row's
        // index as a multiplier, which is what both callers write.
        if self.lobby.game_rules == 8 && self.tech[who as usize].team == 0 {
            return (self.lobby.starting_resources2 + 1) * base;
        }
        base
    }

    /// The tree entry of basic good `g`, if the tree has one: the goods come
    /// out of `resourcerules.xml` in order and the first six are
    /// [`economy::Resource`]'s own.
    fn good_tree_type(&self, g: usize) -> Option<tech::TypeId> {
        self.tech_tree
            .types
            .iter()
            .enumerate()
            .filter(|(_, d)| matches!(d.kind, tech::Kind::Good))
            .map(|(i, _)| i)
            .nth(g)
            .filter(|_| g < economy::RESOURCES)
    }

    /// `Leader::gain_tech`, as far as the simulation acts on it: the tree's
    /// bits and counters with their cascades, then the researched bit of every
    /// unit type the gain reached, and the population cap when a Military
    /// epoch arrived. The events come back so a caller can see what cascaded;
    /// `docs/TECH.md`, "`gain_tech`: owning it".
    pub fn gain_tech(&mut self, who: Player, t: tech::TypeId) -> Vec<tech::Gained> {
        let frame = self.frame;
        // `Leader::gain_tech@006dcb60` line 189, *before* the epoch counter
        // it is about to raise: a Science level re-prices every technology
        // waiting in the player's first library.
        self.reprice_library(who, t);
        // And `has_preq` on the six goods, read into a stack array at line
        // 297 — *before* the bit is set, which is what makes the grant
        // below fire once and only for a good the leader did not already
        // hold the prerequisite for.
        let had_preq: [bool; economy::RESOURCES] = std::array::from_fn(|g| {
            self.good_tree_type(g).is_some_and(|x| {
                self.tech_tree
                    .has_preq(&self.setup, &self.tech[who as usize], x)
            })
        });
        let events = self
            .tech_tree
            .gain_tech(&self.setup, &mut self.tech[who as usize], t, frame);
        self.pay_arriving_goods(who, &had_preq, &events);
        // Step 7's **object** half, in the order the cascade set the bits:
        // every standing unit of the line converts in place.
        for e in &events {
            if let tech::Gained::UnitUpgrade { to } = *e {
                self.upgrade_units_to(who, to);
            }
        }
        self.apply_gained(who);
        // **An age re-places every one of the leader's units where it
        // already stands** — `Leader::gain_tech@006dcb60:2366`, gated on
        // `TypeData::is_age_type` (the type vtable's `+0x34`, `is_epoch_type`
        // is `+0x38` and does not qualify). The loop is
        // `Unit::update_gpiece` then `Unit::set_new_location(u, u.x, u.y,
        // 1, 1)`, and it is the **snap** flags that matter here rather than
        // the graphic: `param_4` runs `Guy::set_angle(guy 0, unit->angle,
        // 1)`, which writes the figure's own facing outright, and `param_3`
        // puts guy 0 and every tracked crew figure on their points.
        //
        // For a unit standing still, or one already facing the way it is
        // going, this changes nothing — which is why a once-per-age event
        // is invisible almost everywhere. For a unit **mid-turn** it is a
        // free turn: the frame's step has already been taken along the old
        // facing, and the next one starts from the heading.
        //
        // [`Sim::tick`] runs the buildings after the unit loop, so the snap
        // lands after the frame's own step, which is what run86 shows.
        if matches!(self.tech_tree.kind(t), tech::Kind::Age(_)) {
            self.age_snap_units(who);
        }
        // `Leader::gain_tech`'s tail: `check_transport` (`docs/TRANSPORT.md`
        // §4).
        self.check_transport(who);
        events
    }

    /// The age's re-placement loop, for one leader — `Leader::gain_tech`'s
    /// `is_age_type` arm (`docs/TECH.md`, "An age snaps every figure").
    ///
    /// `Unit::set_new_location(u, u.x, u.y, 1, 1)` on a unit that is already
    /// there reduces to three writes: the figure's facing takes the unit's
    /// own `+0x50` (this crate's [`crate::Movement::heading`]), guy 0's body
    /// goes onto the unit, and each tracked crew figure is **put** on its
    /// offset rather than told to walk to it — [`Sim::crew_des`]'s `snap`.
    fn age_snap_units(&mut self, who: Player) {
        for u in 0..self.units.len() {
            if !self.units[u].alive() || self.units[u].owner != who {
                continue;
            }
            let pos = self.units[u].pos;
            let angle = self.units[u].movement.heading;
            self.units[u].movement.set_facing(angle);
            self.units[u].movement.body.pos = pos;
            self.crew_des(u, pos, angle, true);
        }
    }

    /// `Leader::gain_tech@006dcb60`'s goods loop: **the starting grant
    /// arrives with the good, not at `Leader::init`.**
    ///
    /// The original walks the six basic goods on every gain and pays
    /// `bucket_add(g, game->starting[g])` for one that passes both halves
    /// of a two-part gate:
    ///
    /// - the leader did **not** hold `has_preq(g)` before this call — the
    ///   stack array read at line 297, ahead of the bit — which is what
    ///   stops a re-gain of a tech already owned from paying twice; and
    /// - `goodtypes[g] + 0x30` — `TypeData::preq[0]`, the good's **first**
    ///   prerequisite, read raw off the type record rather than through
    ///   `get_preq`'s substitutions — **is the tech just gained**.
    ///
    /// So knowledge and metal are paid as the Classical age lands and oil
    /// as the Industrial one does, and a nation that holds a good early
    /// through a `has_preq` waiver — the Germans' metal, the Greeks'
    /// knowledge — fails the first half here and was paid at
    /// [`Sim::lay_starting_goods`] instead. The Greeks are the one case
    /// where both halves miss, because `GREEK_DELAY_KNOWLEDGE` takes the
    /// init grant back off them; the original's own branch at the foot of
    /// the loop re-grants it on the Classical age, and that is the
    /// `greek` arm below.
    ///
    /// `docs/COSTS.md`, "The starting grant arrives with the good".
    fn pay_arriving_goods(
        &mut self,
        who: Player,
        had_preq: &[bool; economy::RESOURCES],
        events: &[tech::Gained],
    ) {
        let w = who as usize;
        let greek = self
            .tech_tree
            .has_tribe_bonus(&self.setup, &self.tech[w], 5)
            && self.tuning.greek_delay_knowledge != 0;
        for e in events {
            let tech::Gained::Type(x) = *e else { continue };
            for (g, &held) in had_preq.iter().enumerate() {
                if held {
                    continue;
                }
                let Some(id) = self.good_tree_type(g) else {
                    continue;
                };
                if self.tech_tree.types[id].preq[0] != tech::Preq::Of(x) {
                    continue;
                }
                let amount = self.starting_good(who, g);
                self.ledgers[w].bucket[g] += amount;
                self.ledgers[w].dirty = true;
            }
            // The Greeks' delayed knowledge, re-granted on the first age.
            if greek && matches!(self.tech_tree.kind(x), tech::Kind::Age(0)) {
                let amount = self.starting_good(who, 3);
                self.ledgers[w].bucket[3] += amount;
                self.ledgers[w].dirty = true;
            }
        }
    }

    /// `Leader::gain_tech`'s refund pass: gaining a **Science** library tech
    /// re-prices every technology queued in the player's first library and
    /// hands back the difference — `Build::refund_cost@00620490`,
    /// `docs/COSTS.md`, "The discounts".
    ///
    /// The gained tech is skipped, and so is anything queued that is not a
    /// technology. Nothing else in the executable calls `refund_cost`.
    fn reprice_library(&mut self, who: Player, t: tech::TypeId) {
        let tech::Kind::Epoch { line, .. } = self.tech_tree.kind(t) else {
            return;
        };
        if line != tech::Line::Science {
            return;
        }
        let Some(lib) = self.first_library(who) else {
            return;
        };
        let level = self.tech[who as usize].epoch[tech::Line::Science.index()];
        for slot in 0..self.buildings[lib].queue.items.len() {
            let item = &self.buildings[lib].queue.items[slot];
            let Some(id) = item.tech else { continue };
            if id == t || !self.tech_tree.kind(id).is_tech() {
                continue;
            }
            let age = self.tech_tree.types[id].age;
            let mut back = [0i32; economy::RESOURCES];
            let item = &mut self.buildings[lib].queue.items[slot];
            for pair in 0..production::PAIRS {
                let good = item.good[pair];
                if good < 0 {
                    continue;
                }
                let paid = i32::from(item.cost[pair]);
                let now = cost::reprice(&self.tuning, paid, level, age);
                back[good as usize] += paid - now;
                item.cost[pair] = now as i16;
            }
            let ledger = &mut self.ledgers[who as usize];
            for (g, amount) in back.iter().enumerate() {
                ledger.bucket[g] += amount;
            }
        }
    }

    /// `LeaderData::type_avail(t, 1)` for a tree entry: 0, 2 or 4.
    pub fn type_avail(&self, who: Player, t: tech::TypeId) -> i32 {
        self.tech_tree
            .type_avail(&self.setup, &self.tech[who as usize], t, true)
    }

    /// After the tree changed: the researched bits follow it, and so does the
    /// Military level the population cap is indexed by — and the wall stats
    /// are stale, since a tech can change a site's clock and a building's
    /// hits.
    fn apply_gained(&mut self, who: Player) {
        self.wall_stats_dirty[who as usize] = true;
        self.sync_researched();
        self.sync_goods_available(who);
        let level = self.tech[who as usize].military_level();
        if self.muster[who as usize].military_level != level {
            self.muster[who as usize].military_level = level;
            self.recompute_pop_caps();
        }
        // The Military epoch's sibling: a **Civic** epoch widens every one of
        // this leader's cities, because `compute_reg_territory` reads the
        // level rather than a cached bonus. Guarded on the table actually
        // changing so an ordinary tech does not repaint 3,600 cells.
        if self.borders[who as usize] != self.player_borders(who) {
            self.sync_territory();
        }
    }

    /// Which of the six basic resources this player may spend, and which they
    /// hold the prerequisite for — the two arrays `crates/sim/src/cost.rs`
    /// steers its redirect tables by.
    ///
    /// `docs/COSTS.md`, "Three of the six resources are not available from
    /// the start": `resourcerules.xml` gives Knowledge and Metal the
    /// Classical Age as their prerequisite and Oil the Industrial Age, so a
    /// price written in one of those is charged somewhere else until the age
    /// arrives — Knowledge in food at three halves, Metal in timber at five
    /// quarters, Oil in metal at three halves and then on into timber. The
    /// test is `type_avail`, which `docs/TECH.md` ("Two questions this
    /// answers") settles for a good: `has_preq` — slot 0 is the unlocking
    /// age, slot 1 nothing, `obs` never — and a `type_eligible` that is
    /// always 4. So `available` and `discovered` agree for a good, which is
    /// why the obsolete table never fires in a stock game.
    ///
    /// The goods come out of the tree in `resourcerules.xml` order, and the
    /// first six of them are [`economy::Resource`]'s own.
    fn sync_goods_available(&mut self, who: Player) {
        let w = who as usize;
        let goods: Vec<tech::TypeId> = self
            .tech_tree
            .types
            .iter()
            .enumerate()
            .filter(|(_, d)| matches!(d.kind, tech::Kind::Good))
            .map(|(i, _)| i)
            .take(economy::RESOURCES)
            .collect();
        for (g, t) in goods.into_iter().enumerate() {
            let p = &self.tech[w];
            let discovered = self.tech_tree.has_preq(&self.setup, p, t);
            let available = self.tech_tree.type_avail(&self.setup, p, t, true) == tech::AVAILABLE;
            self.holdings[w].discovered[g] = discovered;
            self.holdings[w].available[g] = available;
        }
    }

    /// The tree owns the bit for every unit type that is in it; `Muster`
    /// keeps a copy because production reads it per slot per frame.
    fn sync_researched(&mut self) {
        for (who, tech) in self.tech.iter().enumerate() {
            for (ty, ut) in self.unit_types.iter().enumerate() {
                if let Some(id) = ut.tree {
                    self.muster[who].researched[ty] = tech.tech[id];
                }
            }
        }
    }

    /// [`Sim::track_tree_queued`], reachable from a test — the ramp's
    /// group counter has six writers and this is how one of them is
    /// exercised without standing a whole city up.
    #[cfg(test)]
    pub(crate) fn track_queued_for_test(&mut self, who: Player, ty: usize, delta: i32) {
        self.track_tree_queued(who, ty, delta);
    }

    /// The tree's own queued count for the government-pairing rule, and the
    /// ramp's group count beside it.
    fn track_tree_queued(&mut self, who: Player, ty: usize, delta: i32) {
        if let Some(id) = self.unit_types[ty].tree {
            self.tech[who as usize].queued[id] += delta;
        }
        // The ramp's group half, kept on the same statement as the type
        // half it sits beside ([`Muster::queued_by_group`]). Every site
        // that moves `queued_by_type` calls this, which is what makes the
        // two counts agree by construction rather than by inspection.
        if let Some(g) = self.unit_types[ty].group {
            self.muster[who as usize].queued_by_group[g] += delta;
        }
    }

    /// `Leader::track_unit_type`, and the two lines every one of its call
    /// sites carries beside it: `control` and **`active`**.
    ///
    /// `active` is not a census output. `Unit::set_type@00612fa0:74,284`
    /// moves `num_units`, `control` and `active` in one guarded block in
    /// each direction, and `Objects::init_unit@0065e0c0:92,174` undoes all
    /// three when the new unit turns out to be a squad follower; the
    /// sweep's own recount (`Leader::plan_strategy@006b9620:127,508`) only
    /// re-derives every two hundred frames what these writers have been
    /// keeping live. Modelling the recount alone left `active` a stale
    /// snapshot — 31 against the original's 32 on 62 of run91's 86 blocks,
    /// item 303. `docs/AI.md` §37.
    ///
    /// Keeping the four on one statement is the same reasoning as
    /// [`Sim::track_tree_queued`]: they agree by construction rather than
    /// by inspection.
    pub fn track_unit_type(&mut self, who: Player, ty: usize, delta: i32) {
        let pop = self.unit_types[ty].price.pop;
        let group = self.unit_types[ty].group;
        let muster = &mut self.muster[who as usize];
        muster.by_type[ty] += delta;
        muster.control += pop * delta;
        if let Some(g) = group {
            muster.by_group[g] += delta;
        }
        self.ai[who as usize].census.active += delta;
    }

    /// Builds one unit of a type, if the player can pay for it and has room.
    ///
    /// This is the *instant* path — the price and the unit in one call, with
    /// no queue and no time. [`Sim::queue_up`] is the ordinary one.
    ///
    /// The order is the original's and it is observable: **the population is
    /// checked before the price**, so a player at the cap is refused without
    /// being charged. Everything after that is bookkeeping the ramp depends
    /// on — a count that is not incremented leaves the next one just as cheap.
    pub fn produce(&mut self, who: Player, ty: usize, pos: Pos) -> Result<usize, Refused> {
        let charges = self.price_of(who, ty);
        let pop = self.unit_types[ty].price.pop;
        let muster = &self.muster[who as usize];
        if cost::exceeds_population(muster.cap, muster.control, pop) {
            return Err(Refused::Population);
        }

        let available = self.holdings[who as usize].available;
        if !cost::can_pay(&charges, &self.ledgers[who as usize], &available, 1) {
            return Err(Refused::Cost);
        }
        cost::pay(&charges, &mut self.ledgers[who as usize], &available, false);

        self.track_unit_type(who, ty, 1);

        let index = self
            .find_free(who, UNIT_BASE, BUILD_BASE)
            .unwrap_or(i16::MAX);
        let mut unit = Unit::new(who, index, pos, self.unit_types[ty].hits);
        unit.kind = self.unit_types[ty].kind;
        unit.ty = Some(ty);
        let at = self.add_unit(unit);
        self.init_guys(at, Some(ty));
        self.economy_changed(who);
        Ok(at)
    }

    /// Gives a dead unit's supply slot back — `Unit::close` — and takes it
    /// out of both collision indices, which is `Object::remove_from_world`
    /// (`docs/COLLISION.md` §2, §3). This is every death path's tail.
    fn close_supply(&mut self, unit: usize) {
        let owner = self.units[unit].owner as usize;
        if let Some(slot) = self.units[unit].supply_slot.take() {
            self.supply[owner].list.close(slot);
        }
        // `Unit::close@0060ee50`'s own: the route goes back with the unit.
        if let Some(slot) = self.units[unit].caravan.take() {
            let who = self.units[unit].owner;
            self.close_caravan(who, slot);
        }
        self.coll_remove(unit);
        self.chain_remove(unit);
    }

    /// Whether anything of `owner`'s supplies a unit standing at `at`.
    ///
    /// A source's liveness is resolved here rather than stored, exactly as the
    /// original resolves it: the supply record holds an index and nothing
    /// else, so a wagon that dies or boards a transport stops supplying with
    /// no bookkeeping anywhere.
    pub fn supplied_at(&self, owner: Player, at: Pos) -> bool {
        self.supply[owner as usize].supplies(&self.tuning, at, |u| {
            self.units.get(u).map(|w| supply::Wagon {
                pos: w.pos,
                active: w.alive(),
                on_map: w.on_map,
            })
        })
    }

    /// `Leader::set_diplo(whom, level)` as the console's three verbs reach
    /// it — `ally` 2, `peace` 1, `war` 0 (`run_cmd` cases 0x2c–0x2e,
    /// `docs/INPUT.md` §11). The table this crate keeps is two booleans,
    /// so only the ends of that scale are modelled: 0 is
    /// [`Sim::declare_war`], 2 sets the alliance both ways, and 1 clears
    /// both. SEAM: the original's `diplos` byte carries more than three
    /// values (met, tribute state) and none of that is here.
    pub fn set_diplo(&mut self, a: Player, b: Player, level: i32) {
        let (x, y) = (a as usize, b as usize);
        if x >= self.at_war.len() || y >= self.at_war.len() || x == y {
            return;
        }
        match level {
            0 => {
                self.allied[x][y] = false;
                self.allied[y][x] = false;
                self.declare_war(a, b);
            }
            2 => {
                self.at_war[x][y] = false;
                self.at_war[y][x] = false;
                self.allied[x][y] = true;
                self.allied[y][x] = true;
            }
            _ => {
                self.at_war[x][y] = false;
                self.at_war[y][x] = false;
                self.allied[x][y] = false;
                self.allied[y][x] = false;
            }
        }
    }

    /// `Leader::set_age(n)` on one leader, which is all the `age` cheat
    /// does — the four epochs stay where they were (`docs/INPUT.md` §11).
    pub fn set_leader_age(&mut self, who: Player, n: i32) -> Vec<tech::Gained> {
        let frame = self.frame;
        let w = who as usize;
        if w >= self.tech.len() {
            return Vec::new();
        }
        self.tech_tree
            .set_age(&self.setup, &mut self.tech[w], n, frame)
    }

    /// `Leader::set_epoch(line, level)` — the `military`, `civic`,
    /// `commerce` and `science` cheats, and the four `library` spends.
    pub fn set_leader_epoch(&mut self, who: Player, cat: i32, level: i32) -> Vec<tech::Gained> {
        let frame = self.frame;
        let w = who as usize;
        let Some(line) = tech::Line::of(cat) else {
            return Vec::new();
        };
        if w >= self.tech.len() {
            return Vec::new();
        }
        self.tech_tree
            .set_epoch(&self.setup, &mut self.tech[w], line, level, frame)
    }

    /// Sets two players at war with each other.
    pub fn declare_war(&mut self, a: Player, b: Player) {
        let changed = !self.at_war[a as usize][b as usize];
        self.at_war[a as usize][b as usize] = true;
        self.at_war[b as usize][a as usize] = true;
        // `Leader::set_diplo` → `Armies::diplo_change` on the declarer
        // (`docs/ARMY.md` §15.9).
        if changed && (a as usize) < self.armies.len() {
            self.armies_diplo_change(a);
        }
    }

    /// Copies the territory the border pass produced into the holdings the
    /// territory tax reads, and tells the world how much land there is.
    ///
    /// The original keeps both numbers itself — `LeaderData::territory` and
    /// `WorldData::land_size` — and the tax is their ratio. **The unit
    /// cancels**, so counting cells here where the original counts tiles gives
    /// the same wealth; what would not survive is counting one of them in one
    /// unit and the other in the other.
    ///
    /// Marks every player's economy dirty, since this is exactly the kind of
    /// change the original's flag exists for.
    pub fn update_territory_holdings(&mut self) {
        let mut land = 0;
        let mut owned = vec![0; self.players.len()];
        for (region, terrain) in self.world.regions().collect::<Vec<_>>() {
            if terrain != Terrain::Land {
                continue;
            }
            for cell in self.world.cells_in(region) {
                land += 1;
                if let Some(p) = self.world.owner(cell).player() {
                    owned[p as usize] += 1;
                }
            }
        }
        for (who, holdings) in self.holdings.iter_mut().enumerate() {
            holdings.territory = owned[who];
            holdings.land_size = land;
        }
        for l in &mut self.ledgers {
            l.dirty = true;
        }
    }

    /// The per-player half of the border bonus, read from live state —
    /// `World::compute_reg_territory@006b0bb0`'s own opening loop, which
    /// rebuilds all eight rows on every pass.
    ///
    /// The Civic level is `data_encrypted->epoch[1] ^ 0x63187`, the same
    /// field `LeaderData::get_city_limit` reads, and it feeds both
    /// `CIVIC_UPGRADE_TERR` and the Russians' per-step flat bonus.
    ///
    /// **The gem term is live**, and it is the one of the four seams a
    /// capture ever reached: Great Lakes' AI has a Merchant on the map's
    /// one Gems deposit by frame 23,999 (run80), so its border is wider
    /// than a gem-less table's. `compute_reg_territory`'s arm is an
    /// inlined `LeaderData::has_rare(`[`economy::GEMS`]`)` over
    /// `rare | rare_conquest`, which is what [`Sim::has_rare`] answers.
    ///
    /// **Still seams**, because nothing here models them: the temple and
    /// fort border levels (`has_preq(TEMPLEBORDERS2..4)`,
    /// `has_preq(FORTBORDERS2..4)` — bonus types `0x2c8..0x2ca` and
    /// `0x2d1..0x2d3`, which this crate loads as `bonus_preqs` and does not
    /// expose), the Colosseum and Eiffel Tower, and the AI handicap
    /// allowance. All three are inert on every capture so far — no player
    /// in one holds a Temple, a Fort or either wonder, and the lobbies run
    /// at handicap 0 — and the first two are blocked at their
    /// prerequisite rather than merely unreached (`docs/ATTRITION.md`,
    /// "Territory").
    fn player_borders(&self, who: Player) -> territory::PlayerBorders {
        let w = who as usize;
        let civic = self.tech[w].epoch[tech::Line::Civic.index()].max(0);
        let n = &self.nation[w];
        territory::PlayerBorders::new(
            &self.tuning,
            civic as usize,
            1,
            1,
            &territory::Wonders {
                colosseum: false,
                tikal: n.tikal,
                eiffel_tower: false,
            },
            &territory::NationBonuses {
                roman: n.romans,
                russian: n.russians,
                gems: self.has_rare(who, economy::GEMS),
                civic,
            },
            0,
        )
    }

    /// Recomputes every border from scratch.
    ///
    /// Wholesale, which is what the original does at game setup and **not**
    /// what it does in play: there, `GameDaemon::check_borders` recomputes
    /// invalidated regions on a shared budget of 256 cells a frame, so a cell
    /// can carry stale ownership for up to `land cells / 256` frames after a
    /// city changes hands. Steady-state ownership is identical; the transient
    /// is a deliberate simplification until a recorded-game diff says it
    /// matters. See `docs/ATTRITION.md`, "Territory".
    pub fn recompute_territory(&mut self) {
        let players = u8::try_from(self.players.len()).expect("too many players");
        territory::compute_all_territory(&mut self.world, &self.tuning, &self.sources, players);
    }

    /// Records the stream's word *before* the phase named runs, so the
    /// draws between two marks belong to the earlier one. Nothing at all
    /// unless [`Sim::trace_phases`] is set.
    ///
    /// `Sim::tick` marks its phases and each unit the loop visits; a
    /// **mechanic** may mark its own draw sites on top of that, under the
    /// original's own offsets (`crate::scout` is the first), which is what
    /// turns the fold from a count into a sequence comparable with
    /// `rondata::trace`'s.
    pub(crate) fn mark(&mut self, phase: &str) {
        if self.trace_phases {
            self.phase_marks.push((phase.to_string(), self.rng.seed));
        }
    }

    /// `GameAccess::spelltypes[spell]` — the craft's row, or `None` when
    /// the index is not a craft or the table was never loaded.
    pub fn spell(&self, spell: i32) -> Option<orders::SpellType> {
        let i = usize::try_from(spell - orders::spell::FIRST).ok()?;
        self.spells.get(i).copied()
    }

    /// `SpellTypeData::get_job_time(o, who)@00675800` — the row's own
    /// `JOB_TIME`, which is the whole answer for every craft this crate
    /// casts.
    ///
    /// SEAM, and it is a list rather than a shrug: the function adjusts
    /// nine of the fifty-five rows and **none of them is one this crate
    /// issues**. `0x27d` Entrench takes the French tribe bonus and
    /// Antipater's rate; `0x275` Bribe and `0x27f` Informer halve under
    /// `SPIES_CRAFT_FASTER`; `0x28b`/`0x28c`, the siege pack pair, take
    /// the Turkish bonus, Napoleon's, a half for two type masks and a
    /// quarter for a third; `0x28d`/`0x28e`, the machine gun's, halve for
    /// one; `0x280`/`0x281`, Sabotage and Sniper, halve under a tribe
    /// bonus and flatten to 10 for one mask. The fishing boat's `0x292`
    /// and the transport `0x28a` are named by no arm, so the record's
    /// field is the number — run58's forty frames between the queue on
    /// 4948 and the unpack on 4989 (`docs/ORDERS.md` §6.9).
    pub fn spell_job_time(&self, spell: i32) -> i16 {
        self.spell(spell).map_or(0, |s| s.job_time)
    }

    /// `GameAccess::rnd(n)@0043cca0` — `Random::get(game_random, 0, 0xffff)
    /// % n`, and **zero without a draw** when `n <= 1`. The early return is
    /// the whole reason this is a function rather than a `%`: a caller that
    /// passes a one-wide span spends nothing, and a frame's draw count is
    /// what the diff compares (`docs/ORDERS.md` §6.5).
    pub(crate) fn rnd(&mut self, n: i32) -> i32 {
        if n <= 1 {
            return 0;
        }
        self.rng.roll() % n
    }

    /// Advances one frame.
    ///
    /// The cadence is the part of this mechanic that would be easiest to get
    /// plausibly wrong, and it is not a countdown. `Unit::process` recomputes
    /// the period every 32 frames and, on **every** frame, applies damage when
    /// `(frame + unit index) % period == 0`. Two things follow that a
    /// countdown would not give:
    ///
    /// - **The damage is phase-locked to the global frame**, not to when the
    ///   unit entered hostile ground. Change a period from 48 to 24 and the
    ///   unit re-locks to the 24-frame grid immediately, mid-interval.
    /// - **Leaving hostile territory does not stop the bleeding at once.** The
    ///   stale period survives until the next refresh, so a unit can take one
    ///   more tick up to 32 frames after walking out — and, symmetrically,
    ///   takes nothing for up to 32 frames after walking in.
    pub fn tick(&mut self) -> Vec<Tick> {
        let frame = self.frame;
        let mut events = Vec::new();
        // The staged input's marks, taken before the clear and put back at
        // the head of the frame's own — `do_frame`'s entry, which is where
        // the cheat channel spends them (`docs/INPUT.md` §11.1).
        let staged: Vec<(String, u32)> = self.staged_marks.drain(..).collect();
        if self.trace_phases {
            self.phase_marks.clear();
            self.phase_marks.extend(staged);
        }
        self.mark("income");

        // Income first. `Game::do_frame` runs `Leaders::process_all` before
        // `Objects::process_all`, so every player is paid for the frame before
        // any unit in it moves, fights or bleeds. That ordering is observable:
        // a citizen that dies this frame was already paid for it.
        for who in 0..self.players.len() {
            let player = u8::try_from(who).expect("too many players");
            // `Leader::calc_gather` assembles its inputs from the live state
            // inside the same gate it recomputes under, so the holdings are
            // rebuilt only on the frames the rate is actually reassembled —
            // which makes income appear on exactly the frame it does there
            // (`crates/sim/src/holdings.rs`).
            if self.holdings_due(player, frame) {
                self.assemble_holdings(player);
            }
            economy::process(
                &self.tuning,
                &mut self.ledgers[who],
                &self.holdings[who],
                player,
                frame,
            );
            // `Leader::gather`'s own tail, every frame: `rare = rare_owned |
            // rare_conquest`, and a change raises **`0xc000000`**
            // (`006ce280`, the `operator!=` arm's last line) — the
            // unit-stats pass below *and* the wall-stats one, which
            // re-bakes every unfinished building's construction clock.
            // `rare_owned` only moves on a recompute frame, so this can
            // only fire on one — but the original tests it every frame and
            // so does this ([`crate::rares`]).
            //
            // **The wall half arrived with item 261.** It was `0x4000000`
            // alone here, and the missing half is a whole rare's building
            // bonus never reaching a site: Great Lakes' AI collects
            // Tobacco on 6751 and the original's Tower drops from 100000
            // to **90909** on 6752, finishing at 7176 where this crate's
            // ran on (`docs/CITIES.md` §3.2).
            let rare = self.holdings[who].rare_owned;
            if self.ledgers[who].rare != rare {
                // `Leader::calc_gather`'s own border arm, and it is one
                // bit wide: the writer compares `rare.ptr[2] >> 7` before
                // and after and calls `Regions::fix_all_borders` only when
                // **the gem bit** moved, because no other rare is in the
                // border table. `fix_all_borders` merely zeroes each
                // region's border stamp, so the recompute it schedules
                // reads the *new* mask; this crate recomputes on the spot
                // and therefore assigns first (`docs/ATTRITION.md`,
                // "Territory"; `docs/audit/2026-09-05-economy-vs-code.md`
                // R18).
                let gem = 1u64 << (economy::GEMS - economy::BASE_RARE);
                let gems_moved = (self.ledgers[who].rare ^ rare) & gem != 0;
                self.ledgers[who].rare = rare;
                self.unit_stats_dirty[who] = true;
                self.wall_stats_dirty[who] = true;
                if gems_moved {
                    self.sync_territory();
                }
            }
            // `Leader::process` also answers the wall-stats dirty flag here,
            // before any building is touched.
            if self.wall_stats_dirty[who] {
                self.calc_wall_stats(player);
            }
            // And then `0x4000000` — `Leader::calc_unit_stats`, which is
            // where a rare that appeared in this frame's `Leader::gather`
            // reaches the units. `Leaders::process_all` runs before
            // `Objects::process_all`, so the new speed is the one the
            // frame's own step uses (`docs/ECONOMY.md`, "What an owned
            // rare does").
            if self.unit_stats_dirty[who] {
                self.unit_stats_dirty[who] = false;
                self.calc_unit_stats(player);
            }
        }

        self.mark("strategy_all");
        // `Leaders::strategy_all` — the production AI, between the income
        // and the objects (`Game::do_frame` line 267; `docs/AI.md` §2.1).
        self.strategy_all();

        // `GameDaemon::process_all`'s **first** act, before anything it
        // does for vision or the market: every player's repath pressure is
        // halved and snapped to zero under three
        // (`docs/PATHFINDER.md` §8). It is what keeps the collision
        // throttle of `docs/COLLISION.md` §6 step 6 a *rate* rather than a
        // lifetime count — without it the counter only climbs, and a unit
        // that has collided four times in a game is throttled for the rest
        // of it. It draws nothing, so it takes no mark.
        for r in &mut self.repaths {
            *r /= 2;
            if *r < 3 {
                *r = 0;
            }
        }

        // `GameDaemon::process_all` → `calc_danger`, the third thing that
        // function does: the danger map, rebuilt from scratch every two
        // hundredth frame (`crate::danger`). It draws nothing, so it takes
        // no mark.
        if frame % crate::danger::PERIOD == 0 {
            self.calc_danger();
        }

        // `GameDaemon::process_all` → `update_all_seen`, the fourth thing
        // that function does and the one before `calc_markets`
        // (`docs/VISION.md` §6). It draws nothing, so it takes no mark.
        if frame % 100 == 0x21 {
            self.update_all_seen();
        }

        self.mark("markets");
        // `GameDaemon::process_all` → `calc_markets`: the market's price
        // cycle, between the AI and the objects (`docs/SYNC.md` §3.1). On
        // frame 0 it is eighteen draws, the frame's 2nd to 19th.
        self.calc_markets(frame);

        self.mark("armies");
        // `Armies::process_all` — after the daemon, before the objects
        // (`Game::do_frame` line 272; `docs/ARMY.md` §5).
        self.armies_process_all();

        self.mark("unit-loop");
        // `Objects::process_all` rotates the owners: slot `(frame + i) % 10`
        // goes `i`-th, so player `frame % 10`'s units run first this frame
        // and, within an owner, in **object** order (`docs/SYNC.md` §3.2).
        // The sync stream sees the rotation — at frame 101 the AI's farmers
        // draw their re-targets before the human's (§4.1) — so the order of
        // the visits is the order of the draws. An owner outside the ten
        // slots (none today) would go last, in index order.
        //
        // **The bound is re-read every iteration, and that is observable.**
        // `Objects::process_all@0065dce0`'s inner loop tests
        // `o < unit_mark[who]` at the bottom out of the array rather than
        // out of a local, so a unit created *inside* the loop with an
        // object number above the one being walked is processed on the
        // frame it is born. The transport barge is exactly that unit: it is
        // cast by `Unit::do_cast` from a unit whose `o` is lower, takes the
        // caster's move order and steps — which sets its guy's animation to
        // the walk before `Objects::inc_time` reaches it, so its clock
        // never wraps. Every other newborn in run57 is a **trained** unit,
        // born in `Build::do_queue` in the second loop, and every one of
        // them does wrap on its birth frame (`docs/TRANSPORT.md` §13).
        let mut slots: Vec<std::collections::BTreeMap<i16, usize>> =
            vec![std::collections::BTreeMap::new(); 10];
        // `unit_mark[who]`, and the highest slot actually filled — a test
        // that seats a unit by hand does not go through [`Sim::find_free`],
        // so the mark alone would leave it unwalked.
        let mut bound = [0i16; 10];
        for (w, b) in bound.iter_mut().enumerate() {
            *b = self.marks[w].unit;
        }
        for i in 0..self.units.len() {
            let u = &self.units[i];
            if u.owner < 10 {
                let w = u.owner as usize;
                bound[w] = bound[w].max(u.index.saturating_add(1));
                if u.alive() {
                    slots[w].insert(u.index, i);
                }
            }
        }
        let mut seen = self.units.len();
        for slot in 0..10 {
            let who = u8::try_from((frame + slot).rem_euclid(10)).expect("a slot");
            let w = who as usize;
            let mut o: i16 = 0;
            while o < bound[w] {
                if let Some(&i) = slots[w].get(&o) {
                    self.process_unit(i, frame, &mut events);
                }
                // The re-read: anything the step just created joins the
                // walk, in its own owner's band and at its own slot.
                if self.units.len() > seen {
                    for j in seen..self.units.len() {
                        let u = &self.units[j];
                        if u.owner < 10 {
                            let b = u.owner as usize;
                            bound[b] = bound[b].max(u.index.saturating_add(1));
                            if u.alive() {
                                slots[b].insert(u.index, j);
                            }
                        }
                    }
                    seen = self.units.len();
                }
                bound[w] = bound[w].max(self.marks[w].unit);
                o += 1;
            }
        }
        for i in (0..self.units.len())
            .filter(|&i| self.units[i].owner >= 10)
            .collect::<Vec<_>>()
        {
            self.process_unit(i, frame, &mut events);
        }

        self.mark("buildings");
        // Then the buildings — **after** every unit, not before.
        // `Objects::process_all@0065dce0` is two loops: the units, rotated by
        // owner, and then a *second, unrotated* one over each player's
        // buildings (object numbers from 2,000) and then their walls (from
        // 3,000). `docs/SYNC.md` §3.2 has said so since it was written; the
        // tick ran the buildings first until 2026-08-30, and East Indies'
        // frame 219 is where it first showed: the original spends a farmer's
        // `Unit::do_job+0x67` re-target and *then* the frame's road search,
        // and this crate spent them the other way round.
        //
        // What the order costs is a frame of latency in both directions, and
        // both are the original's: a unit a queue hands over this frame waits
        // for the next one to move, and the per-frame counters `Build::process`
        // clears — `helpers`, `gather_bumped` — are cleared *behind* the
        // gatherers and builders that set them rather than in front, which is
        // the same net state at the start of a frame.
        //
        // `Wall::process` first — the under-attack decay, the helpers reset,
        // the building's own attrition, ejection, the capture re-test, the
        // assimilation tick and the city heal (`docs/CITIES.md`) — then the
        // queue, then the tower. Splitting the three into three passes over
        // the list is still ours; the original does all three inside one
        // `Build::process`, per building.
        for b in 0..self.buildings.len() {
            self.buildings[b].gather_bumped = false;
            self.process_building(b, frame);
        }
        self.process_queues();
        // A building that shoots does so from `Build::process` too.
        for b in 0..self.buildings.len() {
            self.process_building_combat(b, frame);
        }

        self.mark("gaia");
        // The tail of `Objects::process_all`: the birds' sampling every 32
        // frames and one herd's walk every 64 (`gaia.rs`).
        self.process_gaia(frame);
        self.mark("guys_inc_time");
        // `Objects::inc_time`: every guy's animation clock first — leader
        // order, no rotation (`anim.rs`) — then the ammo list, then the
        // farms' crop cells (`farms.rs`), and the sites' hit points
        // refreshed from the progress the builders just made,
        // `Wall::inc_time`.
        self.guys_inc_time();
        self.mark("projectiles");
        self.process_projectiles(frame);
        self.mark("farms");
        self.farms_inc_time();
        self.mark("end");
        self.refresh_site_hits();
        self.frame += 1;
        events
    }

    /// Gives a unit a build order on a placed building — a player's
    /// `Group::action_swarm_around(BUILD_AT)` for one unit, replacing
    /// whatever it was doing: the approach is the order's own first step.
    pub fn order_build(&mut self, unit: usize, at: usize) {
        self.add_build_order(unit, at, orders::QueuePos::New, true);
    }

    /// Gives a unit a repair order — `Unit::add_repair_order`, as the player
    /// issues it.
    pub fn order_repair(&mut self, unit: usize, at: usize) {
        self.add_repair_order(unit, at, orders::QueuePos::New, true);
    }

    /// One unit's turn inside `Objects::process_all`'s first loop —
    /// `Unit::process@00610bc0`, or `Animal::process` for gaia's.
    fn process_unit(&mut self, i: usize, frame: i64, events: &mut Vec<Tick>) {
        // `who/o` — the unit's own [`Unit::index`], not its slot in this
        // vector. Every other `who/o` in the project is the game's `o`,
        // and a label that quietly meant the slot cost a probe run once.
        self.mark(&format!(
            "unit {}/{}",
            self.units[i].owner, self.units[i].index
        ));
        if !self.units[i].alive() {
            return;
        }
        self.units[i].movement.frame_facing = self.units[i].movement.facing;
        // Gaia's animals: `Animal::process` is `Unit::process` without
        // a player behind it — no heal, no attrition, no reload; the
        // order step (`Animal::do_idle` when idle) and the body follow.
        if self.units[i].is_gaia() {
            if self.units[i].on_map {
                self.work(i, frame);
                if self.units[i].alive() && self.units[i].on_map {
                    self.process_movement(i);
                }
            }
            return;
        }
        // `process_healing` runs for every unit, inside or out; the
        // garrison branch is the only heal this mechanic owns.
        self.garrison_heal(i, frame);
        if !self.units[i].on_map {
            return;
        }
        // `Unit::process` begins by counting the reload down, before
        // anything else the unit does this frame.
        if self.units[i].combat.recharging > 0 {
            self.units[i].combat.recharging -= 1;
        }
        // **`ObjectData::targeted` decays**, and its slot is the one the
        // attrition refresh is nested inside. `Unit::process@00610bc0`
        // reads, under `inside_up < 0`:
        //
        //     if ((frame + o) % 16 == 0) {
        //         targeted = targeted / 4;          // signed, toward zero
        //         process_cloak();
        //         if ((frame + o) % 32 == 0) { … process_attrition(); … }
        //     }
        //
        // Nothing else in the executable writes `+0x3d` but
        // `Object::init` (zero), `Object::find_nearby_target` (the bump on
        // its winner) and `Wall::process` (the same decay) — so a count
        // that is never decremented when an attacker drops its target is
        // *deliberate*: the field is a decaying crowding penalty, not a
        // reference count. `docs/COMBAT.md` §33.
        //
        // The counter is clamped non-negative here, so the original's
        // round-toward-zero idiom is a plain `/ 4`.
        if self.units[i].phase(frame) % TARGETED_DECAY_FRAMES == 0
            && self.units[i].inside.is_none()
            && self.units[i].inside_unit.is_none()
        {
            self.units[i].combat.targeted /= 4;
        }
        // Attrition first, movement second. That is the order inside
        // `Unit::process`, and it is observable: a unit that steps over a
        // border this frame is not standing there when this frame's
        // attrition looks, so it cannot bleed for the crossing until the
        // next one — and, because the period is only refreshed every 32
        // frames, usually not for a good while after that.
        if let Some(tick) = self.process_attrition(i, frame) {
            events.push(tick);
        }
        if !self.units[i].alive() {
            return;
        }
        // Then the order step — `Unit::work` → `do_job` on the front
        // order (`docs/ORDERS.md` §2.3): a move steps the unit, a build
        // runs the clock, an attack runs `fight`, an idle unit thinks.
        self.work(i, frame);
        if !self.units[i].alive() || !self.units[i].on_map {
            return;
        }
        self.process_movement(i);
    }

    /// One unit's attrition for one frame — the refresh, the supply veto, and
    /// the damage.
    fn process_attrition(&mut self, i: usize, frame: i64) -> Option<Tick> {
        let phase = self.units[i].phase(frame);

        // The refresh. `process_attrition` clears the period on entry, so an
        // exempt unit comes out of it with nothing pending.
        if phase % ATTRITION_REFRESH_FRAMES == 0 {
            let outcome = self.attrition_for(i);
            let unit = &mut self.units[i];
            unit.sheltered = false;
            match outcome {
                attrition::Outcome::Exempt(_) => {
                    unit.attrition = 0;
                    unit.ignores_supply = false;
                }
                attrition::Outcome::Period {
                    frames,
                    ignores_supply,
                } => {
                    unit.attrition = frames;
                    unit.ignores_supply = ignores_supply;
                }
            }
        }

        let unit = &self.units[i];
        if unit.attrition == 0 || phase % i64::from(unit.attrition) != 0 {
            return None;
        }
        // Supply gets first refusal — `Unit::process_supply`. A unit inside a
        // friendly supply radius takes no attrition at all, which is the whole
        // reason an army can campaign abroad. The two refusals before the
        // search are the interesting ones: a peace or assassin bleed gives up
        // immediately, and militia and supply units are never sheltered.
        let sheltered = !unit.ignores_supply
            && unit.kind.shelterable()
            && self.supplied_at(unit.owner, unit.pos);

        let unit = &mut self.units[i];
        if sheltered {
            unit.sheltered = true;
            return None;
        }
        // The damage is sixteenths of a hit point per figure, carried through
        // a fractional accumulator the way `Object::take_damage` carries it,
        // so a lone figure loses one whole point a tick and a figure in a
        // squad of four loses one every fourth tick.
        let sixteenths = attrition::damage(unit.squad_size);
        let (lost, frac) = attrition::take_damage(unit.damage_frac, sixteenths);
        unit.damage_frac = frac;
        unit.health -= lost;
        let killed = !unit.alive();
        if killed {
            self.close_supply(i);
        }
        Some(Tick {
            unit: i,
            frame,
            sixteenths,
            lost,
            killed,
        })
    }

    /// [`turning_of`] for a type this simulation holds.
    pub fn turning_for(&self, ty: usize) -> movement::Turning {
        turning_of(&self.unit_types[ty])
    }

    /// The body's frame, `Guy::move` through `Guy::process`, which
    /// `Unit::process` reaches after the order step: the body lands on where
    /// the unit *now* is. The unit step itself is the move order's
    /// (`orders::do_move`), in the same frame and before this.
    fn process_movement(&mut self, i: usize) {
        let unit = &self.units[i];
        let m = unit.movement;
        let facing = m.facing;
        let heading = m.heading;
        let pos = unit.pos;
        // `Guy::move`'s animation half runs on the body as it stood before
        // the follow: a body away from its destination starts the walk, one
        // standing on it a frame after arriving goes idle (`anim.rs`).
        let was_at_des = m.body.pos == pos;
        // `Guy::move:55`'s `des_angle == angle` — the heading against the
        // facing, read before this frame's turn. It decides both the
        // arrival's animation arm and whether the crew's destination is
        // rewritten below.
        let facing_settled = facing == heading;
        // The body's rate is mode 1: the base, always. It reads `last_speed`
        // — but **a body already standing on its unit reads a zero**, because
        // `Guy::move@005d9240:53` writes `last_speed = 0` at the head of its
        // at-des branch, ahead of everything else in it and so ahead of
        // `turn_towards`. That is not bookkeeping: `GuyData::turn_speed:29`
        // returns `0x80000000` — instant — for `last_speed == 0` on a foot or
        // mounted type (`guy_flags & 0x10`), so **a standing body swallows
        // whatever turn it is owed in one frame**, however large. The frame it
        // arrives on still reads the step it just took, so the turn is one
        // frame and not two: `docs/SYNC.md` §3.11's arrival pair, whose second
        // draw the third frame would have made a third.
        let turned = facing != m.frame_facing;
        let turning = movement::Turning {
            packed: self.units[i].combat.packed,
            ..m.turning
        };
        let rate = movement::turn_speed(
            &self.tuning,
            &turning,
            if was_at_des { 0 } else { m.body.last_speed },
            m.body.avg_speed,
            movement::TurnMode::Body,
        );
        let mut follow = movement::body_follow(m.body, facing, pos, heading, turned, rate);
        if !self.world.accepts(follow.body.pos) {
            follow.body.pos = m.body.pos;
        }
        self.guys_follow(i, was_at_des);
        // `Guy::move:109`'s `turn_towards(des_angle, …, 1)`, which hands
        // `Guy::do_turn` the same override `move_step`'s two turn-in-place
        // arms do — so a standing guy owed a turn asks for its turn
        // animation here as well, and pays the idle roll if it has none
        // (`docs/ANIM.md` §4.8). The gate is the original's own: `guy_flags
        // & 2` clear, which is `turned`, and the body on its unit.
        if was_at_des && !turned && !facing_settled {
            self.mark(anim::SITE_TURN_STAND);
            self.do_turn_anim(i, facing, follow.facing, heading);
        }
        let unit = &mut self.units[i];
        unit.movement.facing = follow.facing;
        unit.movement.body = follow.body;
        // **Where a crew guy is told to be**, and it is written twice
        // over — the two arms of `Guy::move` reach it by different
        // functions and both end in the same rotation:
        //
        // - **moving**: `Guy::set_new_location(des, 0)`, whose crew loop
        //   (`005d88da–005d89cc`) rewrites `des_angle` and `des` from guy
        //   0's own **new** position and facing. The snap flag is zero, so
        //   the crew is told where to be and not put there.
        // - **standing but still owed a turn**: `Guy::turn_towards →
        //   do_turn → Guy::set_angle(new facing, 0)`, whose crew loop
        //   (`005d90ad–005d9192`) is the same rotation about guy 0's
        //   position, with the angle it has just turned to. `do_turn` is
        //   called whether or not the facing actually moved.
        //
        // And **not written at all** on the third: a standing guy 0 whose
        // facing has reached the heading takes `Guy::move:55`'s settled
        // arm straight to the average, past the turn — so the crew keeps
        // the point it was last given and walks on toward it. That is the
        // whole of why a scout's dog is still walking four frames after
        // the man has stopped.
        //
        // Guy 0's position after either arm is the unit's own, so the two
        // are one expression here.
        if !was_at_des || !facing_settled {
            let (pos, facing) = (self.units[i].pos, follow.facing);
            self.crew_des(i, pos, facing, false);
        }
        for g in 0..self.units[i].guys.len() {
            if self.units[i].guys[g].follow.is_some() {
                self.process_follower(i, g);
            }
        }
        // `Guy::process`'s own tail: the sixty-fourth-frame repaint of the
        // collision block (`docs/COLLISION.md` §2.2). It reads the body as
        // `Guy::move` has just left it, which for guy 0 is the unit's own
        // position.
        self.coll_repaint(i);
    }

    /// **The crew loop**, which is the same eight lines in three places
    /// and belongs to guy 0 in all of them: `Guy::set_angle@005d9010`
    /// (`005d90ad`–`005d9192`), `Guy::set_new_location@005d86f0`
    /// (`005d88da`–`005d89cc`) and `Unit::init`'s seating. Every guy past
    /// `squad_size` is told an angle and a point — `des_angle = angle`,
    /// `des = from + rotate(track, angle)`, each axis clamped into the
    /// world — and nothing else about it is touched.
    ///
    /// `from` is guy 0's own `x` / `y`, which is the unit's position on
    /// every frame the unit's step was taken.
    ///
    /// **`snap` is the callers' `param_3`**, and where it is set the crew
    /// is not told the point but *put* on it: the loop then runs
    /// `set_angle(crew, des_angle, 1)` and `set_new_location(crew, des,
    /// 1)`, which write the figure's facing and its body outright. Only
    /// the two teleporting callers carry it —
    /// `Unit::set_new_location(…, 1, …)`, which is `Unit::init`'s seating
    /// and `resolve_unit_collision`'s cell-centre snap
    /// (`docs/COLLISION.md` §6 step 6). `Unit::set_angle`'s call and
    /// `Guy::move`'s both pass zero.
    fn crew_des(&mut self, u: usize, from: Pos, angle: movement::Angle, snap: bool) {
        let bound = Pos::new(
            self.world.width() * world::UNITS_PER_CELL,
            self.world.height() * world::UNITS_PER_CELL,
        );
        for g in 0..self.units[u].guys.len() {
            let Some(f) = &mut self.units[u].guys[g].follow else {
                continue;
            };
            let track = f.track;
            f.des_angle = angle;
            f.des = movement::follower_des(from, angle, track, bound);
            if snap {
                f.facing = angle;
                f.body.pos = f.des;
            }
        }
    }

    /// One crew guy's own frame — `Guy::process → Guy::move` for a guy
    /// with a track offset, which runs after guy 0's and reads the `des`
    /// guy 0 has just written.
    ///
    /// The two arms are `Guy::move`'s own. Standing on its destination it
    /// writes `last_speed = 0` and turns toward `des_angle` — its
    /// `guy_flags & 2` is never set by anything else, because
    /// `Guy::do_turn@005d97a0:37` recurses only into the crew that has
    /// *no* track, so the turn is never skipped. Walking, it is
    /// [`movement::follower_step`]. `docs/MOVEMENT.md`, "The follower's
    /// destination".
    fn process_follower(&mut self, i: usize, g: usize) {
        let Some(f) = self.units[i].guys[g].follow else {
            return;
        };
        let speed = self.units[i].movement.speed;
        let at_des = f.body.pos == f.des;
        self.guy_follow_anim(i, g, at_des, f.facing == f.des_angle);
        // A tracked crew guy's rate is [`movement::CREW_TURN_SPEED`] and
        // nothing else: `GuyData::turn_speed` returns it before the
        // instant-from-a-stop test, so neither the type's `TURN_SPEED`
        // nor a zero `last_speed` is read for this guy at all.
        let rate = movement::CREW_TURN_SPEED;
        let mut next = f;
        if at_des {
            next.body.last_speed = 0;
            if next.facing != next.des_angle {
                // `Guy::move:109` again, and for this guy in its own right
                // (`docs/ANIM.md` §4.8): the standing arm's
                // `turn_towards(des_angle, ..., 1)` hands `Guy::do_turn`
                // the same override guy 0's does. The gate `guy_flags & 2`
                // is always open here — the only writer is `do_turn`, and
                // `do_turn@005d97a0:37` recurses into the crew that has
                // **no** track, never this one — so a tracked crew figure
                // of a packing type asks for a turn animation it has not
                // got and pays the idle roll.
                let was = next.facing;
                next.facing = movement::turn_towards(next.facing, next.des_angle, rate).0;
                self.mark(anim::SITE_TURN_STAND);
                self.guy_do_turn_anim(i, g, was, next.facing, next.des_angle);
            }
            next.body.avg_speed = (next.body.avg_speed * 3 + next.body.last_speed) / 4;
        } else {
            let (facing, body) = movement::follower_step(f.body, f.facing, f.des, rate, speed);
            next.facing = facing;
            if let Some(mut body) = body {
                if !self.world.accepts(body.pos) {
                    body.pos = f.body.pos;
                }
                next.body = body;
                self.units[i].guys[g].stopped = false;
            }
        }
        self.units[i].guys[g].follow = Some(next);
    }

    /// Sends a unit somewhere. It faces whatever way it already faces and turns
    /// as it goes — instantly, if it is a foot or mounted type standing still.
    pub fn order_move(&mut self, unit: usize, dest: Pos) {
        self.add_move_order(
            unit,
            dest,
            orders::MoveKind::MoveTo,
            orders::QueuePos::New,
            true,
        );
    }

    fn attrition_for(&self, i: usize) -> attrition::Outcome {
        let unit = &self.units[i];
        let ground = self.world.owner_at(unit.pos);
        let victim = &self.players[unit.owner as usize];
        let at_war = ground
            .player()
            .is_none_or(|o| self.at_war[unit.owner as usize][o as usize]);
        let situation = attrition::Situation {
            ground,
            in_free_zone: false,
            mutual_treaty: false,
            at_war,
            in_war_grace: false,
            assassin: false,
            conquest_exempt: false,
        };
        attrition::process(
            &self.tuning,
            &unit.kind,
            unit.owner,
            victim,
            &situation,
            |p| self.players[p as usize],
        )
    }
}

/// One figure taking one tick of attrition damage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tick {
    pub unit: usize,
    pub frame: i64,
    /// Sixteenths of a hit point dealt — [`attrition::damage`] for the
    /// figure's squad size.
    pub sixteenths: i32,
    /// Whole hit points actually deducted this tick, after the fractional
    /// accumulator. Zero on most ticks for a figure in a squad of four.
    pub lost: i32,
    pub killed: bool,
}

#[cfg(test)]
#[path = "harness_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "cities_tests.rs"]
mod cities_tests;

// The hard constraint, checked against this crate's own source rather than
// trusted: `docs/DECISIONS.md` 16, `CLAUDE.md`.
#[cfg(test)]
mod docs_guard;
#[cfg(test)]
mod memcap_guard;
#[cfg(test)]
mod no_float;

// Generated games, each played twice: determinism on paths no hand-written
// scenario walks, and a panic hunt on the way.
#[cfg(test)]
mod soak;

/// The install-backed tests' one question: where is the game.
#[cfg(test)]
pub(crate) mod testenv {
    /// The install's root: `$RON_INSTALL`, else the checkout's `game/`, else
    /// — from a worktree under `.claude/worktrees/<name>/` — the main
    /// checkout's `game/`, three directories up. The last is what stops
    /// every worktree session typing the variable by hand (2,147 times in
    /// 120 sessions, by the transcripts, before 2026-09-01).
    pub(crate) fn install_root() -> Option<String> {
        if let Ok(r) = std::env::var("RON_INSTALL") {
            return Some(r);
        }
        let here = env!("CARGO_MANIFEST_DIR");
        [
            format!("{here}/../../game"),
            format!("{here}/../../../../../game"),
        ]
        .into_iter()
        .find(|g| std::path::Path::new(g).join("riseofnations.exe").is_file())
    }
}
