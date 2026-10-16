//! A single movable replay prefix inside an explicit, immutable-fixture batch.
//! Never caches verdicts or comparison records; every requested block is compared.
use crate::diff::Built;
use std::cell::RefCell;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Key {
    install: PathBuf,
    inputs: Vec<Option<(PathBuf, u64, SystemTime)>>,
}
impl Key {
    pub(super) fn new(install: &std::path::Path, names: &[&str]) -> Self {
        Self {
            install: install.to_owned(),
            inputs: names
                .iter()
                .map(|name| {
                    crate::testenv::dump(name).map(|path| {
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
    reused_ticks: i64,
    loaded: Option<(PathBuf, std::sync::Arc<crate::load::Loaded>)>,
}
thread_local! {
    static BATCH: RefCell<Batch> = RefCell::new(Batch::default());
}

/// Inputs, install and tuning remain immutable until this guard drops.
/// Only one Built is retained, moved into the next walk (never cloned).
pub(crate) struct Scope(std::marker::PhantomData<std::rc::Rc<()>>);
impl Scope {
    pub(crate) fn enter() -> Self {
        BATCH.with(|batch| {
            let mut batch = batch.borrow_mut();
            assert!(!batch.active, "nested replay prefix batch");
            // Diagnostic runs promise a complete from-zero trace of watched state.
            let diagnostic = std::env::vars_os().any(|(key, _)| {
                let key = key.to_string_lossy();
                key.starts_with("RON_DEBUG")
                    || matches!(
                        key.as_ref(),
                        "RON_SWEEP" | "RON_ROW_WALK" | "RON_SITE_SET" | "RON_STANDING"
                    )
            });
            batch.active = !diagnostic && std::env::var_os("RON_DISABLE_PREFIX_REUSE").is_none();
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
    BATCH.with(|batch| batch.borrow().active)
}

pub(super) fn loaded(install: &crate::Install) -> std::sync::Arc<crate::load::Loaded> {
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

pub(super) fn take(key: &Key, first_block: i64) -> Option<Built> {
    BATCH.with(|batch| {
        let mut batch = batch.borrow_mut();
        if !batch.active {
            return None;
        }
        // Incompatible or overlapping walks fall back to a fresh start.
        let (previous, built) = batch.prefix.take()?;
        if previous != *key || built.sim.frame >= first_block {
            return None;
        }
        batch.reused_ticks += built.sim.frame;
        Some(built)
    })
}
pub(super) fn put(key: Key, built: Built) {
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
        let mut sim = sim::Sim::new(sim::tuning::Tuning::RON, sim::world::World::new(2, 2), 2);
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
