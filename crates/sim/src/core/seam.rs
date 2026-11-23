//! `seam` — which seam notes a run walks through, and on which state
//! (item 4580, DECISIONS 153 §10).
//!
//! A seam note says where this crate knowingly takes another path than the
//! original: a fallback, an arm it answers in place of one it does not carry.
//! Read as prose, a note cannot say whether a game ever reaches it. The
//! [`seam!`](crate::seam!) hook placed on the note's own arm can: under the
//! `seam-count` feature each hook records, on its thread, the first state it
//! was entered on and how often, and a walk (`rondata`'s held-out scores)
//! reads the record back beside its first parting.
//!
//! **Without the feature the hook is nothing**: the macro expands to an empty
//! block, so the simulation's release code, every checksum, the soak's digest
//! and every floor are the code as it stood. With it, the hook only observes:
//! it writes a thread-local map the simulation never reads. The record is off
//! until a walk [`start`]s it, so a feature build that never starts one
//! counts nothing.
//!
//! A hook's name is its module path and a short tag, so `rg '<tag>'` under
//! the module finds the note it stands on.

/// Enter a seam: under `seam-count`, record the state it is entered on.
/// `seam!("tag")` names the hook `module::path::tag`; `seam!("tag", cond)`
/// enters it only when `cond` holds, the arm the original takes and this
/// crate does not. Without the feature both expand to nothing, `cond`
/// included, so a predicate costs no build without the feature.
#[macro_export]
macro_rules! seam {
    ($tag:literal) => {{
        #[cfg(feature = "seam-count")]
        $crate::core::seam::hit(concat!(module_path!(), "::", $tag));
    }};
    ($tag:literal, $cond:expr) => {{
        #[cfg(feature = "seam-count")]
        if $cond {
            $crate::core::seam::hit(concat!(module_path!(), "::", $tag));
        }
    }};
}

#[cfg(feature = "seam-count")]
pub use counter::*;

#[cfg(feature = "seam-count")]
mod counter {
    use std::cell::RefCell;
    use std::collections::BTreeMap;

    /// One hook's record over a walk: the first state it was entered on and
    /// how many times.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Entered {
        pub first: i32,
        pub count: u64,
    }

    struct Record {
        state: i32,
        hits: BTreeMap<&'static str, Entered>,
    }

    thread_local! {
        static RECORD: RefCell<Option<Record>> = const { RefCell::new(None) };
    }

    /// Start a fresh record on this thread, at state 0 (setup).
    pub fn start() {
        RECORD.with(|r| {
            *r.borrow_mut() = Some(Record {
                state: 0,
                hits: BTreeMap::new(),
            })
        });
    }

    /// The state the hooks entered from now on are counted against: the walk
    /// sets it to the state the next tick produces.
    pub fn at(state: i32) {
        RECORD.with(|r| {
            if let Some(rec) = r.borrow_mut().as_mut() {
                rec.state = state;
            }
        });
    }

    /// Record one entry of the named hook, when a record is started.
    pub fn hit(name: &'static str) {
        RECORD.with(|r| {
            if let Some(rec) = r.borrow_mut().as_mut() {
                let state = rec.state;
                rec.hits
                    .entry(name)
                    .and_modify(|e| e.count += 1)
                    .or_insert(Entered {
                        first: state,
                        count: 1,
                    });
            }
        });
    }

    /// End this thread's record: every hook entered, sorted by its first
    /// state, then by name.
    pub fn take() -> Vec<(&'static str, Entered)> {
        let hits = RECORD
            .with(|r| r.borrow_mut().take())
            .map(|rec| rec.hits)
            .unwrap_or_default();
        let mut out: Vec<_> = hits.into_iter().collect();
        out.sort_by_key(|(name, e)| (e.first, *name));
        out
    }
}

#[cfg(all(test, feature = "seam-count"))]
mod tests;
