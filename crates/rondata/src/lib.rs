//! Reads Rise of Nations' shipped data tables out of an installed copy.
//!
//! Nothing from the game lives in this repository and nothing ever will. This
//! crate takes a path to the user's own install and reads from it, which is
//! both the legal line and the reason the tools have to be careful about where
//! they look.
//!
//! # What the data actually is
//!
//! Six ordered tables and a pool of global constants, all of them C arrays
//! serialised to XML by an editor in 2002. The engine parses them
//! **positionally** — see [`table`] and `docs/FORMATS.md` — so a record's
//! index is its type id and element names are documentation.
//!
//! The shipped `.dtd` files are stale XMLSpy output and describe neither the
//! current structure nor the current ordering. They are not used here.
//!
//! # Example
//!
//! ```no_run
//! use rondata::Install;
//!
//! let install = Install::new("/path/to/Rise of Nations");
//! let rules = install.rules()?;
//! println!("{} constants, {} nations", rules.constants.len(), rules.tribes.len());
//! # Ok::<(), rondata::Error>(())
//! ```

pub mod balance;
pub mod dump;
pub mod gamelog;
pub mod scalar;
pub mod table;
pub mod tuning;

use std::path::{Path, PathBuf};

pub use scalar::{Cost, Range, Resource, Scalar};
pub use table::{Error, Field, Record, Table};
pub use tuning::{Drift, drift};

/// A path to the user's installed copy of the game.
#[derive(Clone, Debug)]
pub struct Install {
    root: PathBuf,
}

/// The tables that live in `rules.xml`.
///
/// One file, several unrelated arrays, because in 2002 that was one header.
#[derive(Clone, Debug, Default)]
pub struct Rules {
    /// Global tuning. 723 slots in the shipped file, of which the stale DTD
    /// describes 156 — and five names appear twice.
    pub constants: Table,
    /// Technology bonuses, each carrying one prerequisite.
    pub tech_bonuses: Table,
    /// Unit formations: line, wedge, column, and the rest.
    pub formations: Table,
    /// Gatherable land types, each with the resources it makes.
    pub lands: Table,
    /// The 24 nations, as pointers to per-nation files.
    pub tribes: Table,
    /// Setup enumerations, keyed by the `id` attribute of their parent —
    /// `mapstyles`, `victories`, `poplimits`, and so on.
    pub categories: Vec<(String, Table)>,
}

impl Install {
    /// Points at an install root — the directory holding `riseofnations.exe`.
    pub fn new(root: impl AsRef<Path>) -> Install {
        Install {
            root: root.as_ref().to_path_buf(),
        }
    }

    /// The install root.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Whether this looks like a Rise of Nations install rather than an
    /// arbitrary directory.
    pub fn looks_valid(&self) -> bool {
        self.root.join("Data/rules.xml").is_file()
    }

    /// The path to the matched private debug symbols, if the depot shipped
    /// them. Extended Edition does; see `docs/FORMATS.md`.
    pub fn pdb(&self) -> Option<PathBuf> {
        let p = self.root.join("sbl/rise.pdb");
        p.is_file().then_some(p)
    }

    pub(crate) fn data(&self, file: &str) -> PathBuf {
        self.root.join("Data").join(file)
    }

    /// Reads and parses `rules.xml`.
    pub fn rules(&self) -> Result<Rules, Error> {
        let path = self.data("rules.xml");
        let text = read(&path)?;
        let doc = parse(&path, &text)?;
        let root = doc.root_element();

        let mut rules = Rules::default();
        for node in root.children().filter(|n| n.is_element()) {
            let name = node.tag_name().name();
            let of = |n: roxmltree::Node<'_, '_>| Table {
                name: name.to_string(),
                records: n
                    .children()
                    .filter(|c| c.is_element())
                    .map(table::record_from)
                    .collect(),
            };
            match name {
                // CONSTANTS' children are leaves, not records: each is one
                // slot. Wrapping each as a record keeps indexing uniform.
                "CONSTANTS" => {
                    rules.constants = Table {
                        name: name.to_string(),
                        records: node
                            .children()
                            .filter(|c| c.is_element())
                            .map(|c| Record {
                                tag: c.tag_name().name().to_string(),
                                fields: vec![table::field_from(c)],
                            })
                            .collect(),
                    }
                }
                "TECHBONUSES" => rules.tech_bonuses = of(node),
                "FORMATIONS" => rules.formations = of(node),
                "LANDS" => rules.lands = of(node),
                "TRIBES" => rules.tribes = of(node),
                // TRIBES_TRIAL_VERSION is the 18-nation demo roster. Ignored.
                "CATEGORIES" => {
                    let id = node.attribute("id").unwrap_or_default().to_string();
                    rules.categories.push((id, of(node)));
                }
                _ => {}
            }
        }

        if rules.constants.is_empty() {
            return Err(Error::Missing {
                path: path.display().to_string(),
                what: "CONSTANTS".into(),
            });
        }
        Ok(rules)
    }

    /// Reads `unitrules.xml`. 364 records in the shipped file, of which many
    /// are per-nation art variants of the same unit selected by `TRIBE_MASK`.
    pub fn units(&self) -> Result<Table, Error> {
        self.records("unitrules.xml", "UNIT")
    }

    /// Reads `buildingrules.xml`.
    pub fn buildings(&self) -> Result<Table, Error> {
        self.records("buildingrules.xml", "BUILDING")
    }

    /// Reads `techrules.xml`.
    pub fn techs(&self) -> Result<Table, Error> {
        self.records("techrules.xml", "TECH")
    }

    /// Reads `resourcerules.xml`. The first six records are the basic goods in
    /// the engine's own order; the other forty-four are the rare resources.
    ///
    /// This one file wraps its records in a `RESOURCES` element rather than
    /// hanging them off the root the way the other tables do — the same
    /// inconsistency `rules.xml` has, and the reason [`Install::records`] takes
    /// the container to look inside.
    pub fn resources(&self) -> Result<Table, Error> {
        self.records_in("resourcerules.xml", Some("RESOURCES"), "RESOURCE")
    }

    /// Reads a flat table of same-named records from a data file.
    ///
    /// `COMMENTS` elements are skipped: they are a designer's header, and
    /// counting one as a record would shift every type id by one.
    fn records(&self, file: &str, tag: &str) -> Result<Table, Error> {
        self.records_in(file, None, tag)
    }

    /// [`Install::records`], optionally descending through one wrapper element
    /// first.
    fn records_in(&self, file: &str, container: Option<&str>, tag: &str) -> Result<Table, Error> {
        let path = self.data(file);
        let text = read(&path)?;
        let doc = parse(&path, &text)?;
        let root = doc.root_element();
        let parent = match container {
            None => Some(root),
            Some(c) => root
                .children()
                .find(|n| n.is_element() && n.tag_name().name() == c),
        };
        let records: Vec<Record> = parent
            .into_iter()
            .flat_map(|p| p.children())
            .filter(|n| n.is_element() && n.tag_name().name() == tag)
            .map(table::record_from)
            .collect();
        if records.is_empty() {
            return Err(Error::Missing {
                path: path.display().to_string(),
                what: tag.to_string(),
            });
        }
        Ok(Table {
            name: tag.to_string(),
            records,
        })
    }
}

impl Rules {
    /// A named constant's scalar, by first occurrence.
    ///
    /// Names are labels, not keys — five are duplicated in the shipped file.
    /// This is the convenient path; anything load-bearing should go by index
    /// and check [`Table::duplicate_tags`] first.
    pub fn constant(&self, name: &str) -> Option<Scalar> {
        let rec = self.constants.records.iter().find(|r| r.tag == name)?;
        Scalar::parse(rec.fields.first()?.scalar_text()?)
    }

    /// The `entry0`..`entryN` values of an array-valued constant.
    pub fn constant_entries(&self, name: &str) -> Option<Vec<Scalar>> {
        let rec = self.constants.records.iter().find(|r| r.tag == name)?;
        Some(
            rec.fields
                .first()?
                .entries()
                .into_iter()
                .filter_map(Scalar::parse)
                .collect(),
        )
    }

    /// The nation keys, in engine order. Index is the bit position used by
    /// `TRIBE_MASK`, counting from the *right* of the mask string.
    pub fn tribe_keys(&self) -> Vec<String> {
        self.tribes
            .records
            .iter()
            .filter_map(|r| r.text("KEY").map(str::to_string))
            .collect()
    }
}

/// Decodes a `TRIBE_MASK`: a binary string, most-significant bit first, in
/// which bit *i* selects tribe *i*.
///
/// The ordering is the surprising part and it is settled by evidence rather
/// than convention — `Samurai` resolves to japanese and `Cossack` to russians
/// only under this reading; taking the string left to right yields turks and
/// french. See `docs/FORMATS.md`.
pub fn tribe_mask(mask: &str) -> Vec<usize> {
    let m = mask.trim();
    let n = m.len();
    m.char_indices()
        .filter(|(_, c)| *c == '1')
        .map(|(i, _)| n - 1 - i)
        .collect()
}

pub(crate) fn read(path: &Path) -> Result<String, Error> {
    std::fs::read_to_string(path).or_else(|e| {
        // Some shipped files carry stray non-UTF-8 bytes in designer comments.
        // Losing a character of commentary is fine; failing to load is not.
        std::fs::read(path)
            .map(|b| String::from_utf8_lossy(&b).into_owned())
            .map_err(|_| Error::Io {
                path: path.display().to_string(),
                source: e,
            })
    })
}

pub(crate) fn parse<'a>(path: &Path, text: &'a str) -> Result<roxmltree::Document<'a>, Error> {
    roxmltree::Document::parse(text).map_err(|source| Error::Xml {
        path: path.display().to_string(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tribe_mask_is_msb_first() {
        // 24 nations; koreans are index 16. CITIZENSKOREAN's mask has its only
        // set bit at string position 7, which is bit 16 counting from the right.
        let m = "000000010000000000000000";
        assert_eq!(tribe_mask(m), vec![16]);

        // And the base CITIZENS record is its exact complement.
        let base = "111111101111111111111111";
        let mut bits = tribe_mask(base);
        bits.sort_unstable();
        assert_eq!(bits.len(), 23);
        assert!(!bits.contains(&16));
    }

    #[test]
    fn tribe_mask_handles_all_and_none() {
        assert_eq!(tribe_mask(&"0".repeat(24)), Vec::<usize>::new());
        assert_eq!(tribe_mask(&"1".repeat(24)).len(), 24);
    }
}
