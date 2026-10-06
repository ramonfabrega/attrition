//! The production AI's driver — `docs/AI.md` §2.
//!
//! What the original calls the "production AI" is one `Leader` per computer
//! player taking stock of everything it owns (`plan_strategy`, the census),
//! filling an eleven-slot shopping list (`MakeList`) from a handful of
//! producers, and spending from it (`make_stuff`). This module holds the
//! parts that are pure state and arithmetic — the personality roll, the
//! make list, the goods picture, the cadence and the step machine — so each
//! can be pinned on its own before the producers that read them land.
//!
//! Everything here is integer arithmetic at the original's own scales and
//! draws, where it draws, from the sync stream (`combat::Rng`), in the
//! original's order. A draw out of order is a desync, so the order is the
//! specification and the tests count draws.

use crate::combat::Rng;
use crate::economy::{Ledger, RESOURCES};

/// The nation roster's indices — `rules.xml`'s `TRIBES` order, which is
/// what `LeaderData::tribe` holds and what `has_tribe_bonus(n)` compares
/// against (`docs/TECH.md` §"`has_preq`"). The AI hardcodes several.
pub mod tribe {
    pub const AZTECS: usize = 0;
    pub const MAYA: usize = 1;
    pub const INCA: usize = 2;
    pub const BANTU: usize = 3;
    pub const NUBIANS: usize = 4;
    pub const GREEKS: usize = 5;
    pub const ROMANS: usize = 6;
    pub const EGYPTIANS: usize = 7;
    pub const TURKS: usize = 8;
    pub const SPANISH: usize = 9;
    pub const FRENCH: usize = 10;
    pub const BRITISH: usize = 11;
    pub const GERMANS: usize = 12;
    pub const RUSSIANS: usize = 13;
    pub const CHINESE: usize = 14;
    pub const JAPANESE: usize = 15;
    pub const KOREANS: usize = 16;
    pub const MONGOLS: usize = 17;
    pub const IROQUOIS: usize = 18;
    pub const LAKOTA: usize = 19;
    pub const AMERICANS: usize = 20;
    pub const INDIANS: usize = 21;
    pub const DUTCH: usize = 22;
    pub const PERSIANS: usize = 23;
}

/// `Personality` — `LeaderData+0x6dd4`, 24 ints, each −1/0/1 unless a
/// nation pins it. Rolled once per computer leader by
/// [`Personality::roll`] inside `Leader::init` (`docs/AI.md` §6); the
/// producers read them as biases.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Personality {
    pub rush: i32,
    pub cities: i32,
    pub upgrades: i32,
    pub arms: i32,
    pub army: i32,
    pub army_size: i32,
    pub raid: i32,
    pub invade: i32,
    pub target: i32,
    pub strategy: i32,
    pub raze: i32,
    pub spells: i32,
    pub forts: i32,
    pub nukes: i32,
    pub air: i32,
    pub naval: i32,
    pub market: i32,
    pub scouts: i32,
    pub civilians: i32,
    /// Set by `Leader::init` when the leader gets the `defensive` script.
    pub early_army: i32,
    pub friendly_human: i32,
    pub alliance_human: i32,
    pub friendly_ai: i32,
    pub alliance_ai: i32,
}

/// What [`Personality::roll`] reads about the other leaders: each other
/// leader with `leader_flags & 3 == 3` (active and alive — **the human
/// too**; the first reading's "computer leader" was wrong, corrected
/// 2026-08-24 against `random_personality`'s loop), not allied with the
/// roller and not flagged `leader_flags & 0x10`, by tribe. Order is leader
/// order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rival {
    pub tribe: usize,
}

/// The lobby facts the roll reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RollSetup {
    /// `world+0x34` — the map style's sea class (`World::sea_map`; 0 land
    /// maps … 4 Colonial Powers), which the roll reads as "small map" when
    /// `< 3` and "sea map" when `> 2`.
    pub sea_map: i32,
    /// `info.rush_rules == 8`.
    pub rush_rules_off: bool,
}

/// A three-way roll: `Random::get(0, 0xffff) % 3 − 1`.
fn tri(rng: &mut Rng) -> i32 {
    rng.roll() % 3 - 1
}

impl Personality {
    /// `Leader::random_personality@006cfd00`, read whole (`docs/AI.md` §6).
    /// The draw order is the specification.
    pub fn roll(rng: &mut Rng, tribe: usize, rivals: &[Rival], setup: &RollSetup) -> Personality {
        use tribe::*;
        // `early_army` starts at 0; `Leader::init` sets it for `defensive`.
        let mut p = Personality {
            rush: tri(rng),
            ..Personality::default()
        };
        // The rush roll, bent by the nation. Each guard's draw happens
        // only when the pattern and the first test hold, as in the original.
        match tribe {
            // Boomers: a rush roll reverts four times in five.
            CHINESE | INCA | EGYPTIANS | NUBIANS | GREEKS if p.rush == 1 && rng.roll() % 5 != 0 => {
                p.rush = -1;
            }
            GERMANS | TURKS if p.rush == 1 && rng.roll() % 3 != 0 => {
                p.rush = 0;
            }
            // Rushers: a boom roll becomes a rush on a small map, two times
            // in three.
            AZTECS | BANTU | MONGOLS | ROMANS | JAPANESE if p.rush < 0 && rng.roll() % 3 != 0 => {
                p.rush = i32::from(setup.sea_map < 3);
            }
            _ => {}
        }
        // Then bent again by who is across the map.
        for r in rivals {
            match r.tribe {
                CHINESE | RUSSIANS | MAYA if p.rush == 1 && rng.roll() & 3 != 0 => {
                    p.rush = 0;
                }
                AZTECS | JAPANESE if p.rush == -1 && rng.roll() & 1 != 0 => {
                    p.rush = 0;
                }
                _ => {}
            }
        }
        if p.rush == 1 && setup.sea_map > 2 && rng.roll() % 3 != 0 {
            p.rush = 0;
        }
        p.cities = tri(rng);
        // `arms` has a two-stage roll: a coin picks a five-way or a
        // three-way. `upgrades` is not rolled here.
        p.arms = if rng.roll() & 1 == 0 {
            rng.roll() % 5 - 2
        } else {
            tri(rng)
        };
        p.army = tri(rng);
        p.army_size = tri(rng);
        p.raid = tri(rng);
        if (tribe == MONGOLS || tribe == AZTECS) && rng.roll() % 3 != 0 {
            p.raid = 1;
        }
        if setup.rush_rules_off {
            p.raid = -1;
        }
        p.invade = tri(rng);
        p.target = tri(rng);
        p.raze = 0;
        p.strategy = tri(rng);
        p.spells = tri(rng);
        p.forts = tri(rng);
        if tribe == ROMANS {
            p.forts = 1;
        }
        p.nukes = tri(rng);
        p.air = tri(rng);
        if tribe == GERMANS {
            p.air = 1;
        }
        let naval_roll = rng.roll();
        p.naval = naval_roll % 3 - 1;
        if (tribe == JAPANESE || tribe == BRITISH || tribe == SPANISH) && p.naval < 1 {
            p.naval = naval_roll % 3;
        }
        p.market = tri(rng);
        p.scouts = tri(rng);
        if tribe == SPANISH {
            p.scouts = 1;
        }
        p.civilians = tri(rng);
        p.friendly_human = tri(rng);
        let alliance = tri(rng);
        p.friendly_ai = p.friendly_human;
        p.alliance_human = alliance;
        p.alliance_ai = alliance;
        p
    }
}

/// One entry of the make list — `MakeObject`, 0x28 bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MakeObject {
    /// The type to make; `-1` is an empty slot.
    pub t: i32,
    pub val: i32,
    pub escrow: i32,
    pub city: i32,
    pub up: i32,
    pub o: i32,
    pub num: i32,
    pub cat: i32,
    pub wx: i32,
    pub wy: i32,
}

impl MakeObject {
    /// The record `MakeList::clear@006c9db0` writes into every slot — all
    /// ten fields, not `t` alone: `t −1, val −1, escrow 0, city −1, up 0,
    /// o −1, num 1, cat 0, wx 0, wy 0`. It is also what a `LEADERS=9` dump
    /// prints for a slot nothing has touched (run9's frame 1) and for every
    /// slot after step 2's clear (run18b's dump-frame 6577, run19's 8177).
    ///
    /// `val −1` is load-bearing: `make_me` inserts on `list[k].val < val`,
    /// so an entry offered at `val 0` lands in a cleared slot (run18b's
    /// citizen at dump-frame 6382) and would not in one left at `val 0`.
    pub const EMPTY: MakeObject = MakeObject {
        t: -1,
        val: -1,
        escrow: 0,
        city: -1,
        up: 0,
        o: -1,
        num: 1,
        cat: 0,
        wx: 0,
        wy: 0,
    };
}

/// The number of slots — `MakeList::init` allocates eleven, and
/// `make_stuff` walks `0x1b8 / 0x28`.
pub const MAKE_SLOTS: usize = 11;

/// The leader's shopping list: slot 0 the best, slots 1–3 a ranked list of
/// runners-up, slots 4–10 one per category. `docs/AI.md` §2.11.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MakeList {
    pub list: [MakeObject; MAKE_SLOTS],
}

impl Default for MakeList {
    fn default() -> MakeList {
        MakeList::new()
    }
}

impl MakeList {
    pub const fn new() -> MakeList {
        MakeList {
            list: [MakeObject::EMPTY; MAKE_SLOTS],
        }
    }

    /// `MakeList::clear@006c9db0` — every slot to [`MakeObject::EMPTY`],
    /// whole. Only the *expiry* in `make_stuff` (`ai_make.rs`) clears `t`
    /// alone and leaves the other nine fields standing; the two are told
    /// apart in a dump by whether a `t −1` slot still carries its old
    /// `val`.
    pub fn clear(&mut self) {
        for m in &mut self.list {
            *m = MakeObject::EMPTY;
        }
    }

    pub const fn head(&self) -> &MakeObject {
        &self.list[0]
    }

    /// `MakeList::make_me@006c9be0`, read whole. `cat` is the entry's
    /// category *and* the slot index it competes for (4..=10; a producer
    /// passing 0..=3 double-writes the ranked list, which none does).
    #[allow(clippy::too_many_arguments)]
    pub fn make_me(
        &mut self,
        t: i32,
        val: i32,
        escrow: i32,
        cat: i32,
        city: i32,
        up: i32,
        num: i32,
        wx: i32,
        wy: i32,
    ) {
        let entry = MakeObject {
            t,
            val,
            escrow,
            city,
            up,
            o: -1,
            num,
            cat,
            wx,
            wy,
        };
        if self.list[0].val < val {
            // A new best overwrites the head outright; the old head is not
            // shifted down. Duplicates of the type below it are cleared.
            self.list[0] = entry;
            for k in 1..4 {
                if self.list[k].t == t {
                    self.list[k].t = -1;
                }
            }
        } else {
            let mut k = 1;
            while k < 4 {
                if self.list[k].val <= val {
                    // Shift k..=2 down to k+1..=3; slot 3 falls off.
                    let mut j = 3;
                    while j > k {
                        self.list[j] = self.list[j - 1];
                        j -= 1;
                    }
                    for j in k..4 {
                        if self.list[j].t == t {
                            self.list[j].t = -1;
                        }
                    }
                    self.list[k] = entry;
                    break;
                }
                if self.list[k].t == t {
                    // The same type already ranks higher: not inserted.
                    break;
                }
                k += 1;
            }
        }
        let c = usize::try_from(cat).unwrap_or(0).min(MAKE_SLOTS - 1);
        if self.list[c].val < val {
            self.list[c] = entry;
        }
    }
}

/// The step machine's position — `LeaderData::production_step`. `Idle` is
/// the original's 0; the rest are its 1..=11 in order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Step {
    #[default]
    Idle,
    Script,
    Setup,
    Cities,
    Research,
    Upgrades,
    Units,
    Buildings,
    Make,
    Units2,
    Buildings2,
    Make2,
}

impl Step {
    /// The original's integer, for logs and tests.
    pub const fn number(self) -> i32 {
        match self {
            Step::Idle => 0,
            Step::Script => 1,
            Step::Setup => 2,
            Step::Cities => 3,
            Step::Research => 4,
            Step::Upgrades => 5,
            Step::Units => 6,
            Step::Buildings => 7,
            Step::Make => 8,
            Step::Units2 => 9,
            Step::Buildings2 => 10,
            Step::Make2 => 11,
        }
    }

    const fn next(self) -> Step {
        match self {
            Step::Idle => Step::Idle,
            Step::Script => Step::Setup,
            Step::Setup => Step::Cities,
            Step::Cities => Step::Research,
            Step::Research => Step::Upgrades,
            Step::Upgrades => Step::Units,
            Step::Units => Step::Buildings,
            Step::Buildings => Step::Make,
            Step::Make => Step::Units2,
            Step::Units2 => Step::Buildings2,
            Step::Buildings2 => Step::Make2,
            Step::Make2 => Step::Idle,
        }
    }
}

/// What the script's run reported back to `production_ai` — the return
/// value of the `ai economic`/`defensive` function, or that it could not
/// run at all.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScriptResult {
    /// The function returned this integer. `1` is `BLOCK_ON_THIS`, `3` is
    /// `SCRIPT_DONE`; anything else falls through to the next step.
    Returned(i32),
    /// `run_script` failed — not found, parameter mismatch, runtime error.
    /// The original drops the script for the rest of the game.
    Failed,
}

/// One site of the leader's ten — `Site` (`wx, wy, val, reg, dist, rank`),
/// `docs/AI.md` §2.7. `wx, wy` are in cells; `reg` is the sim's region id.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Site {
    pub wx: i32,
    pub wy: i32,
    pub val: i32,
    pub reg: i32,
    pub dist: i32,
    pub rank: i32,
}

/// `Sites` holds ten.
pub const SITES: usize = 10;

/// The per-city AI record — the `CityData` fields the census writes and
/// the producers read (`docs/AI.md` §2.3 steps 2 and 13; `create-buildings.md`
/// §4). Indexed like [`crate::Sim::cities`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CityAi {
    /// Citizens of this city with no action, and with one.
    ///
    /// **`free` is a byte, and it wraps** — `CityData +0x5a` is a `uchar`
    /// and both of its writers are a bare `*p = *p + 1` / `*p = *p - 1`
    /// with no clamp, so a decrement at zero lands on 255 rather than on
    /// −1 (`docs/AI.md` §2.20). Every reader of it in the original widens
    /// it with a zero extension, so the readers here take `i32::from`.
    /// `busy` and the counters under it are the same width in the
    /// original and are still `i32` here — the widening ledger's row.
    pub free: u8,
    pub busy: i32,
    pub gatherers: i32,
    /// The nearest free or gathering citizen, in `dist / 0x300`; 100 when
    /// none.
    pub peasant_dist: i32,
    pub in_port: i32,
    /// The site picture over the city's circle: water tiles, open land
    /// tiles, those too small for a 4×4, dock candidates, the footprint
    /// classes that fit (`space[n − 2]` for `n = 2..4`), and the best
    /// gather amount per good on the occupied tiles.
    pub ocean: i32,
    pub land: i32,
    pub filled: i32,
    pub dock_tile: i32,
    pub space: [i32; 3],
    pub ter: [i32; RESOURCES],
    /// `CityData::ocean_filled` (+0x66) and `bordering` (+0x65): zeroed
    /// by the sweep, read by the site score and the dock family; no
    /// writer is modelled yet.
    pub ocean_filled: i32,
    pub bordering: i32,
}

/// The census — every `LeaderData` count `plan_strategy`'s sweep writes
/// (`docs/AI.md` §2.3), under the PDB's names, so a `LEADERS=9` dump can be
/// diffed against it field by field. The per-region arrays are indexed by
/// the sim's region id (the original folds sea regions `% 0x3f` into
/// 63-entry arrays; ours are one slot per region, land or sea) and are
/// sized by [`Census::resize`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Census {
    // Zeroed every sweep (step 3).
    pub active: i32,
    pub combat: i32,
    pub non_siege: i32,
    pub sea_combat: i32,
    pub siege: i32,
    pub defense: i32,
    pub attack: i32,
    pub naval: i32,
    pub air: i32,
    pub missile: i32,
    pub transports: i32,
    pub fishermen: i32,
    pub idle_fishermen: i32,
    pub peasants: i32,
    pub scholars: i32,
    pub caras: i32,
    pub merchants: i32,
    pub fighters: i32,
    pub bombers: i32,
    pub cruise: i32,
    pub nuke: i32,
    pub free_peasants: i32,
    pub xport_peasants: i32,
    pub gatherers: i32,
    pub attacked: i32,
    pub full_cities: i32,
    // Kept by the unit lifecycle, read here.
    pub pop: i32,
    pub scouts: i32,
    // Territory (step 4).
    pub my_team_terr: i32,
    pub other_team_terr: i32,
    pub min_other_team_terr: i32,
    // Step 5–7.
    pub filled_gather_slots: [i32; RESOURCES],
    pub gather_slots: [i32; RESOURCES],
    pub ally_mask: u32,
    /// `invaders[i]`: how many of *leader i's* non-siege military units
    /// stand on my land — written into every leader by every sweep.
    pub invaders: Vec<i32>,
    // Maxima (step 12).
    pub gather_slots_high: [i32; RESOURCES],
    pub peasant_high: i32,
    pub scholar_high: i32,
    pub caravan_high: i32,
    pub merchant_high: i32,
    pub army_high: i32,
    pub city_high: i32,
    pub village_high: i32,
    pub population_high: i32,
    pub city_pop_high: i32,
    pub resources_controlled: i32,
    // Wars (steps 14–15).
    pub wars: i32,
    pub allies: i32,
    pub active_wars: i32,
    pub active_wars_with: u32,
    /// `home_reg`: the capital's region.
    pub home_reg: i32,
    /// `explored`, `check_explore`'s recount.
    pub explored: i32,
    /// `escrow_rate[6]` — 40 each once the leader holds more than two
    /// cities and villages.
    pub escrow_rate: [i32; RESOURCES],
    /// `LeaderData::village_num` (+0x3fc): villages are not a settlement
    /// kind the simulation founds, so this stays 0 (run8 shows it 0 with a
    /// Small City standing).
    pub village_num: i32,
    /// `LeaderData::wonder_mark` (+0x424): one past the highest entry of
    /// the leader's wonder list in use — written by
    /// `Wonders::init_wonder@0073c860` when a wonder activates and walked
    /// back by `close_wonder@0073c7e0` when one closes
    /// ([`Sim::note_wonders`], `docs/AI.md` §75).
    ///
    /// [`Sim::note_wonders`]: crate::Sim::note_wonders
    pub wonder_mark: i32,
    /// The leader's wonder list (`wonders.list[who]`), by building index:
    /// an entry is the wonder it was written for, or `None` once closed.
    /// Entries at and above [`Census::wonder_mark`] are all `None`.
    pub wonder_slots: Vec<Option<usize>>,
    // Per region, one slot per sim region.
    pub reg_active: Vec<i32>,
    pub reg_combat: Vec<i32>,
    pub reg_attack: Vec<i32>,
    pub reg_naval: Vec<i32>,
    pub reg_transports: Vec<i32>,
    pub reg_defense: Vec<i32>,
    pub reg_attacked: Vec<i32>,
    pub reg_land: Vec<i32>,
    pub reg_peasants: Vec<i32>,
    pub reg_free_peasants: Vec<i32>,
    pub reg_xport_peasants: Vec<i32>,
    pub reg_gatherers: Vec<i32>,
    pub reg_gather_slots: Vec<i32>,
    pub reg_known_rares: Vec<i32>,
    pub reg_unpack_merch: Vec<i32>,
    /// `reg_cities[r]`: the sum of `reg_buildings[r]` over the four city
    /// types, recounted each sweep.
    pub reg_cities: Vec<i32>,
    /// `reg_pop[r]`: kept by the building lifecycle (`gain_/lose_building`).
    pub reg_pop: Vec<i32>,
    /// `reg_docks[r]`: kept by `Dock::init` / `Dock::close`
    /// (`docs/TRANSPORT.md` §5.2–§5.3), never zeroed by the sweep.
    pub reg_docks: Vec<i32>,
    /// `strategy[r]`: bit 1 thin, 2 stronger, 4 weaker, 8 expand.
    pub strategy: Vec<i32>,
    pub reg_wars: Vec<i32>,
    pub reg_allies: Vec<i32>,
    pub reg_neutrals: Vec<i32>,
}

impl Census {
    /// Sizes every per-region array to `regions` slots and `invaders` to
    /// `players`, keeping what is already there.
    pub fn resize(&mut self, regions: usize, players: usize) {
        for v in [
            &mut self.reg_active,
            &mut self.reg_combat,
            &mut self.reg_attack,
            &mut self.reg_naval,
            &mut self.reg_transports,
            &mut self.reg_defense,
            &mut self.reg_attacked,
            &mut self.reg_land,
            &mut self.reg_peasants,
            &mut self.reg_free_peasants,
            &mut self.reg_xport_peasants,
            &mut self.reg_gatherers,
            &mut self.reg_gather_slots,
            &mut self.reg_known_rares,
            &mut self.reg_unpack_merch,
            &mut self.reg_cities,
            &mut self.reg_pop,
            &mut self.reg_docks,
            &mut self.strategy,
            &mut self.reg_wars,
            &mut self.reg_allies,
            &mut self.reg_neutrals,
        ] {
            v.resize(regions, 0);
        }
        self.invaders.resize(players, 0);
    }

    /// A per-region slot, 0 off the table.
    pub fn reg(v: &[i32], r: u16) -> i32 {
        v.get(r as usize).copied().unwrap_or(0)
    }
}

/// The per-leader AI state the driver owns — the `LeaderData` fields
/// `production_step`, `prod_script_run`, `script_step`, `pers`,
/// `make_list`, the goods picture, and the cadence's `ai_speed`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Leader {
    pub step: Step,
    /// `prod_script_run`: the opening script is live.
    pub script_live: bool,
    /// `prod_script`: the script function's name — `"economic"` or
    /// `"defensive"` — chosen by [`Leader::choose_script`]; `None` for a
    /// leader that runs none (a human, or before `Leader::init`).
    pub script: Option<String>,
    /// `script_step`: the `ref step` the script advances. `Leader::init`
    /// sets 1.
    pub script_step: i32,
    pub pers: Personality,
    pub make_list: MakeList,
    /// The unit offers `create_units` made this frame, in the order it
    /// made them — what the original's `MakeList::make_me` is handed
    /// inside its `create_units` bracket, so a `RON_LEADER_PROBE` trace
    /// can be put beside it offer for offer. Cleared at the top of
    /// `create_units`; not a state the make list reads.
    pub unit_offers: Vec<MakeObject>,
    /// The sim-frame `unit_offers` was recorded on — `create_units` is one
    /// step of the leader's cycle and does not run every frame, so a
    /// reader says which frame's offers it is looking at.
    pub unit_offers_frame: i64,
    /// `econ[6]`: bit 1 rate under 30, 2 under `lo`, 4 under `hi`, 8
    /// comfortable — `docs/AI.md` §2.5.
    pub econ: [i32; RESOURCES],
    pub worst_good: usize,
    pub best_good: usize,
    pub shortages: i32,
    /// `LeaderDataEncrypt::rate` — the AI's own per-good rate figure.
    pub rate: [i32; RESOURCES],
    /// `site_mark`, the sites sampler's rolling offset.
    pub site_mark: u32,
    /// `effective_pop = queued_units + control + 1`.
    pub effective_pop: i32,
    /// `wonder_mod` and the five `*_mod` biases, 0x100 = ×1.
    pub wonder_mod: i32,
    pub ground_mod: i32,
    pub air_mod: i32,
    pub sea_mod: i32,
    pub infra_mod: i32,
    pub defense_mod: i32,
    /// The census — `docs/AI.md` §2.3.
    pub census: Census,
    /// The per-city AI records, indexed like `Sim::cities`; grown by the
    /// census.
    pub city_ai: Vec<CityAi>,
    /// The ten city sites — `docs/AI.md` §2.7.
    pub sites: [Site; SITES],
    /// `mil_trainers`: the leader's military trainers, by building index.
    pub mil_trainers: Vec<usize>,
    /// `tech_frame`, `tech_cat_frame[4]`: written only by `Leader::init`.
    pub tech_frame: i64,
    pub tech_cat_frame: [i64; 4],
    /// `frame_attacked`, `attacked_by`: the frame an enemy army last took
    /// a target of this leader's, and whose (`docs/ARMY.md` §12) — and
    /// `frame_attacked` alone also the frame any of this leader's objects
    /// last took a combat hit, at difficulty below 2
    /// (`Object::take_damage`, `docs/AI.md` §71).
    pub frame_attacked: i64,
    pub attacked_by: i32,
    /// `new_rares` (`LeaderData +0x6e6c`) — the **goods-list indices** of
    /// every merchant rare this leader has ever had a fog cell lifted over,
    /// in the order they were seen and never removed.
    /// `Leader::new_rare@006d9e70` is the only writer and
    /// `ScenarioFuncSet::num_rare_resources_seen@009ea010` returns its
    /// length: `docs/ECONOMY.md`, "The rares a leader has seen".
    pub new_rares: Vec<usize>,
    /// `known_rares` (`LeaderData +0x6d4`): `reg_known_rares` summed.
    /// `Leader::calc_gather@006ceee0` is its one writer and writes it
    /// under its own cadence, so it lags the census by up to a recompute;
    /// `create_units`' merchant arm is its one reader (`docs/AI.md` §55).
    pub known_rares: i32,
}

impl Default for Leader {
    fn default() -> Leader {
        Leader::new()
    }
}

impl Leader {
    /// `Leader::init`'s values for these fields.
    pub const fn new() -> Leader {
        Leader {
            step: Step::Idle,
            script_live: false,
            script: None,
            script_step: 1,
            pers: Personality {
                rush: 0,
                cities: 0,
                upgrades: 0,
                arms: 0,
                army: 0,
                army_size: 0,
                raid: 0,
                invade: 0,
                target: 0,
                strategy: 0,
                raze: 0,
                spells: 0,
                forts: 0,
                nukes: 0,
                air: 0,
                naval: 0,
                market: 0,
                scouts: 0,
                civilians: 0,
                early_army: 0,
                friendly_human: 0,
                alliance_human: 0,
                friendly_ai: 0,
                alliance_ai: 0,
            },
            make_list: MakeList::new(),
            unit_offers: Vec::new(),
            unit_offers_frame: -1,
            econ: [0; RESOURCES],
            worst_good: 0,
            best_good: 0,
            shortages: 0,
            rate: [0; RESOURCES],
            site_mark: 0,
            effective_pop: 0,
            wonder_mod: 0,
            ground_mod: 0x100,
            air_mod: 0x100,
            sea_mod: 0x100,
            infra_mod: 0x100,
            defense_mod: 0x100,
            census: Census {
                active: 0,
                combat: 0,
                non_siege: 0,
                sea_combat: 0,
                siege: 0,
                defense: 0,
                attack: 0,
                naval: 0,
                air: 0,
                missile: 0,
                transports: 0,
                fishermen: 0,
                idle_fishermen: 0,
                peasants: 0,
                scholars: 0,
                caras: 0,
                merchants: 0,
                fighters: 0,
                bombers: 0,
                cruise: 0,
                nuke: 0,
                free_peasants: 0,
                xport_peasants: 0,
                gatherers: 0,
                attacked: 0,
                full_cities: 0,
                pop: 0,
                scouts: 0,
                my_team_terr: 0,
                other_team_terr: 0,
                min_other_team_terr: 0,
                filled_gather_slots: [0; RESOURCES],
                gather_slots: [0; RESOURCES],
                ally_mask: 0,
                invaders: Vec::new(),
                gather_slots_high: [0; RESOURCES],
                peasant_high: 0,
                scholar_high: 0,
                caravan_high: 0,
                merchant_high: 0,
                army_high: 0,
                city_high: 0,
                village_high: 0,
                population_high: 0,
                city_pop_high: 0,
                resources_controlled: 0,
                wars: 0,
                allies: 0,
                active_wars: 0,
                active_wars_with: 0,
                home_reg: -1,
                explored: 0,
                escrow_rate: [0; RESOURCES],
                village_num: 0,
                wonder_mark: 0,
                wonder_slots: Vec::new(),
                reg_active: Vec::new(),
                reg_combat: Vec::new(),
                reg_attack: Vec::new(),
                reg_naval: Vec::new(),
                reg_transports: Vec::new(),
                reg_defense: Vec::new(),
                reg_attacked: Vec::new(),
                reg_land: Vec::new(),
                reg_peasants: Vec::new(),
                reg_free_peasants: Vec::new(),
                reg_xport_peasants: Vec::new(),
                reg_gatherers: Vec::new(),
                reg_gather_slots: Vec::new(),
                reg_known_rares: Vec::new(),
                reg_unpack_merch: Vec::new(),
                reg_cities: Vec::new(),
                reg_pop: Vec::new(),
                reg_docks: Vec::new(),
                strategy: Vec::new(),
                reg_wars: Vec::new(),
                reg_allies: Vec::new(),
                reg_neutrals: Vec::new(),
            },
            city_ai: Vec::new(),
            sites: [Site {
                wx: 0,
                wy: 0,
                val: 0,
                reg: 0,
                dist: 0,
                rank: 0,
            }; SITES],
            mil_trainers: Vec::new(),
            tech_frame: 0,
            frame_attacked: 0,
            attacked_by: -1,
            tech_cat_frame: [0; 4],
            new_rares: Vec::new(),
            known_rares: 0,
        }
    }

    /// The step the machine takes on entering `production_ai`, before the
    /// switch: a live script is skipped straight to `Setup` when there is
    /// none, or under the `starting_resources == 8` lobby.
    pub fn enter(&mut self, resources_unlimited: bool) {
        if self.step == Step::Script && (!self.script_live || resources_unlimited) {
            self.step = Step::Setup;
        }
    }

    /// The `Script` step's outcome — `docs/AI.md` §2.4, step 1. Returns
    /// whether the machine disarmed (`BLOCK_ON_THIS`).
    pub fn after_script(&mut self, r: ScriptResult) -> bool {
        match r {
            ScriptResult::Returned(1) => {
                self.step = Step::Idle;
                return true;
            }
            ScriptResult::Returned(3) | ScriptResult::Failed => {
                self.script_live = false;
            }
            ScriptResult::Returned(_) => {}
        }
        self.step = self.step.next();
        false
    }

    /// A producer step (`Cities`, `Research`, `Upgrades`, `Units`,
    /// `Buildings`, `Units2`) done: advance.
    pub fn after_producer(&mut self) {
        self.step = self.step.next();
    }

    /// The `Make` step's outcome: `make_stuff` bought the head → stay armed
    /// for the second pass; otherwise disarm, unless the unlimited lobby.
    pub fn after_make(&mut self, bought: bool, resources_unlimited: bool) {
        self.step = Step::Units2;
        if bought || resources_unlimited {
            return;
        }
        self.step = Step::Idle;
    }

    /// `Buildings2` done → `Make2`; `Make2` done → idle.
    pub fn after_second_pass(&mut self) {
        self.step = match self.step {
            Step::Buildings2 => Step::Make2,
            _ => Step::Idle,
        };
    }

    /// `Leader::init@006e3930` lines 782–826, the computer-leader tail
    /// after `random_personality`: the lobby's per-player flags force the
    /// roll (`+0x6dd4 rush`, `+0x6dec raid`, `+0x6e1c civilians`: `0x1000`
    /// → `rush = raid = 1`; `0x2000` → `rush = raid = −1`, `civilians = 1`;
    /// team style 3's team 1 → `rush = raid = 1`), then the script —
    /// `economic` when `rush < 0`, or `rush == 0` and a
    /// `Random::get(0, 0xffff)` coin comes up odd, and the nation lacks
    /// bonus `0x13` (the Lakota); `defensive` otherwise, with
    /// `early_army = 1`. `prod_script_run = 1` either way.
    pub fn choose_script(
        &mut self,
        rng: &mut Rng,
        player_flags: u32,
        team_style: i32,
        team: i32,
        lakota: bool,
    ) {
        if player_flags & 0x1000 != 0 {
            self.pers.rush = 1;
            self.pers.raid = 1;
        } else if player_flags & 0x2000 != 0 {
            self.pers.rush = -1;
            self.pers.raid = -1;
            self.pers.civilians = 1;
        } else if team_style == 3 && team == 1 {
            self.pers.rush = 1;
            self.pers.raid = 1;
        }
        self.script_live = true;
        let rush = self.pers.rush;
        let economic = (rush < 0 || (rush == 0 && rng.roll() & 1 != 0)) && !lakota;
        if economic {
            self.script = Some("economic".to_string());
        } else {
            self.script = Some("defensive".to_string());
            self.pers.early_army = 1;
        }
    }
}

/// The cadence — `plan_strategy`'s head, `docs/AI.md` §2.2.
pub mod cadence {
    /// `(who × 25 + frame) % (200 / ai_speed)`.
    pub fn phase(who: usize, frame: i64, ai_speed: i32) -> i64 {
        let period = i64::from(200 / ai_speed.max(1));
        (who as i64 * 25 + frame) % period
    }

    /// Whether this frame is a full sweep: frame 0, or the leader's phase.
    pub fn sweep_due(who: usize, frame: i64, ai_speed: i32) -> bool {
        frame == 0 || phase(who, frame, ai_speed) == 0
    }

    /// Whether this frame is the cheap research tick — every 30 phase
    /// frames between sweeps.
    pub fn research_tick_due(who: usize, frame: i64, ai_speed: i32) -> bool {
        !sweep_due(who, frame, ai_speed) && phase(who, frame, ai_speed) % 30 == 0
    }

    /// `check_explore`'s cadence: frame 0, or `(who × 25 + frame + 12)`
    /// on the period.
    pub fn explore_due(who: usize, frame: i64, ai_speed: i32) -> bool {
        frame == 0 || phase(who, frame + 12, ai_speed) == 0
    }
}

/// The lobby — `GameInfo`'s option block (`docs/RECGAME.md` §"the lobby
/// options", the dump's `GAMEINFO` fields) as the AI and its host functions
/// read it. The values are the file's own encodings; the accessors give the
/// meanings the readers need. Defaults are run6/run7's lobby
/// (`docs/INPUT.md` §2): Easiest, Small Town, Ancient, a Great Lakes map, no
/// rush rules, standard victory, capital-countdown elimination.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lobby {
    pub team_style: i32,
    /// `MAP_STYLE`: the index into `map_styles` — `world+0x30`, the map
    /// type `compute_site_stats` tests for `0xc` and `0x11`.
    pub map_style: i32,
    /// The style's name, as `get_mapstyle()` returns it and the scripts
    /// compare it: rules.xml's `mapstyles` category in file order (0
    /// "Random" … 14 "Great Lakes" … 22 "British Isles").
    pub map_style_name: String,
    /// `DIFFICULTY`: 0 Easiest … 5 Toughest. `get_diff()` in a solo game.
    pub difficulty: i32,
    /// `STARTING_TOWN`: 0 nomad, 1 Small, 2 Small Town, 3 Large Town …
    pub starting_town: i32,
    /// `Game::start_list` (`+0x65c`), which the Game record prints as eight
    /// `start_list` lines: a permutation of the eight seats the game
    /// shuffles at start (`1 0 3 2 5 4 7 6`, `4 7 6 1 0 3 2 5`, … — 29 kept
    /// dumps). `start_index` is its inverse. Only `team_style == 2` reads
    /// it ([`crate::Sim::start_list_target`]); the identity is the default.
    pub start_list: [usize; 8],
    /// `STARTING_RESOURCES`: the row; 8 is the unlimited-style lobby that
    /// skips the script and buys after every step.
    pub starting_resources: i32,
    /// `STARTING_RESOURCES2`: team 0's under `GAME_RULES == 8`.
    pub starting_resources2: i32,
    pub game_rules: i32,
    pub tech_cost: i32,
    /// `RUSH_RULES`: 0 off; 8 forces `pers.raid = −1`.
    pub rush_rules: i32,
    /// `ELIMINATION`: 1 is the capital countdown.
    pub elimination: i32,
    /// `VICTORY`: 0 standard, 3 score, 5 musical chairs, 6 wonder, 7
    /// territory, 8 economic, 9 tech race (`host-functions.md` §5).
    pub victory: i32,
    /// The wonder victory's points: rules.xml's `wonderwins` row that
    /// `WONDERWIN` indexes, its `DATA` (`Category.data[0]`, `+0x3c`).
    /// `Game::wonder_winning` and `create_buildings`' wonder arm compare
    /// against it whatever `VICTORY` is. Every capture on file sets
    /// `WONDERWIN 5`, the "8 Wonder Points" row (`docs/AI.md` §124).
    pub wonder_win_points: i32,
    /// `GameInfo.flags & 4`, "No Nation Powers".
    pub no_nation_powers: bool,
    /// `semaphore[2] & 2`: a Conquer-the-World or scenario game.
    pub conquest: bool,
    /// `REVEAL_MAP`: 1 is the lobby's "Normal" (every run so far — the
    /// profile's value, whatever `check.ini` said); `> 1` makes
    /// `WorldData::was_seen` answer true everywhere.
    pub reveal_map: i32,
}

impl Default for Lobby {
    fn default() -> Lobby {
        Lobby {
            team_style: 1,
            map_style: 14,
            map_style_name: "Great Lakes".to_string(),
            difficulty: 0,
            starting_town: 2,
            start_list: [0, 1, 2, 3, 4, 5, 6, 7],
            starting_resources: 1,
            starting_resources2: 1,
            game_rules: 1,
            tech_cost: 3,
            rush_rules: 0,
            elimination: 1,
            victory: 0,
            wonder_win_points: 8,
            no_nation_powers: false,
            conquest: false,
            reveal_map: 1,
        }
    }
}

impl Lobby {
    /// `starting_resources == 8`.
    pub fn resources_unlimited(&self) -> bool {
        self.starting_resources == 8
    }

    /// `get_starting_resources(who)`: the asymmetric-teams row for team 0
    /// under `GAME_RULES == 8`, else the ordinary one.
    pub fn starting_resources_for(&self, team: i32) -> i32 {
        if self.game_rules == 8 && team == 0 {
            self.starting_resources2
        } else {
            self.starting_resources
        }
    }
}

/// The inputs `production_ai_setup` reads besides the ledger.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GoodsSetup {
    pub city_num: i32,
    /// `get_diff()`.
    pub difficulty: i32,
    /// `starting_resources == 8`.
    pub resources_unlimited: bool,
    /// `has_tribe_bonus(0x13)` — the Lakota.
    pub lakota: bool,
    /// The next age's price per good, `get_cost(age_type, g, …)`, when the
    /// current age is an age type; `None` otherwise.
    pub next_age_cost: Option<[i32; RESOURCES]>,
    /// `type_avail(g, 1) != 0` per good.
    pub available: [bool; RESOURCES],
    /// `get_mod_resource_cap(g)` per good, in sixteenths.
    pub cap: [i32; RESOURCES],
}

/// `production_ai_setup@006c83e0`, read whole — `docs/AI.md` §2.5 — up to
/// and excluding `market_speculation`. Writes the leader's `econ`, `rate`,
/// `worst_good`, `best_good`, `shortages`, and on difficulties below 3
/// clamps the ledger's stockpile.
pub fn goods_picture(leader: &mut Leader, ledger: &mut Ledger, s: &GoodsSetup) {
    leader.shortages = 0;
    leader.worst_good = 0;
    leader.best_good = 0;
    if s.resources_unlimited {
        for e in &mut leader.econ {
            *e |= 8;
        }
        return;
    }
    // 2. The easy-difficulty stockpile clamp.
    if s.difficulty < 3 {
        let mut m = match s.next_age_cost {
            Some(cost) => {
                let mut m = 0;
                for (&avail, &c) in s.available.iter().zip(cost.iter()) {
                    if avail && c > m {
                        m = c;
                    }
                }
                let m = m.max(300);
                match s.difficulty {
                    0 => m * 3 / 2,
                    1 => m * 2,
                    _ => m * 5 / 2,
                }
            }
            None => 11000,
        };
        if m < 0 {
            m = 0;
        }
        if m != 0 {
            for g in 0..RESOURCES {
                let bucket = ledger.bucket[g];
                if m < bucket {
                    ledger.escrow[g] = ledger.escrow[g] * m / bucket;
                    ledger.bucket[g] = m;
                }
            }
        }
    }
    // 3. The rate pass.
    let mut lowest = 99_999_999;
    let mut highest = -99_999_999;
    for g in 0..RESOURCES {
        leader.econ[g] = 0;
        let r = s.cap[g].min(ledger.income[g]);
        leader.rate[g] = r / 16;
        if s.available[g] {
            let rate = leader.rate[g];
            if rate < lowest {
                leader.worst_good = g;
                lowest = rate;
            }
            if rate > highest {
                leader.best_good = g;
                highest = rate;
            }
            if rate < 30 {
                leader.econ[g] |= 1;
                leader.shortages += 1;
            }
        }
    }
    // 4. The threshold pass.
    for g in 0..RESOURCES {
        if !s.available[g] {
            leader.econ[g] = 0;
            continue;
        }
        let cap = s.cap[g];
        let mut lo = (s.city_num * 15).min(cap / 32);
        let mut hi = (s.city_num * 30).min(cap / 16 * 4 / 5);
        if s.city_num > 4 {
            lo = cap / 32;
            hi = (cap / 16 * 3 / 4).min(175);
        }
        lo = lo.min(250);
        hi = hi.min(350);
        if g >= 2 && s.city_num < 3 {
            lo = 20;
            hi = s.city_num * 20;
            if leader.econ[g] & 1 != 0 {
                leader.econ[g] &= !1;
                leader.shortages -= 1;
            }
        }
        if ledger.income[g] < cap {
            if leader.rate[g] < lo {
                leader.econ[g] |= 2;
            }
            if leader.rate[g] < hi {
                leader.econ[g] |= 4;
                continue;
            }
        }
        leader.econ[g] |= 8;
    }
    // 5. The two-city food/wood balance.
    if s.city_num > 2 {
        return;
    }
    let clear_rest = |econ: &mut [i32; RESOURCES], bit: i32| {
        for e in econ.iter_mut().skip(2) {
            *e &= !bit;
        }
    };
    if leader.econ[0] & 4 == 0 || s.lakota {
        if leader.econ[1] & 4 != 0 {
            leader.econ[0] &= !4;
            clear_rest(&mut leader.econ, 4);
        }
    } else {
        leader.econ[1] &= !4;
        clear_rest(&mut leader.econ, 4);
    }
    if leader.econ[0] & 2 == 0 || s.lakota {
        if leader.econ[1] & 2 == 0 {
            return;
        }
        if leader.econ[0] & 2 != 0 {
            leader.econ[0] = (leader.econ[0] & !2) | 4;
        }
    } else if leader.econ[1] & 2 != 0 {
        leader.econ[1] = (leader.econ[1] & !2) | 4;
    }
    for e in leader.econ.iter_mut().skip(2) {
        if *e & 2 != 0 {
            *e = (*e & !2) | 4;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Two offers of one type at one value are not one offer**, and the
    /// tie splits the ranked copy from the category copy — item 432.
    ///
    /// `make_me` ranks on `list[k].val <= val` and fills the category slot
    /// on `list[cat].val < val` (`006c9be0`, both senses read off the
    /// decompile). The asymmetry is invisible while every offer has its own
    /// number and decides a purchase the moment two share one: the later
    /// offer **displaces** the earlier in the ranked list and **does not**
    /// in the category slot, so slot 1 and slot `cat` end up naming
    /// different cities for the same type.
    ///
    /// `make_stuff`'s step 6 then walks slot `cat` with the duplicate test
    /// `t == list[k].t && city == list[k].city` over slots 0..4
    /// (`006c8af0:147–167`), finds no match, and **buys the same type a
    /// second time** — one extra `make_this`, one extra `expire` call, one
    /// extra `Leader::make_stuff+0x63d` draw.
    ///
    /// That is what happens on Great Lakes 9382 the moment this crate's
    /// Scholar is given a value it can win with: `create_units` offers the
    /// Scholar once per city and this crate computes the **same** number for
    /// both, where the original computes two (`docs/AI.md` §50). The second
    /// half of the test is the original's shape — distinct values — and
    /// there the category slot follows the better offer and the duplicate
    /// test fires.
    ///
    /// Made to fail on purpose by weakening the ranked insert to `<`:
    /// the tie then does not displace, both copies name city 1, and the
    /// first `assert!` below stops it. The test is about the two
    /// comparison senses, not about the numbers.
    #[test]
    fn a_tie_splits_the_ranked_copy_from_the_category_copy() {
        /// `make_stuff` step 6's duplicate test, verbatim: does slot `at`
        /// repeat a `(t, city)` already among the ranked four?
        fn deduped(l: &MakeList, at: usize) -> bool {
            let m = l.list[at];
            (0..4).any(|k| l.list[k].t == m.t && l.list[k].city == m.city)
        }
        // A head far above both offers, so neither can take slot 0 — the
        // Great Lakes shape, where `t573` stands at 9,999,999.
        let head = |l: &mut MakeList| l.make_me(573, 9_999_999, 1, 10, -1, 0, 1, 0, 0);

        // One value for both cities: this crate's own 45,568 twice.
        let mut tied = MakeList::new();
        head(&mut tied);
        tied.make_me(52, 45_568, 1, 4, 1, 0, 1, 0, 0);
        tied.make_me(52, 45_568, 1, 4, 2, 0, 1, 0, 0);
        assert_eq!((tied.list[1].t, tied.list[1].city), (52, 2));
        assert_eq!((tied.list[4].t, tied.list[4].city), (52, 1));
        // The earlier ranked copy is cleared to `t −1` and keeps its
        // `val` and `city`, which is how a dump tells the two clears apart.
        assert_eq!(
            (tied.list[2].t, tied.list[2].val, tied.list[2].city),
            (-1, 45_568, 1)
        );
        assert!(
            !deduped(&tied, 4),
            "the tie must defeat step 6's duplicate test — that is the              second purchase, and the extra expiry draw with it"
        );

        // The original's shape on the same frame: one value per city,
        // 4,891,136 for its first and 5,755,741 for its second, in that
        // order (run111 block 9381 is what says both the values and the
        // order — `rondata::diff`'s reconstruction of it).
        let mut apart = MakeList::new();
        head(&mut apart);
        apart.make_me(52, 4_891_136, 1, 4, 1, 0, 1, 0, 0);
        apart.make_me(52, 5_755_741, 1, 4, 2, 0, 1, 0, 0);
        assert_eq!((apart.list[1].t, apart.list[1].city), (52, 2));
        assert_eq!((apart.list[4].t, apart.list[4].city), (52, 2));
        assert_eq!(
            (apart.list[2].t, apart.list[2].val, apart.list[2].city),
            (-1, 4_891_136, 1)
        );
        assert!(
            deduped(&apart, 4),
            "with the offers apart the category slot follows the better one              and step 6 skips it: one purchase, two expiry draws"
        );
    }

    fn draws(seed: u32, f: impl FnOnce(&mut Rng)) -> usize {
        let mut a = Rng::new(seed);
        let mut n = 0;
        let mut probe = a;
        f(&mut a);
        while probe != a {
            probe.roll();
            n += 1;
            assert!(n < 100, "diverged");
        }
        n
    }

    #[test]
    fn a_nubian_alone_takes_twenty_or_twenty_one_draws() {
        // Twenty fixed draws — rush, cities, the arms coin and its roll,
        // army, army_size, raid, invade, target, strategy, spells, forts,
        // nukes, air, naval, market, scouts, civilians, friendly, alliance —
        // and a rush roll of 1 costs a Nubian one more.
        let setup = RollSetup {
            sea_map: 1,
            rush_rules_off: false,
        };
        let mut seen = [false; 2];
        for seed in 0..40u32 {
            let n = draws(seed, |r| {
                Personality::roll(r, tribe::NUBIANS, &[], &setup);
            });
            assert!(n == 20 || n == 21, "seed {seed}: {n} draws");
            seen[n - 20] = true;
        }
        assert_eq!(seen, [true, true]);
    }

    #[test]
    fn the_romans_always_want_forts_and_the_germans_air() {
        let setup = RollSetup {
            sea_map: 1,
            rush_rules_off: false,
        };
        for seed in 0..20u32 {
            let p = Personality::roll(&mut Rng::new(seed), tribe::ROMANS, &[], &setup);
            assert_eq!(p.forts, 1);
            let p = Personality::roll(&mut Rng::new(seed), tribe::GERMANS, &[], &setup);
            assert_eq!(p.air, 1);
            assert_eq!(p.friendly_ai, p.friendly_human);
            assert_eq!(p.alliance_ai, p.alliance_human);
            assert_eq!(p.raze, 0);
        }
    }

    #[test]
    fn rush_rules_off_pins_raid() {
        let setup = RollSetup {
            sea_map: 1,
            rush_rules_off: true,
        };
        let p = Personality::roll(&mut Rng::new(3), tribe::AZTECS, &[], &setup);
        assert_eq!(p.raid, -1);
    }

    #[test]
    fn make_me_head_overwrites_and_does_not_shift() {
        let mut m = MakeList::new();
        m.make_me(10, 100, 1, 9, -1, 0, 1, 0, 0);
        m.make_me(11, 200, 1, 8, -1, 0, 1, 0, 0);
        assert_eq!(m.list[0].t, 11);
        assert_eq!(m.list[1].t, -1, "the old head is gone, not demoted");
        assert_eq!(m.list[9].t, 10, "the category slot keeps it");
        assert_eq!(m.list[8].t, 11);
    }

    #[test]
    fn make_me_ranks_runners_up_and_deduplicates() {
        let mut m = MakeList::new();
        m.make_me(1, 500, 1, 4, -1, 0, 1, 0, 0);
        m.make_me(2, 300, 1, 5, -1, 0, 1, 0, 0);
        m.make_me(3, 400, 1, 6, -1, 0, 1, 0, 0);
        m.make_me(4, 100, 1, 7, -1, 0, 1, 0, 0);
        assert_eq!(
            [m.list[0].t, m.list[1].t, m.list[2].t, m.list[3].t],
            [1, 3, 2, 4]
        );
        // A better offer of type 2 displaces its own lower entry.
        m.make_me(2, 450, 1, 5, -1, 0, 1, 0, 0);
        assert_eq!(
            [m.list[0].t, m.list[1].t, m.list[2].t, m.list[3].t],
            [1, 2, 3, -1]
        );
        assert_eq!(m.list[5].val, 450);
        // A worse offer of a type already ranked is dropped.
        m.make_me(3, 350, 1, 6, -1, 0, 1, 0, 0);
        assert_eq!(
            [m.list[0].t, m.list[1].t, m.list[2].t, m.list[3].t],
            [1, 2, 3, -1]
        );
    }

    #[test]
    fn the_sweep_is_frame_zero_then_every_two_hundred_on_the_phase() {
        let due: Vec<i64> = (0..800).filter(|&f| cadence::sweep_due(1, f, 1)).collect();
        assert_eq!(due, vec![0, 175, 375, 575, 775]);
        let due: Vec<i64> = (0..300).filter(|&f| cadence::sweep_due(0, f, 1)).collect();
        assert_eq!(due, vec![0, 200]);
        assert!(cadence::research_tick_due(1, 205, 1));
        assert!(!cadence::research_tick_due(1, 206, 1));
        assert!(cadence::explore_due(1, 163, 1));
    }

    #[test]
    fn the_step_machine_follows_the_script_s_answer() {
        let mut l = Leader::new();
        l.script_live = true;
        l.step = Step::Script;
        l.enter(false);
        assert_eq!(l.step, Step::Script);
        assert!(l.after_script(ScriptResult::Returned(1)));
        assert_eq!(l.step, Step::Idle);
        assert!(l.script_live);

        l.step = Step::Script;
        assert!(!l.after_script(ScriptResult::Returned(3)));
        assert_eq!(l.step, Step::Setup);
        assert!(!l.script_live, "SCRIPT_DONE drops the script");

        l.step = Step::Script;
        l.enter(false);
        assert_eq!(l.step, Step::Setup, "a dead script is skipped");

        l.step = Step::Script;
        l.script_live = true;
        assert!(!l.after_script(ScriptResult::Failed));
        assert!(!l.script_live);
        assert_eq!(l.step, Step::Setup);

        l.step = Step::Make;
        l.after_make(false, false);
        assert_eq!(l.step, Step::Idle);
        l.step = Step::Make;
        l.after_make(true, false);
        assert_eq!(l.step, Step::Units2);
        l.after_producer();
        assert_eq!(l.step, Step::Buildings2);
        l.after_second_pass();
        assert_eq!(l.step, Step::Make2);
        l.after_second_pass();
        assert_eq!(l.step, Step::Idle);
    }

    #[test]
    fn the_goods_picture_is_about_the_rate() {
        let mut l = Leader::new();
        let mut ledger = Ledger {
            income: [40 * 16, 10 * 16, 0, 0, 0, 0],
            bucket: [5000, 100, 0, 0, 0, 0],
            ..Ledger::default()
        };
        let s = GoodsSetup {
            city_num: 1,
            difficulty: 3,
            resources_unlimited: false,
            lakota: false,
            next_age_cost: None,
            available: [true, true, false, false, false, false],
            cap: [1000 * 16; RESOURCES],
        };
        goods_picture(&mut l, &mut ledger, &s);
        assert_eq!(l.rate[0], 40);
        assert_eq!(l.rate[1], 10);
        assert_eq!(l.shortages, 1, "wood under 30");
        assert_eq!(l.worst_good, 1);
        assert_eq!(l.best_good, 0);
        // lo = min(15, 500, 250) = 15, hi = min(30, 800, 350) = 30: food is
        // at 40 → comfortable; wood at 10 → under both.
        assert_eq!(l.econ[0], 8);
        assert_eq!(l.econ[1], 1 | 2 | 4);
        assert_eq!(l.econ[2], 0, "unavailable");
        assert_eq!(ledger.bucket[0], 5000, "no clamp at difficulty 3");
    }

    #[test]
    fn an_easy_ai_throws_its_stockpile_away() {
        let mut l = Leader::new();
        let mut ledger = Ledger {
            bucket: [5000, 5000, 0, 0, 0, 0],
            escrow: [1000, 0, 0, 0, 0, 0],
            ..Ledger::default()
        };
        let s = GoodsSetup {
            city_num: 1,
            difficulty: 0,
            resources_unlimited: false,
            lakota: false,
            next_age_cost: Some([200, 100, 0, 0, 0, 0]),
            available: [true; RESOURCES],
            cap: [1000 * 16; RESOURCES],
        };
        goods_picture(&mut l, &mut ledger, &s);
        // m = max(200, 300) × 3/2 = 450.
        assert_eq!(ledger.bucket[0], 450);
        assert_eq!(ledger.escrow[0], 1000 * 450 / 5000);
        assert_eq!(ledger.bucket[1], 450);
    }
}
