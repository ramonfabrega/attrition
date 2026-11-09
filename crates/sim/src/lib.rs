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
