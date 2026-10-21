//! **`Unit::land_plane@005e9950`** (item 3396): a plane (or a flock's bird)
//! comes down, as a pure walk over a [`LandPlaneView`].
//!
//! **The port is `Sim::land_plane` (`air.rs`)**, which delegates here through
//! `SimLand`; the animal arm is `flock.rs`'s `flockbird_dies`, and the sweep is
//! the only caller that asks for it (`SimLand::is_animal` answers false: the
//! callers that hold a bird route it themselves).
//!
//! In the order it runs:
//!
//! 1. **an animal** (vslot `0x30`): if its vslot `0x158` is `Unit::die`
//!    (`0x60eda0`) the call is `Object::die(0, -1, 0.0)`, otherwise the slot
//!    is called with the same three arguments; either way it returns;
//! 2. **the front order's `AirOrder`** (vslot `0x84`): the home's `o`/`who`
//!    are read and `returning` (`+0x18`) is cleared;
//! 3. **a strafe** (`get_type == 0x10`) with **no target** (the strafe
//!    record's `+0x8` or `+0xc` negative) goes: `clear_orders` when its
//!    `mandatory` byte (`+0x1c`) is set, `kill_current_order(0)` otherwise;
//! 4. **the home's vslot `0xf0`** (`has_repeat_air`): clear, the unit's
//!    `+0x68` bit `0x4000000` and `+0xc0` go, `close_orders(0)`,
//!    `clear_partial_path` and `update_action` run; set, the order the unit
//!    now holds loses its action bit (`update_order()->flags &= ~4`) unless
//!    step 3 went;
//! 5. **`add_spec_anim_order(0, home.vslot_0x34(), heli ? 3 : 1, ..)`**, the
//!    new order's home `o`/`who` (vslot `0xa8`, `+0x1c`/`+0x20`), the
//!    landing sound when the home answers vslot `0x48(console.who, 0)`, and
//!    the unit's vslot `0x188` (`go_inside`).
//!
//! The crate collapses step 5's order into [`LandPlaneView::go_inside`]
//! (`Sim::go_inside`), which is the order's work run at once
//! (`docs/ORDERS.md` §33.5); the adapter makes the order add, the order's
//! home words and the sound no-ops.
//!
//! **Contract** (not established beyond it): the unit's front order is an
//! air order with a home on the objects table. A front order that is not one
//! answers 0 to vslot `0x84` and the original reads through it; the sweep
//! draws that on neither side.

/// The state `Unit::land_plane` reads and the calls it makes.
pub trait LandPlaneView {
    /// The home: the objects-table slot (`o`, `who`) the order names.
    type Home: Copy;

    /// The unit's vslot `0x30` (`is_animal`).
    fn is_animal(&mut self) -> bool;
    /// Whether the unit's vslot `0x158` is `Unit::die`.
    fn die_slot_is_unit_die(&mut self) -> bool;
    /// `Object::die(this, 0, -1, 0.0)`.
    fn object_die(&mut self);
    /// The unit's vslot `0x158`, handed `(0, -1, 0.0)`.
    fn call_die_slot(&mut self);

    /// The front order's `AirOrder` home: vslot `0x84`, then `+0x4`/`+0x8`.
    /// `None` when the order holds none (the crate's early return).
    fn air_home(&mut self) -> Option<Self::Home>;
    /// `AirOrder +0x18 = 0`.
    fn clear_returning(&mut self);

    /// The order's `get_type() == 0x10` (a strafe).
    fn is_strafe(&mut self) -> bool;
    /// The strafe has a target: record `+0x8 >= 0` and `+0xc >= 0`.
    fn strafe_has_target(&mut self) -> bool;
    /// The strafe's `mandatory` byte (`+0x1c`).
    fn strafe_mandatory(&mut self) -> bool;
    /// `Unit::clear_orders`.
    fn clear_orders(&mut self);
    /// `Unit::kill_current_order(0)`.
    fn kill_current_order(&mut self);

    /// The home's vslot `0xf0` (`has_repeat_air`).
    fn repeat_air(&mut self, home: Self::Home) -> bool;
    /// `update_order(this)->flags &= ~4`.
    fn clear_order_action_bit(&mut self);
    /// `+0x68 &= ~0x4000000`, `+0xc0 = 0`: the unit's path state.
    fn clear_path_state(&mut self);
    /// `Unit::close_orders(0)`.
    fn close_orders(&mut self);
    /// `Unit::clear_partial_path`.
    fn clear_partial_path(&mut self);
    /// `Unit::update_action`.
    fn update_action(&mut self);

    /// The type record's helicopter bit (`+0x2b4 & 0x20`).
    fn is_helicopter(&mut self) -> bool;
    /// The home's vslot `0x34`.
    fn home_slot_34(&mut self, home: Self::Home) -> i32;
    /// `Unit::add_spec_anim_order(0, data1, data2, ..)`.
    fn add_spec_anim_order(&mut self, home: Self::Home, data1: i32, data2: i32);
    /// The new current order's home words: vslot `0xa8`, `+0x1c = o`,
    /// `+0x20 = who`.
    fn set_special_home(&mut self, home: Self::Home);
    /// The home's vslot `0x48(console.who, 0)`.
    fn home_visible(&mut self, home: Self::Home) -> bool;
    /// `SubObjectData::play_sound(S_CARRIER_LANDING, x ^ 0x63637, y ^ 0x63637, o)`.
    fn play_landing_sound(&mut self, home: Self::Home);
    /// The unit's vslot `0x188` (`go_inside`).
    fn go_inside(&mut self, home: Self::Home);
}

/// `get_type` of a `StrafeOrder`.
pub const TYPE_STRAFE: i32 = 0x10;

/// Runs `Unit::land_plane(this)`.
pub fn land_plane_steps<V: LandPlaneView>(v: &mut V) {
    if v.is_animal() {
        if v.die_slot_is_unit_die() {
            v.object_die();
        } else {
            v.call_die_slot();
        }
        return;
    }
    let Some(home) = v.air_home() else {
        return;
    };
    v.clear_returning();
    let mut killed = false;
    if v.is_strafe() && !v.strafe_has_target() {
        if v.strafe_mandatory() {
            v.clear_orders();
        } else {
            v.kill_current_order();
        }
        killed = true;
    }
    if v.repeat_air(home) {
        if !killed {
            v.clear_order_action_bit();
        }
    } else {
        v.clear_path_state();
        v.close_orders();
        v.clear_partial_path();
        v.update_action();
    }
    let data2 = if v.is_helicopter() { 3 } else { 1 };
    let data1 = v.home_slot_34(home);
    v.add_spec_anim_order(home, data1, data2);
    v.set_special_home(home);
    if v.home_visible(home) {
        v.play_landing_sound(home);
    }
    v.go_inside(home);
}
