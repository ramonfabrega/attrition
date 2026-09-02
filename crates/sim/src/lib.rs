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
pub mod anim;
pub mod army;
pub mod attrition;
pub mod balance;
pub mod bhs;
pub mod build;
pub mod city;
pub mod collide;
pub mod combat;
pub mod cost;
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
pub mod market;
pub mod movement;
pub mod nations;
pub mod orders;
pub mod path;
pub mod place;
pub mod production;
pub mod roads;
pub mod scout;
pub mod supply;
pub mod tech;
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
    /// The per-player idle-citizen option, as the threshold `think_peasant`
    /// compares `idle` against; 2 by default.
    pub idle_threshold: u8,
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
            damage_frac: 0,
            squad_size: 1,
            kind: attrition::UnitKind::default(),
            on_map: true,
            supply_slot: None,
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
            idle_threshold: 2,
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
    /// Whether each player has been defeated — `leader_flags & 2` clear.
    pub defeated: Vec<bool>,
    /// `LeaderData::lost_city_stamp`: the frame each player last lost a city.
    pub lost_city_stamp: Vec<Option<i64>>,
    /// `LeaderData::cities_built`, `cities_captured`, `cities_lost`.
    pub city_tally: Vec<city::Tally>,
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
        Sim {
            transport: vec![transport::LeaderTransport::default(); players],
            docks: vec![transport::Docks::default(); players],
            armies: (0..players)
                .map(|w| army::Armies::new(w as Player))
                .collect(),
            tech: (0..players)
                .map(|_| tech::PlayerTech::new(&tech_tree))
                .collect(),
            tech_tree,
            setup: tech::Setup::STANDARD,
            trace_phases: false,
            plan_roads: true,
            phase_marks: Vec::new(),
            trace_costs: false,
            cost_marks: Vec::new(),
            players: vec![attrition::PlayerState::default(); players],
            sources: Vec::new(),
            units: Vec::new(),
            supply: vec![supply::Network::default(); players],
            at_war: vec![vec![false; players]; players],
            holdings: vec![economy::Holdings::new(); players],
            ledgers: vec![economy::Ledger::starting(&tuning); players],
            unit_types: Vec::new(),
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
            defeated: vec![false; players],
            lost_city_stamp: vec![None; players],
            city_tally: vec![city::Tally::default(); players],
            removed: Vec::new(),
            wall_stats_dirty: vec![false; players],
            // Ten slots, not `players`: gaia's animals and birds are units
            // of owners 8 and 9 and take numbers from their own bands.
            marks: vec![Marks::default(); players.max(10)],
            lobby: ai::Lobby::default(),
            script_env: ai_host::ScriptEnv::default(),
            scripts: None,
            ai: vec![ai::Leader::new(); players],
            ai_speed: 1,
            coll: collide::CollGrid::new(world.width(), world.height()),
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
        self.ledgers.push(economy::Ledger::starting(&self.tuning));
        self.muster.push(Muster::new(&self.tuning));
        self.mods.push(combat::Modifiers::default());
        self.nation.push(city::Nation::default());
        self.borders
            .push(territory::PlayerBorders::plain(&self.tuning));
        self.defeated.push(false);
        self.lost_city_stamp.push(None);
        self.city_tally.push(city::Tally::default());
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
        u8::try_from(who).expect("too many players")
    }

    /// Sets two players as mutual allies — both `diplos` entries at 2.
    pub fn make_allies(&mut self, a: Player, b: Player) {
        self.allied[a as usize][b as usize] = true;
        self.allied[b as usize][a as usize] = true;
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
        // `Object::add_to_world`: both collision indices
        // (`docs/COLLISION.md` §2, §3).
        self.coll_add(i);
        if self.units[i].alive() && self.units[i].on_map {
            self.chain_add(i);
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
            of_group: unit.group.map_or(0, |g| muster.by_group[g]),
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
                let pos = self.buildings[at].pos;
                // No refund on completion, and no skip-forward: the original
                // passes false for both, and they are the same argument.
                let mut ledger = economy::Ledger::default();
                self.buildings[at].queue.unqueue(slot, false, &mut ledger);

                let muster = &mut self.muster[who as usize];
                muster.queued_by_type[ty] -= 1;
                muster.by_type[ty] += 1;
                muster.control += pop;
                if let Some(g) = self.unit_types[ty].group {
                    muster.by_group[g] += 1;
                }
                self.track_tree_queued(who, ty, -1);

                // `Objects::init_unit` → `find_free(who, 0, 2000, …)`; a full
                // band (2,000 live units of one player) is not modelled as a
                // refusal here, so the number saturates instead.
                let index = self
                    .find_free(who, UNIT_BASE, BUILD_BASE)
                    .unwrap_or(i16::MAX);
                let mut unit = Unit::new(who, index, pos, self.unit_types[ty].hits);
                unit.kind = self.unit_types[ty].kind;
                unit.ty = Some(ty);
                unit.type_index = self.unit_types[ty].type_index;
                unit.movement.speed = self.unit_types[ty].moves;
                unit.movement.turning = self.turning_for(ty);
                let unit = self.add_unit(unit);
                // `Unit::init` → `Guy::init_real`: the figure's one draw.
                // (The unit's `ty` stays unset here, as it always has; the
                // piece lookup takes the type directly.)
                self.init_guys(unit, Some(ty));
                // **The trained unit is born inside its trainer and walks
                // out.** `Build::train@0062f9b0` creates it at the
                // building's own position, calls `Unit::go_inside`, and
                // then `Unit::come_out` — which is what puts it on the exit
                // ring rather than on the building's centre tile. The
                // arms `train` takes before that last call — a
                // gather-inside building that keeps its worker, a dock's
                // boat count, the player's own text bubble — are not
                // modelled; every trainer here lets its unit straight out.
                self.go_inside(unit, at);
                self.come_out(unit);
                self.economy_changed(who);
                Advanced::Trained(Produced { unit, ty, at })
            }
        }
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
        events
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
        let events = self
            .tech_tree
            .gain_tech(&self.setup, &mut self.tech[who as usize], t, frame);
        self.apply_gained(who);
        // `Leader::gain_tech`'s tail: `check_transport` (`docs/TRANSPORT.md`
        // §4).
        self.check_transport(who);
        events
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

    /// The tree's own queued count for the government-pairing rule.
    fn track_tree_queued(&mut self, who: Player, ty: usize, delta: i32) {
        if let Some(id) = self.unit_types[ty].tree {
            self.tech[who as usize].queued[id] += delta;
        }
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

        let muster = &mut self.muster[who as usize];
        muster.by_type[ty] += 1;
        if let Some(g) = self.unit_types[ty].group {
            muster.by_group[g] += 1;
        }
        muster.control += pop;

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
    /// **Still seams**, because nothing here models them: the temple and
    /// fort border levels (`has_preq(TEMPLEBORDERS2..4)`,
    /// `has_preq(FORTBORDERS2..4)` — bonus types `0x2c8..0x2ca` and
    /// `0x2d1..0x2d3`, which this crate loads as `bonus_preqs` and does not
    /// expose), the Colosseum and Eiffel Tower, a gem rare, and the AI
    /// handicap allowance. All four are inert on every capture so far: no
    /// player in one holds a Temple, a Fort, either wonder or a gem, and
    /// the lobbies run at handicap 0.
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
                gems: false,
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
        if self.trace_phases {
            self.phase_marks.clear();
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
            // `Leader::process` also answers the wall-stats dirty flag here,
            // before any building is touched.
            if self.wall_stats_dirty[who] {
                self.calc_wall_stats(player);
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
        self.mark(&format!("unit {}/{}", self.units[i].owner, i));
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
            let bound = Pos::new(
                self.world.width() * world::UNITS_PER_CELL,
                self.world.height() * world::UNITS_PER_CELL,
            );
            let (pos, facing) = (self.units[i].pos, follow.facing);
            for g in 0..self.units[i].guys.len() {
                let Some(f) = &mut self.units[i].guys[g].follow else {
                    continue;
                };
                let track = f.track;
                f.des_angle = facing;
                f.des = movement::follower_des(pos, facing, track, bound);
            }
        }
        for g in 0..self.units[i].guys.len() {
            if self.units[i].guys[g].follow.is_some() {
                self.process_follower(i, g);
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
                next.facing = movement::turn_towards(next.facing, next.des_angle, rate).0;
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
