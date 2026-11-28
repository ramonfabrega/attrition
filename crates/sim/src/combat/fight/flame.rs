//! **The Flamethrower's garrison arm** — `Object::do_damage@0064a480:953`–
//! `:1050`, on a hit that did not kill (`docs/COMBAT.md` §7.3, item 4784).
use crate::Sim;
use crate::combat::{Obj, role};

impl Sim {
    /// A Flamethrower's blow on a building that holds someone burns stand-in
    /// Citizens of its owner (`docs/COMBAT.md` §7.3).
    ///
    /// The listing's order, after `Armies::emergency`: the attacker answers
    /// `is(FLAMETHROWER)` (`ObjectData::is`, vslot `0xb8`, with `0x83`); the
    /// target's vslot `0x18` answers 0 (a building); it cannot carry air
    /// (`ObjectData::can_carry(AIR)`, the hangars). Then `n =
    /// num_inside(1)`, the live objects on its inside chain, and
    /// `eject_contents(1, -1, 0, 1)` — on a building, `build_masks |=
    /// 0x4000` and nothing else ([`crate::cities::Building::eject_pending`])
    /// — whatever `n` is. With `n` non-zero, `k = x_size >> 1` (the type's
    /// `+0x234`), and when the death pool holds fewer than `k` live records
    /// on a piece above `first_build_piece` (`0xc606`, every scorched body)
    /// it makes `k` units of the owner's Citizen (`types[PEASANTS]`, or the
    /// type's vslot `0x68` for a tribe `tribe_can_type` refuses), each
    /// `Objects::init_unit` on the building's point, `go_inside` and
    /// `come_out`, and then closes each one through vslot `0x150`:
    /// `Unit::close(4, -1, 0)`, a burning death.
    ///
    /// run1784's 5995: who=1's Flamethrower `1/180` strikes who=2's Small
    /// City `2/2040` with `2/1` inside; theirs spends three
    /// `Guy::init_real < Objects::init_unit` draws and three `Unit::close`
    /// triples, and on 5996 the city holds `0x5030`, leader 2's
    /// `units_built` is 208 against 205, and the pool holds three scorched
    /// citizens `2/171`, `2/179`, `2/180` of age 1.
    pub(crate) fn flame_garrison(&mut self, attacker: Obj, b: usize) {
        if !self.profile(attacker).is(role::IS_FLAMETHROWER) {
            return;
        }
        let Some(ty) = self.buildings[b].ty else {
            return;
        };
        if self.is_hangar(ty) {
            return;
        }
        let inside = self.buildings[b]
            .garrison
            .iter()
            .any(|&c| self.units[c].alive());
        self.buildings[b].eject_pending = true;
        if !inside {
            return;
        }
        // The type's `+0x234`, the footprint's width in tiles, on every
        // building (a combat profile carries it only for one that fires).
        let k = self.tables.build_types[ty].x_size >> 1;
        if self.scorched_bodies() >= k {
            return;
        }
        let who = self.buildings[b].owner;
        let Some(citizen) = self.flame_citizen(who) else {
            return;
        };
        let at = self.buildings[b].pos;
        let mut made = Vec::with_capacity(k.max(0) as usize);
        for _ in 0..k {
            let u = self.init_unit(who, citizen, at);
            self.go_inside(u, b);
            self.come_out(u);
            made.push(u);
        }
        for u in made {
            self.close_through_slot(u, 4);
        }
    }

    /// The death pool's live records on a piece above `first_build_piece`
    /// — the arm's loop over `Objects +0x14c` (`+0x140` slots, `0xa4`
    /// apart), testing the slot's `valid` word and `DeathObj +0x20`.
    fn scorched_bodies(&self) -> i32 {
        let mut n = 0;
        for slot in self.combat.death_pool.iter().flatten() {
            let Some(d) = self.live_death(slot) else {
                continue;
            };
            if d.figures.gpiece[usize::from(slot.figure)] > crate::units::anim::TOTAL_UNIT_PIECES {
                n += 1;
            }
        }
        n
    }

    /// `types[PEASANTS]`, when `LeaderData::tribe_can_type` answers it for
    /// `who`; as a unit record.
    ///
    /// SEAM: the other arm, the type's vslot `0x68` handed the leader's
    /// tribe, is not read; no tribe on file refuses the Citizen.
    fn flame_citizen(&self, who: crate::world::Player) -> Option<usize> {
        let p = self.tech.get(usize::from(who))?;
        crate::seam!(
            "flame_citizen_tribe_type",
            self.tables
                .tech_tree
                .tribe_can_type(&self.tables.setup, p, crate::ids::ty::PEASANTS)
                == 0
        );
        let _ = p;
        self.unit_rec_of_index(crate::spells::cast::PEASANTS)
    }
}
