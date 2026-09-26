//! The script host — the 55 `ScenarioFuncSet` functions the opening scripts
//! call, over the simulation.
//!
//! `docs/AI.md` §3 and §12.1; the specification is
//! `~/ghidra-projects/reports/ai/host-functions.md` (ratified in §11), read
//! function by function. Every entry there is one arm of [`ScriptHost::call`]
//! here, and the conventions the report establishes are the module's:
//!
//! - **`who` is 1-based**, and every function that takes it fails with `-1`
//!   for a slot that is not a live leader (`leader_flags & 1|2`;
//!   `num_cities` tests bit 1 only).
//! - **A type argument is a display name**, matched case-insensitively
//!   against the tree in `TypeIndex` order — first match wins, so a name two
//!   records share resolves to the lower one. `-1` when nothing matches.
//! - **Object handles are the per-player numbers** [`Sim::find_free`] hands
//!   out: units in `[0, 2000)`, buildings in `[2000, 3000)`.
//! - **The finders rotate.** `ScenarioData::find_counters[]` is one global
//!   array of resume cursors, kept in [`ScriptEnv`]; successive calls return
//!   different objects, and the scripts rely on it.
//! - **Timers are in ticks**, one tick per fifteen frames, keyed by the
//!   decimal spelling of the player number the scripts pass.
//! - **`rand_int` is the sync stream** — `Random::get(game_random, lo, hi)`,
//!   upper bound exclusive.
//!
//! What lands on the producers — `place_*` on `Leader::produce_building`
//! and `place_city_with_cost` on `compute_sites` + `found_cities` — is the
//! seam [`Sim::produce_building`] / [`Sim::place_city_ai`]; see their notes.

use crate::bhs::{self, Host, Program, RunError, Source, State, Ty, Value};
use crate::orders::{Body, MoveKind, QueuePos, index};
use crate::tech::{Kind, TypeId};
use crate::world::{Pos, UNITS_PER_TILE};
use crate::{BUILD_BASE, Player, Sim, UNIT_BASE, cost, tech};

/// `MathUtilFuncSet::rand_int@009e1890+0x18` — the one host function that
/// steps the sync stream, and the only draw site in the whole script VM.
/// The trace names it under `ScriptFuncSet::call_func+0x401`, the
/// interpreter's own call-out; on run14 every one of its eight draws is
/// frame 1's `rand_int(1, 10)` in the AI's opening strategy pick
/// (`docs/AI.md` §12.1, `docs/SYNC.md` §3.10).
pub const SITE_RAND_INT: &str = "MathUtilFuncSet::rand_int+0x18";

/// `ScenarioData::find_counters[]` — as many as the highest index the
/// scripts reach, and the timers. Sim state: digested by the soak.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptEnv {
    pub find_counters: [i32; 32],
    /// `ScriptTimers`: `(key, expiry tick)` in insertion-ordered order.
    pub timers: Vec<(String, i64)>,
}

/// The cursor indices, from the report's "global cursors, collected".
pub mod counter {
    pub const FIND_UNIT: usize = 1;
    pub const FIND_BUILD: usize = 2;
    pub const FIND_IDLE_CITIZEN: usize = 4;
    pub const FIND_BUILD_AT_CITY: usize = 0x16;
    pub const FIND_BUILD_AT_CITY_WHO: usize = 0x17;
    pub const FIND_INACTIVE_BUILD: usize = 0x18;
    pub const TRAIN_PRODUCER: usize = 0x1c;
    pub const RESEARCH_PRODUCER: usize = 0x1e;
}

impl Default for ScriptEnv {
    fn default() -> ScriptEnv {
        let mut c = [0; 32];
        c[counter::FIND_BUILD] = i32::from(BUILD_BASE);
        c[counter::FIND_BUILD_AT_CITY] = -1;
        c[counter::FIND_BUILD_AT_CITY_WHO] = -1;
        c[counter::FIND_INACTIVE_BUILD] = i32::from(BUILD_BASE);
        c[counter::TRAIN_PRODUCER] = i32::from(BUILD_BASE);
        c[counter::RESEARCH_PRODUCER] = i32::from(BUILD_BASE);
        ScriptEnv {
            find_counters: c,
            timers: Vec::new(),
        }
    }
}

/// `ScriptTimers`' capacity.
pub const MAX_TIMERS: usize = 100;

/// Frames per tick — `Game::do_frame` advances `tick` every fifteen frames.
pub const FRAMES_PER_TICK: i64 = 15;

/// The loaded scripts and the statics they keep between calls.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scripts {
    pub program: Program,
    pub state: State,
}

/// The engine's registration table — name, parameter types, return type —
/// as `ScenarioFuncSet::init_funcs` declares the 55 the scripts use, plus
/// `MathUtilFuncSet::rand_int`. The compiler binds a call by name and arity
/// against this and auto-casts each argument to its declared type.
pub const HOST: &[(&str, &[Ty], Ty)] = {
    use Ty::{Int as I, Str as S};
    &[
        ("age", &[I], I),
        ("population", &[I], I),
        ("num_cities", &[I], I),
        ("num_type", &[I, S], I),
        ("num_type_with_queued", &[I, S], I),
        ("at_least_type", &[I, I, S], I),
        ("have_tech", &[I, S], I),
        ("researching_tech", &[I, S], I),
        ("can_pay_cost", &[I, S], I),
        ("find_nation", &[I], S),
        ("find_city_with_num", &[I, I], S),
        ("find_city_id", &[S], I),
        ("find_unit", &[I, S], I),
        ("find_build", &[I, S], I),
        ("find_inactive_build", &[I, S], I),
        ("get_techs_per_age", &[I], I),
        ("num_rare_resources_seen", &[I], I),
        ("find_build_at_city", &[I, S, S, I], I),
        ("num_city_buildings", &[I, S, S, I], I),
        ("find_idle_citizen", &[I], I),
        ("find_num_idle_unit", &[I, S], I),
        ("num_workers_at_building", &[I, I], I),
        ("max_workers_at_building", &[I, I], I),
        ("building_started", &[I, I], I),
        ("num_type_queued", &[I, I, S], I),
        ("object_position_x", &[I, I], I),
        ("object_position_y", &[I, I], I),
        ("was_city_attacked", &[I, S, I], I),
        ("was_city_raided", &[I, S, I], I),
        ("place_building_with_cost", &[I, S, S], I),
        ("place_orphan_building_with_cost", &[I, S, I], I),
        ("place_building_upgrade_with_cost", &[I, S, S], I),
        ("place_city_with_cost", &[I], I),
        ("train_unit_with_cost", &[I, I, S], I),
        ("train_unit_at_with_cost", &[I, I, S, I], I),
        ("research_tech_with_cost", &[I, S], I),
        ("unit_move_order", &[I, I, I, I], I),
        ("citizen_repair_order", &[I, I, I], I),
        ("destroy_building", &[I, I], I),
        ("get_is_no_nation_powers", &[], I),
        ("get_rush_rules", &[], I),
        ("get_mapstyle", &[], S),
        ("is_conquest_scenario", &[], I),
        ("is_victory_score", &[], I),
        ("is_victory_musical_chairs", &[], I),
        ("is_victory_wonder", &[], I),
        ("is_victory_territory", &[], I),
        ("is_victory_economic", &[], I),
        ("is_victory_tech_race", &[], I),
        ("get_starting_town_size", &[I], I),
        ("get_starting_resources", &[I], I),
        ("set_timer", &[S, I], I),
        ("stop_timer", &[S], I),
        ("timer_expired", &[S], I),
        ("rand_int", &[I, I], I),
    ]
};

/// The table alone — what [`Program::load`] binds against. Calling it is a
/// runtime error; only [`ScriptHost`] runs.
pub struct Signatures;

impl Host for Signatures {
    fn signature(&self, name: &str, nargs: usize) -> Option<(Vec<Ty>, Ty)> {
        HOST.iter()
            .find(|(n, p, _)| n.eq_ignore_ascii_case(name) && p.len() == nargs)
            .map(|(_, p, r)| (p.to_vec(), *r))
    }
    fn call(&mut self, name: &str, _args: &[Value]) -> Result<Value, String> {
        Err(format!("`{name}` called on the signature table"))
    }
}

/// The host proper: the simulation, and the leader the script runs for.
pub struct ScriptHost<'a> {
    pub sim: &'a mut Sim,
    /// The running leader, 0-based — the `who − 1` the engine pushed.
    pub who: Player,
}

/// Which class of type a tree entry is, for the finders' range tests.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Class {
    Good,
    Unit,
    Build,
    Tech,
}

const OK: Value = Value::Int(1);
const NO: Value = Value::Int(0);
const BAD: Value = Value::Int(-1);

fn int(v: i32) -> Value {
    Value::Int(v)
}

impl Host for ScriptHost<'_> {
    fn signature(&self, name: &str, nargs: usize) -> Option<(Vec<Ty>, Ty)> {
        Signatures.signature(name, nargs)
    }

    fn call(&mut self, name: &str, args: &[Value]) -> Result<Value, String> {
        let a = |i: usize| args.get(i).map_or(0, Value::as_int);
        let s = |i: usize| args.get(i).map_or_else(String::new, Value::as_str);
        Ok(match name.to_ascii_lowercase().as_str() {
            // ---- queries ----
            "age" => self
                .leader(a(0))
                .map_or(BAD, |w| int(self.sim.tech[w as usize].ages)),
            "population" => self
                .leader(a(0))
                .map_or(BAD, |w| int(self.sim.muster[w as usize].control)),
            "num_cities" => self
                .leader_any(a(0))
                .map_or(BAD, |w| int(self.sim.city_num(w))),
            "num_type" => self.num_type(a(0), &s(1), false, false),
            "num_type_with_queued" => self.num_type(a(0), &s(1), true, true),
            "at_least_type" => {
                let n = self.num_type(a(0), &s(2), false, false);
                match n {
                    Value::Int(c) if c >= 0 => int((a(1) <= c) as i32),
                    _ => BAD,
                }
            }
            "have_tech" => match (self.leader(a(0)), self.type_index(&s(1))) {
                (Some(w), Some(t)) => int(self.sim.tech[w as usize].tech[t] as i32),
                _ => BAD,
            },
            "researching_tech" => match (self.leader_any(a(0)), self.type_index(&s(1))) {
                (Some(w), Some(t)) if self.class(t) == Class::Tech => {
                    int(self.sim.researching(w, t) as i32)
                }
                _ => BAD,
            },
            "can_pay_cost" => self.can_pay_cost(a(0), &s(1)),
            "find_nation" => match self.leader(a(0)) {
                Some(w) => {
                    let tribe = self.sim.tech[w as usize].tribe;
                    Value::Str(
                        self.sim
                            .tech_tree
                            .tribes
                            .get(tribe)
                            .map(|t| t.name.clone())
                            .unwrap_or_default(),
                    )
                }
                None => Value::Str(String::new()),
            },
            "find_city_with_num" => match self.leader_any(a(0)) {
                Some(w) => {
                    let n = a(1);
                    let name = (n >= 1)
                        .then(|| self.cities_of(w).get((n - 1) as usize).copied())
                        .flatten()
                        .map(city_name)
                        .unwrap_or_default();
                    Value::Str(name)
                }
                None => Value::Str(String::new()),
            },
            "find_city_id" => {
                let name = s(0);
                let found = self
                    .sim
                    .cities
                    .iter()
                    .enumerate()
                    .find(|(c, city)| city.alive && city_name(*c).eq_ignore_ascii_case(&name))
                    .map(|(_, city)| i32::from(self.sim.buildings[city.building].index));
                found.map_or(BAD, int)
            }
            "find_unit" => self.find_unit(a(0), &s(1)),
            "find_build" => self.find_build(a(0), &s(1), counter::FIND_BUILD, true),
            "find_inactive_build" => {
                self.find_build(a(0), &s(1), counter::FIND_INACTIVE_BUILD, false)
            }
            "get_techs_per_age" => match self.leader(a(0)) {
                Some(w) => {
                    let p = &self.sim.tech[w as usize];
                    if p.ages > 6 {
                        NO
                    } else {
                        match self
                            .sim
                            .tech_tree
                            .ages
                            .get(p.ages as usize)
                            .copied()
                            .flatten()
                        {
                            Some(age) => {
                                int(self.sim.tech_tree.techs_per_age(&self.sim.setup, p, age))
                            }
                            None => BAD,
                        }
                    }
                }
                None => BAD,
            },
            // `ScenarioFuncSet::num_rare_resources_seen@009ea010`: the
            // length of the leader's `new_rares` list, and −1 for a slot
            // that is not a live leader. `docs/ECONOMY.md`, "The rares a
            // leader has seen" — and `economic.bhs` divides its merchant
            // count by it, so a stub of zero trained none at all.
            "num_rare_resources_seen" => self
                .leader(a(0))
                .map_or(BAD, |w| int(self.sim.ai[w as usize].new_rares.len() as i32)),
            "find_build_at_city" => self.find_build_at_city(a(0), &s(1), &s(2), a(3) != 0),
            "num_city_buildings" => self.num_city_buildings(a(0), &s(1), &s(2), a(3) != 0),
            "find_idle_citizen" => self.find_idle_citizen(a(0)),
            "find_num_idle_unit" => self.find_num_idle_unit(a(0), &s(1)),
            "num_workers_at_building" => match self.gather_building(a(0), a(1)) {
                Some(b) => int(self.sim.num_gatherers(b, false, false)),
                None => BAD,
            },
            "max_workers_at_building" => match self.gather_building(a(0), a(1)) {
                Some(b) => int(self.sim.buildings[b].gather_max.unwrap_or(0)),
                None => BAD,
            },
            "building_started" => match self.valid_build(a(0), a(1)) {
                Some(b) => int(self.sim.buildings[b].started as i32),
                None => BAD,
            },
            "num_type_queued" => self.num_type_queued(a(0), a(1), &s(2)),
            "object_position_x" => self.object_tile(a(0), a(1)).map_or(BAD, |p| int(p.x)),
            "object_position_y" => self.object_tile(a(0), a(1)).map_or(BAD, |p| int(p.y)),
            "was_city_attacked" => self.was_city_attacked(a(0), &s(1), a(2), false),
            "was_city_raided" => self.was_city_attacked(a(0), &s(1), a(2), true),

            // ---- placements ----
            "place_building_with_cost" => match self.city_building_o(a(0), &s(2)) {
                Some(o) => self.place_orphan(a(0), &s(1), o, false),
                None => BAD,
            },
            "place_orphan_building_with_cost" => self.place_orphan(a(0), &s(1), a(2), false),
            "place_building_upgrade_with_cost" => match self.city_building_o(a(0), &s(2)) {
                Some(o) => self.place_orphan(a(0), &s(1), o, true),
                None => BAD,
            },
            "place_city_with_cost" => match self.leader(a(0)) {
                Some(w) => {
                    if self.sim.at_city_limit(w) {
                        BAD
                    } else {
                        let before = self.sim.total_cities(w);
                        self.sim.place_city_ai(w);
                        int((self.sim.total_cities(w) > before) as i32)
                    }
                }
                None => BAD,
            },

            // ---- training and research ----
            "train_unit_with_cost" => self.train_unit(a(0), a(1), &s(2), None),
            "train_unit_at_with_cost" => self.train_unit(a(0), a(1), &s(2), Some(a(3))),
            "research_tech_with_cost" => self.research_tech(a(0), &s(1)),

            // ---- orders ----
            "unit_move_order" => self.unit_move_order(a(0), a(1), a(2), a(3)),
            "citizen_repair_order" => self.citizen_repair_order(a(0), a(1), a(2)),
            "destroy_building" => match self.valid_build(a(0), a(1)) {
                Some(b) => {
                    self.sim.close_building(b, false);
                    OK
                }
                None => BAD,
            },

            // ---- the lobby ----
            "get_is_no_nation_powers" => int(self.sim.lobby.no_nation_powers as i32),
            "get_rush_rules" => int(self.sim.lobby.rush_rules),
            "get_mapstyle" => Value::Str(self.sim.lobby.map_style_name.clone()),
            "is_conquest_scenario" => int(self.sim.lobby.conquest as i32),
            "is_victory_score" => int((self.sim.lobby.victory == 3) as i32),
            "is_victory_musical_chairs" => int((self.sim.lobby.victory == 5) as i32),
            "is_victory_wonder" => int((self.sim.lobby.victory == 6) as i32),
            "is_victory_territory" => int((self.sim.lobby.victory == 7) as i32),
            "is_victory_economic" => int((self.sim.lobby.victory == 8) as i32),
            "is_victory_tech_race" => int((self.sim.lobby.victory == 9) as i32),
            "get_starting_town_size" => match self.leader(a(0)) {
                // `leader_flags2 & 0x80` (a scenario's nomad flag) is not
                // modelled; the patch-version branch answers "do I have a
                // city" only in a scenario or CtW game.
                Some(w) if self.sim.lobby.conquest => int((self.sim.city_num(w) > 0) as i32),
                Some(_) => int(self.sim.lobby.starting_town),
                None => BAD,
            },
            "get_starting_resources" => match self.leader(a(0)) {
                Some(w) => int(self
                    .sim
                    .lobby
                    .starting_resources_for(self.sim.tech[w as usize].team)),
                None => BAD,
            },

            // ---- timers ----
            "set_timer" => self.set_timer(&s(0), a(1)),
            "stop_timer" => {
                let key = s(0);
                let env = &mut self.sim.script_env;
                match env.timers.iter().position(|(k, _)| *k == key) {
                    Some(i) => {
                        env.timers.remove(i);
                        OK
                    }
                    None => BAD,
                }
            }
            "timer_expired" => {
                let key = s(0);
                let tick = self.sim.frame / FRAMES_PER_TICK;
                let env = &mut self.sim.script_env;
                match env.timers.iter().position(|(k, _)| *k == key) {
                    Some(i) if tick >= env.timers[i].1 => {
                        env.timers.remove(i);
                        OK
                    }
                    Some(_) => NO,
                    None => BAD,
                }
            }

            // ---- the sync stream ----
            "rand_int" => {
                self.sim.mark(SITE_RAND_INT);
                int(self.sim.rng.get(a(0), a(1)))
            }

            other => return Err(format!("no host function `{other}`")),
        })
    }
}

/// The name a city answers to. The original returns `CityData.name`, drawn
/// from the nation's city list; the scripts only ever pass the string back
/// or test it `> -1` (a string compare that any letter-led name passes), so
/// a synthetic name behaves identically — `docs/AI.md` §13.
pub fn city_name(c: usize) -> String {
    format!("City {c}")
}

impl ScriptHost<'_> {
    // ---- guards ----

    /// `who` (1-based) as a player, when the slot is a live leader
    /// (`leader_flags & 1|2`).
    fn leader(&self, who: i32) -> Option<Player> {
        let w = self.leader_any(who)?;
        (!self.sim.defeated[w as usize]).then_some(w)
    }

    /// `leader_flags & 1` only — `num_cities`' weaker guard.
    fn leader_any(&self, who: i32) -> Option<Player> {
        let w0 = who.checked_sub(1)?;
        (0..self.sim.players.len() as i32)
            .contains(&w0)
            .then_some(w0 as Player)
    }

    /// `valid_build_o`: a leader's live building by number.
    fn valid_build(&self, who: i32, o: i32) -> Option<usize> {
        let w = self.leader(who)?;
        let o = i16::try_from(o).ok()?;
        if !(BUILD_BASE..BUILD_BASE + 1000).contains(&o) {
            return None;
        }
        self.sim.building_by_o(w, o)
    }

    /// `active_unit_slot` / `valid_unit_o`: a leader's live unit by number.
    fn valid_unit(&self, who: i32, o: i32) -> Option<usize> {
        let w = self.leader(who)?;
        let o = i16::try_from(o).ok()?;
        if !(UNIT_BASE..BUILD_BASE).contains(&o) {
            return None;
        }
        self.sim.unit_by_o(w, o)
    }

    // ---- types ----

    /// `ScenarioFuncSet::get_type_index(name, 0)`: the tree scanned in
    /// `TypeIndex` order for the first display name that matches.
    fn type_index(&self, name: &str) -> Option<TypeId> {
        self.sim
            .tech_tree
            .types
            .iter()
            .position(|d| d.name.eq_ignore_ascii_case(name))
    }

    fn class(&self, t: TypeId) -> Class {
        match self.sim.tech_tree.kind(t) {
            Kind::Good => Class::Good,
            Kind::Unit(_) => Class::Unit,
            Kind::Building { .. } => Class::Build,
            _ => Class::Tech,
        }
    }

    /// The unit record a tree id names.
    fn unit_record(&self, t: TypeId) -> Option<usize> {
        self.sim.unit_types.iter().position(|u| u.tree == Some(t))
    }

    /// The building record a tree id names.
    fn build_record(&self, t: TypeId) -> Option<usize> {
        self.sim.build_types.iter().position(|b| b.tree == Some(t))
    }

    /// A good's ledger slot: the tree's goods are the six common ones first.
    fn good_slot(&self, t: TypeId) -> Option<usize> {
        (t < crate::economy::RESOURCES).then_some(t)
    }

    /// `ObjectData::is(t, 0)` on a building: its type's tree entry against
    /// `t`, a line match.
    fn building_is(&self, b: usize, t: TypeId) -> bool {
        self.sim.buildings[b]
            .ty
            .and_then(|r| self.sim.build_types[r].tree)
            .is_some_and(|bt| self.sim.tech_tree.is(bt, t, false))
    }

    /// The same on a unit.
    fn unit_is(&self, u: usize, t: TypeId) -> bool {
        self.sim.units[u]
            .ty
            .and_then(|r| self.sim.unit_types[r].tree)
            .is_some_and(|ut| self.sim.tech_tree.is(ut, t, false))
    }

    /// `types.list[BASE_UNITTYPES]` — the Citizen, the tree's first unit.
    fn citizen_type(&self) -> Option<TypeId> {
        self.sim
            .tech_tree
            .types
            .iter()
            .position(|d| d.kind.is_unit())
    }

    /// `LeaderData::current_upgrade` then `get_graft`, the normalisation
    /// `num_type_with_queued` and the trainers apply.
    fn line_of(&self, w: Player, t: TypeId) -> TypeId {
        let tree = &self.sim.tech_tree;
        let p = &self.sim.tech[w as usize];
        let up = tree.current_upgrade(&self.sim.setup, p, t);
        tree.get_graft(&self.sim.setup, p, Some(up)).unwrap_or(up)
    }

    /// `LeaderData::num_buildings[t]`: live, finished buildings of exactly
    /// this record.
    fn num_buildings(&self, w: Player, rec: usize) -> i32 {
        self.sim
            .buildings
            .iter()
            .filter(|b| b.alive && b.active && b.owner == w && b.ty == Some(rec))
            .count() as i32
    }

    /// `num_queued[t]` for a building: placed sites not yet active.
    fn num_sites(&self, w: Player, rec: usize) -> i32 {
        self.sim
            .buildings
            .iter()
            .filter(|b| b.alive && !b.active && b.owner == w && b.ty == Some(rec))
            .count() as i32
    }

    /// The price of one of `t` for `w` — the type's own `get_cost`.
    fn price(&self, w: Player, t: TypeId) -> Option<[i32; crate::economy::RESOURCES]> {
        match self.class(t) {
            Class::Unit => self.unit_record(t).map(|r| self.sim.price_of(w, r)),
            Class::Build => self.build_record(t).map(|r| self.sim.building_price(w, r)),
            Class::Tech => Some(self.sim.tech_price(w, t)),
            Class::Good => None,
        }
    }

    /// `TypeData::can_pay_cost(who, −1, −1, 1)`: how many are affordable
    /// with escrow ignored; [`cost::PLENTY`] when nothing is costed.
    fn affordable(&self, w: Player, t: TypeId) -> i32 {
        match self.price(w, t) {
            Some(charges) => cost::affordable(
                &charges,
                &self.sim.ledgers[w as usize],
                &self.sim.holdings[w as usize].available,
                true,
            ),
            None => cost::PLENTY,
        }
    }

    // ---- queries ----

    /// `num_type` / `num_type_with_queued`: the count by class, the latter
    /// normalised to the player's line and including the queue.
    fn num_type(&self, who: i32, name: &str, queued: bool, line: bool) -> Value {
        let Some(w) = self.leader(who) else {
            return BAD;
        };
        let Some(mut t) = self.type_index(name) else {
            return BAD;
        };
        if line {
            t = self.line_of(w, t);
        }
        let m = &self.sim.muster[w as usize];
        match self.class(t) {
            Class::Unit => match self.unit_record(t) {
                Some(r) => int(m.by_type[r] + if queued { m.queued_by_type[r] } else { 0 }),
                None => NO,
            },
            Class::Build => match self.build_record(t) {
                Some(r) => {
                    int(self.num_buildings(w, r) + if queued { self.num_sites(w, r) } else { 0 })
                }
                None => NO,
            },
            Class::Good => match self.good_slot(t) {
                Some(g) => int(self.sim.ledgers[w as usize].bucket[g]),
                None => BAD,
            },
            Class::Tech => BAD,
        }
    }

    /// `can_pay_cost`: `type_avail == 4` (else `-1`), then the count.
    fn can_pay_cost(&self, who: i32, name: &str) -> Value {
        let Some(w) = self.leader(who) else {
            return BAD;
        };
        // The index must be `> 0`, not `>= 0` — type 0 is unanswerable.
        let Some(t) = self.type_index(name).filter(|&t| t > 0) else {
            return BAD;
        };
        if self.sim.type_avail(w, t) != tech::AVAILABLE {
            return BAD;
        }
        int((self.affordable(w, t) != 0) as i32)
    }

    /// The leader's live cities in slot order.
    fn cities_of(&self, w: Player) -> Vec<usize> {
        self.sim
            .cities
            .iter()
            .enumerate()
            .filter(|(_, c)| c.alive && c.owner == w)
            .map(|(i, _)| i)
            .collect()
    }

    /// `get_city_index`: the leader's city by name.
    fn city_index(&self, w: Player, name: &str) -> Option<usize> {
        if name.is_empty() {
            return None;
        }
        self.cities_of(w)
            .into_iter()
            .find(|&c| city_name(c).eq_ignore_ascii_case(name))
    }

    /// A city's centre building number, for the `place_*` adapters.
    fn city_building_o(&self, who: i32, name: &str) -> Option<i32> {
        let w = self.leader(who)?;
        let c = self.city_index(w, name)?;
        Some(i32::from(
            self.sim.buildings[self.sim.cities[c].building].index,
        ))
    }

    /// The city's member chain, the centre first — `CityData.o` then
    /// `city_down` links, tail-appended (`docs/CITIES.md`).
    fn chain(&self, c: usize) -> Vec<usize> {
        let city = &self.sim.cities[c];
        std::iter::once(city.building)
            .chain(city.members.iter().copied())
            .collect()
    }

    fn find_build_at_city(
        &mut self,
        who: i32,
        city: &str,
        ty: &str,
        count_inactive: bool,
    ) -> Value {
        let Some(w) = self.leader_any(who) else {
            return BAD;
        };
        let Some(c) = self.city_index(w, city) else {
            return BAD;
        };
        // An empty type name is "any building".
        let t = if ty.is_empty() {
            None
        } else {
            match self.type_index(ty) {
                Some(t) => Some(t),
                None => return BAD,
            }
        };
        let chain = self.chain(c);
        let fc = &self.sim.script_env.find_counters;
        let cursor = fc[counter::FIND_BUILD_AT_CITY];
        let mut start = 0;
        let mut resumed = false;
        if fc[counter::FIND_BUILD_AT_CITY_WHO] == i32::from(w) && cursor >= 0 {
            let remembered = chain.iter().position(|&h| {
                let b = &self.sim.buildings[h];
                i32::from(b.index) == cursor && b.alive && b.city == Some(c)
            });
            if let Some(pos) = remembered {
                start = pos;
                resumed = true;
            }
        }
        let accept = |h: usize| -> bool {
            let b = &self.sim.buildings[h];
            if !count_inactive && !(b.alive && b.active) {
                return false;
            }
            t.is_none_or(|t| self.building_is(h, t))
        };
        let order: Vec<usize> = if resumed {
            chain[start..]
                .iter()
                .chain(chain[..start].iter())
                .copied()
                .collect()
        } else {
            chain.clone()
        };
        let found = order.into_iter().find(|&h| accept(h));
        let fc = &mut self.sim.script_env.find_counters;
        match found {
            Some(h) => {
                let pos = chain.iter().position(|&x| x == h).unwrap_or(0);
                fc[counter::FIND_BUILD_AT_CITY] = chain
                    .get(pos + 1)
                    .map_or(-1, |&n| i32::from(self.sim.buildings[n].index));
                fc[counter::FIND_BUILD_AT_CITY_WHO] = i32::from(w);
                int(i32::from(self.sim.buildings[h].index))
            }
            None => {
                fc[counter::FIND_BUILD_AT_CITY] = -1;
                BAD
            }
        }
    }

    fn num_city_buildings(&self, who: i32, city: &str, ty: &str, count_inactive: bool) -> Value {
        let Some(w) = self.leader_any(who) else {
            return BAD;
        };
        let Some(c) = self.city_index(w, city) else {
            return BAD;
        };
        let Some(t) = self.type_index(ty) else {
            return BAD;
        };
        let n = self
            .chain(c)
            .into_iter()
            .filter(|&h| {
                let b = &self.sim.buildings[h];
                b.alive && (count_inactive || b.active) && self.building_is(h, t)
            })
            .count();
        int(n as i32)
    }

    /// One lap over the leader's unit numbers from `cursor + 1`, wrapping
    /// to 0.
    fn lap_units(&self, w: Player, cursor: i32) -> impl Iterator<Item = i16> + '_ {
        let mark = i32::from(self.sim.marks[w as usize].unit);
        let start = (cursor + 1).clamp(0, mark.max(0));
        (start..mark)
            .chain(0..start.min(mark))
            .filter_map(|o| i16::try_from(o).ok())
    }

    /// One lap over the leader's building numbers from `cursor + 1`,
    /// wrapping to 2000.
    fn lap_builds(&self, w: Player, cursor: i32) -> impl Iterator<Item = i16> + '_ {
        let base = i32::from(BUILD_BASE);
        let mark = i32::from(self.sim.marks[w as usize].build);
        let start = (cursor + 1).clamp(base, mark.max(base));
        (start..mark)
            .chain(base..start.min(mark))
            .filter_map(|o| i16::try_from(o).ok())
    }

    fn find_unit(&mut self, who: i32, ty: &str) -> Value {
        let Some(w) = self.leader(who) else {
            return BAD;
        };
        let Some(t) = self
            .type_index(ty)
            .filter(|&t| self.class(t) == Class::Unit)
        else {
            return BAD;
        };
        let cursor = self.sim.script_env.find_counters[counter::FIND_UNIT].max(0);
        let found = self
            .lap_units(w, cursor)
            .find(|&o| self.sim.unit_by_o(w, o).is_some_and(|u| self.unit_is(u, t)));
        match found {
            Some(o) => {
                self.sim.script_env.find_counters[counter::FIND_UNIT] = i32::from(o);
                int(i32::from(o))
            }
            None => BAD,
        }
    }

    /// `find_build` (`active`) and `find_inactive_build` (`!active`).
    fn find_build(&mut self, who: i32, ty: &str, slot: usize, active: bool) -> Value {
        let Some(w) = self.leader(who) else {
            return BAD;
        };
        let Some(t) = self
            .type_index(ty)
            .filter(|&t| self.class(t) == Class::Build)
        else {
            return BAD;
        };
        let cursor = self.sim.script_env.find_counters[slot];
        let found = self.lap_builds(w, cursor).find(|&o| {
            self.sim
                .building_by_o(w, o)
                .is_some_and(|b| self.building_is(b, t) && self.sim.buildings[b].active == active)
        });
        match found {
            Some(o) => {
                self.sim.script_env.find_counters[slot] = i32::from(o);
                int(i32::from(o))
            }
            None => BAD,
        }
    }

    /// `UnitData::order_type() == NONE` and a captain not inside anything.
    fn idle_captain(&self, u: usize) -> bool {
        self.sim.units[u].inside.is_none() && self.sim.order_type(u) == index::NONE
    }

    fn find_idle_citizen(&mut self, who: i32) -> Value {
        let Some(w) = self.leader(who) else {
            return BAD;
        };
        let Some(citizen) = self.citizen_type() else {
            return BAD;
        };
        // The nation's substitute when it cannot use the plain type.
        let p = &self.sim.tech[w as usize];
        let t = if self
            .sim
            .tech_tree
            .tribe_can_type(&self.sim.setup, p, citizen)
            == 0
        {
            self.sim
                .tech_tree
                .get_graft(&self.sim.setup, p, Some(citizen))
                .unwrap_or(citizen)
        } else {
            citizen
        };
        let mark = i32::from(self.sim.marks[w as usize].unit);
        let cursor =
            self.sim.script_env.find_counters[counter::FIND_IDLE_CITIZEN].clamp(0, mark.max(0));
        let found = self.lap_units(w, cursor).find(|&o| {
            self.sim
                .unit_by_o(w, o)
                .is_some_and(|u| self.unit_is(u, t) && self.idle_captain(u))
        });
        match found {
            Some(o) => {
                self.sim.script_env.find_counters[counter::FIND_IDLE_CITIZEN] = i32::from(o);
                int(i32::from(o))
            }
            None => {
                self.sim.script_env.find_counters[counter::FIND_IDLE_CITIZEN] = 0;
                BAD
            }
        }
    }

    fn find_num_idle_unit(&self, who: i32, ty: &str) -> Value {
        let Some(w) = self.leader(who) else {
            return BAD;
        };
        let Some(t) = self
            .type_index(ty)
            .filter(|&t| self.class(t) == Class::Unit)
        else {
            return BAD;
        };
        let n = self
            .sim
            .units
            .iter()
            .enumerate()
            .filter(|(u, unit)| {
                unit.owner == w && unit.alive() && self.unit_is(*u, t) && self.idle_captain(*u)
            })
            .count();
        int(n as i32)
    }

    /// A valid building whose type gathers (`build_flags & 0x40`).
    fn gather_building(&self, who: i32, o: i32) -> Option<usize> {
        let b = self.valid_build(who, o)?;
        self.sim.is_gather_type(b).then_some(b)
    }

    fn num_type_queued(&self, who: i32, o: i32, ty: &str) -> Value {
        let Some(b) = self.valid_build(who, o) else {
            return BAD;
        };
        let q = &self.sim.buildings[b].queue;
        if ty.is_empty() || ty.starts_with(' ') {
            return int(q.items.len() as i32);
        }
        let Some(t) = self.type_index(ty) else {
            return BAD;
        };
        let n = q
            .items
            .iter()
            .filter(|i| match i.tech {
                Some(tt) => self.sim.tech_tree.is(tt, t, false),
                None => self.sim.unit_types[i.ty]
                    .tree
                    .is_some_and(|ut| self.sim.tech_tree.is(ut, t, false)),
            })
            .count();
        int(n as i32)
    }

    /// `object_position_x/y`: a unit's or building's tile; a garrisoned
    /// unit reports its container's.
    fn object_tile(&self, who: i32, o: i32) -> Option<Pos> {
        let pos = if let Some(u) = self.valid_unit(who, o) {
            match self.sim.units[u].inside {
                Some(b) => self.sim.buildings[b].pos,
                None => self.sim.units[u].pos,
            }
        } else {
            self.sim.buildings[self.valid_build(who, o)?].pos
        };
        Some(Pos::new(pos.x / UNITS_PER_TILE, pos.y / UNITS_PER_TILE))
    }

    /// `was_city_attacked` / `was_city_raided`. The simulation keeps
    /// `attack_stamp` on the city record; a raid stamp is not modelled, so
    /// a raid query reads "never" (`docs/AI.md` §13).
    fn was_city_attacked(&self, who: i32, city: &str, seconds: i32, raid: bool) -> Value {
        let Some(w) = self.leader(who) else {
            return BAD;
        };
        let frame = self.sim.frame;
        let hit = |c: usize| -> bool {
            let stamp = if raid {
                0
            } else {
                self.sim.cities[c].attack_stamp
            };
            if seconds < 1 {
                stamp != 0
            } else {
                (frame - stamp) / FRAMES_PER_TICK <= i64::from(seconds)
            }
        };
        match self.city_index(w, city) {
            Some(c) => int(hit(c) as i32),
            None => int(self.cities_of(w).into_iter().any(hit) as i32),
        }
    }

    // ---- placements ----

    /// `place_orphan_building_with_cost`, and its upgrade variant: the city
    /// hint from the reference building, the price with escrow ignored, then
    /// `Leader::produce_building`.
    fn place_orphan(&mut self, who: i32, ty: &str, near_o: i32, upgrade: bool) -> Value {
        let Some(w) = self.leader(who) else {
            return BAD;
        };
        let Some(near) = self.valid_build(who, near_o) else {
            return BAD;
        };
        let Some(mut t) = self
            .type_index(ty)
            .filter(|&t| self.class(t) == Class::Build)
        else {
            return BAD;
        };
        if upgrade {
            t = self
                .sim
                .tech_tree
                .current_upgrade(&self.sim.setup, &self.sim.tech[w as usize], t);
        }
        let Some(rec) = self.build_record(t) else {
            return BAD;
        };
        // The city hint: only from an active city building.
        let nb = &self.sim.buildings[near];
        let city = if nb.active && self.sim.building_is_city(near) {
            nb.city
        } else {
            None
        };
        if self.affordable(w, t) == 0 {
            return NO;
        }
        int(self.sim.produce_building(w, rec, near, city) as i32)
    }

    // ---- training and research ----

    /// `train_unit_with_cost` (`at == None`, the producer found by the
    /// rotating search) and `train_unit_at_with_cost`.
    fn train_unit(&mut self, who: i32, num: i32, ty: &str, at: Option<i32>) -> Value {
        let Some(w) = self.leader(who) else {
            return BAD;
        };
        if !(1..=2000).contains(&num) {
            return BAD;
        }
        let Some(t) = self.type_index(ty) else {
            return BAD;
        };
        let t = self.line_of(w, t);
        let Some(rec) = self.unit_record(t) else {
            return BAD;
        };
        let b = match at {
            None => {
                // The producing building: the type's `where`, upgraded to
                // the player's level.
                let Some(where_) = self.sim.tech_tree.types[t].where_ else {
                    return BAD;
                };
                let producer = self.sim.tech_tree.current_upgrade(
                    &self.sim.setup,
                    &self.sim.tech[w as usize],
                    where_,
                );
                if self.affordable(w, t) < num {
                    return NO;
                }
                let cursor = self.sim.script_env.find_counters[counter::TRAIN_PRODUCER];
                let found = self.lap_builds(w, cursor).find(|&o| {
                    self.sim.building_by_o(w, o).is_some_and(|b| {
                        self.sim.buildings[b].active && self.building_is(b, producer)
                    })
                });
                let Some(o) = found else { return NO };
                self.sim.script_env.find_counters[counter::TRAIN_PRODUCER] = i32::from(o);
                self.sim.building_by_o(w, o).unwrap_or(0)
            }
            Some(o) => {
                if self.affordable(w, t) < num {
                    return NO;
                }
                let Some(b) = self.valid_build(who, o) else {
                    return BAD;
                };
                // `could_queue`: can_make and room.
                let bd = &self.sim.buildings[b];
                if !bd.active || !bd.alive || !bd.queue.has_room() {
                    return BAD;
                }
                b
            }
        };
        self.action_queue_up(w, b, QueueType::Unit(rec, t), num);
        int(i32::from(self.sim.buildings[b].index))
    }

    fn research_tech(&mut self, who: i32, ty: &str) -> Value {
        let Some(w) = self.leader(who) else {
            return BAD;
        };
        let Some(t) = self.type_index(ty) else {
            return BAD;
        };
        if self.sim.tech[w as usize].tech[t] {
            return OK;
        }
        let Some(where_) = self.sim.tech_tree.types[t].where_ else {
            return BAD;
        };
        let producer =
            self.sim
                .tech_tree
                .current_upgrade(&self.sim.setup, &self.sim.tech[w as usize], where_);
        let cursor = self.sim.script_env.find_counters[counter::RESEARCH_PRODUCER];
        let found = self.lap_builds(w, cursor).find(|&o| {
            self.sim
                .building_by_o(w, o)
                .is_some_and(|b| self.sim.buildings[b].active && self.building_is(b, producer))
        });
        let Some(o) = found else { return NO };
        self.sim.script_env.find_counters[counter::RESEARCH_PRODUCER] = i32::from(o);
        let Some(b) = self.sim.building_by_o(w, o) else {
            return NO;
        };
        // `can_queue`: the price through the building, then `could_queue`.
        let charges = self.sim.tech_price(w, t);
        let paid = cost::can_pay(
            &charges,
            &self.sim.ledgers[w as usize],
            &self.sim.holdings[w as usize].available,
            1,
        );
        if !paid || !self.sim.buildings[b].queue.has_room() {
            return NO;
        }
        self.action_queue_up(w, b, QueueType::Tech(t), 1);
        int(i32::from(o))
    }

    /// `Group::action_queue_up` for a one-member group: the repeat path for
    /// an owned unit type, the research path (once, and only when not
    /// already researching) for everything else. A refused `queue_up` is
    /// silent, as it is in the original.
    fn action_queue_up(&mut self, w: Player, b: usize, what: QueueType, num: i32) {
        // The building group every caller pushes first (`009f4595`,
        // `009ee881`, …; `docs/GROUPS.md` §28.4): it takes a pool slot.
        self.sim.push_building_group(w, b);
        match what {
            QueueType::Unit(rec, t) if self.sim.tech[w as usize].tech[t] => {
                for _ in 0..num {
                    let _ = self.sim.queue_up(b, rec);
                }
            }
            QueueType::Unit(rec, t) => {
                if !self.sim.researching(w, t) {
                    let _ = self.sim.queue_up(b, rec);
                }
            }
            QueueType::Tech(t) => {
                if !self.sim.researching(w, t) {
                    let _ = self.sim.queue_tech(b, t);
                }
            }
        }
    }

    // ---- orders ----

    fn unit_move_order(&mut self, who: i32, o: i32, x: i32, y: i32) -> Value {
        let Some(u) = self.valid_unit(who, o) else {
            return BAD;
        };
        if !self.sim.world.tile_in_bounds(Pos::new(x, y)) {
            return BAD;
        }
        // The captain of whatever carries it; the simulation's units are
        // all captains, and a garrisoned one is ordered where it stands.
        self.sim.clear_orders(u);
        let dest = Pos::new(
            x * UNITS_PER_TILE + UNITS_PER_TILE / 2,
            y * UNITS_PER_TILE + UNITS_PER_TILE / 2,
        );
        self.sim
            .add_move_order(u, dest, MoveKind::MoveTo, QueuePos::New, true);
        OK
    }

    fn citizen_repair_order(&mut self, who: i32, unit_o: i32, build_o: i32) -> Value {
        let (Some(u), Some(b)) = (self.valid_unit(who, unit_o), self.valid_build(who, build_o))
        else {
            return BAD;
        };
        if !self.sim.units[u].alive() || !self.citizen_type().is_some_and(|c| self.unit_is(u, c)) {
            return BAD;
        }
        self.sim.clear_orders(u);
        let body = if self.sim.buildings[b].active {
            Body::Repair(b)
        } else {
            Body::Build(b)
        };
        // `action_swarm_around(…, QUEUE_FIRST, kind, 1)`: the approach move
        // ahead of the order, at the head of an already-cleared list.
        self.sim.swarm_around(u, b, body, true);
        OK
    }

    // ---- timers ----

    fn set_timer(&mut self, key: &str, seconds: i32) -> Value {
        if seconds <= 0 || key.is_empty() {
            return BAD;
        }
        let tick = self.sim.frame / FRAMES_PER_TICK;
        let env = &mut self.sim.script_env;
        if let Some(i) = env.timers.iter().position(|(k, _)| k == key) {
            env.timers.remove(i);
        }
        if env.timers.len() >= MAX_TIMERS {
            return BAD;
        }
        env.timers
            .push((key.to_string(), tick + i64::from(seconds)));
        OK
    }
}

/// What `action_queue_up` is asked to queue.
#[derive(Clone, Copy)]
enum QueueType {
    /// A unit record and its tree id.
    Unit(usize, TypeId),
    Tech(TypeId),
}

impl Sim {
    /// Loads the opening scripts — `Leaders::init_production_script`'s
    /// `Compiler::compile` over the files the install ships — binding host
    /// calls against [`HOST`].
    pub fn load_scripts(
        &mut self,
        roots: &[&str],
        sources: &[Source],
    ) -> Result<(), bhs::CompileError> {
        let program = Program::load(roots, sources, &Signatures)?;
        let state = State::new(&program);
        self.scripts = Some(Scripts { program, state });
        Ok(())
    }

    /// `production_ai` step 1's call: `run_script(script_run_time, name)`
    /// with `(who + 1, ref step, boom_vs_rush, num_loops)`. Returns the
    /// script's return value and the `step` it wrote back. The program and
    /// its statics leave the simulation for the run, since the host is the
    /// simulation.
    pub fn run_script(
        &mut self,
        who: Player,
        name: &str,
        step: i32,
        boom_vs_rush: i32,
        num_loops: i32,
    ) -> Result<(i32, i32), RunError> {
        let Some(mut scripts) = self.scripts.take() else {
            return Err(RunError::NotFound);
        };
        let mut args = [
            Value::Int(i32::from(who) + 1),
            Value::Int(step),
            Value::Int(boom_vs_rush),
            Value::Int(num_loops),
        ];
        let result = {
            let mut host = ScriptHost { sim: self, who };
            bhs::run(
                &scripts.program,
                &mut scripts.state,
                &mut host,
                name,
                &mut args,
            )
        };
        self.scripts = Some(scripts);
        result.map(|r| (r.as_int(), args[1].as_int()))
    }

    /// `place_city_with_cost`'s body — `compute_sites(1)` then
    /// `found_cities` ([`crate::ai_sites`]).
    pub fn place_city_ai(&mut self, who: Player) {
        self.place_city_ai_impl(who);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_binds_every_shipped_call_and_nothing_else() {
        assert_eq!(
            HOST.len(),
            55,
            "the report's 55: 54 ScenarioFuncSet calls and rand_int"
        );
        assert_eq!(
            Signatures.signature("find_build_at_city", 4),
            Some((vec![Ty::Int, Ty::Str, Ty::Str, Ty::Int], Ty::Int))
        );
        assert_eq!(Signatures.signature("find_build_at_city", 3), None);
        assert_eq!(
            Signatures.signature("get_mapstyle", 0).map(|s| s.1),
            Some(Ty::Str)
        );
        assert!(Signatures.signature("print_game_msg", 1).is_none());
    }

    #[test]
    fn timers_are_ticks_keyed_by_string_and_expiry_consumes() {
        let mut sim = Sim::new(crate::Tuning::RON, crate::World::new(2, 2), 1);
        let mut h = ScriptHost {
            sim: &mut sim,
            who: 0,
        };
        assert_eq!(h.set_timer("1", 0), BAD);
        assert_eq!(h.set_timer("", 5), BAD);
        assert_eq!(h.set_timer("1", 2), OK);
        assert_eq!(
            h.call("timer_expired", &[Value::Str("1".into())]).unwrap(),
            NO
        );
        assert_eq!(
            h.call("timer_expired", &[Value::Str("2".into())]).unwrap(),
            BAD
        );
        h.sim.frame = 2 * FRAMES_PER_TICK;
        assert_eq!(
            h.call("timer_expired", &[Value::Str("1".into())]).unwrap(),
            OK
        );
        // Consumed: asking again finds nothing.
        assert_eq!(
            h.call("timer_expired", &[Value::Str("1".into())]).unwrap(),
            BAD
        );
        assert_eq!(
            h.call("stop_timer", &[Value::Str("1".into())]).unwrap(),
            BAD
        );
        assert_eq!(h.set_timer("1", 3), OK);
        assert_eq!(h.call("stop_timer", &[Value::Str("1".into())]).unwrap(), OK);
    }

    #[test]
    fn rand_int_is_the_sync_stream_with_an_exclusive_bound() {
        let mut sim = Sim::new(crate::Tuning::RON, crate::World::new(2, 2), 1);
        let mut oracle = sim.rng;
        let mut h = ScriptHost {
            sim: &mut sim,
            who: 0,
        };
        for _ in 0..50 {
            let v = h
                .call("rand_int", &[Value::Int(1), Value::Int(10)])
                .unwrap()
                .as_int();
            assert_eq!(v, oracle.get(1, 10));
            assert!((1..10).contains(&v));
        }
    }

    #[test]
    fn a_bad_who_is_minus_one_everywhere_and_a_bad_name_too() {
        let mut sim = Sim::new(crate::Tuning::RON, crate::World::new(2, 2), 1);
        let mut h = ScriptHost {
            sim: &mut sim,
            who: 0,
        };
        for who in [0, 2, -1] {
            assert_eq!(h.call("age", &[Value::Int(who)]).unwrap(), BAD, "who {who}");
            assert_eq!(h.call("num_cities", &[Value::Int(who)]).unwrap(), BAD);
            assert_eq!(
                h.call("find_nation", &[Value::Int(who)]).unwrap(),
                Value::Str(String::new())
            );
        }
        assert_eq!(h.call("age", &[Value::Int(1)]).unwrap(), NO);
        assert_eq!(
            h.call("num_type", &[Value::Int(1), Value::Str("Nothing".into())])
                .unwrap(),
            BAD
        );
        assert_eq!(
            h.call("find_city_with_num", &[Value::Int(1), Value::Int(1)])
                .unwrap(),
            Value::Str(String::new())
        );
        assert_eq!(
            h.call("find_city_id", &[Value::Str(String::new())])
                .unwrap(),
            BAD
        );
    }
}
