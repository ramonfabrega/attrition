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
    /// `push_size` (`+0x2f8`) and `push_circles` (`+0x2fc`): the pushing
    /// unit's collision profile, as `UnitType::init` stores them.
    pub push_size: i32,
    pub push_circles: i32,
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
        field!(push_size, i32);
        field!(push_circles, i32);
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

/// One assertion of [`compare`]: what it checked, whether it held, and the
/// detail a reader wants either way (the counts, the first offenders).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Check {
    pub what: String,
    pub ok: bool,
    pub detail: String,
}

/// One line of [`compare`]'s report, in the order the CLI prints them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Line {
    /// A count or a summary, already indented for the CLI.
    Note(String),
    Check(Check),
}

/// [`compare`]'s report: the notes and the checks, interleaved.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Report {
    pub lines: Vec<Line>,
}

impl Report {
    fn note(&mut self, s: impl Into<String>) {
        self.lines.push(Line::Note(s.into()));
    }

    fn check(&mut self, what: impl Into<String>, ok: bool, detail: impl Into<String>) {
        self.lines.push(Line::Check(Check {
            what: what.into(),
            ok,
            detail: detail.into(),
        }));
    }

    /// Every check, in order.
    pub fn checks(&self) -> impl Iterator<Item = &Check> {
        self.lines.iter().filter_map(|l| match l {
            Line::Check(c) => Some(c),
            Line::Note(_) => None,
        })
    }

    /// The checks that did not hold.
    pub fn failures(&self) -> Vec<&Check> {
        self.checks().filter(|c| !c.ok).collect()
    }
}

/// The program's own loaded types and composed combat table, against the
/// loader's `Kind`s and the table it builds from them (`docs/COMBAT.md`
/// §15): the inputs every `Kind` is built from, the derived words no column
/// carries (`docs/DATALAYER.md`), the buildings' six derived bits, the
/// eleven per-tech AI weights, and the whole 493 × 493 table, all four
/// quadrants.
///
/// `verbose` lists every offender instead of the first few. The CLI runs
/// this under `--types`; the install-gated test below runs it against run3's
/// dump on every `cargo test`, so the check cannot be skipped by omission.
pub fn compare(loaded: &crate::load::Loaded, dump: &TypesDump, verbose: bool) -> Report {
    use sim::attrition::Domain;

    let mut r = Report::default();
    let units = dump.units();
    let shown = |bad: &[String], few: usize| -> String {
        let v: Vec<&str> = bad
            .iter()
            .take(if verbose { usize::MAX } else { few })
            .map(String::as_str)
            .collect();
        format!(
            "{} differ{}{}",
            bad.len(),
            if v.is_empty() { "" } else { ": " },
            v.join(", ")
        )
    };

    r.note(format!("  {:<24} {:>5}", "types", dump.types.len()));
    r.note(format!("  {:<24} {:>5}", "unit types", units.len()));
    r.note(format!(
        "  {:<24} {:>5}",
        "COMBATTABLE",
        if dump.combat.is_some() { "yes" } else { "no" }
    ));
    r.check(
        "the dump's unit types are the loader's, in TypeIndex order",
        units.len() == loaded.kinds.len()
            && units
                .iter()
                .enumerate()
                .all(|(i, t)| t.type_index == i as i32 + 0x32),
        format!("{} in the dump, {} loaded", units.len(), loaded.kinds.len()),
    );
    let n = units.len().min(loaded.kinds.len());
    let name = |i: usize| -> String {
        loaded
            .unit_names
            .get(i)
            .cloned()
            .unwrap_or_else(|| format!("#{i}"))
    };

    // ---- the inputs: every field `Kind` is built from ----
    let mut by_field: Vec<(&str, Vec<String>)> = Vec::new();
    {
        let mut masks = Vec::new();
        let mut age = Vec::new();
        let mut siege = Vec::new();
        let mut caravan = Vec::new();
        let mut domain = Vec::new();
        for (i, (d, k)) in units.iter().zip(&loaded.kinds).enumerate().take(n) {
            if d.obj_masks != k.masks {
                masks.push(format!("{} {:#x}≠{:#x}", name(i), d.obj_masks, k.masks));
            }
            // The dump's `age` is the stored field; `−1` (the gaia animals)
            // resolves through `get_age_slow` to 0.
            if d.age.max(0) != k.age {
                age.push(format!("{} {}≠{}", name(i), d.age, k.age));
            }
            if (d.unit_flags & 0x20000 != 0) != k.siege {
                siege.push(name(i));
            }
            if (d.unit_flags2 & 8 != 0) != k.caravan {
                caravan.push(name(i));
            }
            let dd = match d.domain {
                0 => Domain::Land,
                1 => Domain::Sea,
                _ => Domain::Air,
            };
            if dd != k.domain {
                domain.push(format!("{} {}≠{:?}", name(i), d.domain, k.domain));
            }
        }
        by_field.push(("obj_masks", masks));
        by_field.push(("age", age));
        by_field.push(("is_siege (unit_flags & 0x20000)", siege));
        by_field.push(("is_caravan (unit_flags2 & 8)", caravan));
        by_field.push(("domain", domain));
    }

    // ---- the derived words: `docs/DATALAYER.md` ----
    {
        let mut role = Vec::new();
        let mut uf = Vec::new();
        let mut uf2 = Vec::new();
        let mut cat = Vec::new();
        let mut carry = Vec::new();
        for (i, d) in units.iter().enumerate().take(n) {
            let c = loaded.unit_types[i].cols;
            if d.role != c.role {
                role.push(format!("{} {:#x}≠{:#x}", name(i), d.role, c.role));
            }
            if d.unit_flags != c.unit_flags {
                uf.push(format!(
                    "{} {:#x}≠{:#x}",
                    name(i),
                    d.unit_flags,
                    c.unit_flags
                ));
            }
            if d.unit_flags2 != c.unit_flags2 {
                uf2.push(format!(
                    "{} {:#x}≠{:#x}",
                    name(i),
                    d.unit_flags2,
                    c.unit_flags2
                ));
            }
            if d.cat != c.cat {
                cat.push(format!("{} {}≠{}", name(i), d.cat, c.cat));
            }
            if d.carry != c.carry {
                carry.push(format!("{} {}≠{}", name(i), d.carry, c.carry));
            }
        }
        // The name group's four disagreements: the two of them the simulation
        // carries (`docs/DATALAYER.md`).
        let mut armor = Vec::new();
        let mut splash = Vec::new();
        for (i, d) in units.iter().enumerate().take(n) {
            let p = &loaded.unit_types[i].combat;
            if d.armor != p.armor {
                armor.push(format!("{} {}≠{}", name(i), d.armor, p.armor));
            }
            if d.splash_percent != p.splash_percent {
                splash.push(format!(
                    "{} {}≠{}",
                    name(i),
                    d.splash_percent,
                    p.splash_percent
                ));
            }
        }
        // `push_size`/`push_circles` (`docs/COLLISION.md` §13): the size
        // is `BLOCK_RADIUS`'s when there is one circle.
        let mut push = Vec::new();
        for (i, d) in units.iter().enumerate().take(n) {
            let p = &loaded.unit_types[i].combat;
            if (d.push_size, d.push_circles) != (p.push_size, p.push_circles) {
                push.push(format!(
                    "{} {}/{}≠{}/{}",
                    name(i),
                    d.push_size,
                    d.push_circles,
                    p.push_size,
                    p.push_circles
                ));
            }
        }
        by_field.push(("push_size/push_circles", push));
        by_field.push(("armor (the name group)", armor));
        by_field.push(("splash_percent (the name group)", splash));
        by_field.push(("role (determine_roles)", role));
        by_field.push(("unit_flags", uf));
        by_field.push(("unit_flags2 (init_final_flags + the casters)", uf2));
        by_field.push(("cat", cat));
        by_field.push(("carry", carry));
    }
    for (what, bad) in &by_field {
        r.check(
            format!("every unit's {what} is the program's"),
            bad.is_empty(),
            shown(bad, 8),
        );
    }

    // ---- the building inputs ----
    let builds: Vec<&TypeRow> = {
        let mut v: Vec<_> = dump.types.iter().filter(|t| !t.unit).collect();
        v.sort_by_key(|t| t.type_index);
        v
    };
    r.check(
        "the dump's building types are the loader's, in TypeIndex order",
        builds.len() == loaded.build_kinds.len()
            && builds
                .iter()
                .enumerate()
                .all(|(i, t)| t.type_index == i as i32 + 0x19e),
        format!(
            "{} in the dump, {} loaded",
            builds.len(),
            loaded.build_kinds.len()
        ),
    );
    let bname = |i: usize| -> String {
        loaded
            .build_names
            .get(i)
            .cloned()
            .unwrap_or_else(|| format!("#{i}"))
    };
    {
        let mut masks = Vec::new();
        let mut age = Vec::new();
        let mut domain = Vec::new();
        for (i, (d, k)) in builds.iter().zip(&loaded.build_kinds).enumerate() {
            if d.obj_masks != k.masks {
                masks.push(format!("{} {:#x}≠{:#x}", bname(i), d.obj_masks, k.masks));
            }
            if d.age.max(0) != k.age {
                age.push(format!("{} {}≠{}", bname(i), d.age, k.age));
            }
            let dd = match d.domain {
                0 => Domain::Land,
                1 => Domain::Sea,
                _ => Domain::Air,
            };
            if dd != k.domain {
                domain.push(format!("{} {}≠{:?}", bname(i), d.domain, k.domain));
            }
        }
        let mut bflags = Vec::new();
        for (i, d) in builds.iter().enumerate() {
            let Some(bt) = loaded.build_types.get(i) else {
                continue;
            };
            if d.build_flags != bt.flags {
                bflags.push(format!("{} {:#x}≠{:#x}", bname(i), d.build_flags, bt.flags));
            }
        }
        for (what, bad) in [
            ("obj_masks", masks),
            ("age", age),
            ("domain", domain),
            ("build_flags (the six derived bits)", bflags),
        ] {
            r.check(
                format!("every building's {what} is the program's"),
                bad.is_empty(),
                shown(&bad, 8),
            );
        }
    }

    // ---- the eleven per-tech weights, `compute_ai_values` ----
    if dump.techs.is_empty() {
        r.note("  (no TECHTYPE blocks in this dump; the AI weights are unchecked)");
    } else {
        let mut techs: Vec<&TechRow> = dump.techs.iter().collect();
        techs.sort_by_key(|t| t.type_index);
        let mut bad: Vec<String> = Vec::new();
        for t in &techs {
            let rec = usize::try_from(t.type_index - 0x220).unwrap_or(usize::MAX);
            let Some(&id) = loaded.tech_tree.get(rec) else {
                continue;
            };
            let ours = loaded.tree.types[id].ai;
            if ours != t.ai {
                bad.push(format!(
                    "{} {:?}≠{:?}",
                    loaded.tech_names.get(rec).cloned().unwrap_or_default(),
                    t.ai,
                    ours
                ));
            }
        }
        r.check(
            format!(
                "every one of the {} techs' ai[11] is the program's",
                techs.len()
            ),
            bad.is_empty(),
            shown(&bad, 4).replace(", ", "; "),
        );
    }

    // ---- the output: the whole table, units then buildings ----
    if dump.combat.is_none() {
        r.note("  (no COMBATTABLE in this dump; the table is unchecked)");
        return r;
    }
    let nb = builds.len().min(loaded.build_kinds.len());
    let side = n + nb;
    let at = |i: usize| {
        if i < n {
            sim::combat::TypeRef::Unit(i)
        } else {
            sim::combat::TypeRef::Build(i - n)
        }
    };
    let ti = |i: usize| -> i32 {
        if i < n {
            i as i32 + 0x32
        } else {
            (i - n) as i32 + 0x19e
        }
    };
    let label = |i: usize| if i < n { name(i) } else { bname(i - n) };
    let mut mism = 0usize;
    let mut quadrant = [0usize; 4];
    let mut by_attacker = vec![0usize; side];
    let mut by_target = vec![0usize; side];
    let mut examples: Vec<String> = Vec::new();
    for a in 0..side {
        for b in 0..side {
            let want = dump.entry(ti(a), ti(b)).unwrap_or(100);
            let ours = loaded.table.pct_of(at(a), at(b));
            if i32::from(want) != ours {
                mism += 1;
                quadrant[usize::from(a >= n) * 2 + usize::from(b >= n)] += 1;
                by_attacker[a] += 1;
                by_target[b] += 1;
                if examples.len() < if verbose { 400 } else { 10 } {
                    examples.push(format!(
                        "{} → {}: {want} vs ours {ours}",
                        label(a),
                        label(b)
                    ));
                }
            }
        }
    }
    r.note(format!(
        "  combat table {side}×{side}: {mism} of {} cells differ (unit→unit {}, unit→building {}, building→unit {}, building→building {})",
        side * side,
        quadrant[0],
        quadrant[1],
        quadrant[2],
        quadrant[3]
    ));
    let top = |counts: &[usize], what: &str| -> Option<String> {
        let mut v: Vec<(usize, usize)> = counts
            .iter()
            .copied()
            .enumerate()
            .filter(|(_, c)| *c > 0)
            .collect();
        v.sort_by(|x, y| y.1.cmp(&x.1).then(x.0.cmp(&y.0)));
        (!v.is_empty()).then(|| {
            format!(
                "    worst {what}: {}",
                v.iter()
                    .take(8)
                    .map(|(i, c)| format!("{} ({c})", label(*i)))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
    };
    if let Some(s) = top(&by_attacker, "attackers") {
        r.note(s);
    }
    if let Some(s) = top(&by_target, "targets") {
        r.note(s);
    }
    for e in &examples {
        r.note(format!("    {e}"));
    }
    r.check(
        "the combat table is the program's, all four quadrants",
        mism == 0,
        format!("{mism} cells differ"),
    );
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Run3's type dump, on every `cargo test`** — the same comparison
    /// `rondata --types` runs, install-gated like the `SITES` pin in
    /// `diff.rs`, so the derived words and the combat table are a test
    /// rather than a flag nobody has to pass (`docs/QUEUE.md`, owed item 1
    /// of 2026-08-25). The dump's shape is asserted first — the 364 unit
    /// types, the 129 buildings, the techs, the table — so the test cannot
    /// pass on a dump that has nothing to check.
    #[test]
    fn run3_s_type_dump_is_the_program_s_on_every_check() {
        let Some(inst) = crate::testenv::install() else {
            eprintln!("skipping: no install (set RON_INSTALL)");
            return;
        };
        let Some(path) = crate::testenv::dump("gamelog-run3-fulldump-types.txt") else {
            eprintln!("skipping: no gamelog-run3-fulldump-types.txt (set RON_GAMELOG_DIR)");
            return;
        };
        let dump = read(&path).unwrap();
        assert_eq!(
            dump.units().len(),
            364,
            "run3's dump has the 364 unit types"
        );
        assert_eq!(
            dump.types.iter().filter(|t| !t.unit).count(),
            129,
            "run3's dump has the 129 building types"
        );
        assert_eq!(dump.techs.len(), 85, "run3's dump has the 85 tech types");
        assert!(dump.combat.is_some(), "run3's dump has the COMBATTABLE");

        let loaded = crate::load::load(&inst).unwrap();
        let report = compare(&loaded, &dump, true);
        // The twenty-one checks the CLI prints: the two orderings, the
        // thirteen unit fields, the four building fields, the techs, the
        // table.
        assert_eq!(
            report.checks().count(),
            21,
            "the CLI's twenty-one checks, no fewer"
        );
        let failed: Vec<String> = report
            .failures()
            .iter()
            .map(|c| format!("{}: {}", c.what, c.detail))
            .collect();
        assert!(failed.is_empty(), "{}", failed.join("\n"));
    }

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
