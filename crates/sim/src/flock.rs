//! Birds flushed from a wood — `Objects::add_flock@0065c0e0` and the
//! flight of the `FLOCKBIRD`s it makes (`docs/AI.md` §99.12).
//!
//! A group move's follower whose formation slot the world refuses, standing
//! within `0x300` of that slot, and in a group, sends up two or three birds
//! when the slot's cell is `FOREST` (`Unit::do_group_move@005e79a0`,
//! `5e83f0`–`5e84cb`). `add_flock` jitters three cells off the slot's, looks
//! for a wood round the last, and gives each bird four air attacks on the
//! ground, one on a random point of each cell. The birds are gaia's
//! (owner 9), nothing dumps them, and they spend three kinds of draw: their
//! making (ten a bird, six more for the flock), their altitude redraw every
//! eighth frame of their flight, and the first wing-beat of `set_anim`.
//!
//! **A player sends up one flock in 225 frames** (`LeaderData::flock_stamp`,
//! `0xe1`): the stamp is read and written before the first draw, so a
//! refused flock spends only the group move's own roll (run46's 342 and
//! 466, 62 and 186 frames after its 280).

use crate::Player;
use crate::Sim;
use crate::air::{Flew, HALF_QUARTER, SITE_AIR_ALT, SITE_AIR_TURN, owed};
use crate::movement::{Angle, cos_component, find_angle, sin_component};
use crate::orders::{AirAttackGroundOrder, Body, CRUISING_ALT, Order, PathData};
use crate::world::{Cell, Pos, UNITS_PER_CELL, UNITS_PER_TILE, cell, vector_dist};

/// `TypeIndex::FLOCKBIRD`, the one type `add_flock` makes.
pub const FLOCKBIRD: crate::tech::TypeId = 0x193;

/// `add_flock`'s gap: a player's next flock waits until `frame ≥
/// flock_stamp + 0xe1` (`65c11f`–`65c12c`).
pub const FLOCK_GAP: i64 = 0xe1;

/// `Unit::do_group_move@005e79a0`'s roll for the flock's size, `% 2 + 2`
/// (`5e849e`, returning to `5e84a3`).
pub const SITE_GROUP_FLOCK: &str = "Unit::do_group_move+0xb03";
/// `add_flock`'s first cell: the slot's, jittered by `% 3 − 1` in `x`…
pub const SITE_FLOCK_JITTER_X: &str = "Objects::add_flock+0x104";
/// …and in `y`.
pub const SITE_FLOCK_JITTER_Y: &str = "Objects::add_flock+0x121";
/// The second: the first, spread by `% 5 − 2` in `x`…
pub const SITE_FLOCK_SPREAD_X: &str = "Objects::add_flock+0x185";
/// …and in `y`.
pub const SITE_FLOCK_SPREAD_Y: &str = "Objects::add_flock+0x1a4";
/// The wood search's first index, `% circle_radius[3]`.
pub const SITE_FLOCK_RING: &str = "Objects::add_flock+0x227";
/// The flock's heading, `% 12` twelfths of a turn.
pub const SITE_FLOCK_ANGLE: &str = "Objects::add_flock+0x2dc";
/// Each bird's step off the last one's heading, `(% 5 + 3)` degrees.
pub const SITE_FLOCK_TURN: &str = "Objects::add_flock+0x35c";
/// A point's offset in its cell: the first roll is the `y`…
pub const SITE_FLOCK_POINT_Y: &str = "Objects::add_flock+0x3e9";
/// …and the second the `x`, each `& 0x1ff`.
pub const SITE_FLOCK_POINT_X: &str = "Objects::add_flock+0x411";

/// `div_3_table[v >> 8]`, the cell of a world coordinate as the original
/// reads it, clamped into the world the way `WorldData::restrict@006b50a0`
/// does.
fn restricted_cell(sim: &Sim, p: Pos) -> Cell {
    let c = p.cell();
    Cell::new(
        c.x.clamp(0, sim.world.width() - 1),
        c.y.clamp(0, sim.world.height() - 1),
    )
}

impl Sim {
    /// Is `u` a flock's bird?
    pub(crate) fn is_flockbird(&self, u: usize) -> bool {
        self.units[u]
            .ty
            .and_then(|t| self.unit_types[t].tree)
            .is_some_and(|t| t == FLOCKBIRD)
    }

    /// **`do_group_move`'s wood arm** (`5e83f0`–`5e84cb`), for a follower
    /// whose slot `at` — the order's `dest_x/dest_y`, this crate's
    /// waypoint — `invalid_loc` has just refused. Within `0x300` Manhattan
    /// of the unit, the slot's cell (`restrict`ed) a `FOREST` (`WData.flags
    /// & 0x20`), and the unit in a group (`UnitData +0x80 ≥ 0`): roll the
    /// flock's size and send it up from that cell's centre tile for the
    /// group order's `whose`. The follower is ungrouped after, by its
    /// caller, whichever way this went.
    pub(crate) fn group_flock(&mut self, u: usize, at: Pos) {
        let here = self.units[u].pos;
        if (at.x - here.x).abs() + (at.y - here.y).abs() >= 0x300 {
            return;
        }
        let c = restricted_cell(self, at);
        if self.world.cell_data(c).flags & cell::FOREST == 0 {
            return;
        }
        if self.units[u].group_ptr.is_none() {
            return;
        }
        self.mark(SITE_GROUP_FLOCK);
        // `and $0x80000001` with the sign fix: `% 2`, on a roll that is
        // never negative.
        let n = self.rng.roll() % 2 + 2;
        let who = self.units[u].owner;
        self.add_flock(Pos::new(c.x * 4 + 2, c.y * 4 + 2), Some(who), n);
    }

    /// **`Objects::add_flock(x, y, who, n)`** — `x`, `y` in tiles (the
    /// cell is `>> 2`), `who` the player whose stamp gates it or `None`
    /// for the −1 `Object::take_damage` passes, `n` the birds.
    ///
    /// The four cells, in the order they are listed: the given one; it
    /// jittered by `% 3 − 1` on each axis; that spread by `% 5 − 2`; and
    /// the first `FOREST` cell round the spread one, from a random index
    /// under `circle_radius[3]` up to `circle_radius[8]`, or the given one
    /// again when there is none. Every cell is clamped into the world.
    /// Then a heading, `(% 12) × 30°`, and each bird: made at the tile's
    /// centre (`x · 0xc0 + 0x60`), turned to the heading, the heading
    /// stepped `(% 5 + 3)` degrees on for the next, and one air attack on
    /// the ground per cell, at a random point of it — **each linked in
    /// behind the others**, so the bird flies the cells in the order they
    /// were listed and lands on the wood. The ring's current order is
    /// `field_0xdc->prev` (`+0x4`, as `land_plane` reads it) and the new
    /// node becomes `field_0xdc`, so the first linked stays current;
    /// run500's proxied `do_air_physics` flies the three birds at the
    /// given cell's points first on 8787.
    ///
    /// The figure is put 200 over the ground (`update_z`, then `z +
    /// 200`), which is where its `pitch_aircraft` starts from.
    pub(crate) fn add_flock(&mut self, t: Pos, who: Option<Player>, n: i32) {
        if let Some(w) = who
            && let Some(stamp) = self.flock_stamp.get_mut(w as usize)
        {
            if *stamp + FLOCK_GAP > self.frame {
                return;
            }
            *stamp = self.frame;
        }
        let (w, h) = (self.world.width(), self.world.height());
        let clamp = |x: i32, y: i32| (x.max(0).min(w - 1), y.max(0).min(h - 1));
        let (cx, cy) = (t.x >> 2, t.y >> 2);
        let mut cells = vec![(cx, cy)];
        self.mark(SITE_FLOCK_JITTER_X);
        let jx = cx - 1 + self.rng.roll() % 3;
        self.mark(SITE_FLOCK_JITTER_Y);
        let jy = cy - 1 + self.rng.roll() % 3;
        let (jx, jy) = clamp(jx, jy);
        cells.push((jx, jy));
        self.mark(SITE_FLOCK_SPREAD_X);
        let kx = jx + self.rng.roll() % 5 - 2;
        self.mark(SITE_FLOCK_SPREAD_Y);
        let ky = jy + self.rng.roll() % 5 - 2;
        let (kx, ky) = clamp(kx, ky);
        cells.push((kx, ky));
        let circle = crate::ai_place::circle();
        let r3 = circle.radius[3] as i32;
        let mut i = if r3 - 1 > 0 {
            self.mark(SITE_FLOCK_RING);
            (self.rng.roll() % r3) as usize
        } else {
            0
        };
        let mut wood = (cx, cy);
        while i < circle.radius[8] {
            let (x, y) = (circle.x[i] + kx, circle.y[i] + ky);
            if x >= 0
                && y >= 0
                && x < w
                && y < h
                && self.world.cell_data(Cell::new(x, y)).flags & cell::FOREST != 0
            {
                wood = (x, y);
                break;
            }
            i += 1;
        }
        cells.push(wood);
        self.mark(SITE_FLOCK_ANGLE);
        let mut angle = (self.rng.roll() % 12).wrapping_mul(0x1555_5555);
        let ty = self
            .unit_types
            .iter()
            .position(|u| u.tree == Some(FLOCKBIRD));
        let at = Pos::new(
            t.x * UNITS_PER_TILE + UNITS_PER_TILE / 2,
            t.y * UNITS_PER_TILE + UNITS_PER_TILE / 2,
        );
        for _ in 0..n {
            // `init_unit` failing (`js 65c651`) skips the bird whole, its
            // draws with it.
            let Some(b) = ty.and_then(|ty| self.flockbird(ty, at)) else {
                continue;
            };
            // `Unit::set_angle(bird, angle, ·, 1)`.
            self.unit_set_angle(b, Angle(angle));
            self.units[b].movement.facing = Angle(angle);
            self.mark(SITE_FLOCK_TURN);
            angle = angle.wrapping_add((self.rng.roll() % 5 + 3).wrapping_mul(0x00b6_0b60));
            // `Unit::update_z`: the figure on `find_data_z` of its point;
            // then `last_z` that, and `z` 200 over it (`65c45b`–`65c48b`).
            let p = self.units[b].pos;
            let ground = self.world.data_z(p.x, p.y).0;
            let af = &mut self.units[b].airframe;
            af.last_z = ground;
            af.z = ground + 200;
            for &(px, py) in &cells {
                self.mark(SITE_FLOCK_POINT_Y);
                let oy = self.rng.roll() & 0x1ff;
                self.mark(SITE_FLOCK_POINT_X);
                let ox = self.rng.roll() & 0x1ff;
                // `AIR_ATTACK_GROUND` from the recycler: the point, no
                // target, `cruising_alt` 0x640, the action bit cleared,
                // and linked in at the ring's tail.
                let order = Order {
                    flags: 0,
                    body: Body::AirAttackGround(AirAttackGroundOrder {
                        at: Pos::new(
                            px * UNITS_PER_CELL + 0x80 + ox,
                            py * UNITS_PER_CELL + 0x80 + oy,
                        ),
                        home: None,
                        cruising_alt: CRUISING_ALT,
                        sharp_turn: 0,
                        returning: false,
                    }),
                };
                self.units[b].orders.push_back(order);
                self.update_action(b);
            }
        }
    }

    /// `Objects::init_unit(9, FLOCKBIRD, x, y, −1, −1, −1)` — gaia's bird,
    /// made as a dock makes its gull (`crate::transport`): the lowest free
    /// slot, the type's own speed and turn, and `Guy::init_real`'s draw.
    /// `Unit::init@00612100:69–72` seats it on the 48-unit grid's centre,
    /// `div_3_table[v >> 4] · 0x30 + 0x18`: run500's birds are born 24 on
    /// from the tile centre `add_flock` passes.
    fn flockbird(&mut self, ty: usize, at: Pos) -> Option<usize> {
        let at = Pos::new(
            crate::combat::snap(at.x) + 0x18,
            crate::combat::snap(at.y) + 0x18,
        );
        let index = self.find_free(9, crate::UNIT_BASE, crate::BUILD_BASE)?;
        let mut unit = crate::Unit::new(9, index, at, self.unit_types[ty].hits);
        unit.kind = self.unit_types[ty].kind;
        unit.ty = Some(ty);
        unit.type_index = self.unit_types[ty].type_index;
        unit.movement.speed = self.type_speed(9, ty);
        unit.movement.turning = self.turning_for(ty);
        unit.stance = self.init_stance(9, ty);
        let u = self.add_unit(unit);
        self.init_guys(u, Some(ty));
        Some(u)
    }

    /// **`Unit::do_air_physics@005e86d0` for a `FLOCKBIRD`**, under
    /// `do_air_attack_ground`, which returns at once after it: this path
    /// leaves `recharging` (`+0xae`) at 1, so the attack arm is never
    /// reached.
    ///
    /// - **The redraw** (`+0xba`) on `(o + frame) & 7 == 0`: a `0x193`
    ///   passes the `is_animal() == 0 || type == 0x193` test a wild bird
    ///   fails, so it throws `cruising_alt = (% 7 + 13) · 100`.
    /// - No `check_fuel` (an animal). `recharging = 0`, and on the stack's
    ///   **last** order the flight is a return (`AirOrder +0x18` set).
    /// - Returning, inside a step and a half of the (unclamped) point:
    ///   `land_plane`, which for an animal is `Object::die`.
    /// - The heading as a plane's: keep it owing over 45° within
    ///   `min_range · 0xc0 + 0x300` (`0x300` returning), the edge turn,
    ///   `bank_aircraft` and `pitch_aircraft` with the animal's arms
    ///   ([`Sim::bank_plane`], [`Sim::pitch_plane`]) — the aim `z` the
    ///   ground under the bird on a return and 0 otherwise, and a return
    ///   inside `0x600` at half speed — the step, the edge coin.
    /// - `set_anim(CHAR_WALK, 0, 1)`.
    /// - Then the tail: returning cleared, `recharging = 1`, and within
    ///   `0x30` Manhattan of the point with an order behind this one,
    ///   `kill_current_order`.
    pub(crate) fn flock_air_physics(&mut self, u: usize, goal: Pos, frame: i64) -> Flew {
        let Some(air) = self.current_air(u) else {
            return Flew::Done;
        };
        if (i64::from(self.units[u].index) + frame) & 7 == 0 {
            self.mark(SITE_AIR_ALT);
            let alt = (self.rng.roll() % 7 + 13) * 100;
            self.set_cruising_alt(u, alt);
        }
        self.units[u].path.clear();
        self.units[u].combat.recharging = 0;
        let returning = air.returning || self.units[u].orders.len() == 1;
        if returning {
            self.set_returning(u, true);
        }
        let xs = self.world.width() * UNITS_PER_CELL;
        let ys = self.world.height() * UNITS_PER_CELL;
        let (gx, gy) = (goal.x.clamp(0, xs - 1), goal.y.clamp(0, ys - 1));
        let clamped = (gx, gy) != (goal.x, goal.y);
        let at = self.units[u].pos;
        let (dx, dy) = (gx - at.x, gy - at.y);
        let mut speed = self.get_speed(u, 1);
        if self.ai_speed > 1 {
            speed *= self.ai_speed;
        }
        if returning && dx.abs() + dy.abs() < speed * 3 / 2 && !clamped {
            self.flockbird_dies(u);
            return Flew::Done;
        }
        self.units[u].path.push(PathData {
            to: Pos::new(gx, gy),
            tolerance: 0,
            flags: 0,
        });
        let heading = self.units[u].movement.heading;
        let mut des = find_angle(dx, dy);
        let radius = if returning {
            0
        } else {
            self.profile(crate::combat::Obj::Unit(u)).min_range * 0xc0
        };
        if owed(heading, des) > HALF_QUARTER && vector_dist(dx, dy) < radius + 0x300 {
            des = heading;
        }
        let turn = air.sharp_turn;
        if turn != 0 {
            des = Angle(heading.0.wrapping_add(turn.wrapping_shl(30)));
        }
        self.bank_plane(u, des, &mut speed, returning);
        let aim_z = if returning {
            self.world.tile_z(at.tile())
        } else {
            0
        };
        self.pitch_plane(u, dx, dy, aim_z, &mut speed, returning);
        let heading = self.units[u].movement.heading;
        self.units[u].movement.facing = heading;
        let nx = at.x + sin_component(heading, speed);
        let ny = at.y - cos_component(heading, speed);
        if nx >= 0 && ny >= 0 && nx < xs && ny < ys {
            self.set_sharp_turn(u, 0);
        } else if turn == 0 {
            self.mark(SITE_AIR_TURN);
            let t = if self.rng.roll() & 1 != 0 { 1 } else { -1 };
            self.set_sharp_turn(u, t);
        }
        let to = Pos::new(nx.clamp(0, xs - 1), ny.clamp(0, ys - 1));
        self.set_new_location(u, to, false);
        self.set_anim(u, crate::anim::WALK, false, true);
        self.set_returning(u, false);
        self.units[u].combat.recharging = 1;
        let now = self.units[u].pos;
        if (gx - now.x).abs() + (gy - now.y).abs() < 0x30 && self.units[u].orders.len() > 1 {
            self.kill_current_order(u);
            return Flew::Done;
        }
        Flew::On
    }

    /// `land_plane` for an animal: `Object::die(0, −1, 0.0)`, as a
    /// missile's end is ([`Sim::missile_dies`]'s shape, less the
    /// population a bird never held).
    fn flockbird_dies(&mut self, u: usize) {
        self.units[u].health = self.units[u].health.min(0);
        self.relink_squad(u);
        self.close_dead_orders(u);
        self.hold_dead_slot(u);
        self.forget(crate::combat::Obj::Unit(u));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::Rng;

    fn sim() -> Sim {
        let mut s = Sim::new(
            crate::tuning::Tuning::RON,
            crate::world::World::new(60, 60),
            2,
        );
        s.rng = Rng::new(7);
        let t = s.add_unit_type(crate::UnitType {
            hits: 1,
            moves: 35,
            turn_speed: crate::movement::degrees_to_angle(5).0,
            kind: crate::attrition::UnitKind {
                domain: crate::attrition::Domain::Air,
                ..crate::attrition::UnitKind::default()
            },
            ..crate::UnitType::default()
        });
        // Named after the fact: the fixture's tech tree has no row 0x193.
        s.unit_types[t].tree = Some(FLOCKBIRD);
        s
    }

    fn rolled(mut r: Rng, n: usize) -> Rng {
        for _ in 0..n {
            let _ = r.roll();
        }
        r
    }

    fn birds(s: &Sim) -> Vec<usize> {
        (0..s.units.len())
            .filter(|&u| s.units[u].owner == 9 && s.is_flockbird(u))
            .collect()
    }

    /// **Six draws for the flock and ten a bird, and each bird flies the
    /// cells in the order they were listed** — the given one first, the
    /// wood last (run500's proxied `do_air_physics` on 8787). Each bird is
    /// born 24 on from the tile centre, on the 48-unit grid (run500's
    /// `set_new_location`), and a player's second flock inside 225 frames
    /// spends nothing and makes nothing.
    #[test]
    fn a_flock_is_six_draws_and_ten_a_bird_flown_first_cell_first() {
        let mut s = sim();
        s.frame = 1000;
        let before = s.rng;
        let (cx, cy) = (10, 12);
        s.add_flock(Pos::new(cx * 4 + 2, cy * 4 + 2), Some(1), 3);
        assert_eq!(s.rng, rolled(before, 6 + 3 * 10), "36 draws");
        let b = birds(&s);
        assert_eq!(b.len(), 3);
        for &u in &b {
            assert_eq!(s.units[u].orders.len(), 4);
            // `(4c + 2) · 0xc0 + 0x60`, snapped: `c · 768 + 504`.
            assert_eq!(
                s.units[u].pos,
                Pos::new(cx * 768 + 504, cy * 768 + 504),
                "born on the grid's centre"
            );
            let Body::AirAttackGround(g) = s.units[u].orders[0].body else {
                panic!("an air attack on the ground");
            };
            assert_eq!(
                (g.at.x - 0x80).div_euclid(UNITS_PER_CELL),
                cx,
                "the given cell is flown first"
            );
            assert_eq!((g.at.y - 0x80).div_euclid(UNITS_PER_CELL), cy);
            assert_eq!(s.units[u].airframe.z, s.units[u].airframe.last_z + 200);
        }
        assert_eq!(s.flock_stamp[1], 1000);
        s.frame = 1000 + FLOCK_GAP - 1;
        let before = s.rng;
        s.add_flock(Pos::new(cx * 4 + 2, cy * 4 + 2), Some(1), 3);
        assert_eq!(s.rng, before, "a refused flock spends nothing");
        assert_eq!(birds(&s).len(), 3);
        s.frame = 1000 + FLOCK_GAP;
        s.add_flock(Pos::new(cx * 4 + 2, cy * 4 + 2), Some(1), 2);
        assert_eq!(birds(&s).len(), 5, "225 frames on, the next one flies");
    }

    /// **The group move's wood arm**: a slot in a `FOREST` cell within
    /// `0x300` of a unit in a group rolls the flock's size and sends it up
    /// (`5e83f0`–`5e84cb`); out of a group, or on open ground, it spends
    /// nothing.
    #[test]
    fn a_slot_in_a_wood_flushes_a_flock_from_a_unit_in_a_group() {
        let mut s = sim();
        s.frame = 1000;
        let here = Pos::new(10 * 768 + 400, 12 * 768 + 400);
        let mut unit = crate::Unit::new(1, 5, here, 10);
        unit.group_ptr = Some(1);
        let u = s.add_unit(unit);
        let slot = Pos::new(10 * 768 + 600, 12 * 768 + 300);
        let before = s.rng;
        s.group_flock(u, slot);
        assert_eq!(s.rng, before, "open ground: nothing");
        let c = Cell::new(10, 12);
        let mut d = s.world.cell_data(c);
        d.flags |= cell::FOREST;
        s.world.set_cell_data(c, d);
        s.units[u].group_ptr = None;
        s.group_flock(u, slot);
        assert_eq!(s.rng, before, "in no group: nothing");
        s.units[u].group_ptr = Some(1);
        s.group_flock(u, Pos::new(slot.x + 0x300, slot.y));
        assert_eq!(s.rng, before, "0x300 away: nothing");
        s.group_flock(u, slot);
        let n = birds(&s).len();
        assert!(n == 2 || n == 3, "two or three birds, {n}");
        assert_eq!(
            s.rng,
            rolled(before, 1 + 6 + 10 * n),
            "the size, then the flock"
        );
    }
}
