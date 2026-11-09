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
    /// The load-time tables: the unit, building and tech types, the lobby
    /// setup and the tuning (`core::Tables`).
    pub tables: crate::core::Tables,
    pub world: World,
    /// The `territory/` subsystem's state (`docs/STRUCTURE.md` §3).
    pub territory: crate::territory::TerritoryState,
    pub units: Vec<Unit>,
    /// The `leaders/` subsystem's state (`docs/STRUCTURE.md` §3).
    pub leaders: crate::leaders::LeadersState,
    /// The `economy/` subsystem's state (`docs/STRUCTURE.md` §3).
    pub economy: crate::economy::EconomyState,
    /// The `units/` subsystem's state (`docs/STRUCTURE.md` §3).
    pub roster: crate::units::UnitsState,
    /// The production buildings, each with its own queue.
    pub buildings: Vec<Building>,
    /// The `cities/` subsystem's state (`docs/STRUCTURE.md` §3).
    pub cities_state: crate::cities::CitiesState,
    /// One per player: the tech bits and counters — `LeaderData::tech`,
    /// `ages`, `epochs`, `epoch[4]`.
    pub tech: Vec<tech::PlayerTech>,
    /// The `combat/` subsystem's state (`docs/STRUCTURE.md` §3).
    pub combat: crate::combat::CombatState,
    /// The `orders/` subsystem's state (`docs/STRUCTURE.md` §3).
    pub orders: crate::orders::OrdersState,
    /// The game's random stream, `game_random`. Combat draws from it for
    /// projectile scatter and the one-in-five retarget roll.
    pub rng: combat::Rng,
    /// Draw marks spent by **staged input** at `Game::do_frame`'s entry,
    /// before the frame's first phase — the cheat channel's, which
    /// `rontrace.dll` hands to `ConsoleWin::parse_cmd` there
    /// (`docs/INPUT.md` §11). [`Sim::tick`] clears [`Sim::phase_marks`] at
    /// its head, so a staged draw marked before the call would vanish from
    /// the fold while still moving the stream; these are moved to the head
    /// of the frame's own marks instead, which is where the original
    /// spends them. `crate::golden::Script::stage` is the only writer.
    pub staged_marks: Vec<(String, u32)>,
    /// The `ai/` subsystem's state (`docs/STRUCTURE.md` §3).
    pub ai_state: crate::ai::AiState,
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
    /// The `movement/` subsystem's state (`docs/STRUCTURE.md` §3).
    pub movement: crate::movement::MovementState,
    /// The stream's word at each phase boundary of [`Sim::tick`], filled
    /// only while [`Sim::trace_phases`] is set. It is what
    /// `tools/trace/report.py sites` is for the original — a frame's draws
    /// attributed, here to a phase and a unit rather than a return address
    /// — and comparing the two folds is how the frame-0 gap was taken
    /// apart (`docs/SYNC.md` §4.2).
    pub phase_marks: Vec<(String, u32)>,
    /// How many [`Sim::ground_z`] reads have touched a height corner that
    /// is not known to be the original's own single — one whose six-decimal
    /// print a neighbouring single shares, or one a terraform rewrote in
    /// millionths (`docs/COMBAT.md` §46.3). Every window the harness replays
    /// pins it, so the day a read lands on one is a failing test by name.
    pub ground_inexact: u32,
    /// `LeaderData::flock_stamp` (`+0x7b8`): the frame of the player's
    /// last flock of birds, which `Objects::add_flock` reads to allow one
    /// in 225 frames per player (`crate::flock`). The leader dump prints
    /// it.
    pub flock_stamp: Vec<i64>,
    /// Buildings disbanded or died this frame, for tests.
    pub removed: Vec<usize>,
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
    /// The herds and the birds' sampling (`gaia.rs`).
    pub gaia: gaia::Gaia,
    /// The `world/` subsystem's state (`docs/STRUCTURE.md` §3).
    pub terrain: crate::world::WorldState,
    /// `GameInfo::seed`, the lobby's map seed, which picks a gaia guy's
    /// piece by `(seed + o) % 3`.
    pub game_seed: i32,
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
    /// The order a frame's building walk ran its five steps in — written
    /// only under test — for the test that holds `Build::process`'s sequence (group 18).
    #[allow(dead_code)]
    pub(crate) building_log: Vec<(usize, &'static str)>,
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
            tech: (0..players)
                .map(|_| tech::PlayerTech::new(&tech_tree))
                .collect(),
            staged_marks: Vec::new(),
            trace_phases: false,
            phase_marks: Vec::new(),
            trace_costs: false,
            cost_marks: Vec::new(),
            road_marks: Vec::new(),
            units: Vec::new(),
            buildings: Vec::new(),
            orders: crate::orders::OrdersState {
                pushed: Vec::new(),
                last_group: [0; 16],
                set_aside: None,
                spells: Vec::new(),
            },
            rng: combat::Rng::new(0),
            ground_inexact: 0,
            economy: crate::economy::EconomyState {
                caravans: vec![caravan::Caravans::default(); players],
                holdings: vec![economy::Holdings::new(); players],
                ledgers: vec![economy::Ledger::starting(&tuning); players],
                good_types: Vec::new(),
                redirects: cost::Redirects::RON,
                market: market::Market::default(),
            },
            gaia: gaia::Gaia::default(),
            game_seed: 0,
            flock_stamp: vec![0; players],
            leaders: crate::leaders::LeadersState {
                at_war: vec![vec![false; players]; players],
                allied: vec![vec![false; players]; players],
                treaties: vec![vec![0; players]; players],
                agendas: vec![vec![0; players]; players],
                forced_march: vec![false; players],
                defeated: vec![false; players],
            },
            removed: Vec::new(),
            cities_state: crate::cities::CitiesState {
                farm_order: Vec::new(),
                cities: Vec::new(),
                nation: vec![city::Nation::default(); players],
                lost_city_stamp: vec![None; players],
                city_tally: vec![city::Tally::default(); players],
                building_high: vec![Vec::new(); players],
                wall_stats_dirty: vec![false; players],
                wonders_built: 0,
            },
            territory: crate::territory::TerritoryState {
                players: vec![attrition::PlayerState::default(); players],
                sources: Vec::new(),
                supply: vec![supply::Network::default(); players],
                borders: vec![territory::PlayerBorders::plain(&tuning); players],
                borders_fixed: false,
                border_pass: None,
                territory_won: Vec::new(),
                good_seen_bits: Vec::new(),
            },
            goody_items: Vec::new(),
            in_play: false,
            roster: crate::units::UnitsState {
                unit_stats_dirty: vec![false; players],
                muster: vec![Muster::new(&tuning); players],
                art: anim::Art::default(),
                // Ten slots, not `players`: gaia's animals and birds are units
                // of owners 8 and 9 and take numbers from their own bands.
                marks: vec![Marks::default(); players.max(10)],
                leader_options: vec![stance::LeaderOptions::default(); players.max(10)],
            },
            ai_state: crate::ai::AiState {
                transport: vec![transport::LeaderTransport::default(); players],
                docks: vec![transport::Docks::default(); players],
                armies: (0..players)
                    .map(|w| army::Armies::new(w as Player))
                    .collect(),
                ai_off: false,
                lobby: ai::Lobby::default(),
                script_env: ai_host::ScriptEnv::default(),
                scripts: None,
                ai: vec![ai::Leader::new(); players],
                ai_speed: 1,
            },
            sweep_watch: None,
            probe_refuse: None,
            terrain: crate::world::WorldState {
                mountains: Vec::new(),
                cliffs: Vec::new(),
                chain_heads: vec![None; (world.width() * world.height()) as usize],
            },
            movement: crate::movement::MovementState {
                mesh,
                plan_roads: true,
                coll: collide::CollGrid::new(world.width(), world.height()),
                path_memo: std::collections::BTreeMap::new(),
                coll_copies: std::cell::RefCell::default(),
                repaths: vec![0; players.max(10)],
                pf_start: Pos::default(),
            },
            building_log: Vec::new(),
            combat: crate::combat::CombatState {
                table: combat::Table::uniform(0),
                projectiles: Vec::new(),
                nukes: nuke::Nukes::default(),
                deaths: Vec::new(),
                death_pool: Vec::new(),
                mods: vec![combat::Modifiers::default(); players],
                hits: Vec::new(),
                retargets: vec![0; players.max(10)],
                second_range: None,
            },
            tables: crate::core::Tables {
                tech_tree,
                setup: tech::Setup::STANDARD,
                unit_types: Vec::new(),
                build_types: Vec::new(),
                tuning,
            },
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
        let who = self.territory.players.len();
        self.territory
            .players
            .push(attrition::PlayerState::default());
        self.tech
            .push(tech::PlayerTech::new(&self.tables.tech_tree));
        self.territory.supply.push(supply::Network::default());
        self.economy.holdings.push(economy::Holdings::new());
        self.roster.unit_stats_dirty.push(false);
        self.economy
            .ledgers
            .push(economy::Ledger::starting(&self.tables.tuning));
        self.roster.muster.push(Muster::new(&self.tables.tuning));
        self.combat.mods.push(combat::Modifiers::default());
        self.cities_state.nation.push(city::Nation::default());
        self.territory
            .borders
            .push(territory::PlayerBorders::plain(&self.tables.tuning));
        self.leaders.defeated.push(false);
        self.cities_state.lost_city_stamp.push(None);
        self.cities_state.city_tally.push(city::Tally::default());
        self.cities_state.building_high.push(Vec::new());
        self.cities_state.wall_stats_dirty.push(false);
        self.roster.marks.push(Marks::default());
        self.movement.repaths.push(0);
        self.combat.retargets.push(0);
        self.ai_state.ai.push(ai::Leader::new());
        self.ai_state
            .transport
            .push(transport::LeaderTransport::default());
        self.ai_state.docks.push(transport::Docks::default());
        self.ai_state.armies.push(army::Armies::new(
            u8::try_from(who).expect("too many players"),
        ));
        for row in &mut self.leaders.at_war {
            row.push(false);
        }
        self.leaders.at_war.push(vec![false; who + 1]);
        for row in &mut self.leaders.allied {
            row.push(false);
        }
        self.leaders.allied.push(vec![false; who + 1]);
        for row in &mut self.leaders.treaties {
            row.push(0);
        }
        self.leaders.treaties.push(vec![0; who + 1]);
        for row in &mut self.leaders.agendas {
            row.push(0);
        }
        self.leaders.agendas.push(vec![0; who + 1]);
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
        self.economy.ledgers[who as usize].dirty = true;
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
