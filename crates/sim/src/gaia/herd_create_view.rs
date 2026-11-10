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

// ---------------------------------------------------------------------------
// The crate's view
// ---------------------------------------------------------------------------

/// Gaia's animals: `Objects::init_unit(8, …)`.
pub const HERD_OWNER: crate::Player = 8;
/// `find_nearby_spot`'s inner radius in `create_units` (`0xc0`); its outer
/// radius and step are the defaults (`-1`, `0`).
pub const HERD_SPOT_MIN: i32 = 0xc0;
/// The sweep's bearing, `0x55555555`.
pub const HERD_SPOT_BEARING: i32 = 0x5555_5555;

/// [`HerdView`] over a [`crate::Sim`] (item 4063): the founding a lobby's
/// start walks after the players' units (`Setup::build_game@005ac190`'s
/// own loop over the herd list, `docs/START.md` §2 step 8). Each animal is born as
/// [`crate::Sim::spawn_bird_at`] births a bird — `Unit::init`'s snap, the
/// type's record, one `Guy::init_real` draw a figure — but of owner 8, and
/// moved by `UnitType::find_nearby_spot(…, 0xc0, -1, 0, 0x55555555,
/// FILTER_NOT_ME, o, who, …)` ([`crate::Sim::find_nearby_spot_type_for`]).
/// Not modelled: what `Animal::init` adds to `Unit::init` (no field the
/// start block compares), and a refused sweep's out-words, which the
/// original leaves as its uninitialised locals (here the herd's centre).
pub struct SimHerd<'a> {
    pub sim: &'a mut crate::Sim,
    /// The animals made, in birth order.
    pub made: Vec<usize>,
}

impl SimHerd<'_> {
    fn ty(&self, unit: i32) -> Option<usize> {
        self.sim.units[unit as usize].ty
    }
}

impl HerdView for SimHerd<'_> {
    fn init_unit(&mut self, kind: i32, x: i32, y: i32) -> i32 {
        let Some(ty) = self
            .sim
            .tables
            .unit_types
            .iter()
            .position(|t| t.type_index == kind)
        else {
            return -1;
        };
        let at = crate::Pos::new(crate::gaia::init_snap(x), crate::gaia::init_snap(y));
        let Some(index) = self
            .sim
            .find_free(HERD_OWNER, crate::UNIT_BASE, crate::BUILD_BASE)
        else {
            return -1;
        };
        let t = &self.sim.tables.unit_types[ty];
        let mut unit = crate::Unit::new(HERD_OWNER, index, at, t.hits.max(1));
        unit.kind = t.kind;
        unit.ty = Some(ty);
        unit.type_index = t.type_index;
        unit.form = crate::init_form(unit.type_index);
        unit.movement.speed = self.sim.type_speed(HERD_OWNER, ty);
        unit.movement.turning = self.sim.turning_for(ty);
        unit.stance = self.sim.init_stance(HERD_OWNER, ty);
        let u = self.sim.add_unit(unit);
        self.sim.init_guys(u, Some(ty));
        self.made.push(u);
        u as i32
    }
    fn unit_dims(&mut self, unit: i32) -> (u8, i16) {
        let u = &self.sim.units[unit as usize];
        (u.owner, u.index)
    }
    fn find_nearby_spot(&mut self, unit: i32, x: i32, y: i32, _dims: (u8, i16)) -> (i32, i32) {
        let Some(ty) = self.ty(unit) else {
            return (x, y);
        };
        self.sim
            .find_nearby_spot_type_for(
                ty,
                unit as usize,
                crate::Pos::new(x, y),
                HERD_SPOT_MIN,
                -1,
                crate::movement::Angle(HERD_SPOT_BEARING),
            )
            .map_or((x, y), |p| (p.x, p.y))
    }
    fn is_inlined(&mut self, _unit: i32) -> bool {
        true
    }
    fn unit_is(&mut self, unit: i32, t: i32) -> bool {
        self.sim.units[unit as usize].type_index == t
    }
    fn random(&mut self) -> i32 {
        self.sim.rng.get(0, 0xffff)
    }
    fn degrees_to_angle(&mut self, rem: i32) -> i32 {
        crate::movement::degrees_to_angle(rem).0
    }
    fn set_angle(&mut self, unit: i32, angle: i32, _left: Left) {
        self.sim.units[unit as usize]
            .movement
            .set_facing(crate::movement::Angle(angle));
    }
    fn set_new_location(&mut self, unit: i32, x: i32, y: i32) {
        self.sim
            .set_new_location(unit as usize, crate::Pos::new(x, y), true);
    }
    fn set_tag(&mut self, unit: i32, tag: i16) {
        self.sim.units[unit as usize].herd = usize::try_from(tag).ok();
    }
}

impl crate::Sim {
    /// **`Herd::create_units@007417f0`** for herd `h`, over [`SimHerd`]:
    /// the animals it founded, in birth order, and whether every one was.
    pub fn found_herd(&mut self, h: &HerdRec) -> (Vec<usize>, bool) {
        let mut view = SimHerd {
            sim: self,
            made: Vec::new(),
        };
        let done = create_units(h, &mut view);
        (view.made, done)
    }
}
