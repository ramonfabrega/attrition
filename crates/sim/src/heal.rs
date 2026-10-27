//! The on-map heals of `Unit::process_healing@005e0670` — the civilian
//! heal (`docs/COMBAT.md` §72) and the CEO's (`docs/AI.md` §348.1).
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
    /// **The CEO's heal** — the third of `process_healing`'s three
    /// economic-patriot arms (`005e0670:149`–`:232`, item 3686,
    /// `docs/AI.md` §348):
    ///
    /// ```text
    /// captain, not air, has_damage(1), inside_up < 0     # the head
    /// domain == SEA → the ship heal;  is_supply → return
    /// FRENCH_SUPPLY_HEAL_RATE != 0 and (o + frame) % rate == 0:
    ///   …the Monarch's (0x161) and the Comrade's (0x163) arms…  # not built
    ///   leader counts a 0x165:
    ///     unit_masks & 0x80 and !(unit_masks2 & 0x40000) and !is_supply
    ///                                         → skip to the next heal
    ///     has_general(0, 0x165) → repair_damage(1, 1, 1)
    /// ```
    ///
    /// Unlike the other two it asks nothing of the cell's owner, so a CEO
    /// heals its army abroad. `unit_masks & 0x80` is the period
    /// `Unit::process_attrition` assigns (it clears the bit on entry and
    /// sets it with the period), so it is read here as
    /// [`crate::Unit::attrition`]. The sparkle arm (the second argument)
    /// draws `internal_random` only for the console's allies, which is no
    /// stream this crate keeps; the third is the Conquer-the-World bonus.
    /// `healing` (`+0x38`) and the `0x4000` clear are not carried.
    ///
    /// SEAM: the Monarch's and the Comrade's arms, which also ask an
    /// allied cell; read, not built (parked 3686a).
    pub(crate) fn ceo_heal(&mut self, i: usize, frame: i64) {
        const THECEO: i32 = 0x165;
        let rate = i64::from(self.tuning.french_supply_heal_rate);
        if rate == 0 || (i64::from(self.units[i].index) + frame) % rate != 0 {
            return;
        }
        let u = &self.units[i];
        if !u.alive() || !u.on_map || u.inside.is_some() || u.inside_unit.is_some() {
            return;
        }
        if !u.captain || !matches!(u.kind.domain, Domain::Land) {
            return;
        }
        let Some(t) = u.ty else {
            return;
        };
        let squad = self.squad_members(i);
        let damaged = squad.iter().any(|&f| {
            let f = &self.units[f];
            f.alive() && (f.health < f.max_health || f.damage_frac != 0)
        });
        if !damaged || self.unit_types[t].cols.flag2(uflags2::SUPPLY_OR_HERO) {
            return;
        }
        if u.attrition != 0 && u.unit_masks2 & 0x40000 == 0 {
            return;
        }
        if !self.has_general(i, THECEO) {
            return;
        }
        self.repair_unit_damage(i, 1);
    }

    /// **`Unit::repair_damage@0060de10`** on a figure, the whole-point
    /// half: the figure's own damage first — `amount` off it, floored at
    /// none, and its fraction (`+0x3b`) zeroed when it had any — and what
    /// is left over, `amount − damage`, handed down `o_down` to the next
    /// live figure with the same call. A figure with no damage hands the
    /// whole of it on, so a captain's heal reaches the hurt figure below.
    ///
    /// SEAM: past the chain's last live figure, a squad under its
    /// `uber_size` (`+0x308`) **trains a figure back** (a free object slot,
    /// `init`, a spot and a move order, the group joins): read, not built,
    /// and here the remainder is dropped (parked 3686b).
    pub(crate) fn repair_unit_damage(&mut self, i: usize, amount: i32) {
        let mut cur = Some(i);
        let mut left = amount;
        while let Some(f) = cur {
            if left <= 0 || !self.units[f].alive() {
                return;
            }
            let u = &mut self.units[f];
            let damage = u.max_health - u.health;
            if damage != 0 {
                u.health += left.min(damage);
                u.damage_frac = 0;
            }
            left -= damage;
            cur = self.units[f].o_down;
        }
    }

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

    /// **A CEO heals its army abroad, a point every 20 frames, down the
    /// squad** (item 3686, `docs/AI.md` §348.1): `process_healing`'s third
    /// economic-patriot arm runs on the captain, phased on the captain's
    /// `o`, and its `repair_damage(1, …)` hands the point down `o_down`
    /// past an unhurt figure to the hurt one, zeroing its fraction. The
    /// cell is nobody's. Off the phase, without a CEO, or under an
    /// attrition period, nothing. run1338b's `1/36` (captain `1/35`) went
    /// 4 and 14/16 to 3 and 6/16 on block 3167, a point and then the
    /// bleed. Made to fail with the repair stopping at the captain
    /// (`tools/mutate.py`, `cur = None`).
    #[test]
    fn a_ceo_heals_the_hurt_figure_of_a_squad_abroad_every_20_frames() {
        use crate::ai_load::{UnitCols, uflags2};
        use crate::world::{Pos, World};
        let mut sim = crate::Sim::new(crate::tuning::Tuning::RON, World::new(60, 60), 2);
        let rifle = sim.add_unit_type(crate::UnitType {
            hits: 645,
            type_index: 137,
            combat: crate::combat::Profile {
                uber_size: 3,
                ..crate::combat::Profile::default()
            },
            ..crate::UnitType::default()
        });
        let ceo = sim.add_unit_type(crate::UnitType {
            hits: 110,
            type_index: 0x165,
            cols: UnitCols {
                unit_flags2: uflags2::GENERAL | uflags2::SUPPLY_OR_HERO,
                ..UnitCols::default()
            },
            ..crate::UnitType::default()
        });
        let at = Pos::new(30 * 0x300 + 0x198, 30 * 0x300 + 0x198);
        let put = |sim: &mut crate::Sim, t: usize, hits: i32| {
            let index = i16::try_from(sim.units.len()).unwrap();
            let mut u = crate::Unit::new(1, index, at, hits);
            u.ty = Some(t);
            u.type_index = sim.unit_types[t].type_index;
            u.on_map = true;
            u.kind = sim.unit_types[t].kind;
            sim.add_unit(u)
        };
        let squad = [
            put(&mut sim, rifle, 215),
            put(&mut sim, rifle, 215),
            put(&mut sim, rifle, 215),
        ];
        sim.units[squad[0]].captain = true;
        sim.units[squad[0]].o_down = Some(squad[1]);
        sim.units[squad[1]].captain = false;
        sim.units[squad[1]].o_up = Some(squad[0]);
        sim.units[squad[1]].o_down = Some(squad[2]);
        sim.units[squad[2]].captain = false;
        sim.units[squad[2]].o_up = Some(squad[0]);
        let hurt = squad[2];
        sim.units[hurt].health = 215 - 4;
        sim.units[hurt].damage_frac = 14;
        let c = squad[0];
        let o = i64::from(sim.units[c].index);
        let due = 20 * 160 - o;
        let state = |sim: &crate::Sim| (sim.units[hurt].health, sim.units[hurt].damage_frac);
        sim.ceo_heal(c, due);
        assert_eq!(state(&sim), (211, 14), "no CEO, nothing");
        let boss = put(&mut sim, ceo, 110);
        sim.units[boss].captain = true;
        sim.ceo_heal(c, due - 1);
        assert_eq!(state(&sim), (211, 14), "off the phase, nothing");
        sim.ceo_heal(hurt, due);
        assert_eq!(
            state(&sim),
            (211, 14),
            "a figure below the captain is not asked"
        );
        sim.ceo_heal(c, due);
        assert_eq!(
            state(&sim),
            (212, 0),
            "a point down the chain, and the fraction"
        );
        assert_eq!(
            sim.units[squad[1]].health, 215,
            "the unhurt figure passes it on"
        );
        sim.units[c].attrition = 48;
        sim.ceo_heal(c, due + 20);
        assert_eq!(state(&sim), (212, 0), "under an attrition period, nothing");
    }
}
