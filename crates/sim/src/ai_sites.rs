//! City sites and the city AI — `Leader::compute_sites@006cc950`,
//! `Leader::compute_site_stats@006cd040`, `Leader::found_cities@006c7a60`
//! and `Leader::produce_city@006cb120`; `docs/AI.md` §2.7, §2.13, §2.12,
//! §2.17.

use crate::tech::TypeId;
use crate::{Player, Sim};

impl Sim {
    /// `compute_sites(force)`: re-score the kept sites, sample every region
    /// where the leader has peasants, keep the best ten, rank them. Draws
    /// from the sync stream per large region (§2.7). `force` is the
    /// argument `place_city_with_cost` passes (1).
    pub fn compute_sites(&mut self, who: Player, force: bool) {
        // STUB — the sites worker fills this in.
        let _ = (who, force);
    }

    /// `found_cities`: the city AI — offers the best site to the make list
    /// and, under the city limit, buys it on the spot (§2.12).
    pub fn found_cities(&mut self, who: Player) {
        // STUB — the sites worker fills this in.
        let _ = who;
    }

    /// `produce_city(t, wx, wy, escrow)`: `true` when the site was placed
    /// and a citizen sent (the original's 0).
    pub fn produce_city(&mut self, who: Player, t: TypeId, wx: i32, wy: i32, escrow: i32) -> bool {
        // STUB — the sites worker fills this in.
        let _ = (who, t, wx, wy, escrow);
        false
    }
}
