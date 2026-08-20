//! Checks the constants against the original's own dump of them.
//!
//! `rules.xml` writes a constant the way a designer typed it — `2/3 (light
//! infantry in rocks)` — and `Constants::init` loads it in whatever
//! representation the consumer wants: a plain integer, 8.8 fixed point,
//! hundredths, or position units. Which is which is a fact about the loader,
//! one constant at a time, and until now it has been read off the decompile
//! for the 232 constants `sim::Tuning::RON` carries and assumed for the rest.
//!
//! The `BEGIN CONSTANTS` block of a start-of-game dump ([`crate::gamelog`]) is
//! the loaded struct written back out, field by field, under the lowercased
//! tag. So the representation of *every* constant can be classified by
//! asking which rescaling of the written value reproduces the dumped one —
//! and every `Slot` the simulation claims can be checked directly, with no
//! rescaling at all, because both sides are already in memory form.

use crate::gamelog::ConstantDump;
use crate::{Drift, Rules, Scalar};
use sim::tuning::{Slot, Tuning};

/// How the engine turned a written constant into the number it holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Loaded {
    /// The written integer, or a rational's numerator: `Scalar::written_int`.
    Plain,
    /// `num * 256 / den` — 8.8 fixed point.
    Scaled256,
    /// `num * 100 / den` — hundredths.
    Scaled100,
    /// `num * 192 / den` — position units, 192 to the tile.
    Scaled192,
    /// The written integer times 48 — a `UCoord`, a quarter tile in position
    /// units. `UNIT_BLOCK_RADIUS` (`1 UCoord`) is the one: `Constants::init`
    /// multiplies it by `0x30`.
    Scaled48,
    /// The written integer times ten — an attack value, in the tenths every
    /// `ATTACK` column is stored in. `CARAVAN_ATTACK_BONUS` (`2 attack`) is
    /// the one.
    Scaled10,
    /// More than one rule reproduces the dump — `0` and `1/1` are the usual
    /// culprits — so the dump cannot tell them apart.
    Ambiguous(Vec<&'static str>),
    /// No rule reproduces the dump: a boolean the loader normalises, a
    /// value computed from others, or a representation not yet named.
    Unexplained,
}

/// One constant's written form, dumped form, and the rule relating them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Classified {
    /// The `rules.xml` tag.
    pub name: String,
    /// The written value(s), parsed.
    pub written: Vec<Scalar>,
    /// The dumped value(s).
    pub dumped: Vec<i64>,
    pub loaded: Loaded,
}

/// A named rescaling rule.
type Rule = (&'static str, fn(Scalar) -> i64);

const RULES: [Rule; 6] = [
    ("plain", |s| i64::from(s.written_int())),
    ("x256", |s| i64::from(s.fraction(256))),
    ("x100", |s| i64::from(s.fraction(100))),
    ("x192", |s| i64::from(s.fraction(192))),
    ("x48", |s| i64::from(s.written_int()) * 48),
    ("x10", |s| i64::from(s.written_int()) * 10),
];

fn rule_for(written: &[Scalar], dumped: &[i64]) -> Loaded {
    // An array in the file can be longer than the struct's (`KOREAN_CITIZENS`
    // writes nine into an `int[8]`), and the dump can write less than the
    // struct holds (`scholar_rate` is `int[6]`, logged as five); compare the
    // overlap.
    let n = written.len().min(dumped.len());
    if n == 0 {
        return Loaded::Unexplained;
    }
    let hits: Vec<&'static str> = RULES
        .iter()
        .filter(|(_, f)| (0..n).all(|i| f(written[i]) == dumped[i]))
        .map(|(name, _)| *name)
        .collect();
    match hits.as_slice() {
        [] => Loaded::Unexplained,
        ["plain"] => Loaded::Plain,
        ["x256"] => Loaded::Scaled256,
        ["x100"] => Loaded::Scaled100,
        ["x192"] => Loaded::Scaled192,
        ["x48"] => Loaded::Scaled48,
        ["x10"] => Loaded::Scaled10,
        many => Loaded::Ambiguous(many.to_vec()),
    }
}

/// The `rules.xml` tags whose `Constants` field is not the tag lowercased.
///
/// Read off `Constants::init`: the loader reads the tag on the left into the
/// field on the right. Everything else is the lowercased tag.
///
/// (`taj_caravan`, which the dump holds at `-1`, is *not* `TAJ_CARAVAN_LIMIT`:
/// `-1` is what `get_item` returns for a key the file does not have, and
/// `kremlin_spy_instant`, `german_light_cavalry` and `russian_uber_spies`
/// sit at `-1` for the same reason.)
pub const ALIASES: [(&str, &str); 1] = [("CITY_UPGRADE_TERR", "city_level_territory_bonus")];

/// The `Constants` field a `rules.xml` tag is loaded into.
pub fn field_of(tag: &str) -> String {
    ALIASES
        .iter()
        .find(|(t, _)| *t == tag)
        .map_or_else(|| tag.to_ascii_lowercase(), |(_, f)| (*f).to_string())
}

/// The dumped constant under a `rules.xml` tag, if the dump has it.
///
/// The dump's key is the struct field — the tag lowercased, or its
/// [`ALIASES`] entry. This is by-name, so the five duplicated tags resolve to
/// whichever field the engine happens to name that way; the caller sees them
/// through [`crate::Table::duplicate_tags`].
pub fn dumped<'a>(dump: &'a [ConstantDump<'_>], tag: &str) -> Option<&'a ConstantDump<'a>> {
    let want = field_of(tag);
    dump.iter().find(|c| c.name == want)
}

/// Classifies every `rules.xml` constant that the dump also carries.
pub fn classify(rules: &Rules, dump: &[ConstantDump<'_>]) -> Vec<Classified> {
    let mut out = Vec::new();
    let mut seen: Vec<&str> = Vec::new();
    for rec in &rules.constants.records {
        if seen.contains(&rec.tag.as_str()) {
            continue;
        }
        seen.push(&rec.tag);
        let Some(d) = dumped(dump, &rec.tag) else {
            continue;
        };
        let Some(field) = rec.fields.first() else {
            continue;
        };
        let entries = field.entries();
        let written: Vec<Scalar> = if entries.is_empty() {
            field
                .scalar_text()
                .and_then(Scalar::parse)
                .into_iter()
                .collect()
        } else {
            entries.into_iter().filter_map(Scalar::parse).collect()
        };
        let loaded = rule_for(&written, &d.values);
        out.push(Classified {
            name: rec.tag.clone(),
            written,
            dumped: d.values.clone(),
            loaded,
        });
    }
    out
}

/// The tags the file has and the dump does not, and the dump keys the file
/// has no tag for.
pub fn unmatched(rules: &Rules, dump: &[ConstantDump<'_>]) -> (Vec<String>, Vec<String>) {
    let mut xml_only = Vec::new();
    for rec in &rules.constants.records {
        if dumped(dump, &rec.tag).is_none() && !xml_only.contains(&rec.tag) {
            xml_only.push(rec.tag.clone());
        }
    }
    let mut dump_only = Vec::new();
    for c in dump {
        let has = rules
            .constants
            .records
            .iter()
            .any(|r| field_of(&r.tag) == c.name);
        if !has && !dump_only.iter().any(|n: &String| n == c.name) {
            dump_only.push(c.name.to_string());
        }
    }
    (xml_only, dump_only)
}

/// [`tuning_drift`]'s two outcomes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TuningReport {
    /// Slots the dump carries with a different value: a real disagreement.
    pub mismatched: Vec<Drift>,
    /// Slots the dump does not carry at all. `Constants::log_data` omits
    /// some loaded fields — `liberty_free_upgrades` is one — so absence is
    /// not evidence of anything.
    pub missing: Vec<&'static str>,
}

/// Checks every slot of [`Tuning::RON`] against the dump.
///
/// Both sides are in memory representation, so this is a straight
/// comparison; a drift here is either our transcription or our *reading of
/// the loader* being wrong, and unlike [`crate::drift`] it cannot be the
/// former alone, because the dump is what the program holds.
pub fn tuning_drift(dump: &[ConstantDump<'_>]) -> TuningReport {
    let mut out = TuningReport::default();
    for (name, slot) in Tuning::ron_slots() {
        let theirs = dumped(dump, name).map(|c| c.values.clone());
        let (ours, same): (String, bool) = match slot {
            Slot::Value(v) | Slot::Ratio256(v) | Slot::Ratio100(v) | Slot::Ratio192(v) => (
                v.to_string(),
                theirs.as_deref() == Some(&[i64::from(v)][..]),
            ),
            Slot::Entries(e) | Slot::Entries256(e) => {
                let ours: Vec<i64> = e.iter().map(|&v| i64::from(v)).collect();
                // Compare the overlap. The dump can be shorter than the
                // struct: `scholar_rate` is `int[6]` in the symbols and
                // `Constants::log_data` writes five of it.
                let same = theirs.as_ref().is_some_and(|t| {
                    let n = t.len().min(ours.len());
                    n > 0 && t[..n] == ours[..n]
                });
                (list(&ours), same)
            }
        };
        match theirs {
            None => out.missing.push(name),
            Some(t) if !same => out.mismatched.push(Drift {
                name,
                ours,
                theirs: Some(list(&t)),
            }),
            Some(_) => {}
        }
    }
    out
}

fn list(v: &[i64]) -> String {
    v.iter().map(i64::to_string).collect::<Vec<_>>().join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn k<'a>(name: &'a str, values: &[i64]) -> ConstantDump<'a> {
        ConstantDump {
            name,
            values: values.to_vec(),
            array: values.len() > 1,
        }
    }

    #[test]
    fn rules_are_told_apart_where_the_numbers_allow() {
        let r = |t: &str| Scalar::parse(t).unwrap();
        assert_eq!(rule_for(&[r("2/3")], &[170]), Loaded::Scaled256);
        assert_eq!(rule_for(&[r("6/5")], &[120]), Loaded::Scaled100);
        assert_eq!(rule_for(&[r("1/2 tile")], &[96]), Loaded::Scaled192);
        assert_eq!(rule_for(&[r("48 frames")], &[48]), Loaded::Plain);
        assert_eq!(rule_for(&[r("10 resources")], &[2560]), Loaded::Scaled256);
        // 50% is 50 plain — the engine stores the face value.
        assert_eq!(rule_for(&[r("50%")], &[50]), Loaded::Plain);
        assert_eq!(rule_for(&[r("1 UCoord")], &[48]), Loaded::Scaled48);
        assert_eq!(rule_for(&[r("2 attack")], &[20]), Loaded::Scaled10);
        assert_eq!(rule_for(&[r("7")], &[9]), Loaded::Unexplained);
        assert_eq!(
            rule_for(&[r("0")], &[0]),
            Loaded::Ambiguous(vec!["plain", "x256", "x100", "x192", "x48", "x10"])
        );
        // Arrays compare element-wise over the overlap.
        assert_eq!(
            rule_for(&[r("2"), r("4"), r("6"), r("9")], &[2, 4, 6, 9]),
            Loaded::Plain
        );
        assert_eq!(
            rule_for(&[r("1/1"), r("3/2")], &[256, 384]),
            Loaded::Scaled256
        );
        // Overlap, either way round.
        assert_eq!(
            rule_for(&[r("5"), r("7"), r("10")], &[1280, 1792]),
            Loaded::Scaled256
        );
        assert_eq!(rule_for(&[r("1"), r("3")], &[1, 3, 5]), Loaded::Plain);
        assert_eq!(rule_for(&[], &[1]), Loaded::Unexplained);
    }

    #[test]
    fn dump_lookup_is_by_lowercased_tag() {
        let d = [
            k("rocky_modifier", &[170]),
            k("fort_upgrade_terr", &[2, 4, 6, 9]),
        ];
        assert_eq!(dumped(&d, "ROCKY_MODIFIER").unwrap().values, vec![170]);
        assert_eq!(dumped(&d, "FORT_UPGRADE_TERR").unwrap().values.len(), 4);
        assert!(dumped(&d, "NOPE").is_none());
        // The renamed field.
        let d = [k("city_level_territory_bonus", &[0, 3, 6])];
        assert_eq!(
            dumped(&d, "CITY_UPGRADE_TERR").unwrap().values,
            vec![0, 3, 6]
        );
    }

    #[test]
    fn tuning_drift_agrees_with_ron_on_a_consistent_dump() {
        // Build a dump straight from the table: no drift by construction.
        let mut owned: Vec<(String, Vec<i64>)> = Vec::new();
        for (name, slot) in Tuning::ron_slots() {
            let vals = match slot {
                Slot::Value(v) | Slot::Ratio256(v) | Slot::Ratio100(v) | Slot::Ratio192(v) => {
                    vec![i64::from(v)]
                }
                Slot::Entries(e) | Slot::Entries256(e) => e.iter().map(|&v| i64::from(v)).collect(),
            };
            owned.push((field_of(name), vals));
        }
        let dump: Vec<ConstantDump<'_>> = owned.iter().map(|(n, v)| k(n, v)).collect();
        let r = tuning_drift(&dump);
        assert!(r.mismatched.is_empty());
        assert!(r.missing.is_empty());
        // A single changed value is reported by name; a dropped one as missing.
        let mut bad = dump.clone();
        bad[0].values[0] += 1;
        let r = tuning_drift(&bad);
        assert_eq!(r.mismatched.len(), 1);
        assert_eq!(r.mismatched[0].name.to_ascii_lowercase(), bad[0].name);
        let r = tuning_drift(&dump[1..]);
        assert!(r.mismatched.is_empty());
        assert_eq!(r.missing.len(), 1);
    }
}
