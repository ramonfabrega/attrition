//! `Army::find_target@006f69b0`, **the forts pass** (`6f7f01..6f81ed`,
//! `docs/ARMY.md` §12; item 4341): after a leader's cities, the same
//! leader's forts, scored on the same `>=` into the same best.
//!
//! The listing, in order: the army's siege summed over its groups
//! (`GroupData::count(COUNT_SIEGE)`, `6f7f38`) must reach two and the
//! wagon count the head kept (`-0x54(%ebp)`, `count(NON_DECOY_TYPE,
//! SUPPLYWAGON)`) be non-zero; `L` is not me and is at war with me either
//! way (`diplos` 0 on either row, `6f7f89..6f7fa5`); then every fort record
//! below `L.fort_mark` (`0xe3a7b8`, `+0x428`) in use (`+6 & 1`) whose
//! region (`+4`, the building cell's region `Fort::init@0073eac0` stored)
//! is the army's (`+8`) draws once (`6f8026`, returning to `+0x167b`):
//!
//! ```text
//! v = (Random::get(0, 0xffff) % 200 + 900) × ((L.ages ^ 0x62766) + 1)
//! if the building's city (`BuildData +0x72`) < 0: v /= 2
//! for each of my 16 armies A, valid, not mustering, targeting (fort, L):
//!     A is this army → v ×= 2
//!     else → the building `is_under_attack` (Build `+0x60 & 0x20`,
//!            vslot `+0x188`) ? v ×= 4 : v /= 4
//! v /= vector_dist(army point − fort point) / 0x300 + 1
//! best on `>=`
//! ```

use crate::Player;
use crate::ai::army::{SITE_FIND_TARGET_FORT, status};
use crate::combat::Obj;
use crate::world::{UNITS_PER_CELL, vector_dist};

/// What the cities' pass leaves for the forts: the army's siege and wagons.
#[derive(Clone, Copy, Debug)]
pub(crate) struct FortsCtx {
    /// `GroupData::count(COUNT_SIEGE)` over the army's groups.
    pub siege: i32,
    /// `count(NON_DECOY_TYPE, SUPPLYWAGON)`, the head's.
    pub supply: i32,
}

impl crate::Sim {
    /// `L`'s fort buildings in fort-record order (`fort_slot`), the records
    /// below the mark and in use.
    pub(crate) fn forts_of(&self, l: Player) -> Vec<usize> {
        let mark = self
            .leaders
            .marks
            .get(usize::from(l))
            .map_or(0, |m| m.forts.mark);
        let mut v: Vec<(usize, usize)> = self
            .buildings
            .iter()
            .enumerate()
            .filter(|(_, bd)| bd.owner == l)
            .filter_map(|(b, bd)| bd.fort_slot.filter(|&s| s < mark).map(|s| (s, b)))
            .collect();
        v.sort_unstable();
        v.into_iter().map(|(_, b)| b).collect()
    }

    /// The forts pass for leader `l` (the module doc), folding into `best`,
    /// whose entries are `(v, building, leader)`.
    pub(crate) fn target_forts(
        &mut self,
        who: Player,
        slot: usize,
        l: Player,
        ctx: FortsCtx,
        best: &mut Option<(i32, usize, Player)>,
    ) {
        if ctx.siege < 2 || ctx.supply == 0 || l == who {
            return;
        }
        if !(self.at_war_with(who, l) || self.at_war_with(l, who)) {
            return;
        }
        let army_reg = self.ai.armies[usize::from(who)].list[slot]
            .reg
            .map_or(-1, i32::from);
        let ages = self.tech.get(usize::from(l)).map_or(0, |p| p.ages);
        for b in self.forts_of(l) {
            let pos = self.buildings[b].pos;
            let reg = self.world.region_of(pos.cell()).map_or(-1, i32::from);
            if reg != army_reg {
                continue;
            }
            self.mark(SITE_FIND_TARGET_FORT);
            let draw = self.rng.get(0, 0xffff) as i32;
            let mut v = (draw % 200 + 900).wrapping_mul(ages.wrapping_add(1));
            if self.buildings[b].city.is_none() {
                v /= 2;
            }
            let fort = Obj::Building(b);
            for j in 0..crate::ai::army::SLOTS {
                let a = &self.ai.armies[usize::from(who)].list[j];
                if !a.valid || a.status & status::MUSTERING != 0 || a.target != Some(fort) {
                    continue;
                }
                if j == slot {
                    v = v.wrapping_mul(2);
                } else if self.buildings[b].is_under_attack() {
                    v = v.wrapping_shl(2);
                } else {
                    v /= 4;
                }
            }
            let a = &self.ai.armies[usize::from(who)].list[slot];
            let d = vector_dist(a.pos.x - pos.x, a.pos.y - pos.y);
            v /= d / UNITS_PER_CELL + 1;
            if best.is_none_or(|(bv, _, _)| bv <= v) {
                *best = Some((v, b, l));
            }
        }
    }
}
