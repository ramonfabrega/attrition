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
        if self.trace.trace_phases {
            self.trace
                .phase_marks
                .push((phase.to_string(), self.rng.seed));
        }
    }
}

/// The `trace/` subsystem's fields of [`crate::Sim`] (`docs/STRUCTURE.md`
/// §3, item 4016): moved whole, each with its doc, in `Sim`'s order.
#[derive(Clone, Debug)]
pub struct TraceState {
    /// Draw marks spent by **staged input** at `Game::do_frame`'s entry,
    /// before the frame's first phase — the cheat channel's, which
    /// `rontrace.dll` hands to `ConsoleWin::parse_cmd` there
    /// (`docs/INPUT.md` §11). [`Sim::tick`] clears [`Sim::phase_marks`] at
    /// its head, so a staged draw marked before the call would vanish from
    /// the fold while still moving the stream; these are moved to the head
    /// of the frame's own marks instead, which is where the original
    /// spends them. `crate::golden::Script::stage` is the only writer.
    pub staged_marks: Vec<(String, u32)>,
    /// Whether [`Sim::tick`] records [`Sim::phase_marks`]. Off by default:
    /// the harness turns it on, the soak pays nothing.
    pub trace_phases: bool,
    /// Whether the search records [`Sim::cost_marks`]. Off by default, and
    /// the reason it exists is that the original can now answer the same
    /// question: `rontrace.cfg`'s `callwin` proxies
    /// `PathFinder::calc_cost` and logs its arguments **and its answer**
    /// (`docs/PATHFINDER.md` §10).
    pub trace_costs: bool,
    /// Every step the search priced, in order, while [`Sim::trace_costs`]
    /// is set. Nothing in the simulation reads it.
    pub cost_marks: Vec<crate::path::CostMark>,
    /// The same for the **road** search, filled under the same flag:
    /// `PathFinder::calc_road_cost`'s arguments and answer, node for node
    /// (`docs/ROADS.md` §7.2). Nothing in the simulation reads it.
    pub road_marks: Vec<crate::roads::RoadCostMark>,
    /// The stream's word at each phase boundary of [`Sim::tick`], filled
    /// only while [`Sim::trace_phases`] is set. It is what
    /// `tools/trace/report.py sites` is for the original — a frame's draws
    /// attributed, here to a phase and a unit rather than a return address
    /// — and comparing the two folds is how the frame-0 gap was taken
    /// apart (`docs/SYNC.md` §4.2).
    pub phase_marks: Vec<(String, u32)>,
    /// How many [`Sim::ground_z`] reads have touched a height corner that
    /// is not known to be the original's own single — one whose six-decimal
    /// print a neighbouring single shares, or one a terraform rewrote in
    /// millionths (`docs/COMBAT.md` §46.3). Every window the harness replays
    /// pins it, so the day a read lands on one is a failing test by name.
    pub ground_inexact: u32,
    /// Buildings disbanded or died this frame, for tests.
    pub removed: Vec<usize>,
    /// **The run116 comparison's recorder** (`docs/COLLISION.md` §9.6),
    /// and `None` in every run but a test's.
    ///
    /// A sweep's four steps are only comparable **where the original's
    /// own bracket sits** — mid-frame, after the units that step earlier
    /// have moved the occupancy index under it. Read at a frame boundary
    /// instead, this crate's probe on Great Lakes 10161 answers a
    /// different cell and names a different mechanism (§9.2), so the
    /// comparison cannot be a replay-then-ask. This is the shape
    /// [`ai::Leader::unit_offers`] takes for `docs/AI.md` §52's offers:
    /// the simulation fills a recorder as it runs and the test reads it
    /// afterwards.
    pub sweep_watch: Option<crate::collide::SweepWatch>,
    /// A unit cell `valid_ucoord` refuses to one unit on one frame
    /// ([`path::RefuseProbe`]), and `None` in every run but a test's.
    #[doc(hidden)]
    pub probe_refuse: Option<crate::path::RefuseProbe>,
    /// The order a frame's building walk ran its five steps in — written
    /// only under test — for the test that holds `Build::process`'s sequence (group 18).
    #[allow(dead_code)]
    pub(crate) building_log: Vec<(usize, &'static str)>,
}
