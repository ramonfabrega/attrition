//! Transports and docks — the sea half of the AI. `docs/TRANSPORT.md` is
//! the specification; section numbers below refer to it.
//!
//! A land unit ordered across water becomes its own transport at the shore.
//! What this module carries is the machinery *around* that: the leader's
//! transport level and the unit bit that gate it (§2–§4), the docks registry
//! that keeps `reg_docks` and spawns the gull (§5), the dock-tile rule the
//! census counts (§5.6), and the shore predicate the pathfinder asks (§6).
//! The civilian's island choice (§7) and the army's transporting step (§8)
//! are documented and not implemented: the first needs the unit AI's
//! `think` and the danger grid's writers, the second the `Army` family.
//!
//! Integers throughout; the one draw here is `Random::get(0, 0xffff)` for
//! the gull's angle, on the sync stream, in the order the original takes it
//! (§5.2).

use crate::attrition::Domain;
use crate::build;
use crate::orders::Worker;
use crate::tech::TypeId;
use crate::world::{Cell, Pos, TILES_PER_CELL, tile};
use crate::{BUILD_BASE, Player, Sim, UNIT_BASE, Unit};

/// `TypeIndex` values this mechanic compares against (§3.1, §3.3, §5.2).
pub mod ty {
    use crate::tech::TypeId;
    /// `SCOUT` — the lineage test `is(0x45)`.
    pub const SCOUT: TypeId = 0x45;
    /// `CARAVAN` — `is(0x3b)`.
    pub const CARAVAN: TypeId = 0x3b;
    /// `MERCHANT`, `ECHINESEMERCHANT`, `MERCHANT2` — exact base types.
    pub const MERCHANTS: [TypeId; 3] = [0x3d, 0x3e, 0x190];
    /// `AIRCRAFTCARRIER` — the one carrier that can never transport.
    pub const AIRCRAFTCARRIER: TypeId = 0x15f;
    /// `GULLBIRD` — what a finished dock spawns.
    pub const GULLBIRD: TypeId = 0x194;
}

/// `TransportType` — the ladder a unit's kind sits on and a leader's level
/// climbs (§2). A unit may auto-transport when its kind is at or below the
/// leader's level.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
#[repr(u8)]
pub enum TransportType {
    #[default]
    None = 0,
    /// `TRANSPORT_SCOUT`, `leader_flags & 0x400`.
    Scout = 1,
    /// `TRANSPORT_MILITARY`, `leader_flags & 0x200`.
    Military = 2,
    /// `TRANSPORT_CIVILIAN`, `leader_flags & 0x100`.
    Civilian = 3,
}

/// The leader's side: the three level bits of `leader_flags`, the scenario
/// lock in `leader_flags2`, and the "scouts auto-transport" option
/// (`LeaderOptionData.flags & 2`, on by default).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LeaderTransport {
    /// `leader_flags & 0x100`.
    pub civilian: bool,
    /// `leader_flags & 0x200`.
    pub military: bool,
    /// `leader_flags & 0x400`.
    pub scout: bool,
    /// `leader_flags2 & 0x20` — `LeaderData::locked_transport`; only a
    /// scenario sets it.
    pub locked: bool,
    /// `LeaderOptionData.flags & 2`.
    pub scouts_option: bool,
}

impl Default for LeaderTransport {
    fn default() -> LeaderTransport {
        LeaderTransport {
            civilian: false,
            military: false,
            scout: false,
            locked: false,
            scouts_option: true,
        }
    }
}

impl LeaderTransport {
    /// `LeaderData::can_transport@006e0c60`: the highest bit set.
    pub const fn level(&self) -> TransportType {
        if self.civilian {
            TransportType::Civilian
        } else if self.military {
            TransportType::Military
        } else if self.scout {
            TransportType::Scout
        } else {
            TransportType::None
        }
    }

    /// `leader_flags & 0x300` — the test the armies make (§8.1).
    pub const fn at_least_military(&self) -> bool {
        self.civilian || self.military
    }

    const fn set_all(&mut self) {
        self.civilian = true;
        self.military = true;
        self.scout = true;
    }

    const fn clear(&mut self) {
        self.civilian = false;
        self.military = false;
        self.scout = false;
    }
}

/// One `Dock` record (§5): the building it stands for, the land region it
/// counted itself into, and its gull. A free slot has no building.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct DockSlot {
    pub building: Option<usize>,
    pub reg: Option<u16>,
    pub gull: Option<usize>,
}

impl DockSlot {
    /// `dock_flags & 1`.
    pub const fn active(&self) -> bool {
        self.building.is_some()
    }
}

/// `PtrArray<Dock>` for one leader, with the leader's `dock_mark` (§5.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Docks {
    pub slots: Vec<DockSlot>,
    /// `LeaderData::dock_mark`: one past the highest slot ever used, walked
    /// down past trailing free slots by `close_dock`.
    pub mark: usize,
}

/// `Docks::init` preallocates twenty `Dock`s per leader.
pub const DOCK_SLOTS: usize = 20;

impl Default for Docks {
    fn default() -> Docks {
        Docks {
            slots: vec![DockSlot::default(); DOCK_SLOTS],
            mark: 0,
        }
    }
}

/// `Dock::init` places the gull one tile up and left of the dock: `x − 0xc0,
/// y − 0xc0` in position units.
pub const GULL_OFFSET: i32 = 0xc0;

/// `needs_transport`'s answers (§6).
pub const DISEMBARK: i32 = 1;
pub const EMBARK: i32 = 2;

/// `orthog_x[1..=4]`, `orthog_y[1..=4]` — north, east, south, west
/// (`.rdata 0x00add250` / `0x00add210`).
const ORTHOG: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

impl Sim {
    // ------------------------------------------------------------------
    // §2–§3: the predicates
    // ------------------------------------------------------------------

    /// `LeaderData::can_transport`.
    pub fn transport_level(&self, who: Player) -> TransportType {
        self.transport
            .get(who as usize)
            .map_or(TransportType::None, LeaderTransport::level)
    }

    /// `ObjectData::is(t, 0)` on a unit's type — the lineage test.
    fn unit_is(&self, u: usize, t: TypeId) -> bool {
        self.units[u]
            .ty
            .and_then(|i| self.unit_types[i].tree)
            .is_some_and(|ut| {
                ut == t || (ut < self.tech_tree.types.len() && self.tech_tree.is(ut, t, false))
            })
    }

    /// The unit type's domain; a typeless unit is what its kind says.
    fn unit_domain_of(&self, u: usize) -> Domain {
        self.units[u].ty.map_or(self.units[u].kind.domain, |t| {
            self.unit_types[t].combat.domain
        })
    }

    /// `UnitData::transport_type@0046f790` (§3.1).
    pub fn unit_transport_type(&self, u: usize) -> TransportType {
        if self.unit_is(u, ty::SCOUT) {
            return TransportType::Scout;
        }
        let unit = &self.units[u];
        let worker = unit.ty.map_or(Worker::None, |t| self.unit_types[t].worker);
        if worker != Worker::None {
            return TransportType::Civilian;
        }
        let base = unit.ty.and_then(|t| self.unit_types[t].tree);
        if self.unit_is(u, ty::CARAVAN) || base.is_some_and(|b| ty::MERCHANTS.contains(&b)) {
            return TransportType::Civilian;
        }
        TransportType::Military
    }

    /// `UnitData::can_transport@0046f960` (§3.2): the bit, unless vetoed, or
    /// a type that always transports (`unit_flags & 0x10`).
    pub fn unit_can_transport(&self, u: usize) -> bool {
        let unit = &self.units[u];
        let always = unit
            .ty
            .is_some_and(|t| self.unit_types[t].cols.unit_flags & 0x10 != 0);
        (unit.auto_transport && !unit.never_transport) || always
    }

    /// `UnitData::can_ever_transport@0046f290` (§3.3): a land unit, or a
    /// carrier (`carry != 0`) that is not an aircraft carrier.
    pub fn unit_can_ever_transport(&self, u: usize) -> bool {
        if self.unit_domain_of(u) == Domain::Land {
            return true;
        }
        let carry = self.units[u]
            .ty
            .is_some_and(|t| self.unit_types[t].cols.carry != 0);
        carry && !self.unit_is(u, ty::AIRCRAFTCARRIER)
    }

    /// `UnitData::needs_transport@00609920` (§6), on two **tile**
    /// coordinates: 0 on the same side of the shore, [`DISEMBARK`] from
    /// water to land, [`EMBARK`] from land to water. The pathfinder's
    /// `calc_cost` is its caller (`path.rs`).
    pub fn needs_transport(&self, from: Pos, to: Pos) -> i32 {
        if from == to {
            return 0;
        }
        let water = |t: Pos| self.world.tile_mask(t) & tile::SURFACE == tile::SURFACE_OCEAN;
        let (a, b) = (water(from), water(to));
        if a == b {
            0
        } else if a {
            DISEMBARK
        } else {
            EMBARK
        }
    }

    // ------------------------------------------------------------------
    // §4: the level, maintained
    // ------------------------------------------------------------------

    /// `has_preq(TRANSPORT_BONUS)`: the bonus row's one prerequisite — the
    /// tree names it in `roles.transport_preq`; a tree that does not know it
    /// grants it, the way every type outside the tree is ungated.
    fn transport_preq_held(&self, who: Player) -> bool {
        match self.tech_tree.roles.transport_preq {
            Some(t) => self
                .tech_tree
                .has_tech(&self.setup, &self.tech[who as usize], t),
            None => true,
        }
    }

    /// `num_buildings[DOCK] + get_buildings(DOCK.to, …)`: the leader's
    /// active docks, counting the whole upgrade chain.
    fn dock_count(&self, who: Player) -> i32 {
        self.buildings
            .iter()
            .filter(|b| {
                b.owner == who
                    && b.alive
                    && b.active
                    && b.ty.is_some_and(|t| build::is_dock(&self.build_types, t))
            })
            .count() as i32
    }

    /// `Leader::check_transport@006bc5f0` (§4). Run when a dock is finished
    /// or lost and after every technology gained.
    pub fn check_transport(&mut self, who: Player) {
        let w = who as usize;
        if w >= self.transport.len() || self.transport[w].locked {
            return;
        }
        let want = if self.transport_preq_held(who) {
            TransportType::Civilian
        } else {
            TransportType::None
        };
        if want != TransportType::None && self.dock_count(who) != 0 {
            if self.transport[w].level() == want {
                return;
            }
            self.transport[w].set_all();
            let option = self.transport[w].scouts_option;
            for u in 0..self.units.len() {
                let unit = &self.units[u];
                if unit.owner != who || !unit.alive() || !self.unit_can_ever_transport(u) {
                    continue;
                }
                if !option && self.unit_is(u, ty::SCOUT) {
                    continue;
                }
                if self.unit_transport_type(u) <= want {
                    self.units[u].auto_transport = true;
                }
            }
            return;
        }
        if self.transport[w].level() != TransportType::None {
            self.transport[w].clear();
            for unit in &mut self.units {
                if unit.owner == who && unit.alive() {
                    unit.auto_transport = false;
                }
            }
        }
    }

    /// `Unit::init`'s clause (§3.4): a new unit whose kind is at or below
    /// the leader's level starts with the bit, scouts only under the option.
    pub(crate) fn transport_init_unit(&mut self, u: usize) {
        let who = self.units[u].owner;
        let Some(lt) = self.transport.get(who as usize).copied() else {
            return;
        };
        if self.unit_transport_type(u) <= lt.level()
            && self.unit_can_ever_transport(u)
            && (lt.scouts_option || !self.unit_is(u, ty::SCOUT))
        {
            self.units[u].auto_transport = true;
        }
    }

    /// `Group::action_set_transport(flag)` (§3.4): the player's toggle over
    /// a set of units — with a level of 0 the flag is forced off.
    pub fn set_transport(&mut self, who: Player, units: &[usize], on: bool) {
        let on = on && self.transport_level(who) != TransportType::None;
        for &u in units {
            if self.units[u].owner == who
                && self.units[u].alive()
                && self.unit_can_ever_transport(u)
            {
                self.units[u].auto_transport = on;
            }
        }
    }

    // ------------------------------------------------------------------
    // §5: the docks registry
    // ------------------------------------------------------------------

    /// `Docks::init_dock` + `Dock::init` (§5.1–§5.2), on a building that
    /// has just activated as a dock. Returns the slot. Two draws when the
    /// gull type exists: its creation, then its angle.
    pub(crate) fn dock_open(&mut self, b: usize) -> usize {
        let who = self.buildings[b].owner;
        let w = who as usize;
        if self.docks.len() <= w {
            self.docks.resize_with(w + 1, Docks::default);
        }
        let slot = {
            let docks = &mut self.docks[w];
            let mark = docks.mark.min(docks.slots.len());
            let reused = (0..mark).find(|&i| !docks.slots[i].active());
            let slot = match reused {
                Some(i) => i,
                None if docks.mark < docks.slots.len() => docks.mark,
                None => {
                    docks.slots.push(DockSlot::default());
                    docks.slots.len() - 1
                }
            };
            docks.mark = docks.mark.max(slot + 1);
            slot
        };
        self.buildings[b].dock_slot = Some(slot);
        // `Dock::init`: the building's cell region, counted when it is land.
        let pos = self.buildings[b].pos;
        let reg = self.world.region_of(pos.cell());
        if let Some(r) = reg
            && self.world.terrain(r) == crate::world::Terrain::Land
            && w < self.ai.len()
        {
            let regs = self.world.region_count();
            let cs = &mut self.ai[w].census;
            if cs.reg_docks.len() < regs {
                cs.reg_docks.resize(regs, 0);
            }
            cs.reg_docks[r as usize] += 1;
        }
        // The gull: `Objects::init_unit(9, GULLBIRD, x − 0xc0, y − 0xc0)`,
        // then `Random::get(0, 0xffff)` for its angle. A type space without
        // the gull spawns nothing and draws nothing, as `init_unit` failing
        // would.
        let gull_ty = self
            .unit_types
            .iter()
            .position(|t| t.tree == Some(ty::GULLBIRD));
        let mut gull = None;
        if let Some(gt) = gull_ty
            && let Some(index) = self.find_free(9, UNIT_BASE, BUILD_BASE)
        {
            let at = Pos::new(pos.x - GULL_OFFSET, pos.y - GULL_OFFSET);
            let mut unit = Unit::new(9, index, at, self.unit_types[gt].hits);
            unit.kind = self.unit_types[gt].kind;
            unit.ty = Some(gt);
            let u = self.add_unit(unit);
            self.init_guys(u, Some(gt));
            // `(get(0, 0xffff) % 7) × 0x0aaaaaaa − 0x40000000`: the heading.
            // The strafe order and the heading itself are not modelled; the
            // draw is.
            let _angle = (self.rng.get(0, 0xffff) % 7).wrapping_mul(0x0aaa_aaaa) - 0x4000_0000;
            gull = Some(u);
        }
        self.docks[w].slots[slot] = DockSlot {
            building: Some(b),
            reg,
            gull,
        };
        slot
    }

    /// `Docks::close_dock` + `Dock::close` (§5.3), before the building is
    /// marked dead: the region's count comes back down, the gull is closed,
    /// and the mark walks down past trailing free slots.
    pub(crate) fn dock_close(&mut self, b: usize) {
        let Some(slot) = self.buildings[b].dock_slot.take() else {
            return;
        };
        let w = self.buildings[b].owner as usize;
        let Some(docks) = self.docks.get_mut(w) else {
            return;
        };
        let Some(rec) = docks.slots.get_mut(slot).copied() else {
            return;
        };
        if rec.building == Some(b)
            && self.buildings[b].alive
            && let Some(r) = rec.reg
            && self.world.terrain(r) == crate::world::Terrain::Land
            && w < self.ai.len()
            && let Some(n) = self.ai[w].census.reg_docks.get_mut(r as usize)
        {
            *n -= 1;
        }
        // `Dock::close` clears `o`, `reg`, `who` and the active bit and
        // leaves `gull_o` as it was (audit B.13) — a dump of a closed slot
        // shows the stale gull.
        self.docks[w].slots[slot] = DockSlot {
            building: None,
            reg: None,
            gull: rec.gull,
        };
        if let Some(g) = rec.gull
            && let Some(gull) = self.units.get_mut(g)
        {
            // `Unit::close(0, −1, 0)`.
            gull.health = 0;
            gull.on_map = false;
        }
        let docks = &mut self.docks[w];
        while docks.mark > 0 && !docks.slots[docks.mark - 1].active() {
            docks.mark -= 1;
        }
    }

    // ------------------------------------------------------------------
    // §5.6: the dock-tile rule
    // ------------------------------------------------------------------

    /// `BuildTypeData::is_dock_tile@00636700` on a **cell**: an uncoastal
    /// water cell with a shore next to it whose two tile rows just outside
    /// this cell, four wide, are free of placed buildings and footprints,
    /// and whose first tile is neither mountain nor forest.
    pub fn is_dock_tile(&self, c: Cell) -> bool {
        if !self.world.contains(c) {
            return false;
        }
        let d = self.world.cell_data(c);
        if d.flags & 0x100 != 0 || !(d.land == 1 || d.land == 2) {
            return false;
        }
        let tw = self.world.width() * TILES_PER_CELL;
        let th = self.world.height() * TILES_PER_CELL;
        let (tx0, ty0) = (c.x * TILES_PER_CELL, c.y * TILES_PER_CELL);
        for (ox, oy) in ORTHOG {
            // A positive step lands on the next cell's first tile row, a
            // negative one on the previous cell's last: one tile outside
            // this cell's edge either way.
            let sx = if ox > 0 { ox * TILES_PER_CELL } else { ox };
            let sy = if oy > 0 { oy * TILES_PER_CELL } else { oy };
            let (mut tx, mut ty) = (tx0 + sx, ty0 + sy);
            if tx < 0 || ty < 0 || tx >= tw || ty >= th {
                continue;
            }
            let there = self
                .world
                .cell_data(Cell::new(tx / TILES_PER_CELL, ty / TILES_PER_CELL));
            if there.flags & 0x100 == 0 && (there.land == 1 || there.land == 2) {
                // Open water that way: not a shore.
                continue;
            }
            let m = self.world.tile_mask(Pos::new(tx, ty));
            if m & tile::OBJECT == tile::OBJECT_MOUNTAIN
                || m & tile::SURFACE == tile::SURFACE_FOREST
            {
                continue;
            }
            let blocked =
                |m: u16| m & tile::PLACED != 0 || m & tile::OBJECT == tile::OBJECT_BUILDING;
            let mut ok = true;
            'strip: for pass in 0..2 {
                if pass != 0 {
                    if sx == 0 {
                        ty += sy.signum();
                    } else {
                        tx += sx.signum();
                    }
                }
                for i in 0..TILES_PER_CELL {
                    let t = if sx == 0 {
                        Pos::new(tx0 + i, ty)
                    } else {
                        Pos::new(tx, ty0 + i)
                    };
                    if blocked(self.world.tile_mask(t)) {
                        ok = false;
                        break 'strip;
                    }
                }
            }
            if ok {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::{BuildType, Ident, flags};
    use crate::tuning::Tuning;
    use crate::world::{Terrain, UNITS_PER_TILE, World};
    use crate::{UnitType, cost};

    fn tile_pos(tx: i32, ty: i32) -> Pos {
        Pos::new(
            tx * UNITS_PER_TILE + UNITS_PER_TILE / 2,
            ty * UNITS_PER_TILE + UNITS_PER_TILE / 2,
        )
    }

    /// A 12×8 world: land on the left eight cells, sea on the right four,
    /// the sea tiles surfaced as ocean, the two shore columns marked the way
    /// the map maker marks them (`land` SANDY/OCEAN on the water, the land
    /// cells flagged coastal with the sea as `region2`).
    struct Fix {
        sim: Sim,
        dock: usize,
        citizen: usize,
        scout: usize,
        ship: usize,
        land: u16,
        sea: u16,
    }

    fn fix() -> Fix {
        let mut w = World::new(12, 8);
        let land = w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(7, 7));
        let sea = w.fill_region(Terrain::Sea, Cell::new(8, 0), Cell::new(11, 7));
        for y in 0..8 {
            for x in 0..12 {
                let c = Cell::new(x, y);
                let mut d = w.cell_data(c);
                if x >= 8 {
                    d.land = 2;
                    for tx in x * 4..x * 4 + 4 {
                        for ty in y * 4..y * 4 + 4 {
                            w.set_tile_field(Pos::new(tx, ty), tile::SURFACE, tile::SURFACE_OCEAN);
                        }
                    }
                } else if x == 7 {
                    d.flags |= 0x100;
                    d.region2 = Some(sea);
                }
                w.set_cell_data(c, d);
            }
        }
        let mut sim = Sim::new(Tuning::RON, w, 2);
        sim.nation[1].human = false;
        let dock = sim.add_build_type(BuildType {
            ident: Ident::Dock,
            x_size: 4,
            y_size: 4,
            flags: flags::parse("eab"),
            job_time: 100,
            hits: 500,
            price: cost::Price {
                kind: cost::Kind::Building,
                ..cost::Price::free()
            },
            ..BuildType::default()
        });
        let citizen = sim.add_unit_type(UnitType {
            worker: Worker::Citizen,
            hits: 40,
            ..UnitType::default()
        });
        // No tree in this fixture: a `TypeIndex` is set after registration
        // so the lineage test short-circuits on equality.
        let scout = sim.add_unit_type(UnitType {
            hits: 40,
            ..UnitType::default()
        });
        sim.unit_types[scout].tree = Some(ty::SCOUT);
        let mut ship_t = UnitType {
            hits: 100,
            ..UnitType::default()
        };
        ship_t.combat.domain = Domain::Sea;
        let ship = sim.add_unit_type(ship_t);
        Fix {
            sim,
            dock,
            citizen,
            scout,
            ship,
            land,
            sea,
        }
    }

    fn unit(sim: &mut Sim, who: Player, t: usize, at: Pos) -> usize {
        let index = sim.find_free(who, UNIT_BASE, BUILD_BASE).unwrap();
        let mut u = Unit::new(who, index, at, sim.unit_types[t].hits);
        u.ty = Some(t);
        u.kind.domain = sim.unit_types[t].combat.domain;
        sim.add_unit(u)
    }

    fn place_dock(sim: &mut Sim, who: Player, dock: usize, at: Pos) -> usize {
        let b = sim.add_building(who, at, 1);
        sim.buildings[b].ty = Some(dock);
        sim.buildings[b].orig_ty = Some(dock);
        sim.buildings[b].started = true;
        b
    }

    #[test]
    fn transport_types_follow_the_kind_ladder() {
        let mut f = fix();
        let c = unit(&mut f.sim, 1, f.citizen, tile_pos(4, 4));
        let s = unit(&mut f.sim, 1, f.scout, tile_pos(4, 5));
        let b = unit(&mut f.sim, 1, f.ship, tile_pos(36, 4));
        assert_eq!(f.sim.unit_transport_type(c), TransportType::Civilian);
        assert_eq!(f.sim.unit_transport_type(s), TransportType::Scout);
        assert_eq!(f.sim.unit_transport_type(b), TransportType::Military);
        // A land unit can ever transport; a warship without `carry` cannot.
        assert!(f.sim.unit_can_ever_transport(c));
        assert!(!f.sim.unit_can_ever_transport(b));
        f.sim.unit_types[f.ship].cols.carry = 4;
        assert!(f.sim.unit_can_ever_transport(b));
    }

    #[test]
    fn the_level_is_granted_by_a_dock_and_revoked_with_it() {
        let mut f = fix();
        let c = unit(&mut f.sim, 1, f.citizen, tile_pos(4, 4));
        let s = unit(&mut f.sim, 1, f.scout, tile_pos(4, 5));
        // Born at level 0: no bit.
        assert!(!f.sim.units[c].auto_transport);
        assert_eq!(f.sim.transport_level(1), TransportType::None);
        let b = place_dock(&mut f.sim, 1, f.dock, tile_pos(30, 16));
        f.sim.activate(b, false, true);
        assert_eq!(f.sim.transport_level(1), TransportType::Civilian);
        assert!(f.sim.units[c].auto_transport);
        assert!(f.sim.units[s].auto_transport, "scouts under the option");
        assert!(f.sim.unit_can_transport(c));
        // A unit born now starts with the bit.
        let c2 = unit(&mut f.sim, 1, f.citizen, tile_pos(5, 5));
        assert!(f.sim.units[c2].auto_transport);
        // The other player has no dock and no level.
        let h = unit(&mut f.sim, 0, f.citizen, tile_pos(2, 2));
        assert!(!f.sim.units[h].auto_transport);
        // Losing the last dock revokes it for every unit at once.
        f.sim.close_building(b, false);
        assert_eq!(f.sim.transport_level(1), TransportType::None);
        assert!(!f.sim.units[c].auto_transport);
        assert!(!f.sim.units[c2].auto_transport);
        assert!(!f.sim.unit_can_transport(c));
    }

    #[test]
    fn scouts_stay_off_without_the_option_and_the_veto_holds() {
        let mut f = fix();
        f.sim.transport[1].scouts_option = false;
        let s = unit(&mut f.sim, 1, f.scout, tile_pos(4, 5));
        let c = unit(&mut f.sim, 1, f.citizen, tile_pos(4, 4));
        let b = place_dock(&mut f.sim, 1, f.dock, tile_pos(30, 16));
        f.sim.activate(b, false, true);
        assert!(!f.sim.units[s].auto_transport);
        assert!(f.sim.units[c].auto_transport);
        // The scenario veto beats the bit; the type flag beats the veto.
        f.sim.units[c].never_transport = true;
        assert!(!f.sim.unit_can_transport(c));
        f.sim.unit_types[f.citizen].cols.unit_flags |= 0x10;
        assert!(f.sim.unit_can_transport(c));
        // The lock freezes the level.
        f.sim.transport[1].locked = true;
        f.sim.close_building(b, false);
        assert_eq!(f.sim.transport_level(1), TransportType::Civilian);
    }

    #[test]
    fn the_registry_keeps_reg_docks_and_the_slot_rule() {
        let mut f = fix();
        let a = place_dock(&mut f.sim, 1, f.dock, tile_pos(30, 8));
        f.sim.activate(a, false, true);
        let b = place_dock(&mut f.sim, 1, f.dock, tile_pos(30, 20));
        f.sim.activate(b, false, true);
        assert_eq!(f.sim.buildings[a].dock_slot, Some(0));
        assert_eq!(f.sim.buildings[b].dock_slot, Some(1));
        assert_eq!(f.sim.docks[1].mark, 2);
        assert_eq!(f.sim.ai[1].census.reg_docks[f.land as usize], 2);
        assert_eq!(f.sim.docks[1].slots[0].reg, Some(f.land));
        // Close the first: its slot frees, the mark stays at 2 (slot 1 is
        // still active); a third dock reuses slot 0.
        f.sim.close_building(a, false);
        assert_eq!(f.sim.ai[1].census.reg_docks[f.land as usize], 1);
        assert!(!f.sim.docks[1].slots[0].active());
        assert_eq!(f.sim.docks[1].mark, 2);
        let c = place_dock(&mut f.sim, 1, f.dock, tile_pos(30, 8));
        f.sim.activate(c, false, true);
        assert_eq!(f.sim.buildings[c].dock_slot, Some(0));
        // Close both: the mark walks down to 0.
        f.sim.close_building(b, false);
        f.sim.close_building(c, false);
        assert_eq!(f.sim.docks[1].mark, 0);
        assert_eq!(f.sim.ai[1].census.reg_docks[f.land as usize], 0);
        assert_eq!(f.sim.transport_level(1), TransportType::None);
    }

    #[test]
    fn a_dock_with_a_gull_type_draws_twice() {
        let mut f = fix();
        let gull = f.sim.add_unit_type(UnitType {
            hits: 10,
            gaia: true,
            ..UnitType::default()
        });
        f.sim.unit_types[gull].tree = Some(ty::GULLBIRD);
        let seed0 = f.sim.rng.seed;
        let a = place_dock(&mut f.sim, 1, f.dock, tile_pos(30, 8));
        f.sim.activate(a, false, true);
        let mut probe = f.sim.rng;
        probe.seed = seed0;
        // Two draws: the gull's creation (`Guy::init_real`) and its angle.
        probe.roll();
        probe.roll();
        assert_eq!(f.sim.rng.seed, probe.seed);
        let g = f.sim.docks[1].slots[0].gull.expect("a gull");
        assert_eq!(f.sim.units[g].owner, 9);
        assert_eq!(f.sim.units[g].ty, Some(gull));
        let dock_pos = f.sim.buildings[a].pos;
        assert_eq!(
            f.sim.units[g].pos,
            Pos::new(dock_pos.x - GULL_OFFSET, dock_pos.y - GULL_OFFSET)
        );
        // Closing the dock closes the gull, and the slot keeps its number
        // (`Dock::close` does not clear `gull_o`).
        f.sim.close_building(a, false);
        assert!(!f.sim.units[g].alive());
        assert!(!f.sim.docks[1].slots[0].active());
        assert_eq!(f.sim.docks[1].slots[0].gull, Some(g));
    }

    #[test]
    fn needs_transport_reads_the_shore() {
        let f = fix();
        let land = Pos::new(20, 10);
        let land2 = Pos::new(21, 10);
        let water = Pos::new(40, 10);
        assert_eq!(f.sim.needs_transport(land, land2), 0);
        assert_eq!(f.sim.needs_transport(land, water), EMBARK);
        assert_eq!(f.sim.needs_transport(water, land), DISEMBARK);
        assert_eq!(f.sim.needs_transport(water, water), 0);
    }

    #[test]
    fn dock_tiles_are_water_cells_on_a_free_shore() {
        let mut f = fix();
        // The water cell next to the coastal column: its west neighbour is
        // the shore, whose last tile column is free.
        assert!(f.sim.is_dock_tile(Cell::new(8, 3)));
        // Open water two cells out: every neighbour is water or the map's
        // edge.
        assert!(!f.sim.is_dock_tile(Cell::new(10, 3)));
        // A land cell is never one; nor is a coastal-flagged cell.
        assert!(!f.sim.is_dock_tile(Cell::new(7, 3)));
        // A footprint on the shore's edge tiles blocks that direction.
        for ty in 12..16 {
            f.sim
                .world
                .set_tile_field(Pos::new(31, ty), tile::OBJECT, tile::OBJECT_BUILDING);
        }
        assert!(!f.sim.is_dock_tile(Cell::new(8, 3)));
        assert!(f.sim.is_dock_tile(Cell::new(8, 4)));
        // A forest on the first shore tile blocks it too.
        f.sim
            .world
            .set_tile_field(Pos::new(31, 16), tile::SURFACE, tile::SURFACE_FOREST);
        assert!(!f.sim.is_dock_tile(Cell::new(8, 4)));
    }

    #[test]
    fn coasts_are_computed_from_the_cells() {
        let f = fix();
        assert!(f.sim.world.is_coast(f.land, f.sea));
        assert!(f.sim.world.is_coast(f.sea, f.land));
        assert!(f.sim.world.is_coast(f.land, f.land));
        assert_eq!(f.sim.world.num_coasts(f.land), 1);
        // A sea region counts itself, and nothing else (`Region::num_coasts`).
        assert_eq!(f.sim.world.num_coasts(f.sea), 1);
    }
}
