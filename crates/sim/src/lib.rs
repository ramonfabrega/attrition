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
pub mod attrition;
pub mod balance;
pub mod build;
pub mod city;
pub mod combat;
pub mod cost;
pub mod economy;
pub mod fight;
pub mod garrison;
pub mod movement;
pub mod orders;
pub mod path;
pub mod place;
pub mod production;
pub mod supply;
pub mod tech;
pub mod territory;
pub mod tuning;
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
    /// `unit_masks & 0x400`: has been given a build or repair order.
    pub was_builder: bool,
    /// `SubObjectData::flags & 0x10`: could not reach its target.
    pub cant_reach: bool,
    /// `unit_masks & 1`: a decoy; not counted as a gatherer.
    pub decoy: bool,
    /// `UnitData::avoid_x/avoid_y`: the point `find_path` recorded as
    /// unreachable, which `valid_wcoord` refuses. Cleared before each step
    /// (`docs/ORDERS.md` §4.5).
    pub avoid: Option<Pos>,
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
    /// Which way the unit faces — guy 0's angle, which the unit step turns and
    /// the body shares. North is zero.
    pub facing: movement::Angle,
    /// The facing as it stood at the start of this frame — what the body
    /// follow compares against after the order step has turned the unit.
    pub frame_facing: movement::Angle,
    /// The heading the last unit step recorded as desired, `GuyData::des_angle`.
    /// An idle body turns toward it.
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
    /// A unit standing at `pos`, facing north, with its body on it and stopped
    /// — which, for a foot or mounted type, is what lets the first order turn
    /// it instantly.
    pub const fn at(pos: Pos) -> Movement {
        Movement {
            facing: movement::Angle::NORTH,
            frame_facing: movement::Angle::NORTH,
            des_angle: movement::Angle::NORTH,
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

    /// Faces the unit and its body a given way at once — `Guy::set_angle` with
    /// the snap flag, which writes both the facing and the desired angle.
    pub const fn set_facing(&mut self, facing: movement::Angle) {
        self.facing = facing;
        self.des_angle = facing;
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
    /// Which worker kind this is — a citizen or scholar may gather and is
    /// what `think_peasant` runs for (`docs/ORDERS.md` §6.1).
    pub worker: orders::Worker,
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
            was_builder: false,
            cant_reach: false,
            decoy: false,
            avoid: None,
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
    /// The game's random stream, `game_random`. Combat draws from it for
    /// projectile scatter and the one-in-five retarget roll.
    pub rng: combat::Rng,
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
    pub frame: i64,
}

/// A building: a production queue, a combat profile, and — when it has a
/// type — a footprint, a construction clock, a city and a garrison.
/// `docs/CITIES.md` §1.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Building {
    pub owner: Player,
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
}

/// `Farms`' per-farm record, as far as the farmer's stand reads it: a
/// state byte and a growth count per tile of the footprint.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Farm {
    pub state: [u8; 16],
    pub percent: [i32; 16],
}

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
        Sim {
            tech: (0..players)
                .map(|_| tech::PlayerTech::new(&tech_tree))
                .collect(),
            tech_tree,
            setup: tech::Setup::STANDARD,
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
            table: combat::Table::uniform(0),
            rng: combat::Rng::new(0),
            projectiles: Vec::new(),
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
            tuning,
            world,
            frame: 0,
        }
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

    /// `LeaderData::is_ally`: the same player, or allied both ways.
    pub fn is_ally(&self, a: Player, b: Player) -> bool {
        a == b || (self.allied[a as usize][b as usize] && self.allied[b as usize][a as usize])
    }

    /// `LeaderData::is_enemy`: different players with war declared either way.
    pub fn is_enemy(&self, a: Player, b: Player) -> bool {
        a != b && (self.at_war[a as usize][b as usize] || self.at_war[b as usize][a as usize])
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
        self.units.push(unit);
        if source {
            self.units[i].supply_slot = Some(self.supply[owner].list.register(i));
        }
        i
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
        self.buildings.push(Building {
            owner,
            pos,
            queue: production::Queue::new(capacity),
            is_library: false,
            combat: None,
            hits: 0,
            health: 0,
            damage_frac: 0,
            active: true,
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
        self.muster[who as usize].queued_by_type[item.ty] -= 1;
        self.track_tree_queued(who, item.ty, -1);
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
        let muster = &self.muster[b.owner as usize];
        production::train_time(
            &self.tuning,
            &self.unit_types[item.ty].times,
            muster.researched[item.ty],
            muster.by_type[item.ty],
            &[],
        )
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

                let index = i16::try_from(self.units.len()).unwrap_or(i16::MAX);
                let mut unit = Unit::new(who, index, pos, self.unit_types[ty].hits);
                unit.kind = self.unit_types[ty].kind;
                let unit = self.add_unit(unit);
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
        let events = self
            .tech_tree
            .gain_tech(&self.setup, &mut self.tech[who as usize], t, frame);
        self.apply_gained(who);
        events
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

        let index = i16::try_from(self.units.len()).unwrap_or(i16::MAX);
        let mut unit = Unit::new(who, index, pos, self.unit_types[ty].hits);
        unit.kind = self.unit_types[ty].kind;
        unit.ty = Some(ty);
        let at = self.add_unit(unit);
        self.economy_changed(who);
        Ok(at)
    }

    /// Gives a dead unit's supply slot back — `Unit::close`.
    fn close_supply(&mut self, unit: usize) {
        let owner = self.units[unit].owner as usize;
        if let Some(slot) = self.units[unit].supply_slot.take() {
            self.supply[owner].list.close(slot);
        }
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
        self.at_war[a as usize][b as usize] = true;
        self.at_war[b as usize][a as usize] = true;
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

        // Income first. `Game::do_frame` runs `Leaders::process_all` before
        // `Objects::process_all`, so every player is paid for the frame before
        // any unit in it moves, fights or bleeds. That ordering is observable:
        // a citizen that dies this frame was already paid for it.
        for who in 0..self.players.len() {
            let player = u8::try_from(who).expect("too many players");
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

        // Then the buildings. `Build::process` and `Unit::process` are both
        // reached from `Objects::process_all`, so in the original they
        // interleave by object index rather than running in two passes. Doing
        // buildings first is the choice that keeps a unit handed over this
        // frame visible to this frame's unit loop, which is what the original
        // does whenever the building's index is the lower of the two — and it
        // is, for a building that existed before the unit it just made.
        //
        // `Wall::process` first — the under-attack decay, the helpers reset,
        // the building's own attrition, ejection, the capture re-test, the
        // assimilation tick and the city heal (`docs/CITIES.md`) — then the
        // queue, then the tower.
        for b in 0..self.buildings.len() {
            self.buildings[b].gather_bumped = false;
            self.process_building(b, frame);
        }
        self.process_queues();
        // A building that shoots does so from `Build::process`, which in the
        // original interleaves with the units by index; here the buildings
        // go first, as they do for the queues.
        for b in 0..self.buildings.len() {
            self.process_building_combat(b, frame);
        }

        for i in 0..self.units.len() {
            if !self.units[i].alive() {
                continue;
            }
            self.units[i].movement.frame_facing = self.units[i].movement.facing;
            // `process_healing` runs for every unit, inside or out; the
            // garrison branch is the only heal this mechanic owns.
            self.garrison_heal(i, frame);
            if !self.units[i].on_map {
                continue;
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
                continue;
            }
            // Then the order step — `Unit::work` → `do_job` on the front
            // order (`docs/ORDERS.md` §2.3): a move steps the unit, a build
            // runs the clock, an attack runs `fight`, an idle unit thinks.
            self.work(i, frame);
            if !self.units[i].alive() || !self.units[i].on_map {
                continue;
            }
            self.process_movement(i);
        }
        // Ammo after every object — `Objects::inc_time` runs the ammo list
        // after `process_all` — and the sites' hit points refreshed from the
        // progress the builders just made, `Wall::inc_time`.
        self.process_projectiles(frame);
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

    /// The body's chase, `Guy::move` through `Guy::process`, which
    /// `Unit::process` reaches after the order step: the body follows where
    /// the unit *now* is. The unit step itself is the move order's
    /// (`orders::do_move`), in the same frame and before this.
    fn process_movement(&mut self, i: usize) {
        let unit = &self.units[i];
        let m = unit.movement;
        let facing = m.facing;
        let des_angle = m.des_angle;
        let pos = unit.pos;
        // The body's rate is mode 1: the base, always. It reads `last_speed`
        // as it stood before this frame, the way `Guy::move` does.
        let turned = facing != m.frame_facing;
        let rate = movement::turn_speed(
            &self.tuning,
            &m.turning,
            m.body.last_speed,
            m.body.avg_speed,
            movement::TurnMode::Body,
        );
        let mut follow =
            movement::body_follow(m.body, facing, pos, des_angle, turned, m.speed, rate);
        if !self.world.accepts(follow.body.pos) {
            follow.body.pos = m.body.pos;
        }
        let unit = &mut self.units[i];
        unit.movement.facing = follow.facing;
        unit.movement.body = follow.body;
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
mod no_float;

// Generated games, each played twice: determinism on paths no hand-written
// scenario walks, and a panic hunt on the way.
#[cfg(test)]
mod soak;
