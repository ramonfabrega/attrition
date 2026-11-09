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
/// (item 4033): the harness's own lobby parts at the map (§4).
pub const START_BUILT: usize = 0;

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
/// 4. **the map** — `Map::make`, `Terrain::init` — which no path in this
///    crate composes into a `World` yet: the walk stops here, and every
///    step after it (the start permutation, `Leader::init`'s personality,
///    each `build_empire`, `Herd::create_units`, `compute_all_territory`)
///    is listed as not built.
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
    let mut sim = loaded.sim(tuning, World::new(n, n), players);
    // 2. `init_rules_and_teams`.
    sim.ai.lobby = lobby_of(
        game_info,
        &loaded.map_styles,
        &loaded.wonder_wins,
        &loaded.starting_resources,
        &loaded.pop_wins,
        &loaded.rush_minutes,
    );
    sim.setup().sync_setup_from_lobby();
    let seed = int("(int)seed").unwrap_or(0) as u32;
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
    // 3. `build_game`'s re-seed and the world seed.
    let mut rng = sim::combat::Rng::new(seed);
    sim.game_seed = rng.get(0, 0xffff) % 0xffff + 1;
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
    // 4. The map, and everything after it.
    for step in [
        "Map::make and Terrain::init",
        "the start permutation",
        "Leader::init (the personality roll)",
        "build_empire: build_cities, produce_building, build_civ_specific",
        "build_empire: build_units, place_unit",
        "World::analyze_map, compute_all_territory",
        "Herd::create_units",
    ] {
        steps.push((step, false));
    }
    notes.push("the walk stops at the map: no path composes Map::make into a World".into());
    FromLobby { sim, steps, notes }
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

    /// **The harness's own lobby, built from its choices and compared with
    /// the installed start** (item 4033): run9 (`WORLD=6`, the start's
    /// cells) with its siblings run11 (the setup trace) and run3 (the
    /// heights and regions), seed 12345, Great Lakes, Small, two players.
    /// The installed `Sim` is [`build_sim`]'s; ours is [`from_lobby`] on
    /// the `GAMEINFO` block alone. Pins the first parted row and the
    /// count of rows that agree, so a later item that builds further
    /// moves both. `START_BUILT` stays 0 while any row parts.
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
                for p in parted.iter().take(40) {
                    eprintln!("{p}");
                }
                eprintln!("agree {agree} of {}", b.len());
            }
            let first = parted.first().map(ToString::to_string);
            assert_eq!(
                (first.as_deref(), agree, b.len()),
                (START_FIRST_PARTING, START_AGREE.0, START_AGREE.1),
                "the harness lobby's start (RON_START_DIFF=1 prints the value diff)"
            );
            assert_eq!(START_BUILT, usize::from(parted.is_empty()));
        });
    }

    /// The harness lobby's first parted row, `record field: ours theirs`:
    /// the map's first cell, which no path composes into a `World` yet
    /// (item 4047; `docs/START.md` §4). Item 4033 pinned `LEADER 0 at_war`,
    /// ours `[false, false]` theirs `[false, true]`; item 4046 built its
    /// writer, `Leader::init`'s diplomacy row, and both war rows agree.
    /// Every row ahead agrees: the leaders, the extent, the world seed.
    const START_FIRST_PARTING: Option<&str> = Some("CELL 0 0 region: ours false theirs true");
    /// The harness lobby's agreeing fields, of the installed block's:
    /// 9,089 of 18,264 (item 4046, +2 on 4033's 9,087: the two war rows) —
    /// the parted are the map's cells (3,600 region, 2,303 val, 1,706
    /// flags, 790 land, 527 owner), the personality, 13 buildings and 52
    /// units (each of four fields) and the sync word.
    const START_AGREE: (usize, usize) = (9089, 18264);
}
