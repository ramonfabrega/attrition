//! The danger map — `GameDaemon::calc_danger@00732d10` and
//! `GameDaemon::do_danger@00732390`.
//!
//! One `int` grid per leader over a **half-resolution cell** grid
//! (`reg_xs × reg_ys`, each entry two world cells square), rebuilt from
//! scratch every two hundredth frame and read all over: the world-grid
//! pathfinder prices a step by `danger / 8` (`docs/PATHFINDER.md` §5), and
//! five of the production AI's scores divide by `danger + 1`.
//!
//! It is not a threat map so much as a **balance of force** map, and the
//! sign is the whole of it: your own things make a square *negative* and
//! an enemy's make it positive. That is why leaving it at zero is not a
//! neutral simplification — around your own city the original prices a
//! path step 8 to 16 cheaper than a bare formula does, which is exactly
//! what sent East Indies' caravan the wrong way (`docs/CARAVAN.md` §4.1).
//!
//! The specification is `docs/DANGER.md`.

use crate::Player;
use crate::build;
use crate::combat::Obj;
use crate::world::Pos;

/// The eight neighbours the spread reaches, in the executable's own tables
/// — `move_x + 4` at `0xadcaf4` and `move_y + 4` at `0xadc404`, read out of
/// the PE. They are exactly `crate::path`'s `MOVE_X[1..=8]`/`MOVE_Y[1..=8]`,
/// which is why the danger footprint is a 3×3 block and not a disc.
const SPREAD: [(i32, i32); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
];

/// How often the map is rebuilt: `GameDaemon::process_all@00732700:64`,
/// `if (game->frame % 200 == 0) calc_danger()`. Nothing decays it in
/// between, so a map is up to 199 frames stale wherever it is read.
pub const PERIOD: i64 = 200;

impl crate::Sim {
    /// `GameDaemon::calc_danger` — the whole rebuild.
    ///
    /// Three passes: zero every **active** leader's row, then every
    /// **military** unit of every leader in play, then every finished
    /// building of every leader in play. Gaia is not a leader here — the
    /// loops run over slots 0..8 — so its animals and its goodies weigh
    /// nothing.
    pub(crate) fn calc_danger(&mut self) {
        for viewer in self.danger_viewers() {
            self.world.clear_danger(viewer);
        }
        self.danger_units();
        self.danger_buildings();
    }

    /// Leaders with `leader_flags & 2` (`LEADER_ACTIVE`) — the rows that
    /// exist, and the only viewers any contribution is written for.
    fn danger_viewers(&self) -> Vec<Player> {
        (0..self.territory.players.len().min(8))
            .map(|w| u8::try_from(w).expect("under eight"))
            .filter(|&w| !self.defeated[w as usize])
            .collect()
    }

    /// `LeaderData::diplos[a][b]` — 0 war, 1 peace, 2 alliance
    /// (`docs/ARMY.md` §1), reassembled from the two matrices this crate
    /// keeps instead.
    fn diplo(&self, a: Player, b: Player) -> i32 {
        if self.at_war_with(a, b) {
            0
        } else if self.allied_with(a, b) {
            2
        } else {
            1
        }
    }

    /// The half-cell a position falls in — `div3(x >> 9)`, `div3(y >> 9)`,
    /// which is the cell halved.
    fn danger_half_of(p: Pos) -> (i32, i32) {
        let c = p.cell();
        (c.x / 2, c.y / 2)
    }

    /// The unit pass: every **military** type (`role & 0x10000`) that is on
    /// the map, at half its `attack`, and only for a viewer it is at war
    /// with — the arm that can only ever *add*.
    fn danger_units(&mut self) {
        let viewers = self.danger_viewers();
        let rows: Vec<(usize, Player, i16, i32, i32)> = (0..self.units.len())
            .filter(|&i| {
                let u = &self.units[i];
                u.alive()
                    && u.on_map
                    && u.owner < 8
                    && u.ty.is_some_and(|t| {
                        self.tables.unit_types[t]
                            .cols
                            .is(crate::ai_load::role::ROLE_MILITARY)
                    })
            })
            .map(|i| {
                let u = &self.units[i];
                let (hx, hy) = Self::danger_half_of(u.pos);
                (i, u.owner, u.index, hx, hy)
            })
            .collect();
        for (i, owner, _o, hx, hy) in rows {
            // `(attack * 5) / 10` as the original writes it.
            let v = (self.attack_of(Obj::Unit(i)) * 5) / 10;
            let idx = self.danger_index(hx, hy);
            let Some(idx) = idx else { continue };
            for &viewer in &viewers {
                if viewer == owner {
                    continue;
                }
                if self.diplo(owner, viewer) != 0 && self.diplo(viewer, owner) != 0 {
                    continue;
                }
                let seen = self.danger_unit_is_seen(i, viewer);
                self.do_danger(seen, owner, viewer, idx, v);
            }
        }
    }

    /// The building pass: every **finished** building whose type wants a
    /// city or belongs to one, at its own weight, spread over its own
    /// half-cell and the eight around it at half.
    ///
    /// This is the pass that makes the map negative, because it is written
    /// for **every** active viewer including the owner.
    fn danger_buildings(&mut self) {
        let viewers = self.danger_viewers();
        let rows: Vec<(usize, Player, i32, i32, i32)> = (0..self.buildings.len())
            .filter(|&b| self.danger_counts_building(b))
            .map(|b| {
                let bd = &self.buildings[b];
                let (hx, hy) = Self::danger_half_of(bd.pos);
                (b, bd.owner, hx, hy, self.danger_building_value(b))
            })
            .collect();
        for (b, owner, hx, hy, v) in rows {
            for &viewer in &viewers {
                for (dx, dy) in SPREAD {
                    if let Some(idx) = self.danger_index(hx + dx, hy + dy) {
                        let seen = self.danger_build_is_seen(b, viewer);
                        self.do_danger(seen, owner, viewer, idx, v / 2);
                    }
                }
                if let Some(idx) = self.danger_index(hx, hy) {
                    let seen = self.danger_build_is_seen(b, viewer);
                    self.do_danger(seen, owner, viewer, idx, v);
                }
            }
        }
    }

    /// The row index, or `None` off the grid — the bounds test the spread
    /// makes and the centre does not (the centre's own half-cell is on the
    /// map by construction).
    fn danger_index(&self, hx: i32, hy: i32) -> Option<i32> {
        if hx < 0 || hy < 0 || hx >= self.world.reg_xs() || hy >= self.world.reg_ys() {
            return None;
        }
        Some(hy * self.world.reg_xs() + hx)
    }

    /// `GameDaemon::do_danger` — the one write, and the three arms that
    /// decide its sign.
    ///
    /// `seen` is the object's own `is_seen(viewer)`, which only the enemy
    /// arm reads.
    fn do_danger(&mut self, seen: bool, owner: Player, viewer: Player, idx: i32, value: i32) {
        if viewer == owner {
            self.world.add_danger(viewer, idx, -value);
            return;
        }
        if self.diplo(owner, viewer) == 2 {
            self.world.add_danger(viewer, idx, -(value / 2));
            return;
        }
        let mut value = value;
        if !seen {
            value /= 2;
        }
        if self.diplo(owner, viewer) == 1 {
            value /= 2;
        }
        self.world.add_danger(viewer, idx, value);
    }

    /// The building filter: finished (`WallData::is_active`, `flags & 4`)
    /// and either a member of a city or a type that needs none
    /// (`build_flags & 0x10`, `NO_CITY`).
    fn danger_counts_building(&self, b: usize) -> bool {
        let bd = &self.buildings[b];
        if !bd.alive || !bd.active || bd.owner >= 8 {
            return false;
        }
        bd.city.is_some()
            || bd
                .ty
                .is_some_and(|t| self.tables.build_types[t].has(build::flags::NO_CITY))
    }

    /// What one building is worth: a fort or a tower its own remaining hit
    /// points halved, a city, an Airbase or a Dock a flat hundred, and
    /// everything else ten — fifty for a military trainer
    /// (`build_flags & 0x40000000`).
    fn danger_building_value(&self, b: usize) -> i32 {
        let bd = &self.buildings[b];
        let ty = bd.ty;
        if ty.is_some_and(|t| {
            build::is_fort(&self.tables.build_types, t)
                || build::is_tower(&self.tables.build_types, t)
        }) {
            return bd.health.max(0) / 2;
        }
        if self.building_is_city(b) {
            return 100;
        }
        if ty.is_some_and(|t| {
            build::is(&self.tables.build_types, t, build::Ident::Airbase)
                || build::is_dock(&self.tables.build_types, t)
        }) {
            return 100;
        }
        // The trainer bit is the **basic type's** (`BuildTypeData::basic_type`,
        // the root of the `FROM` chain: `calc_danger@00732d10:158`), so an
        // Auto Plant, upgraded from a Factory, weighs fifty and not ten
        // (`docs/AI.md` §159).
        if ty.is_some_and(|t| {
            self.tables.build_types[self.build_root(t)].has(build::flags::MILITARY_TRAINER)
        }) {
            50
        } else {
            10
        }
    }

    /// `BuildData::is_seen(viewer, 0)` — vslot `+0x48`, the one
    /// `do_danger`'s enemy arm asks — which is [`Sim::build_is_seen`]: the
    /// building's own `visible` and `ever_seen` bytes, never the fog
    /// (`BuildData::is_seen@0062e1a0` → `WallData::is_seen@00642bd0`,
    /// `docs/DANGER.md` §5).
    ///
    /// It read the fog through `was_seen` until item 3367, and the two
    /// answered alike until a Spanish viewer: `build_empire` raises
    /// `leader_flags & 0x1000` for tribe bonus 9, `was_seen`'s leader arm
    /// then calls every cell seen, and no human building was ever halved
    /// for it — the East Indies Spanish's danger row stood at about twice
    /// the original's around the human's city (run993b, block 8959: half-cell
    /// (4, 4) ours 125, theirs 60), and its scout chose another cell on 8959.
    fn danger_build_is_seen(&self, b: usize, viewer: Player) -> bool {
        self.build_is_seen(b, viewer)
    }

    /// `UnitData::is_seen@00607a60(viewer, 0)`, the fog half: the *live*
    /// line of sight at the unit's own half-cell, then the unit's
    /// `visible` byte (set by attacking the viewer) — not the ever-seen bit
    /// a building takes (item 2703, `docs/GOLDEN.md` §100).
    ///
    /// The stealth arm above the fog test is [`Sim::unit_stealth_hides`]
    /// (item 2725, `docs/GOLDEN.md` §101).
    fn danger_unit_is_seen(&self, u: usize, viewer: Player) -> bool {
        let un = &self.units[u];
        if un.owner == viewer || !self.world.has_fog() {
            return true;
        }
        if self.unit_stealth_hides(u, viewer) {
            return false;
        }
        self.world_sees(un.pos, viewer) || un.visible & Self::who_bit(viewer) != 0
    }
}

#[cfg(test)]
mod tests;
