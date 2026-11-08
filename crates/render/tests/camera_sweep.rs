//! The camera in register: this crate's projection against the original's
//! own `Camera::update_transforms` and `MiscAccess::world_to_screen`, run
//! under unicorn on synthesized camera states (`tools/render/camera_oracle.py`,
//! `docs/RENDERER.md` §3.4).
//!
//! The rows are derived from the user's executable and graphics DLL and stay
//! outside the repository, as every emulator table does: the oracle runs
//! against `$RON_INSTALL` through `tools/emu`'s project (its harness is
//! imported, never edited), and with no install or no `uv` the test skips
//! and says so.

mod common;

/// The sweep's seed, states and points a state.
const SEED: u32 = 4018;
const STATES: usize = 240;
const POINTS: usize = 40;

/// Camera states in register — every point within one pixel on both axes
/// and the off-screen flag equal — of the sweep's `STATES`. Measured by this
/// test on its first run against the original (item 4018, 2026-11-08):
/// 240 of 240, 9,600 of 9,600 points, every `to_screen` element
/// bit-identical. With the negative-angle reduction read as `360 − (−a mod
/// 360)` instead of the original's `359 − …` it read 160 of 240 (7,574
/// points): the guard failing on purpose.
const IN_REGISTER: usize = 240;

#[test]
fn the_camera_is_in_register_with_the_original() {
    let Some(install) = common::install() else {
        return;
    };
    let exe = install.join("riseofnations.exe");
    let tail = [
        "sweep".to_string(),
        SEED.to_string(),
        STATES.to_string(),
        POINTS.to_string(),
    ];
    let Some(rows) = common::oracle("camera_oracle.py", &[&exe], &tail) else {
        return;
    };
    let states = common::parse(&rows);
    assert_eq!(states.len(), STATES, "the oracle printed every state");
    for (i, s) in states.iter().enumerate() {
        assert_eq!(s.points.len(), POINTS, "state {i} has every point");
    }
    let score = common::in_register(&states);
    eprintln!(
        "camera states in register: {} of {STATES}; points {} of {}; worst to_screen element {:e} relative",
        score.states, score.points, score.of_points, score.worst_matrix
    );
    assert_eq!(
        score.states, IN_REGISTER,
        "re-pin IN_REGISTER with its delta in its comment"
    );
}
