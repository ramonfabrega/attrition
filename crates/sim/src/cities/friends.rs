//! `BuildTypeData::find_friends@00639270` as steps over a view (item 3339,
//! `docs/AI.md` §26, §99.7, §100.2): the neighbour count the AI's placement
//! spiral scores a cell by. [`find_friends_steps`] is the original's control
//! flow in its order; [`FriendsView`] is what it reads and the callees it
//! makes, which [`SimFriends`] answers from the crate's [`Sim`] and
//! `sweep/build_type_data_find_friends.rs` from the emulated original's
//! windows. The method's account is on [`Sim::find_friends`].
//!
//! Every method of the view is **one reach into the original**: the
//! neighbour methods are each a fresh `objects[who][nb]->vtable[0x3c]()`
//! and then one read or call on what it answered, which is how the
//! original asks (it never keeps the record between questions), so the
//! sweep's per-call windows are asked in the same order.

use crate::ai_place::{MOVE_X, MOVE_Y, enhancing_good, gather_good};
use crate::build::{self, Ident, flags};
use crate::world::{Cell, TILES_PER_CELL};
use crate::{Player, Sim};

/// What `find_friends` reads of the type, the world and the neighbour
/// buildings, and the callees it makes. A neighbour is the index
/// `find_building_placed_at` answered.
pub trait FriendsView {
    /// `this->attack` (`+0x1e8`) is non-zero.
    fn attack(&self) -> bool;
    /// `this->vtable[0x60](ident, 0)`: the lineage test on this type.
    fn this_is(&mut self, id: Ident) -> bool;
    /// `build_flags & 0x10` (`NO_CITY`).
    fn no_city(&self) -> bool;
    /// The world's width and height in cells (`worldc +0`, `+4`).
    fn world_cells(&self) -> (i32, i32);
    /// `ObjectsData::find_building_placed_at(tx, ty, who, -1, -1)`: the
    /// building's index, negative for none.
    fn placed_at(&mut self, tx: i32, ty: i32) -> i32;
    /// The neighbour's city (`+0x72`, a short); `-1` for none.
    fn nb_city(&mut self, nb: i32) -> i32;
    /// `BuildTypeData::is_gather_enhancer(this)`.
    fn this_gather_enhancer(&mut self) -> bool;
    /// `BuildTypeData::get_enhancing_good(this)`.
    fn this_enhancing_good(&mut self) -> i32;
    /// `BuildTypeData::get_good(neighbour's type)`.
    fn nb_good(&mut self, nb: i32) -> i32;
    /// `BuildTypeData::is_military_trainer(this)`.
    fn this_military_trainer(&mut self) -> bool;
    /// `BuildTypeData::is_military_trainer(neighbour's type)`.
    fn nb_military_trainer(&mut self, nb: i32) -> bool;
    /// The neighbour's `+8 & 0x20` (a city).
    fn nb_flag20(&mut self, nb: i32) -> bool;
    /// The lineage test on the neighbour's type.
    fn nb_is(&mut self, nb: i32, id: Ident) -> bool;
    /// `vtable[0x90]` of the neighbour's type: it gathers.
    fn nb_gathers(&mut self, nb: i32) -> bool;
    /// `BuildTypeData::is_gather_enhancer(neighbour's type)`.
    fn nb_gather_enhancer(&mut self, nb: i32) -> bool;
    /// `vtable[0x1c]` of the neighbour's type: a wonder.
    fn nb_wonder(&mut self, nb: i32) -> bool;
}

/// The original's control flow. `x`, `y` the candidate cell, `city` the
/// city index (`-1` for none).
pub fn find_friends_steps<V: FriendsView>(v: &mut V, x: i32, y: i32, city: i32) -> i32 {
    if (v.attack() && !v.this_is(Ident::Tower) && !v.this_is(Ident::Lookout))
        || v.this_is(Ident::Woodcutter)
    {
        return 0;
    }
    // A type that wants no city compares none; a city of `-1` compares none
    // either (`-1 < iVar1` is the test).
    let want = if v.no_city() { -1 } else { city };
    let mut n = 0;
    for d in 1..=8usize {
        let (nx, ny) = (MOVE_X[d] + x, MOVE_Y[d] + y);
        let (w, h) = v.world_cells();
        if nx < 0 || ny < 0 || nx >= w || ny >= h {
            continue;
        }
        let nb = v.placed_at(nx * TILES_PER_CELL + 2, ny * TILES_PER_CELL + 2);
        if nb < 0 {
            continue;
        }
        if want >= 0 && v.nb_city(nb) != want {
            continue;
        }
        let add = if d & 1 == 1 { 1 } else { 2 };
        if v.this_gather_enhancer() {
            if v.this_enhancing_good() == v.nb_good(nb) {
                n += add;
            }
        } else if v.this_military_trainer() {
            if v.nb_military_trainer(nb) && !v.nb_flag20(nb) {
                n += add;
            }
        } else if v.this_is(Ident::Tower) || v.this_is(Ident::Lookout) {
            if v.nb_gathers(nb) && !v.nb_is(nb, Ident::University) {
                n += 2;
            } else if v.nb_wonder(nb) {
                n += if v.this_is(Ident::Lookout) { 8 } else { 4 };
            }
        } else if v.this_is(Ident::Farm) {
            if v.nb_is(nb, Ident::Farm) || v.nb_is(nb, Ident::Granary) {
                n += add;
            }
        } else if !(v.nb_gathers(nb) && !v.nb_is(nb, Ident::University))
            && !v.nb_gather_enhancer(nb)
            && !(v.nb_military_trainer(nb) && !v.nb_flag20(nb))
            && !v.nb_wonder(nb)
        {
            n += add;
        }
    }
    n
}

/// [`FriendsView`] on the crate's state.
pub struct SimFriends<'a> {
    pub(crate) sim: &'a Sim,
    pub(crate) rec: usize,
    pub(crate) who: Player,
}

impl SimFriends<'_> {
    fn ty(&self, nb: i32) -> Option<usize> {
        self.sim.buildings[usize::try_from(nb).expect("a building")].ty
    }
}

impl FriendsView for SimFriends<'_> {
    fn attack(&self) -> bool {
        self.sim.build_types[self.rec].attack != 0
    }
    fn this_is(&mut self, id: Ident) -> bool {
        build::is(&self.sim.build_types, self.rec, id)
    }
    fn no_city(&self) -> bool {
        self.sim.build_types[self.rec].has(flags::NO_CITY)
    }
    fn world_cells(&self) -> (i32, i32) {
        (self.sim.world.width(), self.sim.world.height())
    }
    fn placed_at(&mut self, tx: i32, ty: i32) -> i32 {
        let cell = Cell::new(tx / TILES_PER_CELL, ty / TILES_PER_CELL);
        self.sim
            .building_placed_at(self.who, cell)
            .map_or(-1, |b| i32::try_from(b).expect("a building index"))
    }
    fn nb_city(&mut self, nb: i32) -> i32 {
        self.sim.buildings[usize::try_from(nb).expect("a building")]
            .city
            .map_or(-1, |c| i32::try_from(c).expect("a city index"))
    }
    fn this_gather_enhancer(&mut self) -> bool {
        build::is_gather_enhancer_by(|i| build::is(&self.sim.build_types, self.rec, i))
    }
    fn this_enhancing_good(&mut self) -> i32 {
        enhancing_good(self.sim.build_types[self.rec].ident)
            .map_or(-1, |g| i32::try_from(g).expect("a good"))
    }
    fn nb_good(&mut self, nb: i32) -> i32 {
        let ni = self
            .sim
            .building_ident(usize::try_from(nb).expect("a building"));
        gather_good(ni).map_or(-1, |g| i32::try_from(g).expect("a good"))
    }
    fn this_military_trainer(&mut self) -> bool {
        build::is_military_trainer(&self.sim.build_types, self.rec)
    }
    fn nb_military_trainer(&mut self, nb: i32) -> bool {
        self.ty(nb)
            .is_some_and(|t| build::is_military_trainer(&self.sim.build_types, t))
    }
    fn nb_flag20(&mut self, nb: i32) -> bool {
        self.sim
            .building_is_city(usize::try_from(nb).expect("a building"))
    }
    fn nb_is(&mut self, nb: i32, id: Ident) -> bool {
        self.sim
            .building_is(usize::try_from(nb).expect("a building"), id)
    }
    fn nb_gathers(&mut self, nb: i32) -> bool {
        self.ty(nb)
            .is_some_and(|t| self.sim.build_types[t].has(flags::GATHER))
    }
    fn nb_gather_enhancer(&mut self, nb: i32) -> bool {
        self.ty(nb).is_some_and(|t| {
            build::is_gather_enhancer_by(|i| build::is(&self.sim.build_types, t, i))
        })
    }
    fn nb_wonder(&mut self, nb: i32) -> bool {
        self.sim
            .building_ident(usize::try_from(nb).expect("a building"))
            == Ident::Wonder
    }
}
