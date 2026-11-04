//! **Map generation's reader** (`docs/MAPGEN.md` §3, §15): a start dump's
//! cells packed into the `WData` bytes `World::walk_data@006b5cf0`'s part 5
//! walks, so that `sim::mapgen`'s port can be checked against the setup
//! trace's own Adler-32 at every phase's checkpoint.
//!
//! A start dump's `WORLD` block sits past the `CONSTANTS` and type tables,
//! 15 MB into a file, and the frames after it run to gigabytes; [`start_head`]
//! reads a file only to the end of that block, and the crate's own
//! [`Log::parse`] reads what it returns.

use crate::gamelog::{CellDump, Log, WorldChecksum, world_cells};
use sim::map_resources::{Footprint, GOODY, Inputs as ResourceInputs, NO_TYPE, Row};
use sim::mapgen::{Adler32, WCell};
use sim::terrain_groups::{Group, Inputs, Template, TilesetGroups};
use std::io::{self, BufRead, BufReader};
use std::path::Path;

/// `land_key[]`, the names a cell's level-2 line prints, in index order.
pub const LAND_KEY: [&str; 4] = ["BASELAND", "SANDY", "OCEAN", "NONE"];

/// One dumped cell as its `WData`, or `None` for a terrain name outside
/// [`LAND_KEY`]. Every field is stored at the width `types.txt` gives it,
/// so a dumped value outside that width wraps as the original's store does.
pub fn wcell(c: &CellDump<'_>) -> Option<WCell> {
    Some(WCell {
        flags: c.flags as u16,
        land: LAND_KEY.iter().position(|k| *k == c.land)? as i8,
        land_sub: c.land_sub as u8,
        region: c.region as i16,
        region2: c.region2 as i16,
        down: c.down as i16,
        down_who: c.down_who as i16,
        val: c.val as u8,
        goods: c.goods as u8,
        light: c.light as u8,
        who: c.who as i8,
        who2: c.who2 as i8,
        blocked: c.blocked as u8,
        bad: c.bad as u8,
        solid: c.solid as i8,
        was_seen: c.was_seen as u8,
    })
}

/// Part 5 of a dumped grid: every cell's walked bytes, in dump order
/// (`wdata[0..size]`, row-major), as one Adler-32.
pub fn cell_part(cells: &[WCell]) -> u32 {
    let mut s = Adler32::default();
    for c in cells {
        s.feed(&c.walked());
    }
    s.sum()
}

/// A file's text up to the end of its first `WORLD` block — the preamble's
/// setup trace, `GAME INFO`, and the start dump's grid. `None` when the
/// preamble carries no `World` line (no setup trace at
/// `check_all_level ≥ 14`), which is most of the archive and is decided in
/// its first few thousand lines. A file with no `WORLD` before its first
/// frame is returned to that frame.
pub fn start_head(path: &Path) -> io::Result<Option<String>> {
    let mut r = BufReader::with_capacity(1 << 20, std::fs::File::open(path)?);
    let mut out = String::new();
    let mut line = Vec::new();
    let mut in_preamble = true;
    let mut traced = false;
    let mut world: Option<usize> = None;
    loop {
        line.clear();
        if r.read_until(b'\n', &mut line)? == 0 {
            break;
        }
        let text = String::from_utf8_lossy(&line);
        let body = text.trim_start();
        let indent = text.len() - body.len();
        let body = body.trim_end();
        if in_preamble && body.starts_with("BEGIN ") {
            if !traced {
                return Ok(None);
            }
            in_preamble = false;
        }
        if in_preamble && body.starts_with("World ") {
            traced = true;
        }
        if let Some(w) = world {
            if !body.is_empty() && indent <= w {
                break;
            }
        } else if body == "BEGIN WORLD" {
            world = Some(indent);
        } else if body.starts_with("BEGIN FRAME") {
            break;
        }
        out.push_str(&text);
    }
    Ok(traced.then_some(out))
}

/// What one file's head holds for the falsifier: its setup trace, the
/// start grid, and the lobby the generator read.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StartGrid {
    /// `(file, line, seed, parts)` per record — [`WorldChecksum`], owned.
    pub checksums: Vec<(String, i64, u32, [u32; 16])>,
    /// The start dump's cells; `None` when the head has no `WORLD` grid.
    pub cells: Option<Vec<WCell>>,
    /// Dumped cells whose terrain names no [`LAND_KEY`] entry, which
    /// [`StartGrid::cells`] leaves out.
    pub unnamed: usize,
    pub xs: i32,
    pub ys: i32,
    /// The start dump's `World.seed` (+0x7c), which `Setup::build_game` draws
    /// before `Map::make` and every `Fractal` of map generation is seeded
    /// from.
    pub seed: i32,
    /// `GAMEINFO`'s `MAP_STYLE` (`MapStyleIndex`: Great Sahara 7).
    pub style: i64,
    /// The `PLAYER` blocks under `GAMEINFO`: the nation count `Map::make`
    /// takes as its first argument.
    pub nations: i32,
}

impl StartGrid {
    /// The record at `file`:`line`, the first if the trace prints it twice.
    pub fn at(&self, file: &str, line: i64) -> Option<&(String, i64, u32, [u32; 16])> {
        self.checksums
            .iter()
            .find(|(f, l, _, _)| f == file && *l == line)
    }
}

/// Reads [`start_head`]'s text into a [`StartGrid`].
pub fn start_grid(head: &str) -> StartGrid {
    let log = Log::parse(head);
    let checksums = log
        .world_checksums()
        .iter()
        .map(|c: &WorldChecksum<'_>| (c.file.to_string(), c.line, c.seed, c.parts))
        .collect();
    let mut g = StartGrid {
        checksums,
        ..StartGrid::default()
    };
    let Some(init) = log.initial() else {
        return g;
    };
    let int = |k: &str| {
        init.game_info
            .iter()
            .find(|(f, _)| *f == k)
            .and_then(|(_, v)| v.trim().parse::<i64>().ok())
    };
    g.style = int("MAP_STYLE").unwrap_or(-1);
    g.nations = init.players.len() as i32;
    let world_int = |k: &str| {
        init.world
            .iter()
            .find(|(f, _)| *f == k)
            .and_then(|(_, v)| v.trim().parse::<i32>().ok())
            .unwrap_or(0)
    };
    g.xs = world_int("xs");
    g.ys = world_int("ys");
    g.seed = world_int("seed");
    let dumped = world_cells(&init.world);
    if !dumped.is_empty() {
        let cells: Vec<WCell> = dumped.iter().filter_map(wcell).collect();
        g.unnamed = dumped.len() - cells.len();
        g.cells = Some(cells);
    }
    g
}

/// What `TerrainGroups::init_tileset_data@006a61f0` reads for
/// `fill_fertile` (`docs/MAPGEN.md` §6): the tileset's `CLUMP_FACTOR`, the
/// `Fractal`'s smoothing, and one `BASELAND` frequency per base tile.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fertility {
    pub tileset: String,
    pub clump_factor: i32,
    pub baseland: Vec<i32>,
}

/// [`Fertility`] for the style whose file is `mapstyles/<style_file>`, from
/// the install at `root`; `None` when the style's `TILECHANCE` list is not a
/// single entry (the tileset is then `init_map_data`'s draw, §4) or a file
/// is missing. The frequencies are `default.xml`'s `TILESET_DATA` entry for
/// the tileset, `frequency_0..` on its `baseland` `LANDKEY`, one per `BASE`
/// under the tileset's `BASELAND` in `Data/tilesets.xml`; the style file's
/// own entry, if it has one, is read over it as the second call reads it —
/// a frequency it leaves out becomes −1, `get_attrib_num`'s default.
pub fn fertility(root: &Path, style_file: &str) -> Option<Fertility> {
    let read = |rel: &str| std::fs::read_to_string(root.join(rel)).ok();
    let style_text = read(&format!("mapstyles/{style_file}"))?;
    let style = roxmltree::Document::parse(&style_text).ok()?;
    let chances: Vec<_> = style
        .descendants()
        .filter(|n| n.has_tag_name("TILECHANCE"))
        .collect();
    let [only] = chances[..] else {
        return None;
    };
    let tileset = only.attribute("type")?.to_string();
    let sets_text = read("Data/tilesets.xml")?;
    let sets = roxmltree::Document::parse(&sets_text).ok()?;
    let set = sets.descendants().find(|n| {
        n.has_tag_name("TILESET")
            && n.attribute("name")
                .is_some_and(|v| v.eq_ignore_ascii_case(&tileset))
    })?;
    let clump_factor = set
        .descendants()
        .find(|n| n.has_tag_name("CLUMP_FACTOR"))
        .and_then(|n| n.attribute("value")?.trim().parse().ok())
        .unwrap_or(-1);
    let tiles = set
        .descendants()
        .find(|n| n.has_tag_name("BASELAND"))?
        .children()
        .filter(|n| n.has_tag_name("BASE"))
        .count();
    let mut baseland = vec![0; tiles];
    let default_text = read("mapstyles/default.xml")?;
    let default = roxmltree::Document::parse(&default_text).ok()?;
    for doc in [&default, &style] {
        let Some(entry) = doc
            .descendants()
            .find(|n| n.has_tag_name("TILESET_DATA"))
            .and_then(|d| {
                d.children()
                    .find(|n| n.tag_name().name().eq_ignore_ascii_case(&tileset))
            })
        else {
            continue;
        };
        for key in entry.children().filter(|n| {
            n.has_tag_name("LANDKEY")
                && n.attribute("name")
                    .is_some_and(|v| v.eq_ignore_ascii_case("baseland"))
        }) {
            for (i, f) in baseland.iter_mut().enumerate() {
                *f = key
                    .attribute(format!("frequency_{i}").as_str())
                    .and_then(|v| v.trim().parse().ok())
                    .unwrap_or(-1);
            }
        }
    }
    Some(Fertility {
        tileset,
        clump_factor,
        baseland,
    })
}

/// One setup draw of `game_random`, as a draw-site trace records it: the
/// return address into the drawing function, normalised to
/// [`crate::trace::IMAGE_BASE`], and the word **before** the step.
pub type SetupDraw = (u32, u32);

/// Every `game_random` draw of a draw-site trace's **setup path**, in order
/// — the records before its first `FRAME` (`crate::trace`'s format). A trace
/// runs to gigabytes and map generation is in its first few megabytes, so
/// this reads records only until that `FRAME` and never parses the rest.
/// `None` for a file that is not a trace.
pub fn setup_draws(path: &Path) -> io::Result<Option<Vec<SetupDraw>>> {
    use crate::trace::{IMAGE_BASE, RVA_GAME_RANDOM};
    use std::io::Read;
    let mut r = BufReader::with_capacity(1 << 20, std::fs::File::open(path)?);
    let mut rec = [0u8; 32];
    if r.read_exact(&mut rec).is_err() || &rec[..4] != b"RONT" {
        return Ok(None);
    }
    let word = |b: &[u8; 32], i: usize| u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]);
    let base = word(&rec, 8);
    let norm = |va: u32| va.wrapping_sub(base).wrapping_add(IMAGE_BASE);
    let mut out = Vec::new();
    while r.read_exact(&mut rec).is_ok() {
        match word(&rec, 0) {
            2 => break,
            1 | 3 if norm(word(&rec, 8)) == IMAGE_BASE + RVA_GAME_RANDOM => {
                out.push((norm(word(&rec, 4)), word(&rec, 12)));
            }
            _ => {}
        }
    }
    Ok(Some(out))
}

/// **`Map::scale_number@006a0320`**: an XML count, its leading integer
/// scaled by the word after the first space. A negative value is per
/// nation (`−v · mult`); `SCALE` scales by the map's width and `AREA` by
/// its area, each against `standard`, `rules.xml`'s Standard map size
/// (`map_sizes` record 3); any other word (`min`, `mindist`, …) leaves the
/// value as written. `SCALEWATER` and `SCALELAND` take a float path no
/// Great Sahara group reaches, and are refused.
pub fn scale_number(s: &str, mult: i32, xs: i32, size: i32, standard: i32) -> Option<i32> {
    let mut v = leading_int(s);
    if v < 0 {
        v = -(v * mult);
    }
    let suffix = s.find(' ').map_or("", |i| s[i..].trim_start_matches(' '));
    let by = |v: i32, n: i32, b: i32| {
        if v < 1 {
            0
        } else {
            ((b / 2 + n * v) / b).max(1)
        }
    };
    Some(match suffix.to_ascii_uppercase().as_str() {
        "AREA" => by(v, size, standard * standard),
        "SCALE" => by(v, xs, standard),
        "SCALEWATER" | "SCALELAND" => return None,
        _ => v,
    })
}

/// `String::convert_int`: the leading integer, sign included; 0 when there
/// is none.
fn leading_int(s: &str) -> i32 {
    let t = s.trim_start();
    let neg = t.starts_with('-');
    let digits: String = t
        .trim_start_matches(['-', '+'])
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    let v: i32 = digits.parse().unwrap_or(0);
    if neg { -v } else { v }
}

/// `rules.xml`'s `mapsizes` record 3, the size `scale_number` measures
/// against (`*(map_sizes.list.data + 0x144)`: stride 0x58, value at +0x3c).
pub fn standard_map_size(root: &Path) -> Option<i32> {
    let text = std::fs::read_to_string(root.join("Data/rules.xml")).ok()?;
    let doc = roxmltree::Document::parse(&text).ok()?;
    let sizes = doc
        .descendants()
        .find(|n| n.has_tag_name("CATEGORIES") && n.attribute("id") == Some("mapsizes"))?;
    let rec = sizes
        .children()
        .filter(|n| n.has_tag_name("CATEGORY"))
        .nth(3)?;
    rec.children()
        .find(|n| n.has_tag_name("DATA"))?
        .text()?
        .trim()
        .parse()
        .ok()
}

/// **`TerrainGroups::init_terrain_data@006a6540`** on the style file's
/// `TERRAIN_GROUPS` (`docs/MAPGEN.md` §7): one [`Group`] per kept
/// `GROUPENTRY`, in document order. The style file's list replaces
/// `default.xml`'s, which is read only when the style has none. `None`
/// for an entry this reader does not take: a map-size condition, a land-key
/// type, or a float-scaled count.
pub fn terrain_groups(
    root: &Path,
    style_file: &str,
    xs: i32,
    ys: i32,
    mult: i32,
) -> Option<Vec<Group>> {
    let standard = standard_map_size(root)?;
    let read = |rel: &str| std::fs::read_to_string(root.join(rel)).ok();
    let style_text = read(&format!("mapstyles/{style_file}"))?;
    let default_text = read("mapstyles/default.xml")?;
    let style = roxmltree::Document::parse(&style_text).ok()?;
    let default = roxmltree::Document::parse(&default_text).ok()?;
    fn list<'a, 'i>(d: &'a roxmltree::Document<'i>) -> Option<roxmltree::Node<'a, 'i>> {
        d.root_element()
            .children()
            .find(|n| n.has_tag_name("TERRAIN_GROUPS"))
    }
    let list = list(&style).or_else(|| list(&default))?;
    let mut out = Vec::new();
    for e in list.children().filter(|n| n.has_tag_name("GROUPENTRY")) {
        let attr = |k: &str| e.attribute(k).unwrap_or("");
        let num = |k: &str| e.attribute(k).map_or(-1, leading_int);
        let scaled = |k: &str| scale_number(attr(k), mult, xs, xs * ys, standard);
        let space = |k: &str| scaled(k).map(|v| v.clamp(0, 0x40));
        let kind = match attr("type").to_ascii_lowercase().as_str() {
            "trees" => 4,
            "mountains" => 5,
            "rocks" => 6,
            "oil" | "sea oil" => 7,
            "cliffs" => 8,
            _ => return None,
        };
        let pattern = match attr("pattern") {
            "player" => 0,
            "continent" => 1,
            "world" => 2,
            "nonplayer" => 3,
            "corner" => 4,
            _ => return None,
        };
        let size = |k: &str| if kind == 5 { Some(num(k)) } else { scaled(k) };
        let (min_oil, max_oil) = if kind == 6 {
            (num("min_oil"), num("max_oil"))
        } else {
            (0, 0)
        };
        out.push(Group {
            kind,
            chance: num("chance"),
            grouping: num("grouping"),
            min_clumps: scaled("min_clumps")?,
            max_clumps: scaled("max_clumps")?,
            pattern,
            min_size: size("min_size")?,
            max_size: size("max_size")?,
            start_min: scaled("city_keep_away")?,
            start_max: scaled("city_stay_near")?.min(0x40),
            forest_space: space("forest_space")?,
            mount_space: space("mount_space")?,
            rock_space: space("rock_space")?,
            coast_space: space("coast_space")?,
            cent_min: scaled("cent_keep_away")?,
            cent_max: scaled("cent_stay_near")?.min(0x40),
            edge_min: scaled("edge_keep_away")?,
            edge_max: scaled("edge_stay_near")?.min(0x40),
            corner_min: scaled("corner_keep_away")?,
            corner_max: scaled("corner_stay_near")?.min(0x40),
            min_oil,
            max_oil,
        });
    }
    Some(out)
}

/// The goods `resourcerules.xml` lists, in record order (the good index):
/// name, footprint, `RANDOM_RARE_DROP`, and `PREQ0`.
fn goods_table(root: &Path) -> Option<Vec<(String, Footprint, bool, String)>> {
    let text = std::fs::read_to_string(root.join("Data/resourcerules.xml")).ok()?;
    let doc = roxmltree::Document::parse(&text).ok()?;
    let out = doc
        .descendants()
        .filter(|n| n.has_tag_name("RESOURCE"))
        .map(|r| {
            let text = |k: &str| {
                r.children()
                    .find(|n| n.has_tag_name(k))
                    .and_then(|n| n.text())
                    .unwrap_or("")
                    .trim()
                    .to_string()
            };
            let num = |k: &str| text(k).parse::<i32>().unwrap_or(0);
            (
                text("NAME"),
                Footprint {
                    x_size: num("X_SIZE").max(1),
                    y_size: num("Y_SIZE").max(1),
                    block: num("BLOCK") != 0,
                },
                num("RANDOM_RARE_DROP") != 0,
                text("PREQ0"),
            )
        })
        .collect();
    Some(out)
}

/// The ages from Gunpowder on: a good whose `PREQ0` names one is a late
/// good (`ObjectTypeData::get_age` ≥ 3, `ResourceDivvyPool::add_good`).
const LATE_AGES: [&str; 5] = [
    "Gunpowder Age",
    "Enlightenment Age",
    "Industrial Age",
    "Modern Age",
    "Information Age",
];

/// **What `Map::place_resources@0068f4f0` reads** (`docs/MAPGEN.md` §8): the
/// `BONUSES`, `FISH ` and `GOODIES` sections — the style file's where it has
/// the section, even empty, else `default.xml`'s (the style's key `FISH `
/// keeps its trailing space, which the lookup ignores) — each `BONUS` row
/// resolved as the ladder reads it; the divvy pool's lists (goods 6..49 with
/// `RANDOM_RARE_DROP`, the ocean's goods to the water list with Fish ten
/// times, the rest early or late by `PREQ0`'s age); the footprints; and
/// `default.xml`'s `LAND_<c>` rare lists. `None` for a row this reader does
/// not take: a float-scaled count.
pub fn resource_inputs(
    root: &Path,
    style_file: &str,
    xs: i32,
    ys: i32,
    nations: i32,
) -> Option<ResourceInputs> {
    let standard = standard_map_size(root)?;
    let read = |rel: &str| std::fs::read_to_string(root.join(rel)).ok();
    let style_text = read(&format!("mapstyles/{style_file}"))?;
    let default_text = read("mapstyles/default.xml")?;
    let style = roxmltree::Document::parse(&style_text).ok()?;
    let default = roxmltree::Document::parse(&default_text).ok()?;
    let goods = goods_table(root)?;
    let good_key = |name: &str| {
        goods
            .iter()
            .position(|g| g.0.eq_ignore_ascii_case(name))
            .map(|i| i as i32)
    };
    let mut lands: Vec<Vec<i32>> = Vec::new();
    for land in default.descendants().filter(|n| {
        n.tag_name()
            .name()
            .strip_prefix("LAND_")
            .is_some_and(|k| k.parse::<usize>().is_ok())
    }) {
        let c: usize = land.tag_name().name()[5..].parse().ok()?;
        if lands.len() <= c {
            lands.resize(c + 1, Vec::new());
        }
        lands[c] = land
            .descendants()
            .filter(|n| n.has_tag_name("RARE"))
            .filter_map(|n| good_key(n.attribute("name")?))
            .collect();
    }
    let ocean = lands.get(2).cloned().unwrap_or_default();
    let (mut early, mut late, mut water) = (Vec::new(), Vec::new(), Vec::new());
    for (g, (_, _, drop, preq)) in goods.iter().enumerate().take(0x32).skip(6) {
        if !drop {
            continue;
        }
        let g = g as i32;
        if ocean.contains(&g) {
            water.extend(std::iter::repeat_n(g, if g == 6 { 10 } else { 1 }));
        } else if LATE_AGES.iter().any(|a| a.eq_ignore_ascii_case(preq)) {
            late.push(g);
        } else {
            early.push(g);
        }
    }
    let section = |style_key: &str, default_key: &str| {
        let find = |d: &'_ roxmltree::Document<'_>, k: &str| {
            d.root_element()
                .children()
                .find(|n| n.has_tag_name(k.trim_end()))
                .map(|n| n.range())
        };
        find(&style, style_key)
            .map(|r| (true, r))
            .or_else(|| find(&default, default_key).map(|r| (false, r)))
    };
    let mut sections = Vec::new();
    let mut pattern = 0;
    for (sk, dk) in [
        ("BONUSES", "BONUSES"),
        ("FISH ", "FISH"),
        ("GOODIES", "GOODIES"),
    ] {
        let mut rows = Vec::new();
        if let Some((in_style, range)) = section(sk, dk) {
            let doc = if in_style { &style } else { &default };
            let node = doc.descendants().find(|n| n.range() == range)?;
            for e in node.children().filter(|n| n.has_tag_name("BONUS")) {
                let attr = |k: &str| e.attribute(k).unwrap_or("");
                let num = |k: &str| e.attribute(k).map_or(-1, leading_int);
                let scaled = |k: &str| scale_number(attr(k), nations, xs, xs * ys, standard);
                let ty = attr("type");
                let (ty, kind) = match good_key(ty) {
                    Some(g) => (g, 0),
                    None if ty.eq_ignore_ascii_case("Random") => (NO_TYPE, 1),
                    None if ty.len() >= 5 && ty[..5].eq_ignore_ascii_case("Goody") => (GOODY, 0),
                    None if ty.eq_ignore_ascii_case("Early") => (NO_TYPE, 2),
                    None if ty.eq_ignore_ascii_case("Late") => (NO_TYPE, 3),
                    None => continue,
                };
                pattern = match attr("pattern") {
                    "player" => 0,
                    "continent" => 1,
                    "world" => 2,
                    "nonplayer" => 3,
                    "corner" => 4,
                    _ => pattern,
                };
                rows.push(Row {
                    ty,
                    kind,
                    chance: num("chance"),
                    group: num("group"),
                    pattern,
                    saturate: num("saturate").max(0),
                    numrare: scaled("numrare")?,
                    player_keep_away: num("player_keep_away"),
                    player_stay_near: scaled("player_stay_near")?,
                    spacing: scaled("spacing")?.max(1),
                    group_spacing: scaled("group_spacing")?.max(1),
                    center_keep_away: scaled("center_keep_away")?,
                    center_stay_near: scaled("center_stay_near")?,
                    edge_keep_away: scaled("edge_keep_away")?,
                    edge_stay_near: scaled("edge_stay_near")?,
                    corner_keep_away: scaled("corner_keep_away")?,
                    corner_stay_near: scaled("corner_stay_near")?,
                });
            }
        }
        sections.push(rows);
    }
    Some(ResourceInputs {
        sections,
        early,
        late,
        water,
        goods: goods.iter().map(|g| g.1).collect(),
        lands,
    })
}

/// The tileset's `TERRAINGROUP` numbers `place_all` reads, for the style's
/// one tileset ([`fertility`]'s), and `num_tiles[0]`, its `BASELAND` count.
pub fn tileset_groups(root: &Path, tileset: &str) -> Option<TilesetGroups> {
    let text = std::fs::read_to_string(root.join("Data/tilesets.xml")).ok()?;
    let doc = roxmltree::Document::parse(&text).ok()?;
    let set = doc.descendants().find(|n| {
        n.has_tag_name("TILESET")
            && n.attribute("name")
                .is_some_and(|v| v.eq_ignore_ascii_case(tileset))
    })?;
    let group = set.children().find(|n| n.has_tag_name("TERRAINGROUP"))?;
    let value = |k: &str| {
        group
            .children()
            .find(|n| n.has_tag_name(k))
            .and_then(|n| n.attribute("value")?.trim().parse().ok())
            .unwrap_or(-1)
    };
    let base_tiles = set
        .children()
        .find(|n| n.has_tag_name("BASELAND"))?
        .children()
        .filter(|n| n.has_tag_name("BASE"))
        .count() as i32;
    Some(TilesetGroups {
        bush_clump_prob: value("BUSH_CLUMP_PROB"),
        mnt_rock_clump_prob: value("MOUNTAIN_ROCK_CLUMP_PROB"),
        mnt_fringe_tree_prob: value("MOUNTAIN_FRINGE_TREE_PROB"),
        base_tiles,
        forest_base: value("FOREST_BASE"),
        forest_growth_prob: value("FOREST_GROWTH_PROB"),
        mountain_base: value("MOUNTAIN_BASE"),
        mountain_growth_prob: value("MOUNTAIN_GROWTH_PROB"),
        coast_base: value("COAST_BASE"),
        coast_growth_prob: value("COAST_GROWTH_PROB"),
    })
}

/// A [`crate::mountains`] template as `place_all` lays it: its tiles, the
/// cells holding them in first-appearance order of the tile scan (`mount_wx`
/// /`_wy`, `tx >> 2`), and its solid cells.
pub fn group_template(t: &sim::gather::MountainTemplate) -> Template {
    let mut cells = Vec::new();
    for &(tx, ty) in &t.tiles {
        let c = (tx.div_euclid(4), ty.div_euclid(4));
        if !cells.contains(&c) {
            cells.push(c);
        }
    }
    Template {
        tiles: t.tiles.clone(),
        cells,
        solid: t.solid.clone(),
    }
}

/// Everything `place_all` reads from the install for `style_file` on an
/// `xs × ys` map: the groups, the sixteen templates, and the tileset's
/// numbers.
pub fn place_all_inputs(
    root: &Path,
    style_file: &str,
    xs: i32,
    ys: i32,
    map_style: i32,
) -> Option<Inputs> {
    let fert = fertility(root, style_file)?;
    let templates = crate::mountains::templates(&crate::Install::new(root))
        .iter()
        .map(group_template)
        .collect();
    Some(Inputs {
        groups: terrain_groups(root, style_file, xs, ys, 1)?,
        templates,
        tileset: tileset_groups(root, &fert.tileset)?,
        map_style,
    })
}

/// The draws of `Map::make` in a draw-site trace from `place_all` on: from
/// the first `Mountains::randomize_mountains` draw through the last
/// `World::compute_val@006b0350` draw of `compute_all_vals` (`docs/MAPGEN.md`
/// §2: `place_all`, its tail at 7861..7870, `place_resources` and the
/// values, with nothing drawing between them). Reads only that far. `None`
/// for a file that is not a trace, or whose setup path never reaches
/// `compute_all_vals`.
pub fn make_draws(path: &Path) -> io::Result<Option<Vec<SetupDraw>>> {
    use crate::trace::{IMAGE_BASE, RVA_GAME_RANDOM};
    use std::io::Read;
    let mut r = BufReader::with_capacity(1 << 16, std::fs::File::open(path)?);
    let mut rec = [0u8; 32];
    if r.read_exact(&mut rec).is_err() || &rec[..4] != b"RONT" {
        return Ok(None);
    }
    let word = |b: &[u8; 32], i: usize| u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]);
    let base = word(&rec, 8);
    let norm = |va: u32| va.wrapping_sub(base).wrapping_add(IMAGE_BASE);
    let mut out = Vec::new();
    while r.read_exact(&mut rec).is_ok() {
        match word(&rec, 0) {
            2 => return Ok(None),
            1 | 3 if norm(word(&rec, 8)) == IMAGE_BASE + RVA_GAME_RANDOM => {
                let site = norm(word(&rec, 4));
                let val = (0x006b_0350..0x006b_07f0).contains(&site);
                if !val && out.last().is_some_and(|&(s, _)| COMPUTE_VAL.contains(&s)) {
                    return Ok(Some(out));
                }
                if !out.is_empty() || (0x0089_ca70..0x0089_cb50).contains(&site) {
                    out.push((site, word(&rec, 12)));
                }
            }
            _ => {}
        }
    }
    Ok(out
        .last()
        .is_some_and(|&(s, _)| COMPUTE_VAL.contains(&s))
        .then_some(out))
}

/// `World::compute_val@006b0350`'s extent: the values' draw site.
const COMPUTE_VAL: std::ops::Range<u32> = 0x006b_0350..0x006b_07f0;

/// A [`make_draws`] segment up to the values: through `place_resources`, to
/// the first `World::compute_val` draw.
pub fn resources_part(seg: &[SetupDraw]) -> &[SetupDraw] {
    let end = seg
        .iter()
        .position(|&(s, _)| COMPUTE_VAL.contains(&s))
        .unwrap_or(seg.len());
    &seg[..end]
}

/// **`LandData::make`** of each `Land` record (`docs/MAPGEN.md` §9):
/// `rules.xml`'s `LANDS`, in file order, each `MAKE` slot's `type` as its
/// `TypeIndex` — `Food` 0, `Timber` 1, `Wealth` 2, `Knowledge` 3, `Metal` 4,
/// `Oil` 5 (the `enums/TypeIndex.txt` values) and `none` `TYPE_NONE` (−1).
/// `None` for a land with other than four slots, or a type outside these.
pub fn land_makes(root: &Path) -> Option<sim::map_values::LandMakes> {
    let text = std::fs::read_to_string(root.join("Data/rules.xml")).ok()?;
    let doc = roxmltree::Document::parse(&text).ok()?;
    let lands = doc.descendants().find(|n| n.has_tag_name("LANDS"))?;
    lands
        .children()
        .filter(|n| n.has_tag_name("LAND"))
        .map(|land| {
            let slots: Vec<i32> = land
                .children()
                .filter(|n| n.has_tag_name("MAKE"))
                .map(|m| match m.attribute("type")?.trim() {
                    "Food" => Some(0),
                    "Timber" => Some(1),
                    "Wealth" => Some(2),
                    "Knowledge" => Some(3),
                    "Metal" => Some(4),
                    "Oil" => Some(5),
                    "none" => Some(-1),
                    _ => None,
                })
                .collect::<Option<_>>()?;
            slots.try_into().ok()
        })
        .collect()
}

/// The `place_all` prefix of a [`make_draws`] segment: up to its first
/// `TerrainGroups::nubify_forest` draw.
pub fn place_all_part(seg: &[SetupDraw]) -> &[SetupDraw] {
    let end = seg
        .iter()
        .position(|&(s, _)| (0x006a_93b0..0x006a_9a60).contains(&s))
        .unwrap_or(seg.len());
    &seg[..end]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::MAPGEN;
    use sim::combat::Rng;
    use sim::map_resources::place_resources;
    use sim::map_values::{adjust_for_blocking, compute_all_vals};
    use sim::mapgen::{
        Fractal, MapGrid, Regions, fertile_partitions, great_sahara_make_continents,
    };
    use sim::terrain_groups::{Draws, change_bases, nubify_forest, place_all};
    use std::sync::OnceLock;

    /// Every kept dump's [`StartGrid`], read once for the tests below: the
    /// archive's files whose preamble carries the setup trace. `None` on a
    /// machine without the archive.
    fn archive() -> Option<&'static [(String, StartGrid)]> {
        static ALL: OnceLock<Option<Vec<(String, StartGrid)>>> = OnceLock::new();
        ALL.get_or_init(|| {
            let dir = std::env::var("RON_GAMELOG_DIR").unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_default();
                format!("{home}/ron-data/AppData/Roaming/Microsoft Games/Rise of Nations/Logs")
            });
            let mut names: Vec<String> = std::fs::read_dir(&dir)
                .ok()?
                .filter_map(|e| e.ok()?.file_name().into_string().ok())
                .filter(|n| n.starts_with("gamelog") && n.ends_with(".txt"))
                .collect();
            names.sort();
            let paths: Vec<(String, String)> = names
                .into_iter()
                .filter_map(|n| Some((crate::testenv::dump(&n)?, n)))
                .collect();
            let chunk = paths.len().div_ceil(4).max(1);
            let mut out: Vec<(String, StartGrid)> = std::thread::scope(|s| {
                let hands: Vec<_> = paths
                    .chunks(chunk)
                    .map(|part| {
                        s.spawn(move || {
                            part.iter()
                                .filter_map(|(p, n)| {
                                    let head = start_head(Path::new(p)).ok()??;
                                    Some((n.clone(), start_grid(&head)))
                                })
                                .collect::<Vec<_>>()
                        })
                    })
                    .collect();
                hands
                    .into_iter()
                    .flat_map(|h| h.join().expect("a sweep thread"))
                    .collect()
            });
            out.sort_by(|a, b| a.0.cmp(&b.0));
            Some(out)
        })
        .as_deref()
    }

    /// The records from `setup.cpp` 1240 on: the checkpoints after which the
    /// setup path writes no cell (`docs/MAPGEN.md` §3).
    fn after_setup(g: &StartGrid) -> &[(String, i64, u32, [u32; 16])] {
        let from = g
            .checksums
            .iter()
            .position(|(f, l, _, _)| f == "setup.cpp" && *l == 1240)
            .unwrap_or(g.checksums.len());
        &g.checksums[from..]
    }

    /// **§3's identity, pinned** (item 3776): on every kept file whose
    /// setup trace reaches `setup.cpp` 1240 and whose head holds a start
    /// grid, the packed grid's Adler-32 is part 5 of every record from 1240
    /// on. A file that parts is named with its cell count and both sums;
    /// the file count is a floor at [`MAPGEN`]'s.
    #[test]
    fn the_cell_part_is_the_packed_start_dump() {
        let Some(all) = archive() else {
            eprintln!("skipped: no kept dumps on this machine");
            return;
        };
        let mut both = 0;
        let mut parted = Vec::new();
        for (name, g) in all {
            let (Some(cells), false) = (&g.cells, after_setup(g).is_empty()) else {
                continue;
            };
            assert_eq!(g.unnamed, 0, "{name}: a cell names no `land_key` terrain");
            both += 1;
            let ours = cell_part(cells);
            if let Some((f, l, _, p)) = after_setup(g).iter().find(|(_, _, _, p)| p[5] != ours) {
                parted.push(format!(
                    "{name}: {} cells, ours {ours} against {} at {f} {l}",
                    cells.len(),
                    p[5]
                ));
            }
        }
        assert!(
            parted.is_empty(),
            "{} of {both} files part:\n{}",
            parted.len(),
            parted.join("\n")
        );
        // A floor, not a count: every capture a lane files moves `both`,
        // and none of them should turn this red (parked 3776b).
        assert!(
            both >= MAPGEN.checksum_of && MAPGEN.checksum == MAPGEN.checksum_of,
            "{both} files carry both, under the pinned {} of {}: a kept dump \
             went missing, or the pin moved",
            MAPGEN.checksum,
            MAPGEN.checksum_of
        );
    }

    /// **Great Sahara's mask at `map.cpp` 7807** (`docs/MAPGEN.md` §5, §15
    /// step 1): from the seed the trace prints at 7805, the port's four draws
    /// land on 7807's seed, its start ring on part 2 and its grid on part 5,
    /// on every Great Sahara file that carries both checkpoints. The files
    /// reduce to the generator runs §2 lists — 2 and 3 nations on one seed.
    #[test]
    fn great_sahara_s_mask_is_the_traced_7807() {
        let Some(all) = archive() else {
            eprintln!("skipped: no kept dumps on this machine");
            return;
        };
        let mut files = 0;
        let mut runs = std::collections::BTreeSet::new();
        for (name, g) in all.iter().filter(|(_, g)| g.style == 7) {
            let (Some(before), Some(after)) = (g.at("map.cpp", 7805), g.at("map.cpp", 7807)) else {
                continue;
            };
            assert!(g.xs > 0 && g.nations > 0, "{name}: no grid size or nations");
            let mut grid = MapGrid::new(g.xs, g.ys);
            let mut rng = Rng::new(before.2);
            great_sahara_make_continents(&mut grid, g.nations, &mut rng);
            let ours = (rng.seed, grid.start_part(), grid.cell_part());
            let theirs = (after.2, after.3[2], after.3[5]);
            assert_eq!(
                ours, theirs,
                "{name}: (seed, part 2, part 5) at 7807, ours against theirs; \
                 {} nations on {}×{}, starts {:?} {:?}",
                g.nations, g.xs, g.ys, grid.starts[0].list, grid.starts[1].list
            );
            files += 1;
            runs.insert((before.2, g.nations, g.xs));
        }
        eprintln!("7807: {files} Great Sahara files, generator runs {runs:?}");
        let pinned = MAPGEN
            .styles
            .iter()
            .find(|s| s.map == "GreatSahara")
            .expect("MAPGEN's GreatSahara row");
        assert_eq!(
            pinned.ported, GREAT_SAHARA_CHECKED,
            "a Great Sahara phase was ported or lost: re-pin `floors::MAPGEN` \
             and the handoff's `Mapgen:` line"
        );
        assert!(
            files > 0 && runs.len() >= 2,
            "the 7807 check reached {files} files and {} generator runs",
            runs.len()
        );
    }

    /// **Great Sahara's zero-draw passes, 7835 to 7854** (`docs/MAPGEN.md`
    /// §6, §15 step 2): from 7807's grid, `Map::make`'s next four
    /// checkpoints, each against every Great Sahara file that carries them,
    /// in order, so the first that parts names its pass — 7835 after
    /// `Regions::clear_all`, `find_all` and `fix_diag_land`; 7843 after
    /// `make_coastlines`; 7846 after the region pair again; 7854 after
    /// `fill_fertile` over `Fractal::init` on the world seed and the
    /// tileset's partitions, read from the install. No draw: the seed must
    /// hold at 7807's through all four.
    #[test]
    fn great_sahara_s_zero_draw_passes_are_the_traced_7835_to_7854() {
        let Some(all) = archive() else {
            eprintln!("skipped: no kept dumps on this machine");
            return;
        };
        let Some(root) = crate::testenv::install_root() else {
            eprintln!("skipped: no install on this machine");
            return;
        };
        let fert = fertility(Path::new(&root), "SAHARA.xml").expect("SAHARA.xml's one tileset");
        let partitions = fertile_partitions(&fert.baseland);
        let mut files = 0;
        for (name, g) in all.iter().filter(|(_, g)| g.style == 7) {
            let Some(before) = g.at("map.cpp", 7805) else {
                continue;
            };
            let marks: Vec<_> = [7807, 7835, 7843, 7846, 7854]
                .iter()
                .map(|&l| g.at("map.cpp", l))
                .collect();
            if marks.iter().any(Option::is_none) {
                continue;
            }
            let mut grid = MapGrid::new(g.xs, g.ys);
            let mut rng = Rng::new(before.2);
            great_sahara_make_continents(&mut grid, g.nations, &mut rng);
            let mut regions = Regions::default();
            let check = |grid: &MapGrid, line: i64, pass: &str| {
                let (_, _, seed, p) = g.at("map.cpp", line).expect("checked above");
                assert_eq!(
                    (rng.seed, grid.start_part(), grid.cell_part()),
                    (*seed, p[2], p[5]),
                    "{name}: (seed, part 2, part 5) at {line}, after {pass}, ours against theirs"
                );
            };
            regions.clear_all(&mut grid);
            regions.find_all(&mut grid);
            grid.fix_diag_land();
            check(&grid, 7835, "the regions and fix_diag_land");
            grid.make_coastlines();
            check(&grid, 7843, "make_coastlines");
            regions.clear_all(&mut grid);
            regions.find_all(&mut grid);
            check(&grid, 7846, "the regions again");
            let fractal = Fractal::init(g.xs, g.ys, fert.clump_factor, 0, g.seed as u32);
            grid.fill_fertile(&fractal, &partitions);
            check(&grid, 7854, "fill_fertile");
            files += 1;
        }
        eprintln!("7835..7854: {files} Great Sahara files, tileset {fert:?}");
        assert!(files > 0, "the 7854 check reached no file");
    }

    /// **`find_all`'s labels on every start grid** (`docs/MAPGEN.md` §6):
    /// a start dump's `region` is the last region pair's (7927 → 7937) on
    /// every cell but the half-land, whose `region` `Terrain::mark_halfwater`
    /// writes after it. Relabelling the dumped cells with
    /// [`Regions::find_all`] gives every cell back whole — `region2`
    /// included — on every start grid but [`HALF_LAND_REORDERS`]'s, where a
    /// half-land row that was open water at 7937 puts an island ahead of
    /// another in the scan and swaps land regions 1 and 2. The dump cannot
    /// say which half-land cells were land at 7937, so those two are named,
    /// and any other file that parts fails.
    #[test]
    fn find_all_relabels_every_start_grid() {
        let Some(all) = archive() else {
            eprintln!("skipped: no kept dumps on this machine");
            return;
        };
        let mut checked = 0;
        let mut parted = Vec::new();
        let mut styles = std::collections::BTreeMap::new();
        for (name, g) in all {
            let Some(cells) = &g.cells else { continue };
            if g.xs * g.ys != cells.len() as i32 {
                continue;
            }
            let mut grid = MapGrid::new(g.xs, g.ys);
            grid.cells = cells.clone();
            let mut regions = Regions::default();
            regions.clear_all(&mut grid);
            regions.find_all(&mut grid);
            checked += 1;
            *styles.entry(g.style).or_insert(0) += 1;
            let wrong: Vec<_> = (0..cells.len())
                .filter(|&i| grid.cells[i] != cells[i])
                .collect();
            if let Some(&i) = wrong.first() {
                let (x, y) = (i as i32 % g.xs, i as i32 / g.xs);
                parted.push((
                    name.as_str(),
                    format!(
                        "{name} (style {}): {} cells, first ({x}, {y}) ours {:?} theirs {:?}",
                        g.style,
                        wrong.len(),
                        (grid.cells[i].region, grid.cells[i].region2),
                        (cells[i].region, cells[i].region2)
                    ),
                ));
            }
        }
        eprintln!("relabelled {checked} start grids, by style {styles:?}");
        let unexplained: Vec<_> = parted
            .iter()
            .filter(|(n, _)| !HALF_LAND_REORDERS.contains(n))
            .map(|(_, why)| why.as_str())
            .collect();
        assert!(
            unexplained.is_empty(),
            "{} of {checked} part:\n{}",
            unexplained.len(),
            unexplained.join("\n")
        );
        assert!(
            checked >= MAPGEN.checksum_of,
            "{checked} start grids relabelled, under the pinned {}",
            MAPGEN.checksum_of
        );
    }

    /// **Great Sahara's groups as `init_terrain_data` parses them** for a
    /// 60×60 map (`docs/MAPGEN.md` §7): every `SCALE` count is `(35 + 60v) /
    /// 70`, mountain sizes are unscaled, and the patterns are `continent`,
    /// `player` and `world` — reader A's table, row by row.
    #[test]
    fn great_sahara_s_groups_parse_as_read() {
        let Some(root) = crate::testenv::install_root() else {
            eprintln!("skipped: no install on this machine");
            return;
        };
        let root = Path::new(&root);
        assert_eq!(standard_map_size(root), Some(70));
        let groups = terrain_groups(root, "SAHARA.xml", 60, 60, 1).expect("SAHARA.xml's groups");
        // kind, clumps, pattern, size, start min/max, forest, mount, rock,
        // coast, cent min/max, edge max, oil.
        type Row = (
            i32,
            (i32, i32),
            i32,
            (i32, i32),
            (i32, i32),
            [i32; 4],
            (i32, i32),
            i32,
            (i32, i32),
        );
        let rows: [Row; 11] = [
            (
                4,
                (2, 2),
                1,
                (14, 21),
                (8, 0),
                [3, 0, 1, 1],
                (0, 4),
                0,
                (0, 0),
            ),
            (
                4,
                (1, 2),
                0,
                (3, 4),
                (2, 4),
                [1, 2, 1, 1],
                (0, 0),
                0,
                (0, 0),
            ),
            (
                4,
                (2, 2),
                0,
                (5, 7),
                (8, 14),
                [2, 1, 1, 1],
                (0, 0),
                0,
                (0, 0),
            ),
            (
                4,
                (3, 3),
                2,
                (5, 7),
                (15, 0),
                [3, 1, 1, 1],
                (10, 0),
                0,
                (0, 0),
            ),
            (
                4,
                (15, 21),
                2,
                (1, 2),
                (15, 0),
                [3, 1, 1, 1],
                (10, 0),
                13,
                (0, 0),
            ),
            (
                5,
                (1, 1),
                0,
                (2, 2),
                (6, 7),
                [0, 4, 0, 1],
                (0, 0),
                0,
                (0, 0),
            ),
            (
                5,
                (1, 1),
                0,
                (2, 2),
                (10, 16),
                [0, 4, 0, 1],
                (0, 0),
                0,
                (0, 0),
            ),
            (
                5,
                (3, 4),
                1,
                (2, 2),
                (6, 0),
                [3, 4, 1, 1],
                (0, 0),
                0,
                (0, 0),
            ),
            (
                5,
                (3, 3),
                1,
                (3, 3),
                (10, 0),
                [3, 4, 1, 1],
                (0, 9),
                0,
                (0, 0),
            ),
            (
                6,
                (1, 2),
                0,
                (3, 3),
                (3, 7),
                [1, 1, 10, 1],
                (0, 0),
                0,
                (1, 2),
            ),
            (
                6,
                (3, 5),
                0,
                (3, 5),
                (8, 24),
                [1, 1, 10, 1],
                (0, 0),
                0,
                (1, 2),
            ),
        ];
        assert_eq!(groups.len(), rows.len());
        for (i, (g, r)) in groups.iter().zip(rows).enumerate() {
            let ours: Row = (
                g.kind,
                (g.min_clumps, g.max_clumps),
                g.pattern,
                (g.min_size, g.max_size),
                (g.start_min, g.start_max),
                [g.forest_space, g.mount_space, g.rock_space, g.coast_space],
                (g.cent_min, g.cent_max),
                g.edge_max,
                (g.min_oil, g.max_oil),
            );
            assert_eq!(ours, r, "group {i}");
            assert_eq!(
                (g.chance, g.grouping, g.edge_min, g.corner_min, g.corner_max),
                (100, 0, 0, 0, 0)
            );
        }
        let fert = fertility(root, "SAHARA.xml").expect("one tileset");
        let ts = tileset_groups(root, &fert.tileset).expect("GS_Desert");
        assert_eq!(
            (
                ts.bush_clump_prob,
                ts.mnt_rock_clump_prob,
                ts.mnt_fringe_tree_prob
            ),
            (60, 50, 40)
        );
        let inputs = place_all_inputs(root, "SAHARA.xml", 60, 60, 7).expect("the inputs");
        assert_eq!(inputs.templates.len(), 16);
        assert!(inputs.templates.iter().all(|t| !t.tiles.is_empty()));
    }

    /// The kept dumps' directory, as [`archive`] reads it.
    fn gamelog_dir() -> String {
        std::env::var("RON_GAMELOG_DIR").unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_default();
            format!("{home}/ron-data/AppData/Roaming/Microsoft Games/Rise of Nations/Logs")
        })
    }

    /// The Great Sahara traces whose `place_all` segment the stream check
    /// walks: a floor, as [`MAPGEN`]'s file counts are.
    const PLACE_ALL_TRACES: usize = 280;

    /// One Great Sahara generator run, as the archive's files key it: the
    /// seed at 7805, the nations and the size, with the world seed.
    type GsRun = (u32, i32, i32, i32, i32);

    /// Each Great Sahara run's grid and stream as they stand at 7854 — 3798's
    /// chain, which its own test pins.
    fn great_sahara_at_7854(run: GsRun, fert: &Fertility) -> (MapGrid, Rng) {
        let (seed, nations, xs, ys, world) = run;
        let mut grid = MapGrid::new(xs, ys);
        let mut rng = Rng::new(seed);
        great_sahara_make_continents(&mut grid, nations, &mut rng);
        let mut regions = Regions::default();
        regions.clear_all(&mut grid);
        regions.find_all(&mut grid);
        grid.fix_diag_land();
        grid.make_coastlines();
        regions.clear_all(&mut grid);
        regions.find_all(&mut grid);
        let fractal = Fractal::init(xs, ys, fert.clump_factor, 0, world as u32);
        grid.fill_fertile(&fractal, &fertile_partitions(&fert.baseland));
        (grid, rng)
    }

    /// Every kept draw-site trace's `Map::make` segment from `place_all` on
    /// ([`make_draws`]), read once. `None` without the archive.
    fn make_traces() -> Option<&'static [(String, Vec<SetupDraw>)]> {
        type Segments = Vec<(String, Vec<SetupDraw>)>;
        static ALL: OnceLock<Option<Segments>> = OnceLock::new();
        ALL.get_or_init(|| {
            let mut names: Vec<String> = std::fs::read_dir(gamelog_dir())
                .ok()?
                .filter_map(|e| e.ok()?.file_name().into_string().ok())
                .filter(|n| n.starts_with("rontrace-") && n.ends_with(".log"))
                .collect();
            names.sort();
            let paths: Vec<(String, String)> = names
                .into_iter()
                .filter_map(|n| Some((crate::testenv::dump(&n)?, n)))
                .collect();
            let chunk = paths.len().div_ceil(4).max(1);
            let mut out: Vec<(String, Vec<SetupDraw>)> = std::thread::scope(|s| {
                let hands: Vec<_> = paths
                    .chunks(chunk)
                    .map(|part| {
                        s.spawn(move || {
                            part.iter()
                                .filter_map(|(p, n)| {
                                    Some((n.clone(), make_draws(Path::new(p)).ok()??))
                                })
                                .collect::<Vec<_>>()
                        })
                    })
                    .collect();
                hands
                    .into_iter()
                    .flat_map(|h| h.join().expect("a sweep thread"))
                    .collect()
            });
            out.sort_by(|a, b| a.0.cmp(&b.0));
            Some(out)
        })
        .as_deref()
    }

    /// The first index where two draw streams part, by site or by word, and
    /// the shorter length when one is a prefix of the other.
    fn first_parting(ours: &[SetupDraw], theirs: &[SetupDraw]) -> Option<usize> {
        let n = ours.len().min(theirs.len());
        (0..n)
            .find(|&i| ours[i] != theirs[i])
            .or((ours.len() != theirs.len()).then_some(n))
    }

    /// The archive's Great Sahara generator runs and the files of each.
    fn great_sahara_runs(
        all: &[(String, StartGrid)],
    ) -> std::collections::BTreeMap<GsRun, Vec<&StartGrid>> {
        let mut runs: std::collections::BTreeMap<GsRun, Vec<&StartGrid>> = Default::default();
        for (_, g) in all.iter().filter(|(_, g)| g.style == 7) {
            if let Some(before) = g.at("map.cpp", 7805) {
                runs.entry((before.2, g.nations, g.xs, g.ys, g.seed))
                    .or_default()
                    .push(g);
            }
        }
        runs
    }

    /// The traces that are Great Sahara's: a gamelog of the trace's run names
    /// the map. Other styles (Himalayas, the click-free Amazon) enter
    /// `place_all` on the same seed and part at the ladder, by their own
    /// groups.
    fn sahara_traces(
        traces: &'static [(String, Vec<SetupDraw>)],
    ) -> Vec<&'static (String, Vec<SetupDraw>)> {
        let gamelogs: Vec<String> = std::fs::read_dir(gamelog_dir())
            .map(|r| {
                r.filter_map(|e| e.ok()?.file_name().into_string().ok())
                    .collect()
            })
            .unwrap_or_default();
        let sahara = |trace: &str| {
            let run = trace
                .trim_start_matches("rontrace-")
                .trim_end_matches(".log");
            gamelogs.iter().any(|g| {
                g.strip_prefix("gamelog-")
                    .and_then(|g| g.strip_prefix(run))
                    .is_some_and(|rest| rest.starts_with('-') && rest.contains("greatsahara"))
            })
        };
        traces.iter().filter(|(n, _)| sahara(n)).collect()
    }

    /// Each Great Sahara trace's phase (`cut` of its segment) against the
    /// ported streams that open on its seed: the trace passes when it is one
    /// run's stream whole. Returns the nation counts met and how often, and
    /// each parting trace named with its first parted draw — site and word,
    /// both sides.
    fn stream_partings(
        traces: &[&(String, Vec<SetupDraw>)],
        ported: &[(GsRun, Vec<SetupDraw>)],
        cut: fn(&[SetupDraw]) -> &[SetupDraw],
    ) -> (std::collections::BTreeMap<i32, usize>, Vec<String>) {
        let mut met = std::collections::BTreeMap::new();
        let mut parted = Vec::new();
        for (name, seg) in traces.iter().map(|t| (&t.0, cut(&t.1))) {
            let opens: Vec<_> = ported
                .iter()
                .filter(|(_, log)| log.first().map(|x| x.1) == seg.first().map(|x| x.1))
                .collect();
            if opens.is_empty() {
                parted.push(format!(
                    "{name}: opens on {:?}, no run's 7854 seed",
                    seg.first()
                ));
                continue;
            }
            let partings: Vec<_> = opens
                .iter()
                .map(|(run, log)| (run, log, first_parting(log, seg)))
                .collect();
            if let Some((run, _, _)) = partings.iter().find(|(_, _, p)| p.is_none()) {
                *met.entry(run.1).or_insert(0) += 1;
                continue;
            }
            let (run, log, at) = partings
                .iter()
                .max_by_key(|(_, _, p)| *p)
                .map(|(r, l, p)| (r, l, p.expect("parted")))
                .expect("a candidate");
            let site = |v: &[SetupDraw]| v.get(at).map(|&(s, w)| format!("{s:#x} word {w:#x}"));
            parted.push(format!(
                "{name}: {} nations, draw {at} of ours {} / theirs {}: ours {:?}, theirs {:?}",
                run.1,
                log.len(),
                seg.len(),
                site(log),
                site(seg)
            ));
        }
        parted.sort();
        (met, parted)
    }

    /// The partings, a dozen named, as a test failure says them.
    fn say_partings(parted: &[String]) -> String {
        parted
            .iter()
            .take(12)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// One run's grid and stream through `place_all` (7859).
    fn great_sahara_at_7859(run: GsRun, fert: &Fertility, root: &Path) -> (MapGrid, Draws) {
        let (mut grid, rng) = great_sahara_at_7854(run, fert);
        let inputs = place_all_inputs(root, "SAHARA.xml", run.2, run.3, 7).expect("the inputs");
        let mut d = Draws::new(rng.seed);
        place_all(&mut grid, &mut d, &inputs);
        (grid, d)
    }

    /// The checkpoint's `(seed, parts 2, 5, 6)` on every file of `run`
    /// that carries `line`, against ours; the files checked.
    fn at_checkpoint(
        runs: &std::collections::BTreeMap<GsRun, Vec<&StartGrid>>,
        run: &GsRun,
        line: i64,
        grid: &MapGrid,
        d: &Draws,
    ) -> usize {
        let mut files = 0;
        for g in &runs[run] {
            let Some((_, _, seed, p)) = g.at("map.cpp", line) else {
                continue;
            };
            assert_eq!(
                (
                    d.rng.seed,
                    grid.start_part(),
                    grid.cell_part(),
                    grid.tile_part()
                ),
                (*seed, p[2], p[5], p[6]),
                "(seed, parts 2, 5, 6) at {line} on a {}-nation file, ours against theirs",
                run.1
            );
            files += 1;
        }
        files
    }

    /// **`place_all` on Great Sahara, draw for draw** (`docs/MAPGEN.md` §7,
    /// §15 step 3): from each generator run's 7854 grid and seed, the port's
    /// tagged stream against every kept trace whose `place_all` segment
    /// opens on that seed, site by site — the first parting names its draw
    /// site and so its sub-pass. The 2- and 3-player runs enter on one
    /// seed and one grid, so a trace passes when it is one run's stream
    /// whole; both runs must be met. Then seed and parts 2, 5 and 6 at
    /// 7859 on every Great Sahara file that carries it.
    #[test]
    fn great_sahara_s_place_all_is_the_traced_stream_and_7859() {
        let (Some(all), Some(traces)) = (archive(), make_traces()) else {
            eprintln!("skipped: no kept dumps on this machine");
            return;
        };
        let Some(root) = crate::testenv::install_root() else {
            eprintln!("skipped: no install on this machine");
            return;
        };
        let root = Path::new(&root);
        let fert = fertility(root, "SAHARA.xml").expect("SAHARA.xml's one tileset");
        let runs = great_sahara_runs(all);
        let ported: Vec<(GsRun, MapGrid, Draws)> = runs
            .keys()
            .map(|&run| {
                let (grid, d) = great_sahara_at_7859(run, &fert, root);
                (run, grid, d)
            })
            .collect();
        let sahara = sahara_traces(traces);
        let logs: Vec<_> = ported.iter().map(|(r, _, d)| (*r, d.log.clone())).collect();
        let (met, parted) = stream_partings(&sahara, &logs, place_all_part);
        assert!(
            parted.is_empty(),
            "{} Great Sahara traces part from the port's place_all (met whole: {met:?}):\n{}",
            parted.len(),
            say_partings(&parted)
        );
        assert!(
            met.len() >= 2 && sahara.len() >= PLACE_ALL_TRACES,
            "the place_all stream met {met:?} nation counts on {} Great Sahara \
             traces, under the pinned {PLACE_ALL_TRACES}; 2 and 3 are both kept",
            sahara.len()
        );
        let mut files = 0;
        for (run, grid, d) in &ported {
            // What the parts below are checking: every kind of write this
            // phase makes reaches the grid, oil included (the OIL goods
            // object `set_oil_at` makes walks no part; `goods` stays 0).
            let count = |bit: u16| grid.cells.iter().filter(|c| c.flags & bit != 0).count();
            let placed = [0x20, 0x10, 0x8, 0x800].map(count);
            eprintln!(
                "{} nations: forest, mountain, rock, oil cells {placed:?}",
                run.1
            );
            assert!(
                placed.iter().all(|&n| n > 0),
                "a kind placed nothing: {placed:?}"
            );
            assert!(grid.cells.iter().all(|c| c.goods == 0));
            files += at_checkpoint(&runs, run, 7859, grid, d);
        }
        eprintln!("place_all: traces met whole {met:?}; 7859 on {files} files");
        assert!(files > 0, "the 7859 check reached no file");
    }

    /// The groups' phase of a [`make_draws`] segment: `place_all` and its
    /// tail, to the first draw outside `TerrainGroups`, `TerrainGroup` and
    /// `Mountains` — `Map::place_resources`' first.
    fn groups_part(seg: &[SetupDraw]) -> &[SetupDraw] {
        let end = seg
            .iter()
            .position(|&(s, _)| {
                !(0x006a_0000..0x006b_0000).contains(&s) && !(0x0089_0000..0x008a_0000).contains(&s)
            })
            .unwrap_or(seg.len());
        &seg[..end]
    }

    /// **Step 3's tail on Great Sahara** (`docs/MAPGEN.md` §7, §15 step 3,
    /// item 3805): `check_player_forest` (7861), `nubify_forest` (7865) and
    /// the base passes with `fix_transitions` (7870), after the port's
    /// `place_all` — the stream draw for draw against every Great Sahara
    /// trace through the last groups draw, and seed and parts 2, 5 and 6 at
    /// each of the three checkpoints on every file that carries it.
    #[test]
    fn great_sahara_s_group_tail_is_the_traced_stream_and_7861_to_7870() {
        let (Some(all), Some(traces)) = (archive(), make_traces()) else {
            eprintln!("skipped: no kept dumps on this machine");
            return;
        };
        let Some(root) = crate::testenv::install_root() else {
            eprintln!("skipped: no install on this machine");
            return;
        };
        let root = Path::new(&root);
        let fert = fertility(root, "SAHARA.xml").expect("SAHARA.xml's one tileset");
        let ts = tileset_groups(root, &fert.tileset).expect("GS_Desert");
        assert_eq!(
            (
                ts.forest_base,
                ts.forest_growth_prob,
                ts.mountain_base,
                ts.mountain_growth_prob,
                ts.coast_base,
                ts.coast_growth_prob
            ),
            (3, 30, 2, 30, 3, 30),
            "GS_Desert's bases and growth chances, as tilesets.xml writes them"
        );
        let runs = great_sahara_runs(all);
        let mut logs = Vec::new();
        let mut files = [0; 3];
        for &run in runs.keys() {
            let (mut grid, mut d) = great_sahara_at_7859(run, &fert, root);
            files[0] += at_checkpoint(&runs, &run, 7861, &grid, &d);
            nubify_forest(&mut grid, &mut d);
            files[1] += at_checkpoint(&runs, &run, 7865, &grid, &d);
            change_bases(&mut grid, &mut d, &ts);
            files[2] += at_checkpoint(&runs, &run, 7870, &grid, &d);
            logs.push((run, d.log));
        }
        let sahara = sahara_traces(traces);
        let (met, parted) = stream_partings(&sahara, &logs, groups_part);
        assert!(
            parted.is_empty(),
            "{} Great Sahara traces part from the port's groups and tail (met whole: {met:?}):\n{}",
            parted.len(),
            say_partings(&parted)
        );
        assert!(
            met.len() >= 2 && sahara.len() >= PLACE_ALL_TRACES,
            "the groups' stream met {met:?} nation counts on {} traces",
            sahara.len()
        );
        eprintln!("group tail: traces met whole {met:?}; 7861, 7865, 7870 on {files:?} files");
        assert!(
            files.iter().all(|&n| n > 0),
            "a checkpoint reached no file: {files:?}"
        );
    }

    /// **Resources on Great Sahara** (`docs/MAPGEN.md` §8, §15 step 4, item
    /// 3805): from each run's 7870 grid (the groups and their tail), nothing
    /// moves through 7877 and 7880, then `Map::place_resources` — the whole
    /// stream from `place_all` to `compute_all_vals`, draw for draw against
    /// every Great Sahara trace, and seed and parts 2, 5 and 6 at 7927 on
    /// every file that carries it.
    #[test]
    fn great_sahara_s_resources_are_the_traced_stream_and_7927() {
        let (Some(all), Some(traces)) = (archive(), make_traces()) else {
            eprintln!("skipped: no kept dumps on this machine");
            return;
        };
        let Some(root) = crate::testenv::install_root() else {
            eprintln!("skipped: no install on this machine");
            return;
        };
        let root = Path::new(&root);
        let fert = fertility(root, "SAHARA.xml").expect("SAHARA.xml's one tileset");
        let ts = tileset_groups(root, &fert.tileset).expect("GS_Desert");
        let runs = great_sahara_runs(all);
        let mut logs = Vec::new();
        let mut files = [0; 3];
        let mut placed = Vec::new();
        for &run in runs.keys() {
            let (mut grid, mut d) = great_sahara_at_7859(run, &fert, root);
            nubify_forest(&mut grid, &mut d);
            change_bases(&mut grid, &mut d, &ts);
            files[0] += at_checkpoint(&runs, &run, 7877, &grid, &d);
            files[1] += at_checkpoint(&runs, &run, 7880, &grid, &d);
            let inputs =
                resource_inputs(root, "SAHARA.xml", run.2, run.3, run.1).expect("the rows");
            let n = place_resources(&mut grid, &mut d, &inputs, run.1);
            placed.push((run.1, n));
            files[2] += at_checkpoint(&runs, &run, 7927, &grid, &d);
            logs.push((run, d.log));
        }
        let sahara = sahara_traces(traces);
        let (met, parted) = stream_partings(&sahara, &logs, resources_part);
        assert!(
            parted.is_empty(),
            "{} Great Sahara traces part from the port's resources (met whole: {met:?}):\n{}",
            parted.len(),
            say_partings(&parted)
        );
        assert!(
            met.len() >= 2 && sahara.len() >= PLACE_ALL_TRACES,
            "the resources' stream met {met:?} nation counts on {} traces",
            sahara.len()
        );
        eprintln!(
            "resources: placed {placed:?}; traces met whole {met:?}; 7877, 7880, 7927 on {files:?} files"
        );
        assert!(
            files.iter().all(|&n| n > 0),
            "a checkpoint reached no file: {files:?}"
        );
    }

    /// One run's grid and stream through `place_resources` (7927), the chain
    /// the tests above pin.
    fn great_sahara_at_7927(
        run: GsRun,
        fert: &Fertility,
        ts: &TilesetGroups,
        root: &Path,
    ) -> (MapGrid, Draws) {
        let (mut grid, mut d) = great_sahara_at_7859(run, fert, root);
        nubify_forest(&mut grid, &mut d);
        change_bases(&mut grid, &mut d, ts);
        let inputs = resource_inputs(root, "SAHARA.xml", run.2, run.3, run.1).expect("the rows");
        place_resources(&mut grid, &mut d, &inputs, run.1);
        (grid, d)
    }

    /// The region records as the labelling leaves them: each label's cell
    /// count, and the sea counter at its highest label. `adjust_for_blocking`
    /// re-runs `find_all` on these; on Great Sahara, with no region
    /// released and every cell labelled, it writes nothing.
    fn regions_of(grid: &MapGrid) -> Regions {
        let mut r = Regions::default();
        for c in &grid.cells {
            r.count[c.region as usize] += 1;
            if c.region >= 0x40 {
                r.sea = r.sea.max(c.region);
            }
        }
        r
    }

    /// **Values on Great Sahara** (`docs/MAPGEN.md` §9, §15 step 5, item
    /// 3807): from each run's 7927 grid, `World::adjust_for_blocking` — seed
    /// and parts 2, 5 and 6 at 7937 — then `World::compute_all_vals`: the
    /// whole stream from `place_all` through the last value draw, draw for
    /// draw against every Great Sahara trace, one draw a cell, and seed and
    /// parts 2, 5 and 6 at 7945 on every file that carries it.
    #[test]
    fn great_sahara_s_values_are_the_traced_stream_and_7937_and_7945() {
        let (Some(all), Some(traces)) = (archive(), make_traces()) else {
            eprintln!("skipped: no kept dumps on this machine");
            return;
        };
        let Some(root) = crate::testenv::install_root() else {
            eprintln!("skipped: no install on this machine");
            return;
        };
        let root = Path::new(&root);
        let fert = fertility(root, "SAHARA.xml").expect("SAHARA.xml's one tileset");
        let ts = tileset_groups(root, &fert.tileset).expect("GS_Desert");
        let makes = land_makes(root).expect("rules.xml's LANDS");
        assert_eq!(
            makes[..2],
            [[3, 0, -1, -1], [-1; 4]],
            "Land makes Knowledge and Food, Sandy nothing"
        );
        let runs = great_sahara_runs(all);
        let mut logs = Vec::new();
        let mut files = [0; 2];
        for &run in runs.keys() {
            let (mut grid, mut d) = great_sahara_at_7927(run, &fert, &ts, root);
            let inputs = place_all_inputs(root, "SAHARA.xml", run.2, run.3, 7).expect("inputs");
            let mut regions = regions_of(&grid);
            let was = grid.clone();
            adjust_for_blocking(&mut grid, &mut regions, &inputs.templates);
            // What 7937 checks: every kind of write reaches the grid.
            let cells = |bit: u16| grid.cells.iter().filter(|c| c.flags & bit != 0).count();
            let tiles = |bit: u16| grid.tiles.iter().filter(|&&m| m & bit != 0).count();
            let changed = (0..grid.cells.len())
                .filter(|&i| grid.cells[i].flags & 0x10 != was.cells[i].flags & 0x10)
                .count();
            let wrote = [cells(0x200), tiles(0x8000), tiles(0x38), changed];
            eprintln!(
                "{} nations: 0x200 cells, 0x8000 and 0x38 tiles, dead spots {wrote:?}",
                run.1
            );
            assert!(
                wrote[..3].iter().all(|&n| n > 0),
                "a mark wrote nothing: {wrote:?}"
            );
            files[0] += at_checkpoint(&runs, &run, 7937, &grid, &d);
            let before = d.log.len();
            compute_all_vals(&mut grid, &mut d, &makes);
            assert_eq!(
                d.log.len() - before,
                grid.cells.iter().filter(|c| !c.is_ocean()).count(),
                "one draw a non-water cell"
            );
            let vals: std::collections::BTreeSet<u8> = grid.cells.iter().map(|c| c.val).collect();
            let goods: std::collections::BTreeSet<u8> =
                grid.cells.iter().map(|c| c.goods).collect();
            eprintln!(
                "{} nations: vals {}..={}, goods {goods:x?}",
                run.1,
                vals.first().unwrap(),
                vals.last().unwrap()
            );
            files[1] += at_checkpoint(&runs, &run, 7945, &grid, &d);
            logs.push((run, d.log));
        }
        let sahara = sahara_traces(traces);
        let (met, parted) = stream_partings(&sahara, &logs, |s| s);
        assert!(
            parted.is_empty(),
            "{} Great Sahara traces part from the port's values (met whole: {met:?}):\n{}",
            parted.len(),
            say_partings(&parted)
        );
        assert!(
            met.len() >= 2 && sahara.len() >= PLACE_ALL_TRACES,
            "the values' stream met {met:?} nation counts on {} traces",
            sahara.len()
        );
        eprintln!("values: traces met whole {met:?}; 7937, 7945 on {files:?} files");
        assert!(
            files.iter().all(|&n| n > 0),
            "a checkpoint reached no file: {files:?}"
        );
    }

    /// The start grids whose half-land reorders `find_all`'s scan: the fuzz
    /// lobby's East Indies (seed 424242), 106 cells of land regions 1 and 2
    /// swapped from (31, 2).
    const HALF_LAND_REORDERS: [&str; 2] = [
        "gamelog-fuzz-424242-early.txt",
        "gamelog-fuzz-424242-heights.txt",
    ];

    /// The §15 phases this module checks on Great Sahara: the mask (7807),
    /// the zero-draw passes (7835..7854), `place_all` with its tail (7859,
    /// 7861..7870) and the resources (7927).
    const GREAT_SAHARA_CHECKED: i64 = 4;
}
