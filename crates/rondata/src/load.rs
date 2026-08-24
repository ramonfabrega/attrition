//! Turns the shipped tables into the simulation's typed data.
//!
//! `unitrules.xml`, `buildingrules.xml`, `techrules.xml`, `resourcerules.xml`
//! and `rules.xml` arrive here as positional [`crate::Table`]s; this module
//! reads them the way the original's loaders do — `UnitType::init`,
//! `BuildType::init`, `TechType::init`, `GoodType::init`, `Types::finalize`
//! — into `sim::UnitType`, `sim::build::BuildType`, a `sim::tech::TechTree`
//! and a combat `sim::combat::Table`. Every scale and every derived field is
//! the one the mechanic documents established; where a field's column is
//! not named by any reading the loader leaves the default and says so under
//! "not established" in `docs/DATALAYER.md`.
//!
//! # Index spaces
//!
//! The simulation keeps four tables with four index spaces, by design
//! (`docs/DECISIONS.md` entry 18): unit types, building types, the tree, and
//! the six resources. This loader keeps record index as identity in each —
//! unit *i* is `unitrules.xml` record *i*, building *j* is record *j* — and
//! lays the tree out in the original's `TypeIndex` order without its gaps:
//!
//! | tree ids | what | `TypeIndex` |
//! | --- | --- | --- |
//! | `0..50` | the 50 `resourcerules.xml` goods | `0x000 + i` |
//! | `50..414` | the 364 unit records, gaia included | `0x032 + i` |
//! | `414..543` | the 129 building records | `0x19e + i` |
//! | `543..628` | the 85 techs | `0x220 + i` |
//!
//! so [`Loaded::type_index`] recovers the original's id for any tree entry,
//! which is what the eventual gamelog diff will key on.

use crate::{Cost, Install, Range, Record, Rules, Scalar, Table, tribe_mask};
use sim::attrition::{self, Domain, KindFacts, UnitKind};
use sim::build::{self, BuildType, Ident};
use sim::combat::{self, BuildClass, Profile, mask, role};
use sim::cost::{self, Price, Progression, RampClass};
use sim::economy::{RESOURCES, Resource};
use sim::garrison;
use sim::movement::degrees_to_angle;
use sim::production::Times;
use sim::tech::{self, Kind, Line, Preq, TechTree, TypeDef, TypeId, UnitTraits};
use sim::{Tuning, UnitType};

/// The original's `TypeIndex` bases, `docs/TECH.md` §"The type space".
pub const BASE_UNITTYPES: i32 = 0x32;
pub const BASE_BUILDTYPES: i32 = 0x19e;
pub const BASE_TECHTYPES: i32 = 0x220;
/// The six government patriots, `0x160–0x165`, as unit-record indices.
pub const PATRIOTS: std::ops::RangeInclusive<usize> = 302..=307;
/// The seventeen wonders, `0x20e–0x21e`, as building-record indices.
pub const WONDERS: std::ops::RangeInclusive<usize> = 112..=128;
/// `UNIT_BLOCK_RADIUS` as loaded: one `UCoord`, 48 position units.
const UNIT_BLOCK_RADIUS: i32 = 48;

/// Everything the tables load into, plus the maps between the index spaces.
#[derive(Clone, Debug)]
pub struct Loaded {
    pub unit_types: Vec<UnitType>,
    pub build_types: Vec<BuildType>,
    pub tree: TechTree,
    /// The combat table over unit ids, from `balance.xml` and the hardcoded
    /// chain — see [`crate::balance`].
    pub table: combat::Table,
    /// What the table was built from, one per unit id: the masks, the age,
    /// the siege and caravan flags, the domain, the named lineages. Exposed so
    /// the original's type dump can check the inputs as well as the output
    /// (`rondata --types`).
    pub kinds: Vec<sim::balance::Kind>,
    /// The same for the building ids: the masks, `get_age`, the
    /// `return_pack` lines, the wonder range.
    pub build_kinds: Vec<sim::balance::Kind>,
    /// The `NAME` column of each record: what the interface shows.
    pub unit_names: Vec<String>,
    pub build_names: Vec<String>,
    pub tech_names: Vec<String>,
    pub good_names: Vec<String>,
    /// `TypeData::type_name`: the `TYPENAME` column, or `NAME` when the
    /// record has none or leaves it empty. This is what `Types::unit_key`,
    /// `build_key` and `tech_key` compare a `FROM`/`JUMP`/`GRAFT`/`WHERE`/
    /// `PREQn` text against — `UnitType::init` reads `NAME` into
    /// `Type::set_name` and `TYPENAME` into `+0xb0`, falling back to the name
    /// when the column is empty, and the keys search `+0xb0`. The shipped
    /// file relies on it: `FROM Marines`, `JUMP Arquebus Immortal` and
    /// `JUMP ECONQUISTADOR` are `TYPENAME`s of records whose `NAME`s are
    /// `Continental Marines`, `Arqimmortal` and `Elite Conquistador`.
    pub unit_type_names: Vec<String>,
    pub build_type_names: Vec<String>,
    /// Tree id of each unit record, building record, tech record, good.
    pub unit_tree: Vec<TypeId>,
    pub build_tree: Vec<TypeId>,
    pub tech_tree: Vec<TypeId>,
    pub good_tree: Vec<TypeId>,
    /// Names the tables referred to that resolved to nothing, the way the
    /// original's `tech_key`/`unit_key`/`build_key` would have logged an
    /// error. Empty for the shipped files.
    pub warnings: Vec<String>,
    /// rules.xml's `mapstyles` category keys in file order — `map_styles`,
    /// which `GameInfo.map_style` indexes and `get_mapstyle()` names.
    pub map_styles: Vec<String>,
    /// The opening scripts under `ai/scripts/`, `(file name, text)`:
    /// `economic.bhs`, `defensive.bhs` and the library they include
    /// (`docs/AI.md` §3). Empty when loaded from tables alone.
    pub scripts: Vec<(String, String)>,
}

impl Loaded {
    /// The original's `TypeIndex` of a tree entry.
    pub fn type_index(&self, id: TypeId) -> i32 {
        let g = self.good_tree.len();
        let u = g + self.unit_tree.len();
        let b = u + self.build_tree.len();
        let id32 = id as i32;
        if id < g {
            id32
        } else if id < u {
            BASE_UNITTYPES + (id32 - g as i32)
        } else if id < b {
            BASE_BUILDTYPES + (id32 - u as i32)
        } else {
            BASE_TECHTYPES + (id32 - b as i32)
        }
    }

    /// The unit record a `TypeIndex` names, if it is a unit.
    pub fn unit_of_type_index(&self, t: i32) -> Option<usize> {
        let i = t - BASE_UNITTYPES;
        (0..self.unit_types.len() as i32)
            .contains(&i)
            .then_some(i as usize)
    }

    /// The building record a `TypeIndex` names, if it is a building.
    pub fn build_of_type_index(&self, t: i32) -> Option<usize> {
        let i = t - BASE_BUILDTYPES;
        (0..self.build_types.len() as i32)
            .contains(&i)
            .then_some(i as usize)
    }

    /// A unit record by name, case-insensitively, first match — the
    /// `TYPENAME` the keys use first, then the `NAME` the interface shows.
    pub fn unit_named(&self, name: &str) -> Option<usize> {
        find_name(&self.unit_type_names, name).or_else(|| find_name(&self.unit_names, name))
    }

    /// A building record by name, the same way.
    pub fn build_named(&self, name: &str) -> Option<usize> {
        find_name(&self.build_type_names, name).or_else(|| find_name(&self.build_names, name))
    }

    /// A tech record by name.
    pub fn tech_named(&self, name: &str) -> Option<usize> {
        find_name(&self.tech_names, name)
    }

    /// Installs the tree, the types and the table into a fresh simulation.
    ///
    /// The order matters: the tree first, because `Sim::add_unit_type` reads
    /// each type's tree bit to seed its researched flag; then the types; then
    /// the combat table — `add_unit_type` grows the table with 100s as it
    /// goes, and the loaded one replaces it whole.
    pub fn install_into(&self, sim: &mut sim::Sim) {
        sim.set_tech_tree(self.tree.clone());
        for ty in &self.unit_types {
            sim.add_unit_type(ty.clone());
        }
        for ty in &self.build_types {
            sim.add_build_type(ty.clone());
        }
        sim.table = self.table.clone();
    }

    /// A simulation with the loaded data and `players` players, each holding
    /// the starting position.
    pub fn sim(&self, tuning: Tuning, world: sim::World, players: usize) -> sim::Sim {
        let mut s = sim::Sim::new(tuning, world, players);
        self.install_into(&mut s);
        for who in 0..players {
            s.start_techs(who as sim::Player);
        }
        s
    }
}

fn find_name(names: &[String], name: &str) -> Option<usize> {
    let want = name.trim();
    names.iter().position(|n| n.eq_ignore_ascii_case(want))
}

/// Reads every table of an install and loads them.
pub fn load(install: &Install) -> Result<Loaded, crate::Error> {
    let rules = install.rules()?;
    let units = install.units()?;
    let buildings = install.buildings()?;
    let techs = install.techs()?;
    let goods = install.resources()?;
    let balance = install.balance()?;
    let mut loaded = load_tables(&rules, &units, &buildings, &techs, &goods, Some(&balance));
    // The nations' names, from the per-nation files rules.xml points at.
    let names = install.tribe_names(&rules)?;
    for (tribe, name) in loaded.tree.tribes.iter_mut().zip(names) {
        tribe.name = name;
    }
    // The opening scripts — `Leaders::prod_script_path` is `.\ai\scripts\`.
    // A missing file is not an error: a game without scripts is a game whose
    // AI skips to step 2, which is what the original does when
    // `Compiler::compile` fails.
    for name in ["economic.bhs", "defensive.bhs", "aibestbuildlibrary.bhs"] {
        let path = install.root().join("ai").join("scripts").join(name);
        if let Ok(text) = std::fs::read_to_string(&path) {
            loaded.scripts.push((name.to_string(), text));
        }
    }
    Ok(loaded)
}

/// Loads already-read tables. `balance` may be absent, in which case the
/// combat table is the hardcoded chain alone.
pub fn load_tables(
    rules: &Rules,
    units: &Table,
    buildings: &Table,
    techs: &Table,
    goods: &Table,
    balance: Option<&crate::balance::BalanceXml>,
) -> Loaded {
    let mut warnings = Vec::new();
    let name_of = |r: &Record| r.text("NAME").unwrap_or("").trim().to_string();
    let unit_names: Vec<String> = units.records.iter().map(name_of).collect();
    let build_names: Vec<String> = buildings.records.iter().map(name_of).collect();
    let tech_names: Vec<String> = techs.records.iter().map(name_of).collect();
    let good_names: Vec<String> = goods.records.iter().map(name_of).collect();
    let type_name_of = |r: &Record| {
        let t = r.text("TYPENAME").unwrap_or("").trim();
        if t.is_empty() {
            name_of(r)
        } else {
            t.to_string()
        }
    };
    let unit_type_names: Vec<String> = units.records.iter().map(type_name_of).collect();
    let build_type_names: Vec<String> = buildings.records.iter().map(type_name_of).collect();

    // ---- tree ids ----
    let g = good_names.len();
    let good_tree: Vec<TypeId> = (0..g).collect();
    let unit_tree: Vec<TypeId> = (0..unit_names.len()).map(|i| g + i).collect();
    let u = g + unit_names.len();
    let build_tree: Vec<TypeId> = (0..build_names.len()).map(|i| u + i).collect();
    let b = u + build_names.len();
    let tech_tree: Vec<TypeId> = (0..tech_names.len()).map(|i| b + i).collect();

    // ---- the columns every record shares, read once ----
    let unit_cols: Vec<UnitCols> = units
        .records
        .iter()
        .enumerate()
        .map(|(i, r)| UnitCols::read(i, r, &unit_type_names, &build_type_names, &mut warnings))
        .collect();
    let build_cols: Vec<BuildCols> = buildings
        .records
        .iter()
        .enumerate()
        .map(|(i, r)| BuildCols::read(i, r, &build_type_names, &mut warnings))
        .collect();

    // ---- lineage, as `ObjectTypeData::is(x, 0)`: self, `from` chain, graft ----
    let unit_from: Vec<Option<usize>> = unit_cols.iter().map(|c| c.from).collect();
    let unit_graft: Vec<Option<usize>> = unit_cols.iter().map(|c| c.graft).collect();
    let build_from: Vec<Option<usize>> = build_cols.iter().map(|c| c.from).collect();
    let unit_is = |t: usize, root: usize| is_lineage(t, root, &unit_from, &unit_graft);
    let build_is = |t: usize, root: usize| is_lineage(t, root, &build_from, &[]);
    let uname = |n: &str| find_name(&unit_type_names, n).or_else(|| find_name(&unit_names, n));
    let bname = |n: &str| find_name(&build_type_names, n).or_else(|| find_name(&build_names, n));
    let is_named_unit = |t: usize, n: &str| uname(n).is_some_and(|r| unit_is(t, r));
    // The named lineages the mechanics test.
    let citizen = |t: usize| t < 4; // PEASANTS, PEASANTSKOREAN, SCHOLARS, SCHOLARSKOREAN by id
    let scholar = |t: usize| t == 2 || t == 3;
    let trader = |t: usize| t == 11 || t == 12 || t == 350; // MERCHANT, MERCHANTDUTCH, FURTRAPPER
    let militia = |t: usize| is_named_unit(t, "Militia");
    let caravan = |t: usize| is_named_unit(t, "Caravan") || is_named_unit(t, "Merchant Fleet");
    let scout = |t: usize| is_named_unit(t, "Scout");
    let spy = |t: usize| is_named_unit(t, "Spy");
    let supply = |t: usize| is_named_unit(t, "Supply Wagon") || t == 302 || t == 304 || t == 306;
    let hero = |t: usize| unit_cols[t].obj_masks & 0x0400_0000 != 0;
    let packs = |t: usize| {
        [
            "Catapult",
            "Flaming Arrow",
            "Merchant",
            "Armed Merchant",
            "Fur Trapper",
            "Machine Gun",
            "Fishermen",
            "Katyusha Rocket",
        ]
        .iter()
        .any(|n| is_named_unit(t, n))
    };
    let barracks = bname("Barracks");
    let stable = bname("Stable");
    let factory = bname("Factory");

    // ---- the tech tree ----
    let mut tree = TechTree::new();
    tree = tree.with_tuning(&Tuning::RON);
    let tech_key = |text: Option<&str>, warnings: &mut Vec<String>| -> Preq {
        match text.map(str::trim) {
            None | Some("") => Preq::None,
            Some(t) if t.eq_ignore_ascii_case("none") => Preq::None,
            Some(t) if t.eq_ignore_ascii_case("disable") => Preq::Disabled,
            Some(t) => match find_name(&tech_names, t) {
                Some(i) => Preq::Of(tech_tree[i]),
                None => {
                    warnings.push(format!("tech_key: no tech named {t:?}"));
                    Preq::Disabled
                }
            },
        }
    };
    for (i, r) in goods.records.iter().enumerate() {
        let mut d = TypeDef::good(&good_names[i]);
        d.preq[0] = tech_key(r.text("PREQ0"), &mut warnings);
        d.preq[1] = tech_key(r.text("PREQ1"), &mut warnings);
        d.obs = tech_key(r.text("OBS"), &mut warnings);
        d.tribe_mask = u32::MAX;
        let id = tree.add(d);
        debug_assert_eq!(id, good_tree[i]);
    }
    for (i, r) in units.records.iter().enumerate() {
        let c = &unit_cols[i];
        let traits = UnitTraits {
            free: c.flags & 0x80 != 0,
            jumpable: c.flags & 0x200 != 0,
            unique: c.flags & 0x0100_0000 != 0,
            hero: hero(i),
            patriot: PATRIOTS.contains(&i),
            combat: c.attack != 0,
        };
        let mut d = TypeDef::unit(&unit_names[i], traits);
        d.preq[0] = tech_key(r.text("PREQ0"), &mut warnings);
        d.preq[1] = tech_key(r.text("PREQ1"), &mut warnings);
        d.from = c.from.map(|f| unit_tree[f]);
        d.jump = c.jump.map(|j| unit_tree[j]);
        d.graft = c.graft.map(|g| unit_tree[g]);
        d.obs = Preq::Disabled;
        d.where_ = c.where_.map(|w| build_tree[w]);
        d.tribe_mask = mask_bits(r.text("TRIBE_MASK"));
        let id = tree.add(d);
        debug_assert_eq!(id, unit_tree[i]);
    }
    for (i, r) in buildings.records.iter().enumerate() {
        let c = &build_cols[i];
        let mut d = TypeDef::new(
            &build_names[i],
            Kind::Building {
                auto: c.flags & build::flags::UPGRADES != 0,
                wonder: WONDERS.contains(&i),
            },
        );
        d.preq[0] = tech_key(r.text("PREQ0"), &mut warnings);
        d.preq[1] = tech_key(r.text("PREQ1"), &mut warnings);
        d.preq[2] = tech_key(r.text("PREQ2"), &mut warnings);
        d.from = c.from.map(|f| build_tree[f]);
        d.jump = c.jump.map(|j| build_tree[j]);
        d.obs = tech_key(r.text("OBSOLETE"), &mut warnings);
        d.tribe_mask = mask_bits(r.text("TRIBE_MASK"));
        let id = tree.add(d);
        debug_assert_eq!(id, build_tree[i]);
    }
    for (i, r) in techs.records.iter().enumerate() {
        let kind = tech_kind(i);
        let mut d = TypeDef::new(&tech_names[i], kind);
        d.cost = cost_slots(r.text("COST"));
        d.job_time = int(r, "JOB_TIME").unwrap_or(0);
        d.preq[0] = tech_key(r.text("PREQ0"), &mut warnings);
        d.preq[1] = tech_key(r.text("PREQ1"), &mut warnings);
        d.preq[2] = tech_key(r.text("PREQ2"), &mut warnings);
        d.age = r
            .text("AGE")
            .and_then(Scalar::parse)
            .map_or(-1, Scalar::written_int);
        d.where_ = r.text("WHERE").and_then(|w| {
            let w = w.trim();
            if w.eq_ignore_ascii_case("none") || w.eq_ignore_ascii_case("disable") {
                return None;
            }
            let found = bname(w).map(|b| build_tree[b]);
            if found.is_none() {
                warnings.push(format!("build_key: no building named {w:?}"));
            }
            found
        });
        d.tribe_mask = mask_bits(r.text("TRIBE_MASK"));
        let id = tree.add(d);
        debug_assert_eq!(id, tech_tree[i]);
    }
    // The 24 nations. `Tribe::graft` is identity and `barbarian` is false:
    // neither the graft tables nor the barbarian flags have a located source
    // (`docs/TECH.md`, "What is not established"). `TechTree::new` seeds one
    // default tribe; the roster replaces it.
    tree.tribes.clear();
    for _ in 0..rules.tribes.len() {
        tree.add_tribe(tech::Tribe {
            graft: vec![None; unit_names.len()],
            barbarian: false,
            name: String::new(),
        });
    }
    // The roles, by the shipped names. A name that does not resolve leaves
    // the role inert, which is the tree's own rule for a missing role.
    {
        let bt = |n: &str| bname(n).map(|i| build_tree[i]);
        let ut = |n: &str| uname(n).map(|i| unit_tree[i]);
        let gt = |n: &str| find_name(&good_names, n).map(|i| good_tree[i]);
        let tt = |n: &str| find_name(&tech_names, n).map(|i| tech_tree[i]);
        let r = &mut tree.roles;
        r.market = bt("Market");
        r.knowledge = gt("Knowledge");
        r.university = bt("University");
        r.tower = bt("Tower");
        r.fortx = bt("Fort");
        r.granary = bt("Granary");
        r.lumbermill = bt("Lumber Mill");
        r.smelter = bt("Smelter");
        r.metal = gt("Metal");
        r.mine = bt("Mine");
        r.temple = bt("Temple");
        r.refinery = bt("Refinery");
        r.oilwell = bt("Oil Well");
        r.town = bt("Large City");
        r.senate = bt("Senate");
        r.machinegun = ut("Machine Gun");
        r.rifleman = ut("Riflemen");
        r.infantry = ut("Infantry");
        r.mechinfantry = ut("Mech. Infantry").or_else(|| ut("Mechanized Infantry"));
        r.cataphract = ut("Cataphract");
        r.horsearchers = ut("Horse Archer");
        r.nuclearmissile = ut("Nuclear Missile");
        r.icbm = ut("ICBM");
        // Tower through Redoubt, `0x1b7–0x1be`.
        r.fort_line = (0x1b7..=0x1be)
            .map(|t| t - BASE_BUILDTYPES as usize)
            .filter(|&i| i < build_tree.len())
            .map(|i| build_tree[i])
            .collect();
        // The carpentry, agriculture and metal lines.
        r.german_industry = [
            "Carpentry",
            "Logging Industry",
            "Papermill",
            "Agriculture",
            "Crop Rotation",
            "Food Industry",
            "Metal Alloys",
            "Cold Casting",
            "Steel",
        ]
        .iter()
        .filter_map(|n| tt(n))
        .collect();
        // What every final requires: the last age and the four last epochs.
        r.final_needs = [
            "Information Age",
            "Computerization",
            "Globalization",
            "International Law",
            "Selective Service",
        ]
        .iter()
        .filter_map(|n| tt(n))
        .collect();
    }
    tree.finalize();

    // ---- the unit types ----
    let age_of_tree = |id: TypeId| -> i32 {
        // `ObjectTypeData::get_age_slow`: the first prerequisite that is a
        // tech decides — an age tech's own `AGE` column **plus one** (the
        // column is 0 for Classical … 6 for Information, and a unit needing
        // Classical is an age-1 unit), any other tech's `AGE` as written. No
        // tech prerequisite is 0. An earlier draft returned the age tech's
        // index without the +1 and put 306 of 364 units one age early.
        let d = &tree.types[id];
        for p in d.preq {
            if let Preq::Of(t) = p {
                let td = &tree.types[t];
                match td.kind {
                    Kind::Age(_) => return td.age + 1,
                    _ if td.kind.is_tech() => return td.age,
                    _ => {}
                }
            }
        }
        -1
    };
    let mut unit_types = Vec::with_capacity(units.len());
    for (i, r) in units.records.iter().enumerate() {
        let c = &unit_cols[i];
        let worker = citizen(i);
        let merchant = trader(i);
        let is_supply = supply(i);
        let is_caravan = caravan(i);
        let kind = UnitKind {
            siege: c.siege,
            militia: militia(i),
            trader: merchant,
            domain: c.domain,
            exempt_kind: attrition::kind_exempt(&KindFacts {
                worker,
                merchant,
                hero: hero(i),
                supply: is_supply,
                attack: c.attack,
                special: scout(i),
                spy: spy(i),
                caravan: is_caravan,
            }),
            supply_unit: is_supply,
            gathering_non_flat: false,
            idle: false,
        };
        let class = if scholar(i) {
            RampClass::Scholar
        } else if worker || merchant || is_caravan {
            RampClass::Worker
        } else if c.obj_masks & mask::CIVILIAN != 0 {
            RampClass::OtherCivilian
        } else {
            RampClass::Military
        };
        let progression = Progression(c.progression as u8);
        let group = if progression.by_group() && c.attack != 0 {
            c.where_
        } else {
            None
        };
        let mut price = Price {
            kind: cost::Kind::Unit,
            base: c.cost,
            support: [None; 2],
            progression,
            class,
            pop: c.pop,
        };
        for (res, n) in &c.support {
            price = price.with_support(*res, *n);
        }
        let mut roles = 0u32;
        let set = |roles: &mut u32, bit: u32, on: bool| {
            if on {
                *roles |= bit;
            }
        };
        set(&mut roles, role::CITIZEN, citizen(i));
        set(&mut roles, role::MILITIA, militia(i));
        set(&mut roles, role::V2ROCKET, is_named_unit(i, "V2 Rocket"));
        set(
            &mut roles,
            role::FACTORY_MADE,
            c.where_.is_some() && c.where_ == factory,
        );
        set(
            &mut roles,
            role::BARRACKS_MADE,
            c.where_.is_some() && c.where_ == barracks,
        );
        set(
            &mut roles,
            role::STABLE_MADE,
            c.where_.is_some() && c.where_ == stable,
        );
        set(&mut roles, role::BARK, is_named_unit(i, "Bark"));
        set(&mut roles, role::CATAPULT, is_named_unit(i, "Catapult"));
        set(
            &mut roles,
            role::FLAMETHROWER,
            is_named_unit(i, "Flamethrower"),
        );
        set(&mut roles, role::BOMBARD, is_named_unit(i, "Bombard"));
        set(&mut roles, role::MERCHANT, merchant);
        set(&mut roles, role::SUPPLY, is_supply);
        set(&mut roles, role::CARAVAN, is_caravan);
        set(&mut roles, role::IMMORTALS, is_named_unit(i, "Immortals"));
        set(
            &mut roles,
            role::PATROLBOAT,
            is_named_unit(i, "Patrol Boat"),
        );
        let age = age_of_tree(unit_tree[i]).max(0);
        let combat = Profile {
            attack: c.attack,
            to_hit: c.to_hit,
            attenuate: c.attenuate,
            min_range: c.min_range,
            max_range: c.max_range,
            second_max_range: c.second_max_range,
            splash_area: c.splash_area,
            splash_percent: c.splash_percent,
            ammo_per_att: c.ammo_per_att,
            recharge: c.recharge,
            armor: c.armor,
            proj_speed: c.proj_speed,
            obj_masks: c.obj_masks,
            roles,
            uber_size: c.uber_size,
            target_size: c.target_size,
            guy_radius: 0,
            domain: c.domain,
            siege: c.siege,
            packs: packs(i),
            age,
            x_size: 0,
            y_size: 0,
            base_arrows: 0,
            most_shots: 0,
            block_radius: c.block_radius,
            big_radius: 0,
            combat_role: c.attack != 0 && c.obj_masks & mask::CIVILIAN == 0,
            cost: c.cost.iter().sum::<i32>() * 10,
            build_class: BuildClass::Other,
        };
        let garrison = garrison::UnitTraits {
            fortify: c.flags & 0x1800 != 0,
            trained_at: c.where_,
            gov_hero: PATRIOTS.contains(&i),
            transport: c.flags & 0x10 != 0 && c.domain == Domain::Sea,
        };
        let _ = r;
        unit_types.push(UnitType {
            price,
            kind,
            group,
            hits: c.hits,
            times: c.times,
            combat,
            tree: Some(unit_tree[i]),
            garrison,
            moves: c.moves,
            turn_speed: c.turn_speed,
            // PEASANTS, PEASANTSKOREAN, SCHOLARS, SCHOLARSKOREAN — by id
            // (`TypeIndex − 0x32`), the four kinds `Build::add_gatherer`
            // admits (`docs/ORDERS.md` §6.1).
            worker: match i {
                0 | 1 => sim::orders::Worker::Citizen,
                2 | 3 => sim::orders::Worker::Scholar,
                _ => sim::orders::Worker::None,
            },
        });
    }

    // ---- the building types ----
    let ident_of = |i: usize| -> Ident {
        if WONDERS.contains(&i) && !build_names[i].eq_ignore_ascii_case("Forbidden City") {
            return Ident::Wonder;
        }
        match build_names[i].as_str() {
            "Small City" => Ident::Village,
            "Large City" => Ident::Town,
            "Major City" => Ident::Metropolis,
            "Forbidden City" => Ident::ForbiddenCity,
            "Farm" => Ident::Farm,
            "Woodcutter's Camp" => Ident::Woodcutter,
            "Mine" => Ident::Mine,
            "University" => Ident::University,
            "Oil Well" => Ident::OilWell,
            "Oil Platform" => Ident::OilPlatform,
            "Granary" => Ident::Granary,
            "Lumber Mill" => Ident::Lumbermill,
            "Smelter" => Ident::Smelter,
            "Refinery" => Ident::Refinery,
            "Library" => Ident::Library,
            "Market" => Ident::Market,
            "Temple" => Ident::Temple,
            "Senate" => Ident::Senate,
            "Tower" => Ident::Tower,
            "Fort" => Ident::Fort,
            "Lookout" => Ident::Lookout,
            "Barracks" => Ident::Barracks,
            "Stable" => Ident::Stable,
            "Auto Plant" => Ident::AutoPlant,
            "Siege Factory" => Ident::SiegeFactory,
            "Factory" => Ident::Factory,
            "Dock" => Ident::Dock,
            "Airbase" => Ident::Airbase,
            "Missile Silo" => Ident::MissileSilo,
            "Air Defense Gun" => Ident::AirDefense,
            "Red Fort" => Ident::RedFort,
            _ => Ident::Other,
        }
    };
    let trains_military: Vec<bool> = (0..build_names.len())
        .map(|bi| {
            unit_cols
                .iter()
                .any(|c| c.where_ == Some(bi) && c.attack != 0)
        })
        .collect();
    let trains_any: Vec<bool> = (0..build_names.len())
        .map(|bi| unit_cols.iter().any(|c| c.where_ == Some(bi)))
        .collect();
    // The building lineages the mechanics test, rooted by `TypeIndex` as the
    // calls name them (`is(VILLAGE 0x19e)`, `is(TOWER 0x1b7)`, `is(FORTX
    // 0x1bb)`, `is(AIRBASE 0x1bf)`, `is(LOOKOUT 0x209)`); building-table
    // order is `TypeIndex − 0x19e`.
    let is_city = |i: usize| build_is(i, VILLAGE);
    let is_tower = |i: usize| build_is(i, TOWER);
    let is_fort = |i: usize| build_is(i, FORTX);
    // `BuildType::set_domain`: `BUILD_FLAGS b` ("can be built on sea
    // squares", bit 1) makes a building Sea — or Air with `a` (bit 0) as
    // well — else Land. The four sea buildings are a third against the
    // Airbase (`type_damage` step 30), which is how the rule was found.
    let build_domain = |flags: u32| -> Domain {
        if flags & 2 != 0 {
            if flags & 1 != 0 {
                Domain::Air
            } else {
                Domain::Sea
            }
        } else {
            Domain::Land
        }
    };
    let mut build_types = Vec::with_capacity(buildings.len());
    let mut build_ages = Vec::with_capacity(buildings.len());
    for (i, _r) in buildings.records.iter().enumerate() {
        let c = &build_cols[i];
        let is_city = is_city(i);
        let is_fort = is_fort(i);
        let is_tower = is_tower(i);
        let mut price = Price {
            kind: cost::Kind::Building,
            base: c.cost,
            support: [None; 2],
            progression: Progression::default(),
            class: RampClass::default(),
            pop: 0,
        };
        for (res, n) in &c.support {
            price = price.with_support(*res, *n);
        }
        let mut roles = 0u32;
        if is_fort {
            roles |= role::FORT;
        }
        if is_city {
            roles |= role::CITY;
        }
        if build_names[i].eq_ignore_ascii_case("Red Fort") {
            roles |= role::REDFORT;
        }
        if build_names[i].eq_ignore_ascii_case("Supercollider") {
            roles |= role::SUPERCOLLIDER;
        }
        let build_class = if is_city {
            BuildClass::City
        } else if is_tower
            || is_fort
            || build_names[i].eq_ignore_ascii_case("Airbase")
            || c.attack != 0
        {
            BuildClass::Defensive
        } else if trains_military[i] {
            BuildClass::MilitaryTrainer
        } else if trains_any[i] {
            BuildClass::Training
        } else {
            BuildClass::Other
        };
        let age = age_of_tree(build_tree[i]).max(0);
        build_ages.push(age);
        let combat = Profile {
            attack: c.attack,
            to_hit: c.to_hit,
            attenuate: c.attenuate,
            min_range: c.min_range,
            max_range: c.max_range,
            second_max_range: 0,
            splash_area: c.splash_area,
            splash_percent: c.splash_percent,
            ammo_per_att: c.ammo_per_att,
            recharge: c.recharge,
            armor: c.armor,
            proj_speed: c.proj_speed,
            obj_masks: c.obj_masks,
            roles,
            uber_size: 0,
            target_size: 0,
            guy_radius: 0,
            // `BuildType::set_domain`: `BUILD_FLAGS b` is a sea building.
            domain: build_domain(c.flags),
            siege: false,
            packs: false,
            age,
            x_size: c.x_size,
            y_size: c.y_size,
            base_arrows: c.base_arrows,
            most_shots: c.most_shots,
            block_radius: 0,
            big_radius: 0,
            combat_role: false,
            cost: c.cost.iter().sum::<i32>() * 10,
            build_class,
        };
        build_types.push(BuildType {
            ident: ident_of(i),
            from: c.from,
            to: None,
            x_size: c.x_size,
            y_size: c.y_size,
            flags: c.flags,
            job_time: c.job_time,
            hits: c.hits,
            garrison_max: c.garrison_max,
            attack: c.attack,
            plunder_value: c.plunder,
            plunder_good: c.plunder_good.map(|r| r.index()),
            wonder: WONDERS.contains(&i),
            price,
            tree: Some(build_tree[i]),
            combat: Some(combat),
        });
    }
    // `BuildTypeData::to`: the successor by `FROM`, as `BuildType::init` sets
    // it — `B[from].to = this`, unconditionally, in record order, so the last
    // record naming a `FROM` wins. The Forbidden City (record 117) names the
    // Small City, after the Large City (record 1) has, and the program's
    // Small City therefore points at the Forbidden City. Reproduced as read.
    for i in 0..build_types.len() {
        if let Some(f) = build_types[i].from {
            build_types[f].to = Some(i);
        }
    }

    // `BuildType::init_final_flags@00632070`: the `FLAT` bit is **derived**,
    // not a `BUILD_FLAGS` letter — no shipped row carries `3`. It must run
    // after `from` is linked, because the test is a lineage `is()`.
    sim::build::init_final_flags(&mut build_types);

    // ---- the combat table over unit ids ----
    let t = Tuning::RON;
    let kinds: Vec<sim::balance::Kind> = (0..unit_types.len())
        .map(|i| {
            let c = &unit_cols[i];
            let mut lines = 0u64;
            for (bit, root, strict) in LINE_ROOTS {
                let on = match root {
                    Root::Ids(ids) => ids.contains(&i),
                    Root::Unit(ti) => {
                        let r = ti - 0x32;
                        if strict {
                            // `is_slow(x, 1)`: the type itself, or a direct
                            // graft of it when `x` is not a unique (`y`) unit.
                            i == r || (unit_graft[i] == Some(r) && unit_cols[r].flags & UNIQUE == 0)
                        } else {
                            unit_is(i, r)
                        }
                    }
                    Root::Build => false,
                };
                if on {
                    lines |= bit;
                }
            }
            sim::balance::Kind {
                masks: c.obj_masks,
                age: unit_types[i].combat.age,
                unit: true,
                build: false,
                wonder: false,
                gaia: i >= 352,
                siege: c.siege,
                caravan: caravan(i),
                domain: c.domain,
                lines,
                unit_index: Some(i),
            }
        })
        .collect();
    // The building kinds: `return_pack`'s line ladder (FORTS, then CITIES,
    // then OBSPOST, then TOWERS), the BUILDINGS object, `get_age`, the masks;
    // and for `type_damage`, the building and wonder ranges, the AIRBASE
    // lineage and the domain. A building has no siege/caravan flag.
    let build_kinds: Vec<sim::balance::Kind> = (0..build_types.len())
        .map(|i| {
            use sim::balance::line;
            let mut lines = 0u64;
            if build_is(i, FORTX) {
                lines |= line::FORT;
            }
            if build_is(i, VILLAGE) {
                lines |= line::CITY;
            }
            if build_is(i, LOOKOUT) {
                lines |= line::LOOKOUT;
            }
            if build_is(i, TOWER) {
                lines |= line::TOWER;
            }
            if build_is(i, AIRBASE) {
                lines |= line::AIRBASE;
            }
            sim::balance::Kind {
                masks: build_cols[i].obj_masks,
                age: build_ages[i],
                unit: false,
                build: true,
                // `BASE_WONDERTYPES 0x20e .. END_BUILDTYPES 0x21e`.
                wonder: i >= FIRST_WONDER,
                gaia: false,
                siege: false,
                caravan: false,
                domain: build_domain(build_cols[i].flags),
                lines,
                unit_index: None,
            }
        })
        .collect();
    let xml: Vec<i16> = match balance {
        Some(bx) => bx.table(&crate::balance::category_names(units)),
        None => vec![100; crate::balance::CATEGORIES * crate::balance::CATEGORIES],
    };
    let kind_of = |r: combat::TypeRef| match r {
        combat::TypeRef::Unit(i) => &kinds[i],
        combat::TypeRef::Build(i) => &build_kinds[i],
    };
    let table = combat::Table::build(unit_types.len(), build_types.len(), |a, b| {
        crate::balance::entry(&t, &xml, kind_of(a), kind_of(b))
    });

    Loaded {
        unit_types,
        build_types,
        tree,
        table,
        kinds,
        build_kinds,
        unit_names,
        build_names,
        tech_names,
        good_names,
        unit_type_names,
        build_type_names,
        unit_tree,
        build_tree,
        tech_tree,
        good_tree,
        warnings,
        map_styles: rules
            .categories
            .iter()
            .find(|(id, _)| id == "mapstyles")
            .map(|(_, t)| {
                t.records
                    .iter()
                    .map(|r| {
                        r.attrs
                            .iter()
                            .find(|(k, _)| k == "key")
                            .map(|(_, v)| v.clone())
                            .unwrap_or_default()
                    })
                    .collect()
            })
            .unwrap_or_default(),
        scripts: Vec::new(),
    }
}

/// `unit_flags` letter `y`, "a non-standard or unique unit": `is_slow(x, 1)`
/// refuses a graft match when the root carries it.
const UNIQUE: u32 = 0x0100_0000;

/// The building lineage roots `return_pack` and `type_damage` name, and the
/// first wonder, as building-table ids (`TypeIndex − BASE_BUILDTYPES`).
const BUILD0: usize = BASE_BUILDTYPES as usize;
const FIRST_WONDER: usize = 0x20e - BUILD0;
const VILLAGE: usize = 0x19e - BUILD0;
const TOWER: usize = 0x1b7 - BUILD0;
const FORTX: usize = 0x1bb - BUILD0;
const AIRBASE: usize = 0x1bf - BUILD0;
const LOOKOUT: usize = 0x209 - BUILD0;

/// Where a named lineage is rooted.
#[derive(Clone, Copy)]
enum Root {
    /// A unit `TypeIndex`, as the `is(x, …)` call names it.
    Unit(usize),
    /// A building `TypeIndex` (`AIRBASE`, 0x1bf); a building root never
    /// matches a unit type, and the table over buildings is not built yet.
    Build,
    /// A set of unit ids tested by equality, not lineage.
    Ids(&'static [usize]),
}

/// The named lineages `sim::balance::line` tests, rooted by the `TypeIndex`
/// constant each `ObjectTypeData::is(x, strict)` call in `Balance::type_damage`
/// passes (`docs/COMBAT.md` §5.3). Unit-table order is `TypeIndex` order, so a
/// root is `TypeIndex − 0x32` into the unit table — never a name: an earlier
/// draft keyed these by display name and rooted `ECOMPANION` (0xe8, Royal
/// Companion) at Companion, `BOMBARDSHIP` (0x15a, Bomb Vessel) at a name no
/// record carries, `CAMELRANGE2` (0xbf, Camel Archer) likewise and
/// `HALBERDIERS` (0x95, Scutari) at a different unit. `strict` is the
/// graft-only sense.
const LINE_ROOTS: [(u64, Root, bool); 43] = {
    use sim::balance::line::*;
    [
        (ARMOREDCAR, Root::Unit(0xd8), false),
        (LIGHTTANK, Root::Unit(0xef), false),
        (MACHINEGUN, Root::Unit(0x7b), false),
        (FLAMETHROWER, Root::Unit(0x83), false),
        (FLAMETHROWER_STRICT, Root::Unit(0x83), true),
        (MILITIA, Root::Unit(0x42), false),
        // MERCHANT, MERCHANTDUTCH, FURTRAPPER — by id.
        (
            MERCHANT,
            Root::Ids(&[0x3d - 0x32, 0x3e - 0x32, 0x190 - 0x32]),
            false,
        ),
        // PEASANTS, PEASANTSKOREAN, SCHOLARS, SCHOLARSKOREAN — by id.
        (CITIZEN, Root::Ids(&[0, 1, 2, 3]), false),
        (BOMBARDSHIP, Root::Unit(0x15a), false),
        (BARK, Root::Unit(0x143), false),
        (SUB, Root::Unit(0x152), false),
        (FIRERAFT, Root::Unit(0x14e), false),
        (TRIREME, Root::Unit(0x154), false),
        (BOMBER, Root::Unit(0x130), false),
        (FIGHTERBOMBER, Root::Unit(0x134), false),
        (HELICOPTER, Root::Unit(0x136), false),
        (V2ROCKET, Root::Unit(0x139), false),
        (SUPPLYWAGON, Root::Unit(0x3f), false),
        (BALAMOBSLINGERS, Root::Unit(0x58), false),
        (KUSHITEARCHERS, Root::Unit(0xae), false),
        (KUSHITEARCHERS_STRICT, Root::Unit(0xae), true),
        (INTICLUBMEN, Root::Unit(0x5b), false),
        (CAMELRANGE2, Root::Unit(0xbf), false),
        (CHARIOT, Root::Unit(0xc3), false),
        (NOMAD, Root::Unit(0xc7), false),
        (RUSINYLANCER, Root::Unit(0xe0), false),
        (LONGBOWMEN_STRICT, Root::Unit(0xb1), true),
        (ELONGBOWMEN_STRICT, Root::Unit(0xb2), true),
        (KINGSYEOMANRY_STRICT, Root::Unit(0xb3), true),
        (ECOMPANION, Root::Unit(0xe8), false),
        (LEGIONS, Root::Unit(0x92), false),
        (SAMURAI_STRICT, Root::Unit(0xa0), true),
        (HALBERDIERS, Root::Unit(0x95), false),
        (TERCIOS, Root::Unit(0x97), false),
        (RECOILGUN, Root::Unit(0x90), false),
        (HIGHLANDERS, Root::Unit(0x72), false),
        (MG42_STRICT, Root::Unit(0x82), true),
        (TIGERTANK_STRICT, Root::Unit(0x102), true),
        (LEOPARDTANK_STRICT, Root::Unit(0x103), true),
        (FLAMINGARROW, Root::Unit(0x117), false),
        (BASILICABOMBARD, Root::Unit(0x114), false),
        (MORTAR, Root::Unit(0x112), false),
        (AIRBASE, Root::Build, false),
    ]
};

/// `ObjectTypeData::is(x, 0)`: `t` is `root`, or its `from` chain or its
/// graft reaches `root`.
fn is_lineage(t: usize, root: usize, from: &[Option<usize>], graft: &[Option<usize>]) -> bool {
    let mut cur = Some(t);
    let mut guard = 0;
    while let Some(c) = cur {
        if c == root {
            return true;
        }
        if graft.get(c).copied().flatten() == Some(root) {
            return true;
        }
        cur = from.get(c).copied().flatten();
        guard += 1;
        if guard > 64 {
            break;
        }
    }
    false
}

/// The `Kind` of tech record `i`, from its position: seven ages, four lines
/// of seven epochs (Science, Commerce, Civic, Military in file order), four
/// finals, forty building techs, six governments in three tiers.
fn tech_kind(i: usize) -> Kind {
    match i {
        0..=6 => Kind::Age(i as u8),
        7..=34 => {
            let n = i - 7;
            let line = match n / 7 {
                0 => Line::Science,
                1 => Line::Commerce,
                2 => Line::Civic,
                _ => Line::Military,
            };
            Kind::Epoch {
                line,
                level: (n % 7) as u8,
            }
        }
        35..=38 => Kind::Final,
        79..=84 => {
            let n = i - 79;
            Kind::Gov {
                tier: (n / 2) as u8,
                column: (n % 2) as u8,
            }
        }
        _ => Kind::Plain,
    }
}

/// `TRIBE_MASK` as the bit set the tree keeps: bit *i* is nation *i*.
fn mask_bits(text: Option<&str>) -> u32 {
    match text {
        None => u32::MAX,
        Some(m) => tribe_mask(m)
            .into_iter()
            .filter(|&i| i < 32)
            .fold(0u32, |acc, i| acc | (1 << i)),
    }
}

/// The `FLAGS` string of a unit, as `UnitType::init` folds it: a lowercase
/// letter `c` sets bit `c − 'a'`, a digit `d` sets bit `d − '0' + 26`.
pub fn unit_flags(s: &str) -> u32 {
    s.trim().chars().fold(0u32, |acc, c| match c {
        'a'..='z' => acc | (1 << (c as u32 - 'a' as u32)),
        '0'..='9' => acc | (1 << (c as u32 - '0' as u32 + 26)),
        _ => acc,
    })
}

/// A resource word as `buildingrules.xml` and `resourcerules.xml` write it.
pub fn resource_word(s: &str) -> Option<Resource> {
    match s.trim().to_ascii_lowercase().as_str() {
        "food" => Some(Resource::Food),
        "timber" | "wood" => Some(Resource::Timber),
        "wealth" | "gold" => Some(Resource::Wealth),
        "knowledge" => Some(Resource::Knowledge),
        "metal" => Some(Resource::Metal),
        "oil" => Some(Resource::Oil),
        _ => None,
    }
}

fn to_sim_resource(r: crate::Resource) -> Resource {
    match r {
        crate::Resource::Food => Resource::Food,
        crate::Resource::Timber => Resource::Timber,
        crate::Resource::Gold => Resource::Wealth,
        crate::Resource::Knowledge => Resource::Knowledge,
        crate::Resource::Metal => Resource::Metal,
        crate::Resource::Oil => Resource::Oil,
    }
}

/// `Type::load_cost`: the six-slot array, written order, duplicates overwrite.
fn cost_slots(text: Option<&str>) -> [i32; RESOURCES] {
    let mut out = [0; RESOURCES];
    if let Some(c) = text.and_then(Cost::parse) {
        for (r, n) in c.0 {
            out[to_sim_resource(r).index()] = n;
        }
    }
    out
}

/// `RANGE`: `min-max`; no `-` means `max = min`.
fn range_of(text: Option<&str>) -> (i32, i32) {
    let Some(t) = text else {
        return (0, 0);
    };
    if let Some(r) = Range::parse(t) {
        return (r.min, r.max);
    }
    let n = Scalar::parse(t).map_or(0, Scalar::written_int);
    (n, n)
}

fn int(r: &Record, tag: &str) -> Option<i32> {
    r.text(tag).and_then(Scalar::parse).map(Scalar::written_int)
}

fn domain_of(text: Option<&str>) -> Domain {
    match text.map(str::trim).map(str::to_ascii_lowercase).as_deref() {
        Some("sea") | Some("naval") | Some("water") => Domain::Sea,
        Some("air") => Domain::Air,
        _ => Domain::Land,
    }
}

/// A name column that may be `none`/`disable`, resolved in a name list.
fn key(
    text: Option<&str>,
    names: &[String],
    what: &str,
    warnings: &mut Vec<String>,
) -> Option<usize> {
    let t = text?.trim();
    if t.is_empty() || t.eq_ignore_ascii_case("none") || t.eq_ignore_ascii_case("disable") {
        return None;
    }
    let found = find_name(names, t);
    if found.is_none() {
        warnings.push(format!("{what}: no type named {t:?}"));
    }
    found
}

/// The columns of one `UNIT` record, as `UnitType::init` reads them.
#[derive(Clone, Debug)]
struct UnitCols {
    flags: u32,
    obj_masks: u32,
    attack: i32,
    hits: i32,
    moves: i32,
    turn_speed: i32,
    cost: [i32; RESOURCES],
    support: Vec<(Resource, i32)>,
    progression: i32,
    pop: i32,
    times: Times,
    to_hit: i32,
    attenuate: i32,
    min_range: i32,
    max_range: i32,
    second_max_range: i32,
    splash_area: i32,
    splash_percent: i32,
    ammo_per_att: i32,
    recharge: i32,
    armor: i32,
    proj_speed: i32,
    uber_size: i32,
    target_size: i32,
    block_radius: i32,
    domain: Domain,
    siege: bool,
    from: Option<usize>,
    jump: Option<usize>,
    graft: Option<usize>,
    where_: Option<usize>,
}

impl UnitCols {
    fn read(
        _i: usize,
        r: &Record,
        unit_names: &[String],
        build_names: &[String],
        warnings: &mut Vec<String>,
    ) -> UnitCols {
        let flags = unit_flags(r.text("FLAGS").unwrap_or(""));
        let attack = int(r, "ATTACK").unwrap_or(0) * 10;
        let (min_range, mut max_range) = range_of(r.text("RANGE"));
        let mut second_max_range = 0;
        if flags & 0x400 != 0 {
            second_max_range = max_range;
            max_range = 0;
        }
        let mut proj_speed = int(r, "PROJ_SPEED").unwrap_or(0);
        if proj_speed == 0 && (max_range > 0 || second_max_range > 0) {
            proj_speed = 200;
        }
        let support = r
            .text("SUPPORT")
            .and_then(Cost::parse)
            .map(|c| {
                c.0.into_iter()
                    .map(|(res, n)| (to_sim_resource(res), n))
                    .collect()
            })
            .unwrap_or_default();
        let job_extra_time = r
            .text("JOB_EXTRA_TIME")
            .and_then(Scalar::parse)
            .map_or(0, |s| s.fraction(100));
        let research_premium_time = r
            .text("RESEARCH_PREMIUM_TIME")
            .and_then(Scalar::parse)
            .map_or(0, |s| s.fraction(256));
        UnitCols {
            flags,
            obj_masks: r.text("OBJ_MASK").map_or(0, mask::parse),
            attack,
            hits: int(r, "HITS").unwrap_or(0),
            moves: int(r, "MOVES").unwrap_or(0),
            turn_speed: degrees_to_angle(int(r, "TURN_SPEED").unwrap_or(0)).0,
            cost: cost_slots(r.text("COST")),
            support,
            progression: int(r, "PROGRESSION").unwrap_or(0),
            pop: int(r, "POP").unwrap_or(0),
            times: Times {
                job_time: int(r, "JOB_TIME").unwrap_or(0),
                research_premium_time,
                job_extra_time,
            },
            to_hit: int(r, "TO_HIT").unwrap_or(-1),
            attenuate: int(r, "ATTENUATE").unwrap_or(0).abs(),
            min_range,
            max_range,
            second_max_range,
            splash_area: int(r, "SPLASH").unwrap_or(0),
            splash_percent: int(r, "SPLASH_PERCENT").unwrap_or(100),
            ammo_per_att: int(r, "AMMO_PER_ATT").unwrap_or(0),
            recharge: int(r, "RECHARGE").unwrap_or(0),
            armor: int(r, "ARMOR").unwrap_or(0),
            proj_speed,
            uber_size: int(r, "UBER_SIZE").unwrap_or(1),
            target_size: int(r, "TARGET_SIZE").unwrap_or(0) * UNIT_BLOCK_RADIUS,
            block_radius: int(r, "BLOCK_RADIUS").unwrap_or(0) * UNIT_BLOCK_RADIUS,
            domain: domain_of(r.text("DOMAIN")),
            siege: flags & 0x20000 != 0,
            from: key(r.text("FROM"), unit_names, "unit_key FROM", warnings),
            jump: key(r.text("JUMP"), unit_names, "unit_key JUMP", warnings),
            graft: key(r.text("GRAFT"), unit_names, "unit_key GRAFT", warnings),
            where_: key(r.text("WHERE"), build_names, "build_key WHERE", warnings),
        }
    }
}

/// The columns of one `BUILDING` record, as `BuildType::init` reads them.
#[derive(Clone, Debug)]
struct BuildCols {
    flags: u32,
    obj_masks: u32,
    attack: i32,
    hits: i32,
    cost: [i32; RESOURCES],
    support: Vec<(Resource, i32)>,
    job_time: i32,
    garrison_max: i32,
    plunder: i32,
    plunder_good: Option<Resource>,
    x_size: i32,
    y_size: i32,
    to_hit: i32,
    attenuate: i32,
    min_range: i32,
    max_range: i32,
    splash_area: i32,
    splash_percent: i32,
    ammo_per_att: i32,
    recharge: i32,
    armor: i32,
    proj_speed: i32,
    base_arrows: i32,
    most_shots: i32,
    from: Option<usize>,
    jump: Option<usize>,
}

impl BuildCols {
    fn read(
        _i: usize,
        r: &Record,
        build_names: &[String],
        warnings: &mut Vec<String>,
    ) -> BuildCols {
        let (min_range, max_range) = range_of(r.text("RANGE"));
        let mut support = Vec::new();
        for n in 0..2 {
            let word = r.text(&format!("SUPPORT{n}"));
            let value = int(r, &format!("SUPPORTVALUE{n}")).unwrap_or(0);
            if let Some(res) = word.and_then(resource_word) {
                support.push((res, value));
            }
        }
        BuildCols {
            flags: build::flags::parse(r.text("BUILD_FLAGS").unwrap_or("")),
            obj_masks: r.text("OBJ_MASKS").map_or(0, mask::parse),
            attack: int(r, "ATTACK").unwrap_or(0) * 10,
            hits: int(r, "HITS").unwrap_or(0),
            cost: cost_slots(r.text("COST")),
            support,
            job_time: int(r, "JOB_TIME").unwrap_or(0),
            garrison_max: int(r, "GARRISON_MAX").unwrap_or(0),
            plunder: int(r, "PLUNDER").unwrap_or(0),
            plunder_good: r.text("PLUNDER_GOOD").and_then(resource_word),
            x_size: int(r, "X_SIZE").unwrap_or(1),
            y_size: int(r, "Y_SIZE").unwrap_or(1),
            to_hit: int(r, "TO_HIT").unwrap_or(-1),
            attenuate: int(r, "ATTENUATE").unwrap_or(0).abs(),
            min_range,
            max_range,
            splash_area: int(r, "SPLASH_AREA").unwrap_or(0),
            splash_percent: int(r, "SPLASH_PERCENT").unwrap_or(100),
            ammo_per_att: int(r, "AMMO_PER_ATT").unwrap_or(0),
            recharge: int(r, "RECHARGE").unwrap_or(0),
            armor: int(r, "ARMOR").unwrap_or(0),
            proj_speed: int(r, "PROJ_SPEED").unwrap_or(200),
            base_arrows: int(r, "BASE_ARROWS").unwrap_or(0),
            most_shots: int(r, "MOST_SHOTS").unwrap_or(0),
            from: key(r.text("FROM"), build_names, "build_key FROM", warnings),
            jump: key(r.text("JUMP"), build_names, "build_key JUMP", warnings),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_flags_fold_letters_and_digits() {
        assert_eq!(unit_flags("lmah"), (1 << 11) | (1 << 12) | 1 | (1 << 7));
        assert_eq!(unit_flags("r") & 0x20000, 0x20000);
        assert_eq!(unit_flags("k"), 0x400);
        assert_eq!(unit_flags("y"), 0x0100_0000);
        assert_eq!(unit_flags("0"), 1 << 26);
        assert_eq!(unit_flags(""), 0);
    }

    #[test]
    fn tech_kinds_by_position() {
        assert_eq!(tech_kind(0), Kind::Age(0));
        assert_eq!(tech_kind(6), Kind::Age(6));
        assert_eq!(
            tech_kind(7),
            Kind::Epoch {
                line: Line::Science,
                level: 0
            }
        );
        assert_eq!(
            tech_kind(34),
            Kind::Epoch {
                line: Line::Military,
                level: 6
            }
        );
        assert_eq!(tech_kind(35), Kind::Final);
        assert_eq!(tech_kind(39), Kind::Plain);
        assert_eq!(tech_kind(78), Kind::Plain);
        assert_eq!(tech_kind(79), Kind::Gov { tier: 0, column: 0 });
        assert_eq!(tech_kind(84), Kind::Gov { tier: 2, column: 1 });
    }

    #[test]
    fn ranges_without_a_dash_are_min_equals_max() {
        assert_eq!(range_of(Some("0-12rng")), (0, 12));
        assert_eq!(range_of(Some("3")), (3, 3));
        assert_eq!(range_of(None), (0, 0));
    }

    #[test]
    fn lineage_walks_from_and_graft() {
        // 0 ← 1 ← 2 (from), 3 grafts 1.
        let from = [None, Some(0), Some(1), None];
        let graft = [None, None, None, Some(1)];
        assert!(is_lineage(2, 0, &from, &graft));
        assert!(is_lineage(1, 1, &from, &graft));
        assert!(is_lineage(3, 1, &from, &graft));
        assert!(!is_lineage(3, 0, &from, &graft));
        assert!(!is_lineage(0, 2, &from, &graft));
    }

    #[test]
    fn tribe_mask_bits_are_lsb_first() {
        // Koreans are bit 16, the only one set here.
        assert_eq!(mask_bits(Some("000000010000000000000000")), 1 << 16);
        assert_eq!(mask_bits(None), u32::MAX);
    }

    #[test]
    fn type_index_round_trips_through_the_layout() {
        let l = Loaded {
            unit_types: vec![UnitType::default(); 3],
            build_types: vec![BuildType::default(); 2],
            tree: TechTree::new(),
            table: combat::Table::uniform(3),
            kinds: vec![],
            build_kinds: vec![],
            unit_names: vec![],
            build_names: vec![],
            tech_names: vec![],
            good_names: vec![],
            unit_type_names: vec![],
            build_type_names: vec![],
            unit_tree: vec![6, 7, 8],
            build_tree: vec![9, 10],
            tech_tree: vec![11, 12],
            good_tree: (0..6).collect(),
            warnings: vec![],
            map_styles: vec![],
            scripts: vec![],
        };
        assert_eq!(l.type_index(0), 0);
        assert_eq!(l.type_index(6), BASE_UNITTYPES);
        assert_eq!(l.type_index(8), BASE_UNITTYPES + 2);
        assert_eq!(l.type_index(9), BASE_BUILDTYPES);
        assert_eq!(l.type_index(11), BASE_TECHTYPES);
        assert_eq!(l.unit_of_type_index(BASE_UNITTYPES + 1), Some(1));
        assert_eq!(l.unit_of_type_index(BASE_UNITTYPES + 3), None);
        assert_eq!(l.build_of_type_index(BASE_BUILDTYPES + 1), Some(1));
    }

    /// The install, when it is where the tools expect it. Tests that need it
    /// return early otherwise, so `cargo test` passes on a machine without
    /// the game; `cargo run -p rondata -- <install>` is the check with teeth.
    fn install() -> Option<Install> {
        let root = std::env::var("RON_INSTALL").ok().or_else(|| {
            let here = env!("CARGO_MANIFEST_DIR");
            Some(format!("{here}/../../game"))
        })?;
        let i = Install::new(root);
        i.looks_valid().then_some(i)
    }

    #[test]
    fn the_shipped_tables_load_without_a_warning() {
        let Some(i) = install() else { return };
        let l = load(&i).unwrap();
        assert!(l.warnings.is_empty(), "{:?}", l.warnings);
        assert_eq!(l.unit_types.len(), 364);
        assert_eq!(l.build_types.len(), 129);
        assert_eq!(l.tech_names.len(), 85);
        assert_eq!(l.good_names.len(), 50);
        assert_eq!(l.tree.types.len(), 50 + 364 + 129 + 85);
    }

    #[test]
    fn the_citizen_loads_as_the_harness_transcribed_it() {
        let Some(i) = install() else { return };
        let l = load(&i).unwrap();
        let c = &l.unit_types[0];
        assert_eq!(l.unit_names[0], "Citizen");
        assert_eq!(c.hits, 40);
        assert_eq!(c.times.job_time, 50);
        assert_eq!(c.times.research_premium_time, 512);
        assert_eq!(c.times.job_extra_time, 10);
        assert_eq!(c.price.base[Resource::Food.index()], 2);
        assert_eq!(c.price.support[0], Some((Resource::Food, 1)));
        assert_eq!(c.price.support[1], None);
        assert_eq!(c.price.class, RampClass::Worker);
        assert_eq!(c.price.pop, 1);
        assert_eq!(c.combat.attack, 40);
        assert_eq!(c.moves, 25);
        assert!(c.garrison.fortify);
        assert_eq!(c.garrison.trained_at, l.build_named("Small City"));
        assert!(!c.kind.exempt_kind);
        assert_eq!(c.group, None);
        // Hoplites: attack 13 -> 130, a Barracks unit in a jump chain.
        let h = l.unit_named("Hoplites").unwrap();
        let ht = &l.unit_types[h];
        assert_eq!(ht.combat.attack, 130);
        assert_eq!(ht.combat.armor, 4);
        assert_eq!(ht.price.class, RampClass::Military);
        assert_eq!(ht.group, l.build_named("Barracks"));
        assert_eq!(ht.price.support[0], Some((Resource::Food, 1)));
        assert_eq!(ht.price.support[1], Some((Resource::Metal, 1)));
        let tree = &l.tree;
        let hd = &tree.types[l.unit_tree[h]];
        assert_eq!(hd.jump, Some(l.unit_tree[l.unit_named("Phalanx").unwrap()]));
        // The implicit Military epoch: Hoplites name no prerequisite, so
        // none is implied; Phalanx names the Classical Age and gets The Art
        // of War as its second.
        let p = l.unit_named("Phalanx").unwrap();
        let pd = &tree.types[l.unit_tree[p]];
        assert_eq!(pd.preq[0], Preq::Of(l.tech_tree[0]));
        assert_eq!(
            pd.preq[1],
            Preq::Of(l.tech_tree[l.tech_named("The Art of War").unwrap()])
        );
        // Pikemen name the Phalanx as their FROM, so the Phalanx's upgrade
        // back-link is the Pikemen; the Hoplites' is the Phalanx.
        assert_eq!(
            pd.upgrade,
            Some(l.unit_tree[l.unit_named("Pikemen").unwrap()])
        );
        // The Hoplites' own back-link is whichever record named them last:
        // `U[from].upgrade = this` in record order, as `UnitType::init` does.
        let up = hd.upgrade.unwrap();
        assert_eq!(tree.types[up].from, Some(l.unit_tree[h]));
        // And the keys resolve by TYPENAME: `FROM Marines` is the
        // Continental Marines.
        let cm = l.unit_named("Continental Marines").unwrap();
        assert_eq!(l.unit_type_names[cm], "Marines");
        assert_eq!(l.unit_named("Marines"), Some(cm));
    }

    #[test]
    fn the_buildings_load_their_lineages_and_flags() {
        let Some(i) = install() else { return };
        let l = load(&i).unwrap();
        let small = l.build_named("Small City").unwrap();
        let large = l.build_named("Large City").unwrap();
        let major = l.build_named("Major City").unwrap();
        assert_eq!(l.build_types[small].ident, Ident::Village);
        assert_eq!(l.build_types[large].from, Some(small));
        // The last FROM writer wins, as in `BuildType::init`: the Forbidden
        // City names the Small City after the Large City does.
        let forbidden = l.build_named("Forbidden City").unwrap();
        assert_eq!(l.build_types[forbidden].from, Some(small));
        assert_eq!(l.build_types[small].to, Some(forbidden));
        assert_eq!(l.build_types[large].to, Some(major));
        assert_eq!(l.build_types[small].x_size, 7);
        assert_eq!(l.build_types[small].hits, 1200);
        assert_eq!(l.build_types[small].garrison_max, 10);
        assert!(l.build_types[small].has(build::flags::NO_CITY));
        assert_eq!(
            l.build_types[small].price.support[0],
            Some((Resource::Food, 50))
        );
        assert_eq!(
            l.build_types[small].price.support[1],
            Some((Resource::Timber, 50))
        );
        let barracks = l.build_named("Barracks").unwrap();
        assert_eq!(l.build_types[barracks].plunder_value, 40);
        assert_eq!(
            l.build_types[barracks].plunder_good,
            Some(Resource::Food.index())
        );
        assert_eq!(l.build_types[barracks].job_time, 420);
        let keep = l.build_named("Keep").unwrap();
        assert!(build::is_tower(&l.build_types, keep));
        let castle = l.build_named("Castle").unwrap();
        assert!(build::is_fort(&l.build_types, castle));
        assert!(l.build_types[l.build_named("Pyramids").unwrap()].wonder);
        assert!(!l.build_types[barracks].wonder);
        assert_eq!(
            l.build_types[small].combat.unwrap().build_class,
            BuildClass::City
        );
        assert_eq!(
            l.build_types[barracks].combat.unwrap().build_class,
            BuildClass::MilitaryTrainer
        );
    }

    #[test]
    fn the_tree_files_ages_epochs_and_governments() {
        let Some(i) = install() else { return };
        let l = load(&i).unwrap();
        let t = &l.tree;
        assert_eq!(t.ages[0], Some(l.tech_tree[0]));
        assert_eq!(t.ages[6], Some(l.tech_tree[6]));
        assert_eq!(t.epochs[Line::Science.index()][0], Some(l.tech_tree[7]));
        assert_eq!(t.epochs[Line::Military.index()][0], Some(l.tech_tree[28]));
        assert_eq!(t.govs[0][0], Some(l.tech_tree[79]));
        assert_eq!(t.govs[2][1], Some(l.tech_tree[84]));
        assert_eq!(
            t.roles.senate,
            Some(l.build_tree[l.build_named("Senate").unwrap()])
        );
        assert_eq!(t.roles.fort_line.len(), 8);
        assert_eq!(t.roles.final_needs.len(), 5);
        assert_eq!(t.roles.german_industry.len(), 9);
        assert_eq!(t.tribes.len(), 24);
        // The Barracks needs The Art of War, and the tree id maps back.
        let b = l.build_named("Barracks").unwrap();
        let bd = &t.types[l.build_tree[b]];
        assert_eq!(
            bd.preq[0],
            Preq::Of(l.tech_tree[l.tech_named("The Art of War").unwrap()])
        );
        assert_eq!(l.type_index(l.build_tree[b]), BASE_BUILDTYPES + b as i32);
        assert_eq!(l.type_index(l.unit_tree[0]), BASE_UNITTYPES);
    }

    #[test]
    fn the_combat_table_is_not_uniform_and_a_known_pair_holds() {
        let Some(i) = install() else { return };
        let l = load(&i).unwrap();
        let hop = l.unit_named("Hoplites").unwrap();
        let bow = l.unit_named("Bowmen").unwrap();
        // The worked example `rondata` prints: with the file, Hoplites hit
        // Bowmen at a percentage that is not 100.
        assert_ne!(l.table.pct(hop, bow), 100);
        assert_eq!(l.table.width(), 364);
    }

    #[test]
    fn the_kinds_carry_the_program_s_ages_and_lineages() {
        use sim::balance::line;
        let Some(i) = install() else { return };
        let l = load(&i).unwrap();
        // `get_age_slow`: a unit needing the **Classical Age** tech is age 1,
        // not 0 — the age tech's own `AGE` column plus one. The Phalanx needs
        // it; the Hoplites it replaces is an Ancient-age unit with no tech
        // prerequisite at all, which `get_age_slow` resolves to 0. (The
        // example was the Hoplites and asserted 1, which the program's own
        // `UNITTYPE` dump contradicts — `--types` has both at 0 and 1 and
        // reports no difference.)
        let pha = l.unit_named("Phalanx").unwrap();
        assert_eq!(l.kinds[pha].age, 1);
        let hop = l.unit_named("Hoplites").unwrap();
        assert_eq!(l.kinds[hop].age, 0);
        let cit = l.unit_named("Citizen").unwrap();
        assert_eq!(l.kinds[cit].age, 0);
        // `ECOMPANION` (0xe8) is Royal Companion: Companion is not in the
        // lineage, Royal Companion and its `from` descendants are.
        let comp = l.unit_named("Companion").unwrap();
        let royal = l.unit_named("Royal Companion").unwrap();
        let strat = l.unit_named("Stratiotai").unwrap();
        assert_eq!(l.kinds[comp].lines & line::ECOMPANION, 0);
        assert_ne!(l.kinds[royal].lines & line::ECOMPANION, 0);
        assert_ne!(l.kinds[strat].lines & line::ECOMPANION, 0);
        // `BOMBARDSHIP` (0x15a) is the Bomb Vessel.
        let bv = l.unit_named("Bomb Vessel").unwrap();
        assert_ne!(l.kinds[bv].lines & line::BOMBARDSHIP, 0);
        // And the table they build: Citizen doubles against light infantry
        // (0x14000 is O|Q), not against the mounted General.
        let sling = l.unit_named("Slingers").unwrap();
        let general = l.unit_named("General").unwrap();
        assert_eq!(l.table.pct(cit, sling), 200);
        assert_eq!(l.table.pct(cit, general), 100);
        // The building roots, by index, are the records their names say.
        assert_eq!(l.build_names[VILLAGE], "Small City");
        assert_eq!(l.build_names[TOWER], "Tower");
        assert_eq!(l.build_names[FORTX], "Fort");
        assert_eq!(l.build_names[AIRBASE], "Airbase");
        assert_eq!(l.build_names[LOOKOUT], "Lookout");
        let castle = l.build_named("Castle").unwrap();
        assert_ne!(l.build_kinds[castle].lines & line::FORT, 0);
        let major = l.build_named("Major City").unwrap();
        assert_ne!(l.build_kinds[major].lines & line::CITY, 0);
        assert!(l.build_kinds[l.build_named("Pyramids").unwrap()].wonder);
        assert!(!l.build_kinds[castle].wonder);
        // Siege against a building: ×430 and the rest is the file.
        let cat = l.unit_named("Catapult").unwrap();
        assert!(
            l.table
                .pct_of(combat::TypeRef::Unit(cat), combat::TypeRef::Build(castle))
                > 100
        );
    }

    #[test]
    fn a_sim_can_be_built_from_the_load() {
        let Some(i) = install() else { return };
        let l = load(&i).unwrap();
        let s = l.sim(Tuning::RON, sim::World::new(4, 4), 2);
        assert_eq!(s.unit_types.len(), 364);
        assert_eq!(s.build_types.len(), 129);
        assert_eq!(s.tech_tree.types.len(), l.tree.types.len());
        assert_eq!(s.table.width(), 364);
    }
}
