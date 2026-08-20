//! Checks the simulation's copy of the tuned numbers against a real install.
//!
//! `sim::Tuning::RON` writes down the constants attrition depends on, because
//! a formula whose inputs nobody can see is not a specification. Writing them
//! down is also how they go stale. This re-reads them from the user's own
//! `rules.xml` and reports anything that has drifted, so a patch that
//! rebalances attrition shows up as a failed check rather than as a
//! simulation that is quietly wrong.
//!
//! It is the same discipline `docs/FORMATS.md` gets from the structural
//! checks, applied to values instead of structure.

use crate::{Rules, Scalar};
use sim::tuning::{Slot, Tuning};

/// One constant that does not match what the simulation believes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Drift {
    /// The constant's name in `rules.xml`.
    pub name: &'static str,
    /// What `sim::Tuning::RON` says.
    pub ours: String,
    /// What the install says, or `None` if the constant is not there at all.
    pub theirs: Option<String>,
}

/// Re-derives every constant behind [`Tuning::RON`] from an install's
/// `rules.xml` and returns the ones that disagree.
///
/// An empty result means the simulation's tuning table is exactly what the
/// user's copy of the game ships.
pub fn drift(rules: &Rules) -> Vec<Drift> {
    let mut out = Vec::new();
    for (name, slot) in Tuning::ron_slots() {
        match slot {
            Slot::Value(ours) => {
                let theirs = rules.constant(name).map(Scalar::written_int);
                if theirs != Some(ours) {
                    out.push(Drift {
                        name,
                        ours: ours.to_string(),
                        theirs: theirs.map(|v| v.to_string()),
                    });
                }
            }
            Slot::Ratio256(ours) => {
                // The file writes a rational and the engine loads it as 8.8
                // fixed point, so the check has to rescale rather than compare
                // digits. It rescales through the engine's own routine rather
                // than through `Fx`, which would be exact for the denominators
                // this slot happens to use and is not exact in general — see
                // `Ratio100` below for the case that proves it.
                let theirs = rules.constant(name).map(|s| s.fraction(256));
                if theirs != Some(ours) {
                    out.push(Drift {
                        name,
                        ours: ours.to_string(),
                        theirs: theirs.map(|v| v.to_string()),
                    });
                }
            }
            Slot::Ratio100(ours) => {
                // The same rescale against a different denominator.
                // `get_fraction(name, 100)` is what fixes it, and it sits
                // three lines from a `get_fraction(name, 0x100)` in the same
                // loader; see `docs/PRODUCTION.md`.
                //
                // This is the slot that forced the check off `Fx`.
                // `UNIT_RATE_BASE` is `6/5`, a fifth is not a dyadic rational,
                // and the Q16.16 route reported 119 against the original's
                // exact 120.
                let theirs = rules.constant(name).map(|s| s.fraction(100));
                if theirs != Some(ours) {
                    out.push(Drift {
                        name,
                        ours: ours.to_string(),
                        theirs: theirs.map(|v| v.to_string()),
                    });
                }
            }
            Slot::Ratio192(ours) => {
                // `get_fraction(name, 0xc0)`: a length in position units.
                let theirs = rules.constant(name).map(|s| s.fraction(192));
                if theirs != Some(ours) {
                    out.push(Drift {
                        name,
                        ours: ours.to_string(),
                        theirs: theirs.map(|v| v.to_string()),
                    });
                }
            }
            Slot::Entries256(ours) => {
                let theirs: Option<Vec<i32>> = rules
                    .constant_entries(name)
                    .map(|v| v.into_iter().map(|s| s.fraction(256)).collect());
                let same = theirs
                    .as_ref()
                    .is_some_and(|t| t.len() >= ours.len() && t[..ours.len()] == *ours);
                if !same {
                    out.push(Drift {
                        name,
                        ours: list(ours),
                        theirs: theirs.map(|t| list(&t)),
                    });
                }
            }
            Slot::Entries(ours) => {
                let theirs: Option<Vec<i32>> = rules
                    .constant_entries(name)
                    .map(|v| v.into_iter().map(Scalar::written_int).collect());
                // The shipped arrays are sometimes longer than the part the
                // engine indexes, so compare only as far as we claim to know.
                let same = theirs
                    .as_ref()
                    .is_some_and(|t| t.len() >= ours.len() && t[..ours.len()] == *ours);
                if !same {
                    out.push(Drift {
                        name,
                        ours: list(ours),
                        theirs: theirs.map(|t| list(&t)),
                    });
                }
            }
        }
    }
    out
}

fn list(v: &[i32]) -> String {
    v.iter().map(i32::to_string).collect::<Vec<_>>().join(", ")
}
