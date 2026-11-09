//! **The free runner** (`docs/RENDERER.md` §2.4, item 4020): a game stepped
//! from its start with nothing installed and nothing compared, handing a
//! client one [`Scene`] a tick.
//!
//! The diff harness (`diff::run_traced_observed`) steps the same simulation,
//! but it reads every frame of the dump to compare against and installs the
//! original's sync word and figure clocks wherever a capture traced them. A
//! client has no dump. So this runner:
//!
//! - builds the `Sim` with [`crate::diff::build_sim`] from the **setup
//!   blocks** of the captures alone — the start dump, its siblings, and the
//!   pasture's animals from the trace — and never reads a frame;
//! - each tick applies the order stream, when there is one
//!   ([`crate::input::Stream::apply`]), and calls `Sim::tick` through the
//!   order door (`sim::command`);
//! - reads the state through the read door (`sim::observe`) and nothing
//!   else.
//!
//! The setup door is reached only inside `build_sim`. A runner that read
//! `Sim`'s fields would have rebuilt the coupling item 4017's doors removed,
//! so the frame count is the runner's own.
//!
//! On a pair the game is uncorrected from frame 1 (`docs/DECISIONS.md` entry
//! 100 §4), so every scene this runner hands out must equal the harness's on
//! the same frame. `tests::the_free_run_s_scenes_are_the_harness_s` holds
//! that on every tick of a whole game.

use crate::capture::indexed::IndexedCapture;
use crate::diff::{Built, borrow_from_siblings, borrow_pasture, build_sim};
use crate::gamelog::{Initial, Log};
use crate::input::Stream;
use crate::load::Loaded;
use sim::Tuning;
use sim::observe::Scene;
use std::path::Path;

/// A game stepped from its start, with no dump past the setup block.
pub struct FreeRunner {
    built: Built,
    stream: Option<Stream>,
    /// Ticks run. After `n` ticks the state is the dump's `FRAME n`.
    frame: i64,
}

impl FreeRunner {
    /// A runner over the game `init` starts. `init` is a start dump's
    /// `Initial`, with whatever its siblings lend it already borrowed.
    pub fn new(loaded: &Loaded, init: &Initial<'_>, tuning: Tuning) -> Self {
        Self {
            built: build_sim(loaded, init, tuning),
            stream: None,
            frame: 0,
        }
    }

    /// A runner over a capture's game: `long`'s setup block, each
    /// `sibling`'s (the `DUMP_ALL` start a pair is stood up from) with what
    /// its frames print of the start — the figures' clocks and the
    /// animation lengths, the inputs `diff::borrow_from_siblings` lends —
    /// and the pasture's animals from `trace` (`diff::borrow_pasture`; no
    /// dump prints owner 9). No frame of `long` is read.
    pub fn from_captures(
        loaded: &Loaded,
        long: &Path,
        siblings: &[&Path],
        trace: Option<&crate::trace::Trace>,
        tuning: Tuning,
    ) -> std::io::Result<Self> {
        let text = IndexedCapture::open(long)?.read_replay_setup()?;
        let log = Log::parse(&text);
        with_sibling_initials(siblings, &[], &mut |refs| {
            let mut init = log
                .replay_initial()
                .ok_or_else(|| std::io::Error::other("no initial state"))?;
            borrow_from_siblings(&mut init, refs);
            if let Some(tr) = trace {
                borrow_pasture(&mut init, tr);
            }
            Ok(Self::new(loaded, &init, tuning))
        })
    }

    /// The order stream a recorded game carries, applied before each tick.
    pub fn with_stream(mut self, stream: Stream) -> Self {
        self.stream = Some(stream);
        self
    }

    /// Ticks run so far: the state is the dump's `FRAME n`.
    pub fn frame(&self) -> i64 {
        self.frame
    }

    /// One tick: the stream's orders for this frame, then `Sim::tick`.
    pub fn step(&mut self) -> Vec<sim::Tick> {
        if let Some(s) = &mut self.stream {
            // The harness's frame convention (`diff::Replay::step`): the
            // package for recording frame `n` is processed by the tick that
            // produces `FRAME n + 1`.
            s.apply(self.frame as i32, &mut self.built);
        }
        let ticks = self.built.sim.command().tick();
        self.frame += 1;
        ticks
    }

    /// This tick's drawable state as `viewer` sees it (`Sim::observe`).
    pub fn scene(&self, viewer: Option<sim::Player>) -> Scene {
        self.built.sim.observe(viewer)
    }

    /// The terrain grids as they stand (`Sim::observe_terrain`).
    pub fn terrain(&self) -> sim::observe::TerrainScene {
        self.built.sim.observe_terrain()
    }

    /// Step to `FRAME last`, handing each tick's scene to `see`.
    pub fn run_to(&mut self, last: i64, viewer: Option<sim::Player>, mut see: impl FnMut(&Scene)) {
        while self.frame < last {
            self.step();
            see(&self.scene(viewer));
        }
    }
}

/// Each sibling's `Initial` with its frames' observations, scoped to `f`.
fn with_sibling_initials<R>(
    paths: &[&Path],
    inits: &[&Initial<'_>],
    f: &mut dyn FnMut(&[&Initial<'_>]) -> std::io::Result<R>,
) -> std::io::Result<R> {
    let Some((path, rest)) = paths.split_first() else {
        return f(inits);
    };
    IndexedCapture::open(path)?.with_replay_initial(|init| {
        let mut refs = inits.to_vec();
        refs.push(&init);
        with_sibling_initials(rest, &refs, f)
    })?
}

#[cfg(test)]
mod tests;
