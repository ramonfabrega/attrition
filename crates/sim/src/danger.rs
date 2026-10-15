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
        (0..self.players.len().min(8))
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
                    && u.ty
                        .is_some_and(|t| self.unit_types[t].cols.is(crate::ai_load::role::MILITARY))
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
                .is_some_and(|t| self.build_types[t].has(build::flags::NO_CITY))
    }

    /// What one building is worth: a fort or a tower its own remaining hit
    /// points halved, a city, an Airbase or a Dock a flat hundred, and
    /// everything else ten — fifty for a military trainer
    /// (`build_flags & 0x40000000`).
    fn danger_building_value(&self, b: usize) -> i32 {
        let bd = &self.buildings[b];
        let ty = bd.ty;
        if ty.is_some_and(|t| {
            build::is_fort(&self.build_types, t) || build::is_tower(&self.build_types, t)
        }) {
            return bd.health.max(0) / 2;
        }
        if self.building_is_city(b) {
            return 100;
        }
        if ty.is_some_and(|t| {
            build::is(&self.build_types, t, build::Ident::Airbase)
                || build::is_dock(&self.build_types, t)
        }) {
            return 100;
        }
        // The trainer bit is the **basic type's** (`BuildTypeData::basic_type`,
        // the root of the `FROM` chain: `calc_danger@00732d10:158`), so an
        // Auto Plant, upgraded from a Factory, weighs fifty and not ten
        // (`docs/AI.md` §159).
        if ty.is_some_and(|t| {
            self.build_types[self.build_root(t)].has(build::flags::MILITARY_TRAINER)
        }) {
            50
        } else {
            10
        }
    }

    /// `BuildData::is_seen(viewer, 0)`, as this crate spells seeing: the
    /// owner always, and otherwise the ever-seen fog bit at the object's
    /// own half-cell.
    fn danger_build_is_seen(&self, b: usize, viewer: Player) -> bool {
        let bd = &self.buildings[b];
        bd.owner == viewer || self.was_seen_fog(bd.pos.tile().x >> 1, bd.pos.tile().y >> 1, viewer)
    }

    /// `UnitData::is_seen(viewer, 0)`.
    ///
    /// SEAM: the original's is the live visibility read plus the stealth and
    /// detection machinery; this is the same ever-seen fog bit the building
    /// takes. A unit standing in ground the viewer has explored and left
    /// therefore counts double here where the original halves it.
    fn danger_unit_is_seen(&self, u: usize, viewer: Player) -> bool {
        let un = &self.units[u];
        un.owner == viewer || self.was_seen_fog(un.pos.tile().x >> 1, un.pos.tile().y >> 1, viewer)
    }
}

#[cfg(test)]
mod tests {
    use crate::build::{BuildType, Ident, flags};
    use crate::world::{Cell, Pos, Terrain, UNITS_PER_TILE, World};
    use crate::{Sim, Tuning};

    fn tile_pos(tx: i32, ty: i32) -> Pos {
        Pos::new(
            tx * UNITS_PER_TILE + UNITS_PER_TILE / 2,
            ty * UNITS_PER_TILE + UNITS_PER_TILE / 2,
        )
    }

    /// A 16 × 16 land world with two players at war, a city record and a
    /// barracks record. Nothing here goes near the tech tree: the danger
    /// map reads the `BuildType` row and the object, and nothing else.
    fn fx() -> (Sim, usize, usize) {
        let mut w = World::new(16, 16);
        w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(15, 15));
        let mut sim = Sim::new(Tuning::RON, w, 2);
        sim.at_war[0][1] = true;
        sim.at_war[1][0] = true;
        let city = sim.add_build_type(BuildType {
            ident: Ident::Village,
            x_size: 7,
            y_size: 7,
            hits: 1200,
            ..BuildType::default()
        });
        let barracks = sim.add_build_type(BuildType {
            ident: Ident::Other,
            x_size: 3,
            y_size: 3,
            hits: 400,
            flags: flags::MILITARY_TRAINER,
            ..BuildType::default()
        });
        (sim, city, barracks)
    }

    /// A finished building of a type that needs no city and belongs to
    /// none is skipped; one that belongs to a city is counted. The filter
    /// is `WallData::is_active` **and** (`city >= 0` or `NO_CITY`).
    #[test]
    fn the_filter_wants_a_finished_building_with_a_city_or_a_type_that_needs_none() {
        let (mut sim, _city, barracks) = fx();
        let b = sim.init_build(0, barracks, tile_pos(8, 8), false);
        assert!(!sim.danger_counts_building(b), "unfinished");
        sim.buildings[b].active = true;
        assert!(!sim.danger_counts_building(b), "finished, but no city");
        sim.build_types[barracks].flags |= flags::NO_CITY;
        assert!(sim.danger_counts_building(b), "NO_CITY needs none");
        sim.build_types[barracks].flags &= !flags::NO_CITY;
        sim.buildings[b].city = Some(0);
        assert!(sim.danger_counts_building(b), "a member of a city");
    }

    /// **An upgraded building weighs its basic type's fifty**
    /// (`calc_danger@00732d10:158`, `basic_type()`): the Auto Plant, a
    /// Factory's upgrade that no unit names as its `WHERE`, is a trainer in
    /// the danger map though its own row carries no trainer bit — Great
    /// Sahara's `1/2025` over (25, 12), priced ten here and fifty there
    /// (item 1584). Made to fail with the row's own flags read.
    #[test]
    fn an_upgrade_weighs_the_trainer_bit_of_its_basic_type() {
        let (mut sim, _city, barracks) = fx();
        let upgrade = sim.add_build_type(BuildType {
            ident: Ident::Other,
            x_size: 3,
            y_size: 3,
            hits: 400,
            from: Some(barracks),
            ..BuildType::default()
        });
        let b = sim.init_build(0, upgrade, tile_pos(8 * 4 + 1, 8 * 4 + 1), false);
        sim.buildings[b].active = true;
        assert_eq!(sim.danger_building_value(b), 50);
        sim.build_types[barracks].flags &= !flags::MILITARY_TRAINER;
        assert_eq!(sim.danger_building_value(b), 10);
    }

    /// **Your own building makes the ground under it negative**, at its
    /// own weight in its own half-cell and half that in the eight around
    /// it. This is the whole reason the map is not a decoration: it is what
    /// `calc_cost` reads as a discount.
    #[test]
    fn an_owner_s_own_building_is_a_negative_block_of_nine() {
        let (mut sim, _city, barracks) = fx();
        sim.build_types[barracks].flags |= flags::NO_CITY;
        // Cell (8, 8) is half-cell (4, 4).
        let b = sim.init_build(0, barracks, tile_pos(8 * 4 + 1, 8 * 4 + 1), false);
        sim.buildings[b].active = true;
        assert_eq!(sim.danger_building_value(b), 50, "a military trainer");
        sim.calc_danger();
        assert_eq!(sim.world.danger_half(0, Cell::new(8, 8)), -50, "the centre");
        assert_eq!(
            sim.world.danger_half(0, Cell::new(6, 8)),
            -25,
            "one half-cell west"
        );
        assert_eq!(
            sim.world.danger_half(0, Cell::new(6, 6)),
            -25,
            "and the diagonal — the spread is a 3 x 3 block"
        );
        assert_eq!(
            sim.world.danger_half(0, Cell::new(4, 8)),
            0,
            "two half-cells out is outside the block"
        );
    }

    /// The same building, seen by the enemy it is at war with: **positive**,
    /// and halved again because the enemy has never seen it
    /// (`do_danger`'s `is_seen` arm). A neutral third party at peace would
    /// halve it once more.
    #[test]
    fn an_enemy_s_building_adds_and_the_unseen_halving_is_the_only_one_at_war() {
        let (mut sim, _city, barracks) = fx();
        sim.build_types[barracks].flags |= flags::NO_CITY;
        // A fog grid nobody has lit: without one `was_seen_fog` answers
        // "everything is seen", which is the fixture's default and not
        // what a capture looks like.
        assert!(sim.world.set_fog(vec![0; 32 * 32]), "a 16 x 16 world's fog");
        let b = sim.init_build(0, barracks, tile_pos(8 * 4 + 1, 8 * 4 + 1), false);
        sim.buildings[b].active = true;
        sim.calc_danger();
        assert_eq!(
            sim.world.danger_half(1, Cell::new(8, 8)),
            25,
            "50, halved once because leader 1 has never seen it"
        );
        // Light leader 1's bit over the building and the halving goes.
        let t = sim.buildings[b].pos.tile();
        sim.world.set_seen(t.x >> 1, t.y >> 1, 1 << 1);
        sim.calc_danger();
        assert_eq!(sim.world.danger_half(1, Cell::new(8, 8)), 50, "seen");
        sim.world.set_fog(vec![0; 32 * 32]);
        // At peace it is halved twice; allied, it is *subtracted* at half.
        sim.at_war[0][1] = false;
        sim.at_war[1][0] = false;
        sim.calc_danger();
        assert_eq!(sim.world.danger_half(1, Cell::new(8, 8)), 12, "50/2/2");
        sim.allied[0][1] = true;
        sim.allied[1][0] = true;
        sim.calc_danger();
        assert_eq!(sim.world.danger_half(1, Cell::new(8, 8)), -25, "an ally's");
    }

    /// The weights: a city a flat hundred, a fort or a tower its own
    /// remaining hit points halved, a military trainer fifty and anything
    /// else ten.
    #[test]
    fn the_four_weights() {
        let (mut sim, city, barracks) = fx();
        let tower = sim.add_build_type(BuildType {
            ident: Ident::Tower,
            x_size: 2,
            y_size: 2,
            hits: 750,
            ..BuildType::default()
        });
        let farm = sim.add_build_type(BuildType {
            ident: Ident::Farm,
            x_size: 4,
            y_size: 4,
            hits: 400,
            ..BuildType::default()
        });
        let c = sim.init_build(0, city, tile_pos(8, 8), false);
        sim.activate(c, false, true);
        assert!(sim.buildings[c].city.is_some(), "activating founds it");
        assert_eq!(sim.danger_building_value(c), 100, "a city");

        let t = sim.init_build(0, tower, tile_pos(20, 8), false);
        sim.buildings[t].health = 500;
        assert_eq!(sim.danger_building_value(t), 250, "half its hit points");

        let bk = sim.init_build(0, barracks, tile_pos(30, 8), false);
        assert_eq!(sim.danger_building_value(bk), 50, "a military trainer");

        let f = sim.init_build(0, farm, tile_pos(40, 8), false);
        assert_eq!(sim.danger_building_value(f), 10, "everything else");
    }

    /// The cadence is `frame % 200`, and **nothing decays the map in
    /// between** — a building finished on frame 201 weighs nothing until
    /// 400.
    #[test]
    fn the_map_is_rebuilt_every_two_hundredth_frame_and_never_between() {
        let (mut sim, _city, barracks) = fx();
        sim.build_types[barracks].flags |= flags::NO_CITY;
        sim.tick(); // frame 0 builds an empty map
        assert_eq!(sim.world.danger_half(0, Cell::new(8, 8)), 0);
        let b = sim.init_build(0, barracks, tile_pos(8 * 4 + 1, 8 * 4 + 1), false);
        sim.buildings[b].active = true;
        for _ in 1..200 {
            sim.tick();
        }
        assert_eq!(
            sim.world.danger_half(0, Cell::new(8, 8)),
            0,
            "199 frames of standing there and the map has not moved"
        );
        sim.tick();
        assert_eq!(sim.frame, 201, "frame 200 is the rebuild");
        assert_eq!(sim.world.danger_half(0, Cell::new(8, 8)), -50);
    }

    /// The map is 8 x `reg_size` and `reg_size` is `((xs · 4) >> 3)²`, not
    /// `(xs / 2)²` — an odd cell width rounds down.
    #[test]
    fn the_grid_is_the_tile_width_halved() {
        let w = World::new(60, 60);
        assert_eq!((w.reg_xs(), w.reg_ys(), w.reg_size()), (30, 30, 900));
        let odd = World::new(61, 61);
        assert_eq!(odd.reg_xs(), 30, "(61 * 4) >> 3 is 30, not 31");
    }
}
