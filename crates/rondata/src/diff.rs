//! The gamelog diff: the original's per-frame dump against the simulation.
//!
//! Phase 3's score is ticks before divergence. This is the harness that
//! produces it: [`build_sim`] stands a `sim::Sim` up from a start-of-game
//! dump ([`crate::gamelog::Initial`]) with the loaded types
//! ([`crate::load::Loaded`]), and [`run`] steps it frame by frame against the
//! dump's `FRAME n` blocks, comparing what both sides hold.
//!
//! # What it can see today, and what it cannot
//!
//! At detail level 0 the original writes, per unit per frame, the object base
//! — owner, object number, `x_internal`/`y_internal`/`z_internal` — and per
//! leader its `score` and flags. So the comparison is **positions of units
//! keyed by `(who, o)`**, and the score is reported but not matched (the
//! simulation has no score yet). The per-frame `GUY` blocks are empty, the
//! leaders carry no goods, and buildings are not written per frame under the
//! categories the logged runs enabled; `DUMP_ALL=1` is the untried lever for
//! more (`docs/ORACLE.md`).
//!
//! And the simulation has no AI and sees no orders: in the logged runs the
//! human's units stand still and the AI's move as soon as it decides to, so
//! the honest expectation is that player 0 matches for as long as the human
//! did nothing and the AI player diverges the frame its first order lands.
//! That is still worth running — it proves the ids, the types, the world
//! scale and the step cadence agree end to end — and it is the scaffold the
//! order stream plugs into when the recorded-game container is read.

use crate::gamelog::{Frame, Initial, Log, Pos as LogPos, UnitDump};
use crate::load::Loaded;
use sim::{Pos, Sim, Tuning, Unit, World};

mod build;
mod city;
mod corrections;
mod endpoint;
mod floors;
#[cfg(test)]
mod golden;
mod harness;
#[cfg(test)]
mod leader;
mod order;
mod report;
#[cfg(test)]
mod second;
mod setup;
mod shutdown;
#[cfg(test)]
mod slots;
mod unit;

#[cfg(test)]
mod ammo;
#[cfg(test)]
mod army;
#[cfg(test)]
mod coverage;
#[cfg(test)]
pub(crate) mod testkit;
#[cfg(test)]
mod world;

/// **The compared recorder** — parked 527, built in the fourteenth Fable
/// pass (2026-09-25). `gamelog::reads` says which keys a parse *asked a
/// block for*; the coverage pin holds that against the dump's own keys and
/// stops at "parsed". Five landings in two tranches turned on the gap
/// after it: a field parsed and never compared (`o_up`, read once at
/// stand-up, 523), units linked on `(who, o)` with their numbers never
/// compared (617), a building the dump held and the crate counted rather
/// than noted (644), a comparison against a literal (661), a widening
/// silent on the word's own block (642). A read is not a comparison, and
/// nothing measured the difference.
///
/// This records the other half: every site of the shared instrument —
/// [`harness::compare`], [`order::compare_orders`] and
/// [`harness::widen_block`] — notes the `Record.field` it compared with
/// **both sides present**, and `coverage`'s pin holds the parser's own
/// records against what arrived. A field whose comparison is gated on
/// the position, the detail level or the other side's presence registers
/// only on the frames it actually ran, which is the point.
///
/// Recording is per thread and off unless a test starts it, so the walk
/// pays one thread-local check per site.
pub(crate) mod compared {
    use std::cell::RefCell;
    use std::collections::BTreeSet;

    thread_local! {
        static ON: RefCell<Option<BTreeSet<String>>> = const { RefCell::new(None) };
    }

    /// Starts recording on this thread; a recording already open is
    /// dropped.
    #[cfg(test)]
    pub(crate) fn start() {
        ON.with(|r| *r.borrow_mut() = Some(BTreeSet::new()));
    }

    /// Stops recording and hands back every `Record.field` noted since
    /// [`start`].
    #[cfg(test)]
    pub(crate) fn stop() -> BTreeSet<String> {
        ON.with(|r| r.borrow_mut().take().unwrap_or_default())
    }

    /// Notes that `record`'s `fields` were compared with both sides
    /// present. Nothing happens when no recording is open.
    pub(crate) fn note(record: &str, fields: &[&str]) {
        ON.with(|r| {
            if let Some(s) = r.borrow_mut().as_mut() {
                for f in fields {
                    s.insert(format!("{record}.{f}"));
                }
            }
        });
    }
}

pub use build::*;
pub use city::*;
pub use corrections::*;
pub use endpoint::*;
pub use floors::*;
pub use harness::*;
pub use order::*;
pub use report::*;
pub use setup::*;
pub use shutdown::*;
pub use unit::*;
