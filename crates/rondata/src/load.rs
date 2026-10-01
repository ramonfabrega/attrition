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
use sim::ai_load;
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
/// `UNIT_FORMATION_SPACING` as loaded: `rules.xml`'s `1/16 tile` through
/// `Constants::init`'s `get_fraction(s, 0xc0)`, so **12** position units —
/// and `GroupData::log_data` prints `unit_formation_spacing 12` beside every
/// dumped group, which is the confirmation. `UnitType::init` multiplies the
/// `X_SPACING`/`Y_SPACING` columns by it (`docs/GROUPS.md` §6.4).
const UNIT_FORMATION_SPACING: i32 = 12;
/// `UNIT_GUY_SPACING` as loaded: `rules.xml` gives it the same `1/16 tile`
/// as `UNIT_FORMATION_SPACING`, so **12** position units.
/// `UnitType::init@0061ab50` multiplies the `GUY_SPACING` column by it, and
/// `Form::compute_dests`' follower arm is its only reader.
const UNIT_GUY_SPACING: i32 = 12;

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
    /// Each `resourcerules.xml` record's `BONUS_TYPE0/NUM0` and
    /// `BONUS_TYPE1/NUM1` — what standing on that good pays, before
    /// `LeaderData::calc_rare`'s two percentages (`sim::economy::GoodType`).
    pub good_types: Vec<sim::economy::GoodType>,
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
    /// The gaia types' animation lengths, `(TypeIndex, variant, slot) →
    /// frames`, read from `unit_graphics.xml`, `anim_graphics.xml` and the
    /// `.bha` files they name (`crate::artdata`). Empty when loaded from
    /// tables alone; it is the only art the loader reads, and the only
    /// place a length exists for a unit no dump has ever printed.
    pub gaia_lengths: crate::artdata::GaiaLengths,
    /// Every player unit graphic piece's `slot → frames`, read from the
    /// same three files (`crate::artdata::piece_lengths`). Empty when
    /// loaded from tables alone. Where a dump and this table both name a
    /// `(piece, slot)` they agree — `the_install_s_piece_lengths_match_
    /// the_dumps` — and where only this one does, it is the difference
    /// between an idle variant with its own length and one silently
    /// played as the default (`docs/ANIM.md` §3.2).
    pub piece_lengths: crate::artdata::PieceLengths,
    /// Every unit graphic piece's follow offset, `gpiece → (track_dx,
    /// track_dy)`, read from `unit_graphics.xml`'s `trackoffsetx` /
    /// `trackoffsety` / `scale` (`crate::artdata::piece_tracks`). Empty
    /// when loaded from tables alone. It is what tells a crew guy — a
    /// scout's dog, a machine gunner's loader — that it walks its own body
    /// behind its leader rather than standing on it (`docs/MOVEMENT.md`,
    /// "The follower's destination").
    pub piece_tracks: crate::artdata::PieceTracks,
    /// Every unit graphic piece's arrow-release frames, `gpiece → (attack
    /// slot → frames)`, read from `unit_graphics.xml`'s `<RELEASEEVENT>`
    /// (`crate::artdata::piece_releases`). Empty when loaded from tables
    /// alone. **Gameplay data, not art**: it is when the original's own
    /// simulation brings an arrow into existence, and a unit that does not
    /// reach one of these frames does not shoot (`docs/COMBAT.md` §9.0).
    pub piece_releases: crate::artdata::PieceReleases,
    /// The unit types that aim a pivot rather than turn to shoot,
    /// `TypeIndex → (node → (minangle, maxangle))`, from
    /// `unit_graphics.xml`'s `<RESTRICTION>` rows
    /// (`crate::artdata::pivot_restrictions`, `docs/COMBAT.md` §52).
    /// Gameplay data for the same reason the releases are: it decides
    /// whether `Unit::fight` turns the unit.
    pub pivot_restrictions: crate::artdata::PivotRestrictions,
    /// `effects_graphics.xml`'s sixteen `<MOUNTAIN>` templates, each one's
    /// tiles and solid cells as `MountainRange::init` derives them from its
    /// `TEMPLATE_TEX` (`crate::mountains`). Indexed as the dump's
    /// `mountain_types` is. Empty when loaded from tables alone.
    pub mountain_templates: Vec<sim::gather::MountainTemplate>,
    /// `craftrules.xml`'s 55 `CRAFT` records in file order — `TypeIndex`
    /// `0x275..=0x2ab`, the `SpellTypeData` table `Unit::do_cast` and
    /// `SpellTypeData::get_job_time` read (`docs/ORDERS.md` §6.9). Empty
    /// when the file is absent.
    pub spells: Vec<sim::orders::SpellType>,
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
        sim.good_types.clone_from(&self.good_types);
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

/// **The name group.** `Types::init@00669cc0` loads the unit table in *five*
/// passes and does not always hand a record its own XML element: it keeps the
/// index of the last record whose `NAME` differed from the running one, and on
/// passes 2, 3 and 4 a record whose `NAME` equals that one's is initialised
/// **from the leader's element**. Passes 0 and 1 — `NAME`, `GRAPH`,
/// `TYPENAME`; `WHERE`, `FROM`, `JUMP`, `TRIBE_MASK`, `GRAFT` — always use the
/// record's own.
///
/// So a run of consecutive records sharing a `NAME` — the nation art variants
/// — shares every other column with the first of the run, and four shipped
/// rows differ from what their own element says: the German General's `FLAGS`
/// (`lmhc`, loaded as `lmhcb`), Riflemen's `ARMOR` (1, loaded as 3), the
/// Anti-tank Rifle's and the Bazooka's `LOS` (12/14, loaded as 11/13) and the
/// Howitzer's `SPLASH_PERCENT` (33, loaded as 25). All four are confirmed
/// against the program's own type dump — reading each record's own column
/// gives 1, 1, 2 and 1 disagreements, and reading the leader's gives none
/// (`docs/DATALAYER.md`).
///
/// Returns each record's leader index. The building and technology tables get
/// two passes with no such comparison, so this applies to units alone.
fn name_group_leaders(names: &[String]) -> Vec<usize> {
    let mut leader: Vec<usize> = (0..names.len()).collect();
    let mut cur = 0;
    for i in 1..names.len() {
        if names[i] == names[cur] {
            leader[i] = leader[cur];
        } else {
            leader[i] = i;
            cur = i;
        }
    }
    leader
}

/// One named grid of `masks.txt` — the per-tile blocking templates
/// `BuildType::init_build_mask@006310b0` reads into `BuildType::mask`.
///
/// `x × y` bytes, row-major by `y`, exactly as
/// `BuildType::mask_me@006312a0` indexes them (`mask[y × x_size + x]`).
/// A byte of 1 is a `World::set_blocked_at(…, 1)`; anything else clears the
/// tile's `0x4000`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TileMask {
    pub x: i32,
    pub y: i32,
    pub cells: Vec<u8>,
}

/// Parses `masks.txt` — the file at the install's **root**, not under
/// `Data/`. Sections open with `#<name>`; `;` starts a comment; the two
/// numbers after the name are `x, y` and the next `x × y` items are the
/// grid, comma- or whitespace-separated. Several sections carry a **second**
/// grid after a blank line, which the original never reads: `init_build_mask`
/// takes exactly `x × y` items and stops. Non-numeric items (the `doobers`
/// grids spell `up, right, left, down`) come back 0, as `String::convert_int`
/// leaves them.
pub fn parse_masks_txt(text: &str) -> Vec<(String, TileMask)> {
    let mut out: Vec<(String, TileMask)> = Vec::new();
    let mut name: Option<String> = None;
    let mut items: Vec<&str> = Vec::new();
    let mut flush = |name: &mut Option<String>, items: &mut Vec<&str>| {
        let Some(n) = name.take() else {
            items.clear();
            return;
        };
        let num = |s: &&str| s.trim().parse::<i32>().unwrap_or(0);
        let x = items.first().map_or(0, num);
        let y = items.get(1).map_or(0, num);
        let want = (x.max(0) as usize) * (y.max(0) as usize);
        if x > 0 && y > 0 && items.len() >= want + 2 {
            let cells = items[2..want + 2]
                .iter()
                .map(|s| u8::try_from(s.trim().parse::<i32>().unwrap_or(0)).unwrap_or(0))
                .collect();
            out.push((n, TileMask { x, y, cells }));
        }
        items.clear();
    };
    for line in text.lines() {
        let l = line.split(';').next().unwrap_or("").trim();
        if let Some(rest) = l.strip_prefix('#') {
            flush(&mut name, &mut items);
            name = Some(rest.trim().to_string());
            continue;
        }
        if name.is_some() {
            items.extend(l.split(',').map(str::trim).filter(|s| !s.is_empty()));
        }
    }
    flush(&mut name, &mut items);
    out
}

/// Each building graphic's blocking-template name, from
/// `Data/building_graphics.xml`: the `mask=` attribute of every `BUILD`
/// element, keyed by the `GRAPH` its `name` starts with
/// (`WOODCUTTER-DEFAULT-AGE0` → `WOODCUTTER`), plus the `FARM` element,
/// which is written as its own tag with the attribute on a nested `AGE0`.
///
/// No shipped graphic gives two masks to one `GRAPH`, so the age and the
/// nation drop out and the map is a function of the rules row's `GRAPH`
/// alone (checked in `graphic_masks_are_one_per_graph`).
pub fn parse_graphic_masks(doc: &roxmltree::Document<'_>) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut push = |graph: &str, mask: &str| {
        if graph.is_empty() || mask.is_empty() {
            return;
        }
        if !out.iter().any(|(g, _)| g == graph) {
            out.push((graph.to_string(), mask.to_string()));
        }
    };
    for n in doc.descendants().filter(roxmltree::Node::is_element) {
        let Some(mask) = n.attribute("mask") else {
            continue;
        };
        let graph = match n.attribute("name") {
            // `<BUILD name="GRAPH-TRIBE-AGEn">`.
            Some(name) => name.split('-').next().unwrap_or("").to_string(),
            // `<FARM …><DEFAULT><AGE0 mask="…"/>`: the graphic's own tag.
            None => n
                .ancestors()
                .find(|a| {
                    a.is_element() && !matches!(a.tag_name().name(), "DEFAULT" | "AGE0" | "ROOT")
                })
                .map_or(String::new(), |a| a.tag_name().name().to_string()),
        };
        push(&graph, mask);
    }
    out
}

/// Reads every table of an install and loads them.
pub fn load(install: &Install) -> Result<Loaded, crate::Error> {
    let rules = install.rules()?;
    let units = install.units()?;
    let buildings = install.buildings()?;
    let techs = install.techs()?;
    let goods = install.resources()?;
    let balance = install.balance()?;
    let crafts = install.crafts().ok();
    let mut loaded = load_tables(
        &rules,
        &units,
        &buildings,
        &techs,
        &goods,
        Some(&balance),
        crafts.as_ref(),
    );
    // The per-tile blocking templates. Two files outside the rules tables —
    // `masks.txt` at the root and the graphics table's `mask=` — decide
    // which of a footprint's tiles take `0x4000`, and nothing in
    // `buildingrules.xml` does. Neither is required: without them every
    // type falls back on `BuildType::blocks`'s flat/non-flat line.
    let masks = crate::read(&install.root().join("masks.txt"))
        .map(|t| parse_masks_txt(&t))
        .unwrap_or_default();
    let gpath = install.data("building_graphics.xml");
    let gtext = crate::read(&gpath).unwrap_or_default();
    let graphics = crate::parse(&gpath, &gtext)
        .map(|d| parse_graphic_masks(&d))
        .unwrap_or_default();
    for (i, r) in buildings.records.iter().enumerate() {
        let Some(graph) = r.text("GRAPH").map(str::trim) else {
            continue;
        };
        let Some((_, name)) = graphics.iter().find(|(g, _)| g == graph) else {
            continue;
        };
        let Some((_, m)) = masks.iter().find(|(n, _)| n == name) else {
            loaded
                .warnings
                .push(format!("{graph}: masks.txt has no {name:?}"));
            continue;
        };
        let b = &mut loaded.build_types[i];
        if m.x != b.x_size || m.y != b.y_size {
            loaded.warnings.push(format!(
                "{graph}: mask {name:?} is {}×{}, the row is {}×{}",
                m.x, m.y, b.x_size, b.y_size
            ));
            continue;
        }
        b.block_mask.clone_from(&m.cells);
    }
    // The nations' names and art styles, from the per-nation files
    // rules.xml points at.
    let defs = install.tribe_defs(&rules)?;
    for (tribe, def) in loaded.tree.tribes.iter_mut().zip(defs) {
        tribe.name = def.name;
        tribe.unit_continent = def.unit_continent;
    }
    // The gaia types' animation lengths, straight from the install's own
    // graphics tables and the `.bha` files they name. It is the only art
    // the loader reads, and the only source there is for a unit no dump
    // has ever printed — gaia's bird (`crate::artdata`).
    loaded.gaia_lengths = crate::artdata::gaia_lengths(install);
    // And every player unit's, keyed by the graphic piece
    // `GraphicPieces::get_unit_gpiece` hands out — the `GRAPH` column of
    // each record placed by `init_piece_ranges`' arithmetic.
    let graphs: Vec<String> = units
        .records
        .iter()
        .map(|r| r.text("GRAPH").unwrap_or_default().trim().to_string())
        .collect();
    loaded.piece_lengths = crate::artdata::piece_lengths(install, &graphs);
    loaded.piece_tracks = crate::artdata::piece_tracks(install, &graphs);
    loaded.piece_releases = crate::artdata::piece_releases(install, &graphs);
    loaded.pivot_restrictions = crate::artdata::pivot_restrictions(install, &graphs);
    // An anti-air building's packet: the three `verify_load` draws as a
    // unit take their `<UNIT>`'s slots and releases (`crate::artdata::
    // wall_packets`, `docs/COMBAT.md` §84.2).
    let build_graphs: Vec<String> = buildings
        .records
        .iter()
        .map(|r| r.text("GRAPH").unwrap_or_default().trim().to_string())
        .collect();
    let packets = crate::artdata::wall_packets(install, &build_graphs);
    for (i, graph) in build_graphs.iter().enumerate() {
        let t = BASE_BUILDTYPES + i as i32;
        if !matches!(t, sim::air::AIRDEFENSE | sim::air::RADAR | sim::air::SAM) {
            continue;
        }
        let (Some(&id), Some((wind, swing, releases))) =
            (loaded.build_tree.get(i), packets.get(graph))
        else {
            continue;
        };
        if let Some(c) = loaded.tree.types[id].wall_cycle.as_mut() {
            c.wind = *wind as i32;
            c.swing = *swing as i32;
            c.releases.clone_from(releases);
        }
    }
    // The mountain templates: the placed ranges' tiles and solid cells,
    // which a mine's reach and its gather list are measured on.
    loaded.mountain_templates = crate::mountains::templates(install);
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
/// combat table is the hardcoded chain alone; `crafts` may be absent, in
/// which case no type is a caster and no building is a craft's home (the two
/// bits `craftrules.xml` seeds — `docs/DATALAYER.md`).
pub fn load_tables(
    rules: &Rules,
    units: &Table,
    buildings: &Table,
    techs: &Table,
    goods: &Table,
    balance: Option<&crate::balance::BalanceXml>,
    crafts: Option<&Table>,
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
    // `rules.xml`'s `<CATEGORIES id="unit_cats">`, by `key`: the nine unit
    // categories `UnitType::init` resolves `CAT` against.
    let unit_cats: Vec<String> = rules
        .categories
        .iter()
        .find(|(id, _)| id == "unit_cats")
        .map(|(_, t)| {
            t.records
                .iter()
                .map(|r| {
                    r.attrs
                        .iter()
                        .find(|(k, _)| k == "key")
                        .map(|(_, v)| v.trim().to_string())
                        .unwrap_or_default()
                })
                .collect()
        })
        .unwrap_or_default();
    // The name groups: a unit record whose `NAME` repeats the previous one's
    // takes every pass-2 column from the first of the run.
    let unit_leader = name_group_leaders(&unit_names);
    let unit_cols: Vec<UnitCols> = units
        .records
        .iter()
        .enumerate()
        .map(|(i, r)| {
            UnitCols::read(
                i,
                r,
                &units.records[unit_leader[i]],
                &unit_type_names,
                &build_type_names,
                &unit_cats,
                &mut warnings,
            )
        })
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
    let barracks = bname("Barracks");
    let stable = bname("Stable");
    let factory = bname("Factory");

    // ---- the derived type words: `role`, `unit_flags2` ----
    //
    // `docs/DATALAYER.md`, "The derived words no column carries". The roots
    // are `TypeIndex − 0x32` **by id**, as the original writes them: a display
    // name is not a reliable key here (`THECITIZEN` is *The Comrade*, the same
    // trap `docs/COMBAT.md` §15.2 caught four times).
    const SCOUT: usize = 0x45 - 0x32;
    const HOPLITES: usize = 0x84 - 0x32;
    const BARK: usize = 0x143 - 0x32;
    const CATAPULT: usize = 0x109 - 0x32;
    const FLAMINGARROW: usize = 0x117 - 0x32;
    const KATYUSHA: usize = 0x116 - 0x32;
    const MACHINEGUN: usize = 0x7b - 0x32;
    const FLAMETHROWER: usize = 0x83 - 0x32;
    const FISHERMEN: usize = 0x13d - 0x32;
    const MERCHANT: usize = 0x3d - 0x32;
    const MERCHANTDUTCH: usize = 0x3e - 0x32;
    const FURTRAPPER: usize = 0x190 - 0x32;
    const SUPPLYWAGON: usize = 0x3f - 0x32;
    const GOV_HEROES: usize = 0x160 - 0x32;
    const THEMONARCH: usize = 0x162 - 0x32;
    const THECITIZEN: usize = 0x164 - 0x32;
    const GENERAL: usize = 0x36 - 0x32;
    const CARA: usize = 0x3b - 0x32;
    const MERCHANTFLEET: usize = 0x13e - 0x32;
    const TRANSPORTBARGE: usize = 0x140 - 0x32;
    /// `BASE_GAIATYPES − BASE_UNITTYPES`: the first of the twelve animals.
    const GAIA: usize = 0x192 - 0x32;
    /// The Archers line's root, and the anti-air one's — the two lineages
    /// `ObjectData::train_time`'s British block tests beside the sea
    /// domain (`docs/PRODUCTION.md`, "The tail").
    const BOWMEN: usize = 0xaa - 0x32;
    const ANTIAIRCRAFTGUN: usize = 0x119 - 0x32;

    let mut cols: Vec<ai_load::UnitCols> = Vec::with_capacity(unit_cols.len());
    for (i, c) in unit_cols.iter().enumerate() {
        let (f2, uf_extra) = ai_load::flags2(&ai_load::Flags2Facts {
            machinegun: unit_is(i, MACHINEGUN),
            flamethrower: unit_is(i, FLAMETHROWER),
            packs_lineage: unit_is(i, CATAPULT)
                || unit_is(i, FLAMINGARROW)
                || unit_is(i, FISHERMEN)
                || unit_is(i, KATYUSHA),
            trader_id: i == MERCHANT || i == MERCHANTDUTCH || i == FURTRAPPER,
            supply_or_hero: unit_is(i, SUPPLYWAGON)
                || unit_is(i, GOV_HEROES)
                || unit_is(i, THEMONARCH)
                || unit_is(i, THECITIZEN),
            general: unit_is(i, GENERAL),
            caravan: unit_is(i, CARA) || unit_is(i, MERCHANTFLEET),
            scout: unit_is(i, SCOUT),
            transport: unit_is(i, TRANSPORTBARGE) || unit_is(i, MERCHANTFLEET),
        });
        let unit_flags = c.flags | uf_extra;
        cols.push(ai_load::UnitCols {
            unit_flags,
            unit_flags2: f2,
            cat: c.cat,
            carry: c.carry,
            research_premium_cost: c.research_premium_cost,
            role: ai_load::determine_roles(&ai_load::RoleFacts {
                citizen_id: i < 4,
                domain: c.domain,
                cat: c.cat,
                attack: c.attack,
                max_range: c.max_range,
                carry: c.carry,
                scout: unit_is(i, SCOUT),
                bark: unit_is(i, BARK),
                hoplite: unit_is(i, HOPLITES),
            }),
        });
    }
    // `SpellType::init` seeds the caster bit on each craft's `FROM`/`FROM2`,
    // and `init_spellcasters` walks it down the graft/from chains.
    let craft_records = crafts.map(|t| t.records.as_slice()).unwrap_or_default();
    // …and the record itself, which `Unit::do_cast` reads: the cast time
    // and the targeting letters (`docs/ORDERS.md` §6.9). The rows land in
    // file order, which is `TypeIndex` `0x275` upward.
    let spells: Vec<sim::orders::SpellType> = craft_records
        .iter()
        .map(|r| sim::orders::SpellType {
            job_time: int(r, "JOB_TIME")
                .and_then(|n| i16::try_from(n).ok())
                .unwrap_or(0),
            flags: craft_flags(r.text("FLAGS").unwrap_or("")),
            // `SpellType::init` keeps the range in internal units, a tile
            // being 192 (run246's `get_range` of 960 is the Informer's 10
            // tiles halved on a building).
            range: int(r, "SPELL_RANGE").unwrap_or(0) * 192,
            mana: int(r, "MANA").unwrap_or(0),
            // Seconds on file, frames in the record (`× 0xf`).
            duration: int(r, "DURATION").unwrap_or(0) * 15,
            duration_upgrade: int(r, "DURATION_UPGRADE").unwrap_or(0) * 15,
            from: ["FROM", "FROM2"].map(|col| {
                r.text(col)
                    .map(str::trim)
                    .and_then(uname)
                    .map(|u| BASE_UNITTYPES as usize + u)
            }),
        })
        .collect();
    let mut caster_seed = vec![false; unit_cols.len()];
    let mut craft_at: Vec<usize> = Vec::new();
    for r in craft_records {
        for col in ["FROM", "FROM2"] {
            let Some(text) = r.text(col) else { continue };
            let t = text.trim();
            if t.is_empty() || t.eq_ignore_ascii_case("none") || t.eq_ignore_ascii_case("disable") {
                continue;
            }
            if let Some(u) = uname(t) {
                caster_seed[u] = true;
            } else if let Some(b) = bname(t) {
                craft_at.push(b);
            } else {
                warnings.push(format!("craft_key {col}: no type named {t:?}"));
            }
        }
    }
    ai_load::spread_casters(&mut cols, &caster_seed, &unit_graft, &unit_from);

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
    // `GoodType::init`'s two bonus pairs. The column is a resource word or
    // `none`; the number is whole resources per period, which
    // `sim::economy::calc_rare` puts into sixteenths.
    let good_types: Vec<sim::economy::GoodType> = goods
        .records
        .iter()
        .map(|r| {
            let pair = |t: &str, n: &str| {
                (
                    r.text(t).and_then(resource_word),
                    r.text(n)
                        .and_then(|v| v.trim().parse::<i32>().ok())
                        .unwrap_or(0),
                )
            };
            sim::economy::GoodType {
                bonus: [
                    pair("BONUS_TYPE0", "BONUS_NUM0"),
                    pair("BONUS_TYPE1", "BONUS_NUM1"),
                ],
            }
        })
        .collect();
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
        // `PREQ0`/`PREQ1` are pass-2 columns, so they come from the name
        // group's leader like the rest ([`name_group_leaders`]); no shipped
        // group disagrees, but the rule is the rule.
        let l = &units.records[unit_leader[i]];
        d.preq[0] = tech_key(l.text("PREQ0"), &mut warnings);
        d.preq[1] = tech_key(l.text("PREQ1"), &mut warnings);
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
        // **An `ANTI_AIR` building other than a Lookout or an Observation
        // Post fires on its `Wall::inc_time` cycle** (item 1112,
        // `docs/COMBAT.md` §84): the rule is the type's, the packet the
        // art's, which [`load`] fills in. From the tables alone the cycle
        // has `get_game_frames`' three-frame answers and no release, so it
        // never fires.
        let t = BASE_BUILDTYPES + i as i32;
        if c.obj_masks & mask::ANTI_AIR != 0
            && t != sim::air::LOOKOUT
            && t != sim::air::OBSERVATIONPOST
        {
            let missing = sim::anim::MISSING as i32;
            d.wall_cycle = Some(sim::air::WallCycle {
                wind: missing,
                swing: missing,
                releases: Vec::new(),
                launch: sim::air::WALL_LAUNCH
                    .iter()
                    .find(|(k, _)| *k == t)
                    .map(|&(_, l)| l),
            });
        }
        let id = tree.add(d);
        debug_assert_eq!(id, build_tree[i]);
    }
    // Each tech's `WHERE` as a *building record*, which is what
    // `TechType::set_research` marks the lineage of.
    let mut tech_cols_where: Vec<Option<usize>> = Vec::with_capacity(techs.records.len());
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
        let where_rec = r.text("WHERE").and_then(|w| {
            let w = w.trim();
            if w.eq_ignore_ascii_case("none") || w.eq_ignore_ascii_case("disable") {
                return None;
            }
            let found = bname(w);
            if found.is_none() {
                warnings.push(format!("build_key: no building named {w:?}"));
            }
            found
        });
        tech_cols_where.push(where_rec);
        d.where_ = where_rec.map(|b| build_tree[b]);
        d.tribe_mask = mask_bits(r.text("TRIBE_MASK"));
        let id = tree.add(d);
        debug_assert_eq!(id, tech_tree[i]);
    }
    // The 24 nations. `Tribe::graft` is `UnitType::init`'s passes 2 and 3
    // over the unit records (`docs/TECH.md` §"The graft table", diffed
    // against run3's `DUMP_ALL`); `barbarian` is false, since the flag has
    // no located source (`docs/TECH.md`, "What is not established").
    // `TechTree::new` seeds one default tribe; the roster replaces it.
    let masks: Vec<u32> = units
        .records
        .iter()
        .map(|r| mask_bits(r.text("TRIBE_MASK")))
        .collect();
    let tables = tribe_grafts(&masks, &unit_graft, rules.tribes.len());
    tree.tribes.clear();
    for table in tables {
        let mut graft = vec![None; g + unit_names.len()];
        for (r, x) in table.into_iter().enumerate() {
            if x != r {
                graft[unit_tree[r]] = Some(unit_tree[x]);
            }
        }
        tree.add_tribe(tech::Tribe {
            graft,
            barbarian: false,
            name: String::new(),
            unit_continent: 0,
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
        r.village = bt("Small City");
        r.dock = bt("Dock");
        r.airbase = bt("Airbase");
        r.market = bt("Market");
        r.knowledge = gt("Knowledge");
        r.mathematics = tt("Mathematics");
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
        // `TAXATION` and the three after it, `0x24d..=0x250` — Taxation,
        // Vassalage, Social Contract and Income Tax in the shipped file.
        r.taxation_line = (0x24d..=0x250)
            .map(|t| (t - BASE_TECHTYPES) as usize)
            .filter(|&i| i < tech_tree.len())
            .map(|i| tech_tree[i])
            .collect();
        // `PYRAMIDS` through `SPACEPROGRAM`, `0x20e..=0x21e`: what
        // `LeaderData::has_wonder`'s argument names (`docs/ECONOMY.md` §15).
        r.wonder_line = (0x20e..=0x21e)
            .map(|t| t - BASE_BUILDTYPES as usize)
            .filter(|&i| i < build_tree.len())
            .map(|i| build_tree[i])
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

    // ---- `Leader::gain_tech` step 13: the nation free-upgrade blocks
    // whose candidates are a **predicate over unit types** rather than an
    // index range (`docs/TECH.md` §13).
    //
    // Each is `for t in BASE_UNITTYPES..0x192: <predicate> && has_preq(t)
    // && any i: get_preq(t, i, who) == gained -> type_eligible(t, 1) &&
    // gain_tech(t)`, which is [`tech::Shape::PreqMatch`]; the predicate is
    // static, so it becomes the candidate list here. The lineage ids are
    // the listing's own pushes, not the decompiler's dropped arguments:
    //
    // | block | constant | predicate |
    // | --- | --- | --- |
    // | Germans (12) | `+0x72c german_heavy_infantry` | Barracks, `is(0x99)` or `is(0x84)` (`6dfd05`, `6dfd18`) |
    // | Germans (12) | `+0x730 german_light_cavalry` | Barracks, `is(0xd1)` (`6dfded`) — **empty**: `0xd1` is Light Horse and a Light Horse is trained at the Stable |
    // | British (11) | `+0x6dc british_archer_upgrades` | Barracks, `is(0xaa)` (`6dfeba`) |
    // | Spanish (9) | `+0x694 spanish_scout_upgrades` | `is(0x45)` (`6dff72`) |
    // | Turks (8) | `+0x66c turk_free_siege_upgrades` | `is(0x109)` (`6e002f`) |
    //
    // `0x1ab` is the Barracks — the `where` column, not a lineage. The
    // German light-cavalry block's candidate list comes out **empty**, as
    // `docs/TECH.md` §13's row said it would: `0xd1` is Light Horse, and
    // the line is trained at the Stable, so the `where` test takes it all.
    // (The row's reason — "`is(HORSE, 0)`: none exist" — names the wrong
    // lineage; the emptiness is the `where`, and the test says so.)
    //
    // The **range** blocks — every row of §13's table whose candidates are
    // a run of tech indices — are not loaded: each endpoint is its own
    // reading and no capture reaches any of them.
    {
        let barracks = bname("Barracks").map(|i| build_tree[i]);
        let unit_ids: Vec<TypeId> = unit_tree.clone();
        // `is(x, 0)` over the loaded lineages, with the candidate's own
        // `where` where the block asks for one.
        let block = |tree: &TechTree, at_barracks: bool, lines: &[TypeId]| -> Vec<TypeId> {
            unit_ids
                .iter()
                .copied()
                .filter(|&t| {
                    if at_barracks && tree.types[t].where_ != barracks {
                        return false;
                    }
                    lines.iter().any(|&l| tree.is(t, l, false))
                })
                .collect()
        };
        let t = &Tuning::RON;
        // `german_light_cavalry` is **absent from `rules.xml`**, so the
        // original's `get_item` answers −1 and the gate is open
        // (`docs/TECH.md` §13). It has no `Tuning` slot for that reason.
        let rules = [
            (
                12,
                t.german_heavy_infantry,
                block(&tree, true, &[0x99, 0x84]),
            ),
            (12, -1, block(&tree, true, &[0xd1])),
            (11, t.british_archer_upgrades, block(&tree, true, &[0xaa])),
            (9, t.spanish_scout_upgrades, block(&tree, false, &[0x45])),
            (8, t.turk_free_siege_upgrades, block(&tree, false, &[0x109])),
        ];
        for (power, enabled, candidates) in rules {
            tree.free_rules.push(tech::FreeRule {
                gate: tech::Gate::Power(power),
                enabled,
                candidates,
                shape: tech::Shape::PreqMatch,
            });
        }
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
            crew_size: c.crew_size,
            target_size: c.target_size,
            circle_radius: c.circle_radius,
            // `ObjectType +0x23c` — `UnitType::init@0061ab50` stores
            // `unit_block_radius × BLOCK_RADIUS` to `+0x23c` and `+0x240`
            // alike (`:753`, item 1200), so a unit type's `guy_radius` is
            // its `block_radius`. `Ammo::do_damage`'s splash takes it off
            // a unit's distance (`678a94`): chapter forty-one's Biplane
            // on 858 (`docs/GOLDEN.md` §50).
            guy_radius: c.block_radius,
            domain: c.domain,
            fly_high: c.fly_high,
            fly_low: c.fly_low,
            siege: c.siege,
            // `needs_packing` is `unit_flags2 & 4`, which is now derived
            // exactly — the three trader ids by identity, everything else by
            // lineage (`docs/DATALAYER.md`).
            packs: cols[i].flag2(ai_load::uflags2::PACKS),
            age,
            x_size: 0,
            y_size: 0,
            x_spacing: c.x_spacing,
            y_spacing: c.y_spacing,
            guy_spacing: c.guy_spacing,
            base_arrows: 0,
            most_shots: 0,
            block_radius: c.block_radius,
            // `ObjectType +0x244` — `UnitType::init@0061ab50` computes it
            // as `(num_guys − 1) × guy_spacing / 2 + block_radius`
            // (`61ba93`..`61baac`), and `num_guys` (`+0x304`) is stored as
            // the literal **1** thirty instructions earlier (`61b9de`) with
            // nothing between, so in the shipped build the first term is
            // always zero and `big_radius` is `block_radius` for every unit
            // type. Nothing else in the export writes `+0x244`.
            big_radius: c.block_radius,
            push_size: c.push_size,
            push_circles: c.push_circles,
            // `role & 0x10000`, exactly — the `CIVILIAN` mask was the first
            // reading's stand-in and disagrees on two of the 364 (the Armed
            // Supply Wagon, which the `Civilian` *category* refuses, and
            // Boadicea, which carries the mask but is `Foot`).
            combat_role: cols[i].is(ai_load::role::MILITARY),
            hoplites: unit_is(i, HOPLITES),
            cost: c.cost.iter().sum::<i32>() * 10,
            build_class: BuildClass::Other,
        };
        let garrison = garrison::UnitTraits {
            fortify: c.flags & 0x1800 != 0,
            trained_at: c.where_,
            gov_hero: PATRIOTS.contains(&i),
            // `unit_flags & 0x10` on the **final** word: `init_final_flags`
            // adds the bit to the Transport Barge and Merchant Fleet lines.
            transport: cols[i].flag(ai_load::uflags::TRANSPORT) && c.domain == Domain::Sea,
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
            los: c.los,
            science_los: c.science_los,
            type_index: BASE_UNITTYPES + i as i32,
            // PEASANTS, PEASANTSKOREAN, SCHOLARS, SCHOLARSKOREAN — by id
            // (`TypeIndex − 0x32`), the four kinds `Build::add_gatherer`
            // admits (`docs/ORDERS.md` §6.1).
            worker: match i {
                0 | 1 => sim::orders::Worker::Citizen,
                2 | 3 => sim::orders::Worker::Scholar,
                _ => sim::orders::Worker::None,
            },
            // `ObjectData::train_time`'s British arm, by root id as the
            // original writes it — a display name is not a key here.
            archer: unit_is(i, BOWMEN),
            anti_air: unit_is(i, ANTIAIRCRAFTGUN),
            cols: cols[i],
            mana: int(r, "MANA").unwrap_or(0),
            gaia: i >= GAIA,
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
            // Not `default()`: the four `*_RAMP_MAX` ceilings belong to
            // `get_cost`'s unit arm, and a building's ramp has none
            // (`sim::cost::RampClass::Building`).
            class: RampClass::Building,
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
            crew_size: 0,
            target_size: 0,
            circle_radius: 0,
            guy_radius: 0,
            // `BuildType::set_domain`: `BUILD_FLAGS b` is a sea building.
            domain: build_domain(c.flags),
            fly_high: c.fly_high,
            fly_low: c.fly_low,
            siege: false,
            packs: false,
            age,
            x_size: c.x_size,
            y_size: c.y_size,
            // A building never stands in a formation: `ObjectType`'s
            // constructor leaves both at −1 and only `UnitType::init`
            // writes them.
            x_spacing: -1,
            y_spacing: -1,
            guy_spacing: -1,
            base_arrows: c.base_arrows,
            most_shots: c.most_shots,
            block_radius: 0,
            big_radius: 0,
            push_size: 0,
            push_circles: 0,
            hoplites: false,
            combat_role: false,
            cost: c.cost.iter().sum::<i32>() * 10,
            build_class,
        };
        build_types.push(BuildType {
            ident: ident_of(i),
            // `BuildData::get_shot@0062dd90`'s switch, by `TypeIndex`.
            shot: match 0x19e + i {
                0x1b9 | 0x1bd => Some(1),
                0x1ba | 0x20b | 0x20c => Some(2),
                0x1be => Some(4),
                0x209 | 0x20a => Some(0),
                0x20d => Some(5),
                _ => None,
            },
            from: c.from,
            to: None,
            x_size: c.x_size,
            y_size: c.y_size,
            los: c.los,
            science_los: c.science_los,
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
            // Filled by [`load`] from `masks.txt` and the graphics table;
            // the tables alone do not carry it.
            block_mask: Vec::new(),
        });
    }
    // `BuildTypeData::to`: the successor by `FROM`, as `BuildType::init` sets
    // it — `B[from].to = this` in record order, so the last record naming a
    // `FROM` wins, **except a record whose `obj_masks` carries `0x4000000`**
    // (`OBJ_MASK` letter `1`, the units' hero bit): the listing tests it and
    // skips the store (`00632da8`–`00632dbe`, `jne 0x632df1`). Two buildings
    // carry it and name a `FROM`, the Forbidden City (record 117, `FROM`
    // the Small City) and the Red Fort (120, `FROM` the Fort), so the Small
    // City's `to` stays the Large City and the Fort's the Castle. Linking
    // them anyway left `get_buildings(VILLAGE)` blind to every Large City
    // and the AI offered no Citizen once its last Small City grew (item
    // 1326, East Indies 12580; `docs/CITIES.md` §13 item 14).
    for i in 0..build_types.len() {
        if let Some(f) = build_types[i].from
            && build_cols[i].obj_masks & 0x0400_0000 == 0
        {
            build_types[f].to = Some(i);
        }
    }

    // The derived half of `build_flags` — `docs/DATALAYER.md`, "The derived
    // words no column carries". **No shipped `BUILD_FLAGS` string contains a
    // digit**, so every bit above 25 is one of these six, and reading the
    // column alone leaves the Library's queue two deep and the Barracks no
    // trainer. It must run after `from` is linked: three of the six are
    // lineage tests.
    let derived = sim::build::Derived {
        trains: unit_cols
            .iter()
            .enumerate()
            .filter(|(i, c)| c.where_.is_some() && !cols[*i].flag(ai_load::uflags::NO_PRODUCE))
            .filter_map(|(_, c)| c.where_)
            .collect(),
        military_trains: unit_cols
            .iter()
            .enumerate()
            .filter(|(i, c)| {
                let w = match c.where_ {
                    Some(w) => w,
                    None => return false,
                };
                !cols[*i].flag(ai_load::uflags::NO_PRODUCE)
                    && c.attack != 0
                    && !cols[*i].is(ai_load::role::CIVILIAN)
                    && !build_is(w, VILLAGE)
            })
            .filter_map(|(_, c)| c.where_)
            .collect(),
        research_at: tech_cols_where.iter().copied().flatten().collect(),
        craft_at,
    };
    sim::build::init_derived_flags(&mut build_types, &derived);

    // `TechType::compute_ai_values` — the eleven weights per tech, from what
    // each tech unlocks. It runs last of all the loader's passes because it
    // reads every one of them: the `role` words, the derived `build_flags`,
    // the finalised tree. The two classes the tree does not carry come in as
    // prerequisites alone: the 55 crafts and `rules.xml`'s 122 `TECHBONUSES`,
    // in file order (the first bonus scores twice).
    let extra_preq = |p: Option<&str>| -> Preq {
        match p.map(str::trim) {
            None | Some("") => Preq::None,
            Some(t) if t.eq_ignore_ascii_case("none") => Preq::None,
            Some(t) if t.eq_ignore_ascii_case("disable") => Preq::Disabled,
            Some(t) => match find_name(&tech_names, t) {
                Some(i) => Preq::Of(tech_tree[i]),
                None => Preq::Disabled,
            },
        }
    };
    let spell_preqs: Vec<ai_load::ExtraPreq> = craft_records
        .iter()
        .map(|r| {
            [
                extra_preq(r.text("PREQ0")),
                extra_preq(r.text("PREQ1")),
                extra_preq(r.text("PREQ2")),
            ]
        })
        .collect();
    let bonus_preqs: Vec<ai_load::ExtraPreq> = rules
        .tech_bonuses
        .records
        .iter()
        .map(|r| {
            let f = r.field("PREQ");
            [
                extra_preq(f.and_then(|f| f.attr("preq0"))),
                extra_preq(f.and_then(|f| f.attr("preq1"))),
                extra_preq(f.and_then(|f| f.attr("preq2"))),
            ]
        })
        .collect();
    // `TRANSPORT_BONUS` is the third bonus (`0x2ae − 0x2ac`) and its `preq0`
    // is what `has_preq` on it tests (`docs/TRANSPORT.md` §4, §13).
    tree.roles.transport_preq = match bonus_preqs.get(2) {
        Some([Preq::Of(t), ..]) => Some(*t),
        _ => None,
    };
    // `COLONIZE_BONUS` is the fourth (`0x2af`), and Coinage is its `preq0`
    // in the shipped file — the gate on a leader's **first city in a new
    // region** (`docs/CITIES.md` §2.6.1).
    tree.roles.colonize_preq = match bonus_preqs.get(3) {
        Some([Preq::Of(t), ..]) => Some(*t),
        _ => None,
    };
    // `MISSILE_DEFENSE_BONUS` is the tenth (`0x2b5`, pushed at `6fbbe9` and
    // `678343`), and Missile Shield is its `preq0` in the shipped file — a
    // missile's order on the holder's object, and its blast in the
    // holder's land (`docs/PRODUCTION.md`, "The missile's other arms").
    tree.roles.missile_defense_preq = match bonus_preqs.get(9) {
        Some([Preq::Of(t), ..]) => Some(*t),
        _ => None,
    };
    // `FISHERMEN1..3` are bonuses 19–21 (`0x2bf`–`0x2c1`) and
    // `MERCHANTS_1..4` are 99–102 (`0x30f`–`0x312`) — the two upgrade
    // ladders `LeaderData::calc_rare` indexes its percentages with
    // (`docs/ECONOMY.md`, step 6).
    let bonus_at = |i: usize| match bonus_preqs.get(i) {
        Some([Preq::Of(t), ..]) => Some(*t),
        _ => None,
    };
    // `LeaderData::get_gov@006d6a20` tests six government bonuses, in
    // this order, and answers a government `TypeIndex` for each:
    // `SOCIALISM_1` (`0x324`) → `0x273`, `CAPITALISM_1` (`0x325`) →
    // `0x274`, `MONARCHY_1` (`0x320`) → `0x271`, `DEMOCRACY_1` (`0x322`) →
    // `0x272`, `DESPOTISM_1` (`0x31a`) → `0x26f`, `REPUBLIC_1` (`0x31d`) →
    // `0x270`. Bonuses sit at `0x2ac` + their `TECHBONUSES` row; techs at
    // `BASE_TECHTYPES` + their record.
    tree.roles.gov_bonuses = [
        (0x324, 0x273),
        (0x325, 0x274),
        (0x320, 0x271),
        (0x322, 0x272),
        (0x31a, 0x26f),
        (0x31d, 0x270),
    ]
    .iter()
    .filter_map(|&(bonus, gov): &(usize, usize)| {
        let preqs = *bonus_preqs.get(bonus - 0x2ac)?;
        let gov = *tech_tree.get(gov - BASE_TECHTYPES as usize)?;
        Some((preqs, gov))
    })
    .collect();
    // Democracy's non-library research price: preserve all prerequisites,
    // as get_cost's has_preq does, with tier two taking precedence.
    tree.roles.democracy_preqs = [
        bonus_preqs.get(0x322 - 0x2ac).copied(),
        bonus_preqs.get(0x323 - 0x2ac).copied(),
    ];
    tree.roles.fishermen_preq = [bonus_at(19), bonus_at(20), bonus_at(21)];
    tree.roles.merchants_preq = [bonus_at(99), bonus_at(100), bonus_at(101), bonus_at(102)];
    // `REPUBLIC_1..3` (`0x31d`–`0x31f`): the commerce cap's republic term
    // (`docs/AI.md` §72).
    tree.roles.republic_preq = [bonus_at(113), bonus_at(114), bonus_at(115)];
    // `DESPOTISM_1..3` (`0x31a`–`0x31c`): the military line's price tail
    // (`docs/AI.md` §99.8).
    tree.roles.despotism_preq = [bonus_at(110), bonus_at(111), bonus_at(112)];
    // And the five ladders `docs/ECONOMY.md` indexes by — `GRANARY2..5`,
    // `LUMBERMILL2..4`, `SMELTER2..4`, `UNIVERSITY2..6` and `TAX_1..4`, at
    // `0x2bb`, `0x2c2`, `0x2c5`, `0x2e1` and `0x30b` off `BASE_BONUSTYPES`
    // (`crate::tech::Roles::granary_preq`). They bracket the two above.
    tree.roles.granary_preq = [bonus_at(15), bonus_at(16), bonus_at(17), bonus_at(18)];
    tree.roles.lumbermill_preq = [bonus_at(22), bonus_at(23), bonus_at(24)];
    tree.roles.smelter_preq = [bonus_at(25), bonus_at(26), bonus_at(27)];
    tree.roles.university_preq = [
        bonus_at(53),
        bonus_at(54),
        bonus_at(55),
        bonus_at(56),
        bonus_at(57),
    ];
    tree.roles.taxation_preq = [bonus_at(95), bonus_at(96), bonus_at(97), bonus_at(98)];
    // `TEMPLEBORDERS2..4` (`0x2c8`) and `FORTBORDERS2..4` (`0x2d1`): the
    // border levels `compute_reg_territory` reads (item 552).
    tree.roles.temple_borders_preq = [bonus_at(28), bonus_at(29), bonus_at(30)];
    tree.roles.fort_borders_preq = [bonus_at(37), bonus_at(38), bonus_at(39)];
    // `ATTRITION1..4` (`0x2dd`): the steps `Leader::calc_attrition` counts.
    tree.roles.attrition_preq = [bonus_at(49), bonus_at(50), bonus_at(51), bonus_at(52)];
    // `TROOPS_LOS_1..3` (`0x2e9`): the levels `get_troops_los_upgrade`
    // counts (`docs/VISION.md` §2, term 6).
    tree.roles.troops_los_preq = [bonus_at(61), bonus_at(62), bonus_at(63)];
    // The three speed-upgrade ladders `train_time` counts, rows 64–66,
    // 58–60 and 76–78 (`docs/PRODUCTION.md`, "The tail's first caller").
    tree.roles.ships_speed_preq = [bonus_at(64), bonus_at(65), bonus_at(66)];
    tree.roles.troops_speed_preq = [bonus_at(58), bonus_at(59), bonus_at(60)];
    tree.roles.vehicles_speed_preq = [bonus_at(76), bonus_at(77), bonus_at(78)];
    ai_load::compute_ai_values(
        &mut tree,
        &tech::Setup::STANDARD,
        &unit_types,
        &build_types,
        &spell_preqs,
        &bonus_preqs,
    );

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
        good_types,
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
        gaia_lengths: Default::default(),
        piece_lengths: Default::default(),
        piece_tracks: Default::default(),
        piece_releases: Default::default(),
        pivot_restrictions: Default::default(),
        mountain_templates: Vec::new(),
        spells,
    }
}

/// `craftrules.xml`'s `FLAGS` column: the file's own legend runs `a`
/// through `m` — researched, targets units, buildings, an area, friendly,
/// enemy, no-cancel, allied, vehicles, non-vehicles, at-peace, no-cloak-
/// blow, group-preference — and a letter is bit `letter - 'a'`. `do_cast`
/// reads `& 0xe` (units, buildings, area) to tell a targeted craft from an
/// untargeted one, and `& 0x800` for the cloak.
///
/// A letter outside `a..=m` would be a column this reading does not know,
/// so it is dropped rather than folded in; the shipped file has none.
fn craft_flags(text: &str) -> u32 {
    text.bytes()
        .filter(|c| c.is_ascii_lowercase() && *c <= b'm')
        .fold(0u32, |f, c| f | 1 << (c - b'a'))
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
/// `Tribe::graft[352]` for every nation, in record space: entry `r` of
/// table `i` is the record nation `i` trains when it is asked for `r`.
///
/// `Tribe::init@006f0230` lays each table down as the identity, and
/// `UnitType::init@0061ab50` then fills it over two of its five passes,
/// each pass over every record in order:
///
/// - **pass 2**: a record with a `GRAFT` writes itself
///   into its graft's slot of **every nation in its own `TRIBE_MASK`** —
///   whatever the graft's mask says. So the British Longbowmen, whose
///   graft is Archers, make `graft[Archers]` the Longbowmen for the
///   British.
/// - **pass 3**: for each record `this` and each nation
///   `i` with `graft[this] ≠ this` and `this` outside `i`'s mask, every
///   **other** record `u` below `0x192 − 0x32` that is outside `i`'s mask
///   and whose own `GRAFT` is `this` takes `graft[this]` too. One level
///   per record, in record order — a chain resolves only as far as the
///   order carries it, and that is the original's answer.
///
/// `Types::finalize_grafting` runs after both and rewrites masks, not
/// these tables.
fn tribe_grafts(masks: &[u32], grafts: &[Option<usize>], tribes: usize) -> Vec<Vec<usize>> {
    /// `Tribe::graft` is 352 entries, `BASE_UNITTYPES` to `0x192`.
    const SLOTS: usize = 0x192 - 0x32;
    let n = masks.len();
    let mut t: Vec<Vec<usize>> = (0..tribes).map(|_| (0..SLOTS).collect()).collect();
    let bit = |i: usize| 1u32.checked_shl(i as u32).unwrap_or(0);
    for r in 0..n {
        let Some(gr) = grafts[r].filter(|&x| x < SLOTS) else {
            continue;
        };
        for (i, table) in t.iter_mut().enumerate() {
            if masks[r] & bit(i) != 0 {
                table[gr] = r;
            }
        }
    }
    for this in 0..n.min(SLOTS) {
        for (i, table) in t.iter_mut().enumerate() {
            if table[this] == this || masks[this] & bit(i) != 0 {
                continue;
            }
            for u in 0..n.min(SLOTS) {
                if u != this && masks[u] & bit(i) == 0 && grafts[u] == Some(this) {
                    table[u] = table[this];
                }
            }
        }
    }
    t
}

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
        // `1`..`9` only, and the shift is `+25` — the original's digit branch
        // is `c − 0x17`, so `1` is bit 26 and not bit 27. Corrected
        // 2026-08-25 against the six patriots, the only shipped rows with a
        // digit: `lmhcbp1` is `0x4009886`, not `0x8009886`.
        '1'..='9' => acc | (1 << (c as u32 - '0' as u32 + 25)),
        // `0` is below the digit branch's `< 0x31` test and falls into the
        // *letter* one, where `(0x30 − 0x61) & 0x1f` is 15 — the `p` bit. No
        // shipped row has one; reproduced rather than corrected.
        '0' => acc | (1 << 15),
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
    /// `LOS` and `SCIENCE_LOS` — `docs/VISION.md` §2. In tiles.
    los: i32,
    science_los: i32,
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
    crew_size: i32,
    target_size: i32,
    /// `CIRCLE_RADIUS` — `ObjectTypeData::x_size` for a unit type, the
    /// ring index of `Unit::update_local_seen`'s reveal
    /// (`docs/VISION.md` §7). `UnitType::init@0061ab50:655`-`662` clamps
    /// it to 10 and writes `+0x234` and `+0x238` with the same value.
    circle_radius: i32,
    block_radius: i32,
    /// `PUSH_SIZE` and `PUSH_CIRCLES` as `UnitType::init@0061ab50:664`-`691`
    /// stores them: the circles clamped to `[1, 100]`; the size clamped
    /// the same way, **replaced by `BLOCK_RADIUS` when there is one
    /// circle**, and then times `UNIT_BLOCK_RADIUS`. Checked against the
    /// start dump's own `push_size`/`push_circles` for every type
    /// (`every_unit_type_s_push_profile_is_the_original_s`).
    push_size: i32,
    push_circles: i32,
    x_spacing: i32,
    y_spacing: i32,
    guy_spacing: i32,
    domain: Domain,
    /// `FLY_HIGH` and `FLY_LOW`, `get_text_num(…, -1)`.
    fly_high: i32,
    fly_low: i32,
    siege: bool,
    from: Option<usize>,
    jump: Option<usize>,
    graft: Option<usize>,
    where_: Option<usize>,
    /// `CAT`, as an index into `rules.xml`'s `unit_cats` — the column
    /// `determine_roles` branches on (`docs/DATALAYER.md`).
    cat: i32,
    /// `CARRY`.
    carry: i32,
    /// `RESEARCH_PREMIUM_COST`, 8.8 — `UnitTypeData +0x2e0`.
    research_premium_cost: i32,
}

impl UnitCols {
    /// `r` is the record's own element and `l` its **name group's leader** —
    /// see [`name_group_leaders`]. Only `FROM`, `JUMP`, `GRAFT` and `WHERE`
    /// are read from `r`; every other column here is a pass-2 read and comes
    /// from `l`.
    fn read(
        _i: usize,
        r: &Record,
        l: &Record,
        unit_names: &[String],
        build_names: &[String],
        unit_cats: &[String],
        warnings: &mut Vec<String>,
    ) -> UnitCols {
        let flags = unit_flags(l.text("FLAGS").unwrap_or(""));
        let attack = int(l, "ATTACK").unwrap_or(0) * 10;
        let (min_range, mut max_range) = range_of(l.text("RANGE"));
        let mut second_max_range = 0;
        if flags & 0x400 != 0 {
            second_max_range = max_range;
            max_range = 0;
        }
        let mut proj_speed = int(l, "PROJ_SPEED").unwrap_or(0);
        if proj_speed == 0 && (max_range > 0 || second_max_range > 0) {
            proj_speed = 200;
        }
        let support = l
            .text("SUPPORT")
            .and_then(Cost::parse)
            .map(|c| {
                c.0.into_iter()
                    .map(|(res, n)| (to_sim_resource(res), n))
                    .collect()
            })
            .unwrap_or_default();
        let job_extra_time = l
            .text("JOB_EXTRA_TIME")
            .and_then(Scalar::parse)
            .map_or(0, |s| s.fraction(100));
        let research_premium_time = l
            .text("RESEARCH_PREMIUM_TIME")
            .and_then(Scalar::parse)
            .map_or(0, |s| s.fraction(256));
        // The next key `UnitType::init` reads, the same way (`:606`–`613`).
        let research_premium_cost = l
            .text("RESEARCH_PREMIUM_COST")
            .and_then(Scalar::parse)
            .map_or(0, |s| s.fraction(256));
        UnitCols {
            flags,
            obj_masks: l.text("OBJ_MASK").map_or(0, mask::parse),
            attack,
            hits: int(l, "HITS").unwrap_or(0),
            moves: int(l, "MOVES").unwrap_or(0),
            turn_speed: degrees_to_angle(int(l, "TURN_SPEED").unwrap_or(0)).0,
            los: int(l, "LOS").unwrap_or(0),
            science_los: int(l, "SCIENCE_LOS").unwrap_or(0),
            cost: cost_slots(l.text("COST")),
            support,
            progression: int(l, "PROGRESSION").unwrap_or(0),
            pop: int(l, "POP").unwrap_or(0),
            times: Times {
                job_time: int(l, "JOB_TIME").unwrap_or(0),
                research_premium_time,
                job_extra_time,
            },
            to_hit: int(l, "TO_HIT").unwrap_or(-1),
            attenuate: int(l, "ATTENUATE").unwrap_or(0).abs(),
            min_range,
            max_range,
            second_max_range,
            splash_area: int(l, "SPLASH").unwrap_or(0),
            splash_percent: int(l, "SPLASH_PERCENT").unwrap_or(100),
            ammo_per_att: int(l, "AMMO_PER_ATT").unwrap_or(0),
            recharge: int(l, "RECHARGE").unwrap_or(0),
            armor: int(l, "ARMOR").unwrap_or(0),
            proj_speed,
            uber_size: int(l, "UBER_SIZE").unwrap_or(1),
            // `UnitType::init@0061ab50:723`-`730`: `squad_size` is the
            // literal 1 and these two are `get_text_num(..., -1)`, so a
            // record with no `CREW_SIZE` would leave a unit with no
            // figures at all. Every one of `unitrules.xml`'s 364 records
            // carries the column.
            crew_size: int(l, "CREW_SIZE").unwrap_or(-1),
            target_size: int(l, "TARGET_SIZE").unwrap_or(0) * UNIT_BLOCK_RADIUS,
            // The original's default is `-1`, which would index
            // `circle_radius` out of its own array; every one of
            // `unitrules.xml`'s records carries the column, and a fixture
            // that does not gets the single centre point instead.
            circle_radius: int(l, "CIRCLE_RADIUS").unwrap_or(0).clamp(0, 10),
            block_radius: int(l, "BLOCK_RADIUS").unwrap_or(0) * UNIT_BLOCK_RADIUS,
            push_circles: int(l, "PUSH_CIRCLES").unwrap_or(-1).clamp(1, 100),
            push_size: if int(l, "PUSH_CIRCLES").unwrap_or(-1).clamp(1, 100) == 1 {
                int(l, "BLOCK_RADIUS").unwrap_or(-1)
            } else {
                int(l, "PUSH_SIZE").unwrap_or(-1).clamp(1, 100)
            } * UNIT_BLOCK_RADIUS,
            x_spacing: int(l, "X_SPACING").unwrap_or(0) * UNIT_FORMATION_SPACING,
            y_spacing: int(l, "Y_SPACING").unwrap_or(0) * UNIT_FORMATION_SPACING,
            guy_spacing: int(l, "GUY_SPACING").unwrap_or(0) * UNIT_GUY_SPACING,
            domain: domain_of(l.text("DOMAIN")),
            fly_high: int(l, "FLY_HIGH").unwrap_or(-1),
            fly_low: int(l, "FLY_LOW").unwrap_or(-1),
            siege: flags & 0x20000 != 0,
            // Pass 1, and therefore the record's own.
            from: key(r.text("FROM"), unit_names, "unit_key FROM", warnings),
            jump: key(r.text("JUMP"), unit_names, "unit_key JUMP", warnings),
            graft: key(r.text("GRAFT"), unit_names, "unit_key GRAFT", warnings),
            where_: key(r.text("WHERE"), build_names, "build_key WHERE", warnings),
            // `Categories::find_key(unit_cats, …)`: the `key` attribute of
            // `rules.xml`'s `<CATEGORIES id="unit_cats">`, −1 when unmatched.
            cat: find_name(unit_cats, l.text("CAT").unwrap_or("")).map_or(-1, |i| i as i32),
            carry: int(l, "CARRY").unwrap_or(0),
            research_premium_cost,
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
    /// `LOS`/`SCIENCE_LOS` — a building's own fog radius (`docs/VISION.md`
    /// §2.1).
    los: i32,
    science_los: i32,
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
    /// `FLY_HIGH` and `FLY_LOW`, `BuildType::init@00632340:319`-`326`.
    fly_high: i32,
    fly_low: i32,
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
            los: int(r, "LOS").unwrap_or(0),
            science_los: int(r, "SCIENCE_LOS").unwrap_or(0),
            to_hit: int(r, "TO_HIT").unwrap_or(-1),
            // **Signed, as `BuildType::init@00632340` keeps it** (item
            // 1112, `docs/COMBAT.md` §84.4): it stores `ATTENUATE` at
            // `+0x1f0` as read, where `UnitType::init@0061ab50` stores its
            // absolute value, and `Ammo::init`'s `to_hit + (−dist / 192) ×
            // +0x1f0` then *raises* a building's accuracy with distance.
            // run404's Radar round on 797 prints `accuracy 310`: 300 and
            // `−5 × −2`.
            attenuate: int(r, "ATTENUATE").unwrap_or(0),
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
            fly_high: int(r, "FLY_HIGH").unwrap_or(-1),
            fly_low: int(r, "FLY_LOW").unwrap_or(-1),
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
        // The patriots' `1` is bit 26, not 27 — the original's digit branch
        // shifts by 25 — and `0` is not a digit to it at all: it falls into
        // the letter branch and lands on bit 15.
        assert_eq!(unit_flags("1"), 1 << 26);
        assert_eq!(unit_flags("lmhcbp1"), 0x0400_9886);
        assert_eq!(unit_flags("0"), 1 << 15);
        assert_eq!(unit_flags(""), 0);
    }

    #[test]
    fn a_name_group_takes_the_first_record_s_columns() {
        let names: Vec<String> = ["Citizen", "Citizen", "Scholar", "General", "General", "Spy"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(name_group_leaders(&names), vec![0, 0, 2, 3, 3, 5]);
        // A repeat that is not adjacent starts its own group: the original
        // compares against the *running* leader, not a table of names.
        let names: Vec<String> = ["A", "B", "A"].iter().map(|s| s.to_string()).collect();
        assert_eq!(name_group_leaders(&names), vec![0, 1, 2]);
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
            good_types: vec![],
            unit_type_names: vec![],
            build_type_names: vec![],
            unit_tree: vec![6, 7, 8],
            build_tree: vec![9, 10],
            tech_tree: vec![11, 12],
            good_tree: (0..6).collect(),
            warnings: vec![],
            map_styles: vec![],
            scripts: vec![],
            gaia_lengths: Default::default(),
            piece_lengths: Default::default(),
            piece_tracks: Default::default(),
            piece_releases: Default::default(),
            pivot_restrictions: Default::default(),
            mountain_templates: Vec::new(),
            spells: vec![],
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
    use crate::testenv::install;

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

    /// **`BuildTypeData::to` skips a hero-masked record** (item 1326,
    /// `docs/CITIES.md` §13 item 14): the Forbidden City (record 117, `FROM`
    /// the Small City) and the Red Fort (120, `FROM` the Fort) carry
    /// `obj_masks` `0x4000000`, which `BuildType::init` tests before its
    /// store (`00632da8`–`00632dbe`). So the Small City's successor is the
    /// Large City and the Fort's the Castle, and `get_buildings(VILLAGE)`
    /// counts a Large City.
    #[test]
    fn a_hero_masked_wonder_is_nobody_s_to() {
        let Some(i) = install() else { return };
        let l = load(&i).unwrap();
        let rec = |ti: i32| (ti - 0x19e) as usize;
        assert_eq!(
            l.build_types[rec(0x19e)].to,
            Some(rec(0x19f)),
            "VILLAGE → TOWN"
        );
        assert_eq!(
            l.build_types[rec(0x19f)].to,
            Some(rec(0x1a0)),
            "TOWN → METROPOLIS"
        );
        assert_eq!(
            l.build_types[rec(0x1bb)].to,
            Some(rec(0x1bc)),
            "FORTX → CASTLE"
        );
        assert_eq!(l.build_types[rec(0x213)].to, None, "FORBIDDENCITY");
        assert_eq!(l.build_types[rec(0x216)].to, None, "REDFORT");
        for (r, b) in l.build_types.iter().enumerate() {
            assert!(
                b.to != Some(rec(0x213)) && b.to != Some(rec(0x216)),
                "record {r} links a hero-masked wonder as its successor"
            );
        }
    }

    /// **An anti-air building's cycle and a building's signed `ATTENUATE`**
    /// (item 1112, `docs/COMBAT.md` §84): the Radar Air Defense takes the
    /// `<UNIT>`'s 20 and 10 frames and its one release on frame 2, and the
    /// measured launch; the Lookout and the Observation Post take no cycle;
    /// the Radar's `ATTENUATE` is −5 as read, where a unit's is its absolute
    /// value (`BuildType::init` against `UnitType::init`).
    #[test]
    fn the_radar_air_defense_winds_up_on_its_unit_s_packet() {
        let Some(i) = install() else { return };
        let l = load(&i).unwrap();
        let cycle = |t: i32| {
            let b = l.build_of_type_index(t).unwrap();
            l.tree.types[l.build_tree[b]].wall_cycle.clone()
        };
        assert_eq!(
            cycle(sim::air::RADAR),
            Some(sim::air::WallCycle {
                wind: 20,
                swing: 10,
                releases: vec![(2, 0, false)],
                launch: Some((136, -136, 227)),
            })
        );
        assert!(cycle(sim::air::AIRDEFENSE).is_some_and(|c| c.releases.len() == 1));
        assert!(cycle(sim::air::SAM).is_some_and(|c| c.releases.len() == 3));
        assert_eq!(cycle(sim::air::LOOKOUT), None);
        assert_eq!(cycle(sim::air::OBSERVATIONPOST), None);
        let radar = l.build_of_type_index(sim::air::RADAR).unwrap();
        let p = l.build_types[radar].combat.as_ref().unwrap();
        assert_eq!((p.to_hit, p.attenuate), (300, -5));
        assert!(l.unit_types.iter().all(|u| u.combat.attenuate >= 0));
    }

    /// One figure's hit, in sixteenths, from the loaded profiles — `get_damage`
    /// then `scale` as `Object::do_damage` runs them (`docs/COMBAT.md` §6–§7),
    /// with the target facing `facing` and the attack arriving along `angle`.
    fn hit_sixteenths(l: &Loaded, a: usize, t: usize, facing: i32, angle: i32) -> i32 {
        use sim::combat::{Modifiers, Side, TypeRef, scale};
        use sim::movement::Angle;
        let ap = &l.unit_types[a].combat;
        let tp = &l.unit_types[t].combat;
        let at = Side {
            unit: true,
            attacks: true,
            ..Side::default()
        };
        let tt = Side {
            unit: true,
            attacks: tp.attack != 0,
            facing: Angle(facing),
            ..Side::default()
        };
        let pct = l.table.pct_of(TypeRef::Unit(a), TypeRef::Unit(t));
        let dmg = combat::get_damage(
            &Tuning::RON,
            ap,
            at,
            tp,
            tt,
            ap.attack,
            tp.armor,
            pct,
            Angle(angle),
            false,
            0,
            &Modifiers::default(),
        );
        let s = scale(dmg, 0x100, true, false, ap.ammo_per_att, ap.uber_size);
        s.whole * 16 + s.frac
    }

    /// The combat run's numbers, predicted before the log was read and then
    /// observed hit for hit (run17, 2026-08-24; `docs/COMBAT.md`, "Behavioural
    /// check"): a hoplite figure on a Supply Wagon is 122 sixteenths from
    /// every bearing (the wagon is CIVILIAN, so step 19 never runs); on a
    /// hoplite it is 48 from the front, 85 from the rear (level 1, ×1.5) and
    /// 117 from the side (level 2, ×2.0); a slinger's stone is 32/58/85 on
    /// a hoplite and 53 on a scout or a wagon. `hits.py` on
    /// `gamelog-run17-combat.txt` shows exactly these values and no others
    /// among the unit-on-unit rises (a frame with two strikes shows their sum).
    #[test]
    fn run17_s_hits_are_the_formula_s() {
        let Some(i) = install() else { return };
        let l = load(&i).unwrap();
        let hop = l.unit_named("Hoplites").unwrap();
        let wag = l.unit_named("Supply Wagon").unwrap();
        let sl = l.unit_named("Slingers").unwrap();
        let sc = l.unit_named("Scout").unwrap();
        const FACING: i32 = 0x5555_5555; // a cheat-placed unit's 120°
        let rear = FACING;
        let side = FACING.wrapping_add(0x4000_0000);
        let front = FACING.wrapping_add(i32::MIN);
        let hit = |a, t, angle| hit_sixteenths(&l, a, t, FACING, angle);
        assert_eq!(
            (
                hit(hop, wag, rear),
                hit(hop, wag, side),
                hit(hop, wag, front)
            ),
            (122, 122, 122)
        );
        assert_eq!(
            (
                hit(hop, hop, front),
                hit(hop, hop, rear),
                hit(hop, hop, side)
            ),
            (48, 85, 117)
        );
        assert_eq!(
            (hit(sl, hop, front), hit(sl, hop, rear), hit(sl, hop, side)),
            (32, 58, 85)
        );
        assert_eq!((hit(sl, sc, front), hit(sl, wag, rear)), (53, 53));
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

    /// `masks.txt`'s shape: `#name`, then `x, y`, then the grid — and the
    /// second grid some sections carry after a blank line is **not** read,
    /// because `init_build_mask` takes `x × y` items and stops.
    #[test]
    fn masks_txt_reads_the_first_grid_of_each_section() {
        let text = "\
;
; Upper left is \"North\", Upper right is \"East\"
;

#2x2 solid
2, 2
1, 1
1, 1

#4x4 gather
4, 4
0, 0, 0, 0
0, 0, 0, 0
0, 0, 0, 0
0, 0, 0, 0

0, 0, 0, 0
0, 1, 1, 0
0, 1, 1, 0
0, 0, 0, 0

#3x3 extra space
3, 3
1, 1, 0
1, 1, 0
0, 0, 0
";
        let m = parse_masks_txt(text);
        assert_eq!(m.len(), 3, "three sections, and the comment is not one");
        assert_eq!(m[0].0, "2x2 solid");
        assert_eq!(m[0].1.cells, vec![1, 1, 1, 1]);
        assert_eq!(m[1].0, "4x4 gather");
        assert_eq!(m[1].1.cells, vec![0; 16], "the second grid is not read");
        assert_eq!(m[2].0, "3x3 extra space");
        assert_eq!(m[2].1.cells, vec![1, 1, 0, 1, 1, 0, 0, 0, 0]);
        assert_eq!((m[2].1.x, m[2].1.y), (3, 3));
    }

    /// The graphics table names the mask, and the rules row names the
    /// graphic: `<BUILD name="GRAPH-TRIBE-AGEn" mask="…">`, with the Farm
    /// written as its own tag instead.
    #[test]
    fn graphic_masks_are_keyed_by_the_graph() {
        let text = r#"<?xml version="1.0"?>
<ROOT><BUILDINGS>
  <FARM texture="x"><DEFAULT><AGE0 mask="4x4 gather S"/></DEFAULT></FARM>
  <BUILD name="WOODCUTTER-DEFAULT-AGE0" mask="2x2 gather"/>
  <BUILD name="WOODCUTTER-DEFAULT-AGE3" mask="2x2 gather"/>
  <BUILD name="MINE-DEFAULT-AGE0" mask="2x2 solid"/>
</BUILDINGS></ROOT>"#;
        let doc = roxmltree::Document::parse(text).unwrap();
        let g = parse_graphic_masks(&doc);
        assert_eq!(
            g,
            vec![
                ("FARM".to_string(), "4x4 gather S".to_string()),
                ("WOODCUTTER".to_string(), "2x2 gather".to_string()),
                ("MINE".to_string(), "2x2 solid".to_string()),
            ]
        );
    }

    /// Against the shipped files: every building type gets a template of its
    /// own size, and the three that matter are what `masks.txt` says.
    ///
    /// The Woodcutter's Camp blocking **nothing** is the whole point — a
    /// citizen returning to it stands on its own footprint, which is where
    /// `find_nearby_spot` puts it and where this crate refused to
    /// (`docs/ORDERS.md` §10, item 44).
    #[test]
    fn every_building_type_takes_its_graphic_s_blocking_template() {
        let Some(i) = install() else { return };
        let l = load(&i).unwrap();
        let missing: Vec<&str> = l
            .build_types
            .iter()
            .enumerate()
            .filter(|(_, b)| b.block_mask.len() != (b.x_size * b.y_size).max(0) as usize)
            .map(|(n, _)| l.build_names[n].as_str())
            .collect();
        assert!(missing.is_empty(), "no blocking template: {missing:?}");
        let of = |name: &str| {
            l.build_types[l.build_named(name).unwrap()]
                .block_mask
                .clone()
        };
        assert_eq!(of("Woodcutter's Camp"), vec![0, 0, 0, 0], "2x2 gather");
        assert_eq!(of("Mine"), vec![1, 1, 1, 1], "2x2 solid");
        assert_eq!(of("Farm"), vec![0u8; 16], "4x4 gather S");
        // `7x7 extra space`: the last row and the last column are free, so a
        // city's 7×7 footprint blocks 6×6.
        let city = of("Small City");
        assert_eq!(city.len(), 49);
        assert!(city[..6].iter().all(|&b| b == 1));
        assert_eq!(city[6], 0, "the last column");
        assert!(city[42..].iter().all(|&b| b == 0), "the last row");
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
        // The last FROM writer wins, as in `BuildType::init` — but the
        // Forbidden City, which names the Small City after the Large City
        // does, is hero-masked and writes nothing (item 1326).
        let forbidden = l.build_named("Forbidden City").unwrap();
        assert_eq!(l.build_types[forbidden].from, Some(small));
        assert_eq!(l.build_types[small].to, Some(large));
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

    /// **Every nation's graft table, against the original's own.** run3's
    /// `DUMP_ALL` prints `Tribe::log_data`'s `graft[i]` for all 24
    /// nations, 352 entries each, at the start of a game — after
    /// `UnitType::init`'s passes and `Types::finalize_grafting`, which is
    /// the table every `get_graft` reads. [`tribe_grafts`] must answer all
    /// 8,448 entries. It was identity before item 706, and the British
    /// free archer came out Bowmen where the original trains Longbowmen.
    #[test]
    fn every_nation_s_graft_table_is_run3_s() {
        let Some(i) = install() else { return };
        let Some(path) = crate::testenv::dump("gamelog-run3-fulldump-types.txt") else {
            eprintln!("skipping: no gamelog-run3-fulldump-types.txt (set RON_GAMELOG_DIR)");
            return;
        };
        let text = crate::read(std::path::Path::new(&path)).unwrap();
        // `tribe N`, then the header's six scalars, then `graft[i] v` × 352.
        let mut theirs: Vec<(i64, Vec<usize>)> = Vec::new();
        let mut open: Option<(i64, Vec<usize>)> = None;
        for line in text.lines() {
            let l = line.trim();
            if let Some(v) = l.strip_prefix("tribe ") {
                if let Some(t) = open.take().filter(|t| t.1.len() == 352) {
                    theirs.push(t);
                }
                open = v.parse().ok().map(|n| (n, Vec::new()));
            } else if let Some(v) = l.strip_prefix("graft[i] ")
                && let Some(t) = open.as_mut()
            {
                t.1.push(v.parse().unwrap());
            }
            if theirs.len() == 24 {
                break;
            }
        }
        if let Some(t) = open.filter(|t| t.1.len() == 352) {
            theirs.push(t);
        }
        let theirs: Vec<Vec<usize>> = theirs.into_iter().take(24).map(|t| t.1).collect();
        assert_eq!(theirs.len(), 24, "run3 prints all 24 nations' tables");

        let l = load(&i).unwrap();
        let mut off = Vec::new();
        for (n, table) in theirs.iter().enumerate() {
            for (k, &v) in table.iter().enumerate() {
                let t = l.unit_tree[k];
                let ours = l.tree.tribes[n].graft[t].unwrap_or(t);
                if ours != v {
                    off.push(format!("tribe {n} graft[{t}]: ours {ours} theirs {v}"));
                }
            }
        }
        assert!(
            off.is_empty(),
            "{} of 8448 entries part:\n{}",
            off.len(),
            off.join("\n")
        );
        // The entry item 706 turned on: the British (11) train Longbowmen
        // (unit record 127) when asked for Archers (121).
        assert_eq!(theirs[11][121], l.unit_tree[127]);
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
        let democracy = l.tech_tree[l.tech_named("Democracy").unwrap()];
        assert_eq!(
            t.roles.democracy_preqs,
            [Some([Preq::Of(democracy), Preq::None, Preq::None]); 2]
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

    /// **`Leader::gain_tech` step 13's five unit blocks, from the install**
    /// (`docs/TECH.md` §13). The one that matters is the British: on
    /// gaining the Classical Age a British player is handed **Archers**
    /// free, and that is what converts run53's three Bowmen on frame 6736.
    #[test]
    fn the_nation_free_upgrade_blocks_name_the_units_they_hand_out() {
        let Some(i) = install() else { return };
        let l = load(&i).unwrap();
        let t = &l.tree;
        assert_eq!(t.free_rules.len(), 5, "the five predicate blocks");
        let by_gate = |power: usize| {
            t.free_rules
                .iter()
                .find(|r| r.gate == sim::tech::Gate::Power(power))
                .expect("a block for this nation")
        };
        let named = |n: &str| l.unit_tree[l.unit_named(n).unwrap()];
        // British (11), `BRITISH_ARCHER_UPGRADES` — Barracks units of the
        // Bowmen line. The block hands out the ones whose prerequisites the
        // gain completes, so the list is the whole lineage.
        let british = by_gate(11);
        assert_ne!(british.enabled, 0, "the constant ships on");
        assert!(british.candidates.contains(&named("Archers")));
        assert!(british.candidates.contains(&named("Crossbowmen")));
        assert!(
            !british.candidates.contains(&named("Hoplites")),
            "another Barracks line is not in it"
        );
        assert!(
            !british.candidates.contains(&named("Scout")),
            "and neither is a unit trained elsewhere"
        );
        // Spanish (9) and Turks (8) have no `where` test at all.
        assert!(by_gate(9).candidates.contains(&named("Scout")));
        assert!(by_gate(8).candidates.contains(&named("Catapult")));
        // Germans (12): two blocks, and the light-cavalry one is **not**
        // empty — `docs/TECH.md` §13's row said its candidates do not
        // exist, and `0xd1` is Light Horse.
        let german: Vec<_> = t
            .free_rules
            .iter()
            .filter(|r| r.gate == sim::tech::Gate::Power(12))
            .collect();
        assert_eq!(german.len(), 2);
        // **And the light-cavalry block really is empty**, which is what
        // `docs/TECH.md` §13's row said and what the `where` test decides:
        // `0xd1` is Light Horse, but a Light Horse is trained at the
        // **Stable**, and the block asks for Barracks units. The heavy
        // infantry one is not empty.
        assert!(german.iter().any(|r| r.candidates.is_empty()));
        assert!(
            german
                .iter()
                .any(|r| r.candidates.contains(&named("Hoplites")))
        );
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
