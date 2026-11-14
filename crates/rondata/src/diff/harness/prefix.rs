//! A single movable replay prefix inside an explicit, immutable-fixture batch.
//! Never caches verdicts or comparison records; every requested block is compared.
//!
//! **Two holders** (item 4216, DECISIONS 132 §11). A [`Scope`] is one
//! thread's batch: the coverage walk opens one a walker and runs a module's
//! windows through it. The **kept walk** is the process's: a thread with no
//! batch of its own takes the state the last widening left, when
//! `RON_SHARE_WALKS=on` — which `tools/isolated_tests.py` says to a group
//! of one game's tests, and nothing else does. The rule is the same for
//! both — the same inputs by path, length and modification time, and a
//! first block strictly after the retained frame — and so is what is never
//! shared: a verdict, a comparison, a recorded field. What the kept walk
//! adds, because its state crosses from one test's thread to the next's:
//!
//! * **the stand-up's fixture requests travel with the state**, and a test
//!   that resumes it is credited with them ([`crate::testenv::replay`]), so
//!   the fixture audit's rows are those the test wrote standing the game up
//!   itself; its key asks the audit for nothing ([`crate::testenv::locate`]);
//! * **the ground guard is answered at the hand-back** by the test that
//!   walked ([`Built::answer_for_the_ground`]), and a kept state dropped on
//!   another test's thread answers nothing ([`Built::retire`]);
//! * a test that panics holding the state drops it, and the next stands
//!   its own up: nothing is inherited from a walk that did not finish.
use crate::diff::Built;
use crate::testenv::Tape;
use std::cell::RefCell;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::SystemTime;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Key {
    install: PathBuf,
    inputs: Vec<Option<(PathBuf, u64, SystemTime)>>,
}
impl Key {
    /// A batch's key requests each input, as it always has; the kept
    /// walk's only locates them, so a resuming test's audit rows are its
    /// stand-up's and no more.
    pub(super) fn new(install: &std::path::Path, names: &[&str]) -> Self {
        let batch = BATCH.with(|batch| batch.borrow().active);
        let find = if batch {
            crate::testenv::dump
        } else {
            crate::testenv::locate
        };
        Self {
            install: install.to_owned(),
            inputs: names
                .iter()
                .map(|name| {
                    find(name).map(|path| {
                        let meta = std::fs::metadata(&path).expect("finalized replay input");
                        (PathBuf::from(path), meta.len(), meta.modified().unwrap())
                    })
                })
                .collect(),
        }
    }
}
#[derive(Default)]
struct Batch {
    active: bool,
    prefix: Option<(Key, Built)>,
    /// The state the last walk started from (Loop 3382): a later walk on
    /// the same inputs whose first block falls inside that walk resumes
    /// here instead of ticking again from frame zero.
    start: Option<(Key, Built)>,
    reused_ticks: i64,
    loaded: Option<(PathBuf, std::sync::Arc<crate::load::Loaded>)>,
}
thread_local! {
    static BATCH: RefCell<Batch> = RefCell::new(Batch::default());
    /// The tape of the stand-up behind the state this thread is walking,
    /// between [`take`] or [`stand_up`] and the hand-back.
    static IN_HAND: RefCell<Option<Tape>> = const { RefCell::new(None) };
}

/// The process's kept walk: the state the last widening handed back, the
/// state that widening started from, each with its stand-up's tape, and
/// the loaded install.
#[derive(Default)]
struct Kept {
    prefix: Option<(Key, Built, Tape)>,
    start: Option<(Key, Built, Tape)>,
    loaded: Option<(PathBuf, std::sync::Arc<crate::load::Loaded>)>,
}
static KEPT: Mutex<Option<Kept>> = Mutex::new(None);

fn kept<T>(with: impl FnOnce(&mut Kept) -> T) -> T {
    // A test that panicked inside left nothing half-written: every slot is
    // whole or empty.
    let mut guard = KEPT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    with(guard.get_or_insert_with(Kept::default))
}

/// Whether this process shares a kept walk between its tests.
fn sharing() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| {
        let asked = match std::env::var("RON_SHARE_WALKS").as_deref() {
            Ok("on") => true,
            Ok("off") | Err(std::env::VarError::NotPresent) => false,
            other => panic!("RON_SHARE_WALKS must be on or off, not {other:?}"),
        };
        asked && !diagnostic() && std::env::var_os("RON_DISABLE_PREFIX_REUSE").is_none()
    })
}

/// Diagnostic runs promise a complete from-zero trace of watched state.
fn diagnostic() -> bool {
    std::env::vars_os().any(|(key, _)| {
        let key = key.to_string_lossy();
        key.starts_with("RON_DEBUG")
            || matches!(
                key.as_ref(),
                "RON_SWEEP" | "RON_ROW_WALK" | "RON_SITE_SET" | "RON_STANDING"
            )
    })
}

/// Whether this thread's walks ride the process's kept states: a group's
/// process, outside a batch of its own ([`super::trunk`]).
pub(super) fn kept_walks_shared() -> bool {
    sharing() && !batch_active()
}

fn batch_active() -> bool {
    BATCH.with(|batch| batch.borrow().active)
}

fn retire(slot: Option<(Key, Built, Tape)>) {
    if let Some((_, built, _)) = slot {
        built.retire();
    }
}

/// Stand a state up with `build`, keeping the fixture requests it makes
/// when the kept walk may later hand the state to another test.
pub(super) fn stand_up(build: impl FnOnce() -> Built) -> Built {
    if batch_active() || !sharing() {
        return build();
    }
    let (built, tape) = crate::testenv::taped_requests(build);
    IN_HAND.with(|hand| *hand.borrow_mut() = Some(tape));
    built
}

/// Inputs, install and tuning remain immutable until this guard drops.
/// Only one Built is retained, moved into the next walk (never cloned).
pub(crate) struct Scope(std::marker::PhantomData<std::rc::Rc<()>>);
impl Scope {
    pub(crate) fn enter() -> Self {
        BATCH.with(|batch| {
            let mut batch = batch.borrow_mut();
            assert!(!batch.active, "nested replay prefix batch");
            batch.active = !diagnostic() && std::env::var_os("RON_DISABLE_PREFIX_REUSE").is_none();
        });
        Self(std::marker::PhantomData)
    }
}
impl Drop for Scope {
    fn drop(&mut self) {
        BATCH.with(|batch| {
            let old = std::mem::take(&mut *batch.borrow_mut());
            if old.active {
                eprintln!(
                    "replay prefix batch: {} repeated ticks avoided",
                    old.reused_ticks
                );
            }
        });
    }
}

pub(super) fn enabled() -> bool {
    batch_active() || sharing()
}

pub(super) fn loaded(install: &crate::Install) -> std::sync::Arc<crate::load::Loaded> {
    if !batch_active() {
        assert!(sharing());
        // Loaded outside the lock: a second test of the process waits on
        // nothing but its own read.
        if let Some(loaded) = kept(|kept| {
            kept.loaded
                .as_ref()
                .filter(|(path, _)| path == install.root())
                .map(|(_, loaded)| loaded.clone())
        }) {
            return loaded;
        }
        let loaded = std::sync::Arc::new(crate::load::load(install).unwrap());
        kept(|kept| kept.loaded = Some((install.root().to_owned(), loaded.clone())));
        return loaded;
    }
    BATCH.with(|batch| {
        let mut batch = batch.borrow_mut();
        assert!(batch.active);
        if let Some((path, loaded)) = &batch.loaded
            && path == install.root()
        {
            return loaded.clone();
        }
        let loaded = std::sync::Arc::new(crate::load::load(install).unwrap());
        batch.loaded = Some((install.root().to_owned(), loaded.clone()));
        loaded
    })
}

/// The kept walk's state for `key`, when it stands before `first_block`:
/// the state the last widening handed back, else the state that widening
/// started from. A state of other inputs, or one at or past the block, is
/// retired: one game is kept, and the next hand-back replaces it.
fn take_kept(key: &Key, first_block: i64) -> Option<Built> {
    let (found, stale) = kept(|kept| {
        let fits = |slot: &Option<(Key, Built, Tape)>| {
            slot.as_ref().is_some_and(|(previous, built, _)| {
                previous == key && built.sim.frame < first_block
            })
        };
        if fits(&kept.prefix) {
            (kept.prefix.take(), None)
        } else if fits(&kept.start) {
            (kept.start.take(), kept.prefix.take())
        } else {
            (None, kept.prefix.take())
        }
    });
    retire(stale);
    let (_, built, tape) = found?;
    // This test stood nothing up: the stand-up's requests are its rows.
    crate::testenv::replay(&tape);
    IN_HAND.with(|hand| *hand.borrow_mut() = Some(tape));
    Some(built)
}

pub(super) fn take(key: &Key, first_block: i64) -> Option<Built> {
    if !batch_active() {
        return sharing().then(|| take_kept(key, first_block)).flatten();
    }
    BATCH.with(|batch| {
        let mut batch = batch.borrow_mut();
        if !batch.active {
            return None;
        }
        // Incompatible or overlapping walks fall back to the last walk's
        // own starting state, and past that to a fresh start.
        let fits = |slot: &Option<(Key, Built)>| {
            slot.as_ref()
                .is_some_and(|(previous, built)| previous == key && built.sim.frame < first_block)
        };
        let slot = if fits(&batch.prefix) {
            &mut batch.prefix
        } else if fits(&batch.start) {
            &mut batch.start
        } else {
            batch.prefix = None;
            return None;
        };
        let (_, built) = slot.take()?;
        batch.prefix = None;
        batch.reused_ticks += built.sim.frame;
        Some(built)
    })
}
/// Keep a copy of the state a walk starts from, for [`take`]'s fallback.
pub(super) fn mark_start(key: &Key, built: &Built) {
    if !batch_active() {
        if sharing() {
            let tape = IN_HAND
                .with(|hand| hand.borrow().clone())
                .unwrap_or_default();
            let copy = (key.clone(), built.clone(), tape);
            retire(kept(|kept| kept.start.replace(copy)));
        }
        return;
    }
    BATCH.with(|batch| {
        let mut batch = batch.borrow_mut();
        if batch.active {
            batch.start = Some((key.clone(), built.clone()));
        }
    });
}
pub(super) fn put(key: Key, built: Built) {
    if !batch_active() {
        if sharing() {
            // The walk is this test's to answer for; the state is the
            // process's from here.
            built.answer_for_the_ground();
            let tape = IN_HAND
                .with(|hand| hand.borrow_mut().take())
                .unwrap_or_default();
            retire(kept(|kept| kept.prefix.replace((key, built, tape))));
        }
        return;
    }
    BATCH.with(|batch| {
        let mut batch = batch.borrow_mut();
        if batch.active {
            batch.prefix = Some((key, built));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scope() -> Scope {
        let scope = Scope::enter();
        // Exercise reuse itself even in a process benchmarking the disabled path.
        BATCH.with(|batch| batch.borrow_mut().active = true);
        scope
    }
    fn key(path: &str) -> Key {
        Key {
            install: path.into(),
            inputs: vec![],
        }
    }
    fn built(frame: i64) -> Built {
        let mut sim = sim::Sim::new(
            sim::core::tuning::Tuning::RON,
            sim::world::World::new(2, 2),
            2,
        );
        sim.frame = frame;
        Built {
            sim,
            gaia_reseat_skip: None,
            correction_audit: None,
            units: vec![],
            builds: vec![],
            region_map: vec![],
            notes: vec![],
            frame_seeds: vec![(1, 42)],
            rng_frames: vec![],
            frame_guys: vec![],
            frame_sites: vec![],
            type_index: vec![],
            unit_type_names: vec![],
            unit_tree: vec![],
            staged: None,
        }
    }
    #[test]
    fn prefixes_require_identical_inputs_and_an_uncompared_future_block() {
        let _scope = scope();
        put(key("a"), built(10));
        assert!(take(&key("b"), 11).is_none());
        put(key("a"), built(10));
        assert!(take(&key("a"), 10).is_none());
        put(key("a"), built(10));
        let resumed = take(&key("a"), 11).unwrap();
        assert_eq!(resumed.sim.frame, 10);
        assert_eq!(resumed.frame_seeds, [(1, 42)]);
        assert!(take(&key("a"), 11).is_none(), "state moves, never aliases");
        let mut changed = key("a");
        changed
            .inputs
            .push(Some(("capture".into(), 12, SystemTime::UNIX_EPOCH)));
        put(changed, built(10));
        assert!(take(&key("a"), 11).is_none());
    }
    #[test]
    fn an_overlapping_walk_resumes_from_the_last_walk_s_start() {
        let _scope = scope();
        mark_start(&key("a"), &built(5));
        put(key("a"), built(10));
        // Block 8 lies inside the last walk: its start, frame 5, serves it.
        assert_eq!(take(&key("a"), 8).unwrap().sim.frame, 5);
        assert!(take(&key("a"), 11).is_none(), "both slots are spent");
        mark_start(&key("a"), &built(5));
        put(key("a"), built(10));
        assert!(take(&key("a"), 5).is_none(), "the start is no earlier");
        mark_start(&key("a"), &built(5));
        assert!(take(&key("b"), 8).is_none(), "other inputs never resume");
    }
    #[test]
    fn dropping_a_batch_discards_its_state_even_on_unwind() {
        let panic = std::panic::catch_unwind(|| {
            let _scope = scope();
            put(key("a"), built(10));
            panic!("exercise guard cleanup");
        });
        assert!(panic.is_err());
        let _scope = scope();
        assert!(take(&key("a"), 11).is_none());
    }
}
