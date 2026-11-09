//! Cloaking and the per-nation masks the cloak reads: the Iroquois' and the
//! marines', the ambush aura and the trench (`docs/STRUCTURE.md` §3,
//! `combat/`).
//!
//! Moved out of `lib.rs` unchanged by item 4014 (`docs/STRUCTURE.md` §5, move 6).

use crate::*;

impl Sim {
    /// `Unit::init@00612100:341-357` (item 3107, `docs/AI.md` §246): an
    /// Iroquois unit (`has_tribe_bonus(0x12)`) that `is(SCOUT, 0)` takes
    /// `unit_masks2 |= 0x4080`, the forest walker among them. A trained
    /// unit takes it in the spawn, and a unit stood up from a start dump
    /// takes it from the harness, since both are born through `init`.
    ///
    /// The arm's first half (item 3958, `docs/AI.md` §402): an Iroquois
    /// unit whose type is land (`+0x218 == 0`) and lacks objmask 4
    /// ([`combat::mask::CIVILIAN`]) takes `unit_masks2 |= 0x80`
    /// ([`combat::umask2::CLOAK_MARK`]), the input of
    /// [`Sim::process_cloak`]'s own-land arm. run1228b's soldiers print it
    /// on every block and its citizens never.
    pub fn init_iroquois_masks(&mut self, u: usize) {
        self.init_marine_mask(u);
        if !self.iroquois(self.units[u].owner) {
            return;
        }
        if let Some(t) = self.units[u].ty {
            let p = &self.tables.unit_types[t].combat;
            if p.domain == attrition::Domain::Land && p.obj_masks & combat::mask::CIVILIAN == 0 {
                self.units[u].unit_masks2 |= combat::umask2::CLOAK_MARK;
            }
        }
        if self.unit_line_is(u, 0x45) {
            self.units[u].unit_masks2 |= combat::umask2::IROQUOIS_SCOUT;
        }
    }

    /// `LeaderData::has_tribe_bonus(0x12)`, the Iroquois.
    pub(crate) fn iroquois(&self, who: Player) -> bool {
        self.tech.get(who as usize).is_some_and(|p| {
            self.tables
                .tech_tree
                .has_tribe_bonus(&self.tables.setup, p, crate::ai::tribe::IROQUOIS)
        })
    }

    /// **`Unit::process_cloak@005e1c20`'s two Iroquois arms** (item 3958,
    /// `docs/AI.md` §402), reached from `Unit::process` on the unit's
    /// sixteen-frame phase, on the map:
    ///
    /// ```text
    /// unit_masks2 &= ~0x8000
    /// if (unit_masks2 & 0x80) && (owner(cell) == who
    ///                             || diplos[who][owner] == 2 && diplos[owner][who] == 2):
    ///     unit_masks2 |= 0x8000                     // own land
    /// if !(unit_masks & 0x2000) && !(leader_flags & 0x4000): unit_masks &= ~0x800
    /// if has_tribe_bonus(0x12) && type.domain == 0:
    ///     unit_masks = tile(pos) & 0x30 == 0x30 ? |0x800 : &~0x800   // forest
    /// ```
    ///
    /// The cell is the territory record's owner byte (`world +0x134`,
    /// `+0xf`), the tile the `TData` mask (`world +0x138`).
    ///
    /// **The hero aura** (`:61-91`): a unit not itself ambushing (`0x2000`,
    /// [`crate::Unit::ambushing`]) is cloaked while one of its owner's
    /// active heroes — not itself — ambushes unblown (`0x3000 == 0x2000`)
    /// on the map within `HeroData::get_radius × 0xc0`, and uncloaked
    /// otherwise. The leader's `0x4000` gate is set with the first such
    /// hero and dropped by `verify_spell_flags` once none is left, so the
    /// walk alone answers it. An ambushing hero keeps the `0x800`
    /// `cast_ambush` gave it.
    pub(crate) fn process_cloak(&mut self, i: usize) {
        let un = &self.units[i];
        let who = un.owner;
        let mut masks2 = un.unit_masks2 & !combat::umask2::LAND_CLOAK;
        if masks2 & combat::umask2::CLOAK_MARK != 0
            && let Some(o) = self.world.owner_at(un.pos).player()
            && (o == who || (self.allied_with(who, o) && self.allied_with(o, who)))
        {
            masks2 |= combat::umask2::LAND_CLOAK;
        }
        let pos = un.pos;
        self.units[i].unit_masks2 = masks2;
        if self.units[i].ambushing.is_none() {
            self.units[i].cloak = self.in_ambush_aura(i);
        }
        self.step_cloak(i, pos);
    }

    /// [`Sim::process_cloak`]'s hero walk: one of `i`'s owner's heroes,
    /// other than `i`, ambushing and not blown, on the map, within its
    /// radius of `i` (`vector_dist(|dx|, |dy|)`).
    pub(crate) fn in_ambush_aura(&self, i: usize) -> bool {
        let me = &self.units[i];
        (0..self.units.len()).any(|h| {
            let x = &self.units[h];
            h != i
                && x.owner == me.owner
                && x.alive()
                && x.on_map
                && x.ambushing.is_some()
                && !x.blown
                && self.is_hero_unit(h)
                && world::vector_dist((x.pos.x - me.pos.x).abs(), (x.pos.y - me.pos.y).abs())
                    <= self.hero_radius(h) * 0xc0
        })
    }

    /// The forest arm alone, at `at`: an Iroquois land unit is cloaked on a
    /// forest tile (`TData & 0x30 == 0x30`) and uncloaked off one; any
    /// other unit is left as it stands. [`Sim::process_cloak`]'s tail and
    /// `Unit::set_new_location@005f8d20`'s tile step both run it.
    pub(crate) fn step_cloak(&mut self, i: usize, at: Pos) {
        let un = &self.units[i];
        if self.iroquois(un.owner)
            && un
                .ty
                .is_some_and(|t| self.tables.unit_types[t].combat.domain == attrition::Domain::Land)
        {
            self.units[i].cloak = self.world.tile_mask(at.tile()) & world::tile::SURFACE
                == world::tile::SURFACE_FOREST;
        }
    }

    /// `Unit::set_type@00612fa0`'s marine mark, which `Unit::init` reaches
    /// through its `set_type` (item 3661): a unit that `is(0x77, 0)` — the
    /// Marines line — takes `unit_masks2 |= 0x800`
    /// ([`combat::umask2::MARINE`]). run1325's Marines print `unit_masks2`
    /// `0x20800`, this and the trench (`docs/AI.md` §341).
    pub(crate) fn init_marine_mask(&mut self, u: usize) {
        if self.speed_is(u, crate::cast_transport::MARINES, false) {
            self.units[u].unit_masks2 |= crate::combat::umask2::MARINE;
        }
    }

    /// **A trench `fight`, `find_melee_target` and `find_nearby_target`
    /// honour** (item 3661): `unit_masks & 0x2000000` without the marines'
    /// `unit_masks2 & 0x20000` — the shape `UnitData::is_entrenched(1)`
    /// names. A marine dug in by `check_idle` still chases and still takes
    /// the reduced blow.
    pub(crate) fn hard_trench(&self, u: usize) -> bool {
        let unit = &self.units[u];
        unit.combat.entrenched && unit.unit_masks2 & combat::umask2::MARINE_TRENCH == 0
    }
}
