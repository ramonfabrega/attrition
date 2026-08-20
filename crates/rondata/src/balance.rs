//! The combat table's XML half — `data/balance.xml` — and the category names
//! the engine indexes it by. `docs/COMBAT.md` §5.
//!
//! `Balance::fill_tables` builds a 399×399 percentage table: one row and one
//! column per category, where the categories are the 352 unit types by their
//! table index, five lines, two object classes, eight ages and the 32
//! `obj_masks` letters. The names are built from `internal_strings.xml` and the
//! unit `<NAME>`s (spaces to underscores, apostrophes dropped), and that is the
//! order the shipped file's rows are in. A row or an attribute the file lacks
//! is 100.
//!
//! What this module does **not** do yet is the other half: the per-type
//! `Kind` a regeneration needs carries the type's age and its named lineages,
//! and both come from the tech tree's `from`/`graft`/prerequisite columns,
//! which `rondata` does not load. [`unit_kind`] fills what the unit record
//! alone gives — the masks, the siege flag, the domain — and says so.

use crate::{Error, Install, Table, parse, read};
use sim::balance::{self, Kind};
use sim::combat::mask;

/// The number of categories.
pub const CATEGORIES: usize = 399;
/// Where the fixed tail of the category list starts: `0x160`.
pub const FIRST_LINE: usize = 0x160;

/// The non-unit category names, in index order from `0x160`: five lines, two
/// object classes, eight ages, thirty-two flags.
pub fn tail_names() -> Vec<String> {
    let mut v: Vec<String> = [
        "SIEGE",
        "FORTS",
        "TOWERS",
        "CITIES",
        "OBSPOST",
        "BUILDINGS",
        "UNITS",
    ]
    .iter()
    .map(|s| (*s).to_string())
    .collect();
    for age in 0..8 {
        v.push(format!("AGE_{age}"));
    }
    const FLAGS: [&str; 32] = [
        "ARMORED",
        "BOMBARD",
        "CIVILIAN",
        "MUSKET_INF",
        "ELEPHANT",
        "FOOT",
        "GUN",
        "HEAVY_INF",
        "MODERN_INF",
        "CARRY_AIR",
        "FOOT_ARCHER",
        "LARGE",
        "MOUNTED",
        "NAVAL",
        "HORSE_ARCHER",
        "SPARSE",
        "LIGHT_INF",
        "ARCHERY",
        "SIEGE",
        "WAR_MACHINE",
        "ARMORPIERCE",
        "VEHICLE",
        "MELEE",
        "EXPLOSIVE",
        "HEAVY_CAV",
        "DETECT",
        "UNUSED",
        "MISSILE",
        "AIR",
        "LIGHT_CAV",
        "PIKE",
        "ANTI_AIR",
    ];
    for (i, f) in FLAGS.iter().enumerate() {
        // Letters A–Z, then 1–6: `'A' + i`, past `'Z'` shifted down by 0x2a.
        let c = if i < 26 {
            (b'A' + i as u8) as char
        } else {
            (b'A' + i as u8 - 0x2a) as char
        };
        v.push(format!("Flag_{c}_OBJMASK_{f}"));
    }
    v
}

/// The unit-row name for a unit record: its `<NAME>` with spaces turned to
/// underscores and apostrophes removed — `lookup_absolute_name` followed by
/// `String::replace(' ', '_')`.
pub fn unit_row_name(name: &str) -> String {
    name.replace('\'', "").replace(' ', "_")
}

/// All 399 category names in index order, from the unit table.
pub fn category_names(units: &Table) -> Vec<String> {
    let mut v: Vec<String> = (0..FIRST_LINE)
        .map(|i| {
            units
                .get(i)
                .and_then(|r| r.text("NAME"))
                .map_or_else(String::new, unit_row_name)
        })
        .collect();
    v.extend(tail_names());
    v
}

/// The shipped `balance.xml`, as read: each `<ENTRY name=…>` row with its
/// attributes in file order.
#[derive(Clone, Debug, Default)]
pub struct BalanceXml {
    pub rows: Vec<(String, Vec<(String, i32)>)>,
}

impl BalanceXml {
    /// The row names in file order.
    pub fn row_names(&self) -> Vec<&str> {
        self.rows.iter().map(|(n, _)| n.as_str()).collect()
    }

    /// The 399×399 table `fill_tables` builds from this file: `[row][col]`,
    /// 100 wherever the file is silent.
    pub fn table(&self, names: &[String]) -> Vec<i16> {
        let mut t = vec![100i16; CATEGORIES * CATEGORIES];
        for (r, rname) in names.iter().enumerate().take(CATEGORIES) {
            let Some((_, attrs)) = self.rows.iter().find(|(n, _)| n == rname) else {
                continue;
            };
            for (c, cname) in names.iter().enumerate().take(CATEGORIES) {
                if let Some((_, v)) = attrs.iter().find(|(a, _)| a == cname) {
                    t[r * CATEGORIES + c] = *v as i16;
                }
            }
        }
        t
    }
}

impl Install {
    /// Reads `data/balance.xml`.
    pub fn balance(&self) -> Result<BalanceXml, Error> {
        let path = self.data("balance.xml");
        let text = read(&path)?;
        let doc = parse(&path, &text)?;
        let root = doc.root_element();
        let table = root
            .children()
            .find(|n| n.is_element() && n.tag_name().name() == "TABLE")
            .ok_or_else(|| Error::Missing {
                path: path.display().to_string(),
                what: "TABLE".to_string(),
            })?;
        let rows = table
            .children()
            .filter(|n| n.is_element() && n.tag_name().name() == "ENTRY")
            .map(|n| {
                let name = n.attribute("name").unwrap_or("").to_string();
                let attrs = n
                    .attributes()
                    .filter(|a| a.name() != "name")
                    .map(|a| (a.name().to_string(), leading_int(a.value())))
                    .collect();
                (name, attrs)
            })
            .collect();
        Ok(BalanceXml { rows })
    }
}

/// `get_attrib_num`: the leading integer of the text, 100 if there is none.
fn leading_int(s: &str) -> i32 {
    let t = s.trim();
    let end = t
        .char_indices()
        .take_while(|(i, c)| c.is_ascii_digit() || (*i == 0 && *c == '-'))
        .map(|(i, c)| i + c.len_utf8())
        .last()
        .unwrap_or(0);
    t[..end].parse().unwrap_or(100)
}

/// What a unit record alone says about its [`Kind`]: the `OBJ_MASK` letters,
/// the `FLAGS` letter `r` (`is_siege`), the domain from `TYPE`, its own row.
/// The age and the named lineages are left at their defaults — see the module
/// note.
pub fn unit_kind(index: usize, r: &crate::Record) -> Kind {
    let masks = r.text("OBJ_MASK").map_or(0, mask::parse);
    let flags = r.text("FLAGS").unwrap_or("");
    let domain = match r.text("TYPE").map(str::to_ascii_lowercase).as_deref() {
        Some("sea") | Some("naval") => sim::attrition::Domain::Sea,
        Some("air") => sim::attrition::Domain::Air,
        _ => sim::attrition::Domain::Land,
    };
    Kind {
        masks,
        age: 0,
        unit: true,
        siege: flags.contains('r'),
        domain,
        unit_index: Some(index),
        ..Kind::default()
    }
}

/// One entry of the table for two unit kinds, from the file's half and the
/// hardcoded half — `compute_modifier` with the kinds [`unit_kind`] can fill.
pub fn entry(t: &sim::Tuning, xml: &[i16], a: &Kind, b: &Kind) -> i32 {
    balance::compute_modifier(t, a, b, |r, c| {
        if r < CATEGORIES && c < CATEGORIES {
            i32::from(xml[r * CATEGORIES + c])
        } else {
            100
        }
    })
}
