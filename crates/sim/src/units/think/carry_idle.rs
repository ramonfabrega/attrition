//! `Unit::think_carry_idle@005f2f30` as steps over a view (item 3411): a
//! boat carrying nothing it can unload looks for **its owner's nearest
//! coastal city with a dock** and walks to a free spot beside it.
//! [`think_carry_idle_steps`] is the original's control flow in its order;
//! [`CarryIdleView`] is what it reads and the callees it makes, which
//! `sweep/unit_think_carry_idle.rs` answers from the emulated original's
//! windows.
//!
//! Wired by item 3995 (`docs/GOLDEN.md` §188): [`SimCarryIdle`] is the live
//! view, and [`Sim::think_carry_idle`] the carry dispatcher's idle arm. The
//! city list is the owner's live cities in slot order (`cities.field_0x10`'s
//! per-player list; the two agree while no city has changed hands); the dock
//! test is an alive, active Dock on the city's chain (`get_building`'s
//! vslots `+0x20` and `+0x4c` read as alive and active); the 16-argument
//! `UnitType::find_nearby_spot(type, city x, y, 0xf00, -1, 0, 0x55555555,
//! FILTER_NOT_ME, o, who, 0, …)` is [`Sim::find_nearby_spot_type_for`]
//! with the boat as the one exempted unit; and the order is the merchant's
//! inlined `add_move_order(…, QUEUE_NEW, 0)`, as the attack carrier's is
//! (the `0x4000000` bit and `+0xc0` this crate does not carry).
//!
//! Established by the sweep (and nothing else): the scan keeps the first city
//! of the least distance (`<`, never `<=`), the first test is `-1`, not a
//! sign; a city counts only when its open bit, `is_coast(my, city region)`
//! and a dock all hold, in that order; the spot is refused when it falls in
//! the unit's own cell **on both axes** (cells being `div_3_table[v >> 8]`).

use crate::Sim;
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

/// The live view: one boat `u` of the simulation.
pub struct SimCarryIdle<'a> {
    sim: &'a mut Sim,
    u: usize,
    cities: Vec<usize>,
}

impl<'a> SimCarryIdle<'a> {
    pub fn new(sim: &'a mut Sim, u: usize) -> Self {
        let who = sim.units[u].owner;
        let cities = (0..sim.cities.cities.len())
            .filter(|&c| sim.cities.cities[c].alive && sim.cities.cities[c].owner == who)
            .collect();
        Self { sim, u, cities }
    }
    fn city(&self, i: i32) -> usize {
        self.cities[i as usize]
    }
}

impl CarryIdleView for SimCarryIdle<'_> {
    fn pos(&self) -> Pos {
        self.sim.units[self.u].pos
    }
    fn div3(&mut self, i: i32) -> i32 {
        i.div_euclid(3)
    }
    fn tregion(&mut self, tx: i32, ty: i32) -> i32 {
        self.sim
            .world
            .tregion_alt(Pos::new(tx, ty))
            .map_or(-1, i32::from)
    }
    fn num_cities(&self) -> i32 {
        self.cities.len() as i32
    }
    fn city_open(&self, i: i32) -> bool {
        self.sim.cities.cities[self.city(i)].alive
    }
    fn city_region(&self, i: i32) -> i32 {
        self.sim.cities.cities[self.city(i)].reg.map_or(-1, i32::from)
    }
    fn is_coast(&mut self, my: i32, city_region: i32) -> bool {
        match (u16::try_from(my), u16::try_from(city_region)) {
            (Ok(a), Ok(b)) => self.sim.world.is_coast(a, b),
            _ => false,
        }
    }
    fn has_dock(&mut self, i: i32) -> bool {
        self.sim
            .count_buildings(self.city(i), crate::cities::build::Ident::Dock, true)
            > 0
    }
    fn city_pos(&self, i: i32) -> Pos {
        self.sim.cities.cities[self.city(i)].pos
    }
    fn nearby_spot(&mut self, i: i32) -> Option<Pos> {
        let ty = self.sim.units[self.u].ty?;
        let at = self.city_pos(i);
        self.sim
            .find_nearby_spot_type_for(ty, self.u, at, 0xf00, -1, Angle(0x5555_5555))
    }
    fn order_move_to(&mut self, to: Pos, angle: Angle) {
        self.sim.add_move_facing_order(
            self.u,
            to,
            crate::orders::MoveKind::MoveTo,
            crate::orders::QueuePos::New,
            false,
            angle,
            None,
            false,
        );
    }
}

impl Sim {
    /// `Unit::think_carry_idle@005f2f30`: the carry dispatcher's arm for a
    /// boat with nothing aboard it can unload (`docs/AI.md` §195,
    /// `docs/GOLDEN.md` §188). `true` ends the think.
    pub(crate) fn think_carry_idle(&mut self, u: usize) -> bool {
        let mut v = SimCarryIdle::new(self, u);
        think_carry_idle_steps(&mut v)
    }
}
