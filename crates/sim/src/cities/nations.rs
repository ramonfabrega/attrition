//! The nation-power layer: a player's nation, and the flags it turns on.
//!
//! Every nation power in the original is one call —
//! `LeaderData::has_tribe_bonus(n)`, where `n` is the nation's index in the
//! roster `rules.xml`'s `TRIBES` block lists. [`TechTree::has_tribe_bonus`]
//! is that call, and it has been here since the tech tree was; what was
//! missing is the wire between it and the number the game actually carries.
//! `LeaderData::tribe` is dumped by `LEADERS=9` on every frame, and until
//! now nothing read it, so every traced game was played with **no nation
//! power on either side** — twenty-four of them inert at once.
//!
//! This module is that wire, and nothing else. [`Sim::set_tribe`] takes the
//! dump's own number; [`Sim::refresh_nation_powers`] recomputes the
//! per-nation booleans on [`city::Nation`] that the mechanics read. Those
//! booleans are a **cache of `has_tribe_bonus`, not a second source**: each
//! is that call's answer for one roster index, so the lobby's "No Nation
//! Powers" flag and the no-city gate apply here exactly as they do there.
//!
//! `docs/TECH.md` ("`has_preq`: are the prerequisites met") is where the
//! roster indices are established; `docs/ECONOMY.md` ("The commerce cap") is
//! the measurement that made the wire worth having.
//!
//! [`TechTree::has_tribe_bonus`]: crate::tech::TechTree::has_tribe_bonus
//! [`city::Nation`]: crate::cities::city::Nation

use crate::tech::TypeId;
use crate::{Player, Sim};

/// The `TypeIndex` values `Build::activate`'s free-unit block names,
/// defined in [`crate::ids::ty`].
pub use crate::ids::ty;

/// The roster indices `Build::activate`'s free-unit block passes
/// `has_tribe_bonus` — [`crate::ids::tribe`] under the name its readers use.
pub(crate) use crate::ids::tribe as power;

/// The nation roster, in the order `rules.xml`'s `TRIBES` block lists it.
///
/// The index is the identity everywhere: it is the `n` of
/// `has_tribe_bonus(n)`, the number `LEADERDATA` prints as `tribe`, and the
/// bit position `TRIBE_MASK` sets (`crate::tech::TypeDef::tribes`). The
/// names are each nation file's own `<TRIBE name="…">`, which is what
/// `ScenarioFuncSet::find_nation` matches — so a name here is a claim about
/// the user's install, and `cargo run -p rondata -- <install>` re-derives
/// the whole list from `rules.xml` and `tribes/` and fails if it has
/// drifted.
pub const ROSTER: [&str; 24] = [
    "Aztecs",
    "Maya",
    "Inca",
    "Bantu",
    "Nubians",
    "Greeks",
    "Romans",
    "Egyptians",
    "Turks",
    "Spanish",
    "French",
    "British",
    "Germans",
    "Russians",
    "Chinese",
    "Japanese",
    "Koreans",
    "Mongols",
    "Iroquois",
    "Lakota",
    "Americans",
    "Indians",
    "Dutch",
    "Persians",
];

/// The roster index of a nation, by the name [`ROSTER`] holds.
pub fn power_of(name: &str) -> Option<usize> {
    ROSTER.iter().position(|n| *n == name)
}

impl Sim {
    /// Sets a player's nation from the dump's own `LeaderData::tribe`, and
    /// refreshes the flags that follow from it.
    ///
    /// The original's `tribe` is a signed index and **−1 is a real value** —
    /// a leader with no nation, which `has_tribe_bonus` answers no to for
    /// every power. That is what `power: None` means here.
    /// [`crate::tech::PlayerTech::tribe`] is a different thing: it indexes
    /// [`crate::tech::TechTree::tribes`] for the unit-graft and unique-unit
    /// tables, so it stays inside the roster and a −1 leaves it at zero,
    /// whose graft table is identity.
    ///
    /// An index past the end of a *loaded* roster still counts as a power —
    /// the tree built by a unit test has one placeholder tribe in it, and a
    /// power is not a table lookup.
    pub fn set_tribe(&mut self, who: Player, tribe: i64) {
        let w = who as usize;
        let power = usize::try_from(tribe).ok();
        self.tech[w].power = power;
        self.tech[w].tribe = power
            .filter(|&i| i < self.tables.tech_tree.tribes.len())
            .unwrap_or(0);
        self.refresh_nation_powers(who);
        // `Leader::init`'s tail runs `calc_anti_attrition` once the tribe is
        // known (`docs/LEADERS.md` §6.2).
        self.calc_anti_attrition(who);
    }

    /// `city_num != 0` — the half of `has_tribe_bonus`'s city gate that
    /// moves (`LeaderData::has_tribe_bonus@006e1370`, group 22) — written
    /// where a city is founded or closed, and the cached flags redone when
    /// it flips.
    pub(crate) fn sync_has_city(&mut self, who: Player) {
        let has = self.city_num(who) != 0;
        let w = who as usize;
        if self.tech[w].has_city != has {
            self.tech[w].has_city = has;
            self.refresh_nation_powers(who);
        }
    }

    /// The lobby's two bits `has_tribe_bonus` reads, as the tech tree's
    /// [`crate::tech::Setup`] holds them — "No Nation Powers" and the
    /// starting town — and every player's city half brought in line.
    pub fn sync_setup_from_lobby(&mut self) {
        self.tables.setup.no_nation_powers = self.ai.lobby.no_nation_powers;
        self.tables.setup.starting_town = self.ai.lobby.starting_town != 0;
        // The age the game opens in, which `Leader::init`'s starting
        // position reads (`docs/TECH.md`, "The starting position"): every
        // lobby on file before item 1466 was Ancient, to the Information
        // age, so the default `Setup::STANDARD` was right until a lobby
        // said otherwise (run650..652).
        self.tables.setup.starting_age = self.ai.lobby.starting_technology;
        self.tables.setup.starting_age2 = self.ai.lobby.starting_technology2;
        self.tables.setup.ending = self.ai.lobby.ending_technology;
        for who in 0..self.tech.len() {
            self.sync_has_city(who as Player);
        }
        self.finalize_type_ages();
    }

    /// **`Types::finalize_grafting@00669840`'s age cache**: every unit and
    /// building type's `age` (`ObjectTypeData +0x278`) is `get_age_slow`
    /// in the game's own lobby ([`crate::tech::TechTree::age_in`]), so a
    /// lobby that shortens the age span moves the age of every type whose
    /// slot-1 prerequisite is a Military epoch. The loader's ages are the
    /// full game's, which is this in a lobby that starts Ancient and runs to
    /// Information. The combat table (`Balance`) is built at load from the
    /// loader's ages and is not rebuilt here (item 3589, `docs/AI.md` §315).
    pub fn finalize_type_ages(&mut self) {
        let (tree, setup) = (&self.tables.tech_tree, &self.tables.setup);
        for u in &mut self.tables.unit_types {
            if let Some(t) = u.tree {
                u.combat.age = tree.age_in(setup, t);
            }
        }
        for b in &mut self.tables.build_types {
            if let (Some(t), Some(c)) = (b.tree, b.combat.as_mut()) {
                c.age = tree.age_in(setup, t);
            }
        }
    }

    /// Recomputes one player's per-nation flags from their power.
    ///
    /// Call it after anything `has_tribe_bonus` reads changes — the power
    /// itself, `has_city`, or the lobby's "No Nation Powers". The seven
    /// nations with no flag on [`crate::cities::city::Nation`] — the Greeks, Spanish,
    /// Japanese, Mongols, Iroquois, Americans and Persians — are not missing:
    /// their powers are read straight off `has_tribe_bonus` at the rules that
    /// use them, and this cache exists only for the mechanics that took a
    /// boolean before the tree did.
    pub fn refresh_nation_powers(&mut self, who: Player) {
        let w = who as usize;
        let on: [bool; ROSTER.len()] = std::array::from_fn(|n| {
            self.tables
                .tech_tree
                .has_tribe_bonus(&self.tables.setup, &self.tech[w], n)
        });
        let n = &mut self.cities.nation[w];
        n.aztecs = on[0];
        n.maya = on[1];
        n.inca = on[2];
        n.bantu = on[3];
        n.nubians = on[4];
        n.romans = on[6];
        n.egyptians = on[7];
        n.turks = on[8];
        n.french = on[10];
        n.british = on[11];
        n.mongols = on[17];
        n.germans = on[12];
        n.russians = on[13];
        n.chinese = on[14];
        n.koreans = on[16];
        n.lakota = on[19];
        n.indians = on[21];
        n.dutch = on[22];
    }
}

/// The Modern age's place in [`crate::tech::TechTree::ages`], Classical
/// first: the sixth age tech, `MODERN_AGE` (`0x225`) in the original.
const MODERN: usize = 5;

impl Sim {
    /// **`Build::activate@00623e20:603–1150` — the high-water mark, and the
    /// nation's free units.**
    ///
    /// It is a **mark**, not a first-one flag: the British get archers with
    /// every Barracks, and what the mark buys is that a *replacement* for
    /// one that died gets none.
    ///
    /// `n = num_buildings[type] + get_buildings(to)` — the finished
    /// buildings of this record and of every record it upgrades to; when
    /// that beats `high_buildings[base_type]` the mark rises, and a
    /// building that is *counted* and did not arrive by capture pays the
    /// owner's nation its free units. The mark is per **lineage root**
    /// (`TypeData::get_base_type`, the type vtable's `+0xe4` — the slot
    /// `Unit::think_scout` hands `FILTER_BASE_TYPE`), so a Keep does not
    /// re-earn what its Tower already claimed.
    ///
    /// `docs/CITIES.md` §4.3.
    pub(crate) fn claim_building_high(&mut self, b: usize, captured: bool, counted: bool) {
        let Some(rec) = self.buildings[b].ty else {
            return;
        };
        let who = self.buildings[b].owner;
        let n = self.buildings_of_line(who, rec);
        let root = self.base_build_type(rec);
        let w = who as usize;
        // Gaia owns buildings in some fixtures and has no leader record.
        if w >= self.cities.building_high.len() {
            return;
        }
        if self.cities.building_high[w].len() <= root {
            self.cities.building_high[w].resize(root + 1, 0);
        }
        if self.cities.building_high[w][root] >= n || captured {
            return;
        }
        self.cities.building_high[w][root] = n;
        if counted {
            self.free_units_for(b, rec);
        }
    }

    /// `TypeData::get_base_type` on a building record: the root of its
    /// `FROM` chain.
    fn base_build_type(&self, rec: usize) -> usize {
        let mut cur = rec;
        let mut guard = 0;
        while let Some(x) = self.tables.build_types[cur].from {
            cur = x;
            guard += 1;
            if guard > 16 {
                break;
            }
        }
        cur
    }

    /// The else-if chain over the finished building's own `TypeIndex` —
    /// which is how the original spells it, so a Keep is not a Tower here
    /// and an Anchorage is named beside its Dock. Only the British arm has a
    /// diff behind it (`docs/CITIES.md` §4.3, Coverage).
    fn free_units_for(&mut self, b: usize, rec: usize) {
        let who = self.buildings[b].owner;
        let t = self.tables.build_types[rec].tree;
        if t == Some(ty::MARKET) {
            // The Nubian caravan. `NUBIAN_FREE_CARAVAN` ships as 0, so the
            // arm is live and inert at once.
            let n = i32::from(self.tables.tuning.nubian_free_caravan != 0);
            self.free_train(b, power::NUBIANS, ty::CARAVAN, n, true);
        } else if t == Some(ty::BARRACKS) {
            self.barracks_free_units(b, who);
        } else if t.is_some_and(|x| ty::SIEGE_FACTORIES.contains(&x)) {
            let siege = self.tables.tuning.turk_free_siege;
            self.free_train(b, power::TURKS, ty::CATAPULT, siege, true);
            // The French wagon and general are the two arms that skip
            // `current_upgrade`: the graft's answer is trained as it stands.
            let wagons = i32::from(self.tables.tuning.french_free_supply != 0);
            self.free_train(b, power::FRENCH, ty::SUPPLYWAGON, wagons, false);
        } else if t.is_some_and(|x| ty::DOCKS.contains(&x)) {
            // The Spanish trireme is gated on the age as well as the count:
            // `ages < 5`, so it stops at Gunpowder.
            let ships = if self.tech[who as usize].ages < 5 {
                self.tables.tuning.spanish_free_trireme
            } else {
                0
            };
            self.free_train(b, power::SPANISH, ty::TRIREME, ships, true);
            let barks = self.tables.tuning.dutch_free_light_ship;
            self.free_train(b, power::DUTCH, ty::BARK, barks, true);
            let fishers = self.tables.tuning.british_free_fishermen;
            self.free_train(b, power::BRITISH, ty::FISHERMEN, fishers, true);
        } else if t == Some(ty::UNIVERSITY) {
            // The third arm with no `current_upgrade`.
            let scholars = self.tables.tuning.americans_free_scholar;
            self.free_train(b, power::AMERICANS, ty::SCHOLARS, scholars, false);
        } else if crate::cities::build::is_fort(&self.tables.build_types, rec) {
            let generals = i32::from(self.tables.tuning.french_free_general != 0);
            self.free_train(b, power::FRENCH, ty::GENERAL, generals, false);
        } else if t == Some(ty::AIRBASE) {
            let fighters = self.tables.tuning.german_free_fighter;
            self.free_train(b, power::GERMANS, ty::BIPLANE, fighters, true);
            // The American bomber wants the Modern age as well as the base:
            // `has_tech(MODERN_AGE)`, the original's `0x225`. The tree lays
            // the techs out at `543 + i` where the original's `TypeIndex` is
            // `0x220 + i` (`rondata::load`'s table), so the age is asked for
            // by its place in the ladder, never by the original's number —
            // `0x225` here is the Information Age (item 3509).
            let modern = self.tables.tech_tree.ages[MODERN].is_some_and(|a| {
                self.tables
                    .tech_tree
                    .has_tech(&self.tables.setup, &self.tech[who as usize], a)
            });
            let bombers = if modern {
                self.tables.tuning.americans_free_bomber
            } else {
                0
            };
            self.free_train(b, power::AMERICANS, ty::BOMBER, bombers, true);
        } else if !self.building_is_city(b) {
            if t.is_some_and(|x| ty::STABLES.contains(&x)) {
                let epoch = self.tech[who as usize].epoch[0];
                let g = &self.tables.tuning;
                let mut horse = if epoch < 2 {
                    g.mongol_start_cavalry
                } else {
                    g.mongol_free_cavalry
                };
                if epoch > 2 && horse < g.mongol_three_mil_cavalry {
                    horse = g.mongol_three_mil_cavalry;
                }
                self.free_train(b, power::MONGOLS, ty::HORSEARCHERS, horse, true);
            }
        } else {
            // `KOREAN_CITIZENS[city_num − 1]`, clamped to the array — one
            // for the first city, three for the second, five after. The
            // count already includes the city being founded, which
            // `Build::activate` raised a few lines above this block.
            let i = (self.city_num(who) - 1).clamp(0, 7) as usize;
            let n = self.tables.tuning.korean_citizens[i];
            self.free_train(b, power::KOREANS, ty::PEASANTS, n, false);
        }
    }

    /// The Barracks' four arms, in the order the original tests them.
    fn barracks_free_units(&mut self, b: usize, who: Player) {
        let scouts = self.tables.tuning.iroquois_free_scout;
        self.free_train(b, power::IROQUOIS, ty::SCOUT, scouts, true);

        let age = self.nation_age(who);
        let g = &self.tables.tuning;
        let legions = free_scaled(
            g.roman_barracks_legion,
            g.roman_max_legion,
            free_tier(
                age,
                g.roman_age_for_1_legion,
                g.roman_age_for_2_legions,
                g.roman_age_for_3_legions,
            ),
        );
        self.free_train(b, power::ROMANS, ty::HOPLITES, legions, true);

        // The British ladder has **no multiplier and no cap**: the tier is
        // the count, where the Roman and Aztec arms scale and clamp theirs.
        let g = &self.tables.tuning;
        let archers = free_tier(
            age,
            g.british_age_for_1_archer,
            g.british_age_for_2_archers,
            g.british_age_for_3_archers,
        );
        self.free_train(b, power::BRITISH, ty::BOWMEN, archers, true);

        // The Aztec ladder is not the other two's: it is 1 below the first
        // age, then 2, then 3 past the third. Only the patch-4 arm is
        // modelled — the older one reads `ages` where this reads the min.
        let g = &self.tables.tuning;
        let tier = if age == 0 { 1 } else { 2 + i32::from(age > 2) };
        let light = free_scaled(g.aztec_barracks_light, g.aztec_max_light, tier);
        self.free_train(b, power::AZTECS, ty::SLINGERS, light, true);
    }

    /// **`Build::process@0061edf0:362–381` — the Persians' Market trains a
    /// caravan** (`docs/AI.md` §146). Every fifteen frames of the Market's
    /// own phase, `(o + frame) % 15 == 0`, a Market (`0x1b4`) whose owner
    /// `has_tribe_bonus(0x17)` trains a `CARA` — straight through
    /// `Build::train`, no queue and no price — while the leader's `caras`
    /// (`+0x980`) is under `get_caravan_limit(1)` and
    /// `check_population(CARA)` says it fits. It is the stretch after
    /// `do_queue` and the gather tiles, behind the Kremlin's spy and the
    /// government hero and before the Terra Cotta's soldier, so the tick
    /// calls it after the gather step and only for an active building.
    ///
    /// Not the graft and not the upgrade: the call names `CARA` itself.
    /// The coverage pair's Persians: the Market `1/2018` completes on 661
    /// and trains `1/15` on 667, its first slot as an active building.
    pub(crate) fn persian_market_caravan(&mut self, b: usize, frame: i64) {
        // The type itself, `== 0x1b4`, not its lineage: the record's own
        // `Ident`.
        let market = self.buildings[b].ty.is_some_and(|r| {
            self.tables.build_types[r].ident == crate::cities::build::Ident::Market
        });
        if !market || self.buildings[b].phase(frame) % 15 != 0 {
            return;
        }
        let who = self.buildings[b].owner;
        let w = who as usize;
        if w >= self.tech.len()
            || !self.tables.tech_tree.has_tribe_bonus(
                &self.tables.setup,
                &self.tech[w],
                power::PERSIANS,
            )
        {
            return;
        }
        let caras = self.ai.ai.get(w).map_or(0, |l| l.census.caras);
        if caras >= self.caravan_limit(who) {
            return;
        }
        let Some(rec) = self.unit_record(ty::CARAVAN) else {
            return;
        };
        let m = &self.roster.muster[w];
        if crate::economy::cost::exceeds_population(
            m.cap,
            m.control,
            self.tables.unit_types[rec].price.pop,
        ) {
            return;
        }
        self.build_train(b, rec);
    }

    /// Whether a building is the `TERRACOTTA` wonder (`TypeIndex` `0x211`).
    fn is_terra_cotta(&self, b: usize) -> bool {
        self.buildings[b]
            .ty
            .is_some_and(|r| self.tables.build_types[r].tree == Some(0x211))
    }

    /// `Build::activate@00623e20:1370`: the wonder finishes with its clock
    /// at `TERRA_COTTA_FREE * 15` — the thirty seconds before its first
    /// soldier. Run721's `1/2042` reads 449 on the block after it finishes
    /// (3395) and its first soldier is born on 3843, 450 frames on.
    pub(crate) fn terra_cotta_activate(&mut self, b: usize) {
        if self.is_terra_cotta(b) {
            self.buildings[b].recharging = self.tables.tuning.terra_cotta_free * 15;
        }
    }

    /// Whether a building is the `KREMLIN` wonder (`TypeIndex` `0x21a`).
    fn is_kremlin(&self, b: usize) -> bool {
        self.buildings[b]
            .ty
            .is_some_and(|r| self.tables.build_types[r].tree == Some(0x21a))
    }

    /// **The Kremlin's start** — `Build::activate@00623e20:1376`..`1400`
    /// and the earlier arm at `:617` (`docs/AI.md` §188). The finished
    /// wonder starts its clock at 1 (`captured` false; a captured one
    /// starts at `KREMLIN_SPY`) and, when none of the owner's live spies
    /// carries the Kremlin's tag (`unit_flags & 0x400`), marks the spy as
    /// due (`leader_flags |= 0x10000000`). `:617` sets that flag for any
    /// uncaptured Kremlin the owner has not held before; one Kremlin
    /// exists per game, so the two agree and only the second is written.
    pub(crate) fn kremlin_activate(&mut self, b: usize, captured: bool) {
        if !self.is_kremlin(b) {
            return;
        }
        let who = self.buildings[b].owner;
        self.buildings[b].recharging = if captured {
            self.tables.tuning.kremlin_spy
        } else {
            1
        };
        let tagged = (0..self.units.len()).any(|u| {
            self.units[u].alive()
                && self.units[u].owner == who
                && self.units[u].kremlin_spy
                && self.unit_is_spy(u)
        });
        if !tagged && (who as usize) < self.tech.len() {
            self.tech[who as usize].kremlin_spy_due = true;
        }
    }

    /// **The Kremlin's spy** — `Build::process@0061edf0:306`..`333`, the
    /// stretch ahead of the Persian caravan (`docs/AI.md` §188). Every
    /// fifteenth frame of the wonder's own phase, `(o + frame) % 15 == 0`,
    /// while the owner's `leader_flags & 0x10000000` is set, the clock
    /// `recharging` loses one; under 1 the flag clears, the clock rewinds
    /// to `KREMLIN_SPY`, and a `SPY` (the type itself) is trained at the
    /// wonder with no queue and no population test. The unit takes the
    /// `0x400` tag and its `control_cost` comes off the owner's `control`
    /// again: the tagged spy stands outside the population.
    ///
    /// Run762's Kremlin `1/2033` finishes on 4573 (`recharging` 1) and its
    /// first slot, `(2033 + 4582) % 15 == 0`, trains `1/169` (`guy` 58) on
    /// the dump's block 4583, `recharging` 30.
    ///
    /// Left out, and named in `docs/AI.md` §188: the tagged spy's close
    /// hands its `control` back and a retyping of it (this crate's deaths
    /// move no `control` at all). The close's re-arm is
    /// [`Sim::kremlin_spy_closed`].
    pub(crate) fn kremlin_spy(&mut self, b: usize, frame: i64) {
        if !self.is_kremlin(b) || self.buildings[b].phase(frame) % 15 != 0 {
            return;
        }
        let who = self.buildings[b].owner;
        let w = who as usize;
        if w >= self.tech.len() || !self.tech[w].kremlin_spy_due {
            return;
        }
        self.buildings[b].recharging -= 1;
        if self.buildings[b].recharging >= 1 {
            return;
        }
        self.tech[w].kremlin_spy_due = false;
        self.buildings[b].recharging = self.tables.tuning.kremlin_spy;
        let Some(rec) = self.unit_record(0x3a) else {
            return;
        };
        let spy = self.build_train(b, rec);
        self.units[spy.unit].kremlin_spy = true;
        self.roster.muster[w].control -= self.tables.unit_types[rec].price.pop;
    }

    /// **`Unit::close@0060ee50:98`..`106`: a hero's death stamps its
    /// leader's respawn** (item 3786, `docs/GOLDEN.md` §149.11). Under the
    /// slot's `flags & 1` and `who < 8`, and ungated by the squad, a unit
    /// whose `is_gov_hero` (vslot `+0xc8`, `UnitData::is_gov_hero` in every
    /// unit vtable: the type's `OBJ_MASK` `0x4000000`, [`UnitTraits::hero`])
    /// answers writes `gov_hero_frame = GOV_HERO_RESPAWN × 15 + frame`;
    /// [`Sim::gov_hero_respawn`] is the train it schedules.
    ///
    /// [`UnitTraits::hero`]: crate::tech::UnitTraits::hero
    pub(crate) fn gov_hero_closed(&mut self, u: usize) {
        let who = self.units[u].owner as usize;
        if who >= 8 || who >= self.tech.len() {
            return;
        }
        let hero = self
            .unit_tree(u)
            .is_some_and(|t| self.tables.tech_tree.kind(t).unit().is_some_and(|x| x.hero));
        if hero {
            self.tech[who].gov_hero_frame =
                i64::from(self.tables.tuning.gov_hero_respawn) * 15 + self.frame;
        }
    }

    /// **`Build::process@0061edf0:332`..`343`, the government hero's
    /// respawn** (item 3786, `docs/GOLDEN.md` §149.11), between the
    /// Kremlin's spy and the Persian caravan: when the owner's
    /// `gov_hero_frame` is `frame + 1`, a Senate (`0x1b6`) trains
    /// `LeaderData::get_gov_hero`'s type with no queue and no population
    /// test. run1413's who=1 loses its Despot `1/28` on 1463, and its
    /// Senate `1/2007` trains The Monarch `1/15` on 1912 — three
    /// `Guy::init_real` — with `gov_hero_frame` 1912 at the closing block
    /// (`Unit::init`'s own stamp, [`Sim::init_unit`]).
    ///
    /// SEAM: the arm after it, a city centre (`flags & 0x20`) of a leader
    /// with no Senate (`+0x558e` plus `get_buildings(SENATE)` under 1)
    /// training the hero itself (parked 3786a).
    pub(crate) fn gov_hero_respawn(&mut self, b: usize, frame: i64) {
        let who = self.buildings[b].owner as usize;
        if who >= self.tech.len() || self.tech[who].gov_hero_frame != frame + 1 {
            return;
        }
        if !self.building_is(b, crate::cities::build::Ident::Senate) {
            return;
        }
        let Some(hero) = self
            .tables
            .tech_tree
            .get_gov_hero(&self.tables.setup, &self.tech[who])
        else {
            return;
        };
        let Some(rec) = self.unit_record(hero) else {
            return;
        };
        self.build_train(b, rec);
    }

    /// `Unit::close@0060ee50:80`..: a tagged spy that dies, its owner
    /// holding a standing Kremlin (`LeaderData::has_wonder`, an activated
    /// building of the type), makes the next one due (`docs/AI.md` §188).
    pub(crate) fn kremlin_spy_closed(&mut self, u: usize) {
        let who = self.units[u].owner as usize;
        if self.units[u].kremlin_spy
            && self.unit_is_spy(u)
            && who < self.tech.len()
            && self.buildings.iter().any(|b| {
                b.alive
                    && b.active
                    && b.owner as usize == who
                    && b.ty
                        .is_some_and(|r| self.tables.build_types[r].tree == Some(0x21a))
            })
        {
            self.tech[who].kremlin_spy_due = true;
        }
    }

    /// **The Terra Cotta Army's soldier** — `Build::process@0061edf0:384`..
    /// `0061f733`, the arm after the Persian caravan. A building whose type
    /// is `TERRACOTTA` (`0x211`) counts its `recharging` (`BuildData +0x7a`)
    /// down by one every frame it is processed; on reaching zero it resets
    /// the clock to
    ///
    /// ```text
    /// TERRA_COTTA_FREE * 15 + barracks_units * 15 / 2      # TERRA_COTTA_DELAY == 7
    /// ```
    ///
    /// and, when the owner has room (`control < cap`, the strict form —
    /// not `check_population`'s), trains the newest type of the Slingers'
    /// line (the Aztec arm's `get_graft` then `current_upgrade`) at the
    /// wonder. `barracks_units` is `Leader +0x9f8`, which
    /// `Leader::track_unit_type@006e0dd0` moves for every armed type whose
    /// trainer (`UnitTypeData +0x40`) is the Barracks — the sum of
    /// `num_units` over them. The shipped file writes `TERRA_COTTA_FREE`
    /// `30 seconds`, which loads as 30; the delay's `7 == "half a second"`.
    ///
    /// Diff-backed on the coverage pair's Persians: `1/2042` (orig_type
    /// 529) holds `recharging` 449 on its first block (3395) and 6 on 3838,
    /// then 607 on 3844 — 450 and 30·15 + 21·15/2 — and births a squad of
    /// the Mech Infantry on 3843 (run754).
    pub(crate) fn terra_cotta_army(&mut self, b: usize) {
        /// The base of the Slingers' line the arm trains from.
        const TERRA_COTTA_BASE: TypeId = 0x52;
        if !self.is_terra_cotta(b) {
            return;
        }
        self.buildings[b].recharging -= 1;
        if self.buildings[b].recharging >= 1 {
            return;
        }
        let who = self.buildings[b].owner;
        let w = who as usize;
        let barracks_units = self.barracks_units(who);
        // The file's `7` is half a second; any other value is frames.
        let delay = self.tables.tuning.terra_cotta_delay;
        let added = if delay == 7 {
            barracks_units * 15 / 2
        } else {
            barracks_units * delay
        };
        self.buildings[b].recharging = self.tables.tuning.terra_cotta_free * 15 + added;
        let Some(rec) = self.free_unit_record(who, TERRA_COTTA_BASE, true) else {
            return;
        };
        let m = &self.roster.muster[w];
        if m.control < m.cap {
            self.build_train(b, rec);
        }
    }

    /// One arm: `n` copies of `base`'s line, each under `check_population`.
    ///
    /// `upgrade` is the arm's own shape — every arm but the American
    /// scholar, the French pair and the Korean citizen runs the graft's
    /// answer through `current_upgrade` first.
    fn free_train(&mut self, b: usize, power: usize, base: TypeId, n: i32, upgrade: bool) {
        if n <= 0 {
            return;
        }
        let who = self.buildings[b].owner;
        let w = who as usize;
        if !self
            .tables
            .tech_tree
            .has_tribe_bonus(&self.tables.setup, &self.tech[w], power)
        {
            return;
        }
        let Some(rec) = self.free_unit_record(who, base, upgrade) else {
            return;
        };
        let pop = self.tables.unit_types[rec].price.pop;
        for _ in 0..n {
            let m = &self.roster.muster[w];
            if crate::economy::cost::exceeds_population(m.cap, m.control, pop) {
                continue;
            }
            self.build_train(b, rec);
        }
    }

    /// `get_graft` then, for all but three arms, `current_upgrade` — as an
    /// index into [`Sim::unit_types`].
    fn free_unit_record(&self, who: Player, base: TypeId, upgrade: bool) -> Option<usize> {
        let p = self.tech.get(who as usize)?;
        if base >= self.tables.tech_tree.types.len() {
            return None;
        }
        let t = self
            .tables
            .tech_tree
            .get_graft(&self.tables.setup, p, Some(base))?;
        let t = if upgrade {
            self.tables
                .tech_tree
                .current_upgrade(&self.tables.setup, p, t)
        } else {
            t
        };
        self.unit_record(t)
    }

    /// `min(epoch[0], ages)` — the number the Roman, British and Aztec
    /// ladders are read against. The Military level and the age are
    /// **different fields** (`LeaderDataEncrypt +0xe8` and `+0xdc`), and the
    /// smaller of the two is the tier.
    fn nation_age(&self, who: Player) -> i32 {
        let p = &self.tech[who as usize];
        p.epoch[0].min(p.ages)
    }
}

/// The 1/2/3 ladder the Roman and British arms share: the highest threshold
/// `age` clears.
const fn free_tier(age: i32, one: i32, two: i32, three: i32) -> i32 {
    if age >= three {
        3
    } else if age >= two {
        2
    } else if age >= one {
        1
    } else {
        0
    }
}

/// `per × tier`, held to `max`. **A zero `max` skips the ladder entirely**
/// and pays `per` — the whole scaling block sits inside `if (max != 0)` —
/// and a negative either side pays nothing.
const fn free_scaled(per: i32, max: i32, tier: i32) -> i32 {
    if max == 0 {
        return per;
    }
    let n = per * tier;
    if n < 0 || max < 0 {
        0
    } else if n > max {
        max
    } else {
        n
    }
}

#[cfg(test)]
mod tests;
