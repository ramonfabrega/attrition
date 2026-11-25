//! **A unit's and a figure's stored height** (item 4660, `docs/RECGAME.md`
//! §13.2; `docs/COLLISION.md` §3.3).
//!
//! The original keeps a unit's `z` (`ObjectData +0xc`) and each figure's
//! `z` and `last_z` (`GuyData +0x14`, `+0x70`) as stored fields, written
//! where it moves them: `Unit::set_new_location@005f8d20` (`find_tcoord_z`
//! at the new tile, on every call), `Guy::set_new_location@005d86f0` (the
//! ground at the new point; with `param_3`, `last_z = z`), `Guy::move`'s
//! head (`last_z = z`, `005d9240:47`) and `Unit::update_z@00606590`. This
//! crate reads them live off the ground, which is the same thing until the
//! ground moves under a unit that does not.
//!
//! **A terraform rewrites no stored height but the ones `Wall::init`
//! refreshes.** `Wall::init@0063e9b0`, after its
//! `TerrainOut::terraform_for_building` (`:70`), walks the world cells
//! around the building — 81 for a city's (`is(0x19e)`, the `VILLAGE`
//! lineage), 25 for any other (`0063eb70`..`0063ec43`, `move_x`/`move_y`)
//! — and each cell's object chain from its head (`WData +0x8`/`+0xa`,
//! `ObjectData +0x2c`/`+0x2e`), and calls `Unit::update_z` on each object
//! whose vslot `+8` answers (a live unit, `flags & 1`) and whose type is of
//! domain 0 (`+0x218`). `update_z` writes the unit's `z` and every figure's
//! `z`, and no `last_z`. A unit on no cell's chain — run1832's standing
//! Phalanxes `2/76` and `2/91`, under Smelter `2/2040` on 3357 — keeps the
//! height it stood at, 22 and 18 where the new ground reads 29 and 25.
//!
//! So this crate holds a height **only where it differs from the ground**:
//! [`crate::Unit::z_held`], [`crate::units::anim::Guy::z_held`] and
//! [`crate::units::anim::Guy::last_z_held`], set by [`Sim::wall_init_z`]
//! from a snapshot taken before the terraform, and dropped where the
//! original writes the field again.
//!
//! Not established: a figure's held `z` is read only while its body stands
//! on the point it was held at, so a `Guy::set_new_location` onto the same
//! point with no `param_3` (the moving arm of `Guy::move` on a body that
//! did not move) keeps it where the original's would rewrite it. The
//! renderer and the movement read the ground, not the stored height.
//! `TerrainOut::terraform_for_nuke`'s crater refreshes nothing here.

use crate::Sim;
use crate::territory::attrition::Domain;
use crate::world::Pos;

/// The heights the units and guys lines print, before a terraform:
/// per unit slot, its `z` and each figure's `(z, last_z)`.
pub(crate) type HeightSnapshot = Vec<(i32, Vec<(i32, i32)>)>;

impl Sim {
    /// The unit's stored `z` (`ObjectData +0xc`): its held height, or the
    /// ground at its tile ([`crate::World::object_z`]), which is what
    /// `Unit::set_new_location` last wrote there.
    pub(crate) fn unit_stored_z(&self, u: usize) -> i32 {
        let unit = &self.units[u];
        unit.z_held
            .unwrap_or_else(|| self.world.object_z(unit.pos.tile()))
    }

    /// The point figure `g` of unit `u` stands on and the point its last
    /// `Guy::move` began from, as the guys line prints them (`+0xc`/`+0x10`,
    /// `+0x68`/`+0x6c`).
    pub(crate) fn guy_points(&self, u: usize, g: usize) -> (Pos, Pos) {
        let unit = &self.units[u];
        let guy = unit.guys[g];
        let body = guy.follow.map_or(unit.movement.body.pos, |f| f.body.pos);
        let last = guy.last.map_or(body, |(at, _)| at);
        (body, last)
    }

    /// Figure `g`'s stored `z` and `last_z` (`GuyData +0x14`, `+0x70`), as
    /// the guys line prints them: a held height where a terraform left one,
    /// else [`Sim::guy_z`] of its point and of its last point.
    ///
    /// **A unit inside something walks no figure** (item 4411,
    /// `docs/CHECKSUM.md` §23.9): `Unit::process@00610bc0:554`'s arm for
    /// `inside_up >= 0` calls no `Guy::process`, and copies every figure's
    /// `z` into `last_z` (`:582`, `0x0061158d`), so `last_z` is the figure's
    /// own `z`. **The arm is chosen at the unit's own `process`** (item
    /// 4467): a unit that went inside after it walked this frame keeps the
    /// ground under `last` (run1725's `1/0` boarding on 4026).
    pub(crate) fn guy_stored_z(&self, u: usize, g: usize) -> (i32, i32) {
        let unit = &self.units[u];
        let guy = unit.guys[g];
        let (body, last) = self.guy_points(u, g);
        let z = match guy.z_held {
            Some((at, v)) if at == body => v,
            _ => self.guy_z(u, g, body, false),
        };
        let last_z =
            if (unit.inside.is_some() || unit.inside_unit.is_some()) && !unit.processed_outside {
                z
            } else {
                guy.last_z_held
                    .unwrap_or_else(|| self.guy_z(u, g, last, true))
            };
        (z, last_z)
    }

    /// Every unit's stored heights, before a terraform moves the ground
    /// under them.
    pub(crate) fn height_snapshot(&self) -> HeightSnapshot {
        (0..self.units.len())
            .map(|u| {
                let guys = (0..self.units[u].guys.len())
                    .map(|g| self.guy_stored_z(u, g))
                    .collect();
                (self.unit_stored_z(u), guys)
            })
            .collect()
    }

    /// **`Wall::init@0063e9b0`'s heights** for building `b`: with
    /// `before` (the terraform ran), every stored height the new ground
    /// would move is held at what it was; then `Unit::update_z` on every
    /// live land unit on the chains of the cells around the building,
    /// which writes its `z` and its figures' `z` from the new ground and
    /// leaves their `last_z`. The walk runs whether or not the terraform
    /// did (a farm, a restore).
    pub(crate) fn wall_init_z(&mut self, b: usize, before: Option<HeightSnapshot>) {
        if let Some(before) = before {
            for (u, (z, guys)) in before.into_iter().enumerate() {
                if self.units[u].z_held.is_none() && self.unit_stored_z(u) != z {
                    self.units[u].z_held = Some(z);
                }
                for (g, (gz, glast)) in guys.into_iter().enumerate() {
                    if g >= self.units[u].guys.len() {
                        break;
                    }
                    let (body, _) = self.guy_points(u, g);
                    if self.guy_stored_z(u, g).0 != gz {
                        self.units[u].guys[g].z_held = Some((body, gz));
                    }
                    if self.guy_stored_z(u, g).1 != glast {
                        self.units[u].guys[g].last_z_held = Some(glast);
                    }
                }
            }
        }
        let cells = if self.building_is_city(b) { 81 } else { 25 };
        let at = self.buildings[b].pos.cell();
        let (w, h) = (self.world.width(), self.world.height());
        for &(dx, dy) in &crate::world::MOVE_289[..cells] {
            let (x, y) = (at.x + dx, at.y + dy);
            if x < 0 || y < 0 || x >= w || y >= h {
                continue;
            }
            let mut next = self.terrain.chain_heads[(y * w + x) as usize];
            while let Some(u) = next {
                next = self.units[u].down;
                if self.units[u].alive() && self.unit_domain_of(u) == Domain::Land {
                    self.unit_update_z(u);
                }
            }
        }
    }

    /// `Unit::update_z@00606590`: the unit's `z` and every figure's `z`
    /// from the ground; no `last_z`.
    fn unit_update_z(&mut self, u: usize) {
        self.units[u].z_held = None;
        for g in &mut self.units[u].guys {
            g.z_held = None;
        }
    }
}
