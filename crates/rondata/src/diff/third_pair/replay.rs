//! One movable Built for this test's immutable source pair and fixed RON tuning.
//! No comparison is cached. Overlap or an input identity change starts over.
use super::*;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Key {
    inputs: Vec<Option<(PathBuf, u64, SystemTime)>>,
    tail: i64,
}
impl Key {
    pub(super) fn new(start: &str, base: &str, trace: Option<&str>, tail: i64) -> Self {
        Self {
            inputs: [Some(start), Some(base), trace]
                .into_iter()
                .map(|path| {
                    path.map(|p| {
                        let path = Path::new(p)
                            .canonicalize()
                            .expect("finalized replay source");
                        let meta = path.metadata().unwrap();
                        (path, meta.len(), meta.modified().unwrap())
                    })
                })
                .collect(),
            tail,
        }
    }
}

pub(super) fn compatible(old: &Key, frame: i64, key: &Key, first: i64) -> bool {
    old == key && frame < first
}

pub(super) fn enabled() -> bool {
    std::env::var_os("RON_DISABLE_THIRD_PAIR_REUSE").is_none()
        && std::env::var_os("RON_DISABLE_PREFIX_REUSE").is_none()
        && !std::env::vars_os().any(|(key, _)| {
            let key = key.to_string_lossy();
            key.starts_with("RON_DEBUG")
                || matches!(
                    key.as_ref(),
                    "RON_SWEEP" | "RON_ROW_WALK" | "RON_SITE_SET" | "RON_STANDING"
                )
        })
}

pub(super) fn build(
    loaded: &Loaded,
    start: &str,
    base: &str,
    trace_path: Option<&str>,
    tail: i64,
) -> Built {
    let mut start_source = crate::capture::indexed::IndexedCapture::open(start).unwrap();
    let start_text = start_source.read_replay_setup().unwrap();
    let start_log = Log::parse(&start_text);
    let mut sibling = start_log.initial().unwrap();
    let (seeds, animations, guys) = start_source
        .with_replay_initial(|observed| {
            (
                observed.frame_seeds.clone(),
                observed.anim_lengths.clone(),
                observed.frame_guys.clone(),
            )
        })
        .unwrap();
    sibling.frame_seeds = seeds;
    sibling.anim_lengths = animations;
    sibling.frame_guys = guys;
    // This widening stops at block `tail`, after sim frames
    // `0..tail`; later sibling observations cannot affect it.
    sibling.frame_seeds.retain(|(frame, _)| *frame < tail);
    sibling.frame_guys.retain(|(frame, _)| *frame < tail);

    let mut base_source = crate::capture::indexed::IndexedCapture::open(base).unwrap();
    let base_text = base_source.read_replay_setup().unwrap();
    let base_log = Log::parse(&base_text);
    let mut init = base_log.initial().unwrap();
    let (seeds, animations, guys) = base_source
        .with_replay_initial(|observed| {
            (
                observed.frame_seeds.clone(),
                observed.anim_lengths.clone(),
                observed.frame_guys.clone(),
            )
        })
        .unwrap();
    init.frame_seeds = seeds;
    init.anim_lengths = animations;
    init.frame_guys = guys;
    assert!(
        same_start(&init, &sibling),
        "all starting building identities and positions"
    );
    assert_eq!(init.game_info, sibling.game_info, "the same lobby");
    assert_eq!(
        init.checksums.last().unwrap().seed,
        sibling.checksums.last().unwrap().seed
    );
    borrow_from_siblings(&mut init, &[&sibling]);
    init.frame_seeds.retain(|(frame, _)| *frame < tail);
    init.frame_guys.retain(|(frame, _)| *frame < tail);
    if let Some(path) = trace_path {
        let trace = crate::trace::Trace::read(std::path::Path::new(path))
            .unwrap()
            .expect("missing RONT header");
        borrow_pasture(&mut init, &trace);
    }
    build_sim(loaded, &init, Tuning::RON)
}

/// Compare the full Debug representation of Built (including Sim, maps, notes,
/// RNG history and correction state), with only future correction inputs removed.
/// The retained input prefix itself must be exactly equal. No hashes or projections.
pub(super) fn assert_same(actual: &mut Built, fresh: &mut Built, tail: i64) {
    let seeds = std::mem::take(&mut actual.frame_seeds);
    let guys = std::mem::take(&mut actual.frame_guys);
    let fresh_seeds = std::mem::take(&mut fresh.frame_seeds);
    let fresh_guys = std::mem::take(&mut fresh.frame_guys);
    assert!(
        seeds
            .iter()
            .filter(|(n, _)| *n < tail)
            .eq(fresh_seeds.iter()),
        "seed prefix changed"
    );
    assert!(
        guys.iter().filter(|(n, _)| *n < tail).eq(fresh_guys.iter()),
        "clock prefix changed"
    );
    assert!(
        format!("{actual:?}") == format!("{fresh:?}"),
        "full Built differs at frame {}",
        actual.sim.frame
    );
    actual.frame_seeds = seeds;
    actual.frame_guys = guys;
    fresh.frame_seeds = fresh_seeds;
    fresh.frame_guys = fresh_guys;
}

#[test]
fn reuse_requires_same_inputs_horizon_and_a_future_window() {
    let key = |tail| Key {
        inputs: vec![None],
        tail,
    };
    assert!(compatible(&key(20), 10, &key(20), 11));
    assert!(!compatible(&key(20), 10, &key(20), 10));
    assert!(!compatible(&key(20), 10, &key(20), 9));
    assert!(!compatible(&key(20), 10, &key(21), 11));
    assert!(!compatible(
        &Key {
            inputs: vec![],
            tail: 20
        },
        10,
        &key(20),
        11
    ));
}

#[test]
fn identity_includes_path_size_time_and_missing_trace() {
    let make = |path: &str, size, modified| Key {
        inputs: vec![Some((path.into(), size, modified))],
        tail: 20,
    };
    let now = SystemTime::UNIX_EPOCH;
    let base = make("a", 1, now);
    for different in [
        make("b", 1, now),
        make("a", 2, now),
        make("a", 1, now + std::time::Duration::from_secs(1)),
        Key {
            inputs: vec![None],
            tail: 20,
        },
    ] {
        assert!(!compatible(&base, 10, &different, 11));
    }
}

#[test]
fn full_state_audit_accepts_only_future_inputs_and_rejects_value_changes() {
    let make = || Built {
        sim: sim::Sim::new(Tuning::RON, World::new(2, 2), 2),
        gaia_reseat_skip: None,
        correction_audit: None,
        units: vec![],
        builds: vec![],
        region_map: vec![],
        notes: vec![],
        frame_seeds: vec![(1, 42), (1, 43)],
        frame_guys: vec![(1, vec![])],
        rng_frames: vec![],
        frame_sites: vec![],
        type_index: vec![],
        unit_type_names: vec![],
        unit_tree: vec![],
    };
    let mut actual = make();
    let mut fresh = make();
    actual.frame_seeds.push((2, 99));
    actual.frame_guys.push((2, vec![]));
    assert_same(&mut actual, &mut fresh, 2);
    assert_eq!(actual.frame_seeds, [(1, 42), (1, 43), (2, 99)]);
    assert_eq!(actual.frame_guys.len(), 2, "audit restores future inputs");
    for change in 0..7 {
        let mut actual = make();
        let mut fresh = make();
        match change {
            0 => actual.sim.rng.seed ^= 1,
            1 => actual.rng_frames.push((1, Some(2), Some(3))),
            2 => actual.builds.push((1, 2)),
            3 => actual.notes.push("different correction".into()),
            4 => actual.frame_seeds.swap(0, 1),
            5 => actual.frame_guys.clear(),
            _ => actual.frame_sites.push((1, vec!["different site".into()])),
        }
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| assert_same(
                &mut actual,
                &mut fresh,
                2
            )))
            .is_err(),
            "audit missed mutation {change}"
        );
    }
}
