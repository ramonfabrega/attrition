//! `trace/` — the sync trace: [`Sim::mark`], the phase word the lockstep diff
//! compares (`docs/STRUCTURE.md` §3). The `SITE_*` draw-site constants it is
//! handed stay in each subsystem's own file, beside the draw they name.
//!
//! Moved out of `lib.rs` unchanged by item 4014 (`docs/STRUCTURE.md` §5, move 6).

use crate::*;

impl Sim {
    /// Records the stream's word *before* the phase named runs, so the
    /// draws between two marks belong to the earlier one. Nothing at all
    /// unless [`Sim::trace_phases`] is set.
    ///
    /// `Sim::tick` marks its phases and each unit the loop visits; a
    /// **mechanic** may mark its own draw sites on top of that, under the
    /// original's own offsets (`crate::scout` is the first), which is what
    /// turns the fold from a count into a sequence comparable with
    /// `rondata::trace`'s.
    pub(crate) fn mark(&mut self, phase: &str) {
        if self.trace_phases {
            self.phase_marks.push((phase.to_string(), self.rng.seed));
        }
    }
}
