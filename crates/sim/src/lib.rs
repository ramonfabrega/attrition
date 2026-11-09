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
pub use units::unit::{
    BUILD_BASE, DEFAULT_POP_LIMIT, HeadSerial, Marks, Movement, Muster, Refused, UNIT_BASE, Unit,
    UnitType, WALL_BASE, init_form, turning_of,
};
pub(crate) use units::unit::{find_free_reusable, find_free_steps, squad_size, squad_walk};
pub use world::{Cell, Owner, Player, Pos, Terrain, World};

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
