//! **The start track** (`docs/START.md`, item 4033): frame zero built from
//! a lobby's choices alone, beside the installer that takes it from the
//! dump ([`build_sim`]), and the two compared record by record.
//!
//! [`from_lobby`] walks `Game::run@00584590`'s setup in the original's
//! order through [`sim::Sim::setup`] for as far as this crate carries the
//! walk, and says where it stops (`docs/START.md` §2). [`start_block`]
//! reads either `Sim` as the start block's records — the leaders, the
//! world, the cities and buildings, the units, gaia, the territory and the
//! sync word — in that walk's order, and [`compare_start`] lists every field the
//! two sides disagree on. The first row is the next item's booking.

use super::*;

/// **The start track's counter** (`docs/START.md` §5): the lobbies whose
/// whole start block [`from_lobby`] builds equal to the installed one, of
/// the lobbies on disk with a `[Start Game]` dump. Pinned red first at 0
/// (item 4033): the harness's own lobby parted at the map (§4). 1 at item
/// 4063: the harness lobby (run9: seed 12345, Great Lakes, Small, two
/// players) agrees on every one of its 18,264 fields. 35 at item 4034
/// (+34), the first count over every lobby
/// (`every_lobby_s_start_built_from_its_choices`): 14 Great Lakes lobbies
/// already whole and never compared, 16 a starting pasture's twenty draws
/// released (`CitiesState::walks_setup`), and 4 whose only parting was a
/// personality no dump prints. 114 at item 4186 (+79): the chain composed
/// for East Indies (42 of its 53 lobbies) and Great Sahara (37 of 47), the
/// same calls Great Lakes' runs ([`crate::mapgen::lakes::composed_map`]);
/// then 143 (+29): `build_units`' and `small_city_buildings`' nation arms,
/// a Spanish, Dutch, Korean, Greek or American seat on the three maps.
/// 146 at item 4196 (+3): Southwest Mesa composed, its three lobbies (15,
/// 114, 115) whole, `Cliffs::init` and all; then 153 (+7): the chains
/// `mapgen`'s walk carries on their own takes composed — Old World,
/// Amazon Basin, Himalayas, African Watering Hole, Mediterranean,
/// Australian Outback and Colonial Powers, one lobby each; then 157 (+4):
/// Nile Delta, Sea Power and New World once the style's territory limits
/// are read (`crate::mapgen::territory_limits`), East Meets West once its
/// permutation is the identity; then 158 (+1): 4033a's thin lobby, its
/// unprinted rows stand-ins and its printed ones whole (4034a). 161 at
/// item 4211 (+3): British Isles' and Warring States' masks ported
/// (`docs/MAPGEN.md` §37) — lobby 24 (run1127) whole on the mask alone,
/// lobbies 19 (run1122) and 119 (run316, a Large Town, its
/// `large_city_buildings` list built) once `CELL flags` is held to the
/// dump's printed word rather than the installer's re-laid roads.
pub const START_BUILT: usize = 161;

/// **The lobby's word on the built start** (`docs/START.md` §7.4, item
/// 4057): `(map, word, length)` — the harness lobby's long capture (run53,
/// Great Lakes) walked from [`from_lobby`]'s `Sim` with no start dump and
/// no correction, every frame's draws against its trace. 24,000 of 24,000,
/// the installed start's own word (`LONG_WORD_GREAT_LAKES`).
pub const START_WALKED: (&str, i64, i64) = ("GreatLakes", 24_000, 24_000);

/// The lobbies on disk with a `[Start Game]` dump: `tools/start_lobbies.py`
/// on 2026-11-09 — 645 captures whose `GAME` carries start records ahead
/// of its first frame, 161 distinct `GAMEINFO` blocks (`same_lobby`'s
/// key). A capture added re-pins it in the landing that files it.
pub const START_LOBBIES: usize = 161;

/// One record field of the start block: the record (`LEADER 1`, `WORLD`,
/// `CELL 12 40`, `BUILD 0/2001`, `UNIT 1/3`, …), the field, its value.
pub type Field = (String, &'static str, String);

/// One parted field: what each side holds, `None` where the record is
/// absent on that side.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parting {
    pub record: String,
    pub field: &'static str,
    pub ours: Option<String>,
    pub theirs: Option<String>,
}

impl std::fmt::Display for Parting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let side = |v: &Option<String>| v.clone().unwrap_or_else(|| "—".into());
        write!(
            f,
            "{} {}: ours {} theirs {}",
            self.record,
            self.field,
            side(&self.ours),
            side(&self.theirs)
        )
    }
}

/// The start block's records read off a `Sim` at frame 0, in the setup
/// walk's order (`docs/START.md` §3): the leaders as `init_rules_and_teams`
/// and `Leader::init` leave them, the world's extent and seed, every cell,
/// the starting cities and buildings by `(who, o)`, the players' units,
/// gaia's, the territory, and the sync word the first frame draws from.
pub fn start_block(sim: &Sim, players: usize) -> Vec<Field> {
    let mut out: Vec<Field> = Vec::new();
    let mut put = |rec: &str, field: &'static str, v: String| out.push((rec.to_string(), field, v));
    for w in 0..players {
        let rec = format!("LEADER {w}");
        put(&rec, "tribe", format!("{:?}", sim.tech[w].power));
        put(&rec, "human", sim.cities.nation[w].human.to_string());
        put(
            &rec,
            "bucket",
            format!("{:?}", sim.economy.ledgers[w].bucket),
        );
        put(&rec, "ages", sim.tech[w].ages.to_string());
        put(
            &rec,
            "techs",
            sim.tech[w].tech.iter().filter(|&&t| t).count().to_string(),
        );
        put(
            &rec,
            "at_war",
            format!("{:?}", &sim.leaders.at_war[w][..players]),
        );
    }
    put("WORLD", "xs", sim.world.width().to_string());
    put("WORLD", "ys", sim.world.height().to_string());
    put("WORLD", "seed", sim.game_seed.to_string());
    for y in 0..sim.world.height() {
        for x in 0..sim.world.width() {
            let c = sim::world::Cell::new(x, y);
            let d = sim.world.cell_data(c);
            let rec = format!("CELL {x} {y}");
            put(&rec, "land", d.land.to_string());
            put(&rec, "flags", d.flags.to_string());
            put(&rec, "val", d.val.to_string());
            put(&rec, "region", sim.world.region_of(c).is_some().to_string());
        }
    }
    for w in 0..players {
        if w < sim.ai.ai.len() && !sim.cities.nation[w].human {
            put(
                &format!("LEADER {w}"),
                "personality",
                format!("{:?}", sim.ai.ai[w].pers),
            );
        }
    }
    let mut builds: Vec<&sim::Building> = sim
        .buildings
        .iter()
        .filter(|b| b.alive && (b.owner as usize) < players)
        .collect();
    builds.sort_by_key(|b| (b.owner, b.index));
    for b in builds {
        let rec = format!("BUILD {}/{}", b.owner, b.index);
        put(&rec, "type", format!("{:?}", b.ty));
        put(&rec, "pos", format!("{} {}", b.pos.x, b.pos.y));
        put(&rec, "active", b.active.to_string());
    }
    let mut units: Vec<&Unit> = sim
        .units
        .iter()
        .filter(|u| u.on_map || u.inside.is_some())
        .collect();
    units.sort_by_key(|u| (u.owner >= players as sim::Player, u.owner, u.index));
    for u in units {
        let rec = format!("UNIT {}/{}", u.owner, u.index);
        put(&rec, "type", format!("{:?}", u.ty));
        put(&rec, "pos", format!("{} {}", u.pos.x, u.pos.y));
        put(&rec, "health", u.health.to_string());
        put(
            &rec,
            "orders",
            format!(
                "{:?}",
                u.orders.iter().map(|o| o.index()).collect::<Vec<_>>()
            ),
        );
    }
    for y in 0..sim.world.height() {
        for x in 0..sim.world.width() {
            let c = sim::world::Cell::new(x, y);
            put(
                &format!("CELL {x} {y}"),
                "owner",
                format!("{:?}", sim.world.owner(c)),
            );
        }
    }
    put("GAME", "rng", format!("{:#010x}", sim.rng.seed));
    out
}

/// **Frame zero wider than the block** (item 4057, `docs/START.md` §7.3):
/// the takes the installer writes that [`start_block`] does not read, as
/// records of their own — the fog (`FOG fx fy`, `seen2`), the goods
/// (`GOOD i`: type, position, live bit, in `goods` order), the goody boxes
/// (`ITEM i`: position, live bit, `ever_seen`), the placed mountains
/// (`MOUNTAIN i`: origin cell, solid cells, tiles), each live building's
/// `ever_seen`, the world's territory limits, each region's `flags` (by
/// the original's region number), each tile's mask and each cell's
/// `land_sub`. A walk reads every one of them.
pub fn start_wide(sim: &Sim, players: usize) -> Vec<Field> {
    let mut out: Vec<Field> = Vec::new();
    let mut put = |rec: &str, field: &'static str, v: String| out.push((rec.to_string(), field, v));
    let w = &sim.world;
    for fy in 0..2 * w.height() {
        for fx in 0..w.fog_xs() {
            put(
                &format!("FOG {fx} {fy}"),
                "seen2",
                format!("{:?}", w.seen2(fx, fy)),
            );
        }
    }
    for (i, g) in w.goods().iter().enumerate() {
        let rec = format!("GOOD {i}");
        put(&rec, "type", g.ty.to_string());
        put(&rec, "pos", format!("{} {}", g.pos.x, g.pos.y));
        put(&rec, "alive", g.alive.to_string());
    }
    for (i, it) in sim.gaia.goody_items.iter().enumerate() {
        let rec = format!("ITEM {i}");
        put(&rec, "pos", format!("{} {}", it.pos.x, it.pos.y));
        put(&rec, "alive", it.alive.to_string());
        put(&rec, "ever_seen", it.ever_seen.to_string());
    }
    for (i, m) in sim.terrain.mountains.iter().enumerate() {
        let rec = format!("MOUNTAIN {i}");
        put(&rec, "loc", format!("{} {}", m.loc.x, m.loc.y));
        let solid: Vec<(i32, i32)> = m.solid.iter().map(|c| (c.x, c.y)).collect();
        put(&rec, "solid", format!("{solid:?}"));
        let tiles: Vec<(i32, i32)> = m.tiles.iter().map(|t| (t.x, t.y)).collect();
        put(&rec, "tiles", format!("{tiles:?}"));
    }
    let mut builds: Vec<&sim::Building> = sim
        .buildings
        .iter()
        .filter(|b| b.alive && (b.owner as usize) < players)
        .collect();
    builds.sort_by_key(|b| (b.owner, b.index));
    for b in builds {
        put(
            &format!("BUILD {}/{}", b.owner, b.index),
            "ever_seen",
            b.ever_seen.to_string(),
        );
    }
    put(
        "WORLD",
        "territory_limits",
        format!("{:?}", w.territory_limits()),
    );
    let mut regions: Vec<(u16, i32)> = (0..w.region_count())
        .map(|r| (w.region_number(r as u16), w.region_flags(r as u16)))
        .collect();
    regions.sort();
    for (n, f) in regions {
        put(&format!("REGION {n}"), "flags", f.to_string());
    }
    let tw = w.width() * sim::world::TILES_PER_CELL;
    for ty in 0..w.height() * sim::world::TILES_PER_CELL {
        for tx in 0..tw {
            put(
                &format!("TILE {tx} {ty}"),
                "mask",
                w.tile_mask(Pos::new(tx, ty)).to_string(),
            );
        }
    }
    for y in 0..w.height() {
        for x in 0..w.width() {
            let d = w.cell_data(sim::world::Cell::new(x, y));
            put(&format!("CELL {x} {y}"), "land_sub", d.land_sub.to_string());
        }
    }
    out
}

/// Every field the two blocks disagree on, in `theirs`' order (the walk's),
/// then the records only `ours` holds.
pub fn compare_start(ours: &[Field], theirs: &[Field]) -> Vec<Parting> {
    use std::collections::HashMap;
    let mine: HashMap<(&str, &str), &str> = ours
        .iter()
        .map(|(r, f, v)| ((r.as_str(), *f), v.as_str()))
        .collect();
    let yours: HashMap<(&str, &str), &str> = theirs
        .iter()
        .map(|(r, f, v)| ((r.as_str(), *f), v.as_str()))
        .collect();
    let mut out = Vec::new();
    for (r, f, v) in theirs {
        let o = mine.get(&(r.as_str(), *f)).copied();
        if o != Some(v.as_str()) {
            out.push(Parting {
                record: r.clone(),
                field: f,
                ours: o.map(str::to_string),
                theirs: Some(v.clone()),
            });
        }
    }
    for (r, f, v) in ours {
        if !yours.contains_key(&(r.as_str(), *f)) {
            out.push(Parting {
                record: r.clone(),
                field: f,
                ours: Some(v.clone()),
                theirs: None,
            });
        }
    }
    out
}

/// What [`from_lobby`] built and where the walk stopped.
pub struct FromLobby {
    pub sim: Sim,
    /// The setup walk's steps (`docs/START.md` §2), each with whether this
    /// crate carried it.
    pub steps: Vec<(&'static str, bool)>,
    pub notes: Vec<String>,
    /// The map chain's checkpoints, in the order the setup trace prints
    /// them (`crate::mapgen::lakes`): what a trace bisects the composed
    /// chain against. Empty when the map was not composed.
    pub checkpoints: Vec<crate::mapgen::lakes::Checkpoint>,
    /// The stream's word at each setup checkpoint the walk reaches past
    /// the map, `(file, line, word)`: what a setup trace checks the draws
    /// of the permutation, `Leader::init` and `build_empire` against.
    pub words: Vec<(&'static str, i64, u32)>,
    /// The stream's word after each step of each `build_empire` this crate
    /// carries, `(seat, step, word)`: the village, then each pre-placed
    /// building in production order. No setup checkpoint falls inside
    /// `build_empire` (1032 to 1148); a draw-site trace of the same lobby
    /// does (`rontrace-run100.log`), and the test holds these against it.
    pub empire_words: Vec<(usize, &'static str, u32)>,
    /// The composed map's region numbers → the sim's region ids
    /// ([`world_from_grid`]'s map): what a walk's record comparison reads a
    /// city's `reg` through, as [`Built::region_map`]. Empty when the map
    /// was not composed.
    pub region_map: Vec<(i64, u16)>,
    /// The composed map's ground, `(tileset, world->seed)`: the tileset
    /// `Map::init_map_data` drew (`LakesMap::tileset`) and the word
    /// `build_game` wrote to `World.seed`, which the client's ground art
    /// reads (item 4215). `None` when the map was not composed.
    pub ground: Option<(String, u32)>,
}

/// **`Setup::small_city_buildings@005aae10` and
/// `Setup::build_civ_specific@005ab760`** (`docs/START.md` §2 step 7;
/// `docs/ORDERS.md` §9.2) for seat `who`, whose village is building `b`
/// of city `c`: each building through `Leader::produce_building`'s frame-0
/// arm, anchored on the village — the Woodcutter's Camp, three Farms
/// (none for the Lakota, `has_tribe_bonus(0x13)`), the Library; then a
/// Market for powers 4 and 0x16, a University for 5 when
/// `GREEK_UNIVERSITY_EARLY > 1`, a Temple for 0x10 when
/// `KOREAN_TEMPLE_UPGRADES > 1`, a Granary for 7 when
/// `EGYPTIAN_GRANARY_EARLY > 1`, a Lumber Mill for 10 when
/// `FRENCH_LUMBERMILL_EARLY > 1`. A Large Town (`starting_town` 3) takes
/// `Setup::large_city_buildings@005aac80`'s list first instead (item 4211).
/// Refused, with the reason: the Iroquois' Senate (its constant is not
/// loaded), and a refused placement. The Americans' farm
/// count is `AMERICANS_STARTING_FARMS` (item 4186).
pub fn pre_placed_buildings(
    sim: &mut Sim,
    loaded: &Loaded,
    who: usize,
    b: usize,
    c: usize,
    starting_town: i64,
    words: &mut Vec<(usize, &'static str, u32)>,
) -> Result<(), String> {
    let bonus = |sim: &Sim, n: usize| {
        sim.tables
            .tech_tree
            .has_tribe_bonus(&sim.tables.setup, &sim.tech[who], n)
    };
    let mut produce = |sim: &mut Sim, t: i32| -> Result<(), String> {
        let rec = loaded
            .build_of_type_index(t)
            .ok_or_else(|| format!("no build type {t:#x}"))?;
        if sim
            .setup()
            .produce_building(who as sim::Player, rec, b, Some(c), false)
        {
            let name = match t {
                0x1a2 => "woodcutter",
                0x1a1 => "farm",
                0x1b3 => "library",
                0x1b7 => "tower",
                _ => "civ-specific",
            };
            words.push((who, name, sim.rng.seed));
            Ok(())
        } else {
            Err(format!("produce_building refused type {t:#x}"))
        }
    };
    match starting_town {
        2 => {
            produce(sim, 0x1a2)?;
            // `small_city_buildings@005aae10`: the Americans take
            // `AMERICANS_STARTING_FARMS` farms (3 as shipped) whatever
            // bonus 0x13 says; the constant at 0 falls to the common arm.
            let farms = sim.tables.tuning.americans_starting_farms;
            if bonus(sim, 0x14) && farms != 0 {
                for _ in 0..farms {
                    produce(sim, 0x1a1)?;
                }
            } else if !bonus(sim, 0x13) {
                for _ in 0..3 {
                    produce(sim, 0x1a1)?;
                }
            }
            produce(sim, 0x1b3)?;
        }
        3 => {
            // `large_city_buildings@005aac80` (item 4211): the Woodcutter's
            // Camp, five Farms unless `has_tribe_bonus(0x13)` (the Lakota),
            // four of `TOWER` (0x1b7), the Library — no Americans' arm.
            produce(sim, 0x1a2)?;
            if !bonus(sim, 0x13) {
                for _ in 0..5 {
                    produce(sim, 0x1a1)?;
                }
            }
            for _ in 0..4 {
                produce(sim, 0x1b7)?;
            }
            produce(sim, 0x1b3)?;
        }
        _ => {}
    }
    let tt = &sim.tables.tech_tree;
    let (greek, korean, egyptian, french) = (
        tt.greek_university_early,
        tt.korean_temple_upgrades,
        tt.egyptian_granary_early,
        tt.french_lumbermill_early,
    );
    if bonus(sim, 4) || bonus(sim, 0x16) {
        produce(sim, 0x1b4)?;
    }
    if bonus(sim, 5) && greek > 1 {
        produce(sim, 0x1a4)?;
    }
    if bonus(sim, 0x10) && korean > 1 {
        produce(sim, 0x1b5)?;
    }
    if bonus(sim, 7) && egyptian > 1 {
        produce(sim, 0x1a7)?;
    }
    if bonus(sim, 10) && french > 1 {
        produce(sim, 0x1a8)?;
    }
    if bonus(sim, 0x12) {
        return Err("the Iroquois' Senate constant is not loaded".into());
    }
    Ok(())
}

/// The leader's own version of a unit `TypeIndex`: `LeaderData::
/// tribe_can_type` or else the tribe's graft (`get_graft`), then
/// `LeaderData::current_upgrade` — what `Setup::build_units` hands
/// `place_unit` and `init_unit` for the scout (`SCOUT`, 0x45) and the
/// citizen (0x32). The unit-type index, or none when the tables lack it.
pub fn leader_unit_type(sim: &Sim, who: usize, type_index: i32) -> Option<usize> {
    let types = &sim.tables.unit_types;
    let tree = types.iter().find(|t| t.type_index == type_index)?.tree?;
    let tt = &sim.tables.tech_tree;
    let p = sim.tech.get(who)?;
    let graft = tt.get_graft(&sim.tables.setup, p, Some(tree))?;
    let cur = tt.current_upgrade(&sim.tables.setup, p, graft);
    types.iter().position(|t| t.tree == Some(cur))
}

/// `SCOUT` and the citizen's `TypeIndex` (`types.list[0x45]`, `[0x32]`).
const SCOUT: i32 = 0x45;
const CITIZEN: i32 = 0x32;
/// `MERCHANTDUTCH`'s `TypeIndex` (62, the export's `enums/TypeIndex.txt`).
const MERCHANT_DUTCH: i32 = 0x3e;

/// **`Setup::build_units@005aafc0(who, slot, city, starting_town)`**
/// (`docs/START.md` §2 step 7, `docs/ORDERS.md` §9.3) for seat `who`
/// whose start cell is `start` and whose village is building `city`:
/// `get_starting_citizens`' `(n, ordered)` and the lobby's and the
/// powers' adjustments; the scout through [`sim::Sim::place_unit`]; then
/// each citizen at its building — the woodcutter for the first
/// `ordered`, then the farm scan, the guard, the woodcutter again while
/// it has room, else `place_unit` idle — born on the building's point by
/// `Objects::init_unit`, stepped off it by `Unit::come_out(u, 0)`
/// ([`sim::Sim::come_out_own_point`]) and given its `GATHER` unless
/// `starting_resources == 8`. The nation arms (item 4186): the
/// Americans' farms add `farms − 3` citizens on a Small Town past 3; past
/// a Nomad start the Spaniards place `SPANISH_EXTRA_SCOUT` more scouts and
/// one more under `reveal_map > 1`, and the Dutch two `MERCHANTDUTCH`, each
/// through `place_unit` beside the first scout; the Koreans add
/// `KOREAN_CITIZENS[0]` citizens. Refused, with the reason: the Greek
/// scholars' `go_inside` (inert while `GREEK_START_SCHOLARS` is 0, as
/// shipped), and `place_unit`'s two unbuilt arms (a Nomad start,
/// `Build::train` after sixty refusals).
pub fn build_units(
    sim: &mut Sim,
    who: usize,
    start: (i32, i32),
    city: usize,
    starting_town: i64,
) -> Result<(), String> {
    let bonus = |sim: &Sim, n: usize| {
        sim.tables
            .tech_tree
            .has_tribe_bonus(&sim.tables.setup, &sim.tech[who], n)
    };
    let (mut n, ordered): (i64, i64) = match starting_town {
        0 => (3, 0),
        1 => (4, 0),
        2 => (5, 2),
        3 => (10, 5),
        _ => (0, 0),
    };
    match sim.ai.lobby.starting_resources {
        7 => n += 8,
        8 => n += 12,
        _ => {}
    }
    let farms = i64::from(sim.tables.tuning.americans_starting_farms);
    if bonus(sim, 0x14) && farms > 3 && starting_town == 2 {
        n += farms - 3;
    }
    if bonus(sim, 0x13) {
        match starting_town {
            2 => n -= 3,
            3 => n -= 5,
            _ => {}
        }
    }
    let at = Pos::new(start.0 * 0x300 + 0x180, start.1 * 0x300 + 0x180);
    let place = |sim: &mut Sim, ty: usize| -> Result<usize, String> {
        sim.setup()
            .place_unit(who as sim::Player, Some(city), ty, at)
            .map_err(|e| format!("place_unit: {e:?} is not built"))
    };
    if starting_town != 0 {
        let scout = leader_unit_type(sim, who, SCOUT).ok_or("no SCOUT unit type")?;
        place(sim, scout)?;
        if bonus(sim, 9) {
            for _ in 0..sim.tables.tuning.spanish_extra_scout {
                place(sim, scout)?;
            }
            if sim.ai.lobby.reveal_map > 1 {
                place(sim, scout)?;
            }
        }
        if bonus(sim, 0x16) {
            let merchant =
                leader_unit_type(sim, who, MERCHANT_DUTCH).ok_or("no MERCHANTDUTCH unit type")?;
            place(sim, merchant)?;
            place(sim, merchant)?;
        }
    }
    if bonus(sim, 5) && sim.tables.tuning.greek_start_scholars != 0 {
        return Err("the Greek scholars' go_inside is not built".into());
    }
    if bonus(sim, 0x10) {
        n += i64::from(sim.tables.tuning.korean_citizens[0]);
    }
    let citizen = leader_unit_type(sim, who, CITIZEN).ok_or("no citizen unit type")?;
    let owner = who as sim::Player;
    let slot = |sim: &Sim, o: i64| {
        sim.buildings
            .iter()
            .position(|b| b.alive && b.owner == owner && i64::from(b.index) == o)
    };
    // `Objects.build_mark[who]`: one past the last building `find_free`
    // numbered for this player.
    let build_mark = sim
        .buildings
        .iter()
        .filter(|b| b.owner == owner && b.index >= sim::BUILD_BASE)
        .map(|b| i64::from(b.index) + 1)
        .max()
        .unwrap_or(i64::from(sim::BUILD_BASE));
    let workable = |sim: &Sim, b: usize| {
        sim.buildings[b].active
            && sim.is_gather_type(b)
            && sim.building_ident(b) != sim::cities::build::Ident::University
    };
    let mut cursor = 0i64;
    for i in 0..n {
        let target = if starting_town < 2 {
            None
        } else if i < ordered {
            // Step 1: `2001`, gated only on the mark and its alive bit.
            (2001 < build_mark).then(|| slot(sim, 2001)).flatten()
        } else {
            // Step 2: the farm scan from `2002 + cursor`; step 3 the
            // stopping object behind the guard; step 4 `2001` behind the
            // guard and its room.
            let mut t = 2002 + cursor;
            let mut stop = None;
            while t < build_mark {
                let Some(h) = slot(sim, t) else { break };
                stop = Some(h);
                if sim.building_ident(h) == sim::cities::build::Ident::Farm {
                    break;
                }
                t += 1;
            }
            cursor = t - 2002 + 1;
            stop.filter(|&h| t < build_mark && workable(sim, h))
                .or_else(|| {
                    slot(sim, 2001).filter(|&h| {
                        workable(sim, h)
                            && sim.buildings[h]
                                .gather_max
                                .is_some_and(|m| sim.num_gatherers(h, false, false) < m)
                    })
                })
        };
        match target {
            Some(b) => {
                let pos = sim.buildings[b].pos;
                let u = sim.setup().init_unit(owner, citizen, pos);
                sim.setup().come_out_own_point(u);
                if sim.ai.lobby.starting_resources != 8 {
                    sim.command()
                        .add_gather_order(u, b, sim::orders::QueuePos::New, false);
                }
            }
            None => {
                place(sim, citizen)?;
            }
        }
    }
    Ok(())
}

/// The herd each herd good founds: `Objects::init_good@00653f30`'s
/// `Herds::init_herd(cell, HERD<type>, slot)` for a good whose type is one
/// of the seven herd goods (`GoodTypeData::is_herd_type`, then the switch
/// on the type's own index): FISH 6 → `HERDFISH` 0x19b, 0xe → `HERDHORSE`
/// 0x197, 0x15 → `HERDBEAR` 0x19a, 0x17 → `HERDBISON` 0x199, 0x19 →
/// `HERDPEACOCK` 0x19d, WHALES 0x1f → `HERDWHALE` 0x19c, 0x21 →
/// `HERDSHEEP` 0x198.
pub const HERD_OF_GOOD: [(i32, i32); 7] = [
    (6, 0x19b),
    (0xe, 0x197),
    (0x15, 0x19a),
    (0x17, 0x199),
    (0x19, 0x19d),
    (0x1f, 0x19c),
    (0x21, 0x198),
];

/// **The map's herds** (`docs/START.md` §2 step 8): one per live herd good
/// of the composed map, in the goods list's slot order, each numbered by
/// its place (`Herds::init_herd` takes the first dead herd, and none dies
/// in map generation here). SEAM: the original numbers herds in the order
/// `init_good` ran, which is the slot order unless a slot was reused —
/// the test holds the list against the start dump's `HERDS`.
pub fn herds_from_goods(
    goods: &[Option<(i32, i32, i32)>],
) -> Vec<sim::gaia::herd_create_view::HerdRec> {
    goods
        .iter()
        .flatten()
        .filter_map(|&(ty, x, y)| {
            HERD_OF_GOOD
                .iter()
                .find(|(g, _)| *g == ty)
                .map(|&(_, kind)| (kind, x, y))
        })
        .enumerate()
        .map(|(i, (kind, cx, cy))| sim::gaia::herd_create_view::HerdRec {
            cx,
            cy,
            kind,
            tag: i as i16,
        })
        .collect()
}

/// **`Setup::build_game`'s start permutation** (`docs/START.md` §2 step
/// 5): `Game::start_list[k]` for the eight seats, each a fresh index — a
/// `Random::get(0, 0xffff) & 7`, re-drawn until it is one no earlier `k`
/// took, when the map randomises its starts (`Map::Map@006a06e0` sets
/// `randomize_starts`, and of the styles only East Meets West's and
/// Warring States' `make_continents` write it again); `k` itself when it
/// does not.
pub fn start_permutation(rng: &mut sim::combat::Rng, randomize: bool) -> [usize; 8] {
    let mut list = [0usize; 8];
    let mut used = [false; 8];
    let mut k = 0;
    while k < 8 {
        let i = if randomize {
            (rng.get(0, 0xffff) & 7) as usize
        } else {
            k
        };
        if !used[i] {
            used[i] = true;
            list[k] = i;
            k += 1;
        }
    }
    list
}

/// **The map's grid as the sim's `World`** (`docs/START.md` §2 step 4):
/// what [`super::world_from`] reads off a `WORLD` dump, read off the
/// composed chain's [`sim::mapgen::MapGrid`] instead — each distinct region
/// number a sim region (land below 0x40, sea from it) carrying the
/// original's number, `region2`, the cell bytes, the owners, the tile
/// masks and the corner heights. Returns the world and the region map.
pub fn world_from_grid(
    grid: &sim::mapgen::MapGrid,
    z: Vec<sim::num::single::Single>,
    sea_map: i32,
) -> (World, Vec<(i64, u16)>) {
    let mut world = World::new(grid.xs, grid.ys);
    let mut region_map: Vec<(i64, u16)> = Vec::new();
    let mut region_of = |n: i64, world: &mut World| -> Option<u16> {
        if n < 0 {
            return None;
        }
        if let Some((_, r)) = region_map.iter().find(|(d, _)| *d == n) {
            return Some(*r);
        }
        let terrain = if n < 0x40 {
            sim::world::Terrain::Land
        } else {
            sim::world::Terrain::Sea
        };
        let r = world.add_region(terrain);
        world.set_region_number(r, n as u16);
        region_map.push((n, r));
        Some(r)
    };
    let owner = |w: i64| -> sim::world::Owner {
        match w {
            -2 => sim::world::Owner::Ambiguous,
            w if (0..=255).contains(&w) => sim::world::Owner::Player(w as sim::Player),
            _ => sim::world::Owner::None,
        }
    };
    for (i, c) in grid.cells.iter().enumerate() {
        let cell = sim::world::Cell::new(i as i32 % grid.xs, i as i32 / grid.xs);
        if let Some(r) = region_of(i64::from(c.region), &mut world) {
            world.set_region(cell, r);
        }
        let region2 = region_of(i64::from(c.region2), &mut world);
        world.set_owner(cell, owner(i64::from(c.who)), owner(i64::from(c.who2)));
        world.set_cell_data(
            cell,
            sim::world::CellData {
                flags: c.flags,
                land: c.land,
                land_sub: c.land_sub,
                region2,
                val: c.val,
                goods: c.goods,
                blocked: c.blocked,
                solid: c.solid,
                bad: c.bad,
                down: c.down,
                down_who: c.down_who as i8,
            },
        );
    }
    let tw = (grid.xs * sim::world::TILES_PER_CELL) as usize;
    for (i, m) in grid.tiles.iter().enumerate() {
        world.set_tile_mask(Pos::new((i % tw) as i32, (i / tw) as i32), *m);
    }
    world.set_corner_singles(z);
    world.set_sea_map(sea_map);
    (world, region_map)
}

/// **The map's objects** (`docs/START.md` §7.2), from the composed grid
/// rather than a dump's `GOOD`, `ITEM` and `MOUNTAINS` blocks: each live
/// slot of the goods list as `Objects::init_good` made it, at its cell's
/// centre, in slot order (the order `down_who` indexes); each goody box
/// the cells mark (`Item::init`: `down` −3, the box's index in
/// `down_who`), in that index's order; and the generator's placed
/// mountains laid from their templates, as [`build_sim`] lays a dump's.
/// A dead goods slot is refused by name: the grid keeps no type or cell
/// for it.
pub fn lay_map_objects(
    sim: &mut Sim,
    loaded: &Loaded,
    goods: &[Option<(i32, i32, i32)>],
    mountains: &[(i32, i32, usize)],
) -> Vec<String> {
    let mut notes = Vec::new();
    let centre = |x: i32, y: i32| Pos::new(x * 0x300 + 0x180, y * 0x300 + 0x180);
    let footprints = loaded
        .install_root
        .as_deref()
        .and_then(crate::mapgen::good_footprints)
        .unwrap_or_default();
    for (slot, g) in goods.iter().enumerate() {
        match *g {
            Some((ty, x, y)) => {
                // `GoodTypeData::snap_center@0066fcb0`: the cell centre for
                // an even `X_SIZE`, half a tile up-left of it for an odd;
                // oil is made without it (`World::set_oil_at`), at the
                // centre.
                let off = |size: i32| if size & 1 != 0 { 0x120 } else { 0x180 };
                let pos = match footprints.get(ty as usize) {
                    Some(fp) if ty as usize != sim::world::OIL_GOOD => {
                        Pos::new(x * 0x300 + off(fp.x_size), y * 0x300 + off(fp.y_size))
                    }
                    _ => centre(x, y),
                };
                sim.world.add_good(sim::world::Good {
                    pos,
                    ty: ty as usize,
                    alive: true,
                });
            }
            None => notes.push(format!("goods: slot {slot} is dead and not laid")),
        }
    }
    let mut boxes: Vec<(i8, i32, i32)> = Vec::new();
    for y in 0..sim.world.height() {
        for x in 0..sim.world.width() {
            let d = sim.world.cell_data(sim::world::Cell::new(x, y));
            if d.down == -3 {
                boxes.push((d.down_who, x, y));
            }
        }
    }
    boxes.sort();
    for (_, x, y) in boxes {
        sim.setup().add_goody_item(centre(x, y), true, 0);
    }
    if !mountains.is_empty() && !loaded.mountain_templates.is_empty() {
        let placed: Vec<(sim::world::Cell, usize)> = mountains
            .iter()
            .map(|&(x, y, t)| (sim::world::Cell::new(x, y), t))
            .collect();
        sim.setup()
            .place_mountains(&placed, &loaded.mountain_templates);
    }
    notes
}

/// **`Game::init@0058c480`'s extent**: `World::init(n, n)` with `n` the
/// `mapsizes` row `MAP_SIZE` indexes (`rules.xml`, `DATA`, read at `+0x3c`
/// of the 0x58-byte record), and on map style 16 `n × 11 / 10` rounded up
/// to even and capped at 100. The half-size arm (`semaphore[1] & 2`) is
/// not a lobby choice and is not taken.
pub fn world_extent(root: &std::path::Path, map_size: i32, map_style: i32) -> Option<i32> {
    let text = std::fs::read_to_string(root.join("Data/rules.xml")).ok()?;
    let doc = roxmltree::Document::parse(&text).ok()?;
    let sizes = doc
        .descendants()
        .find(|n| n.has_tag_name("CATEGORIES") && n.attribute("id") == Some("mapsizes"))?;
    let rec = sizes
        .children()
        .filter(|n| n.has_tag_name("CATEGORY"))
        .nth(usize::try_from(map_size).ok()?)?;
    let mut n: i32 = rec
        .children()
        .find(|n| n.has_tag_name("DATA"))?
        .text()?
        .trim()
        .parse()
        .ok()?;
    if map_style == 16 {
        n = n * 11 / 10;
        if n & 1 != 0 {
            n += 1;
        }
        n = n.min(100);
    }
    Some(n)
}

/// **Frame zero from the lobby's choices alone**: the `GAMEINFO` block's
/// fields and its `PLAYER` rows, and nothing else of the dump. Walks
/// `docs/START.md` §2 in order through [`sim::Sim::setup`]:
///
/// 1. the extent ([`world_extent`]);
/// 2. `init_rules_and_teams`: the lobby, the nations (`PLAYER tribe`), the
///    human bit (`PLAYER flags & 4`), the opening techs and the goods
///    (`Sim::new`'s `starting_goods`; a lobby row with a spread draws, and
///    is refused); the random-nation draw (`rand % 24`) is not walked —
///    `build_game`'s re-seed discards it, so nothing at frame 0 observes it;
/// 3. `Setup::build_game`'s re-seed from `info.seed` and `world->seed =
///    rand % 0xffff + 1`;
/// 4. **the map** — `Map::make`, `Terrain::init` — composed for Great
///    Lakes, East Indies, Great Sahara and Southwest Mesa
///    ([`crate::mapgen::lakes::composed_map`], items 4047, 4186, 4196); on any
///    other style the walk stops here, and every step after it (the start
///    permutation, `Leader::init`'s personality, each `build_empire`,
///    `Herd::create_units`, `compute_all_territory`) is listed as not
///    built.
pub fn from_lobby(
    loaded: &Loaded,
    game_info: &[(&str, &str)],
    player_rows: &[Vec<(&str, &str)>],
    tuning: Tuning,
) -> FromLobby {
    let mut notes = Vec::new();
    let mut steps: Vec<(&'static str, bool)> = Vec::new();
    let int = |key: &str| -> Option<i64> {
        game_info
            .iter()
            .find(|(k, _)| *k == key)
            .and_then(|(_, v)| v.trim().parse().ok())
    };
    let row_int = |p: &Vec<(&str, &str)>, key: &str| -> Option<i64> {
        p.iter()
            .find(|(k, _)| *k == key)
            .and_then(|(_, v)| v.trim().parse().ok())
    };
    let players = player_rows
        .iter()
        .filter_map(|p| row_int(p, "who"))
        .filter(|w| (0..8).contains(w))
        .map(|w| w as usize + 1)
        .max()
        .unwrap_or(0)
        .max(1);
    // 1. The extent.
    let n = loaded.install_root.as_deref().and_then(|root| {
        world_extent(
            root,
            int("MAP_SIZE").unwrap_or(-1) as i32,
            int("MAP_STYLE").unwrap_or(-1) as i32,
        )
    });
    steps.push(("Game::init's extent", n.is_some()));
    let n = n.unwrap_or_else(|| {
        notes.push("no extent: no install root or no mapsizes row".into());
        1
    });
    // 3 and 4 are computed here, ahead of the `Sim` that stands on their
    // world (`Sim::new` seeds its mesh from the world it is handed); the
    // draws are the walk's, in its order, and nothing in step 2 draws on
    // the stream they spend. 3: `build_game`'s re-seed and `world->seed`.
    let seed = int("(int)seed").unwrap_or(0) as u32;
    let mut rng = sim::combat::Rng::new(seed);
    let game_seed = rng.get(0, 0xffff) % 0xffff + 1;
    // 4: the map, on the stream `world->seed` left (`setup.cpp` 716).
    let mut checkpoints = Vec::new();
    let map = (|| {
        let root = loaded.install_root.as_deref()?;
        let Some(which) = int("MAP_STYLE").and_then(crate::mapgen::lakes::Style::of) else {
            notes.push(format!(
                "the map: style {:?} has no composed chain",
                int("MAP_STYLE")
            ));
            return None;
        };
        let range = loaded
            .unit_of_type_index(349)
            .map(|i| loaded.unit_types[i].combat.max_range)?;
        let lobby = crate::mapgen::lakes::LakesLobby {
            at_716: rng.seed,
            xs: n,
            ys: n,
            nations: player_rows.len() as i32,
            map_size: int("MAP_SIZE").unwrap_or(-1),
            seed: game_seed,
            teams: {
                let mut t = [None; 8];
                for p in player_rows {
                    if let Some(w) = row_int(p, "who").filter(|w| (0..8).contains(w)) {
                        t[w as usize] = Some(row_int(p, "team").unwrap_or(8) as i32);
                    }
                }
                t
            },
            team_style: int("TEAM_STYLE").unwrap_or(0),
        };
        let sea = crate::mapgen::sea_map(root, which.file())?;
        match crate::mapgen::lakes::composed_map(root, which, &lobby, range, |c| {
            checkpoints.push(*c)
        }) {
            Ok(m) => Some((m, sea)),
            Err(e) => {
                notes.push(format!("the map: the chain refused, {e:?}"));
                None
            }
        }
    })();
    let mut starts: Vec<(i32, i32)> = Vec::new();
    let mut goods: Vec<Option<(i32, i32, i32)>> = Vec::new();
    let mut mountains: Vec<(i32, i32, usize)> = Vec::new();
    let mut region_map: Vec<(i64, u16)> = Vec::new();
    let mut cliff_mining: Option<Vec<Vec<(i32, i32)>>> = None;
    let mut ground = None;
    let world = match map {
        Some((m, sea)) => {
            rng = m.draws.rng;
            ground = Some((m.tileset.clone(), game_seed as u32));
            cliff_mining = m.cliffs.map(|c| c.mining);
            goods = m.grid.goods.clone();
            // `World.start_x/start_y` (+0x80, +0x9c): the map maker's
            // start cells, which `build_cities` reads by slot.
            starts = m.grid.starts[0]
                .list
                .iter()
                .zip(&m.grid.starts[1].list)
                .map(|(&x, &y)| (x, y))
                .collect();
            let sea = sim::mapgen::terrain_init::sea_map_after_make(
                sim::mapgen::terrain_init::WORLD_SEA_MAP_BEFORE,
                sea,
            );
            mountains = m.grid.mountains.clone();
            let (mut w, rm) = world_from_grid(&m.grid, m.z, sea);
            region_map = rm;
            // The parts of `WorldData` the generator and `build_game` write
            // once (`docs/CHECKSUM.md` §21): the start arrays as the chain
            // left them, `Terrain::init`'s half-land lists, the style and
            // `world->seed`.
            w.set_record(sim::world::WorldRecord {
                starts: Some(m.grid.starts.clone()),
                halfland: Some(m.halfland.clone()),
                map: int("MAP_STYLE").map(|v| v as i32),
                seed: Some(game_seed),
                tallies: None,
            });
            // `tilesets.cur_tileset`'s ground numbers, which every
            // building `Wall::start` places reads (item 4136).
            w.set_land_set(Some(m.land));
            // `World::init`'s `seen2`, allocated and zeroed with the
            // world: every reveal the walk makes from here on lands in it
            // (`docs/START.md` §7.2).
            let fog = vec![0u8; (4 * w.width() * w.height()) as usize];
            w.set_fog(fog);
            w
        }
        None => World::new(n, n),
    };
    let map_built = !checkpoints.is_empty() && world.width() == n;
    let mut sim = loaded.sim(tuning, world, players);
    // The install's takes, before any unit is made: `Guy::init` reads the
    // piece table (`Sim::piece_of`) as each figure is born (§7.1).
    install_art(&mut sim, loaded, &mut notes);
    if map_built {
        notes.extend(lay_map_objects(&mut sim, loaded, &goods, &mountains));
        // A Southwest Mesa map's cliffs (item 4196): `Cliffs::init`'s
        // mining lists by queue index, as `build_sim` lays them from the
        // same chain at the setup trace's 7805 seed (item 3943) — here at
        // the stream the lobby's own `world->seed` draw left.
        if let Some(mining) = &cliff_mining {
            sim.setup().place_cliffs(mining);
        }
        // `Map::make`'s copy of the style's six territory limits into
        // `world+0x38..+0x4c`; `Map::load_map_data` seeds them from the
        // rules' `TERRITORY_LIMIT_BASE/CIVIC/CITY` for both the player and
        // the colonized triple, and each `*_TERRITORY_*_HARD_LIMIT` the
        // style XML names overrides its own (item 4196: Nile Delta, Sea
        // Power and New World name theirs; [`crate::mapgen::territory_limits`]).
        let t = &sim.tables.tuning;
        let triple = [
            t.territory_limit_base,
            t.territory_limit_civic,
            t.territory_limit_city,
        ];
        let named = loaded
            .install_root
            .as_deref()
            .zip(int("MAP_STYLE").and_then(crate::mapgen::lakes::Style::of))
            .and_then(|(root, s)| crate::mapgen::territory_limits(root, s.file()))
            .unwrap_or([None; 6]);
        let pick = |k: usize| named[k].unwrap_or(triple[k % 3]);
        sim.world.set_territory_limits(sim::world::TerritoryLimits {
            player: [pick(0), pick(1), pick(2)],
            colonized: [pick(3), pick(4), pick(5)],
        });
    }
    // 2. `init_rules_and_teams`.
    sim.ai.lobby = lobby_of(
        game_info,
        &loaded.map_styles,
        &loaded.wonder_wins,
        &loaded.starting_resources,
        &loaded.pop_wins,
        &loaded.rush_minutes,
        &loaded.cannon_times,
    );
    sim.setup().sync_setup_from_lobby();
    let (lo, hi) = sim.ai.lobby.starting_resources_row;
    let spread = lo != 0 && hi > lo;
    if spread {
        notes.push(format!(
            "init_starting_resources: the row ({lo}, {hi}) has a spread, whose draws are not walked"
        ));
    }
    for p in player_rows {
        let Some(w) = row_int(p, "who").filter(|w| (0..players as i64).contains(w)) else {
            continue;
        };
        let who = w as usize;
        let tribe = row_int(p, "tribe").unwrap_or(-1);
        sim.setup().set_tribe(who as sim::Player, tribe);
        let human = row_int(p, "flags").unwrap_or(0) & 4 != 0;
        sim.cities.nation[who].human = human;
    }
    for who in 0..players {
        sim.setup().start_techs(who as sim::Player);
    }
    // The AI's random nation (`init_tribes`, `rand % 24`, one draw on the
    // harness lobby) is spent on the stream `build_game` re-seeds next, so
    // it is not walked: `GAMEINFO` prints the rolled nation, which is what
    // `set_tribe` took above.
    steps.push(("init_rules_and_teams", !spread));
    // 3. `build_game`'s re-seed and the world seed (computed above).
    sim.game_seed = game_seed;
    sim.rng = rng;
    steps.push(("build_game: re-seed, world->seed", true));
    // 6, ahead of the map it does not read: `Leader::init`'s diplomacy row
    // (`diplos`, `+0x74`), off the lobby's team style, rush rules and the
    // `PLAYER` rows' teams (item 4046).
    let seats: Vec<sim::leaders::relations::Seat> = player_rows
        .iter()
        .filter_map(|p| {
            let w = row_int(p, "who").filter(|w| (0..players as i64).contains(w))?;
            Some(sim::leaders::relations::Seat {
                who: w as sim::Player,
                flags: row_int(p, "flags").unwrap_or(0) as u32,
                team: row_int(p, "team").unwrap_or(8) as i32,
            })
        })
        .collect();
    for who in 0..players {
        sim.setup().init_diplos(who as sim::Player, &seats);
    }
    steps.push(("Leader::init: diplos", true));
    // 4. The map (composed above).
    steps.push(("Map::make and Terrain::init", map_built));
    let mut words = Vec::new();
    let mut empire_words = Vec::new();
    if map_built {
        // 5. The start permutation, off the map's stream (`setup.cpp` 884
        // -> 955). `Map::Map@006a06e0` sets `randomize_starts`, and East
        // Meets West's `make_continents@00696640` writes it again, to 0 at
        // its head (`Map+0x2ec`, item 4196), as Warring States' does at
        // 0x698b95 (item 4211): their seats keep the starts the mask
        // placed them on, and the permutation draws nothing.
        let randomize = !matches!(int("MAP_STYLE"), Some(19 | 15));
        let start_list = start_permutation(&mut sim.rng, randomize);
        words.push(("setup.cpp", 955, sim.rng.seed));
        steps.push(("the start permutation", true));
        // 6. `Leader::init` for each active seat, in the permutation's
        // order: a computer leader's AI tail rolls its personality and
        // picks its script (`random_personality`, `leaders.cpp` 13383 ->
        // 13457); a human's draws nothing.
        if !loaded.scripts.is_empty() {
            let sources: Vec<sim::ai::bhs::Source> = loaded
                .scripts
                .iter()
                .map(|(name, text)| sim::ai::bhs::Source { name, text })
                .collect();
            if let Err(e) = sim
                .setup()
                .load_scripts(&["economic.bhs", "defensive.bhs"], &sources)
            {
                notes.push(format!("the opening scripts did not compile: {e}"));
            }
        }
        for &who in start_list.iter().filter(|&&w| w < players) {
            if sim.cities.nation[who].human {
                continue;
            }
            let flags = player_rows
                .iter()
                .find(|p| row_int(p, "who") == Some(who as i64))
                .and_then(|p| row_int(p, "flags"))
                .unwrap_or(0) as u32;
            sim.setup().init_leader_ai(who as sim::Player, flags);
            words.push(("leaders.cpp", 13457, sim.rng.seed));
        }
        words.push(("setup.cpp", 1032, sim.rng.seed));
        steps.push(("Leader::init (the personality roll)", true));
        // 7. `build_empire(who, slot)` for each active seat in the
        // permutation's order, the k-th empire built taking start cell k
        // (no team style 7 and no teammates on this lobby: every seat is
        // on team 8). `build_cities`: the village at `snap_center` of the
        // start cell's corner, `Build::init` and `Build::activate(0, 0,
        // 0)` — finished and active, its `City::init` wearing the site
        // values down round it — then `compute_all_territory`.
        let village = loaded.build_named("Small City");
        let town = int("STARTING_TOWN").unwrap_or(0);
        let mut buildings_built = true;
        let mut units_built = true;
        // The empires are the setup stream's own: a starting pasture's
        // five are drawn inside `Build::activate` as the original draws
        // them (item 4034), not borrowed as the installer borrows them.
        sim.cities.walks_setup = true;
        for (slot, &who) in start_list.iter().filter(|&&w| w < players).enumerate() {
            let (Some(ty), Some(&(sx, sy))) = (village, starts.get(slot)) else {
                notes.push(format!(
                    "build_cities: no village type or no start cell {slot}"
                ));
                break;
            };
            let pos = sim.snap_center(ty, Pos::new(sx * 0x300, sy * 0x300));
            let b = sim.setup().init_build(who as sim::Player, ty, pos, false);
            sim.setup().activate(b, false, false);
            // `build_cities`' own `cities_built++` (`docs/CHECKSUM.md` §20).
            sim.setup().count_start_city(who as sim::Player);
            // `build_cities`' `Object::update_seen_ally`: the city's
            // territory radius into its owner's `seen2` (§7.2).
            sim.setup().update_seen_ally_build(b);
            sim.setup().sync_territory();
            sim.setup().settle_borders();
            empire_words.push((who, "village", sim.rng.seed));
            match sim.buildings[b].city {
                Some(c) => {
                    if let Err(e) =
                        pre_placed_buildings(&mut sim, loaded, who, b, c, town, &mut empire_words)
                    {
                        notes.push(format!("build_cities, seat {who}: {e}"));
                        buildings_built = false;
                    }
                    // `build_units(who, slot, city, starting_town)`, the
                    // rest of this empire, before the next is begun.
                    match build_units(&mut sim, who, (sx, sy), b, town) {
                        Ok(()) => empire_words.push((who, "units", sim.rng.seed)),
                        Err(e) => {
                            notes.push(format!("build_units, seat {who}: {e}"));
                            units_built = false;
                        }
                    }
                }
                None => {
                    notes.push(format!(
                        "build_cities, seat {who}: the village founded no city"
                    ));
                    buildings_built = false;
                }
            }
        }
        sim.cities.walks_setup = false;
        steps.push(("build_empire: build_cities (the village)", true));
        steps.push((
            "build_empire: small_city_buildings, build_civ_specific",
            buildings_built,
        ));
        steps.push(("build_empire: build_units, place_unit", units_built));
        words.push(("setup.cpp", 1148, sim.rng.seed));
        // 8. `World::analyze_map` (`docs/MAPGEN.md` §36, item 4147): the
        // ten tallies kept on the world's record, the region bits, the sea
        // class; then `compute_all_territory`, whose owners `build_cities`'
        // pass already left as the original's.
        let standard = loaded
            .install_root
            .as_deref()
            .and_then(crate::mapgen::standard_map_size);
        match standard {
            Some(s) => {
                sim.analyze_map(s);
                steps.push(("World::analyze_map", true));
            }
            None => {
                notes.push("World::analyze_map: no Standard map size in rules.xml".into());
                steps.push(("World::analyze_map", false));
            }
        }
        sim.setup().sync_territory();
        sim.setup().settle_borders();
        steps.push(("World::compute_all_territory", true));
        words.push(("setup.cpp", 1169, sim.rng.seed));
        // `Setup::build_game`'s herd loop: each live herd's
        // `Herd::create_units`, in the herd list's order.
        let herds = herds_from_goods(&goods);
        let mut herds_built = true;
        for h in &herds {
            sim.gaia.gaia.herds.push(sim::gaia::Herd {
                cx: h.cx,
                cy: h.cy,
                wx: h.cx,
                wy: h.cy,
                kind: h.kind,
                alive: true,
            });
        }
        for h in &herds {
            let (_, done) = sim.setup().found_herd(h);
            if !done {
                notes.push(format!("Herd::create_units refused herd {}", h.tag));
                herds_built = false;
            }
        }
        steps.push(("Herd::create_units", herds_built));
        words.push(("setup.cpp", 1240, sim.rng.seed));
        words.push(("setup.cpp", 1244, sim.rng.seed));
        // 9. `Game::run@00584590:721`, past `build_game`: the danger map
        // filled once before the first frame (item 4218). It draws nothing.
        sim.setup().calc_danger();
        steps.push(("Game::run: calc_danger", true));
    } else {
        steps.push(("the start permutation", false));
        steps.push(("Leader::init (the personality roll)", false));
        steps.push(("build_empire: build_cities (the village)", false));
        steps.push((
            "build_empire: small_city_buildings, build_civ_specific",
            false,
        ));
        for step in [
            "build_empire: build_units, place_unit",
            "World::analyze_map",
            "World::compute_all_territory",
            "Herd::create_units",
            "Game::run: calc_danger",
        ] {
            steps.push((step, false));
        }
    }
    if !map_built {
        notes.push("the walk stops at the map".into());
    }
    FromLobby {
        sim,
        steps,
        notes,
        checkpoints,
        words,
        empire_words,
        region_map,
        ground,
    }
}

/// **The built start as a walk's `Built`** (item 4057, `docs/START.md`
/// §7): [`from_lobby`]'s `Sim` (which carries the install's takes,
/// [`install_art`], from before its first unit), every unit and building linked by its own `(who, o)`, and **no
/// correction**: no frame's word, no clock, no re-seat, since nothing of a
/// dump is read. What the lobby's own walk runs from in place of
/// [`build_sim`]'s install.
pub fn built_from_lobby(loaded: &Loaded, built: FromLobby) -> (Built, Vec<String>) {
    let FromLobby {
        mut sim,
        notes,
        region_map,
        ..
    } = built;
    sim.trace.trace_phases = true;
    let units = sim
        .units
        .iter()
        .enumerate()
        .map(|(i, u)| UnitLink {
            who: i64::from(u.owner),
            o: i64::from(u.index),
            unit: i,
            kind: u.ty,
        })
        .collect();
    let builds = sim
        .buildings
        .iter()
        .enumerate()
        .filter(|(_, b)| b.alive)
        .map(|(i, b)| (i, i64::from(b.index)))
        .collect();
    let built = Built {
        gaia_reseat_skip: None,
        correction_audit: None,
        units,
        builds,
        region_map,
        notes: Vec::new(),
        frame_seeds: Vec::new(),
        rng_frames: Vec::new(),
        frame_guys: Vec::new(),
        frame_sites: Vec::new(),
        unit_type_names: loaded.unit_type_names.clone(),
        type_index: (0..loaded.good_tree.len()
            + loaded.unit_tree.len()
            + loaded.build_tree.len()
            + loaded.tech_tree.len())
            .map(|id| loaded.type_index(id))
            .collect(),
        unit_tree: loaded.unit_tree.clone(),
        staged: None,
        sim,
    };
    (built, notes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::testkit::*;
    use crate::testenv::{dump, install};

    /// `Game::init`'s extent off the install's `mapsizes`: Small is 60.
    #[test]
    fn the_extent_is_the_mapsizes_row() {
        let Some(inst) = install() else { return };
        let root = inst.root();
        assert_eq!(world_extent(root, 2, 14), Some(60));
        assert_eq!(world_extent(root, 3, 7), Some(70));
        // Style 16: 60 × 11 / 10 = 66, even.
        assert_eq!(world_extent(root, 2, 16), Some(66));
    }

    /// A block compared with itself parts nowhere, and a moved field is
    /// the one row named.
    #[test]
    fn a_block_parts_only_where_a_field_differs() {
        let a: Vec<Field> = vec![
            ("WORLD".into(), "xs", "60".into()),
            ("UNIT 0/0".into(), "pos", "1 2".into()),
        ];
        assert!(compare_start(&a, &a).is_empty());
        let mut b = a.clone();
        b[1].2 = "1 3".into();
        let p = compare_start(&b, &a);
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].to_string(), "UNIT 0/0 pos: ours 1 3 theirs 1 2");
        let p = compare_start(&a[..1], &a);
        assert_eq!(p[0].ours, None);
    }

    /// **The harness lobby's map, composed from its choices, against the
    /// setup trace** (item 4047; `docs/START.md` §2 step 4): [`from_lobby`]
    /// on run11's `GAMEINFO` (the run9 lobby, whose trace run11 is) runs
    /// Great Lakes' chain on one stream from the lobby's seed, and every
    /// checkpoint it hands back — seed, parts 2, 5, 6 and 10–13 — is the
    /// trace's record at that line, from 7805 to `Terrain.cpp` 894. The
    /// phase tests drive each phase from the trace's own 7805 word and the
    /// start dump's `World.seed`; here neither is read.
    #[test]
    fn the_harness_lobby_s_map_is_the_traced_setup_path() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run11-checksum.txt") else {
            eprintln!("skipping: no gamelog-run11-checksum.txt (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let head = crate::mapgen::start_head(std::path::Path::new(&path))
            .unwrap()
            .expect("run11's preamble carries the setup trace");
        let g = crate::mapgen::start_grid(&head);
        let log = Log::parse(&head);
        let init = log.initial().expect("run11's GAMEINFO");
        let ours = from_lobby(&loaded, &init.game_info, &init.players, Tuning::RON);
        let mut parted = Vec::new();
        for c in &ours.checkpoints {
            let Some((_, _, seed, p)) = g.at(c.file, c.line) else {
                parted.push(format!(
                    "{} {}: no such record in the trace",
                    c.file, c.line
                ));
                continue;
            };
            // At 7805 the cells are `World::init`'s, which `make_continents`
            // writes whole (7807's part 5 is the check of that); the port's
            // grid starts wiped, so part 5 is not compared there.
            let cells = if c.line == 7805 { p[5] } else { c.part5 };
            let mine = (c.seed, c.part2, cells, c.part6, c.lists);
            let theirs = (*seed, p[2], p[5], p[6], [p[10], p[11], p[12], p[13]]);
            if mine != theirs {
                parted.push(format!(
                    "{} {}: (seed, parts 2, 5, 6, 10–13) ours {mine:?} theirs {theirs:?}",
                    c.file, c.line
                ));
            }
        }
        let lines: Vec<i64> = ours.checkpoints.iter().map(|c| c.line).collect();
        eprintln!(
            "the composed map: {} checkpoints {lines:?}, {} part; notes {:?}",
            ours.checkpoints.len(),
            parted.len(),
            ours.notes
        );
        assert!(parted.is_empty(), "{}", parted.join("\n"));
        assert_eq!(
            lines,
            [
                7805, 7807, 7854, 7859, 7861, 7865, 7870, 7927, 7937, 7945, 720, 738, 5012, 5014,
                5076, 5116, 5118, 894, 884
            ],
            "the checkpoints the chain prints"
        );
        // Past the map: the permutation's eight draws (955), the AI's
        // `Leader::init` (13457, its first visit) and `build_empire`'s
        // entry (1032), each the trace's word at that line.
        let theirs: Vec<(&str, i64, u32)> = ours
            .words
            .iter()
            .map(|&(f, l, _)| {
                let (_, _, w, _) = g
                    .checksums
                    .iter()
                    .find(|(ff, ll, _, _)| ff == f && *ll == l)
                    .expect("the line in the trace");
                (f, l, *w)
            })
            .collect();
        assert_eq!(ours.words, theirs, "the stream past the map");
        // And past `build_empire` (item 4063): 1148 after both empires'
        // units, 1169 after `World::analyze_map` and
        // `compute_all_territory` (no draw), 1240 after
        // the herd loop's 44, 1244 the walk's last setup line.
        assert_eq!(
            ours.words.iter().map(|w| w.1).collect::<Vec<_>>(),
            [955, 13457, 1032, 1148, 1169, 1240, 1244]
        );
        // Inside the first `build_empire` (seat 1, the computer): the
        // stream after the village and each pre-placed building, against
        // the draw-site trace of the same lobby (`rontrace-run100.log`,
        // `tools/trace/report.py … draws setup`): the seed before the first
        // draw of each group from 1032's word on. The village draws
        // nothing (`build_cities` calls `Build::init` itself); the camp's
        // `Objects::init_build` spends 244 in `Build::find_gather_tiles`;
        // each farm its `produce_building` candidates (+0xc99), its jitter
        // (+0x1805) and its `Farms::add`; the library its jitter and then
        // `find_road`'s 189 for the city's roads. The next group is
        // `Setup::place_unit`'s, `build_units`', which this crate does not
        // carry (`docs/START.md` §2 step 7).
        let seat1: Vec<(&str, u32)> = ours
            .empire_words
            .iter()
            .filter(|w| w.0 == 1)
            .map(|w| (w.1, w.2))
            .collect();
        assert_eq!(
            seat1,
            [
                ("village", 0xf229_9eda),
                ("woodcutter", 0x726d_bbde),
                ("farm", 0x82cb_af4f),
                ("farm", 0xc770_881b),
                ("farm", 0xa4b0_725a),
                ("library", 0x4a91_dcb1),
                ("units", 0xddf9_d984),
            ],
            "seat 1's build_empire against run100's draw sites"
        );
        // `build_units` (item 4063): `Setup::place_unit`'s tries for the
        // scout (2 here, 3 for seat 0), then `Guy::init_real` once a
        // figure for the scout and the five citizens (7) — the word
        // after them is the seed of seat 0's camp's first
        // `find_gather_tiles` draw. Seat 0's own `build_empire` against
        // the same trace: each word the seed of the next group's first
        // draw — the camp's 244 then farm 1's candidates (`+0xc99`), farm
        // 2's, farm 3's, the library's jitter (`small_city_buildings
        // +0xc0`) and roads, the Market's jitter (`build_civ_specific
        // +0x5c`) and roads, then the units, whose last word is 1148's.
        let seat0: Vec<(&str, u32)> = ours
            .empire_words
            .iter()
            .filter(|w| w.0 == 0)
            .map(|w| (w.1, w.2))
            .collect();
        assert_eq!(
            seat0,
            [
                ("village", 0xddf9_d984),
                ("woodcutter", 0x161f_73ec),
                ("farm", 0x21cf_0167),
                ("farm", 0xaefd_187c),
                ("farm", 0x3161_0a10),
                ("library", 0xc73d_274a),
                ("civ-specific", 0xf150_44cb),
                ("units", 0x1675_cd7d),
            ],
            "seat 0's build_empire against run100's draw sites"
        );
    }

    /// **The harness's own lobby, built from its choices and compared with
    /// the installed start** (item 4033): run9 (`WORLD=6`, the start's
    /// cells) with its siblings run11 (the setup trace) and run3 (the
    /// heights and regions), seed 12345, Great Lakes, Small, two players.
    /// The installed `Sim` is [`build_sim`]'s; ours is [`from_lobby`] on
    /// the `GAMEINFO` block alone. Pins the first parted row and the
    /// count of rows that agree, so a later item that builds further
    /// moves both. The counter, `START_BUILT`, is every lobby's (item
    /// 4034).
    #[test]
    fn the_harness_lobby_s_start_built_from_its_choices() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run9-world6.txt") else {
            eprintln!("skipping: no gamelog-run9-world6.txt (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        with_sibling_initials(|sibs| {
            let text = crate::capture::read(&path);
            let log = Log::parse(&text);
            let mut init = replay_initial(&log).unwrap();
            borrow_from_siblings(&mut init, sibs);
            let players = player_count(&init);
            let theirs = build_sim(&loaded, &init, Tuning::RON);
            let ours = from_lobby(&loaded, &init.game_info, &init.players, Tuning::RON);
            let a = start_block(&ours.sim, players);
            let b = start_block(&theirs.sim, players);
            let parted = compare_start(&a, &b);
            let agree = b.len() - parted.iter().filter(|p| p.theirs.is_some()).count();
            if std::env::var_os("RON_START_DIFF").is_some() {
                for (s, built) in &ours.steps {
                    eprintln!("step {s}: {}", if *built { "built" } else { "not built" });
                }
                let mut by_field: std::collections::BTreeMap<(String, &str), usize> =
                    Default::default();
                for p in &parted {
                    let kind = p.record.split(' ').next().unwrap_or("").to_string();
                    *by_field.entry((kind, p.field)).or_default() += 1;
                }
                for ((k, f), n) in &by_field {
                    eprintln!("parted {k} {f}: {n}");
                }
                let all = std::env::var("RON_START_DIFF").is_ok_and(|v| v == "all");
                for p in parted.iter().take(if all { usize::MAX } else { 40 }) {
                    eprintln!("{p}");
                }
                eprintln!("agree {agree} of {}", b.len());
            }
            // The herd list the walk founded its animals from, against the
            // start dump's `HERDS` (run12's, borrowed): every herd's cell,
            // walk centre, type and alive bit, in the list's order — the
            // order `build_game`'s herd loop visits and the tag each animal
            // carries (item 4063).
            let herd = |h: &sim::gaia::Herd| (h.cx, h.cy, h.wx, h.wy, h.kind, h.alive);
            let (mine, yours): (Vec<_>, Vec<_>) = (
                ours.sim.gaia.gaia.herds.iter().map(herd).collect(),
                theirs.sim.gaia.gaia.herds.iter().map(herd).collect(),
            );
            assert!(!yours.is_empty(), "the installed side carries no HERDS");
            assert_eq!(mine, yours, "the herds, against the start dump's HERDS");
            let first = parted.first().map(ToString::to_string);
            assert_eq!(
                (first.as_deref(), agree, b.len()),
                (START_FIRST_PARTING, START_AGREE.0, START_AGREE.1),
                "the harness lobby's start (RON_START_DIFF=1 prints the value diff)"
            );
            // The counter is every lobby's
            // (`every_lobby_s_start_built_from_its_choices`, item 4034);
            // this lobby is one of them.
            assert!(START_BUILT >= usize::from(parted.is_empty()));
        });
    }

    /// **The counter's harness: every lobby on disk** (item 4034,
    /// `docs/START.md` §5): each lobby `lobby::lobbies_on_disk` finds, its
    /// richest start dump installed by [`build_sim`] (the lobby's other
    /// captures lent through `borrow_from_siblings`), against
    /// [`from_lobby`] of its `GAMEINFO`. A lobby is built when no field
    /// parts. `RON_START_DIFF=1` prints each lobby's first parting by
    /// record and field and the census of them.
    #[test]
    fn every_lobby_s_start_built_from_its_choices() {
        let Some(inst) = install() else { return };
        let Some(dir) = crate::testenv::gamelog_dir() else {
            eprintln!("skipping: no gamelog directory (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let lobbies = crate::lobby::lobbies_on_disk(&dir, crate::corpus::Ledger::of_tree())
            .expect("the logs");
        assert_eq!(
            lobbies.len(),
            START_LOBBIES,
            "tools/start_lobbies.py's count"
        );
        let print = std::env::var_os("RON_START_DIFF").is_some();
        let mut census: std::collections::BTreeMap<String, usize> = Default::default();
        let mut built = 0;
        let only: Option<Vec<usize>> = std::env::var("RON_START_LOBBY")
            .ok()
            .map(|v| v.split(',').filter_map(|s| s.trim().parse().ok()).collect());
        for (n, lobby) in lobbies.iter().enumerate() {
            if only.as_ref().is_some_and(|o| !o.contains(&n)) {
                continue;
            }
            let texts: Vec<(String, String)> = lobby
                .captures
                .iter()
                .filter_map(|c| {
                    let path = dir.join(c);
                    let mut cap = crate::capture::indexed::IndexedCapture::open(&path).ok()?;
                    Some((c.clone(), cap.read_replay_setup().ok()?))
                })
                .collect();
            let logs: Vec<(&str, Log)> = texts
                .iter()
                .map(|(c, t)| (c.as_str(), Log::parse(t)))
                .collect();
            let inits: Vec<(&str, Initial, &Log)> = logs
                .iter()
                .filter_map(|(c, l)| Some((*c, l.replay_initial()?, l)))
                .collect();
            let rich = |i: &Initial| {
                (
                    !crate::gamelog::world_cells(&i.world).is_empty(),
                    !i.checksums.is_empty(),
                    i.units.len() + i.builds.len(),
                )
            };
            let Some(best) = (0..inits.len()).max_by_key(|&k| (rich(&inits[k].1), usize::MAX - k))
            else {
                let line = "no capture reads".to_string();
                *census.entry(line.clone()).or_default() += 1;
                if print {
                    eprintln!("lobby {n} {}: {line}", lobby.captures[0]);
                }
                continue;
            };
            let mut init = inits[best].1.clone();
            let sibs: Vec<&Initial> = inits
                .iter()
                .enumerate()
                .filter(|(k, _)| *k != best)
                .map(|(_, (_, i, _))| i)
                .collect();
            borrow_from_siblings(&mut init, &sibs);
            let players = player_count(&init);
            let mut theirs = build_sim(&loaded, &init, Tuning::RON);
            // The personality the installer rolls is the original's only
            // from the setup trace's bracket; the start dump's own
            // `PERSONALITY` block (`LEADERS=9`) is the original's outright,
            // so where it prints one it is what the built side is held to.
            let brackets = personality_brackets(&init.checksums).len();
            let mut unbacked: Vec<usize> = Vec::new();
            let computers: Vec<usize> = (0..players.min(theirs.sim.ai.ai.len()))
                .filter(|&w| !theirs.sim.cities.nation[w].human)
                .collect();
            for &w in &computers {
                match dump_personality(inits[best].2, w as i64) {
                    Some(p) => theirs.sim.ai.ai[w].pers = p,
                    None if brackets < computers.len() => unbacked.push(w),
                    None => {}
                }
            }
            let ours = from_lobby(&loaded, &init.game_info, &init.players, Tuning::RON);
            let a = start_block(&ours.sim, players);
            let b = start_block(&theirs.sim, players);
            let (parted, mut stand_in) = dump_backed(
                compare_start(&a, &b),
                init.pasture.is_empty(),
                init.checksums.is_empty(),
            );
            // **A thin dump** (4034a, settled by item 4196): a start dump
            // that prints no cells, no `diplos`, or a building without its
            // `orig_type` leaves the installer on its own defaults there
            // (`World::new`'s cells, the war matrix `Leader::init` would
            // write, `start_of_game`'s type by object number, which names
            // no 2005 or 2006) — the crate against itself. Those rows are
            // stand-ins; the rows such a dump does print (the buildings'
            // places, the units, `WORLD`'s extent and seed, the nations)
            // are what it can fail on, and stay compared.
            let printed = crate::gamelog::world_cells(&init.world);
            let no_cells = printed.is_empty();
            // **`CELL flags` is held to the dump's printed word** (item
            // 4211): the installer stands every dumped building up through
            // `activate`, which lays the city's roads again on the dump's
            // cells, so its `ROAD` bit is the crate's road plan, not the
            // original's. Warring States' and British Isles' starts each
            // part from the installer on one cell (run1122 `CELL 7 38`, run316
            // `CELL 15 21`) where the dump prints no road and ours lays none.
            let parted: Vec<Parting> = if no_cells {
                parted
            } else {
                let xs = ours.sim.world.width();
                let flags_of = |fs: &[Field], rec: &str| {
                    fs.iter()
                        .find(|(r, f, _)| r == rec && *f == "flags")
                        .map(|(_, _, v)| v.clone())
                };
                let mut kept: Vec<Parting> = parted
                    .into_iter()
                    .filter(|p| !(p.field == "flags" && p.record.starts_with("CELL ")))
                    .collect();
                for (i, c) in printed.iter().enumerate() {
                    let rec = format!("CELL {} {}", i as i32 % xs, i as i32 / xs);
                    let mine = flags_of(&a, &rec);
                    if mine.as_deref() != Some(c.flags.to_string().as_str()) {
                        kept.push(Parting {
                            record: rec,
                            field: "flags",
                            ours: mine,
                            theirs: Some(c.flags.to_string()),
                        });
                    }
                }
                kept
            };
            let no_diplos = init.leaders.iter().all(|l| l.diplos.is_empty());
            let untyped: Vec<String> = init
                .builds
                .iter()
                .filter(|b| b.orig_type.is_none())
                .map(|b| format!("BUILD {}/{}", b.who, b.o))
                .collect();
            let mut parted: Vec<Parting> = parted
                .into_iter()
                .filter(|p| {
                    let thin = (no_cells && p.record.starts_with("CELL "))
                        || (no_diplos && p.field == "at_war")
                        || (p.field == "type"
                            && p.theirs.as_deref() == Some("None")
                            && untyped.contains(&p.record));
                    stand_in += usize::from(thin);
                    !thin
                })
                .collect();
            // A Southwest Mesa lobby's cliffs (item 4196): the mining lists
            // our chain laid at the lobby's own stream against those
            // `build_sim` lays from the same chain at the setup trace's
            // 7805 seed. No dump prints `cliff_mining_data`, so this holds
            // our walk to the trace's word, not to the original's lists.
            if !theirs.sim.terrain.cliffs.is_empty()
                && ours.sim.terrain.cliffs != theirs.sim.terrain.cliffs
            {
                parted.push(Parting {
                    record: "CLIFFS".into(),
                    field: "mining",
                    ours: Some(format!("{} lists", ours.sim.terrain.cliffs.len())),
                    theirs: Some(format!("{} lists", theirs.sim.terrain.cliffs.len())),
                });
            }
            // Each farm's `farm_type` — the pasture is type 1 — against the
            // start dump's own `FARMS` list where it prints one.
            for f in &init.farms {
                let ours_type = ours
                    .sim
                    .buildings
                    .iter()
                    .find(|bd| {
                        bd.alive && i64::from(bd.owner) == f.who && i64::from(bd.index) == f.o
                    })
                    .map(|bd| i64::from(bd.farm.farm_type));
                if ours_type != Some(f.farm_type) {
                    parted.push(Parting {
                        record: format!("FARM {}/{}", f.who, f.o),
                        field: "farm_type",
                        ours: ours_type.map(|t| t.to_string()),
                        theirs: Some(f.farm_type.to_string()),
                    });
                }
            }
            let parted: Vec<Parting> = parted
                .into_iter()
                .filter(|p| {
                    let drop = p.field == "personality"
                        && unbacked.iter().any(|w| p.record == format!("LEADER {w}"));
                    stand_in += usize::from(drop);
                    !drop
                })
                .collect();
            let agree = b.len() - parted.iter().filter(|p| p.theirs.is_some()).count();
            let style = init
                .game_info
                .iter()
                .find(|(k, _)| *k == "MAP_STYLE")
                .map_or("?", |(_, v)| v.trim());
            let key = match parted.first() {
                None => {
                    built += 1;
                    "built".to_string()
                }
                Some(p) => format!("{} {}", p.record.split(' ').next().unwrap_or(""), p.field),
            };
            *census.entry(key.clone()).or_default() += 1;
            if print {
                let (cells, trace, objs) = rich(&init);
                eprintln!(
                    "lobby {n} {} style {style} cells {cells} trace {trace} objs {objs}: {} agree {agree} of {} (stand-in {stand_in})",
                    inits[best].0,
                    parted
                        .first()
                        .map_or("none".to_string(), ToString::to_string),
                    b.len()
                );
                if only.is_some() {
                    for (s, built) in &ours.steps {
                        eprintln!("  step {s}: {}", if *built { "built" } else { "not built" });
                    }
                    for note in &ours.notes {
                        eprintln!("  note {note}");
                    }
                    let all = std::env::var("RON_START_DIFF").is_ok_and(|v| v == "all");
                    for p in parted.iter().take(if all { usize::MAX } else { 40 }) {
                        eprintln!("  {p}");
                    }
                }
            }
        }
        if only.is_some() {
            return;
        }
        if print {
            for (k, v) in &census {
                eprintln!("census {k}: {v}");
            }
            eprintln!("built {built} of {}", lobbies.len());
        }
        let census: Vec<(&str, usize)> = census.iter().map(|(k, v)| (k.as_str(), *v)).collect();
        assert_eq!(
            (built, census.as_slice()),
            (START_BUILT, START_CENSUS),
            "Start: N of 161 and each lobby's first parting by record and field \
             (RON_START_DIFF=1 prints the census)"
        );
    }

    /// **The census**: every lobby's first parting by record and field,
    /// counted (items 4034, 4186; the value diff per lobby is each item's
    /// journal's). None is left (item 4211: Warring States, run1122, and
    /// British Isles, run1127 and run316, parted at `CELL 0 0 land`, ours 0
    /// theirs 2, with no mask port; item 4186 composed East Indies and Great Sahara; item 4196
    /// Southwest Mesa and the eleven chained styles, and 4034a: 4033a's
    /// scratch lobby, a Great Sahara dump that prints neither cells nor
    /// `diplos`, its unprinted rows stand-ins). The nation arms
    /// (`docs/START.md` §2 step 7, item 4186) took 29 `CELL flags`: a
    /// Spanish, American, Dutch, Korean or Greek seat on Great Lakes (8),
    /// East Indies (11, one the fuzz lobby) and Great Sahara (10, one a
    /// classical melee).
    const START_CENSUS: &[(&str, usize)] = &[("built", 161)];

    /// A computer leader's `PERSONALITY` block under the start dump's
    /// `LEADERDATA` (`LEADERS=9`), as the crate's record; none below that
    /// detail.
    fn dump_personality(log: &Log, who: i64) -> Option<sim::ai::Personality> {
        let block = log
            .game()?
            .kids("LEADERDATA")
            .find(|l| l.int("who") == Some(who))?
            .kid("PERSONALITY")?;
        let mut p = sim::ai::Personality::default();
        let mut n = 0;
        for (k, v) in block.fields().take(24) {
            let Ok(v) = v.trim().parse::<i32>() else {
                continue;
            };
            let slot = match k {
                "rush" => &mut p.rush,
                "cities" => &mut p.cities,
                "upgrades" => &mut p.upgrades,
                "arms" => &mut p.arms,
                "army" => &mut p.army,
                "army_size" => &mut p.army_size,
                "raid" => &mut p.raid,
                "invade" => &mut p.invade,
                "target" => &mut p.target,
                "strategy" => &mut p.strategy,
                "raze" => &mut p.raze,
                "spells" => &mut p.spells,
                "forts" => &mut p.forts,
                "nukes" => &mut p.nukes,
                "air" => &mut p.air,
                "naval" => &mut p.naval,
                "market" => &mut p.market,
                "scouts" => &mut p.scouts,
                "civilians" => &mut p.civilians,
                "early_army" => &mut p.early_army,
                "friendly_human" => &mut p.friendly_human,
                "alliance_human" => &mut p.alliance_human,
                "friendly_ai" => &mut p.friendly_ai,
                "alliance_ai" => &mut p.alliance_ai,
                _ => continue,
            };
            *slot = v;
            n += 1;
        }
        (n == 24).then_some(p)
    }

    /// **The partings a dump can back** (item 4034, `docs/START.md` §3): a
    /// starting pasture's five (owner 9) are in no dump, and the installer
    /// stands them on the farm's centre with one type unless a trace lends
    /// their seeds (`borrow_pasture`). Without seeds their `type` and `pos`
    /// are that stand-in, and with no farm list it stands none: every
    /// owner-9 parting is dropped and counted. The sync word, the setup
    /// trace's, is what holds the twenty draws — and with no trace on disk
    /// for the lobby the installer's word is its own stream's, dropped and
    /// counted the same way.
    fn dump_backed(parted: Vec<Parting>, no_seeds: bool, no_trace: bool) -> (Vec<Parting>, usize) {
        let (kept, dropped): (Vec<Parting>, Vec<Parting>) = parted.into_iter().partition(|p| {
            !(no_seeds && p.record.starts_with("UNIT 9/") || no_trace && p.record == "GAME")
        });
        (kept, dropped.len())
    }

    /// The harness lobby's first parted row, `record field: ours theirs`:
    /// none (item 4063, `docs/START.md` §4). Item 4033 pinned `LEADER 0
    /// at_war`; item 4046 built the war row and the map's first cell
    /// parted; item 4047 composed Great Lakes' chain, the villages and the
    /// pre-placed buildings, and `CELL 2 37 flags` (ours 512 theirs 640)
    /// parted round the human's farms, nine draws short; item 4063 built
    /// `build_units` (the scout's `place_unit`, the citizens born at their
    /// buildings and stepped off by `Unit::come_out(u, 0)`) and the herds.
    const START_FIRST_PARTING: Option<&str> = None;
    /// The harness lobby's agreeing fields, of the installed block's:
    /// 18,264 of 18,264 (item 4063, +225 on 4047's 18,039: the human's
    /// five buildings' positions and 11 footprint flags, the 52 units'
    /// four fields each — the players' 12, gaia's 40 — and the sync word).
    const START_AGREE: (usize, usize) = (18264, 18264);
}
