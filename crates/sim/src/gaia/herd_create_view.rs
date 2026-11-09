//! `Herd::create_units@007417f0` (herd.cpp), written from reading what it does and swept against the
//! original under the emulator (item 3391; `sweep/herd_create_units.rs`). Not wired: the harness does
//! not model the herds' founding (`rondata::diff::setup` names its 44 draws), and the crate's
//! `Sim::create_units` is the AI's train-list valuer, a namesake.
//!
//! A herd founds its animals at game start: three for type `0x19b`, one for `0x19c`, four otherwise.
//! Each is made on the herd's cell centre by `Objects::init_unit` (owner 8, the herd's type), moved to
//! the nearest free spot, turned and placed there. The turn is `0` for the first kind, the unset angle
//! (`i32::MIN`) for the second, and for every other type a random 0..65535 taken modulo 360. The first
//! `init_unit` that fails ends the founding with `false`.

/// The herd record's fields this reads: the cell, the animal type, and the word stamped on each unit.
#[derive(Clone, Copy, Debug)]
pub struct HerdRec {
    pub cx: i32,
    pub cy: i32,
    pub kind: i32,
    pub tag: i16,
}

/// What is left in `ecx` when `Unit::set_angle` is called: its second argument is never read, but a
/// call-by-call comparison with the original sees it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Left {
    /// The type record the inlined `ObjectData::is` called through.
    TypeOf(i32),
    /// The unit, when its `is` slot is not `ObjectData::is`.
    Unit(i32),
    /// The literal `0x168` the random branch loads.
    Degrees,
}

/// The state `create_units` reads and the calls it makes.
pub trait HerdView {
    /// `Objects::init_unit(8, kind, x, y, -1, -1, -1)`: the new unit's index, negative on failure.
    fn init_unit(&mut self, kind: i32, x: i32, y: i32) -> i32;
    /// The unit's `+0x9` byte and `+0xa` short, `find_nearby_spot`'s two size arguments.
    fn unit_dims(&mut self, unit: i32) -> (u8, i16);
    /// `UnitType::find_nearby_spot` on the unit's type: the spot it writes through its two out
    /// pointers.
    fn find_nearby_spot(&mut self, unit: i32, x: i32, y: i32, dims: (u8, i16)) -> (i32, i32);
    /// Whether the unit's `is` slot (`+0xb8`) is `ObjectData::is`, which the code inlines.
    fn is_inlined(&mut self, unit: i32) -> bool;
    /// `unit.is(t, 0)`.
    fn unit_is(&mut self, unit: i32, t: i32) -> bool;
    /// `Random::get(game_random, 0, 0xffff)`.
    fn random(&mut self) -> i32;
    /// `degrees_to_angle` with the remainder in `ecx`.
    fn degrees_to_angle(&mut self, rem: i32) -> i32;
    /// `Unit::set_angle(angle, left, 1)`.
    fn set_angle(&mut self, unit: i32, angle: i32, left: Left);
    /// `Unit::set_new_location(x, y, 1, 1)`.
    fn set_new_location(&mut self, unit: i32, x: i32, y: i32);
    /// The short stored at the unit's `+0x86`.
    fn set_tag(&mut self, unit: i32, tag: i16);
}

/// A cell's centre in the world's 0x300 scale.
fn centre(c: i32) -> i32 {
    c.wrapping_mul(0x300).wrapping_add(0x180)
}

/// The animals founded for a type.
pub fn herd_count(kind: i32) -> i32 {
    if kind == 0x19b {
        3
    } else if kind == 0x19c {
        1
    } else {
        4
    }
}

/// `Herd::create_units`: `true` when every animal was founded.
pub fn create_units<V: HerdView>(h: &HerdRec, v: &mut V) -> bool {
    let n = herd_count(h.kind);
    let mut done = 0;
    loop {
        let (x, y) = (centre(h.cx), centre(h.cy));
        let u = v.init_unit(h.kind, x, y);
        if u < 0 {
            return false;
        }
        let dims = v.unit_dims(u);
        let (sx, sy) = v.find_nearby_spot(u, x, y, dims);
        let left = if v.is_inlined(u) {
            Left::TypeOf(u)
        } else {
            Left::Unit(u)
        };
        let (angle, left) = if v.unit_is(u, 0x19b) {
            (0, left)
        } else if v.unit_is(u, 0x19c) {
            (i32::MIN, left)
        } else {
            let r = v.random();
            (v.degrees_to_angle(r.wrapping_rem(360)), Left::Degrees)
        };
        v.set_angle(u, angle, left);
        v.set_new_location(u, sx, sy);
        done += 1;
        v.set_tag(u, h.tag);
        if done >= n {
            return true;
        }
    }
}
