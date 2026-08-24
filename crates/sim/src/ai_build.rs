//! Buildings — `Leader::create_buildings@006c1be0`,
//! `Leader::produce_upgrade@006cb5d0`, `Leader::produce_spell@006ca720`;
//! the specification is `~/ghidra-projects/reports/ai/create-buildings.md`
//! §1–§3, §5–§7 (ratified in `docs/AI.md` §11). `produce_building` itself
//! is [`crate::ai_place`].

use crate::tech::TypeId;
use crate::{Player, Sim};

impl Sim {
    /// `create_buildings`: every building type the leader could place,
    /// valued per city and offered to the make list.
    pub fn create_buildings(&mut self, who: Player) {
        // STUB — the buildings worker fills this in.
        let _ = who;
    }

    /// `produce_upgrade(t, city, escrow)`: `true` when queued.
    pub fn produce_upgrade(
        &mut self,
        who: Player,
        t: TypeId,
        city: Option<usize>,
        escrow: i32,
    ) -> bool {
        // STUB — the buildings worker fills this in.
        let _ = (who, t, city, escrow);
        false
    }

    /// `produce_spell(t, city, escrow)`: `true` when cast.
    pub fn produce_spell(
        &mut self,
        who: Player,
        t: TypeId,
        city: Option<usize>,
        escrow: i32,
    ) -> bool {
        // STUB — the buildings worker fills this in.
        let _ = (who, t, city, escrow);
        false
    }
}
