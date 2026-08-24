//! Units — `Leader::create_units@006c40a0`, `Leader::upgrade_units@006c6430`,
//! `Leader::produce_unit@006cb9e0`, `Leader::queued_units@006ce000`,
//! `Leader::check_income@006cc800`, `Leader::unit_prod_value@006cc580`;
//! the specification is `~/ghidra-projects/reports/ai/create-units.md`
//! (ratified in `docs/AI.md` §11).

use crate::tech::TypeId;
use crate::{Player, Sim};

impl Sim {
    /// `create_units`: every unit type the leader could train, valued and
    /// offered to the make list.
    pub fn create_units(&mut self, who: Player) {
        // STUB — the units worker fills this in.
        let _ = who;
    }

    /// `upgrade_units`: the unit upgrades, valued and offered.
    pub fn upgrade_units(&mut self, who: Player) {
        // STUB — the units worker fills this in.
        let _ = who;
    }

    /// `produce_unit(t, city, num, escrow)`: `true` when queued (the
    /// original's 0). `city` is a sim city index, `None` for −1.
    pub fn produce_unit(
        &mut self,
        who: Player,
        t: TypeId,
        city: Option<usize>,
        num: i32,
        escrow: i32,
    ) -> bool {
        // STUB — the units worker fills this in.
        let _ = (who, t, city, num, escrow);
        false
    }

    /// `queued_units`: the population the leader's queues will add.
    pub fn queued_units(&self, who: Player) -> i32 {
        // STUB — the units worker fills this in.
        let _ = who;
        0
    }

    /// `check_income(t, scale, city, flag, o, num, escrow)` as the readers
    /// have it: the value factor (0x100 = ×1) research and the producers
    /// multiply by; negative means "cannot".
    #[allow(clippy::too_many_arguments)]
    pub fn check_income(
        &self,
        who: Player,
        t: TypeId,
        scale: i32,
        city: Option<usize>,
        flag: bool,
        o: i32,
        num: i32,
        escrow: i32,
    ) -> i32 {
        // STUB — the units worker fills this in.
        let _ = (who, t, scale, city, flag, o, num, escrow);
        0x100
    }
}
