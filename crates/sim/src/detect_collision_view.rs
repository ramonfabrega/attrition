//! **`Unit::detect_unit_collision@00617060`** (item 3402): may this unit stand
//! on the point it proposes, and if not, who is in the way — as a walk over a
//! [`DetectCollisionView`].
//!
//! **The port is `Sim::detect_unit_collision` (`collide.rs`, `docs/COLLISION.md`
//! §4)**, which runs this walk through `SimDetect` with the arguments its
//! callers leave it: `quick 0`, `top_only 1` (the second arm, §13.2, is the
//! callers'), `nocoll 0`. The crate's `detect_quick` and `blocker_still_there`
//! are the `quick` form collapsed and do not come through here.
//!
//! In the order the original runs it (the numbers are the listing's):
//!
//! 1. **gates.** An air type (`+0x218 == 2`) leaves at once. Unless
//!    `top_only`, a sea type, a siege type (type vslot `0x10c`), a hero
//!    (vslot `0xc4`) or a supply wagon (`0xcc`) takes the *second arm*: with
//!    `nocoll` clear it answers 0 when `quick`, when not `boats`, and when
//!    `detect_boat_collision(x, y, 1)` does. A detour waypoint at the path's
//!    tail (`+0xc0` entries of `0x10`, flags `+0xc & 8`) and `safe` (`+0xb2`)
//!    leave too. **Leaving** is `quick ? 0 : bookkeeping, 0`.
//! 2. **the probe.** The proposed point's unit cell (`>> 4`, the `div_3_table`)
//!    against the unit's own; a different cell asks
//!    `CollCheck::collide_here`, and a hit asks `will_be_corner` and, when
//!    `quick`, returns 1.
//! 3. **the overreach.** An `ATTACK` action whose `TargetOrder` still stands
//!    (vslot `4`) measures `attack_dist − max_range × 0xc0`, kept when positive.
//! 4. **the scan.** The nine world cells around the point (`move_x`/`move_y`,
//!    ring 1), each cell's `down` chain: not me, live (object vslot `0x18`),
//!    not an air type, on the map (vslot `0xbc`), and `is_here` the hit cell.
//!    A candidate is **soft** by the caravan pair, the guard escort, the
//!    attackers pair (`extra > 0x300`) or the group arm, and then only marks
//!    the walk; otherwise a corner pair (`|will − is_corner| == 4`) is
//!    skipped and anything else is **hard**.
//! 5. **a hard hit** records itself (`+0x8a`, `+0xb3`, `+0x8c`), returns 1 when
//!    `nocoll` or the front order is not moving, and otherwise stamps the
//!    move's `+0x3c`/`+0x40` with the point.
//! 6. **no hard hit**: a soft one sets `+0x68 |= 0x100000`; then the
//!    bookkeeping (`+0xb3 = 0xff`, `+0x8a = -1`, `+0x88 = 0` when `+0x48` is
//!    over five frames old, `+0x68 &= !0x40`) and 0.
//!
//! **Contract** (not established beyond it): a *hard* hit with `!nocoll` has a
//! front order (the original reads through a null list otherwise); the point
//! and the unit are in the `div_3_table`'s range; the chain ends.

/// The arguments after `this`: `Coord x, y`, then `quick`, `boats`, an unused
/// fifth, `nocoll` and `top_only`.
#[derive(Clone, Copy, Debug)]
pub struct DetectArgs {
    pub x: i32,
    pub y: i32,
    pub quick: bool,
    pub boats: bool,
    pub nocoll: bool,
    pub top_only: bool,
}

/// What the walk reads of the world and does to it. `Node` is a place in a
/// world cell's chain: an object `(o, who)`, or the end (`o < 0`).
pub trait DetectCollisionView {
    type Node: Copy;

    /// The type record's domain (`+0x218`): 1 sea, 2 air.
    fn domain(&mut self) -> i32;
    /// The type's vslot `0x10c`.
    fn type_is_siege(&mut self) -> bool;
    /// The unit's vslot `0xc4` (`is_hero`; inline `type +0x2b8 & 0x20`).
    fn is_hero(&mut self) -> bool;
    /// The unit's vslot `0xcc` (`is_supply`; inline `type +0x2b8 & 0x40`).
    fn is_supply(&mut self) -> bool;
    /// `Unit::detect_boat_collision(x, y, 1)`.
    fn detect_boat_collision(&mut self, x: i32, y: i32) -> bool;
    /// The path's tail node carries the detour flag (`& 8`); no path, no.
    fn path_detour(&mut self) -> bool;
    /// `+0xb2`.
    fn is_safe(&mut self) -> bool;
    /// The unit's own unit cell (`div_3_table[pos >> 4]`).
    fn own_cell(&mut self) -> (i32, i32);
    /// `CollCheck::collide_here(o, who, cell, size, &hit, nocoll)`.
    fn collide_here(&mut self, cell: (i32, i32), nocoll: bool) -> Option<(i32, i32)>;
    /// `UnitData::will_be_corner(hit, cell)`.
    fn will_be_corner(&mut self, hit: (i32, i32), cell: (i32, i32)) -> i32;
    /// `UnitData::action_type`.
    fn action_type(&mut self) -> i32;
    /// The action's target order (vslot `0xcc`) answers vslot `4` (and its
    /// vslot `0` is called unless it is `TargetOrder::get_target`).
    fn attack_target_exists(&mut self) -> bool;
    /// The unit's vslot `0x130`.
    fn max_range(&mut self) -> i32;
    /// `ObjectData::attack_dist(target.o, target.who, x, y)`.
    fn attack_dist(&mut self, x: i32, y: i32) -> i32;
    /// The world's width and height in world cells.
    fn world_size(&mut self) -> (i32, i32);
    /// `WorldData::get_down(cell)`: the head of the cell's chain.
    fn get_down(&mut self, cell: (i32, i32)) -> Option<Self::Node>;
    /// The node's `(o, who)`; `o < 0` ends the chain.
    fn node_id(&mut self, n: Self::Node) -> (i32, i32);
    /// The node's own link (`objects[who][o] +0x2c/+0x2e`).
    fn node_next(&mut self, n: Self::Node) -> Self::Node;
    /// The node is the unit itself.
    fn node_is_me(&mut self, n: Self::Node) -> bool;
    /// The object's vslot `0x18`.
    fn node_alive(&mut self, n: Self::Node) -> bool;
    /// The node's unit has an air type.
    fn node_air(&mut self, n: Self::Node) -> bool;
    /// The node's unit's vslot `0xbc` (`is_on_map`).
    fn node_on_map(&mut self, n: Self::Node) -> bool;
    /// `UnitData::is_here(hit)`.
    fn node_is_here(&mut self, n: Self::Node, hit: (i32, i32)) -> bool;
    /// `get_action` of the node's unit, and its vslot `0x10`; 0 with none.
    fn node_action_type(&mut self, n: Self::Node) -> i32;
    /// The unit's vslot `0xd8` (`is_moving`: its front order's vslot `0x14`).
    fn is_moving_me(&mut self) -> bool;
    /// The node's unit's vslot `0xd8`.
    fn node_is_moving(&mut self, n: Self::Node) -> bool;
    /// `update_action` of the node's unit, vslot `0x74`: its `+8`/`+0xc` name
    /// this unit.
    fn guard_targets_me(&mut self, n: Self::Node) -> bool;
    /// The node's owner is this unit's (`who == +0x9`).
    fn node_same_owner(&mut self, n: Self::Node) -> bool;
    /// The type's `+0x248`.
    fn my_block(&mut self) -> i32;
    /// The node's unit's type's `+0x248`.
    fn node_block(&mut self, n: Self::Node) -> i32;
    /// The two units share a group slot (`+0x80`), and it is not `-1`.
    fn groups_shared(&mut self, n: Self::Node) -> bool;
    /// The node's unit has a suspended search (`+0x104`).
    fn node_searching(&mut self, n: Self::Node) -> bool;
    /// The node's front order's vslot `0x10`, after the iterator refresh;
    /// `None` when it holds no orders (`+0xdc == 0`).
    fn node_front_order(&mut self, n: Self::Node) -> Option<i32>;
    /// A `CAST_SPELL` front order: its spell type is the base `SpellType` and
    /// one of the four packs, or another class and its vslot `0x50` answers.
    fn node_cast_soft(&mut self, n: Self::Node) -> bool;
    /// `UnitData::is_corner(hit)` of the node's unit.
    fn node_corner(&mut self, n: Self::Node, hit: (i32, i32)) -> i32;
    /// The hit's record: `+0x8a = o`, `+0xb3 = who`, `+0x8c = 0`.
    fn record_hit(&mut self, n: Self::Node);
    /// The unit's front order's vslot `0x14` (after the refresh).
    fn front_is_moving(&mut self) -> bool;
    /// The front order's vslot `0x40` record, `+0x3c = x`.
    fn set_move_x(&mut self, x: i32);
    /// The same, `+0x40 = y`.
    fn set_move_y(&mut self, y: i32);
    /// `+0x68 |= 0x100000`.
    fn set_half_step(&mut self);
    /// `+0xb3 = 0xff`, `+0x8a = -1`.
    fn clear_collider(&mut self);
    /// `+0x48`.
    fn collide_frame(&mut self) -> i64;
    /// `GameAccess::game->frame`.
    fn frame(&mut self) -> i64;
    /// `+0x88 = 0`.
    fn reset_collide_count(&mut self);
    /// `+0x68 &= !0x40`.
    fn clear_wait_flag(&mut self);
}

/// `Domain` of the type record.
pub const DOMAIN_SEA: i32 = 1;
pub const DOMAIN_AIR: i32 = 2;
/// `UnitData::action_type` / `get_type` values the walk tests.
pub const ACTION_ATTACK: i32 = 10;
pub const ACTION_GUARD: i32 = 0xc;
pub const ACTION_CAST: i32 = 0xe;
pub const ACTION_TRADE_ROUTE: i32 = 0xf;

/// `(v >> 4) / 3`: `div_3_table` of a coordinate, the unit cell.
pub const fn unit_cell_of(v: i32) -> i32 {
    (v >> 4).div_euclid(3)
}

/// `(v >> 8) / 3`: the world cell.
pub const fn world_cell_of(v: i32) -> i32 {
    (v >> 8).div_euclid(3)
}

/// The nine cells of ring 1, `move_x`/`move_y` (`00adcaf0`/`00adc400`), the
/// centre first.
pub const RING1: [(i32, i32); 9] = [
    (0, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
];

/// The leave-without-colliding exit: `quick` returns at once, otherwise the
/// bookkeeping runs first.
fn leave<V: DetectCollisionView>(v: &mut V, quick: bool) -> (i32, Option<V::Node>) {
    if !quick {
        bookkeeping(v);
    }
    (0, None)
}

/// `617802`-`617827`: nobody was named.
fn bookkeeping<V: DetectCollisionView>(v: &mut V) {
    v.clear_collider();
    if v.collide_frame() < v.frame().wrapping_sub(5) {
        v.reset_collide_count();
    }
    v.clear_wait_flag();
}

/// Runs `Unit::detect_unit_collision(this, x, y, quick, boats, 0, nocoll,
/// top_only)`: the answer (`eax`) and, on a hard hit, the node it named.
pub fn detect_unit_collision_steps<V: DetectCollisionView>(
    v: &mut V,
    a: DetectArgs,
) -> (i32, Option<V::Node>) {
    let domain = v.domain();
    if domain == DOMAIN_AIR {
        return leave(v, a.quick);
    }
    if !a.top_only {
        let second_arm = domain == DOMAIN_SEA || v.type_is_siege() || v.is_hero() || v.is_supply();
        if second_arm && !a.nocoll {
            if a.quick || !a.boats {
                return (0, None);
            }
            if v.detect_boat_collision(a.x, a.y) {
                return (0, None);
            }
        }
    }
    if v.path_detour() || v.is_safe() {
        return leave(v, a.quick);
    }
    let cell = (unit_cell_of(a.x), unit_cell_of(a.y));
    let own = v.own_cell();
    if cell == own {
        return leave(v, a.quick);
    }
    let Some(hit) = v.collide_here(cell, a.nocoll) else {
        return leave(v, a.quick);
    };
    let will = v.will_be_corner(hit, cell);
    if a.quick {
        return (1, None);
    }
    let (wx, wy) = (world_cell_of(a.x), world_cell_of(a.y));
    let my_action = v.action_type();
    let attacking = my_action == ACTION_ATTACK;
    let mut extra = 0;
    if attacking && v.attack_target_exists() {
        let range = v.max_range();
        let d = v.attack_dist(a.x, a.y);
        let over = d.wrapping_sub(range.wrapping_mul(0xc0));
        extra = over.max(0);
    }
    let (w, h) = v.world_size();
    let mut soft = false;
    for (dx, dy) in RING1 {
        let c = (wx + dx, wy + dy);
        if c.0 < 0 || c.1 < 0 || c.0 >= w || c.1 >= h {
            continue;
        }
        let Some(mut n) = v.get_down(c) else { continue };
        while v.node_id(n).0 >= 0 {
            let next = v.node_next(n);
            if !v.node_is_me(n)
                && v.node_alive(n)
                && !v.node_air(n)
                && v.node_on_map(n)
                && v.node_is_here(n, hit)
            {
                if candidate_is_soft(v, n, my_action, attacking, extra) {
                    soft = true;
                } else {
                    let hard = will == 0 || {
                        let theirs = v.node_corner(n, hit);
                        will.wrapping_sub(theirs).wrapping_abs() != 4
                    };
                    if hard {
                        v.record_hit(n);
                        if a.nocoll || !v.front_is_moving() {
                            return (1, Some(n));
                        }
                        v.set_move_x(a.x);
                        v.set_move_y(a.y);
                        return (1, Some(n));
                    }
                }
            }
            n = next;
        }
    }
    if soft {
        v.set_half_step();
    }
    bookkeeping(v);
    (0, None)
}

/// §4.3's table: the ways the unit in the way is a nudge rather than a
/// collision, in the original's order.
fn candidate_is_soft<V: DetectCollisionView>(
    v: &mut V,
    n: V::Node,
    my_action: i32,
    attacking: bool,
    extra: i32,
) -> bool {
    let theirs = v.node_action_type(n);
    if my_action == ACTION_TRADE_ROUTE && theirs == ACTION_TRADE_ROUTE {
        if v.is_moving_me() && v.node_is_moving(n) {
            return true;
        }
    } else if theirs == ACTION_GUARD {
        if v.guard_targets_me(n) {
            return true;
        }
    } else if theirs == ACTION_ATTACK
        && attacking
        && v.node_same_owner(n)
        && v.my_block() == 1
        && v.node_block(n) == 1
        && v.is_moving_me()
        && v.node_is_moving(n)
        && extra > 0x300
    {
        return true;
    }
    if v.groups_shared(n) && !attacking && !v.node_searching(n) {
        return match v.node_front_order(n) {
            None => true,
            Some(ACTION_CAST) => v.node_cast_soft(n),
            Some(0 | ACTION_GUARD) => true,
            Some(1 | 2 | 3 | 4 | 0x12 | 0x13 | 0x15) => theirs != ACTION_ATTACK,
            Some(_) => false,
        };
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `move_x`/`move_y`'s first nine entries are the crate's ring-1 spiral, which
    /// `docs/COLLISION.md` §2.1 checked against the PE.
    #[test]
    fn the_ring_is_the_spiral_s_first_nine() {
        assert_eq!(RING1.to_vec(), crate::collide::spiral(1));
    }

    /// The two table lookups are the crate's floor divisions.
    #[test]
    fn the_cell_tables_are_floor_divisions() {
        for v in [0, 1, 0x2f, 0x30, 0x2ff, 0x300, 0x1000, 0x2f00] {
            assert_eq!(
                unit_cell_of(v),
                v.div_euclid(crate::collide::UNITS_PER_UCELL)
            );
            assert_eq!(world_cell_of(v), v.div_euclid(crate::world::UNITS_PER_CELL));
        }
    }
}
