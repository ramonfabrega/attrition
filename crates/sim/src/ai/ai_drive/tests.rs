use super::*;
use crate::{Tuning, World};

/// A computer leader with no script takes the frame-0 sweep, walks the
/// eleven steps one a frame and disarms; a human never enters.
#[test]
fn the_step_machine_runs_one_step_a_frame_from_the_sweep() {
    let mut sim = Sim::new(Tuning::RON, World::new(2, 2), 2);
    sim.cities_state.nation[1].human = false;
    sim.ai[1].script_live = false;
    let mut steps = Vec::new();
    for _ in 0..12 {
        sim.tick();
        steps.push(sim.ai[1].step.number());
    }
    // Frame 0 arms; frame 1 enters as Script, skips to Setup and runs
    // it (→ 3); then one producer a frame to Make (8), which disarms.
    assert_eq!(steps, [1, 3, 4, 5, 6, 7, 8, 0, 0, 0, 0, 0]);
    assert_eq!(sim.ai[0].step, Step::Idle, "a human is never stepped");
    // The next sweep for who = 1 is frame 175.
    for _ in 12..175 {
        sim.tick();
    }
    assert_eq!(sim.ai[1].step, Step::Idle);
    sim.tick();
    assert_eq!(sim.ai[1].step, Step::Script, "armed at frame 175");
}

/// **Run18b's two ladders** (`docs/RUNS.md`, "The producers' run").
/// The AI's `LEADERS=9` window holds both, and they differ by one frame:
///
/// - the sweep on which the script *ends* runs step 1 with the script
///   still live, so `SCRIPT_DONE` only advances to `Setup` — sim-frames
///   6375…6383 read `1, 2, 3, 4, 5, 6, 7, 8, 0` in the dump;
/// - every later sweep enters step 1 with the script dead, is promoted
///   to `Setup` *and runs it* in the same call — 6575…6582 read
///   `1, 3, 4, 5, 6, 7, 8, 0`, one frame shorter.
///
/// The step numbers here are the dump's `production_step` at the end of
/// each sim-frame, which is what `tools/gamelog/steps.py` prints.
#[test]
fn run18_s_two_ladders_differ_by_the_script_s_last_call() {
    // The transition sweep: the script is live, returns SCRIPT_DONE (3).
    let mut l = crate::ai::Leader::new();
    l.script_live = true;
    l.step = Step::Script;
    let mut ladder = vec![l.step.number()];
    l.enter(false);
    assert_eq!(l.step, Step::Script, "a live script is not skipped");
    l.after_script(crate::ai::ScriptResult::Returned(3));
    ladder.push(l.step.number());
    for _ in 0..6 {
        l.after_producer();
        ladder.push(l.step.number());
    }
    l.after_make(false, false);
    ladder.push(l.step.number());
    assert_eq!(
        ladder,
        [1, 2, 3, 4, 5, 6, 7, 8, 0],
        "sim-frames 6375..=6383"
    );

    // Every sweep after it: the same machine, one frame shorter.
    l.step = Step::Script;
    let mut ladder = vec![l.step.number()];
    l.enter(false);
    assert_eq!(l.step, Step::Setup, "a dead script is promoted in-call");
    l.after_producer(); // Setup runs on that same frame.
    ladder.push(l.step.number());
    for _ in 0..5 {
        l.after_producer();
        ladder.push(l.step.number());
    }
    l.after_make(false, false);
    ladder.push(l.step.number());
    assert_eq!(ladder, [1, 3, 4, 5, 6, 7, 8, 0], "sim-frames 6575..=6582");
}
