//! **Jamming** (item 4541, `docs/COMBAT.md` §8.6): an enemy special forces
//! unit standing near an anti-air object. `ObjectData::is_jammed@00653660`
//! finds the jammer; `Build::do_attack@006228f0` holds a jammed building's
//! shot and marks both sides; `Unit::process@00610bc0`'s caster arm trades
//! the jammer's mark for the bit that shows it.

use crate::combat::{Obj, mask, umask2};
use crate::territory::attrition::Domain;
use crate::*;

/// `SPECIALFORCES` (`TypeIndex` 0x4b), the lineage
/// `SpecialsData::find_special@00740070` and the caster arm ask
/// (`is(0x4b, 0)`).
const SPECIALFORCES: crate::tech::TypeId = 0x4b;

/// `GENERAL_RADIUS × 0x180`: `find_special`'s reach, in map units
/// (`740139`..`74014b`: `constants +0xc58`, `lea (r, r, 2)`, `shl 7`).
const REACH_SCALE: i32 = 0x180;

impl Sim {
    /// **`ObjectData::is_jammed@00653660`**: the special forces unit jamming
    /// object `o`, as this crate's unit slot, or `None` (the original's −1).
    ///
    /// ```text
    /// if leader(owner).leader_flags & 0x40000          // jam_threat
    ///    and type +0x218 != 2                            // not air
    ///    and has_objmask(0x80000000):                    // ANTI_AIR
    ///   for L in 0..8:
    ///     if L.leader_flags & 1 and L != owner
    ///        and !L.is_ally(owner) and L.leader_flags & 0x20000:
    ///       k = find_special(x, y, L)
    ///       if k >= 0: return specials[L][k].o           // (+0x26)
    /// return -1
    /// ```
    ///
    /// Its callers are `Build::do_attack` (vslot `+0xe0`, with the leader
    /// out-parameter) and `Unit::fight`'s anti-air animation roll
    /// ([`Sim::radar_jams`]).
    pub(crate) fn is_jammed(&self, o: Obj) -> Option<usize> {
        let owner = self.owner_of(o);
        if !self
            .leaders
            .jam_threat
            .get(owner as usize)
            .copied()
            .unwrap_or(false)
        {
            return None;
        }
        let p = self.profile(o);
        if matches!(p.domain, Domain::Air) || !p.has(mask::ANTI_AIR) {
            return None;
        }
        let seats = self
            .economy
            .ledgers
            .len()
            .min(crate::checksum::leaders::LEADER_SLOTS);
        let at = self.pos_of(o);
        (0..seats).find_map(|j| {
            let l = j as Player;
            (l != owner
                && !self.is_ally(l, owner)
                && self.leaders.special_forces.get(j).copied().unwrap_or(false))
            .then(|| self.find_special(at, l))
            .flatten()
        })
    }

    /// **`SpecialsData::find_special@00740070`**: the first slot of `who`'s
    /// specials list, below its mark (`LeaderData +0x43c`), whose record is
    /// in use (`+0x28 & 1`), whose unit is on the map (`UnitData::is_on_map`,
    /// `+0x82`'s sign), within `GENERAL_RADIUS × 0x180` of `at` by
    /// `vector_dist`, and `is(SPECIALFORCES, 0)`. Its unit, by slot here.
    fn find_special(&self, at: Pos, who: Player) -> Option<usize> {
        let list = self.roster.specials.get(who as usize)?;
        let reach = self.tables.tuning.general_radius * REACH_SCALE;
        (0..list.mark)
            .filter(|&k| list.in_use.get(k).copied().unwrap_or(false))
            .find_map(|k| {
                let u = self
                    .units
                    .iter()
                    .position(|x| x.alive() && x.owner == who && x.special == Some(k))?;
                let un = &self.units[u];
                (un.on_map
                    && crate::world::vector_dist(at.x - un.pos.x, at.y - un.pos.y) <= reach
                    && self.unit_is(u, SPECIALFORCES))
                .then_some(u)
            })
    }

    /// Probe (item 4666): the jam's inputs around unit `who/o`.
    pub fn probe_4666_jam(&self, who: Player, o: i16) -> String {
        let mut out = String::new();
        let Some(u) = self
            .units
            .iter()
            .position(|x| x.alive() && x.owner == who && x.index == o)
        else {
            return format!("{who}/{o} absent");
        };
        let un = &self.units[u];
        let ti = un.ty.map(|t| self.tables.unit_types[t].type_index);
        out += &format!(
            "unit {who}/{o} TI {ti:?} special {:?} on_map {} pos ({}, {}) masks2 {:#x} is_sf {}\n  jam_threat {:?}\n  special_forces {:?}\n  specials[{who}] mark {:?} in_use {:?}\n",
            un.special,
            un.on_map,
            un.pos.x,
            un.pos.y,
            un.unit_masks2,
            self.unit_is(u, SPECIALFORCES),
            self.leaders.jam_threat,
            self.leaders.special_forces,
            self.roster.specials.get(who as usize).map(|l| l.mark),
            self.roster
                .specials
                .get(who as usize)
                .map(|l| l.in_use.clone()),
        );
        let reach = self.tables.tuning.general_radius * REACH_SCALE;
        for (b, bd) in self.buildings.iter().enumerate() {
            if !bd.alive || bd.owner == who {
                continue;
            }
            let d = crate::world::vector_dist(bd.pos.x - un.pos.x, bd.pos.y - un.pos.y);
            if d > reach * 8 && !self.profile(Obj::Building(b)).has(mask::ANTI_AIR) {
                continue;
            }
            let p = self.profile(Obj::Building(b));
            out += &format!(
                "  building {}/{} ty {:?} ti {:?} dist {d} reach {reach} anti_air {} air {} jammed_now {} is_jammed {:?}\n",
                bd.owner,
                bd.index,
                bd.ty,
                bd.ty.map(|t| self
                    .tables
                    .build_types
                    .get(t)
                    .map(|x| format!("{:?}", x.ident))),
                p.has(mask::ANTI_AIR),
                matches!(p.domain, Domain::Air),
                bd.jammed,
                self.is_jammed(Obj::Building(b))
                    .map(|k| (self.units[k].owner, self.units[k].index)),
            );
        }
        for (k, x) in self.units.iter().enumerate() {
            if !x.alive() || x.owner == who || !self.profile(Obj::Unit(k)).has(mask::ANTI_AIR) {
                continue;
            }
            let d = crate::world::vector_dist(x.pos.x - un.pos.x, x.pos.y - un.pos.y);
            if d > reach * 2 {
                continue;
            }
            out += &format!(
                "  unit {}/{} TI {:?} dist {d} on_map {} inside {:?}/{:?} air {} is_jammed {:?}\n",
                x.owner,
                x.index,
                x.ty.map(|t| self.tables.unit_types[t].type_index),
                x.on_map,
                x.inside,
                x.inside_unit,
                matches!(self.profile(Obj::Unit(k)).domain, Domain::Air),
                self.is_jammed(Obj::Unit(k))
                    .map(|j| (self.units[j].owner, self.units[j].index)),
            );
        }
        out
    }

    /// **`Build::do_attack@006228f0`'s jam arm** (`6229c6`..`622a1c`), past
    /// the hits test and before the garrison's arrows: a jammer found marks
    /// itself (`unit_masks2 |= 8`, [`umask2::JAMMER`]) and the building
    /// (`build_masks |= 0x8000`, [`crate::Building::jammed`]), plays
    /// `S_MISSFIRE_FROM_JAMMING`, and the call returns without a shot; none
    /// found clears the building's bit. True when jammed.
    pub(crate) fn building_jam(&mut self, b: usize) -> bool {
        match self.is_jammed(Obj::Building(b)) {
            Some(u) => {
                self.units[u].unit_masks2 |= umask2::JAMMER;
                self.buildings[b].jammed = true;
                true
            }
            None => {
                self.buildings[b].jammed = false;
                false
            }
        }
    }

    /// **`Unit::process@00610bc0`'s anti-air arm** (item 4666,
    /// `611629`..`61167a`), in the sixteen-frame on-map block after the
    /// 32-frame attrition refresh: a unit whose type `has_objmask(0x80000000)`
    /// ([`mask::ANTI_AIR`]) asks `is_jammed` (vslot `+0xe0`, the leader as
    /// out-parameter), and the jammer it finds takes `unit_masks2 |= 8`,
    /// as [`Sim::building_jam`]'s does. Nothing else: no sound, no bit on
    /// the unit itself, no shot held. Row 133's AA unit `1/75` (TI 284)
    /// marks who=2's Special Forces `2/22` on run1797's 2565.
    pub(crate) fn unit_jam(&mut self, u: usize) {
        if !self.profile(Obj::Unit(u)).has(mask::ANTI_AIR) {
            return;
        }
        if let Some(k) = self.is_jammed(Obj::Unit(u)) {
            self.units[k].unit_masks2 |= umask2::JAMMER;
        }
    }

    /// **`Unit::process@00610bc0`'s caster arm, the jam shown**
    /// (`610ddc`..`610e58`), reached by every caster (`unit_flags2 & 2`)
    /// that is no decoy (`unit_masks & 1`), past a hero's or a special's
    /// `Caster::process_spells`:
    ///
    /// ```text
    /// if (frame + o) & 0x1f == 0
    ///    and (unit_masks & 0x80000000 or unit_masks2 & 8)
    ///    and is(0x4b, 0):
    ///   if unit_masks2 & 8: unit_masks |= 0x80000000; unit_masks2 &= ~8
    ///                       (and the shockwave, the renderer's)
    ///   else:               unit_masks &= 0x7fffffff
    /// ```
    pub(crate) fn show_jam(&mut self, u: usize, frame: i64) {
        let un = &self.units[u];
        let caster = un.ty.is_some_and(|t| {
            self.tables.unit_types[t]
                .cols
                .flag2(crate::ai::ai_load::uflags2::CASTER)
        });
        if un.decoy || !caster || (frame + i64::from(un.index)).rem_euclid(0x20) != 0 {
            return;
        }
        let marked = un.unit_masks2 & umask2::JAMMER != 0;
        if !(un.jam_shown || marked) || !self.unit_is(u, SPECIALFORCES) {
            return;
        }
        let un = &mut self.units[u];
        un.jam_shown = marked;
        un.unit_masks2 &= !umask2::JAMMER;
    }
}

#[cfg(test)]
mod tests;
