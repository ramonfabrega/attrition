//! **`UnitData::speed@0060aae0`'s steps**, as a walk over a
//! [`UnitSpeedView`] (item 3399): [`crate::Sim::unit_speed`] is this walk
//! over the unit's slot, and the sweep's port is this walk over the
//! emulated original's windows. (`UnitData::get_speed@006086f0`, which
//! item 3390 gave its Monarch siege arm, is the layer above and a different
//! function.)
//!
//! The cached `myspeed` is the answer for a type not of the land. Past it,
//! the **Iroquois spear** (a unit of the Spearmen line standing on a cell
//! whose owner its leader allies adds the entry of the first strict
//! `is(0xa6..0xa9)` it answers), then the **Forced March** under the
//! leader's `0x8000`: the hero is the unit itself when it is active, a hero
//! and its own `units` slot carries `0x8000`, else `HeroesData::find_hero`'s
//! answer; with one, the speed is `FORCED_MARCH_SPEED × UNIT_MOVE_SPEED`
//! (times `ALEXANDER_FORCED_MARCH_SPEED`, shifted out by 8, when the hero
//! or the unit `is(0x166)` or `is(0x167)`), or `myspeed` when that is
//! larger.
//!
//! SEAM: the hero auras after it (Spitamenes, Blucher, Porus, Charles,
//! Napoleon, each behind a leader's power word `+0x59cc..+0x59e8` and a
//! constant the crate does not carry) are not walked; the sweep holds the
//! power words zero.

/// What `UnitData::speed` asks of the unit, its type, its leader and the
/// world.
pub trait UnitSpeedView {
    /// The short at `+0x9a`.
    fn myspeed(&self) -> i32;
    /// The type record's word at `+0x218` is not zero.
    fn not_land(&self) -> bool;
    /// `this`'s vslot `+0xb8`, `ObjectData::is(x, strict)`.
    fn is_this(&mut self, x: i32, strict: bool) -> bool;
    /// The owner byte of the cell the unit stands in (`worldc +0x134`, `+0xf`
    /// of a 0x1c record), `None` when negative.
    fn ground_owner(&mut self) -> Option<i32>;
    /// `LeaderData::is_ally(leader, other)`.
    fn is_ally(&mut self, other: i32) -> bool;
    /// `IROQUOIS_SPEAR_SPEED[k]`.
    fn spear_speed(&self, k: usize) -> i32;
    /// The leader's flag word `& 0x8000`.
    fn leader_marching(&self) -> bool;
    /// `this`'s vslot `+0x18`.
    fn is_active(&mut self) -> bool;
    /// `this`'s vslot `+0xc4`, `UnitData::is_hero`.
    fn is_hero(&mut self) -> bool;
    /// The `units` slot at `this`'s own index carries `0x8000`.
    fn own_slot_marching(&mut self) -> bool;
    /// `this`'s own index (the short at `+0xa`).
    fn own_slot(&self) -> i32;
    /// `HeroesData::find_hero(.., 0x8000, -1, ..)`: a hero's slot, negative
    /// for none.
    fn find_hero(&mut self) -> i32;
    /// `is(x, 0)` asked of the hero object at `slot`.
    fn hero_is(&mut self, slot: i32, x: i32) -> bool;
    /// `FORCED_MARCH_SPEED`.
    fn forced_march_speed(&self) -> i32;
    /// `ALEXANDER_FORCED_MARCH_SPEED`, 8.8.
    fn alexander_forced_march_speed(&self) -> i32;
    /// `UNIT_MOVE_SPEED`.
    fn unit_move_speed(&self) -> i32;
}

/// `UnitData::speed@0060aae0`.
pub fn unit_speed<V: UnitSpeedView>(v: &mut V) -> i32 {
    let base = v.myspeed();
    if v.not_land() {
        return base;
    }
    let mut speed = base;
    if v.is_this(0xa6, false)
        && let Some(o) = v.ground_owner()
        && v.is_ally(o)
    {
        for k in 0..4 {
            if v.is_this(0xa6 + k as i32, true) {
                speed = speed.wrapping_add(v.spear_speed(k));
                break;
            }
        }
    }
    if !v.leader_marching() {
        return speed;
    }
    let hero = if v.is_active() && v.is_hero() && v.own_slot_marching() {
        v.own_slot()
    } else {
        v.find_hero()
    };
    if hero < 0 {
        return speed;
    }
    let alexander = v.hero_is(hero, 0x166)
        || v.is_this(0x166, false)
        || v.hero_is(hero, 0x167)
        || v.is_this(0x167, false);
    let march = if alexander {
        let p = v
            .forced_march_speed()
            .wrapping_mul(v.alexander_forced_march_speed())
            .wrapping_mul(v.unit_move_speed());
        p.wrapping_add((p >> 31) & 0xff) >> 8
    } else {
        v.forced_march_speed().wrapping_mul(v.unit_move_speed())
    };
    march.max(base)
}
