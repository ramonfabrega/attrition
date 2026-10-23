//! The gamelog diff: the original's per-frame dump against the simulation.
//!
//! Phase 3's score is ticks before divergence. This is the harness that
//! produces it: [`build_sim`] stands a `sim::Sim` up from a start-of-game
//! dump ([`crate::gamelog::Initial`]) with the loaded types
//! ([`crate::load::Loaded`]), and [`run`] steps it frame by frame against the
//! dump's `FRAME n` blocks, comparing what both sides hold.
//!
//! # What it can see today, and what it cannot
//!
//! At detail level 0 the original writes, per unit per frame, the object base
//! — owner, object number, `x_internal`/`y_internal`/`z_internal` — and per
//! leader its `score` and flags. So the comparison is **positions of units
//! keyed by `(who, o)`**, and the score is reported but not matched (the
//! simulation has no score yet). The per-frame `GUY` blocks are empty, the
//! leaders carry no goods, and buildings are not written per frame under the
//! categories the logged runs enabled; `DUMP_ALL=1` is the untried lever for
//! more (`docs/ORACLE.md`).
//!
//! And the simulation has no AI and sees no orders: in the logged runs the
//! human's units stand still and the AI's move as soon as it decides to, so
//! the honest expectation is that player 0 matches for as long as the human
//! did nothing and the AI player diverges the frame its first order lands.
//! That is still worth running — it proves the ids, the types, the world
//! scale and the step cadence agree end to end — and it is the scaffold the
//! order stream plugs into when the recorded-game container is read.

use crate::gamelog::{Frame, Initial, Log, Pos as LogPos, UnitDump};
use crate::load::Loaded;
use sim::{Pos, Sim, Tuning, Unit, World};

#[cfg(test)]
mod american_lakes_word;
#[cfg(test)]
mod american_word;
#[cfg(test)]
mod aztec_alltech_pair;
#[cfg(test)]
mod aztec_word;
#[cfg(test)]
mod bantu_word;
#[cfg(test)]
mod battery;
mod build;
mod city;
mod corrections;
#[cfg(test)]
mod coverage_pair;
#[cfg(test)]
mod dutch_sahara_word;
#[cfg(test)]
mod dutch_word;
#[cfg(test)]
mod egyptian_word;
mod endpoint;
mod floors;
#[cfg(test)]
mod french_word;
#[cfg(test)]
mod german_sahara_word;
#[cfg(test)]
mod german_word;
#[cfg(test)]
mod golden;
#[cfg(test)]
mod greek_word;
mod harness;
#[cfg(test)]
mod inca_sahara_word;
#[cfg(test)]
mod inca_word;
#[cfg(test)]
mod indian_sahara_word;
#[cfg(test)]
mod indian_word;
#[cfg(test)]
mod iroquois_sahara_word;
#[cfg(test)]
mod iroquois_word;
#[cfg(test)]
mod japanese_word;
#[cfg(test)]
mod korean_word;
#[cfg(test)]
mod lakota_pair;
#[cfg(test)]
mod lakota_word;
#[cfg(test)]
mod leader;
#[cfg(test)]
mod maya_word;
#[cfg(test)]
mod mongol_pair;
pub mod oracle;
mod order;
#[cfg(test)]
mod persian_word;
mod report;
#[cfg(test)]
mod roman_word;
#[cfg(test)]
mod russian_word;
#[cfg(test)]
mod sahara_coverage;
#[cfg(test)]
mod sahara_mongol;
#[cfg(test)]
mod sahara_toughest;
#[cfg(test)]
mod second;
mod setup;
mod shutdown;
#[cfg(test)]
mod slots;
#[cfg(test)]
mod spanish_east_word;
#[cfg(test)]
mod spanish_lakes_word;
#[cfg(test)]
mod spanish_word;
#[cfg(test)]
mod third;
#[cfg(test)]
mod third_pair;
#[cfg(test)]
mod turkish_word;
mod unit;

#[cfg(test)]
mod ammo;
#[cfg(test)]
mod army;
#[cfg(test)]
mod coverage;
#[cfg(test)]
pub(crate) mod testkit;
#[cfg(test)]
mod world;

/// **The compared recorder** — parked 527, built in the fourteenth Fable
/// pass (2026-09-25). `gamelog::reads` says which keys a parse *asked a
/// block for*; the coverage pin holds that against the dump's own keys and
/// stops at "parsed". Five landings in two tranches turned on the gap
/// after it: a field parsed and never compared (`o_up`, read once at
/// stand-up, 523), units linked on `(who, o)` with their numbers never
/// compared (617), a building the dump held and the crate counted rather
/// than noted (644), a comparison against a literal (661), a widening
/// silent on the word's own block (642). A read is not a comparison, and
/// nothing measured the difference.
///
/// This records the other half: every site of the shared instrument —
/// [`harness::compare`], [`order::compare_orders`] and
/// [`harness::widen_block`] — notes the `Record.field` it compared with
/// **both sides present**, and `coverage`'s pin holds the parser's own
/// records against what arrived. A field whose comparison is gated on
/// the position, the detail level or the other side's presence registers
/// only on the frames it actually ran, which is the point.
///
/// Recording is per thread and off unless a test starts it, so the walk
/// pays one thread-local check per site.
pub(crate) mod compared {
    use std::cell::RefCell;
    use std::collections::{BTreeSet, HashSet};
    use std::hash::{Hash, Hasher};

    // Addresses are only internal identities of immutable, static schema strings.
    // Different allocations with equal text may occupy separate entries; labels()
    // normalizes them by value. Include lengths so shared-prefix slices stay distinct.
    #[derive(Clone, Copy)]
    struct Field(&'static str, &'static str);
    impl PartialEq for Field {
        fn eq(&self, other: &Self) -> bool {
            std::ptr::eq(self.0, other.0) && std::ptr::eq(self.1, other.1)
        }
    }
    impl Eq for Field {}
    impl Hash for Field {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.0.as_ptr().hash(state);
            self.0.len().hash(state);
            self.1.as_ptr().hash(state);
            self.1.len().hash(state);
        }
    }
    struct Fields {
        fast: HashSet<Field>,
        tree: BTreeSet<(&'static str, &'static str)>,
        legacy: bool,
        verify: bool,
    }
    impl Default for Fields {
        fn default() -> Self {
            Self {
                fast: HashSet::new(),
                tree: BTreeSet::new(),
                legacy: std::env::var_os("RON_DISABLE_FAST_COMPARE_NOTES").is_some(),
                verify: std::env::var_os("RON_VERIFY_COMPARE_NOTES").is_some(),
            }
        }
    }
    impl Fields {
        fn insert(&mut self, pair: (&'static str, &'static str)) {
            if !self.legacy {
                self.fast.insert(Field(pair.0, pair.1));
            }
            if self.legacy || self.verify {
                self.tree.insert(pair);
            }
        }
        #[cfg(test)]
        fn iter(&self) -> impl Iterator<Item = (&'static str, &'static str)> {
            self.fast
                .iter()
                .map(|p| (p.0, p.1))
                .chain(self.tree.iter().copied())
        }
    }
    thread_local! {
        static ON: RefCell<Option<Fields>> = const { RefCell::new(None) };
    }

    #[cfg(test)]
    pub(crate) fn start() {
        ON.with(|r| *r.borrow_mut() = Some(Fields::default()));
    }

    /// Allocate the public field labels once per unique field, at the boundary.
    #[cfg(test)]
    fn labels(fields: &Fields) -> BTreeSet<String> {
        if fields.verify && !fields.legacy {
            let fast: BTreeSet<_> = fields
                .fast
                .iter()
                .map(|p| format!("{}.{}", p.0, p.1))
                .collect();
            let tree: BTreeSet<_> = fields
                .tree
                .iter()
                .map(|(r, f)| format!("{r}.{f}"))
                .collect();
            assert_eq!(fast, tree, "complete comparison registration sets differ");
        }
        fields
            .iter()
            .map(|(record, field)| format!("{record}.{field}"))
            .collect()
    }

    #[cfg(test)]
    pub(crate) fn stop() -> BTreeSet<String> {
        ON.with(|r| labels(&r.borrow_mut().take().unwrap_or_default()))
    }

    /// Audit a walk independently, then merge its notes into the outer recorder.
    #[cfg(test)]
    pub(crate) fn recorded<R>(walk: impl FnOnce() -> R) -> (R, BTreeSet<String>) {
        let outer = ON.with(|r| r.replace(Some(Fields::default())));
        let result = walk();
        let notes = ON.with(|r| r.borrow_mut().take().unwrap_or_default());
        ON.with(|r| {
            *r.borrow_mut() = outer.map(|mut seen| {
                for pair in notes.iter() {
                    seen.insert(pair);
                }
                seen
            });
        });
        (result, labels(&notes))
    }

    /// Records are static schema names; no fixture text is retained.
    /// Both sides must still be present before callers note a comparison.
    pub(crate) fn note(record: &'static str, fields: &[&'static str]) {
        ON.with(|r| {
            if let Some(s) = r.borrow_mut().as_mut() {
                for &field in fields {
                    s.insert((record, field));
                }
            }
        });
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn static_identity_preserves_equal_text_and_distinct_prefix_slices() {
            static A: [u8; 8] = *b"UnitDump";
            static B: [u8; 8] = *b"UnitDump";
            let a = std::str::from_utf8(&A).unwrap();
            let b = std::str::from_utf8(&B).unwrap();
            let mut fields = Fields {
                verify: true,
                legacy: false,
                ..Fields::default()
            };
            for record in [a, b, "UnitDump", &"UnitDumpExtra"[..8]] {
                for field in ["angle", &"angle"[..1], "pos", "angle"] {
                    fields.insert((record, field));
                }
            }
            assert_eq!(
                labels(&fields),
                BTreeSet::from([
                    "UnitDump.angle".into(),
                    "UnitDump.a".into(),
                    "UnitDump.pos".into()
                ])
            );
            let mut legacy = Fields {
                legacy: true,
                verify: false,
                ..Fields::default()
            };
            for pair in fields.iter() {
                legacy.insert(pair);
            }
            assert_eq!(labels(&fields), labels(&legacy));
        }
        #[test]
        #[should_panic(expected = "complete comparison registration sets differ")]
        fn recorder_audit_rejects_a_missing_registration() {
            let mut fields = Fields {
                verify: true,
                legacy: false,
                ..Fields::default()
            };
            fields.insert(("Guy", "pos"));
            fields.fast.clear();
            let _ = labels(&fields);
        }
        #[test]
        fn interned_pairs_match_the_original_string_set_and_nested_recording() {
            let rows = [
                ("Guy", &["angle", "pos", "angle"][..]),
                ("UnitDump", &["pos", "who"][..]),
                ("Guy", &[][..]),
            ];
            let mut legacy = BTreeSet::new();
            start();
            for _ in 0..10 {
                for (record, fields) in rows {
                    note(record, fields);
                    for field in fields {
                        legacy.insert(format!("{record}.{field}"));
                    }
                }
            }
            assert_eq!(stop(), legacy);
            note("Ignored", &["off"]);
            assert!(stop().is_empty());
            start();
            note("Outer", &["field"]);
            let (value, inner) = recorded(|| {
                note("Inner", &["field"]);
                42
            });
            assert_eq!(value, 42);
            assert_eq!(inner, BTreeSet::from(["Inner.field".into()]));
            assert_eq!(
                stop(),
                BTreeSet::from(["Outer.field".into(), "Inner.field".into()])
            );
        }
    }
}

pub use build::*;
pub use city::*;
pub use corrections::*;
pub use endpoint::*;
pub use floors::*;
pub use harness::*;
pub use order::*;
pub use report::*;
pub use setup::*;
pub use shutdown::*;
pub use unit::*;
