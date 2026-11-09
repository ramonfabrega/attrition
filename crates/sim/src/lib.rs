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

pub mod groups;
pub mod ids;
pub use self::groups::{action_gather_view, action_spell_view};
pub mod ai;
pub use self::ai::ai_build;
pub use self::ai::ai_census;
pub use self::ai::ai_drive;
pub use self::ai::ai_host;
pub use self::ai::ai_load;
pub use self::ai::ai_make;
pub use self::ai::ai_place;
pub use self::ai::ai_research;
pub use self::ai::ai_sites;
pub use self::ai::ai_types;
pub use self::ai::ai_units;
pub mod air;
pub mod cities;
pub use self::air::airbase;
pub use self::groups::alarm;
pub use self::groups::alarm_view;
pub mod units;
pub use self::ai::army;
pub use self::ai::army::army_do_defending_view;
pub use self::ai::army::army_is_moving_view;
pub use self::ai::army::army_normalize_view;
pub use self::ai::army::army_process_view;
pub use self::ai::bhs;
pub use self::cities::{activate, build};
pub use self::combat::attack_pos;
pub use self::combat::balance;
pub use self::groups::attack_ground_view;
pub use self::groups::attack_view;
pub use self::groups::board_ship_view;
pub use self::movement::astar_view;
pub use self::territory::attrition;
pub use self::territory::border_pass;
pub use self::units::{add_to_world_view, anim};
pub mod spells;
pub use self::combat::build_attack_view;
// `combat::build_family` is private and named through `combat` alone (item 4012).
pub use self::air::carrier_queue;
pub use self::cities::build_unqueue_view;
pub use self::cities::capture;
pub use self::cities::capture_view;
pub use self::cities::city;
pub use self::combat::build_plunder_view;
pub use self::combat::cavarch;
pub use self::economy::calc_gather;
pub use self::economy::calc_gather_type_view;
pub use self::economy::caravan;
pub use self::economy::cliff_mine;
pub use self::mapgen::cliffs;
pub use self::movement::collide;
pub use self::orders::build_spot_view;
pub use self::orders::check_build_view;
pub use self::spells::cast_decoy;
pub use self::spells::cast_transport;
pub use self::spells::castable;
pub use self::spells::{bribe, cast};
pub use self::units::think::carry_civ;
pub use self::units::think::carry_idle;
pub use self::units::think::carry_mil;
pub use self::units::think::civ_transport;
pub mod combat;
pub(crate) use self::cities::construct;
pub use self::units::come_out_view;
pub mod core;
pub use self::combat::damage;
pub use self::economy::cost;
pub use self::movement::detect_collision_view;
pub use self::movement::detour;
pub use self::units::count_inside_view;
pub use self::world::danger;
pub mod leaders;
pub use self::groups::distribute_attack_view;
pub use self::orders::do_attack;
pub use self::spells::do_cast_view;
pub use self::units::disband;
pub mod economy;
pub use self::cities::farms;
pub use self::cities::find_builds_view;
pub use self::cities::friends;
pub use self::combat::fight;
pub use self::gaia::flock;
pub use self::groups::finish_insert_view;
pub use self::groups::form;
pub use self::groups::form_view;
pub use self::movement::find_path_view;
pub use self::units::eject;
pub use self::units::think::fish;
pub mod gaia;
pub use self::economy::gather;
pub use self::economy::gather_tile;
pub use self::gaia::goody;
pub use self::groups::garrison_view;
pub use self::groups::gather_view;
pub use self::groups::group;
pub use self::groups::group_queue_up;
pub use self::groups::grouppath;
pub use self::groups::groups_process_view;
pub use self::groups::halt_view;
pub use self::mapgen::halfland;
pub use self::orders::group_move;
pub use self::units::garrison;
pub use self::units::heal;
pub mod data;
pub use self::air::land_plane_view;
pub use self::cities::location;
pub use self::combat::hero_radius_view;
pub use self::economy::holdings;
pub use self::gaia::herd_create_view;
pub use self::mapgen::map_colonial;
pub use self::mapgen::map_east_indies;
pub use self::mapgen::map_east_west;
pub use self::mapgen::map_heights;
pub use self::mapgen::map_lakes;
pub use self::mapgen::map_mediterranean;
pub use self::mapgen::map_new_world;
pub use self::mapgen::map_nile_delta;
pub use self::mapgen::map_outback;
pub use self::mapgen::map_resources;
pub use self::mapgen::map_rivers;
pub use self::mapgen::map_sea_power;
pub use self::mapgen::map_troughs;
pub use self::mapgen::map_values;
pub use self::mapgen::map_watering_hole;
pub mod mapgen;
pub use self::economy::market;
pub use self::movement::mesh;
pub use self::units::think::merchant;
pub mod movement;
pub use self::ai::army::muster;
pub use self::cities::nations;
pub use self::combat::nuke;
pub use self::groups::normalize_view;
pub mod num;
pub use self::combat::object_attack_view;
pub mod orders;
pub use self::ai::army::target_gate;
pub use self::ai::army::target_score;
pub use self::ai::relative_value;
pub use self::ai::scenario_add;
pub use self::cities::place;
pub use self::cities::production;
pub use self::cities::queue_disband;
pub use self::cities::rally;
pub use self::cities::site_recruit;
pub use self::cities::tcoord;
pub use self::combat::plunder;
pub use self::combat::take_damage_view;
pub use self::data::{launch, pivot};
pub use self::economy::rares;
pub use self::groups::redo_patrol_view;
pub use self::groups::repair_view;
pub use self::groups::swarm;
pub use self::groups::swarm_around_view;
pub use self::leaders::{diplomacy, resign};
pub use self::movement::path;
pub use self::movement::place_roads_view;
pub use self::movement::roads;
pub use self::movement::roads_view;
pub use self::num::{double, single};
pub use self::spells::spell_cast_view;
pub use self::spells::spell_find_view;
pub use self::spells::spell_type_get_job_time_view;
pub use self::spells::spell_valid_target_view;
pub use self::territory::suffer_view;
pub use self::territory::supply;
pub use self::units::remove_inside_view;
use self::units::set_type_adapter;
pub use self::units::stance;
pub use self::units::think::scout;
pub use self::units::think::spellcaster;
pub mod tech;
pub use self::mapgen::terrain_groups;
pub use self::mapgen::terrain_init;
pub use self::world::terrain;
pub mod territory;
pub mod trace;
pub use self::ai::transport;
pub use self::cities::unit_unqueue_view;
pub use self::combat::unit_attack_view;
pub use self::core::type_index;
pub use self::core::{flag_reads, tuning};
pub use self::groups::trade_view;
pub use self::groups::update_positions_view;
pub use self::movement::tpath;
pub use self::movement::upath;
pub use self::orders::topp;
pub use self::territory::victory;
pub use self::units::unit_set_type_view;
pub use self::units::unit_speed_view;
pub use self::units::unit_update_hits_view;
pub mod vision;
pub use self::units::think::work_gate;
pub use self::vision::wall_los_view;
pub mod world;
pub use self::movement::wpath;

pub use self::core::frame::{
    ATTRITION_REFRESH_FRAMES, FRAMES_PER_SECOND, TARGETED_DECAY_FRAMES, Tick, UNIT_UPKEEP_FRAMES,
};
pub use cities::building::{Building, Produced};
#[cfg(test)]
use cities::queue::Advanced;
pub use tuning::Tuning;
pub use world::{Cell, Owner, Player, Pos, Terrain, World};

/// [`Unit::head_serial`]: the head order's identity, the original's
/// `UnitOrder *`. **Identity, not state**: it compares equal to every other
/// serial, so two units that differ only in which object heads their list
/// are the same unit to every comparison of state, and it prints as nothing
/// in particular (a checkpoint's reseat replaces one order with an equal
/// one, and the run69 interventions compare state by value and by `Debug`).
#[derive(Clone, Copy, Default)]
pub struct HeadSerial(pub u32);

impl std::fmt::Debug for HeadSerial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("HeadSerial")
    }
}

impl PartialEq for HeadSerial {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for HeadSerial {}

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
    /// **A unit's own training queue** (item 3981, `docs/AI.md` §413): the
    /// Aircraft Carrier's planes, `UnitData +0xa0 num_queued` (a short)
    /// and `+0x64 queue_time`, the clock (`crate::carrier_queue`).
    pub num_queued: i16,
    pub queue_time: i32,
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
    /// **`UnitData::mana_burn` (`+0x96`)** — the craft a caster has spent
    /// and not yet recovered: `mana_left = mana − mana_burn`. `Unit::init`
    /// starts a **spy** at half its `mana` (`is(SPY, 0)`, the only type it
    /// names), `pay_cast_costs` adds a craft's `MANA`, and `Unit::process`
    /// takes one back a frame while `unit_masks & 0x2a000` is clear
    /// (`docs/GOLDEN.md` §27). SEAM: an aircraft's burn — `do_strafe`'s
    /// `bombing_mana_cost` and `process`'s air recharge — is not carried.
    pub mana_burn: i16,
    /// **`UnitData::full`** (`+0xac`, a signed byte): "nobody can get at
    /// this one". `Unit::find_melee_pos@006010b0` adds 5 to the *target's*
    /// when it finds no open slot round it, capped at `0x1e`; `Unit::
    /// process@00610bc0` takes one off a frame at its head; and
    /// `Object::find_nearby_target`'s score of a unit divides by `full + 1`
    /// (`0064f110`). Item 3385: run1011's woodcutter `0/3` reads 5, 9, 8 …
    /// from block 13278 as two Inti Macemen fail to reach it, and the
    /// Heavy Archer `1/99` takes the citizen `0/4` over it on 13282.
    pub full: i8,
    /// **`UnitData::waiting`** (`+0xad`, a signed byte): `Unit::fight@
    /// 005fd4d0`'s search budget. Its invalid-target arm (`LAB_005fdb9e`)
    /// adds 2 and returns without searching while `waiting < 5` and the
    /// leader's [`Sim::retargets`] is past ten; `Unit::process@00610bc0`
    /// takes one off a frame at its head; `Unit::init@00612100` zeroes it.
    /// Item 3598: run1241b's Phalanx `1/116` reads 2 on block 6718 and 1
    /// on 6719, the frame twelve of who=1's units lost the Citizen `0/12`.
    pub waiting: i8,
    /// **`ObjectData::healing`** (`+0x38`, a short): the heal's display
    /// clock. Every heal `Unit::process_healing@005e0670` lands writes
    /// `max(rate, healing)` with that arm's period, `Unit::process@
    /// 00610bc0`'s head takes one off a frame, and its air arm writes 0 on
    /// the map and 1 on a heal inside; `Unit::init@00612100` zeroes it.
    /// Item 3961 (`docs/AI.md` §405): run1151c's Citizens `0/5` and `0/8`
    /// read 45 on their civilian heal and count down, and who=1's planes
    /// in their Airbase read 1 one frame in 11.
    pub healing: i16,
    /// **`UnitData::cavarch_o`/`cavarch_who` (`+0xa2`/`+0xa8`)** and the
    /// uid beside them (`+0xa6`) — a caster's target, written by
    /// `Group::action_spell` and again by each frame of `do_cast`'s
    /// targeted arm (`docs/GOLDEN.md` §27). `None` is `cavarch_o` −1.
    pub cast_target: Option<combat::Obj>,
    /// **`cavarch_who` (`+0xa8`) itself**: 0 from `Unit::init`, the
    /// target's owner once `action_spell` or `do_cast` writes a target, and
    /// −1 after an untargeted press — `action_spell` writes the pair
    /// `(ox, whom)` = `(−1, −1)` whenever it differs from what is held
    /// (`docs/GOLDEN.md` §48).
    pub cavarch_who: i8,
    /// **`cavarch_o` −2** (`0xfffe`): a fire-on-the-move type's search
    /// found nothing twice running, and the next one waits for the
    /// thirty-two-frame phase (`Unit::cavarch_fight@005ff4b0`'s tail,
    /// `Unit::do_move@005f7b30:66`). `cast_target` is `None` with it.
    /// [`crate::cavarch`] writes it; −1 is `cast_target` `None` without.
    pub cavarch_idle: bool,
    /// **The head order's identity** — bumped by every write that puts a
    /// different order object at the head of the list (a kill, a remove at
    /// the head, a push in front, the ungroup's new plain move). It stands
    /// in for the original's pointer compare `pUVar1 == param_1` after
    /// `do_move` (`Unit::do_attack_to@005f2320`, `Unit::do_explore_to@005f24a0`):
    /// a move whose timer ran out is killed and the guard re-issues an
    /// `ATTACK_TO` to the same point, which a kind-and-point test calls the
    /// same order and the original does not (`docs/AI.md` §135).
    pub head_serial: HeadSerial,
    /// **`unit_masks & 0x20000`** — "the cast has started": set on a
    /// targeted cast's first in-range frame, cleared by
    /// `kill_current_order`, and one of the three bits that stop mana
    /// recovering (`docs/GOLDEN.md` §27).
    pub casting: bool,
    /// **`unit_masks & 0x8000`** — a hero on a Forced March, set by
    /// `SpellType::cast_march` and held to the craft's end frame, the
    /// `ActiveSpell` its hero record keeps (`docs/AI.md` §99.13); one of
    /// the three bits that stop mana recovering.
    pub marching: Option<i64>,
    /// **`unit_masks & 0x2000`** — a hero in an Ambush, set by
    /// `SpellType::cast_ambush@00671610` (`|= 0x2800`) and held to the
    /// craft's end frame, the `ActiveSpell` its hero record keeps, which
    /// `Caster::process_spells@00739ad0` expires (`&= 0xffffd7ff`); the
    /// third of the three bits that stop mana recovering (`docs/AI.md`
    /// §261).
    pub ambushing: Option<i64>,
    /// **`ObjectData::infiltrated` (`+0x3a`)** — the players who have an
    /// informer in this object, one bit per `who`: `cast_double_agent`
    /// sets the caster's (`docs/GOLDEN.md` §27).
    pub infiltrated: u8,
    /// **A plane's figure in the air** — guy 0's bank, pitch and altitude
    /// (`GuyData +0x44/+0x48/+0x4c/+0x50/+0x14/+0x70`), which
    /// `Unit::bank_aircraft` and `Unit::pitch_aircraft` step on every frame
    /// of a flight (`docs/ORDERS.md` §33). Zero on the ground and on every
    /// unit that never flies. A wild bird's bank is `crate::air::Flight`
    /// instead, which predates this and carries no altitude.
    pub airframe: air::Airframe,
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
    /// `unit_flags & 0x400`: the Kremlin's spy, which `Build::process`
    /// tags as it trains it and which stands outside the owner's `control`
    /// (`docs/AI.md` §188).
    pub kremlin_spy: bool,
    /// `ObjectData::mylos` (`+0x3c`): the line of sight in tiles **as
    /// `Unit::update_los` last wrote it** — a cache, not a derivation. The
    /// fog sweep and the follower's spacing read it; [`Sim::update_los`]
    /// is its one writer, called where the original calls vtable `+0x160`
    /// on a unit (`docs/VISION.md` §2).
    pub mylos: i32,
    /// **`UnitData::unit_masks2` (`+0x6c`)** — carried as the word the dump
    /// prints, and one bit of it is written: [`combat::umask2::NOT_FIRING`]
    /// (`docs/COMBAT.md` §43.2, `docs/ANIM.md` §5).
    ///
    /// Every dump on this disk carries this word as `0`, `4`, `0x10` or
    /// `0x40000` and nothing else, so the one bit is the whole of the
    /// field on both headline windows — which is why
    /// [`crate::Sim::guy_inc_time`]'s zero step could be implemented
    /// without modelling the rest of the word.
    pub unit_masks2: u32,
    /// **`ObjectData::hold_frames`** (`+0x32`) — how many more frames this
    /// slot is held after the figure inside it is gone (`docs/COMBAT.md`
    /// §11, §42.3).
    ///
    /// Three writers, all of them on a **dead** object: `Object::die` sets
    /// it to `max(1, …)` over the object's own ammo still in flight, so
    /// that a dead archer's arrow still lands; `Ammo::inc_time` bumps it
    /// once a frame for every in-flight shot whose **shooter** is no
    /// longer active; and `DeathObj::inc_time` bumps it once a frame while
    /// the death animation plays. Nothing writes it on a living unit,
    /// which is exactly what the dump says — 0 on every unit-frame of
    /// both headline windows — and why §9.2's "a target that is no longer
    /// active has its `hold_frames` bumped" was the wrong way round: the
    /// ammo matches on `+0x3c`/`+0x40`, the pair `Ammo::init` fills from
    /// the shooter.
    ///
    /// It holds nothing here: this crate never recycles a unit slot, so
    /// the field is carried to be **compared** rather than acted on.
    pub hold_frames: i32,
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
    /// `UnitData +0x80` — the pool slot this unit's **group back-pointer**
    /// names, `0..64` within its owner's block, or `None` for −1
    /// (`docs/GROUPS.md` §23). The dump prints it as `group`, `who·64 +
    /// slot`.
    ///
    /// It is its own state, not a view of any list: `Unit::set_group`,
    /// `Groups::push_group` and `get_open_slot`'s eviction write it,
    /// `Group::kill` and `do_group_move` clear it, and `Group::add` does
    /// not touch it at all. So a unit can name a group that does not
    /// list it and be listed by one it does not name, and every reader
    /// the original has goes through the pointer: the cap
    /// (`groups[+0x80].speed`), `do_group_move`'s first line,
    /// `Object::get_army` and `Group::normalize`'s prune.
    pub group_ptr: Option<u8>,
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
    /// cached incumbent: `do_move`'s captain retarget (§37.2) and
    /// `Unit::think`'s 31-in-32 fast path (§37.5, item 2049); this crate
    /// has both.
    pub near: Option<combat::Obj>,
    /// `unit_masks & 0x800000`: may auto-transport (`docs/TRANSPORT.md`
    /// §3). Granted at birth under the leader's level and by
    /// `check_transport`; the player's toggle writes it too.
    pub auto_transport: bool,
    /// `unit_masks2 & 0x2000`: a scenario's per-unit veto (§3.2).
    pub never_transport: bool,
    /// `guy_flags & 0x20`, which collapses the idle roll to two variants.
    /// `Unit::set_in_danger` raises it with [`Unit::in_danger`], and
    /// `Guy::process` drops it on a standing guy's sixty-fourth frame
    /// (item 530, `docs/ORDERS.md` §22). **It is guy 0's**:
    /// `set_in_danger` sets it on figures `0 .. guy_mark` alone, so a crew
    /// figure never carries it (`docs/ANIM.md` §13).
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
    /// Where this unit's collision disc stands in [`Sim::coll`] — **guy
    /// 0's unit cell as the occupancy last saw it**, not the unit's
    /// (`docs/COLLISION.md` §16). `CollCheck::move_unit` is called only
    /// from `Guy::set_new_location`, so the bits follow the figure: a
    /// unit moved by someone else's turn (a push) keeps its disc on the
    /// old cell until its own `Guy::process` next runs. `None` while it
    /// is not painted.
    pub coll_at: Option<Pos>,
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
    /// **`unit_masks & 0x10`, struck while moving in contact**
    /// (`Unit::target_opportunity@005fffc0`, `6002c8`): a unit hit by a
    /// unit while its front order is a move, with the attacker inside the
    /// two types' `big_radius` (`+0x244`) summed — unless its own action
    /// is an `ATTACK` whose target is inside twice that. `Object::do_damage`
    /// doubles the blow that raised it and every later one before it
    /// clears (`0064a5dd`), `UnitData::get_speed@00608720` halves a land
    /// unit's speed under it, and the end of `Unit::work@0060d180:449`
    /// clears it — on the paths that reach the end. The dump prints it on
    /// a unit struck after its own work ran (`docs/AI.md` §309).
    pub contact: bool,
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
    /// The other two writers go through `Unit::set_in_danger`, which
    /// marks the whole squad and its guys ([`Sim::set_in_danger`]):
    /// `Object::take_damage@00652020` marks a unit that is hit, and
    /// `Unit::work` marks an attacker and its target every frame the
    /// action is an `ATTACK` (item 530, `docs/ORDERS.md` §22). Nothing
    /// else in the engine reads the bit.
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
    /// `unit_masks & 0x1000`, "blown" (`UnitData::is_blown@0060a6e0`): a
    /// stealth unit that has worked an order is no longer hidden by
    /// `is_seen`'s stealth arm. Set by `Unit::work@0060d180:386` (an order,
    /// a `STEALTH`-not-`CLOAKED` type) and by every `|= 0x11000`; dropped
    /// by `check_idle@006032c0` once the idle counter reaches 10, and only
    /// while [`Unit::fought`] is clear (item 2785, `docs/GOLDEN.md` §103).
    pub blown: bool,
    /// `unit_masks & 0x10000`: `fight@005fd4d0:580`, `do_attack_ground` and
    /// `do_cast` write `0x11000` together; nothing this crate reaches
    /// clears the upper bit, so `check_idle` never drops `blown` after.
    pub fought: bool,
    /// `unit_masks & 0x800`, the cloak `UnitData::is_seen@00607a60` and
    /// `is_cloaked@0060a6a0` read: `Unit::process_cloak@005e1c20`'s
    /// Iroquois forest arm writes it (`docs/AI.md` §402).
    pub cloak: bool,
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
    /// The facing as the body's last `Guy::process` left it — what the
    /// body follow compares against to tell whether anything has turned
    /// guy 0 since (`guy_flags & 2`: set by every `Guy::do_turn` that
    /// moves the angle, a push by another unit included, and cleared only
    /// after `Guy::move`; item 3571).
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
    /// **`UnitData::unit_masks & 2`** — the unit's own mirror, the per-unit
    /// twin of `GroupData::facing` (`docs/GROUPS.md` §25). Two writers, by
    /// the listing: `Unit::set_angle@00605400` flips it whenever the new
    /// heading is `reversing` from the old (`605424`, for every unit, before
    /// the leader test that flips the group), and
    /// `Unit::kill_current_order@005e2cb0` writes a dying move's handed-back
    /// mirror into it (`5e3087`/`5e308d`, again before the leader test).
    /// One reader here: `Unit::do_guard@005e5c70` negates a guard's `dx`
    /// when its **target** carries it.
    pub mirror: bool,
    /// **The trackless crew's `des_angle` (`GuyData +0x64`) stands apart
    /// from its `angle` (`+0x18`)** — set by an order's `Unit::set_angle`
    /// with the snap flag clear ([`Sim::unit_set_angle`]), cleared by the
    /// next `Guy::do_turn`, by guy 0's `Guy::move` and by the snap (item
    /// 3762, `docs/GOLDEN.md` §149.3). `Unit::move_step`'s own `set_angle`
    /// is not counted: its `do_turn` follows it in the same step. It holds
    /// the frame it was written on, because guy 0's `Guy::move` turns the
    /// crew on every frame its heading is not its facing — whatever order
    /// the unit stands on — and this crate runs no body follow for some
    /// of them (a packer's unpack, the Bombard `1/94` of run1254 on 4428). `Guy::set_angle@005d9010`
    /// writes every crew figure's `+0x64` from `+0x304` up and leaves its
    /// `+0x18`; `Guy::do_turn@005d97a0` writes both equal on the figures
    /// it recurses into. Between the two, a trackless crew figure reads
    /// `des_angle != angle` exactly when guy 0's heading is not its facing
    /// — and `Guy::set_anim`'s attack test (`005da38a`–`005da3ba`) defers
    /// its swing. run1413's Bombard `2/23` on 759: `do_attack_ground`
    /// turns it and swings on one frame, and the two trackless crew pay
    /// the swing in `Guy::move+0x166`.
    pub crew_des: Option<i64>,
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
            mirror: false,
            crew_des: None,
        }
    }

    /// `Unit::set_angle`'s mirror flip (`605410`–`605424`): the new
    /// heading against the old `+0x50`, by `reversing`'s inclusive window.
    const fn flip_mirror(&mut self, to: movement::Angle) {
        let d = to.0.wrapping_sub(self.heading.0) as u32;
        if d >= 0x4000_0000 && d <= 0xc000_0000 {
            self.mirror = !self.mirror;
        }
    }

    /// Points the unit and its body a given way at once — `Guy::set_angle`
    /// with the snap flag, the one caller that moves the facing and the
    /// heading together. `Unit::move_step` never uses it: its `set_angle`
    /// passes zero for that flag, so a step moves the heading and leaves the
    /// facing to the turn rate.
    ///
    /// `Guy::set_angle` is not `Guy::do_turn`, so it leaves `guy_flags & 2`
    /// as it found it: the snap is not a turn the body's own `Guy::move`
    /// skips for, and [`Movement::frame_facing`] moves with the facing.
    pub const fn set_facing(&mut self, facing: movement::Angle) {
        self.flip_mirror(facing);
        self.facing = facing;
        self.frame_facing = facing;
        self.heading = facing;
        self.des_angle = facing;
        self.crew_des = None;
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
        self.flip_mirror(heading);
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
            && (foot_or_mounted || t.cols.flag(ai_load::uflags::FLAG_TRANSPORT))
            && !t.cols.flag(ai_load::uflags::CART),
        wide_limit: t.combat.domain != attrition::Domain::Land
            || objmask & combat::mask::VEHICLE != 0,
    }
}

/// A newborn's formation byte (`UnitData +0xaa`, the dump's `form`) —
/// `Unit::init@00612100:81`–`86`: **9** for the four civilian ids,
/// `PEASANTS` to `SCHOLARSKOREAN` (`0x32`–`0x35`, the type's `+4`), and
/// **0** for every other type. Its width twin (`+0xab`) is born −1, which
/// [`Unit::new`] already holds.
///
/// `Sim::group_action_move_to` reads it through `get_form`, and form 9
/// is one of the arms that keep a formation's member off the
/// `GroupMoveOrder`: two Citizens sent together walk two plain moves in the
/// original. Born −1 here, a group of Citizens took form 0 and one group
/// move (item 1330, chapter forty-seven's 838; parked 646).
pub fn init_form(type_index: i32) -> i8 {
    if (0x32..=0x35).contains(&type_index) {
        9
    } else {
        0
    }
}

/// A unit type the simulation knows how to build.
///
/// The price is the data's, the kind is what attrition and supply read, and
/// the group is which production building's output counts it — `None` for a
/// type without attack, which is every civilian in the shipped data. A type
/// whose progression counts by type is still counted in its group; only a
/// by-group progression reads the group back (`cost::Counts::of_group`).
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
    /// `MANA` (`UnitTypeData +0x2ec`), the pool `UnitData::mana@00609a50`
    /// reads for a caster. SEAM: its supply-upgrade, French and space arms.
    pub mana: i32,
    /// One of the twelve gaia types (`BASE_GAIATYPES..END_GAIATYPES`, record
    /// `352..`). The animals are unit types with a `WHERE` of Large City, and
    /// several of the original's loops stop before them.
    pub gaia: bool,
    /// What [`relative_value::relative_value`] reads of the type, fixed at
    /// load as `UnitType::fill_relative_values@0061be50`'s table is.
    pub rel: relative_value::Facts,
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
            hold_frames: 0,
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
            num_queued: 0,
            queue_time: 0,
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
            mana_burn: 0,
            full: 0,
            waiting: 0,
            healing: 0,
            cast_target: None,
            cavarch_who: 0,
            cavarch_idle: false,
            head_serial: HeadSerial(0),
            casting: false,
            marching: None,
            ambushing: None,
            infiltrated: 0,
            airframe: air::Airframe::default(),
            was_builder: false,
            carry: 0,
            cant_reach: false,
            decoy: false,
            kremlin_spy: false,
            mylos: 0,
            unit_masks2: 0,
            avoid: None,
            guys: Vec::new(),
            captain: true,
            o_up: None,
            o_down: None,
            group_ptr: None,
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
            coll_at: None,
            collide_o: -1,
            collide_who: -1,
            collide_guy: -1,
            safe: 0,
            waiting_on: false,
            half_step: false,
            contact: false,
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
            blown: false,
            fought: false,
            cloak: false,
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

/// `Objects::find_free@0065ad60`'s reuse test on one number (`docs/COMBAT.md` §59.2): the
/// object is dead (`flags & 1` clear), its `hold_frames` (`+0x32`) is zero, and it is either no
/// unit (vslot `+0x18` answers 0) or a captain (`UnitData +0x8e` negative).
pub(crate) fn find_free_reusable(alive: bool, hold: i16, is_unit: bool, captain: bool) -> bool {
    !alive && hold == 0 && (!is_unit || captain)
}

/// `Objects::find_free@0065ad60` with `param_5 < 0`: the first number of `[base, *mark)` that
/// `reusable` accepts, tested in order; failing that `*mark` itself, which moves up one, unless
/// it has reached `limit` (then -1).
pub(crate) fn find_free_steps(
    base: i32,
    limit: i32,
    mark: &mut i32,
    mut reusable: impl FnMut(i32) -> bool,
) -> i32 {
    for i in base..*mark {
        if reusable(i) {
            return i;
        }
    }
    if limit <= *mark {
        return -1;
    }
    *mark += 1;
    *mark - 1
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
    /// at its next pass, before any unit moves. Its writers modelled here:
    /// a change in the player's rare mask (`Sim::calc_unit_stats`),
    /// `Leader::gain_tech`'s tail, and a wonder's `Build::activate` and
    /// `Build::close` (`docs/VISION.md` §2).
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
    /// `Groups::last_group[who]` — the pool slot each player pushed into
    /// last, which `push_group` compares against and `get_open_slot` will
    /// not hand out (`docs/GROUPS.md` §19). `Groups::clear` starts every
    /// player on its own slot 0.
    pub(crate) last_group: [u8; 16],
    /// The record `Group::action_siege_attack_to` sets aside while its
    /// stack sub-group holds the seat (`docs/GROUPS.md` §26.3), with the
    /// seat it belongs to: owner, army, pushed slot. A member's own dying
    /// move hands its mirror back through the unit's `+0x80`, which names
    /// the **real** group and never the stack (`docs/AI.md` §212), so
    /// [`Sim::facing_record_mut`] sends that write here. `None` outside the
    /// call.
    pub(crate) set_aside: Option<(Player, Option<usize>, Option<usize>, group::GroupState)>,
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
    /// The nukes' effects and what a nuke writes on its shooter's leader
    /// and silo (`crate::nuke`, item 1091).
    pub nukes: nuke::Nukes,
    /// How many [`Sim::ground_z`] reads have touched a height corner that
    /// is not known to be the original's own single — one whose six-decimal
    /// print a neighbouring single shares, or one a terraform rewrote in
    /// millionths (`docs/COMBAT.md` §46.3). Every window the harness replays
    /// pins it, so the day a read lands on one is a failing test by name.
    pub ground_inexact: u32,
    /// **The death objects**, the `DEATH_OBJS` list the dump prints at the
    /// frame level (`docs/COMBAT.md` §42.1). One is made by `Unit::close`
    /// for every unit death whose `dtype` is non-zero, and it carries the
    /// animation index the draw at `Unit::close+0xcb6` chose.
    ///
    /// Each figure's record ends at its life (`DeathObj::inc_time`,
    /// [`Sim::death_objs_inc_time`], `docs/GOLDEN.md` §127), and the pool
    /// of 100 behind them evicts the oldest non-blocking record when full
    /// ([`Self::death_pool`], §177).
    pub deaths: Vec<combat::Death>,
    /// **The death pool's slots** (item 3971, `docs/GOLDEN.md` §177):
    /// which figure's record each of `Objects`' 100 `DeathObj` slots
    /// holds, in slot order, grown to [`combat::DEATH_POOL`] as records
    /// arrive. [`Self::deaths`] is the list the dump prints; this is the
    /// capacity behind it, and the order `Objects::add_death` evicts in.
    pub death_pool: Vec<Option<combat::DeathSlot>>,
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
    /// `LeaderData::agendas` (`+0xb4`), `int[8]`. One bit is modelled:
    /// **`2`, "your building stood in my armed unit's walk"**, which
    /// `Unit::resolve_block@005fccc0` ors into its owner's slot for a
    /// building of a player it is not at war with (`5fcdda`, item 1209);
    /// one direction only, where `treaty_on` writes both.
    pub agendas: Vec<Vec<i32>>,
    /// `LeaderData::flock_stamp` (`+0x7b8`): the frame of the player's
    /// last flock of birds, which `Objects::add_flock` reads to allow one
    /// in 225 frames per player (`crate::flock`). The leader dump prints
    /// it.
    pub flock_stamp: Vec<i64>,
    /// `leader_flags & 0x8000` per player: one of its heroes is on a
    /// Forced March. `cast_march` sets it and `Leader::verify_spell_flags`
    /// clears it once no hero's `unit_masks & 0x8000` stands; the hero
    /// arm of `think_spellcaster` throws no coin while it is up
    /// (`docs/AI.md` §99.13).
    pub forced_march: Vec<bool>,
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
    /// `Game::wonders` (`Game +0x618`, `int[17]`), a bit per
    /// [`tech::wonder`] offset: a wonder type some player has activated.
    /// `Wonders::init_wonder@0073c860` sets it (`Wonder::init@0073c5e0` has no caller),
    /// nothing clears it, and `BuildTypeData::already_built@0063ce10` reads
    /// it for [`Sim::type_avail`] (`docs/TECH.md`, "A built wonder is
    /// built for everyone").
    pub wonders_built: u32,
    /// Every region's `Region::borders` resume index reset since the last
    /// `GameDaemon::check_borders` (`Region::fix_borders@00680f60`): the
    /// next one zeroes `reg_known_rares` ([`Sim::fix_borders`]).
    pub borders_fixed: bool,
    /// **The fixed pass still reaching the map** (item 1106,
    /// [`border_pass`]): the owners a fix computed and every region's resume
    /// index. `None` once every land region is done.
    pub border_pass: Option<border_pass::BorderPass>,
    /// Per cell (row-major), the city whose claim [`Sim::recompute_territory`]
    /// last found winning it, `−1` for none or a fort's — `compute_reg_territory`'s
    /// `local_1c`, which names the city a runner-up marks `bordering`
    /// (`docs/AI.md` §167).
    pub territory_won: Vec<i32>,
    /// `GoodData::ever_seen` (`+0x20`), the bits `World::compute_reg_territory`'s
    /// per-cell goods scan sets beside `Leader::new_rare` (item 1555,
    /// `docs/AI.md` §152), by goods-list index. `reveal_fog`'s own writes are
    /// answered by the fog read in [`Sim::good_ever_seen`].
    pub good_seen_bits: Vec<u8>,
    /// The goody boxes as `Objects::init_item` laid them down, with each
    /// one's `ItemData::ever_seen` (`+0x20`): a bit per leader that
    /// `World::reveal_fog` ORs in when a unit's sight first changes `seen2`
    /// on one of the box's half-cells, and nothing else (item 3335,
    /// `docs/GOODY.md` §7.2). Empty when the world was stood up without
    /// the dump's `ITEM` records; [`Sim::goody_item_is_seen`] then reads
    /// the fog in its place.
    pub goody_items: Vec<goody::GoodyItem>,
    /// Whether a frame has run. A border fix before the first is setup's —
    /// `World::compute_all_territory` with the budget unlimited, on the
    /// map at once — and one after it takes the daemon's pass
    /// ([`border_pass`], item 1106).
    pub in_play: bool,
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
    /// The map generator's placed mountains, in its own order
    /// (`MountainsData::mountain_locs`/`_types`), each with its template's
    /// tiles and solid cells laid at its location (`gather.rs`,
    /// [`gather::PlacedMountain`]). Empty when nothing supplied them, and
    /// then [`Sim::mountain_range`] stands a range up from the tiles.
    pub mountains: Vec<gather::PlacedMountain>,
    /// `CliffsData::cliff_mining_data`, the scary cliffs a mine may stand
    /// against, by queue index ([`cliff_mine::CliffMining`]). Empty when
    /// nothing laid them ([`Sim::place_cliffs`]); no dump prints them.
    pub cliffs: Vec<cliff_mine::CliffMining>,
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
    /// A unit cell `valid_ucoord` refuses to one unit on one frame
    /// ([`path::RefuseProbe`]), and `None` in every run but a test's.
    #[doc(hidden)]
    pub probe_refuse: Option<path::RefuseProbe>,
    /// **`PathFinder +0x50`, the validity memo — the pathfinder's, not a
    /// search's** (`docs/PATHFINDER.md` §24.5). `valid_ucoord` caches each
    /// 48-cell's verdict here under its metric, and only
    /// `PathFinder::kill_lists@00687ae0` empties it. Every finder calls that
    /// **after** its `astar_*` returns and on none of its early returns, so
    /// a `find_upath` that ends in its goal pre-walk leaves its verdicts for
    /// the next unit's search, on the same frame or a later one.
    pub path_memo: std::collections::BTreeMap<i64, bool>,
    /// **`PathFinder +0x4c`, the `blocklist` — the pathfinder's copies of
    /// world cells' collision blocks, with the memo's lifetime**
    /// (`docs/PATHFINDER.md` §22 of item 678, `docs/COLLISION.md` §4.2). A
    /// `nocoll` probe — `valid_ucoord`'s, and `resolve_unit_collision`'s
    /// unwind — reads a world cell's copy when the tree holds one and
    /// otherwise the live block, and copies each slot it read live
    /// (`CollCheck::fill_slots@006820e0`). Only `kill_lists` empties it,
    /// so a copy taken by a probe outside any search is what the next
    /// search reads, however many frames later. Keyed on the world cell;
    /// the value is its 16×16 unit cells, one row a word, as the probe's
    /// region gate left them.
    pub coll_copies: std::cell::RefCell<std::collections::BTreeMap<(i32, i32), [u16; 16]>>,
    /// `WData::down` per world cell: the head of the object chain (§3).
    pub chain_heads: Vec<Option<usize>>,
    /// `GameDaemon::repaths[who]`: how many 48-grid recoveries this player
    /// has asked for, the throttle collision recovery reads (§6 step 6).
    pub repaths: Vec<i32>,
    /// `PathFinder +0x58/+0x5c`, **the start argument's tile** the wrappers
    /// write (`find_wpath@00688fc0:47-48`, `find_tpath@006897d0:40`,
    /// `find_upath@00682f30:66-67`): the depth-1 shore test in `calc_cost`
    /// reads it, not the unit's own tile (twenty-fourth pass, group 7; A4
    /// row 28).
    pub(crate) pf_start: Pos,
    /// The order a frame's building walk ran its five steps in — written
    /// only under test — for the test that holds `Build::process`'s sequence (group 18).
    #[allow(dead_code)]
    pub(crate) building_log: Vec<(usize, &'static str)>,
    /// `LeaderData::retargets` (`+0x9f4`): the frame's count of attacks
    /// whose target went invalid. `Leader::process@006b88b0` zeroes it
    /// at the head of each leader's frame, `Unit::fight@005fd4d0`'s
    /// invalid-target tail adds one, and `resolve_unit_collision`'s
    /// enemy ladder lets a captain re-search only while it is under ten
    /// (`docs/COLLISION.md` §14). Not dumped per frame at `LEADERS=5`.
    pub retargets: Vec<i32>,
    /// **A melee-and-ranged type's second range, swapped in for one call**
    /// (item 3383, `docs/AI.md` §291): `Unit::cavarch_fight@005ff4b0`
    /// writes the type's `second_max_range` (`UnitTypeData +0x2d0`) into
    /// its `max_range` (`+0x1fc`) before `fight` when the type's flags
    /// carry `0x400` ([`ai_load::uflags::MELEE_AND_RANGED`]), and 0 back after. The
    /// write is to the **type**, so every reader inside the call sees it;
    /// [`Sim::profile`] answers the type's second range for a unit whose
    /// type this holds. `None` outside such a call.
    pub(crate) second_range: Option<usize>,
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

pub use farms::Farm;

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
            last_group: [0; 16],
            set_aside: None,
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
            nukes: nuke::Nukes::default(),
            ground_inexact: 0,
            deaths: Vec::new(),
            death_pool: Vec::new(),
            market: market::Market::default(),
            gaia: gaia::Gaia::default(),
            art: anim::Art::default(),
            mountains: Vec::new(),
            cliffs: Vec::new(),
            game_seed: 0,
            mods: vec![combat::Modifiers::default(); players],
            hits: Vec::new(),
            build_types: Vec::new(),
            cities: Vec::new(),
            nation: vec![city::Nation::default(); players],
            borders: vec![territory::PlayerBorders::plain(&tuning); players],
            allied: vec![vec![false; players]; players],
            treaties: vec![vec![0; players]; players],
            agendas: vec![vec![0; players]; players],
            flock_stamp: vec![0; players],
            forced_march: vec![false; players],
            defeated: vec![false; players],
            lost_city_stamp: vec![None; players],
            city_tally: vec![city::Tally::default(); players],
            building_high: vec![Vec::new(); players],
            removed: Vec::new(),
            wall_stats_dirty: vec![false; players],
            wonders_built: 0,
            borders_fixed: false,
            border_pass: None,
            territory_won: Vec::new(),
            good_seen_bits: Vec::new(),
            goody_items: Vec::new(),
            in_play: false,
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
            probe_refuse: None,
            path_memo: std::collections::BTreeMap::new(),
            coll_copies: std::cell::RefCell::default(),
            chain_heads: vec![None; (world.width() * world.height()) as usize],
            repaths: vec![0; players.max(10)],
            pf_start: Pos::default(),
            building_log: Vec::new(),
            retargets: vec![0; players.max(10)],
            second_range: None,
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
        self.retargets.push(0);
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
        for row in &mut self.agendas {
            row.push(0);
        }
        self.agendas.push(vec![0; who + 1]);
        u8::try_from(who).expect("too many players")
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

    /// `Object::insert_inside@00647e90` (`delta` 1) and
    /// `remove_from_inside@006480f0` (−1): a captain going into or out of a
    /// building moves one of the leader's garrison counts — a peasant, else
    /// a Stable type (`+0x40 == 0x1ac`), else `is_special` (the scout bit),
    /// else a Barracks type (`0x1ab`) — and with `LAKOTA_FOOD` set a Lakota
    /// leader's `0x2000000` (`docs/AI.md` §219), with
    /// `AMERICANS_BARRACKS_GATHER` set an American's for the Barracks arm
    /// (§292).
    ///
    /// The −1 is [`remove_inside_view::remove_from_inside_steps`] itself, the
    /// whole function swept under the emulator (item 3372); the +1 is its
    /// tail alone, [`remove_inside_view::garrison_tail`] behind the captain
    /// test — `insert_inside` is not swept.
    ///
    /// **A decoy moves no count** (item 3964, `docs/AI.md` §408): both
    /// functions test the unit record's `unit_masks & 1` (`+0x68`) ahead of
    /// the captain test — `insert_inside@00647e90`'s `(piVar2[0x1a] & 1) ==
    /// 0`, `remove_from_inside`'s the same — so a French decoy garrisoned
    /// counts as nothing.
    ///
    /// SEAM: the container's slot `0x20` is not modelled, nor
    /// `insert_inside`'s `in_a_building` — every caller here puts the unit in
    /// a building.
    pub(crate) fn track_garrison(&mut self, u: usize, delta: i32) {
        use remove_inside_view::{RemoveInsideView, SimRemoveInside};
        let mut view = SimRemoveInside { sim: self, unit: u };
        if delta < 0 {
            remove_inside_view::remove_from_inside_steps(&mut view);
        } else if !view.masked() && view.is_captain() {
            remove_inside_view::garrison_tail(&mut view, delta);
        }
    }

    /// Adds a unit, registering it as a supply source if its type is one.
    ///
    /// This is `Unit::init`'s half of the supply bookkeeping and the reason to
    /// prefer it over pushing onto `units` directly: a supply wagon that never
    /// registered supplies nobody, silently.
    pub fn add_unit(&mut self, unit: Unit) -> usize {
        // `Unit::init` preserves an existing guy's point until seating it;
        // the collision move uses the new type's radius (COLLISION §23).
        // Only the latest occupant of the object number owns that figure.
        let previous_body = self
            .units
            .iter()
            .rev()
            .find(|old| old.owner == unit.owner && old.index == unit.index)
            .filter(|old| !old.alive() && !old.guys.is_empty())
            .map(|old| old.movement.body.pos);
        // **`path_recursion` (`UnitData +0xaf`) survives the slot**: only
        // `UnitData::UnitData@00606670` zeroes it, and `Unit::init@00612100`
        // never writes it, so a recycled object number keeps its last
        // occupant's count until the next `find_path`. French East Indies'
        // Hoplite captain `1/77`, born on 10765 into the slot a dead `1/77`
        // left, reads 1 on run630's 10766 (item 1458, `docs/GROUPS.md` §37.2).
        let mut unit = unit;
        if let Some(old) = self
            .units
            .iter()
            .rev()
            .find(|old| old.owner == unit.owner && old.index == unit.index && !old.alive())
        {
            unit.path_recursion = old.path_recursion;
        }
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
        // **`UnitData::get_combat_stance@00610a30` answers
        // `COMBAT_STAND_GROUND` for a type with no combat stances**
        // ([`crate::stance::get_stance`]): the `+0xb1` byte is the packer's,
        // the worker's or the caster's own, and every reader of vslot
        // `+0xf4` sees 2. This crate keeps the combat half apart as
        // [`combat::Stance`], so it is written here once. run842's packet
        // (item 2265, `docs/GOLDEN.md` §82): the Catapult `0/9`'s idle
        // search on tick 776 runs `find_nearby_target(0xc00, …)` — no
        // AGGRESSIVE `+0x180` — and never reaches the Small City at 2781.
        // The predicate is `UnitTypeData::get_stance_type@0061d350`'s
        // `Combat` arm: `role & MILITARY` and not `unit_flags2 & 4`. The
        // loader fills the profile's `combat_role` and `packs` from those
        // two columns, and a hand-built type may set either spelling.
        if self.units[i].ty.is_some_and(|t| {
            let ty = &self.unit_types[t];
            let military = ty.combat.combat_role || ty.cols.is(crate::ai_load::role::ROLE_MILITARY);
            let packs = ty.combat.packs || ty.cols.flag2(crate::ai_load::uflags2::PACKS);
            !military || packs
        }) {
            self.units[i].combat.stance = crate::combat::Stance::StandGround;
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
                .flag2(crate::ai_load::uflags2::FLAG2_CARAVAN)
                && self.unit_types[ty].combat.domain == attrition::Domain::Land
        }) {
            let who = self.units[i].owner;
            self.units[i].caravan = self.init_caravan(who, i);
        }
        // `Unit::init@00612100`'s own `update_los` (vtable `+0x160`), on the
        // finished unit — packed bit and all — before `add_to_world` reads
        // it for the disc (`docs/VISION.md` §2).
        self.update_los(i);
        // `Object::add_to_world`: both collision indices
        // (`docs/COLLISION.md` §2, §3) **and the vision disc**.
        if let Some(from) = previous_body {
            self.coll_rebirth(i, from);
        } else {
            self.coll_add(i);
        }
        if self.units[i].alive() && self.units[i].on_map {
            self.chain_add(i);
            // `update_seen(0)`, the whole disc rather than the ring
            // (`docs/VISION.md` §6). This was long credited to
            // `add_to_world`; item 3361 read it as `Unit::init`'s own call,
            // at the trainer's snapped centre — the same disc, so the same
            // effect — and [`Sim::come_out_place`]'s inside arm lights
            // none of its own. A unit *born* on the map once made none, so its owner's line of sight did not exist
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
        // A number nothing carries is free; one an object holds is not.
        let mut free = vec![true; (mark - base).max(0) as usize];
        if base == BUILD_BASE {
            for b in &self.buildings {
                let i = b.index - base;
                if b.owner == who && (base..mark).contains(&b.index) {
                    free[i as usize] &=
                        find_free_reusable(b.alive, i16::from(b.hold_frames != 0), false, true);
                }
            }
        } else {
            for u in &self.units {
                let i = u.index - base;
                if u.owner == who && (base..mark).contains(&u.index) {
                    // The reuse test's captain term, `UnitData +0x8e`
                    // (`o_up`) negative (`0065adde`): a dead tail still
                    // linked under a live squad is held. 2400 read it so and
                    // the third pair fell (parked 2441) because the squad's
                    // last death left its dead tail linked here;
                    // [`Sim::relink_squad`]'s whole-squad arm clears it, as
                    // `Unit::close` does (`docs/AI.md` §324.2).
                    free[i as usize] &= find_free_reusable(
                        u.alive(),
                        i16::from(u.hold_frames != 0),
                        true,
                        u.o_up.is_none(),
                    );
                }
            }
        }
        let mut m = i32::from(mark);
        let got = find_free_steps(i32::from(base), i32::from(limit), &mut m, |i| {
            free[(i - i32::from(base)) as usize]
        });
        if base == BUILD_BASE {
            self.marks[w].build = m as i16;
        } else {
            self.marks[w].unit = m as i16;
        }
        (got >= 0).then_some(got as i16)
    }

    /// The live unit of `who` numbered `o`, if any — the object a script or
    /// a producer holds by handle.
    pub fn unit_by_o(&self, who: Player, o: i16) -> Option<usize> {
        self.units
            .iter()
            .position(|u| u.owner == who && u.index == o && u.alive())
    }

    /// `who`'s unit slots in the original's list order — `units[who][o]`
    /// walked from 0, which is object number, not this crate's slot order
    /// (a reused number sits at the end of `self.units`). Dead units are
    /// kept; a caller filters on `alive()` as the original's `vtable+8`
    /// does.
    pub(crate) fn units_by_o(&self, who: Player) -> Vec<usize> {
        let mut v: Vec<usize> = (0..self.units.len())
            .filter(|&u| self.units[u].owner == who)
            .collect();
        v.sort_by_key(|&u| self.units[u].index);
        v
    }

    /// The live building of `who` numbered `o`, if any.
    pub fn building_by_o(&self, who: Player, o: i16) -> Option<usize> {
        self.buildings
            .iter()
            .position(|b| b.owner == who && b.index == o && b.alive)
    }

    /// Whether building `b`'s object number has gone to a later building —
    /// the only way the original's `uid` test on it can fail, because a
    /// closed building keeps its uid until `Objects::find_free` reuses the
    /// number. A handle that was never valid counts as reused.
    pub(crate) fn building_slot_reused(&self, b: usize) -> bool {
        let Some(bd) = self.buildings.get(b) else {
            return true;
        };
        self.buildings[b + 1..]
            .iter()
            .any(|x| x.owner == bd.owner && x.index == bd.index)
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
    ///
    /// The peacocks term is read off the live rare mask, as the original's
    /// `testb $0x8, 0x6da6(%esi)` reads it on every call ([`economy::PEACOCKS`]),
    /// so a recompute for any other cause sees the mask as it stands. The
    /// Colossus term is `has_wonder(0x20f)`, asked live on the same call
    /// (`006dc490`'s last statement, item 3326, `docs/AI.md` §266).
    pub fn recompute_pop_caps(&mut self) {
        let peacocks = 1u64 << (economy::PEACOCKS - economy::BASE_RARE);
        let colossus: Vec<bool> = (0..self.muster.len())
            .map(|who| {
                u8::try_from(who)
                    .is_ok_and(|p| self.wonders_held(p) & (1 << tech::wonder::COLOSSUS) != 0)
            })
            .collect();
        for (who, m) in self.muster.iter_mut().enumerate() {
            m.bonuses.peacocks = self
                .ledgers
                .get(who)
                .is_some_and(|l| l.rare & peacocks != 0);
            m.bonuses.colossus = colossus[who];
            // `Leader::calc_pop_cap`'s `has_tribe_bonus(3)` (`:68`–`73`): the
            // flag [`cost::pop_cap`]'s Bantu arm reads, which had no writer
            // until the Bantu word's block 1 printed who=1's cap at 50
            // against this crate's 25 (run978, `docs/AI.md` §276).
            m.bonuses.bantu = self.nation.get(who).is_some_and(|n| n.bantu);
            m.cap = cost::pop_cap(&self.tuning, m.military_level, m.limit, &m.bonuses);
        }
    }

    /// `Unit::init@00612100` on one object number — the pass of
    /// [`Sim::init_unit`]'s loop, and what `Unit::repair_damage@0060de10`
    /// calls through vslot `+0x8c` when it regrows a figure (item 3993,
    /// `docs/GOLDEN.md` §186): the unit, its stance, its birth masks, and
    /// `Guy::init_real`'s draw. `pos` is the snapped birth point.
    pub(crate) fn init_figure(&mut self, who: Player, ty: usize, index: i16, pos: Pos) -> usize {
        let mut unit = Unit::new(who, index, pos, self.unit_hits(who, ty));
        unit.kind = self.unit_types[ty].kind;
        unit.ty = Some(ty);
        unit.type_index = self.unit_types[ty].type_index;
        unit.form = init_form(unit.type_index);
        // The stance is a switch on the *type's* stance kind and then,
        // when the trainer shares that kind, the trainer's own byte —
        // `Unit::init@00612100:282–309` and `Build::train@0062f9b0:86`
        // (`crate::stance`, `docs/ORDERS.md` §5.10). A trainer is what
        // [`Sim::build_train`] passes; a free-standing spawn has none.
        unit.stance = self.init_stance(who, ty);
        unit.movement.speed = self.type_speed(who, ty);
        unit.movement.turning = self.turning_for(ty);
        let u = self.add_unit(unit);
        // `Unit::init@00612100:592`: a spy (`is(SPY, 0)`, the lineage)
        // is born with half its pool spent.
        if self.unit_is_spy(u) {
            self.units[u].mana_burn = i16::try_from(self.unit_mana(u) / 2).unwrap_or(0);
        }
        // `Unit::init@00612100:598-611` (item 2165, `docs/GOLDEN.md`
        // §79): a unit of the Bark line (`is(BARK)`, the lineage) takes
        // `unit_masks2 |= 4`, and a computer's (`leader_flags & 4`
        // clear, `is_human` inlined) `Unit::set_stance@00605310(3, 0)`,
        // RAID, over the stance the switch above gave it. run835's
        // `1/8` is born `stance 3`, the human `0/6` and `0/7` at 0;
        // every AI Bark on disk carries both.
        if self.unit_line_is(u, crate::army::BARK) {
            self.units[u].unit_masks2 |= crate::combat::umask2::MASK2_BARK;
            if !self.nation.get(who as usize).is_some_and(|n| n.human) {
                self.units[u].stance = 3;
                self.units[u].combat.stance = crate::combat::Stance::Raid;
            }
        }
        self.init_iroquois_masks(u);
        // `Unit::init@00612100:613-626` (item 2351, `docs/AI.md` §219):
        // a Lakota unit the food term counts — a citizen (`TypeIndex`
        // `0x32`, `0x33`), a scout (`is(SCOUT, 0)`) or a Stable type
        // (`+0x40 == 0x1ac`) — raises the leader's `0x2000000`.
        if self.nation.get(who as usize).is_some_and(|n| n.lakota)
            && self.tuning.lakota_food != 0
            && (matches!(self.unit_types[ty].type_index, 0x32 | 0x33)
                || self.unit_line_is(u, 0x45)
                || self.trainer_ident(ty) == Some(build::Ident::Stable))
        {
            self.economy_changed(who);
        }
        // `Unit::init@00612100:629-641` (item 3384, `docs/AI.md` §292):
        // an American Barracks type (`+0x40 == 0x1ab`) that is not a
        // scout, with `AMERICANS_BARRACKS_GATHER` set, raises it too.
        if self.american_barracks_unit(who, u, ty) {
            self.economy_changed(who);
        }
        // `Unit::init` → `Guy::init_real`: the figure's one draw.
        // (The unit's `ty` stays unset here, as it always has; the
        // piece lookup takes the type directly.)
        self.init_guys(u, Some(ty));
        // **`Unit::init@00612100:549`'s `set_new_location(x, y, 1, 1)`**
        // (item 3804, `docs/GOLDEN.md` §152.3): its guy half is
        // `Guy::set_new_location`'s, so a helicopter's figure takes
        // its first climb at birth ([`Sim::helicopter_climb`], read
        // here off the type, which the unit does not carry yet).
        // run1469's Attack Helicopters fly 30 higher than ours did
        // from their first frame, and their rounds aim at that.
        if self.unit_types[ty].cols.unit_flags & crate::ai_load::uflags::FLAG_HELICOPTER != 0
            && self.unit_types[ty].combat.domain == crate::attrition::Domain::Air
        {
            let ground = self.ground_z(self.units[u].pos);
            let af = &mut self.units[u].airframe;
            af.z += (ground - af.z + 1000).clamp(-30, 30);
            af.last_z = af.z;
        }
        // `Unit::init@00612100:467`: a government patriot's birth is
        // stamped on its leader, and that stamp is what keeps a
        // Senate from training a second one.
        if self.unit_types[ty]
            .tree
            .is_some_and(|t| self.tech_tree.kind(t).unit().is_some_and(|x| x.patriot))
        {
            self.tech[who as usize].gov_hero_frame = self.frame;
        }
        u
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
        // **`Unit::init@00612100:68-73` snaps the birth point for every
        // unit**, whatever its owner or domain (twenty-fourth pass, group
        // 17; A3 row 3): a trained unit at its trainer's whole-tile centre
        // is born 24 south and east of it, which a unit kept inside shows.
        let pos = Pos::new(gaia::init_snap(pos.x), gaia::init_snap(pos.y));
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
            let u = self.init_figure(who, ty, index, pos);
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
                        let before = self.units[u].pos;
                        self.set_new_location(u, spot, true);
                        // **`Unit::set_new_location@005f8d20`'s world-cell
                        // change** brackets the write with `remove_from_world`
                        // and `add_to_world`, which ends in `update_seen(0)`:
                        // the seated member lights the disc at its own spot
                        // (`docs/GOLDEN.md` §72). Only this seating is
                        // built: the same call on every cell crossing moved
                        // the successor pair's first draw parting 3188 → 1387.
                        let (a, b) = (before, self.units[u].pos);
                        if (a.x >> 8, a.y >> 8) != (b.x >> 8, b.y >> 8) {
                            self.update_seen(u, false);
                        }
                    }
                }
                (Some(_), None) => unreachable!("a head is set with its own index"),
            }
            prev = Some(u);
        }
        head.expect("uber_size is at least one")
    }

    /// `Unit::set_type@00612fa0(t, SET_TYPE_NORMAL)`, as far as this
    /// simulation carries state for it.
    ///
    /// The order is the original's: the leader's counters come **off** for
    /// the old type before the swap (`6130d1`'s `track_unit_type(·, −1,
    /// ·)` and the two running totals), the record and the derived stats
    /// change, the guys are re-initialised ([`Sim::reinit_guys_begin`]), and the
    /// counters go **on** for the new one. Only a captain is counted, the
    /// same rule `Objects::init_unit` follows for a squad.
    ///
    /// **Damage carries.** `set_type` never writes `myhits`; the object's
    /// `damage` (`ObjectData +0x24`) is untouched by the swap, so a unit
    /// converted at half health is at the *new* type's hits minus the same
    /// damage. This crate stores the complement, so the subtraction is
    /// explicit.
    ///
    /// The tail's `update_gpiece` is [`Sim::reinit_guys_begin`]' piece write, and
    /// each `init_real` re-pieces its guy before it reads the flags, so a
    /// converted guy's `guy_flags & 8` is the new piece's
    /// ([`crate::anim::Guy::flag_piece`]).
    ///
    /// **No same-type guard** (item 3787, `docs/GOLDEN.md` §150.2): the
    /// original's `set_type` re-initialises the guys whatever the old type
    /// was, so a Senate's second government `set_type`s a standing Despot
    /// to The Despot and spends its three `init_real` draws — who=1's `1/30`
    /// on the second melee's frame 828. A caller that must not convert in
    /// place tests for itself, as `gain_tech`'s upgrade walk does.
    ///
    /// SEAM: the `is(0x165, 1)` CEO bit and its `update_ceo_position`, the
    /// `is(0x77, 0)` flag, and the two vslots `+0x15c` and `+0x160` the
    /// tail calls before `update_armor`/`update_speed` — none has state
    /// here.
    pub(crate) fn unit_set_type(&mut self, u: usize, rec: usize) {
        if self.units[u].ty.is_none() {
            return;
        }
        // The steps are `unit_set_type_view::set_type_steps`, the walk the
        // sweep `unit_set_type_normal` holds to the original's bytes; the
        // adapter says what this crate keeps of each.
        let mut view = set_type_adapter::SimSetType::new(self, u, rec);
        unit_set_type_view::set_type_steps(
            &mut view,
            rec as i32,
            unit_set_type_view::SET_TYPE_NORMAL,
        );
    }

    /// **`Object::take_damage@00652020`'s remainder test** (decompile lines 311-329,
    /// the `+0x308` block): a figure of a squad type takes `hits − (uber − 1)
    /// · hits / uber` — the remainder — only when it is a captain (vslot
    /// `+0xe8`) **and** `UnitData::curr_uber_size(this, 1)` answers 1, the
    /// last figure standing; every other figure takes `hits / uber`. The
    /// count is live: this crate read its stored [`Unit::squad_size`], which
    /// nothing outside the tests writes past 1, so every captain of a whole
    /// squad took the remainder and outlived its members by a point (item
    /// 3618, `docs/AI.md` §326.2: Great Sahara's six Elite Javelineers
    /// squads on 4442, `1/200` falling at 37 where theirs falls at 36).
    pub(crate) fn takes_remainder_share(&self, i: usize) -> bool {
        let u = &self.units[i];
        u.combat.captain == i32::from(u.index) && self.curr_uber_size(i) == 1
    }

    /// The squad a captain heads, captain first — `o_down` walked.
    /// `UnitData::curr_uber_size`: the figures in `i`'s squad now — up
    /// the `o_up` links to the captain, then the captain's chain. A dead
    /// figure is **not** relinked out: `Unit::close` re-appends it at the
    /// chain's tail, and the walk stops at it because `0060a760` follows
    /// `o_down` only while the next figure's `flags & 1` holds, the bit
    /// `Object::close@00647160` clears (item 3612: run1254b's `1/46` dies
    /// on 3225 and `1/47` takes the two-figure share on the same block).
    pub fn curr_uber_size(&self, i: usize) -> i32 {
        squad_size(
            i,
            |u| self.units[u].o_up,
            |u| self.units[u].o_down.filter(|&d| self.units[d].alive()),
        )
    }

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

    /// `Leader::calc_attrition@006cdea0`: the attrition this player's
    /// territory inflicts, from the leading run of `ATTRITION1..4` held
    /// ([`tech::Roles::attrition_preq`]) and the Russian scaling. The
    /// original calls it from `gain_tech`, `calc_unit_stats` and
    /// `Leader::init`; here it runs from [`Sim::apply_gained`], which both
    /// tech paths reach.
    ///
    /// **Unwired until item 552.** Phase 1 built [`attrition::strength`]
    /// and tested it against hand-set [`attrition::PlayerState`]s, and
    /// nothing ever wrote `strength` from the tech tree, so every war-zone
    /// refresh came out at the sentinel 0. Golden chapter four's Allegiance
    /// at 550 gave the original's squad a period of 48 on block 601 and
    /// this crate's 0. The Colosseum and Kremlin arms stay false: this
    /// crate holds no wonder a player owns.
    pub(crate) fn calc_attrition(&mut self, who: Player) {
        let w = who as usize;
        if w >= self.players.len() {
            return;
        }
        let steps = self
            .tech_tree
            .roles
            .attrition_preq
            .iter()
            .take_while(|row| {
                row.is_some_and(|t| self.tech_tree.has_tech(&self.setup, &self.tech[w], t))
            })
            .count();
        let mods = attrition::StrengthMods {
            russian: self.nation.get(w).is_some_and(|n| n.russians),
            ..attrition::StrengthMods::default()
        };
        self.players[w].strength = attrition::strength(&self.tuning, steps, &mods);
    }

    /// **The gate both of the count's per-object writers share**
    /// (`docs/TRANSPORT.md` §16): `Unit::set_type@00612fa0`'s `+1` block
    /// on a birth and `Unit::close@0060ee50`'s `−1` at `0060f3db` on a
    /// close each move `num_units`, `control` and `active` only for a
    /// unit that is no squad follower (`is_captain`, `+0x8e`) and whose
    /// type has population (`+0x2f0 control_cost`) — the `pop == 0` arm
    /// admits `is(0x134)` and a governor-hero alone — with `+0x68 & 1`
    /// clear. So a Merchant Fleet (`POP` 1) is counted and a Transport
    /// Barge (`POP` 0) is not: run462's block 10178 holds the barge `1/78`
    /// and the fleet `1/83`, and its `num_units` has `[268]` 1 and no
    /// `[270]`.
    ///
    /// The governor-hero test is [`Sim::type_is_hero`] (item 3794).
    ///
    /// SEAM: the `pop == 0` arm's `is(0x134)` is not read here, and answers
    /// no; `+0x68 & 1` has no state here.
    pub(crate) fn counts_in_muster(&self, u: usize) -> bool {
        let unit = &self.units[u];
        unit.captain
            && unit
                .ty
                .is_some_and(|t| self.unit_types[t].price.pop != 0 || self.type_is_hero(t))
    }

    /// **`UnitData::is_gov_hero@0046ce70`** on a unit of type `ty`: the
    /// type's `unit_flags & 0x4000000` (`+0x2b4`), the hero digit `1` of its
    /// `OBJ_MASK` — the six government patriots and a scenario's heroes
    /// (`docs/GOLDEN.md` §149.11, §151.1).
    pub(crate) fn type_is_hero(&self, ty: usize) -> bool {
        self.unit_types[ty]
            .tree
            .and_then(|t| self.tech_tree.types.get(t))
            .and_then(|d| d.kind.unit())
            .is_some_and(|x| x.hero)
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
        self.track_civilian_type(who, ty, delta);
    }

    /// The live half of the census's civilian counts (item 1398,
    /// `docs/AI.md` §103): `Leader::track_unit_type@006e0dd0` moves
    /// `peasants` (`+0x978`) and `scholars` (`+0x97c`) by the type's
    /// predicate, and `Unit::set_type@00612fa0` and `Unit::close@0060ee50`
    /// move `caras` (`+0x980`), so a unit born or closed between two
    /// sweeps is already in the count `create_units` and `make_stuff` read.
    /// The sweep zeroes and recounts them, as the original's does.
    fn track_civilian_type(&mut self, who: Player, ty: usize, delta: i32) {
        let worker = self.unit_types[ty].worker;
        // The live writer uses is_caravan (including Merchant Fleets),
        // while the periodic census counts the land CARA lineage alone.
        // A sweep can drop a fleet from caras before its close subtracts
        // one. docs/AI.md §177 holds this observed asymmetry.
        let caravan = self.unit_types[ty]
            .cols
            .flag2(crate::ai_load::uflags2::FLAG2_CARAVAN);
        // `scouts` (`+0x988`) is the third arm of the same chain —
        // `role & 0x10` on a type neither peasant nor scholar — and no
        // sweep recounts it (`docs/AI.md` §219).
        let scout = self.unit_types[ty]
            .cols
            .is(crate::ai_load::role::ROLE_SCOUT);
        let census = &mut self.ai[who as usize].census;
        match worker {
            crate::orders::Worker::Citizen => census.peasants += delta,
            crate::orders::Worker::Scholar => census.scholars += delta,
            crate::orders::Worker::None if caravan => census.caras += delta,
            crate::orders::Worker::None => {}
        }
        if scout && matches!(worker, crate::orders::Worker::None) {
            census.scouts += delta;
        }
    }

    /// Gives a dead unit's supply slot back — `Unit::close` — and takes it
    /// out of both collision indices, which is `Object::remove_from_world`
    /// (`docs/COLLISION.md` §2, §3). This is every death path's tail.
    pub(crate) fn close_supply(&mut self, unit: usize) {
        self.close_supply_in(unit, false);
    }

    /// [`Sim::close_supply`], with `keep_disc` for `Object::close@00647160`'s
    /// `blocks_while_dead` arm: a combat death of a type that carries
    /// `uflags::BLOCKS_WHILE_DEAD` skips `remove_from_world`, and its
    /// occupancy disc stands (`docs/GOLDEN.md` §125).
    pub(crate) fn close_supply_in(&mut self, unit: usize, keep_disc: bool) {
        let owner = self.units[unit].owner as usize;
        if let Some(slot) = self.units[unit].supply_slot.take() {
            self.supply[owner].list.close(slot);
        }
        // `Unit::close@0060ee50`'s own: the route goes back with the unit.
        if let Some(slot) = self.units[unit].caravan.take() {
            let who = self.units[unit].owner;
            self.close_caravan(who, slot);
        }
        // `Unit::close@0060ee50`'s merchant arm: the leader's `0x2000000`
        // for any merchant, and the footprint `cast_unpack` blocked given
        // back when the trader is deployed — neither packed nor packing
        // (`docs/MERCHANT.md` §3.2).
        if self.is_merchant(unit) {
            let who = self.units[unit].owner;
            self.economy_changed(who);
            let packing = matches!(
                self.units[unit].orders.front().map(|o| &o.body),
                Some(orders::Body::Cast(c)) if orders::spell::is_pack(c.spell)
            );
            if !self.units[unit].combat.packed && !packing {
                self.merchant_footprint(unit, false);
            }
        }
        if keep_disc {
            self.units[unit].coll_at = None;
        } else {
            self.coll_remove(unit);
        }
        self.chain_remove(unit);
    }

    /// Whether anything of `owner`'s supplies a unit standing at `at`.
    ///
    /// A source's liveness is resolved here rather than stored, exactly as the
    /// original resolves it: the supply record holds an index and nothing
    /// else, so a wagon that dies or boards a transport stops supplying with
    /// no bookkeeping anywhere.
    pub fn supplied_at(&self, owner: Player, at: Pos) -> bool {
        let upgrades = self.supply_upgrade_level(owner);
        self.supply[owner as usize].supplies(&self.tuning, upgrades, at, |u| {
            self.units.get(u).map(|w| supply::Wagon {
                pos: w.pos,
                active: w.alive(),
                on_map: w.on_map,
            })
        })
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
    /// **The temple and fort border levels** are `1` plus the highest of
    /// `TEMPLEBORDERS2..4` / `FORTBORDERS2..4` held
    /// ([`tech::Roles::temple_borders_preq`]), since item 552: chapter
    /// four's Religion at 400 moved run132's border 296 → 327 and this
    /// crate's not at all while the level was a constant 1.
    ///
    /// **The three wonders are `has_wonder`'s** (item 2106): Tikal
    /// (`0x214`), the Colosseum (`0x212`) and the Eiffel Tower (`0x21c`),
    /// as [`Sim::wonders_held`] answers. Tikal read `Nation::tikal`, which
    /// nothing wrote, and the other two a constant `false`.
    ///
    /// **Still a seam**: the AI handicap allowance, which is unreachable in
    /// any game run here (`docs/ATTRITION.md`, "Territory").
    fn player_borders(&self, who: Player) -> territory::PlayerBorders {
        let w = who as usize;
        let civic = self.tech[w].epoch[tech::Line::Civic.index()].max(0);
        let roles = &self.tech_tree.roles;
        let temple_level = 1 + self.bonus_level(who, &roles.temple_borders_preq) as i32;
        let fort_level = 1 + self.bonus_level(who, &roles.fort_borders_preq) as i32;
        let n = &self.nation[w];
        let held = self.wonders_held(who);
        territory::PlayerBorders::new(
            &self.tuning,
            civic as usize,
            temple_level,
            fort_level,
            &territory::Wonders {
                colosseum: held & (1 << tech::wonder::COLOSSEUM) != 0,
                tikal: held & (1 << tech::wonder::TIKAL) != 0,
                eiffel_tower: held & (1 << tech::wonder::EIFFEL_TOWER) != 0,
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
    /// city changes hands. In play this is the *target* a fix computes, and
    /// [`crate::border_pass`] copies it onto the map at that pace (item 1106,
    /// `docs/ATTRITION.md`, "Territory").
    pub fn recompute_territory(&mut self) {
        let players = u8::try_from(self.players.len()).expect("too many players");
        let by_source = territory::compute_all_territory_won(
            &mut self.world,
            &self.tuning,
            &self.sources,
            players,
        );
        let mut city_of = vec![-1; self.sources.len()];
        for (c, city) in self.cities.iter().enumerate() {
            if let Some(si) = city.source.filter(|&si| si < city_of.len()) {
                city_of[si] = c as i32;
            }
        }
        self.territory_won = by_source
            .into_iter()
            .map(|si| usize::try_from(si).map_or(-1, |si| city_of[si]))
            .collect();
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
        // Gaia's animals: `Animal::process` is `Unit::process` without
        // a player behind it — no heal, no attrition, no reload; the
        // order step (`Animal::do_idle` when idle) and the body follow.
        //
        // `Animal::process@005d72c0` is [`anim::animal_process_steps`]: the
        // `+0x188` think (`Animal::work`), then `Guy::process` on the
        // squad and the crew, the walk handed to the body's frame.
        if self.units[i].is_gaia() {
            if self.units[i].on_map {
                anim::animal_process_steps(
                    self,
                    |s| s.work(i, frame),
                    |s| s.guy_walk_counts(i),
                    |s, walk| {
                        if s.units[i].alive() && s.units[i].on_map {
                            s.process_movement_on(i, walk);
                        }
                    },
                );
            }
            return;
        }
        // `Unit::process@00610bc0`'s head counts the reload down first
        // (`:59`–`:61`, `UnitData +0xae`), for every unit **inside or out**
        // (item 3993, `docs/GOLDEN.md` §186): a unit that garrisons inside
        // its reload comes out with the frames it spent inside spent.
        if self.units[i].combat.recharging > 0 {
            self.units[i].combat.recharging -= 1;
        }
        // `:63`–`65`: `full` and the wait, beside it.
        if self.units[i].full != 0 {
            self.units[i].full -= 1;
        }
        if self.units[i].waiting != 0 {
            self.units[i].waiting -= 1;
        }
        // `:69`–`71`: the heal's clock, beside them (item 3961).
        if self.units[i].healing != 0 {
            self.units[i].healing -= 1;
        }
        // `Unit::process@00610bc0`'s head, a caster's arm: a hero's
        // spells run out, then a point of craft back (`crate::cast`),
        // before the heal and the work.
        self.process_spells(i, frame);
        self.recover_mana(i, frame);
        // A decoy whose age has run out was closed there, and `process`
        // returns before the rest of its frame (item 1351).
        if !self.units[i].alive() {
            return;
        }
        // `process_healing` runs for every unit, inside or out: the
        // garrison branch inside, and on the map the civilian heal
        // (`crate::heal`, `docs/COMBAT.md` §72).
        // `Unit::process@00610bc0:288`–`:420`, after the attrition and
        // before the heal: a unit's own queue (item 3981).
        self.carrier_queue_tick(i);
        self.garrison_heal(i, frame);
        self.ship_heal(i, frame);
        self.iroquois_heal(i, frame);
        self.senator_heal(i, frame);
        self.ceo_heal(i, frame);
        self.supply_heal(i, frame);
        self.civilian_heal(i, frame);
        // **The frozen frame's mark is cleared here** —
        // `Unit::process@00610bc0`'s `unit_masks2 & 0xffffffef`,
        // immediately after `process_healing` and under the same
        // `inside_up < 0` guard the `targeted` decay sits in, but
        // **outside** its sixteen-frame cadence. That is what makes
        // [`combat::umask2::NOT_FIRING`] a one-frame mark: the clear runs
        // before this unit's own order step, `Unit::fight` may set it
        // again inside that step, and phase 7's `Objects::inc_time` reads
        // it at the end of the frame. `docs/COMBAT.md` §43.2.
        if self.units[i].inside.is_none() && self.units[i].inside_unit.is_none() {
            self.units[i].unit_masks2 &= !combat::umask2::NOT_FIRING;
            // `Unit::process_cloak@005e1c20`, reached every sixteenth
            // frame by `o` and gated again to every 512th: a unit with
            // `unit_masks & 0x10000` drops that bit, one without drops
            // `0x1000` — so a fight's `0x11000` takes two passes to go
            // (item 2785, `docs/GOLDEN.md` §103). SEAM: the pass's other
            // writes (`unit_masks2 & 0x60`, the Spy cloak particles, the
            // `0x800` tribe and hero arms) have no reader here.
            if (frame + i64::from(self.units[i].index)).rem_euclid(512) == 0 {
                if self.units[i].fought {
                    self.units[i].fought = false;
                } else {
                    self.units[i].blown = false;
                }
            }
        } else {
            // `Unit::process@00610bc0:557`, the `inside_up >= 0` arm:
            // `unit_masks &= 0xfffeefff` — a garrisoned unit loses both
            // the blown and the fought bits on every frame it is inside
            // (item 2798, `docs/GOLDEN.md` §104; run678's ESF `1/0`
            // enters on 979 and reads `0x840008` on 980).
            self.units[i].blown = false;
            self.units[i].fought = false;
            // The same arm's `unit_masks2 &= 0xffff7f9f` and, for a unit
            // without the hero's `0x2000`, `unit_masks &= ~0x800`: inside,
            // no cloak stands but an ambushing hero's (item 3958,
            // `docs/AI.md` §402).
            self.units[i].unit_masks2 &= !(combat::umask2::LAND_CLOAK | 0x60);
            if self.units[i].ambushing.is_none() {
                self.units[i].cloak = false;
            }
        }
        if !self.units[i].on_map {
            // **The inside arm's figure write** — `Unit::process@00610bc0`'s
            // `else`, `guy+0x70 = guy+0x14` for every figure: `last_z` is
            // put onto `z`. This crate carries a figure's altitude for a
            // plane alone ([`air::Airframe`]), which is what the landed
            // Fighter's `last_z` on run223's 723 is (item 759).
            let af = &mut self.units[i].airframe;
            af.last_z = af.z;
            return;
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
            self.process_cloak(i);
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
        if !self.units[i].alive() {
            // **A boat that stepped ashore in its own work still takes its
            // figures' `Guy::move`** (item 1191, `docs/TRANSPORT.md` §6.4).
            // `Unit::process@00610bc0` runs the `+0x188` think and then
            // `Guy::process` over the guy array with no test between them,
            // and the boat's death inside the think —
            // `Unit::set_new_location`'s sea arm, `eject_contents` then
            // `Object::die(0)` → `Unit::close` → `Object::close` — writes
            // neither the array nor its counts. So a boat standing on its
            // `des` on the walk with `stopped` set pays the arrival stand on
            // the frame it dies: East Indies' `1/56` on 8519, whose draw is
            // invisible in every dump because the boat is gone on 8520.
            //
            // **And the figure's step moves the disc after the close has
            // cleared it** (item 2088, `docs/AI.md` §208). `coll_follow`
            // skips a dead unit, and the reading here was that it could:
            // the figure on its `des`, `Guy::set_new_location` never
            // reached. run825's packet says otherwise. On East Indies'
            // tick 4204 the Freighter `1/191` disembarks; `Object::close`
            // → `remove_from_world@00647970` clears its radius-3 disc round
            // the figure at unit cell (384, 620), and then the boat's own
            // `Guy::move+0x4a5` → `Guy::set_new_location` →
            // `CollCheck::move_unit` steps the figure to (384, 621), whose
            // set pass paints the disc's new row (624) on no unit at all.
            // Those four bits walled `1/183` in thirty-four frames later.
            // `move_unit` asks nothing of the unit's life, so the disc moves
            // with the body here as it would on a live one.
            //
            // [`Sim::disembark`] is the only death this crate takes inside
            // a unit's own work; it leaves the boat dead and off the map.
            // SEAM: a unit killed in its own work with a death animation
            // (`Unit::close`'s `param_1 != 0`) has its figures handed to
            // `Objects::kill_guy` first, and this crate kills none that way
            // there. `coll_repaint` takes the boat too (item 1223): the
            // sixty-fourth-frame repaint re-marks the disc the close cleared.
            if self.units[i].kind.domain == attrition::Domain::Sea && !self.units[i].on_map {
                let from = self.units[i].movement.body.pos;
                self.process_movement(i);
                let to = self.units[i].movement.body.pos;
                self.coll_move(i, from, to);
            }
            return;
        }
        if !self.units[i].on_map {
            // `Unit::process` runs `Guy::process` for each figure after
            // `work` **whether or not the work put the unit inside**
            // (`00610bc0`, the `+0x188` call and the guy loops after it).
            // A plane that landed this frame is standing on its `des`, so
            // its figure takes `Guy::move`'s standing arm once — the
            // `last_speed 0`, `stopped 1` of run223's 722 — and never
            // again, the inside arm writing only `last_z` from then on.
            //
            // **And so is every unit that went inside in its own work**,
            // not an aircraft alone (item 850, `docs/ANIM.md` §15): a
            // walker that casts its barge at the shore is cargo by the
            // work's end, and its figures still take the standing arm —
            // East Indies' Merchant `1/77` on 19413, whose second figure
            // turns to its `des_angle` and pays the turn's draw.
            //
            // SEAM: a squad that garrisons in its own `work` takes the
            // same frame by the reading; no widening names its figures on
            // that frame yet.
            if self.units[i].inside.is_some() || self.units[i].inside_unit.is_some() {
                self.process_movement(i);
            }
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

        if sheltered {
            self.units[i].sheltered = true;
            return None;
        }
        Some(self.suffer_attrition(i, frame))
    }

    /// **`Unit::suffer_attrition@005e1a10`**: one tick's damage, for the
    /// period's bleed above and a detected decoy's
    /// ([`Sim::recover_mana`]). Its argument gates the graphic alone.
    pub(crate) fn suffer_attrition(&mut self, i: usize, frame: i64) -> Tick {
        // The walk is [`suffer_view::suffer_attrition_steps`], the swept one
        // (item 3372): the damage is sixteenths of a hit point per figure,
        // carried through a fractional accumulator the way
        // `Object::take_damage` carries it (a lone figure loses one whole
        // point a tick, a figure in a squad of four one every fourth tick),
        // **and the squad is counted, not stored** (item 552): the view asks
        // `curr_uber_size`, which walks up to the captain and counts down the
        // chain, and a figure falls at its share of the squad's hit points
        // (`docs/COMBAT.md` §7.2 step 8, `docs/GOLDEN.md` §135, run936).
        let mut view = suffer_view::SimSuffer {
            sim: self,
            unit: i,
            outcome: None,
        };
        let killed = suffer_view::suffer_attrition_steps(&mut view, false);
        let (sixteenths, lost, _) = view.outcome.expect("the hurt call is always made");
        if killed {
            // `Object::take_damage`'s `die` with `dtype` 1 — the whole combat
            // tail, `Unit::close`'s draw and orders and the thirty-frame hold
            // on the number (`docs/COMBAT.md` §59.3, `docs/GOLDEN.md` §135).
            self.die_by_attrition(i, frame);
        }
        Tick {
            unit: i,
            frame,
            sixteenths,
            lost,
            killed,
        }
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
        let (guy_mark, squad, len) = self.guy_walk_counts(i);
        let walk: Vec<usize> = anim::process_walk(guy_mark, squad, len).collect();
        self.process_movement_on(i, &walk);
    }

    /// [`Sim::process_movement`] on the figures `Guy::process` runs on, in
    /// [`anim::process_walk`]'s order.
    fn process_movement_on(&mut self, i: usize, walk: &[usize]) {
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
        // `Guy::move:109`'s `turn_towards(des_angle, …, 1)`, which hands
        // `Guy::do_turn` the same override `move_step`'s two turn-in-place
        // arms do — so a standing guy owed a turn asks for its turn
        // animation here as well, and pays the idle roll if it has none
        // (`docs/ANIM.md` §4.8). The gate is the original's own: `guy_flags
        // & 2` clear, which is `turned`, and the body on its unit. It runs
        // inside guy 0's turn in `guys_follow`, before the crew's own
        // `Guy::move` (item 1429).
        let mut turn =
            (was_at_des && !turned && !facing_settled).then_some((facing, follow.facing, heading));
        // **Where a crew guy is told to be**, written before the figures
        // run because `Guy::move` writes it in guy 0's own step and
        // `Guy::process` runs the figures in slot order: a tracked figure
        // later in the array than guy 0 finds its destination already
        // rewritten (item 1510). The write is the one below, moved up; it
        // reads the unit's position and the frame's new facing, neither of
        // which a figure's animation touches.
        if !was_at_des || !facing_settled {
            let pos = self.units[i].pos;
            self.crew_des(i, pos, follow.facing, false);
        }
        self.guys_follow_on(i, walk, was_at_des, &mut turn);
        // A helicopter's figure height (item 1048): `Guy::move:47`'s
        // `last_z = z` at its head, and the moving arm's
        // `Guy::set_new_location(·, 0)` (`:190`) climbs it at the body's
        // new point ([`Sim::helicopter_climb`]). SEAM: `:46`'s
        // `guy_flags & 0x40` exemption, and the write for other figures.
        if self.is_helicopter(i) {
            self.units[i].airframe.last_z = self.units[i].airframe.z;
            if !was_at_des {
                self.helicopter_climb(i, follow.body.pos);
            }
        }
        // A unit with no figure to walk keeps the turn's mark where it was.
        if let Some((was, to, heading)) = turn {
            self.mark(anim::SITE_TURN_STAND);
            self.do_turn_anim(i, was, to, heading);
        }
        let unit = &mut self.units[i];
        unit.movement.facing = follow.facing;
        // **`guy_flags & 2` is cleared here, after `Guy::move`**
        // (`Guy::process@005e0230:28`), not at the head of the unit's
        // process. `Guy::do_turn@005d97a0` sets it on any turn that moves
        // the angle, whoever calls it, so a push turned by a unit processed
        // earlier in the frame keeps the pushed body from taking its own
        // turn in `Guy::move:108` this frame, and one turned later in the
        // frame keeps it from the next. East Indies' General `1/138` on
        // 4336: the Catapult `1/124` pushes it one half-rate step before
        // its own process, and the body turns no further (item 3571,
        // `docs/AI.md` §310).
        unit.movement.frame_facing = follow.facing;
        // Guy 0's `Guy::move` has turned the crew with it, or there was
        // nothing to turn ([`Movement::crew_des`]).
        unit.movement.crew_des = None;
        unit.movement.body = follow.body;
        // `Guy::move`'s `Guy::set_new_location` is what moves the
        // collision disc (`CollCheck::move_unit`, its only caller): the
        // bits follow guy 0 here, not the unit's own step
        // (`docs/COLLISION.md` §16).
        self.coll_follow(i);
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
        // are one expression here — written above, ahead of the figures
        // ([`Sim::guys_follow_on`] runs the tracked ones in their slots).
        // `Guy::process@005e0230:30–56`, right after `Guy::move`: a pivot
        // figure (`guy_flags & 0x100`, its type has `<RESTRICTION>` rows)
        // turns each turret toward where `set_all_pivots` last told it,
        // 15° a frame, snapping inside 15°, and sets the node's bit once
        // there (`docs/COMBAT.md` §55.3).
        if self.units[i]
            .ty
            .and_then(|t| self.art.pivots.get(&self.unit_types[t].type_index))
            .is_some_and(|n| !n.is_empty())
        {
            for g in &mut self.units[i].guys {
                g.turret.step();
            }
        }
        // `Guy::process`'s own tail: the sixty-fourth-frame repaint of the
        // collision block (`docs/COLLISION.md` §2.2). It reads the body as
        // `Guy::move` has just left it, which for guy 0 is the unit's own
        // position.
        self.coll_repaint(i);
        // And the same block's last line, `Guy::process@005e0230:120`:
        // a standing guy on its sixty-fourth frame forgets the in-danger
        // mark `Unit::set_in_danger` gave it (`docs/ORDERS.md` §22).
        // The flag is guy 0's (`set_in_danger` sets no other), so guy
        // 0's gate is the whole clear: a crew figure has none to drop.
        if collide::repaint_due(
            self.frame,
            self.units[i].index,
            self.units[i].movement.body.avg_speed,
        ) {
            self.units[i].guy_flag_0x20 = false;
        }
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
    /// (`docs/COLLISION.md` §6 step 6) — and `Unit::set_angle(…, 1)`,
    /// which `Unit::do_cast`'s turn to its target is (item 3920,
    /// [`Sim::unit_set_angle_as`]). Every other `Unit::set_angle` call this
    /// crate makes, and `Guy::move`'s, pass zero.
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
        // `Guy::move:203` — `GuyData::get_speed`, which is the unit's
        // `get_speed(x, y, 1)` asked at **this figure's** point: the
        // order scale and the slow ground, never the group cap
        // (`docs/MOVEMENT.md`, "The crew's speed"). The type's base
        // speed stepped a guarding computer's crew a quarter slow.
        // Its `+9` for a squad figure on a group order is
        // [`orders::guy_speed_steps`]'s; `g` is this figure's `guy_num`.
        let speed = orders::guy_speed_steps(
            &mut (),
            g as i32,
            |_| self.get_speed_at(i, f.body.pos, 1),
            |_| anim::SQUAD_SIZE as i32,
            |_| !self.units[i].orders.is_empty(),
            |_| self.current_order(i).is_some(),
            |_| {
                matches!(
                    self.order_type(i),
                    orders::index::GROUP_MOVE
                        | orders::index::GROUP_ATTACK_TO
                        | orders::index::GROUP_PATROL
                )
            },
        );
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

/// `UnitData::curr_uber_size@0060a760`'s two walks: up the `o_up` links to
/// the captain (`up` answers `None` there — the original's `is_captain`,
/// `o_up`'s sign bit), then the captain and each `o_down` link whose figure
/// is alive (`down` answers `None` past the last), counted. Both walks stop
/// after sixteen links, a guard the original does not have. Item 2247's
/// sweep holds it.
pub(crate) fn squad_size(
    mut i: usize,
    up: impl Fn(usize) -> Option<usize>,
    down: impl Fn(usize) -> Option<usize>,
) -> i32 {
    let mut guard = 0;
    while let Some(u) = up(i) {
        i = u;
        guard += 1;
        if guard > 16 {
            break;
        }
    }
    let (mut n, mut cur, mut guard) = (1, down(i), 0);
    while let Some(d) = cur {
        n += 1;
        cur = down(d);
        guard += 1;
        if guard > 16 {
            break;
        }
    }
    n
}

/// The figures a squad walk reaches, in its order — **`Unit::set_in_danger@005fcfb0`**
/// and **`Unit::set_group@00605220`** walk alike: the climb up `up` (the
/// original's `o_up`, `None` at the captain — its `is_captain`, the link's
/// sign bit) from `start`, then the captain and each `down` link (the
/// original's `o_down` whose slot is active, `flags & 1`; `None` past the
/// last). Both walks stop after `limit` links, a guard the original does
/// not have: a cycle would hang it. Item 2276's sweeps hold it.
pub(crate) fn squad_walk(
    start: usize,
    limit: usize,
    up: impl Fn(usize) -> Option<usize>,
    down: impl Fn(usize) -> Option<usize>,
) -> Vec<usize> {
    let mut at = start;
    for _ in 0..limit {
        let Some(u) = up(at) else { break };
        at = u;
    }
    let mut out = vec![at];
    for _ in 0..limit {
        let Some(d) = down(at) else { break };
        out.push(d);
        at = d;
    }
    out
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

// The original's functions under the emulator, one module a function: the
// census's backed column (item 1575). Independent sweeps are Cargo integration
// targets under tests/; this module keeps sweeps that need crate-private APIs.
#[cfg(test)]
mod sweep;

/// The install-backed tests' one question: where is the game.
#[cfg(test)]
#[path = "test_support/install.rs"]
pub(crate) mod testenv;
