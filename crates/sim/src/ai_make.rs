//! Spending from the make list — `Leader::make_stuff@006c8af0`,
//! `Leader::make_this@006c94f0`, the market (`Leader::use_market@006c91c0`,
//! `Leader::market_speculation@006c8110`) and the orphan check
//! (`Leader::check_orphaned_buildings@006c9f20`); `docs/AI.md` §2.6, §2.15,
//! §2.8.

use crate::{Player, Sim};

impl Sim {
    /// `make_stuff`: buy what the list and the stockpile allow; `true` when
    /// the head was bought (the original's 1). Draws from the sync stream
    /// per expiring duplicate (§2.6 step 4).
    pub fn make_stuff(&mut self, who: Player) -> bool {
        // STUB — the make worker fills this in.
        let _ = who;
        false
    }

    /// `make_this(slot)`: dispatch one slot to its producer; `true` when it
    /// bought.
    pub fn make_this(&mut self, who: Player, slot: usize) -> bool {
        // STUB — the make worker fills this in.
        let _ = (who, slot);
        false
    }

    /// `use_market`: one buy or one sell per short good, from `make_stuff`.
    pub fn use_market(&mut self, who: Player) {
        // STUB — the make worker fills this in.
        let _ = who;
    }

    /// `market_speculation`: `production_ai_setup`'s last act.
    pub fn market_speculation(&mut self, who: Player) {
        // STUB — the make worker fills this in.
        let _ = who;
    }

    /// `check_orphaned_buildings`: disband marked buildings and stranded
    /// sites, or send a builder.
    pub fn check_orphaned_buildings(&mut self, who: Player) {
        // STUB — the make worker fills this in.
        let _ = who;
    }
}
