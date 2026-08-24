//! Positional tables.
//!
//! Rise of Nations' data files are C arrays serialised to XML. The engine
//! reads the *N*th child into slot *N* and never looks at a tag name; the
//! evidence for that is in `docs/FORMATS.md`, and the short version is that
//! `rules.xml` contains duplicate tag names inside a single parent, none of
//! the distinctive tag names appear anywhere in the shipped binaries, and the
//! loader's own error messages only ever complain about counts.
//!
//! So this module preserves order and treats names as labels. A record's index
//! *is* its type id as far as the engine is concerned, which is very likely how
//! orders encode unit and building types — getting it right here is what will
//! connect the content work to the replay work later.
//!
//! Lookup by name is offered because it is convenient and the names are
//! genuinely descriptive, but [`Record::field`] returns the *first* match and
//! callers that care about a duplicated name must go by index.

use std::fmt;

/// One field of one record: a tag, its text, and its attributes, in file order.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Field {
    /// The element name. A human-readable label, not a key.
    pub tag: String,
    /// Trimmed element text. The shipped files pad text to fixed widths
    /// (`AZTECS.XML   `, `aztecs   `) because they are dumps of fixed-size
    /// `char` arrays; the padding is not data.
    pub text: String,
    /// Attributes in file order. Constants use `value`, or `entry0`..`entryN`
    /// for array-valued ones.
    pub attrs: Vec<(String, String)>,
}

impl Field {
    /// The `value` attribute, if present. Most constants carry exactly this.
    pub fn value(&self) -> Option<&str> {
        self.attr("value")
    }

    /// An attribute by name.
    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }

    /// The `entry0`, `entry1`, … attributes in order, for array-valued
    /// constants such as `POP_CAP` or `GRANARY_BONUS`.
    pub fn entries(&self) -> Vec<&str> {
        let mut out = Vec::new();
        for i in 0.. {
            match self.attr(&format!("entry{i}")) {
                Some(v) => out.push(v),
                None => break,
            }
        }
        out
    }

    /// The scalar payload of this field, wherever it lives.
    ///
    /// Constants put it in `value`; record tables such as `unitrules.xml` put
    /// it in the element text. Trying text first and falling back to `value`
    /// covers both without the caller having to know which file it came from.
    pub fn scalar_text(&self) -> Option<&str> {
        if !self.text.is_empty() {
            Some(&self.text)
        } else {
            self.value()
        }
    }
}

/// One record: a `UNIT`, a `BUILDING`, a `TECH`, or one slot of `CONSTANTS`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Record {
    /// The element name of the record itself.
    pub tag: String,
    /// Fields in file order.
    pub fields: Vec<Field>,
    /// The record element's own attributes — a `<CATEGORY name key/>` in
    /// rules.xml's setup enumerations carries its whole content there.
    pub attrs: Vec<(String, String)>,
}

impl Record {
    /// The first field with this tag.
    ///
    /// Convenient and usually right, but see the module note: names are labels.
    /// Where a name repeats, index into [`Record::fields`] instead.
    pub fn field(&self, tag: &str) -> Option<&Field> {
        self.fields.iter().find(|f| f.tag == tag)
    }

    /// The text of the first field with this tag.
    pub fn text(&self, tag: &str) -> Option<&str> {
        self.field(tag).map(|f| f.text.as_str())
    }

    /// How many fields carry this tag. Greater than one means the name is
    /// ambiguous and positional access is mandatory.
    pub fn count(&self, tag: &str) -> usize {
        self.fields.iter().filter(|f| f.tag == tag).count()
    }
}

/// An ordered table of records. Index is identity.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Table {
    /// The parent element these records came from, for diagnostics.
    pub name: String,
    pub records: Vec<Record>,
}

impl Table {
    /// The record at a type id.
    pub fn get(&self, index: usize) -> Option<&Record> {
        self.records.get(index)
    }

    /// Number of records. The engine asserts this against a compile-time
    /// constant — `NUM_UNITTYPES`, `NUM_TRIBES` and friends — and refuses to
    /// start if it disagrees, so a change here is a change to the engine.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Every tag that appears more than once at record level, with its count.
    ///
    /// This is a correctness check with teeth rather than a curiosity: a
    /// non-empty result is proof that name-keyed loading would be lossy for
    /// this table. `rules.xml`'s constants return five.
    pub fn duplicate_tags(&self) -> Vec<(String, usize)> {
        let mut seen: Vec<(String, usize)> = Vec::new();
        for r in &self.records {
            match seen.iter_mut().find(|(t, _)| *t == r.tag) {
                Some((_, n)) => *n += 1,
                None => seen.push((r.tag.clone(), 1)),
            }
        }
        seen.retain(|(_, n)| *n > 1);
        seen
    }

    /// Records in order, paired with their index.
    pub fn iter(&self) -> impl Iterator<Item = (usize, &Record)> {
        self.records.iter().enumerate()
    }
}

/// Anything that can go wrong reading a data file.
#[derive(Debug)]
pub enum Error {
    /// The file could not be read from the install.
    Io {
        path: String,
        source: std::io::Error,
    },
    /// The file is not well-formed XML.
    Xml {
        path: String,
        source: roxmltree::Error,
    },
    /// The file parsed but did not contain the element we needed.
    Missing { path: String, what: String },
    /// A binary file whose bytes stopped matching the documented format at a
    /// known offset (`docs/RECGAME.md`).
    Format {
        path: String,
        at: usize,
        what: String,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io { path, source } => write!(f, "reading {path}: {source}"),
            Error::Xml { path, source } => write!(f, "parsing {path}: {source}"),
            Error::Missing { path, what } => write!(f, "{path} has no {what}"),
            Error::Format { path, at, what } => write!(f, "{path} at byte {at:#x}: {what}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io { source, .. } => Some(source),
            Error::Xml { source, .. } => Some(source),
            Error::Missing { .. } | Error::Format { .. } => None,
        }
    }
}

/// Builds a [`Record`] from an element, taking its children as fields.
pub(crate) fn record_from(node: roxmltree::Node<'_, '_>) -> Record {
    Record {
        tag: node.tag_name().name().to_string(),
        fields: node
            .children()
            .filter(|n| n.is_element())
            .map(field_from)
            .collect(),
        attrs: node
            .attributes()
            .map(|a| (a.name().to_string(), a.value().to_string()))
            .collect(),
    }
}

/// Builds a [`Field`] from an element.
pub(crate) fn field_from(node: roxmltree::Node<'_, '_>) -> Field {
    Field {
        tag: node.tag_name().name().to_string(),
        text: node.text().unwrap_or("").trim().to_string(),
        attrs: node
            .attributes()
            .map(|a| (a.name().to_string(), a.value().to_string()))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(xml: &str) -> Table {
        let doc = roxmltree::Document::parse(xml).unwrap();
        Table {
            name: "TEST".into(),
            records: doc
                .root_element()
                .children()
                .filter(|n| n.is_element())
                .map(record_from)
                .collect(),
        }
    }

    #[test]
    fn order_is_preserved_and_is_the_identity() {
        let t = parse("<R><A/><B/><C/></R>");
        assert_eq!(t.len(), 3);
        assert_eq!(t.get(0).unwrap().tag, "A");
        assert_eq!(t.get(2).unwrap().tag, "C");
    }

    #[test]
    fn duplicate_record_tags_are_reported() {
        // The shape of rules.xml:539-549, where a five-constant block is
        // repeated. Two slots share a name; only the index tells them apart.
        let t =
            parse("<R><CTW_ATTRITION value='50%'/><X value='1'/><CTW_ATTRITION value='50%'/></R>");
        assert_eq!(t.duplicate_tags(), vec![("CTW_ATTRITION".to_string(), 2)]);
        assert_eq!(t.get(0).unwrap().tag, "CTW_ATTRITION");
        assert_eq!(t.get(2).unwrap().tag, "CTW_ATTRITION");
    }

    #[test]
    fn fixed_width_padding_is_stripped() {
        // <FILE>AZTECS.XML   </FILE> is a char[13], not a value with spaces.
        let t = parse("<R><TRIBE><FILE>AZTECS.XML   </FILE><KEY>aztecs   </KEY></TRIBE></R>");
        let r = t.get(0).unwrap();
        assert_eq!(r.text("FILE"), Some("AZTECS.XML"));
        assert_eq!(r.text("KEY"), Some("aztecs"));
    }

    #[test]
    fn entry_attributes_come_out_in_order() {
        let t = parse("<R><POP_CAP entry0='10' entry1='20' entry2='30'/></R>");
        let f = &t.get(0).unwrap().fields;
        // POP_CAP is a record here, so read it as one.
        assert!(f.is_empty());
        let t = parse("<R><REC><POP_CAP entry0='10' entry1='20' entry2='30'/></REC></R>");
        let e = t.get(0).unwrap().field("POP_CAP").unwrap().entries();
        assert_eq!(e, vec!["10", "20", "30"]);
    }

    #[test]
    fn scalar_text_finds_the_payload_in_either_shape() {
        // Constants put it in an attribute; record tables put it in the text.
        let t = parse("<R><REC><ATTRITION value='48 frames'/><HITS>40</HITS></REC></R>");
        let r = t.get(0).unwrap();
        assert_eq!(
            r.field("ATTRITION").unwrap().scalar_text(),
            Some("48 frames")
        );
        assert_eq!(r.field("HITS").unwrap().scalar_text(), Some("40"));
    }
}
