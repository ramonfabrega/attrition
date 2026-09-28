//! The on-map heals of `Unit::process_healing@005e0670` — the civilian
//! heal alone so far. `docs/COMBAT.md` §72.
//!
//! The function's `inside_up ≥ 0` branch is the garrison heal
//! ([`crate::Sim::garrison_heal`], `docs/CITIES.md` §6.7). Its on-map
//! branch runs up to seven heals one after another, each phased on
//! `(o + frame) % period` and each ending in `repair_damage`. Six of them
//! need a nation bonus, a wonder, a general, a Conquer-the-World hero or
//! a supply period that ships as zero (`docs/SUPPLY.md`, "Consumer 2");
//! the seventh, the **civilian heal**, needs none of these, and it is the
//! one a capture reaches.

use crate::Sim;
use crate::ai_load::uflags2;
use crate::attrition::Domain;
use crate::world::Owner;

/// `ObjectData::is_worker@0046fa10`: the four citizen and scholar type
/// ids, by identity.
const WORKERS: std::ops::RangeInclusive<i32> = 0x32..=0x35;

/// The `FISHERMEN` type id, tested by identity here (`+4 != 0x13d`), not
/// by lineage.
const FISHERMEN: i32 = 0x13d;

/// `UnitData::unit_masks2` bit 0 — the unit was struck lately. It vetoes
/// the civilian heal (and forces a siege unit's slow reload,
/// `UnitData::recharge@0060fdf0`). `Unit::process@00610bc0`'s 32-frame
/// block decays it (`0x2` first, then `0x1`). **No setter is carried
/// here**: the decompile and a listing grep name none, and no dumped
/// unit on run356 or run373 holds `0x1` or `0x2` (`docs/COMBAT.md` §72.6).
pub const STRUCK: u32 = 0x1;

impl Sim {
    /// The civilian heal, the last arm of `process_healing`'s on-map
    /// branch (`005e0670`, `docs/COMBAT.md` §72.2):
    ///
    /// ```text
    /// captain, not air, has_damage(1), inside_up < 0     # the head
    /// domain == SEA                    → the ship heal     # not built
    /// is_supply                        → return
    /// …six heals that need a bonus, a wonder or a hero…    # not built
    /// CIVILIAN_HEAL_RATE != 0 and (o + frame) % rate == 0
    ///   and !(unit_masks2 & 1)
    ///   and (is_worker or is_caravan or is_merchant or type == 0x13d)
    ///   and the cell's owner is an ally:
    ///     repair_damage(1, 0, 1); healing = max(rate, healing)
    /// ```
    ///
    /// `repair_damage` with a whole point of damage takes one off and
    /// zeroes `damage_frac` (`+0x3b`); with none it leaves the fraction
    /// alone. The second argument 0 is the no-sparkle arm, so the heal
    /// draws nothing. `healing` is not carried: the aircraft heal that
    /// also writes it is a seam (`crate::cast`).
    pub(crate) fn civilian_heal(&mut self, i: usize, frame: i64) {
        let rate = i64::from(self.tuning.civilian_heal_rate);
        if rate == 0 || (i64::from(self.units[i].index) + frame) % rate != 0 {
            return;
        }
        let u = &self.units[i];
        if !u.alive() || !u.on_map || u.inside.is_some() || u.inside_unit.is_some() {
            return;
        }
        if self.captain_of(i) != i || !matches!(u.kind.domain, Domain::Land) {
            return;
        }
        let Some(t) = u.ty else {
            return;
        };
        // `UnitData::has_damage(1)`: this figure or one below it.
        let damaged = self.squad_of(i).iter().any(|&f| {
            self.units[f].health < self.units[f].max_health || self.units[f].damage_frac != 0
        });
        if !damaged {
            return;
        }
        let cols = self.unit_types[t].cols;
        if cols.flag2(uflags2::SUPPLY_OR_HERO) || u.unit_masks2 & STRUCK != 0 {
            return;
        }
        let id = self.unit_types[t].type_index;
        let civilian = WORKERS.contains(&id)
            || cols.flag2(uflags2::CARAVAN)
            || self.is_merchant(i)
            || id == FISHERMEN;
        if !civilian {
            return;
        }
        let owner = u.owner;
        let ally = match self.world.owner_at(u.pos) {
            Owner::Player(p) => self.is_ally(owner, p),
            _ => false,
        };
        if !ally {
            return;
        }
        let u = &mut self.units[i];
        if u.health < u.max_health {
            u.health += 1;
            u.damage_frac = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::world::Owner;

    /// **A citizen heals a point every 45 frames in its own territory**,
    /// on the frame `(o + frame) % 45 == 0`, and the heal clears the
    /// fraction (`docs/COMBAT.md` §72). Outside it, or on another frame,
    /// or as a soldier, nothing heals.
    #[test]
    fn a_citizen_heals_a_point_every_45_frames_in_its_own_territory() {
        use crate::world::{Pos, World};
        let mut sim = crate::Sim::new(crate::tuning::Tuning::RON, World::new(60, 60), 2);
        let ty = |sim: &mut crate::Sim, type_index: i32| {
            sim.add_unit_type(crate::UnitType {
                hits: 40,
                type_index,
                combat: crate::combat::Profile {
                    uber_size: 1,
                    ..crate::combat::Profile::default()
                },
                ..crate::UnitType::default()
            })
        };
        let citizen = ty(&mut sim, 0x32);
        let hoplite = ty(&mut sim, 0x60);
        let put = |sim: &mut crate::Sim, t: usize, p: Pos| {
            let index = i16::try_from(sim.units.len()).unwrap();
            let mut u = crate::Unit::new(0, index, p, 40);
            u.ty = Some(t);
            u.on_map = true;
            u.kind = sim.unit_types[t].kind;
            sim.add_unit(u)
        };
        let at = Pos::new(30 * 0x300 + 0x198, 30 * 0x300 + 0x198);
        let c = put(&mut sim, citizen, at);
        let soldier = put(&mut sim, hoplite, at);
        sim.world
            .set_owner(at.cell(), Owner::Player(0), Owner::Player(0));
        for u in [c, soldier] {
            sim.units[u].health = sim.units[u].max_health - 3;
            sim.units[u].damage_frac = 5;
        }
        let o = i64::from(sim.units[c].index);
        let due = 45 * 100 - o;
        sim.civilian_heal(c, due - 1);
        assert_eq!(
            (sim.units[c].health, sim.units[c].damage_frac),
            (sim.units[c].max_health - 3, 5),
            "off the phase, nothing"
        );
        sim.civilian_heal(c, due);
        assert_eq!(
            (sim.units[c].health, sim.units[c].damage_frac),
            (sim.units[c].max_health - 2, 0),
            "on the phase, a point and the fraction"
        );
        let os = i64::from(sim.units[soldier].index);
        sim.civilian_heal(soldier, 45 * 100 - os);
        assert_eq!(
            sim.units[soldier].health,
            sim.units[soldier].max_health - 3,
            "a soldier is not a civilian"
        );
        let cell = sim.units[c].pos.cell();
        sim.world.set_owner(cell, Owner::None, Owner::None);
        sim.civilian_heal(c, due + 45);
        assert_eq!(
            sim.units[c].health,
            sim.units[c].max_health - 2,
            "outside an ally's territory, nothing"
        );
    }
}
