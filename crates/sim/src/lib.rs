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
pub mod air;
pub mod checksum;
pub mod cities;
pub mod groups;
pub mod ids;
pub mod setup;
pub mod spells;
pub mod units;
// `combat::build_family` is private and named through `combat` alone (item 4012).
pub mod combat;
pub mod command;
pub mod core;
pub mod data;
pub mod economy;
pub mod gaia;
pub mod leaders;
pub mod mapgen;
pub mod movement;
pub mod num;
pub mod observe;
pub mod orders;
use self::units::set_type_adapter;
pub mod tech;
pub mod territory;
pub mod trace;
pub mod vision;
pub mod world;

pub use self::core::frame::{
    ATTRITION_REFRESH_FRAMES, FRAMES_PER_SECOND, TARGETED_DECAY_FRAMES, Tick, UNIT_UPKEEP_FRAMES,
};
pub use self::core::tuning::Tuning;
pub use cities::building::{Building, Produced};
#[cfg(test)]
use cities::queue::Advanced;
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
    /// The frame count: how many ticks have run.
    pub frame: i64,
    /// Whether a frame has run. A border fix before the first is setup's —
    /// `World::compute_all_territory` with the budget unlimited, on the
    /// map at once — and one after it takes the daemon's pass
    /// ([`border_pass`], item 1106).
    pub in_play: bool,
    /// `GameInfo::seed`, the lobby's map seed, which picks a gaia guy's
    /// piece by `(seed + o) % 3`.
    pub game_seed: i32,
    /// The game's random stream, `game_random`. Combat draws from it for
    /// projectile scatter and the one-in-five retarget roll.
    pub rng: combat::Rng,
    // The object store, which every subsystem reads and writes.
    pub world: World,
    pub units: Vec<Unit>,
    /// The production buildings, each with its own queue.
    pub buildings: Vec<Building>,
    // One state per subsystem directory (`docs/STRUCTURE.md` §3, item 4016).
    /// The load-time tables: the unit, building and tech types, the lobby
    /// setup and the tuning (`core::Tables`).
    pub tables: crate::core::Tables,
    /// The `world/` subsystem's state (`docs/STRUCTURE.md` §3).
    pub terrain: crate::world::WorldState,
    /// The `territory/` subsystem's state (`docs/STRUCTURE.md` §3).
    pub territory: crate::territory::TerritoryState,
    /// The `units/` subsystem's state (`docs/STRUCTURE.md` §3).
    pub roster: crate::units::UnitsState,
    /// The `movement/` subsystem's state (`docs/STRUCTURE.md` §3).
    pub movement: crate::movement::MovementState,
    /// The `orders/` subsystem's state (`docs/STRUCTURE.md` §3).
    pub orders: crate::orders::OrdersState,
    /// The `combat/` subsystem's state (`docs/STRUCTURE.md` §3).
    pub combat: crate::combat::CombatState,
    /// The `cities/` subsystem's state (`docs/STRUCTURE.md` §3).
    pub cities: crate::cities::CitiesState,
    /// The `economy/` subsystem's state (`docs/STRUCTURE.md` §3).
    pub economy: crate::economy::EconomyState,
    /// The `leaders/` subsystem's state (`docs/STRUCTURE.md` §3).
    pub leaders: crate::leaders::LeadersState,
    /// One per player: the tech bits and counters — `LeaderData::tech`,
    /// `ages`, `epochs`, `epoch[4]`.
    pub tech: Vec<tech::PlayerTech>,
    /// The `ai/` subsystem's state (`docs/STRUCTURE.md` §3).
    pub ai: crate::ai::AiState,
    /// The `gaia/` subsystem's state (`docs/STRUCTURE.md` §3).
    pub gaia: crate::gaia::GaiaState,
    /// The `trace/` subsystem's state (`docs/STRUCTURE.md` §3).
    pub trace: crate::trace::TraceState,
}

pub use self::cities::farms::Farm;

impl Sim {
    pub fn new(tuning: Tuning, world: World, players: usize) -> Sim {
        let tech_tree = tech::TechTree::new().with_tuning(&tuning);
        // `Regions::set_coastals`: the coast masks follow from the cells.
        let mut world = world;
        world.rebuild_coasts();
        let mut mesh = crate::movement::mesh::RoadMesh::default();
        mesh.seed(&world);
        Sim {
            tech: (0..players)
                .map(|_| tech::PlayerTech::new(&tech_tree))
                .collect(),
            units: Vec::new(),
            buildings: Vec::new(),
            orders: crate::orders::OrdersState {
                pushed: Vec::new(),
                last_group: [0; 16],
                set_aside: None,
                spells: Vec::new(),
            },
            rng: combat::Rng::new(0),
            economy: crate::economy::EconomyState {
                caravans: vec![crate::economy::caravan::Caravans::default(); players],
                holdings: vec![economy::Holdings::new(); players],
                ledgers: vec![economy::Ledger::starting(&tuning); players],
                good_types: Vec::new(),
                redirects: crate::economy::cost::Redirects::RON,
                market: crate::economy::market::Market::default(),
            },
            game_seed: 0,
            leaders: crate::leaders::LeadersState {
                at_war: vec![vec![false; players]; players],
                allied: vec![vec![false; players]; players],
                treaties: vec![vec![0; players]; players],
                agendas: vec![vec![0; players]; players],
                forced_march: vec![false; players],
                defeated: vec![false; players],
                city_name: vec![0; players],
                anti_att: vec![crate::leaders::words::ANTI_ATT_BASE; players],
                pop_issues: vec![0; players],
                marks: vec![crate::leaders::marks::Marks::default(); players],
                tallies: vec![crate::leaders::tallies::Tallies::default(); players],
                rates: vec![crate::leaders::rates::Rates::default(); players],
                endgame: vec![crate::leaders::endgame::EndGame::default(); players],
                finished: vec![crate::leaders::finished::Finished::default(); players],
                score: vec![crate::leaders::score::Score::default(); players],
                victory_checked: false,
                game_over: false,
            },
            cities: crate::cities::CitiesState {
                walks_setup: false,
                farm_order: Vec::new(),
                cities: Vec::new(),
                nation: vec![crate::cities::city::Nation::default(); players],
                lost_city_stamp: vec![None; players],
                capturing: None,
                city_tally: vec![crate::cities::city::Tally::default(); players],
                building_high: vec![Vec::new(); players],
                wall_stats_dirty: vec![false; players],
                wonders_built: 0,
            },
            territory: crate::territory::TerritoryState {
                players: vec![crate::territory::attrition::PlayerState::default(); players],
                sources: Vec::new(),
                supply: vec![crate::territory::supply::Network::default(); players],
                borders: vec![territory::PlayerBorders::plain(&tuning); players],
                borders_fixed: false,
                border_pass: None,
                territory_won: Vec::new(),
                good_seen_bits: Vec::new(),
                reg_terr: vec![Vec::new(); players],
                territory_high: vec![0; players],
            },
            gaia: crate::gaia::GaiaState {
                gaia: gaia::Gaia::default(),
                flock_stamp: vec![0; players],
                goody_items: Vec::new(),
            },
            in_play: false,
            roster: crate::units::UnitsState {
                unit_stats_dirty: vec![false; players],
                muster: vec![Muster::new(&tuning); players],
                art: crate::units::anim::Art::default(),
                // Ten slots, not `players`: gaia's animals and birds are units
                // of owners 8 and 9 and take numbers from their own bands.
                marks: vec![Marks::default(); players.max(10)],
                leader_options: vec![
                    crate::units::stance::LeaderOptions::default();
                    players.max(10)
                ],
                specials: vec![crate::units::slots::SlotList::default(); players],
                heroes: vec![crate::units::slots::SlotList::default(); players],
                hold_doobers: crate::units::slots::HoldDoobers::default(),
            },
            ai: crate::ai::AiState {
                transport: vec![crate::ai::transport::LeaderTransport::default(); players],
                docks: vec![crate::ai::transport::Docks::default(); players],
                armies: (0..players)
                    .map(|w| crate::ai::army::Armies::new(w as Player))
                    .collect(),
                ai_off: false,
                lobby: ai::Lobby::default(),
                script_env: crate::ai::ai_host::ScriptEnv::default(),
                scripts: None,
                ai: vec![ai::Leader::new(); players],
                ai_speed: 1,
            },
            terrain: crate::world::WorldState {
                mountains: Vec::new(),
                cliffs: Vec::new(),
                chain_heads: vec![None; (world.width() * world.height()) as usize],
                object_heads: vec![None; (world.width() * world.height()) as usize],
            },
            movement: crate::movement::MovementState {
                mesh,
                plan_roads: true,
                coll: crate::movement::collide::CollGrid::new(world.width(), world.height()),
                path_memo: std::collections::BTreeMap::new(),
                coll_copies: std::cell::RefCell::default(),
                repaths: vec![0; players.max(10)],
                pf_start: Pos::default(),
            },
            trace: crate::trace::TraceState {
                staged_marks: Vec::new(),
                trace_phases: false,
                phase_marks: Vec::new(),
                trace_costs: false,
                cost_marks: Vec::new(),
                road_marks: Vec::new(),
                ground_inexact: 0,
                removed: Vec::new(),
                sweep_watch: None,
                probe_refuse: None,
                building_log: Vec::new(),
            },
            combat: crate::combat::CombatState {
                table: combat::Table::uniform(0),
                projectiles: Vec::new(),
                nukes: crate::combat::nuke::Nukes::default(),
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
            .push(crate::territory::attrition::PlayerState::default());
        self.tech
            .push(tech::PlayerTech::new(&self.tables.tech_tree));
        self.territory
            .supply
            .push(crate::territory::supply::Network::default());
        self.economy.holdings.push(economy::Holdings::new());
        self.roster.unit_stats_dirty.push(false);
        self.economy
            .ledgers
            .push(economy::Ledger::starting(&self.tables.tuning));
        self.roster.muster.push(Muster::new(&self.tables.tuning));
        self.combat.mods.push(combat::Modifiers::default());
        self.cities
            .nation
            .push(crate::cities::city::Nation::default());
        self.territory
            .borders
            .push(territory::PlayerBorders::plain(&self.tables.tuning));
        self.territory.reg_terr.push(Vec::new());
        self.territory.territory_high.push(0);
        self.leaders.defeated.push(false);
        self.leaders
            .endgame
            .push(crate::leaders::endgame::EndGame::default());
        self.leaders
            .finished
            .push(crate::leaders::finished::Finished::default());
        self.leaders
            .score
            .push(crate::leaders::score::Score::default());
        self.cities.lost_city_stamp.push(None);
        self.cities
            .city_tally
            .push(crate::cities::city::Tally::default());
        self.cities.building_high.push(Vec::new());
        self.cities.wall_stats_dirty.push(false);
        self.roster.marks.push(Marks::default());
        self.movement.repaths.push(0);
        self.combat.retargets.push(0);
        self.ai.ai.push(ai::Leader::new());
        self.ai
            .transport
            .push(crate::ai::transport::LeaderTransport::default());
        self.ai.docks.push(crate::ai::transport::Docks::default());
        self.ai.armies.push(crate::ai::army::Armies::new(
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
