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
                // digits. `Scalar::to_fx` is exact for these denominators, and
                // Q16.16 divided down by 256 is Q8.8 with the same truncation.
                let theirs = rules.constant(name).map(|s| s.to_fx().raw() / 256);
                if theirs != Some(ours) {
                    out.push(Drift {
                        name,
                        ours: ours.to_string(),
                        theirs: theirs.map(|v| v.to_string()),
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
