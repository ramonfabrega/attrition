//! The original's start-of-game *type* dump: every loaded `UnitType` and
//! `BuildType` by field, and the composed combat table.
//!
//! A `DUMP_ALL=1` run with `InitialDump=1` (`docs/ORACLE.md`, "The detail
//! level is the knob") writes, before the first frame, `Game::log_rules_data`'s
//! output: 1,820 `BEGIN UNITTYPE` blocks (the 364 unit types, five times —
//! once per loaded copy), 387 `BEGIN BUILDTYPE`, 255 `BEGIN TECHTYPE`, and a
//! `BEGIN COMBATTABLE` of 493 × 493 `final_balance_table[scan][scan2] n`
//! lines. Every field is the program's own loaded value — `obj_masks` as the
//! folded word, `age` as `ObjectTypeData::get_age` returns it, `from`/`graft`
//! as resolved `TypeIndex`es — so this is the oracle for the *inputs* of
//! `sim::balance::Kind` as well as for its output, the table.
//!
//! The file is 150 MB; [`crate::gamelog::Log`] would hold it whole, so this
//! reader streams it and keeps only what it is asked for.

use crate::Error;
use std::io::{BufRead, BufReader};

/// One loaded type, as the program logged it. Only the fields the combat table
/// is built from; `−1` where the program writes `−1` (no `from`, no `graft`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TypeRow {
    /// `UNITTYPE` or `BUILDTYPE`.
    pub unit: bool,
    /// `TypeIndex`: `0x32..0x19e` for a unit, `0x19e..0x21f` for a building.
    pub type_index: i32,
    pub obj_masks: u32,
    /// The stored `+0x278`, as logged: `get_age_slow`'s result for every
    /// type the program finalised, and `−1` for the twelve gaia animals,
    /// which `get_age` would resolve to 0 (no tech prerequisite).
    pub age: i32,
    pub from: i32,
    pub graft: i32,
    pub upgrade: i32,
    pub unit_flags: u32,
    pub unit_flags2: u32,
    pub domain: i32,
    pub cat: i32,
    /// `UnitTypeData::role` (`+0x2c8`) — `determine_roles`' word, and
    /// `BuildTypeData::build_flags` (`+0x2c0`) for a building row. Both are
    /// derived at load and neither is a column: `docs/DATALAYER.md`, "The
    /// derived words no column carries".
    pub role: u32,
    pub build_flags: u32,
    /// `CARRY`, and `ATTACK` — the two other columns `determine_roles` reads
    /// that the row did not previously carry.
    pub carry: i32,
    pub attack: i32,
    /// The stored `max_range` (`+0x1fc`), after `FLAGS k`.
    pub max_range: i32,
    /// Two columns the **name group** decides (`rondata::load`'s
    /// `name_group_leaders`): a variant record takes its leader's, and these
    /// two are where the shipped table disagrees with itself.
    pub armor: i32,
    pub splash_percent: i32,
}

/// One `BEGIN TECHTYPE` block: the eleven `ai[scan]` weights
/// `TechType::compute_ai_values` derives, and the tech's `WHERE`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TechRow {
    /// `TypeIndex`, `0x220..0x275`.
    pub type_index: i32,
    pub ai: [i16; 11],
    pub cat: i32,
    pub where_: i32,
}

/// What the dump holds that this crate checks against.
#[derive(Clone, Debug, Default)]
pub struct TypesDump {
    /// One row per `TypeIndex`, first occurrence kept (the five copies of a
    /// unit type agree on every field read here).
    pub types: Vec<TypeRow>,
    /// One row per tech `TypeIndex`, likewise.
    pub techs: Vec<TechRow>,
    /// `final_balance_table`, row-major `[attacker][target]` over `TypeIndex −
    /// 0x32`, when the dump has a `COMBATTABLE` block. 493 × 493.
    pub combat: Option<Vec<i16>>,
}

/// The table's side: `END_BUILDTYPES − BASE_UNITTYPES`.
pub const SIDE: usize = 493;

impl TypesDump {
    /// The row for a `TypeIndex`, if the dump has one.
    pub fn by_index(&self, type_index: i32) -> Option<&TypeRow> {
        self.types.iter().find(|t| t.type_index == type_index)
    }

    /// The unit rows in `TypeIndex` order (`0x32..`), i.e. unit-table order.
    pub fn units(&self) -> Vec<&TypeRow> {
        let mut v: Vec<&TypeRow> = self.types.iter().filter(|t| t.unit).collect();
        v.sort_by_key(|t| t.type_index);
        v
    }

    /// `final_balance_table[a][b]` for two `TypeIndex`es.
    pub fn entry(&self, a: i32, b: i32) -> Option<i16> {
        let t = self.combat.as_ref()?;
        let (a, b) = (
            usize::try_from(a - 0x32).ok()?,
            usize::try_from(b - 0x32).ok()?,
        );
        if a >= SIDE || b >= SIDE {
            return None;
        }
        t.get(a * SIDE + b).copied()
    }
}

/// Reads the dump at `path`, streaming.
pub fn read(path: &str) -> Result<TypesDump, Error> {
    let io = |source| Error::Io {
        path: path.to_string(),
        source,
    };
    let file = std::fs::File::open(path).map_err(io)?;
    let mut reader = BufReader::with_capacity(1 << 20, file);
    let mut out = TypesDump::default();
    let mut seen = std::collections::HashSet::new();
    let mut tseen = std::collections::HashSet::new();
    let mut cur: Option<TypeRow> = None;
    // A `TECHTYPE` block, which carries the eleven `ai[scan]` weights. Only
    // one of `cur` and `cur_tech` is ever open.
    let mut cur_tech: Option<TechRow> = None;
    let mut tech_ai_n = 0usize;
    // The indent of the open type block's own `BEGIN` line; a `BEGIN` at
    // that depth or shallower closes it, whatever its name, so the last
    // building's record cannot absorb the fields of whatever table follows.
    let mut cur_depth = 0usize;
    let mut in_combat = false;
    let mut combat: Vec<i16> = Vec::new();
    let mut raw = Vec::new();
    loop {
        raw.clear();
        let n = reader.read_until(b'\n', &mut raw).map_err(io)?;
        if n == 0 {
            break;
        }
        let line = String::from_utf8_lossy(&raw);
        let depth = line.len() - line.trim_start().len();
        let t = line.trim();
        if let Some(name) = t.strip_prefix("BEGIN ") {
            let name = name.trim();
            if in_combat {
                // The table block closes at the next BEGIN of any kind.
                in_combat = false;
                out.combat = Some(std::mem::take(&mut combat));
            }
            if depth <= cur_depth {
                if let Some(r) = cur.take()
                    && seen.insert(r.type_index)
                {
                    out.types.push(r);
                }
                if let Some(r) = cur_tech.take()
                    && tseen.insert(r.type_index)
                {
                    out.techs.push(r);
                }
            }
            match name {
                "UNITTYPE" | "BUILDTYPE" => {
                    if let Some(r) = cur.take()
                        && seen.insert(r.type_index)
                    {
                        out.types.push(r);
                    }
                    cur_depth = depth;
                    cur = Some(TypeRow {
                        unit: name == "UNITTYPE",
                        type_index: -1,
                        from: -1,
                        graft: -1,
                        upgrade: -1,
                        ..TypeRow::default()
                    });
                }
                "TECHTYPE" => {
                    if let Some(r) = cur_tech.take()
                        && tseen.insert(r.type_index)
                    {
                        out.techs.push(r);
                    }
                    cur_depth = depth;
                    tech_ai_n = 0;
                    cur_tech = Some(TechRow {
                        type_index: -1,
                        where_: -1,
                        ..TechRow::default()
                    });
                }
                "COMBATTABLE" => {
                    if let Some(r) = cur.take()
                        && seen.insert(r.type_index)
                    {
                        out.types.push(r);
                    }
                    in_combat = true;
                    combat = Vec::with_capacity(SIDE * SIDE);
                }
                // Nested blocks inside a type (OBJECTTYPE, TYPE, the STACKs)
                // do not carry the fields read here and pass through; any
                // other block at the type's depth closed it above.
                _ => {}
            }
            continue;
        }
        if in_combat {
            if let Some(v) = t.strip_prefix("final_balance_table[scan][scan2]")
                && let Ok(v) = v.trim().parse::<i16>()
            {
                combat.push(v);
            }
            continue;
        }
        let mut it = t.splitn(2, char::is_whitespace);
        let key = it.next().unwrap_or("");
        let val = it.next().unwrap_or("").trim();
        if let Some(r) = cur_tech.as_mut() {
            match key {
                // `type`, `cat` and `where` are in the nested `TYPE` block and
                // are written once; the eleven `ai[scan]` follow in order.
                "type" if r.type_index < 0 => r.type_index = val.parse().unwrap_or(-1),
                "cat" => r.cat = val.parse().unwrap_or(0),
                "where" if r.where_ < 0 => r.where_ = val.parse().unwrap_or(-1),
                "ai[scan]" => {
                    if let Some(slot) = r.ai.get_mut(tech_ai_n) {
                        *slot = val.parse().unwrap_or(0);
                        tech_ai_n += 1;
                    }
                }
                _ => {}
            }
            continue;
        }
        let Some(r) = cur.as_mut() else { continue };
        // Each of these keys is written once per type block; the repeated
        // keys in a block (`flags`, `length`, `size`, `increment`, `list`)
        // belong to the nested STACKs and are not read.
        // The words are logged as signed `int`s, so a mask with bit 31 set
        // (`6`, anti-air) prints negative; parse wide and cast.
        macro_rules! field {
            ($field:ident, $ty:ty) => {
                if key == stringify!($field) {
                    if let Ok(v) = val.parse::<i64>() {
                        r.$field = v as $ty;
                    }
                    continue;
                }
            };
        }
        if key == "type" && r.type_index < 0 {
            r.type_index = val.parse().unwrap_or(-1);
            continue;
        }
        field!(obj_masks, u32);
        field!(age, i32);
        field!(unit_flags, u32);
        field!(unit_flags2, u32);
        field!(domain, i32);
        field!(cat, i32);
        field!(from, i32);
        field!(graft, i32);
        field!(upgrade, i32);
        field!(role, u32);
        field!(build_flags, u32);
        field!(carry, i32);
        field!(attack, i32);
        field!(max_range, i32);
        field!(armor, i32);
        field!(splash_percent, i32);
    }
    if let Some(r) = cur.take()
        && seen.insert(r.type_index)
    {
        out.types.push(r);
    }
    if let Some(r) = cur_tech.take()
        && tseen.insert(r.type_index)
    {
        out.techs.push(r);
    }
    if in_combat {
        out.combat = Some(combat);
    }
    if let Some(c) = &out.combat
        && c.len() != SIDE * SIDE
    {
        return Err(Error::Missing {
            path: path.to_string(),
            what: format!("a {SIDE}×{SIDE} COMBATTABLE (found {} entries)", c.len()),
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_types_and_the_table_from_a_small_dump() {
        let text = "\
BEGIN GAME
 BEGIN UNITTYPE
  BEGIN OBJECTTYPE
   BEGIN TYPE
    type 50
    job_time 50
   obj_masks -2144862208
   age 0
   from -1
   graft -1
   BEGIN STACK
    flags 3
   unit_flags 6273
   unit_flags2 2
 BEGIN UNITTYPE
  BEGIN OBJECTTYPE
   BEGIN TYPE
    type 50
   obj_masks 1
 BEGIN BUILDTYPE
  BEGIN OBJECTTYPE
   BEGIN TYPE
    type 414
   obj_masks 7
   age 2
 BEGIN COMBATTABLE
";
        let mut text = text.to_string();
        for i in 0..(SIDE * SIDE) {
            text.push_str(&format!("  final_balance_table[scan][scan2] {}\n", i % 7));
        }
        text.push_str(" BEGIN TRIBE\n  age -1\n  domain -1\n BEGIN WORLD\n  seed 1\n");
        let dir = std::env::temp_dir().join("rondata-typesdump-test");
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("dump.txt");
        std::fs::write(&p, text).unwrap();
        let d = read(p.to_str().unwrap()).unwrap();
        assert_eq!(d.types.len(), 2);
        let u = d.by_index(50).unwrap();
        assert!(u.unit);
        assert_eq!(u.obj_masks, 0x8028_0000);
        assert_eq!(u.unit_flags, 6273);
        assert_eq!(u.unit_flags2, 2);
        assert_eq!(u.from, -1);
        let b = d.by_index(414).unwrap();
        assert!(!b.unit);
        assert_eq!(b.age, 2);
        assert_eq!(b.domain, 0);
        assert_eq!(d.entry(50, 50), Some(0));
        assert_eq!(d.entry(50, 51), Some(1));
        assert_eq!(d.entry(51, 50), Some((SIDE % 7) as i16));
        assert_eq!(d.entry(49, 50), None);
    }
}
