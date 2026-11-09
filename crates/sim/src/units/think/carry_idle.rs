//! `Unit::think_carry_idle@005f2f30` as steps over a view (item 3411): a
//! boat carrying nothing it can unload looks for **its owner's nearest
//! coastal city with a dock** and walks to a free spot beside it.
//! [`think_carry_idle_steps`] is the original's control flow in its order;
//! [`CarryIdleView`] is what it reads and the callees it makes, which
//! `sweep/unit_think_carry_idle.rs` answers from the emulated original's
//! windows.
//!
//! Not wired: the carry dispatcher's idle arm still answers `false`. A live
//! view needs the crate's city list in the original's order, the dock test
//! (`CityData::get_building(DOCK)`) and a mapping of the 16-argument
//! `UnitType::find_nearby_spot` onto `Sim::find_nearby_spot`, none of which
//! this item establishes.
//!
//! Established by the sweep (and nothing else): the scan keeps the first city
//! of the least distance (`<`, never `<=`), the first test is `-1`, not a
//! sign; a city counts only when its open bit, `is_coast(my, city region)`
//! and a dock all hold, in that order; the spot is refused when it falls in
//! the unit's own cell **on both axes** (cells being `div_3_table[v >> 8]`).

use crate::movement::{Angle, find_angle};
use crate::world::{Pos, vector_dist};

/// What `think_carry_idle` reads of the unit and the world, and the callees
/// it makes.
pub trait CarryIdleView {
    /// The unit's position (`+0x10`, `+0x14`, the stored words xor `0x63637`).
    fn pos(&self) -> Pos;
    /// `div_3_table[i]`.
    fn div3(&mut self, i: i32) -> i32;
    /// `WorldData::get_tregion(&tx, &ty)` of the unit's tile.
    fn tregion(&mut self, tx: i32, ty: i32) -> i32;
    /// The owner's city count (`leaders.list[owner].field_0x408`).
    fn num_cities(&self) -> i32;
    /// The city's flag byte (`+4`) bit 0.
    fn city_open(&self, i: i32) -> bool;
    /// The city's region short (`+0xa`), sign-extended.
    fn city_region(&self, i: i32) -> i32;
    /// `Region::is_coast(regions[my], city_region)`.
    fn is_coast(&mut self, my: i32, city_region: i32) -> bool;
    /// `CityData::get_building(DOCK) != -1`.
    fn has_dock(&mut self, i: i32) -> bool;
    /// The city's position (`+0xc`, `+0x10`).
    fn city_pos(&self, i: i32) -> Pos;
    /// `UnitType::find_nearby_spot` about city `i`: `Some` when it answers 0.
    fn nearby_spot(&mut self, i: i32) -> Option<Pos>;
    /// The order's birth, up to its `MOVE_TO`: the unit's `0x4000000` bit
    /// cleared, `+0xc0` zeroed, `close_orders(0)`, `clear_partial_path`,
    /// `update_action`; then the order built, queued and `update_action` again.
    fn order_move_to(&mut self, to: Pos, angle: Angle);
}

/// The original's control flow.
pub fn think_carry_idle_steps<V: CarryIdleView>(v: &mut V) -> bool {
    let pos = v.pos();
    let tx = v.div3(pos.x >> 6);
    let ty = v.div3(pos.y >> 6);
    let my = v.tregion(tx, ty);
    let mut best = -1;
    let mut best_city = -1;
    let mut i = 0;
    while i < v.num_cities() {
        if v.city_open(i) {
            let region = v.city_region(i);
            if v.is_coast(my, region) && v.has_dock(i) {
                let c = v.city_pos(i);
                let d = vector_dist(pos.x.wrapping_sub(c.x), pos.y.wrapping_sub(c.y));
                if best == -1 || d < best {
                    best = d;
                    best_city = i;
                }
            }
        }
        i += 1;
    }
    if best_city < 0 {
        return false;
    }
    let Some(spot) = v.nearby_spot(best_city) else {
        return false;
    };
    if v.div3(spot.x >> 8) == v.div3(pos.x >> 8) && v.div3(spot.y >> 8) == v.div3(pos.y >> 8) {
        return false;
    }
    let angle = find_angle(spot.x.wrapping_sub(pos.x), spot.y.wrapping_sub(pos.y));
    v.order_move_to(spot, angle);
    true
}
