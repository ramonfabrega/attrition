//! **The last building and unit finished** — item 4187, step VI of the
//! leaders line (`docs/LEADERS.md` §19): `last_building_finished`
//! (`+0x6070`, `int[129]`) and `last_unit_finished` (`+0x6274`,
//! `int[352]`).
//!
//! Each cell is the object number `o` of the leader's last building of that
//! type to finish (by building record, `TypeIndex − 0x19e`) or of the last
//! unit of that type trained (by unit record, `TypeIndex − 0x32`). Both are
//! overwrites, never increments; `Leader::init` fills both with −1 for a
//! seated leader, and nothing in a plain game clears a cell again (the
//! trigger reader's read-and-clear is the trigger layer's, cut from v1).
//!
//! The events live where the original's do: `Build::activate@00623e20`'s
//! head (`crate::cities::activate`, `record_last_built`) for a building
//! activated uncaptured and counted, and `Build::train@0062f9b0`
//! ([`Sim::build_train`]) right after `Objects::init_unit` for a unit.
//! Neither leaf is printed by `LeaderData::log_data`: the checksum's leaders
//! line is their only oracle.

use crate::{Player, Sim};

/// `last_building_finished`'s width: the building records
/// `0x19e..0x21f`.
pub const FINISHED_BUILDINGS: usize = 129;
/// `last_unit_finished`'s width: the unit records `0x32..0x192`.
pub const FINISHED_UNITS: usize = 352;

/// One leader's two arrays.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finished {
    /// `last_building_finished`, by building record.
    pub building: Vec<i32>,
    /// `last_unit_finished`, by unit record.
    pub unit: Vec<i32>,
}

impl Default for Finished {
    /// `Leader::init`'s image for a seated leader: every cell −1.
    fn default() -> Finished {
        Finished {
            building: vec![-1; FINISHED_BUILDINGS],
            unit: vec![-1; FINISHED_UNITS],
        }
    }
}

/// The two leaves [`Sim::leader_finished_leaf`] answers.
pub const FINISHED_LEAVES: [&str; 2] = ["last_building_finished", "last_unit_finished"];

impl Sim {
    /// **`Build::activate`'s head store** (§19.2): a building activated
    /// with `captured == 0 && counted != 0` writes its `o` into its owner's
    /// cell of its type, the type read after the Chinese Large City
    /// branch's `set_type`.
    pub(crate) fn note_last_building(&mut self, b: usize) {
        let (who, ty, o) = (
            usize::from(self.buildings[b].owner),
            self.buildings[b].ty,
            self.buildings[b].index,
        );
        if let (Some(f), Some(ty)) = (self.leaders.finished.get_mut(who), ty)
            && let Some(cell) = f.building.get_mut(ty)
        {
            *cell = i32::from(o);
        }
    }

    /// **`Build::train`'s store** (§19.2): the trained type `ty` (the
    /// argument, never a type read back) takes the `o` `init_unit` returned,
    /// a squad's captain.
    pub(crate) fn note_last_unit(&mut self, who: Player, ty: usize, unit: usize) {
        let o = i32::from(self.units[unit].index);
        if let Some(f) = self.leaders.finished.get_mut(usize::from(who))
            && let Some(cell) = f.unit.get_mut(ty)
        {
            *cell = o;
        }
    }

    /// One of [`FINISHED_LEAVES`] as the record holds it.
    pub fn leader_finished_leaf(&self, who: Player, name: &str) -> Option<Vec<i64>> {
        let f = self.leaders.finished.get(usize::from(who))?;
        let v = match name {
            "last_building_finished" => &f.building,
            "last_unit_finished" => &f.unit,
            _ => return None,
        };
        Some(v.iter().map(|&x| i64::from(x)).collect())
    }
}

#[cfg(test)]
mod tests;
