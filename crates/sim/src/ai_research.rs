//! Research — `Leader::research_techs@006c6ba0` and
//! `Leader::produce_tech@006ca980`; `docs/AI.md` §2.14, §2.17.

use crate::tech::TypeId;
use crate::{Player, Sim};

impl Sim {
    /// `research_techs`: every eligible tech valued and offered to the make
    /// list. Draws from the sync stream for governments only (§2.14).
    pub fn research_techs(&mut self, who: Player) {
        // STUB — the research worker fills this in.
        let _ = who;
    }

    /// `produce_tech(t, escrow)`: `true` when queued (the original's 0).
    pub fn produce_tech(&mut self, who: Player, t: TypeId, escrow: i32) -> bool {
        // STUB — the research worker fills this in.
        let _ = (who, t, escrow);
        false
    }
}
