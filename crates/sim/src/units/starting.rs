//! **The starting units' two setup steps** (item 4063, `docs/START.md` §2
//! step 7, `docs/ORDERS.md` §9.3): `Setup::place_unit@005abca0`, the
//! scattered placement a scout takes, and `Unit::come_out@00617c10(u, 0)`
//! for a unit standing on the map inside nothing — the step a starting
//! citizen born on its building's point takes off the footprint.
//!
//! Neither is on a live path. `Setup::build_units@005aafc0` calls both at
//! frame 0, and the lobby path (`rondata::diff::start::from_lobby`) is
//! their one caller here; the installer takes every position from a dump.

use crate::units::come_out_view::SimComeOut;
use crate::units::think::scout::circle;
use crate::{Cell, Player, Pos, Sim};

/// `place_unit`'s tries on a start with a city (`0x3c`).
pub const PLACE_TRIES: i32 = 0x3c;
/// The ring a start with a city draws from: `circle_radius[2]`.
pub const PLACE_RING: usize = 2;

/// Why [`Sim::place_unit`] did not place: an arm this port does not build.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaceRefused {
    /// A Nomad start (`starting_town == 0`): 500 tries, ring 8 for the
    /// first unit and 24 once `LeaderData::active` is set — not built.
    Nomad,
    /// Sixty refusals with a city: `Build::train(city, type)` — not built.
    Train,
}

impl Sim {
    /// **`Setup::place_unit@005abca0(who, city, type, x, y)`** on a start
    /// with a city: the anchor is the city's own point; up to sixty tries,
    /// each `Random::get(0, 0xffff) % circle_radius[2]` (no draw when the
    /// ring holds one point) picking an offset off the anchor's cell; the
    /// first cell [`Sim::place_unit_admits`] gets
    /// `Objects::init_unit(who, type, cell centre)` — and **no `come_out`**:
    /// the unit stands where `Unit::init` snapped it. `ty` is already the
    /// leader's `current_upgrade` (the original takes it again, which is
    /// idempotent).
    pub fn place_unit(
        &mut self,
        who: Player,
        city: Option<usize>,
        ty: usize,
        at: Pos,
    ) -> Result<usize, PlaceRefused> {
        if self.ai.lobby.starting_town == 0 {
            return Err(PlaceRefused::Nomad);
        }
        let anchor = city.map_or(at, |b| self.buildings[b].pos).cell();
        let table = circle::table();
        let n = table.radius[PLACE_RING];
        for _ in 0..PLACE_TRIES {
            let idx = if n == 1 || n - 1 < 0 {
                0
            } else {
                self.rng.get(0, 0xffff) % n
            } as usize;
            let c = Cell::new(
                anchor.x + i32::from(table.x[idx]),
                anchor.y + i32::from(table.y[idx]),
            );
            if self.place_unit_admits(anchor, c) {
                let centre = Pos::new(c.x * 0x300 + 0x180, c.y * 0x300 + 0x180);
                return Ok(self.init_unit(who, ty, centre));
            }
        }
        Err(PlaceRefused::Train)
    }

    /// `place_unit`'s cell test: on the map, the anchor's `region`,
    /// `flags & 0x30` clear, `flags & 0x100` clear, a `land` that is
    /// neither 1 nor 2, no object on the cell (`WData.down` negative — a
    /// good's `-2` and a goody's `-3` pass, a unit's or a building's chain
    /// does not), the centre tile's mask with bit 14 clear and its low two
    /// bits not 3, and `flags` bit 15 clear.
    ///
    /// The object half: [`crate::world::WorldState::chain_heads`] is the
    /// units' chain, which drops a building (the whole chain is held
    /// beside it, `docs/COLLISION.md` §3.2, and not read here), so a
    /// live building whose point is on the cell stands in for the building
    /// on the original's chain (`Object::add_to_world@0064d8c0` threads
    /// every object on its own point's cell).
    pub fn place_unit_admits(&self, anchor: Cell, c: Cell) -> bool {
        let w = &self.world;
        if c.x < 0 || c.y < 0 || c.x >= w.width() || c.y >= w.height() {
            return false;
        }
        if w.region_of(c) != w.region_of(anchor) {
            return false;
        }
        let d = w.cell_data(c);
        if d.flags & 0x30 != 0 || d.flags & 0x100 != 0 || matches!(d.land, 1 | 2) {
            return false;
        }
        let slot = (c.y as usize) * (w.width() as usize) + (c.x as usize);
        let object = d.down >= 0
            || self.terrain.chain_heads[slot].is_some()
            || self.buildings.iter().any(|b| b.alive && b.pos.cell() == c);
        if object {
            return false;
        }
        let mask = w.tile_mask(Pos::new(c.x * 4 + 2, c.y * 4 + 2));
        mask & 0x4000 == 0 && mask & 3 != 3 && d.flags & 0x8000 == 0
    }

    /// **`Unit::come_out@00617c10(u, 0)` for a unit inside nothing**: the
    /// listing's no-host arm — a sweep round the unit's own point from its
    /// type's `block_radius` out `UNIT_DISEMBARK_DISTANCE` under
    /// `FILTER_NOT_ME`, then again collision-off, else where it stands —
    /// and `set_new_location(·, ·, 1, 1)` there. Its tail (the path wipe a
    /// computer's unit takes) is not walked: a Citizen or Scholar
    /// (`0x32..=0x35`) is exempt from it, and a fresh unit has nothing to
    /// wipe.
    pub fn come_out_own_point(&mut self, u: usize) -> Pos {
        let spot = SimComeOut::own_point(self, u).run();
        let at = spot.unwrap_or(self.units[u].pos);
        self.set_new_location(u, at, true);
        at
    }
}

#[cfg(test)]
mod tests;
