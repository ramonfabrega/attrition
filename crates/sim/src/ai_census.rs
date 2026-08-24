//! The census — `Leader::plan_strategy@006b9620`'s sweep after the cadence
//! check, `docs/AI.md` §2.3 steps 1–16: the recount of everything a
//! computer leader owns into [`crate::ai::Census`] and the per-city
//! [`crate::ai::CityAi`] records. The tail (step 17) — the orphan check,
//! `compute_sites`, arming the step machine — is the driver's.
//!
//! Oracle: a `LEADERS=9` dump prints every field under the same name
//! (`tools/gamelog/leader.py FRAME WHO`); run8's frame 1 is the AI leader
//! after its frame-0 sweep.

use crate::{Player, Sim};

impl Sim {
    /// The sweep, steps 1–16. Writes `self.ai[who].census` and
    /// `self.ai[who].city_ai`, and `invaders[who]` on every other leader.
    pub fn census(&mut self, who: Player) {
        // STUB — the census worker fills this in.
        let w = who as usize;
        let regions = self.world.region_count();
        let players = self.players.len();
        self.ai[w].census.resize(regions, players);
        let n = self.cities.len();
        self.ai[w].city_ai.resize(n, crate::ai::CityAi::default());
    }

    /// `Leader::check_explore@006bc860`: the explored-cell recount on its
    /// own cadence (`docs/AI.md` §2.1).
    pub fn check_explore(&mut self, who: Player) {
        // STUB — the census worker fills this in.
        let _ = who;
    }
}
