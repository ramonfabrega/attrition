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
use crate::world::{Cell, Pos, TILES_PER_CELL, UNITS_PER_TILE, tile};
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
    /// `TRANSPORTBARGE` — the boat a land unit becomes at the shore (§6).
    pub const TRANSPORTBARGE: TypeId = 0x140;
    /// `MERCHANTFLEET` — a caravan's own boat.
    pub const MERCHANTFLEET: TypeId = 0x13e;
    /// `GULLBIRD` — what a finished dock spawns.
    pub const GULLBIRD: TypeId = 0x194;
}

/// The bias angle `Unit::do_cast` and `SpellType::cast_transport` both hand
/// `find_nearby_spot` — `0x55555555`, a third of a turn, which
/// `docs/ORDERS.md` §10 records as arbitrary rather than a sentinel.
const BOARD_BEARING: crate::movement::Angle = crate::movement::Angle(0x5555_5555);

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

/// `Dock::init@00740a80+0x125` — the gull's heading roll, the **second**
/// of the two draws a finished dock spends (§5.2). The first is the gull's
/// own `Guy::init_real`, marked where `init_guys` spends it.
pub const SITE_GULL_ANGLE: &str = "Dock::init+0x125";

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
    pub(crate) fn unit_domain_of(&self, u: usize) -> Domain {
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
            // `TypeIndex::GULLBIRD`. `Guy::set_anim` names the three gaia
            // bird types by identity in its walk arm (`set_anim:620`), so a
            // gull that carried the default −1 could never throw the wing
            // beat's coin — the pasture's own bug, one type over
            // (`docs/SYNC.md` §3.11).
            unit.type_index = self.unit_types[gt].type_index;
            unit.movement.speed = self.type_speed(9, gt);
            unit.movement.turning = self.turning_for(gt);
            // `Unit::init@00612100:282–309` (`crate::stance`).
            unit.stance = self.init_stance(9, gt);
            let u = self.add_unit(unit);
            self.init_guys(u, Some(gt));
            // `Unit::set_angle(gull, (r % 7) × 0x0aaaaaaa − 0x40000000, 7,
            // 0)`. The third argument is 0, so `Guy::set_angle` writes the
            // guy's `des_angle` alone — and `do_air_physics` forces the
            // guy's own angle from the unit's on every later frame, which
            // is why nothing here turns it.
            self.mark(SITE_GULL_ANGLE);
            let angle = (self.rng.get(0, 0xffff) % 7).wrapping_mul(0x0aaa_aaaa) - 0x4000_0000;
            self.unit_set_angle(u, crate::movement::Angle(angle));
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
            // `Unit::close(0, −1, 0)`, and `Object::close`'s hold on the
            // number at its foot (`docs/COMBAT.md` §59.3).
            gull.health = 0;
            gull.on_map = false;
            gull.hold_frames = Sim::CLOSE_HOLD;
        }
        let docks = &mut self.docks[w];
        while docks.mark > 0 && !docks.slots[docks.mark - 1].active() {
            docks.mark -= 1;
        }
    }

    // ------------------------------------------------------------------
    // §7: the civilian's island
    // ------------------------------------------------------------------

    /// `ObjectData::is_cargo@00653600`: an on-map unit whose type is a unit
    /// type in `0x32..=0x19d` of the **land** domain. What can ride a boat.
    fn is_cargo(&self, u: usize) -> bool {
        let ti = self.units[u].type_index;
        self.units[u].on_map
            && (0x32..=0x19d).contains(&ti)
            && self.unit_domain_of(u) == Domain::Land
    }

    /// `Unit::think_civilian_transport(colonise)@005f40d0` (§7) — the AI's
    /// "there is nothing left for me here, and there is an island".
    ///
    /// Two callers, and this crate has both. `Unit::think_scout`'s tail
    /// asks with `colonise = 0` when its own search came back empty, and
    /// what it wants is a **region it has not scouted** that its own region
    /// coasts a sea with. `Unit::think_peasant`'s head asks with
    /// `colonise = 1` — an AI citizen offers itself for the boat before it
    /// looks for work — and that arm is the only writer of the census's
    /// `xport_peasants` throttle between sweeps.
    ///
    /// The search: every land region `1..=0x3f` with cells, accepted by
    /// [`Sim::go_here`] (§9.4) and by the colonise/scouted test; then the
    /// first sea region `0x41..=0x7e` coasting both mine and it; then that
    /// region's cell list, **strided** — `max(16, size / 50)`, phased by
    /// `o + frame`, one pass per phase until a pass finds anything — for a
    /// shore cell on the land side (`coast_here` and no water tile in it).
    /// The nearest by `vector_dist × max(1, danger)` wins, and the unit is
    /// put in a group of one and sent to the cell's centre.
    ///
    /// SEAMS: `unit_masks & 0x100` (an exploring unit) is clear on every
    /// unit in every capture, so the three arms that re-enter
    /// `think_scout(1)` are unreachable and the order is always `MOVE_TO`;
    /// the danger grid has no writer, so [`World::danger_half`] answers 0
    /// and every score is its distance.
    ///
    /// The document says the region loop skips the unit's own region. It
    /// does not — `005f4283` gates on `Region.size` alone — and what keeps
    /// a scout from choosing home is the `scouted` bit its own tail has
    /// just set. Amended in §7.
    pub(crate) fn think_civilian_transport(&mut self, u: usize, colonise: bool) -> bool {
        let who = self.units[u].owner;
        let w = who as usize;
        let domain = self.unit_domain_of(u);
        let always = self.units[u]
            .ty
            .is_some_and(|t| self.unit_types[t].cols.unit_flags & 0x10 != 0);
        if domain == Domain::Sea && !always {
            return false;
        }
        if domain != Domain::Air
            && (self.transport_level(who) < self.unit_transport_type(u)
                || !self.unit_can_transport(u)
                || !self.is_cargo(u))
        {
            return false;
        }
        let my_cell = self.units[u].pos.cell();
        let Some(my_region) = self.world.tregion(self.units[u].pos.tile()) else {
            return false;
        };
        if colonise {
            let census = &self.ai[w].census;
            let sent = census.xport_peasants;
            if sent >= self.city_num(who)
                || self.world.owner(my_cell).player() != Some(who)
                || self.reg_cities(who, my_region) == 0
            {
                return false;
            }
        }
        // `Region.size` for every region at once: the loop below asks for
        // sixty-three of them and each answer is a sweep of the grid.
        let sizes: Vec<i32> = (0..self.world.region_count() as u16)
            .map(|r| self.world.region_size(r))
            .collect();
        let size = |r: u16| sizes.get(r as usize).copied().unwrap_or(0);

        let mut best = 99_999_999;
        let mut best_cell: Option<Cell> = None;
        // The original walks region **slots** — land `1..=0x3f`, sea
        // `0x41..=0x7e` — because its own numbering puts the two kinds in
        // those bands. This crate numbers its regions densely in order of
        // first appearance, so the band is the terrain instead.
        //
        // SEAM: that also makes the *order* this crate's rather than the
        // original's, and the order decides a tie — `score < best` keeps
        // the first. On every capture so far the two agree: the dump's land
        // regions come out in the same order as the sweep's, one apart.
        let lands: Vec<u16> = self
            .world
            .regions()
            .filter(|&(_, t)| t == crate::world::Terrain::Land)
            .map(|(r, _)| r)
            .collect();
        let seas: Vec<u16> = self
            .world
            .regions()
            .filter(|&(_, t)| t == crate::world::Terrain::Sea)
            .map(|(r, _)| r)
            .collect();
        for r in lands {
            if size(r) == 0 {
                continue;
            }
            let g = self.go_here(r, who);
            let accept = if g & 1 != 0 {
                if colonise {
                    self.ai[w].census.reg_xport_peasants.get(r as usize) == Some(&0)
                } else {
                    !self.world.region_scouted(r, who)
                }
            } else if colonise || g & 2 == 0 {
                false
            } else {
                !self.world.region_scouted(r, who)
            };
            if !accept {
                continue;
            }
            let Some(s) = seas.iter().copied().find(|&s| {
                size(s) != 0 && self.world.is_coast(s, my_region) && self.world.is_coast(s, r)
            }) else {
                continue;
            };
            let stride = (size(r) / 0x32).max(0x10);
            let len = size(r);
            let mut phase = i32::from(self.units[u].index)
                .wrapping_add(i32::try_from(self.frame).unwrap_or(i32::MAX));
            for _ in 0..stride {
                let start = phase.rem_euclid(stride);
                if start < len {
                    let mut found = false;
                    for c in self.world.region_coords_strided(r, start, stride) {
                        if self.world.coast_here(r, s, c) == 0 || self.world.num_waterhalf(c) != 0 {
                            continue;
                        }
                        found = true;
                        let danger = self.world.danger_half(who, c);
                        let dist = crate::world::vector_dist(c.x - my_cell.x, c.y - my_cell.y);
                        let score = dist * danger.max(1);
                        if score >= 0 && score < best {
                            best = score;
                            best_cell = Some(c);
                        }
                    }
                    if found {
                        break;
                    }
                }
                phase += 1;
            }
        }
        let Some(cell) = best_cell else {
            return false;
        };
        let mut g = crate::group::Group::stack(who);
        self.group_add(&mut g, u);
        if !self.push_group(&mut g, true) {
            return false;
        }
        let to = Pos::new(cell.x * 0x300 + 0x180, cell.y * 0x300 + 0x180);
        self.group_action_move_to(
            &g,
            to,
            crate::orders::QueuePos::New,
            false,
            crate::movement::Angle(0),
            crate::orders::MoveKind::MoveTo,
            false,
        );
        if colonise {
            self.ai[w].census.xport_peasants += 1;
            // **The destination cell's own region**, not `r` — audit A.36's
            // open question is whether that can ever be a sea index.
            if let Some(dr) = self.world.region_of(cell)
                && let Some(n) = self.ai[w].census.reg_xport_peasants.get_mut(dr as usize)
            {
                *n += 1;
            }
        }
        true
    }

    // ------------------------------------------------------------------
    // §6: boarding — the cast order, and the shore conversion
    // ------------------------------------------------------------------

    /// `Unit::do_cast(order)@005ebfe0`, along the **untargeted** arm — the
    /// half a craft with no `b`/`c`/`d` in its `FLAGS` takes
    /// (`docs/TRANSPORT.md` §6, `docs/ORDERS.md` §6.9).
    ///
    /// The whole of the targeted half — a spell with `spell_flags & 0xe`,
    /// which is every spy craft — is a stated seam: nothing in this crate
    /// issues one. What is modelled is the path the transport `0x28a` and
    /// the pack/unpack family walk, and the order they walk it in, because
    /// that order is the frame's draws:
    ///
    /// 1. the cost, once per order (`pay_cast_costs`); the crafts issued
    ///    here have empty `COST`, `COST2` and `MANA`, so it never refuses;
    /// 2. on the first frame only (`spell_time == 0`) the caster's
    ///    animation. `CHAR_PACK` for a pack craft, `CHAR_UNPACK` for an
    ///    unpack one, `CHAR_DEFAULT` for everything else — and the state
    ///    test that goes with it: a pack whose caster is **already** packed
    ///    and an unpack whose caster is **not** both die here, before the
    ///    clock starts. A **rare collector** unpacking is re-seated in
    ///    between: `good_merchant_spot` or the order dies, then
    ///    `set_new_location(unit-cell centre, snap)` — which teleports
    ///    every tracked crew figure onto its offset — then
    ///    `set_angle(guy 0's angle)`;
    /// 3. on that same frame, and for `0x28a` alone, the shore test: no
    ///    water within `unit_board_distance` and the order dies here;
    /// 4. the clock — `spell_time += 1`, then a `JOB_TIME` above it
    ///    returns. Transport's is **0**, so it casts on its first frame;
    ///    the fishing boat's `0x292` is **40**, so it casts on its
    ///    fortieth;
    /// 5. `SpellType::cast`, and then — for **every craft but `0x28a`** —
    ///    `kill_current_order`. The transport is the exception because
    ///    `cast_transport` has already moved the whole list onto the boat,
    ///    and the boat kills this order there.
    ///
    /// SEAMS, all stated: the captain check between 4 and 5 — for `0x28a`
    /// alone, a figure that is not a captain and whose `get_captain`'s
    /// current order is `CAST_SPELL` takes `spell_time − 1` and returns
    /// (`005ebfe0`, after the clock) — is not modelled. The captain's cast
    /// boards its whole squad ([`Sim::cast_transport`]'s `board`), so only
    /// a member holding a transport cast of its own and stepped **before**
    /// its captain reaches it; no staging has one (item 1235). Nor is the
    /// general's `has_general(0, 0x162)` extra `spell_time` step; and nor
    /// is the whole non-spell-type arm, which is
    /// `LeaderData::current_upgrade` + `set_type` and reaches no craft
    /// index at all. `is_rare_collector`'s re-seat is **in** as of item
    /// 210 — step 2's second half.
    pub(crate) fn do_cast(&mut self, u: usize, order: crate::orders::CastOrder) {
        use crate::orders::spell;
        let s = order.spell;
        if self.spell(s).is_some_and(|d| d.targeted()) {
            // The targeted half, `crate::cast` (item 790).
            self.do_cast_targeted(u, order);
            return;
        }
        if !order.paid
            && let Some(front) = self.units[u].orders.front_mut()
            && let crate::orders::Body::Cast(c) = &mut front.body
        {
            c.paid = true;
            // `pay_cast_costs@00676c40`: a unit caster's `mana_burn` takes
            // the craft's `MANA` (`+0x1d0`) — Create Decoys' 1,000 on the
            // General (`docs/GOLDEN.md` §48); the pack, unpack and
            // transport crafts ask none.
            let mana = self.spell(s).map_or(0, |d| d.mana);
            self.units[u].mana_burn = self.units[u]
                .mana_burn
                .saturating_add(i16::try_from(mana).unwrap_or(i16::MAX));
        }
        if self.units[u].spell_time == 0 {
            let packed = self.units[u].combat.packed;
            let anim = if spell::is_pack(s) {
                if packed {
                    self.kill_current_order(u);
                    return;
                }
                crate::anim::PACK
            } else if spell::is_unpack(s) {
                if !packed {
                    self.kill_current_order(u);
                    return;
                }
                // **The rare collector's re-seat**, `005eca9c`–`005ecb0d`,
                // between the packed test and the animation. It is the
                // last thing `do_cast` does before `set_anim`, and for a
                // merchant it is three calls: `good_merchant_spot` on the
                // unit's own tile, which kills the order when it answers
                // no; `Unit::set_new_location(centre, 1, 1)` onto the
                // **unit-cell** centre of where it stands
                // (`div_3_table[(x ^ 0x63637) >> 4] * 0x30 + 0x18`); and
                // `Unit::set_angle(guy 0's angle)`.
                //
                // The point is almost always the one the unit already
                // holds — a merchant walks to a tile corner and stops on
                // a cell centre — so what the call is *for* is its third
                // argument: the crew loop **puts** every tracked figure
                // on its offset instead of leaving it to walk after the
                // leader. See [`Sim::set_new_location`].
                if self.is_rare_collector(u) {
                    let tile = self.units[u].pos.tile();
                    if !self.good_merchant_spot(u, tile) {
                        self.kill_current_order(u);
                        return;
                    }
                    let seat =
                        crate::collide::ucell_centre(crate::collide::ucell(self.units[u].pos));
                    self.set_new_location(u, seat, true);
                    let facing = self.units[u].movement.facing;
                    self.unit_set_angle(u, facing);
                }
                crate::anim::UNPACK
            } else if self.is_hero_unit(u)
                && matches!(s, spell::AMBUSH | spell::FORCED_MARCH | spell::RALLY)
            {
                // A hero's three crafts of its own (`005eca5b`–`005eca9a`):
                // an attack slot each, asked with the third argument 0.
                match s {
                    spell::AMBUSH => crate::anim::ATTACK1,
                    spell::FORCED_MARCH => crate::anim::ATTACK2,
                    _ => crate::anim::ATTACK3,
                }
            } else {
                crate::anim::DEFAULT
            };
            let reroll = !matches!(
                anim,
                crate::anim::ATTACK1 | crate::anim::ATTACK2 | crate::anim::ATTACK3
            );
            self.mark(crate::anim::SITE_CAST);
            self.set_anim(u, anim, false, reroll);
            if s == spell::TRANSPORT {
                let barge = self.transport_type_for(u);
                let spot = barge.and_then(|b| {
                    self.find_nearby_spot_type(
                        b,
                        self.units[u].pos,
                        0,
                        self.tuning.unit_board_distance,
                        0,
                        BOARD_BEARING,
                    )
                });
                if spot.is_none() {
                    self.kill_current_order(u);
                    return;
                }
            }
        }
        self.units[u].spell_time += 1;
        if self.units[u].spell_time < self.spell_job_time(s) {
            return;
        }
        self.units[u].spell_time = 0;
        // `SpellType::cast@00676ce0` — the switch, behind an `is_castable`
        // that has to answer **3**. The two arms this crate reaches:
        if s == spell::TRANSPORT {
            // `is_castable`'s head (`00675c9f`): a **decoy** (`unit_masks & 1`)
            // answers 0 to every craft but pack and unpack, so
            // `SpellType::cast` is a no-op for it and the order stays at the
            // head with the clock back at 0 (item 1401: East Indies' `1/128`
            // on 16160 spends its figure's draw and builds no boat).
            if self.unit_can_transport(u) && !self.units[u].decoy {
                // `is_castable`'s `0x28a` case, the only test it makes for
                // this spell; a cast that fails it falls out of
                // `SpellType::cast` doing nothing at all.
                self.cast_transport(u);
            }
            // …and the transport order is not killed here: see step 5.
            return;
        }
        // `do_cast@005ebfe0`'s tail (`005ece96`): `paid` is cleared before
        // the kill, so a cast that ran its course hands nothing back
        // ([`Sim::kill_current_order`]'s cast arm).
        self.clear_cast_paid(u);
        if spell::is_unpack(s) && self.units[u].combat.packed {
            // `is_castable`'s `0x28c`/`0x28e`/`0x290`/`0x292` case: a map
            // unit that is still packed.
            self.cast_unpack(u);
        }
        if spell::is_pack(s) && !self.units[u].combat.packed {
            // …and its `0x28b`/`0x28d`/`0x28f`/`0x291` mirror: a map unit
            // that is still unpacked (`00675bc0`, item 1370).
            self.cast_pack(u);
        }
        // The untargeted crafts of chapter thirty-nine, behind the same
        // `is_castable(o, who, 1) == 3` (`docs/GOLDEN.md` §48).
        // …and the General's Forced March (`cast_march`, `docs/AI.md`
        // §99.13). SEAM: Ambush's `cast_ambush` and Rally's are not built.
        if matches!(
            s,
            spell::TO_ARMS | spell::CIVILIAN | spell::CREATE_DECOY | spell::FORCED_MARCH
        ) && self.spell_castable(s, u)
        {
            match s {
                spell::TO_ARMS => self.cast_to_arms(u),
                spell::CIVILIAN => self.cast_civilian(u),
                spell::FORCED_MARCH => self.cast_march(u),
                _ => self.cast_create_decoy(u),
            }
        }
        self.kill_current_order(u);
    }

    /// `SpellType::cast_unpack(o, who)@006709c0` — what the fishing boat's
    /// `0x292` and the siege engine's `0x28c` actually do
    /// (`docs/ORDERS.md` §6.9).
    ///
    /// For everything but the two merchants and the fur trapper the only
    /// state it changes is the **packed bit**. What follows the bit here
    /// is derived rather than stored: `Unit::update_los` (`+0x160`) is
    /// what run58's `mylos 4 → 6` on frame 4989 records, and it is the
    /// packed clamp lifting — [`Sim::unit_los`] recomputes on every read,
    /// so clearing the bit *is* the update.
    ///
    /// **And then the disc is lit** (`docs/COMBAT.md` §56): the next call
    /// is `update_seen(0)` (`+0x174`, `push $0x0` at `670b82`), the whole
    /// disc at the new line of sight. Nothing else would light it: the
    /// tail's `set_new_location` is to the unit's own position and crosses
    /// no half-cell, so a siege engine that unpacks where it stands kept
    /// its packed four-tile disc until the next `update_all_seen`. run146's
    /// catapult `0/6` is the diff: unpacked on 779, it searches on 780 at
    /// `idle 1` and takes arena A's hoplite seven tiles off, which the
    /// four-tile disc hid.
    ///
    /// `Unit::update_gpiece` follows the bit, and it is the second half of
    /// the deploy: the `-PACKED` art and the plain piece are two entries of
    /// `unit_graphics.xml` — one carrying `CHAR_UNPACK` and the other
    /// `CHAR_PACK` — so clearing the bit is what puts the boat's guy on the
    /// piece it will play everything else from (`docs/ANIM.md` §3.4).
    ///
    /// **The merchant arm** (`TypeIndex` `0x3d`/`0x3e`/`0x190`, the exact
    /// ids [`Sim::is_merchant`] tests), `006709c0`'s middle: the tile
    /// under the trader, `div_3_table[pos >> 6]`, has to answer
    /// `good_merchant_spot` or the cast returns **before the bit** — the
    /// unit stays packed and `do_cast` kills the order; otherwise
    /// `set_new_location(tile × 0xc0, 1, 1)` snaps it and its crew onto
    /// the tile's corner, the four tiles under it are blocked
    /// ([`Sim::merchant_footprint`]) and the leader's `0x2000000` goes up.
    /// East Indies' AI Merchant `1/20` finishes its unpack on 7662 at the
    /// unit-cell centre (28632, 24024) and stands at (28608, 24000) from
    /// block 7663; the footprint is what sends gaia's sheep `8/1` round
    /// its tile corner on 11577, eight frames longer than a straight line
    /// (`docs/MERCHANT.md` §3.2).
    ///
    /// SEAMS: the arm's `MiscAccess::scene->recalc_builds = 1` and the
    /// head's `UnitData::announce_frame = −1` (`+0x14c`), which feed the
    /// interface and no record here; and the `set_new_location` at the
    /// tail, which re-seats the unit on its own position.
    pub(crate) fn cast_unpack(&mut self, u: usize) {
        if !self.units[u].alive() || !self.units[u].on_map {
            return;
        }
        if self.is_merchant(u) {
            let t = self.units[u].pos.tile();
            if !self.good_merchant_spot(u, t) {
                return;
            }
            let corner = Pos::new(t.x * UNITS_PER_TILE, t.y * UNITS_PER_TILE);
            self.set_new_location(u, corner, true);
            self.merchant_footprint(u, true);
            let who = self.units[u].owner;
            self.economy_changed(who);
        }
        self.units[u].combat.packed = false;
        // `cast_unpack@006709c0`'s `update_los` (vtable `+0x160`): the
        // clamp lifts here and nowhere earlier (`docs/VISION.md` §2).
        self.update_los(u);
        self.update_seen(u, false);
        self.update_gpiece(u);
    }

    /// `SpellType::cast_pack(o, who)@00670be0` — `cast_unpack`'s mirror,
    /// what `Unit::work`'s pack arm casts ([`Sim::pack_before_move`],
    /// `docs/ORDERS.md` §6.9.2).
    ///
    /// For a unit alive and on the map: the merchant arm first — the four
    /// tiles under a trader are **released**, `set_blocked_at(…, 0)` in
    /// `cast_unpack`'s own order, and the leader's `0x2000000` goes up —
    /// then `unit_masks |= 0x80000`, `update_los` (`+0x160`), which takes
    /// the packed four-tile clamp, `update_seen(0)` (`+0x174`),
    /// `update_gpiece`, and `set_new_location` on the unit's own position.
    /// run544's Bombard `1/132` finishes its pack on 15947: block 15948
    /// prints the bit and `mylos 4`, where it stood at 14 through the
    /// whole cast.
    ///
    /// SEAMS: `MiscAccess::options->rebuild = 1` and `UnitData::
    /// announce_frame = −1` (`+0x14c`), which feed the interface and no
    /// record here.
    pub(crate) fn cast_pack(&mut self, u: usize) {
        if !self.units[u].alive() || !self.units[u].on_map {
            return;
        }
        if self.is_merchant(u) {
            self.merchant_footprint(u, false);
            let who = self.units[u].owner;
            self.economy_changed(who);
        }
        self.units[u].combat.packed = true;
        self.update_los(u);
        self.update_seen(u, false);
        self.update_gpiece(u);
        let at = self.units[u].pos;
        self.set_new_location(u, at, true);
    }

    /// **A deployed merchant's four tiles** — the two-by-two whose
    /// bottom-right corner is the tile under it, the same square
    /// `good_merchant_spot` vetted — blocked by `cast_unpack`'s merchant
    /// arm and released by `Unit::close@0060ee50` (and by
    /// `SpellType::cast_pack@00670be0`, which nothing here casts), each
    /// through `World::set_blocked_at` ([`Sim::set_blocked_at`], road and
    /// all) in the original's
    /// order: `(x, y)`, `(x−1, y)`, `(x, y−1)`, `(x−1, y−1)`.
    pub(crate) fn merchant_footprint(&mut self, u: usize, on: bool) {
        let t = self.units[u].pos.tile();
        for (dx, dy) in [(0, 0), (-1, 0), (0, -1), (-1, -1)] {
            self.set_blocked_at(Pos::new(t.x + dx, t.y + dy), on);
        }
    }

    /// The boat a unit becomes: `current_upgrade(MERCHANTFLEET)` for a
    /// caravan, `current_upgrade(TRANSPORTBARGE)` for everything else, each
    /// falling back to its base type — which `TechTree::current_upgrade`
    /// already does by returning what it was given.
    fn transport_type_for(&self, u: usize) -> Option<usize> {
        let base = if self.unit_is(u, ty::CARAVAN) {
            ty::MERCHANTFLEET
        } else {
            ty::TRANSPORTBARGE
        };
        let who = self.units[u].owner as usize;
        // A tree that does not carry the base type — the harness's small
        // fixtures — has no upgrade to offer, and the base is the answer.
        let id = match self.tech.get(who) {
            Some(p) if base < self.tech_tree.types.len() => {
                self.tech_tree.current_upgrade(&self.setup, p, base)
            }
            _ => base,
        };
        self.unit_types.iter().position(|t| t.tree == Some(id))
    }

    /// `SpellType::cast_transport(o, who)@00670db0` (§6) — the shore
    /// conversion itself.
    ///
    /// A land unit that has reached the waterline is not moved onto it: a
    /// boat is created at the unit's own position, walked to the water
    /// [`Sim::find_nearby_spot_type`] found, given the unit's damage, angle,
    /// **order list and path stack**, and the unit goes inside it. The
    /// order list arrives with this very cast at its head, which is why the
    /// boat's first act is `kill_current_order`: it throws the cast away and
    /// keeps the move that was being walked.
    ///
    /// The path is handed over whole — the original inverts the stack and
    /// then pops it onto the boat, which restores the order it was in — and
    /// then its **top** waypoint has the embark flag cleared when its region
    /// is the boat's own, so the boat does not try to board a transport of
    /// its own on the first step.
    ///
    /// SEAMS, all stated: the `MARINES` arm (`unit_masks2 & 0x200` and
    /// `update_speed`), the caravan's slot hand-over, the console sound, the
    /// selection group's swap and `replace_hotunit` — none of them is state
    /// this crate keeps.
    pub(crate) fn cast_transport(&mut self, u: usize) {
        if !self.units[u].alive() || !self.units[u].on_map {
            return;
        }
        if self.unit_domain_of(u) != Domain::Land {
            return;
        }
        let Some(ty) = self.transport_type_for(u) else {
            return;
        };
        let at = self.units[u].pos;
        let Some(spot) = self.find_nearby_spot_type(
            ty,
            at,
            0,
            self.tuning.unit_board_distance,
            0,
            BOARD_BEARING,
        ) else {
            return;
        };
        let who = self.units[u].owner;
        let Some(index) = self.find_free(who, UNIT_BASE, BUILD_BASE) else {
            return;
        };
        let mut boat = Unit::new(who, index, at, self.unit_types[ty].hits);
        boat.kind = self.unit_types[ty].kind;
        boat.ty = Some(ty);
        boat.type_index = self.unit_types[ty].type_index;
        // **`Unit::update_speed`, not the type's raw `MOVES`.** The boat is
        // born like any other unit and takes the cached speed the same way
        // — which for a `NAVAL` type whose owner holds the Whales rare is
        // `MOVES × (WHALES_SHIPS_MOVE + 100) / 100`. run86 measures it:
        // East Indies' barge `1/22` prints `myspeed 30` on a `MOVES` of 25
        // and steps (−27, −13) a frame where this crate stepped (−23, −11),
        // and 190 frames of that is the whole of the 792 the word parted on
        // (`docs/TRANSPORT.md` §6.2, `docs/MOVEMENT.md` §1).
        boat.movement.speed = self.type_speed(who, ty);
        boat.movement.turning = self.turning_for(ty);
        // `Unit::init@00612100:282–309` (`crate::stance`).
        boat.stance = self.init_stance(who, ty);
        let b = self.add_unit(boat);
        // `Unit::init` → `Guy::init_real`: the boat's one figure, one draw.
        self.init_guys(b, Some(ty));
        // **And `Unit::set_type`'s count** (item 1228, §16): the boat is
        // born through `Objects::init_unit`, whose `set_type` moves
        // `num_units`, `control` and `active` for a type with population.
        // A Merchant Fleet is one, a Transport Barge is not; the caravan
        // inside stays counted as it was. Without it the original's
        // `control` stood one above this crate's for every fleet at sea,
        // and East Indies' second `create_units` pass on 10183 passed the
        // population gate here and not there.
        if self.counts_in_muster(b) {
            self.track_unit_type(who, ty, 1);
        }
        self.same_damage(b, u);
        // The boat is born on the caster's land tile and walked to the
        // water; it is a sea unit crossing the shore the other way, so the
        // step is taken while the caster still holds the land.
        self.set_new_location(b, spot, true);
        // **And the move onto the water reveals** (`docs/VISION.md` §11).
        // The call is `set_new_location(boat, x, y, 1, 1)` — `671027`..
        // `67102f` push `1, 1, y, x` — and `005f8d20`'s tile arm, on a
        // half-cell crossing, calls `update_seen(param_3 == 0)`: the
        // **whole disc** at the spot, on top of the one `add_to_world`
        // threw at the caster's point. This crate's `set_new_location`
        // carries no reveal; its move-step caller does it in `moved_to`.
        self.moved_to(b, at, false);
        let angle = self.units[u].movement.heading;
        self.units[b].movement.set_facing(angle);
        // The orders, in order, then the boat throws away the cast at the
        // head of them.
        let list = std::mem::take(&mut self.units[u].orders);
        self.units[b].orders = list;
        self.kill_current_order(b);
        // The path stack, whole, with the top's embark flag dropped when it
        // is already on the boat's own side of the shore.
        let mut path = std::mem::take(&mut self.units[u].path);
        if let Some(top) = path.last_mut()
            && self.world.tregion_alt(top.to.tile()) == self.world.tregion_alt(spot.tile())
        {
            top.flags &= !crate::orders::path_flag::TRANSPORT;
        }
        self.units[b].path = path;
        self.units[b].auto_transport = true;
        self.clear_orders(u);
        self.board(u, b);
    }

    /// `Unit::same_damage(o, who)@005f9400`: the boat comes out of the
    /// conversion as damaged, in 256ths of its own hit points, as the unit
    /// that cast it.
    ///
    /// SEAM: the original walks both squads and spreads the damage figure by
    /// figure, killing whole figures off a squad too hurt to carry it. Every
    /// unit that boards in a capture so far is a single figure at full
    /// health, where the walk and this line agree on zero.
    fn same_damage(&mut self, boat: usize, u: usize) {
        let hits = self.units[u].max_health.max(1);
        let frac = ((hits - self.units[u].health).max(0) << 8) / hits;
        let boat_hits = self.units[boat].max_health;
        self.units[boat].health = boat_hits - ((boat_hits * frac) >> 8);
    }

    /// `Unit::go_inside(o, who, 0)@0061a2e0` with a **unit** for a host —
    /// the boarding half of `docs/CITIES.md` §6's garrison, and the reason a
    /// passenger stops being stepped: `Unit::process` runs no order for a
    /// unit that is inside something.
    ///
    /// **The whole squad boards** (item 1235, `docs/TRANSPORT.md` §17,
    /// `docs/GOLDEN.md` §52). With `param_3 == 0` the function first climbs
    /// `o_up` (`+0x8e`) to the captain — no liveness asked — and inserts
    /// that; then, at `61a48b`, when `o_down` (`+0x90`) is non-negative and
    /// that figure's `flags & 1` is set, it calls itself on it with
    /// `param_3 = 1`, so the chain goes aboard captain first. Between the
    /// two, at `61a450`..`61a486`, a host whose vslot `0x18` answers (it is
    /// `return 1` on `Unit::vftable`, `return 0` on `Build::vftable`) and a
    /// figure whose type's `uber_size` (`+0x308`) is over 1 take `path.length
    /// = 0`, `close_orders(0)`, `clear_partial_path` and `update_action` —
    /// [`Sim::clear_orders`]' four — so a member's own group move dies
    /// aboard. run466 block 1357: barge `0/10` `inside_down 7`, `0/7`
    /// `inside_down 8`, `0/8` `inside_down 9`, and `0/8`/`0/9` `orders_x/y`
    /// on their own points.
    ///
    /// SEAM: the `+0x68 & ~0x4000000` beside that clear (the move-facing
    /// bit `add_move_facing_order` sets) is state this crate does not keep.
    fn board(&mut self, u: usize, boat: usize) {
        let mut head = u;
        for _ in 0..self.units.len() {
            let Some(a) = self.units[head].o_up else {
                break;
            };
            head = a;
        }
        let mut at = Some(head);
        // Bounded by the list: a cycle would hang the original.
        for _ in 0..self.units.len() {
            let Some(f) = at else { break };
            self.coll_remove(f);
            self.chain_remove(f);
            {
                let unit = &mut self.units[f];
                unit.inside_unit = Some(boat);
                unit.on_map = false;
                unit.movement.dest = None;
                unit.combat.target = None;
                unit.combat.mandatory = false;
            }
            if self.units[f]
                .ty
                .is_some_and(|t| self.unit_types[t].combat.uber_size > 1)
            {
                self.clear_orders(f);
            }
            at = self.units[f].o_down.filter(|&d| self.units[d].alive());
        }
    }

    /// `Object::eject_contents(0, -1, 1, 1)` and the death behind it —
    /// `set_new_location`'s other shore arm (§6): a **sea** unit that steps
    /// off the water puts its passengers out where it stands, and dies if
    /// that leaves it carrying nothing.
    ///
    /// **It is `cast_transport` run backwards** (§6.4), and the four steps
    /// are in the original's own order:
    ///
    /// 1. `param_4 = 1`: the passenger's own list and partial path are
    ///    closed before it comes out — vacuous here, because the cast
    ///    emptied them, and kept because the arm is the one the call
    ///    passes;
    /// 2. `Unit::come_out` — the spot, and the army coin at its tail
    ///    ([`Sim::come_out_join_army`], `docs/ARMY.md` §4);
    /// 3. `Unit::same_damage(passenger, boat)` — the mirror of the boarding
    ///    line, the passenger taking the boat's damage in 256ths;
    /// 4. `param_3 = 1`: for a passenger whose type's `uber_size` is
    ///    **1**, the boat's whole order list moves back onto it and the
    ///    boat's path stack is inverted and popped onto its own, which
    ///    restores the order it was in; then the top waypoint's embark flag
    ///    (`4`) is cleared — unconditionally here, where the boarding line
    ///    clears it only on a region match.
    ///
    /// **Step 2's spot is `come_out`'s host arm, and every term of it is
    /// the boat's.** `Unit::come_out@00617c10` splits on whether the host
    /// is a unit or a building (the host's vslot `0x1c`): a building gives
    /// the training ring `docs/CITIES.md` §11 has, and a **unit** gives
    /// `angle = host->angle` (`+0x50`), an inner radius of the **host's**
    /// `block_radius` (`+0x240`) and an outer of that plus
    /// `UNIT_DISEMBARK_DISTANCE` — all three read off the host object in
    /// `eax` at `61845c`..`618483`, which the decompiler folds into the
    /// same local it used for the passenger. Only the choice of fallback
    /// arm is the passenger's — `618490` reads `0x240` off `0x18(%ebx)`,
    /// and `ebx` is `this`: its own `block_radius == 0` sweeps
    /// `FILTER_ALL`, then the doubled ring, then the host's own point;
    /// non-zero sweeps `FILTER_NOT_ME` and then the same ring with
    /// collision off, and **refuses** — the passenger stays aboard — if
    /// that finds nothing.
    ///
    /// **run57 pins all three terms at once.** The barge stands at
    /// `(35740, 26706)` with `angle -13303808` — a degree and a quarter
    /// west of due north — and its `BLOCK_RADIUS 3` makes the ring
    /// `[144, 720]` with the sweep's own step of `(720 − 144) / 8 = 72`.
    /// The first ring at the first bearing projects to `(35737, 26562)`,
    /// which snaps to **`(35736, 26568)`** — the scout's own point in
    /// block 3979, exactly. A ring taken from the *passenger's* radius
    /// starts at 48 and lands a quarter-tile short; a bearing of due
    /// south lands nowhere near.
    ///
    /// **And a squad member leaving a building takes the identical arm**
    /// (2026-09-04, item 227): `come_out` swaps its host for `get_captain()`
    /// at `618022`..`618044`, so the two share
    /// [`Sim::come_out_unit_host_spot`] — `docs/CITIES.md` §6.5.1.
    ///
    /// Step 4's `uber_size > 1` arm is [`Sim::disembark_squad`] (item 1223).
    /// SEAM: `num_inside != 0` after the loop — a passenger `come_out`
    /// refused — leaves the original's boat alive and carrying it
    /// (`005f8fd8`); this crate kills the boat regardless. No staging has
    /// crowded a shore enough to refuse a ring.
    pub(crate) fn disembark(&mut self, boat: usize) {
        let bearing = self.units[boat].movement.heading;
        let riders: Vec<usize> = (0..self.units.len())
            .filter(|&i| self.units[i].inside_unit == Some(boat))
            .collect();
        // **The loop reads the boat's `inside_down` again after every
        // passenger** (`eject_contents@0064cd20`'s `while` re-reads the
        // host's `+0x28`), and a captain's `come_out` takes its squad
        // with it: so a squad is one pass, on the chain's head, and its
        // members are never the loop's own (item 1291). run496's block
        // 1902 has one `GroupMoveOrder` on each of `0/7`..`0/9`.
        for r in riders {
            if self.units[r].inside_unit != Some(boat) {
                continue;
            }
            // 1. the `param_4` arm, `update_action` its last call
            //    (`eject_contents@0064cd20`:115) — on a passenger still at
            //    its boarding point with an empty list, so `orders_x/y`
            //    are that point and step 4 does not move them: run249's
            //    `0/6` prints (8428, 34200) ashore on 1160. The head alone:
            //    a member's list died aboard (§17).
            self.units[r].path.clear();
            self.close_orders(r);
            self.clear_partial_path(r);
            self.update_action(r);
            // 2. `come_out`. **A member climbs to its captain first**
            // (`come_out(this, 0)`'s not-a-captain arm, `00617c10:173`,
            // `o_up` at `+0x8e`), so the one that comes out on the boat's
            // ring is the captain whoever heads the chain.
            let mut captain = r;
            for _ in 0..self.units.len() {
                let Some(a) = self.units[captain].o_up else {
                    break;
                };
                captain = a;
            }
            if self.units[captain].inside_unit != Some(boat) {
                // SEAM: a member whose captain already stands ashore — a
                // member `come_out` refused on an earlier pass. The
                // original re-runs the captain's `get_inside < 0` arm and
                // moves the captain; no staging has refused a member.
                continue;
            }
            // **The ring is the boat's and the arm is the passenger's.**
            // `come_out`'s host branch reads `+0x240` off the *host's* type
            // for the inner radius: at `61845c`..`61846a` the host object
            // is in `eax`, and `0x50(%eax)` and `0x240(host->type)` are the
            // bearing and the ring. The `FILTER_ALL`/`FILTER_NOT_ME` split
            // is a different register — `618490` reads `0x240` off
            // `0x18(%ebx)`, and `ebx` is `this`, the passenger.
            // The arm itself is [`Sim::come_out_unit_host_spot`]: a squad
            // member leaving a building reaches the identical code, because
            // `come_out` swaps its host for its captain (`docs/CITIES.md`
            // §6.5.1). The refusal is the same one — the passenger stays
            // inside, and the boat is left carrying it.
            let Some(at) = self.come_out_unit_host_spot(captain, boat) else {
                continue;
            };
            let captain_angle = self.units[captain].movement.heading;
            self.land_passenger(captain, at);
            // **Then the squad, each member on its captain's ring**
            // (`come_out@00617c10:535`: `o_down ≥ 0` and its `flags & 1`
            // give `come_out(o_down, 1)`, right after `set_new_location`
            // and before any of the captain's tail). A member's host is
            // `get_captain()` (`617f2f`), so the bearing and the ring are
            // the captain's, and the captain's `angle` is still the one it
            // boarded with: its own `set_angle(boat)` is its tail's. run496
            // block 1902: `0/7` at (14712, 31224), `0/8` at (14712,
            // 31368) and `0/9` at (14904, 31224), both members turned to
            // 1084948480, `0/7`'s angle on 1901. A refused member stops the
            // walk there (the refusal returns before the recursion).
            let mut members = Vec::new();
            let mut at_f = self.units[captain].o_down;
            for _ in 0..self.units.len() {
                let Some(f) = at_f.filter(|&d| self.units[d].alive()) else {
                    break;
                };
                if self.units[f].inside_unit != Some(boat) {
                    break;
                }
                let Some(spot) = self.come_out_unit_host_spot(f, captain) else {
                    break;
                };
                self.land_passenger(f, spot);
                members.push(f);
                at_f = self.units[f].o_down;
            }
            // Each tail after its own recursion returns: the chain's last
            // member first, the captain last (`add_to_army`'s coin is the
            // tail's last draw, `0061a1f2`).
            for &f in members.iter().rev() {
                self.land_tail(f, captain_angle);
            }
            self.land_tail(captain, bearing);
            // 3. the damage, back the way it came, on the loop's own
            //    passenger alone.
            self.same_damage(r, boat);
            // 4. the order list and the path, back the way they came.
            if self.units[r]
                .ty
                .is_some_and(|t| self.unit_types[t].combat.uber_size == 1)
            {
                self.units[r].orders = std::mem::take(&mut self.units[boat].orders);
                let mut path = std::mem::take(&mut self.units[boat].path);
                if let Some(top) = path.last_mut() {
                    top.flags &= !crate::orders::path_flag::TRANSPORT;
                }
                self.units[r].path = path;
            } else {
                self.disembark_squad(boat, r);
            }
        }
        self.units[boat].health = 0;
        // `Unit::close@0060ee50`'s count, at `0060f3db` (§16): the boat
        // that came in through `set_type`'s `+1` goes out the same gate.
        if self.counts_in_muster(boat)
            && let Some(ty) = self.units[boat].ty
        {
            let who = self.units[boat].owner;
            self.track_unit_type(who, ty, -1);
        }
        // `Object::die(boat, 0, −1, 0)`: `close`, whose `close_orders`
        // (`60fa1c`, after the count) takes what the passengers were not
        // handed, and whose `Object::close` holds the number thirty frames
        // (`docs/COMBAT.md` §59.3). East Indies' barge `1/62` and Merchant
        // Fleet `1/59` were reused here at once, and the original's next
        // births took the numbers above.
        self.close_dead_orders(boat);
        self.hold_dead_slot(boat);
        self.units[boat].on_map = false;
        self.coll_remove(boat);
        self.chain_remove(boat);
    }

    /// `come_out`'s `set_new_location(·, ·, 1, 1)` on a passenger: out of
    /// the boat, onto `at`, into both collision indices.
    fn land_passenger(&mut self, r: usize, at: crate::Pos) {
        self.units[r].inside_unit = None;
        self.units[r].pos = at;
        // `Movement::at` alone would zero the speed and the turn rate,
        // and a unit put ashore with no speed stands there for ever;
        // `come_out`'s building arm carries the same two across.
        // `dest_angle` (`+0x58`) is step 1's, below.
        //
        // **And the body keeps its speeds** (item 1164, `docs/TRANSPORT.md`
        // §6.4): `Guy::last_speed`/`avg_speed` (`+0x80`/`+0x84`) have
        // three writers, `Guy::move`, `Guy::clear` and `Guy::init_real`,
        // and neither `Unit::set_new_location@005f8d20` nor
        // `Guy::set_new_location@005d86f0` is one. A passenger comes
        // ashore at the average it boarded with, and [`turn_speed`]
        // divides its first turn by it: run420's `1/33` lands on 6734
        // at 12 and takes nine frames to face its path, not seven.
        //
        // [`turn_speed`]: crate::movement::turn_speed
        let body = self.units[r].movement.body;
        self.units[r].movement = crate::Movement {
            speed: self.units[r].movement.speed,
            turning: self.units[r].movement.turning,
            heading: self.units[r].movement.heading,
            facing: self.units[r].movement.facing,
            frame_facing: self.units[r].movement.frame_facing,
            mirror: self.units[r].movement.mirror,
            des_angle: self.units[r].movement.des_angle,
            body: crate::movement::Body { pos: at, ..body },
            ..crate::Movement::at(at)
        };
        self.units[r].on_map = true;
        self.coll_add(r);
        self.chain_add(r);
    }

    /// The rest of `come_out` for a passenger put ashore: the computer's
    /// order reset, `set_angle(host->angle)`, the crew seated, the army
    /// coin. `angle` is the host's: the boat's for the captain, the
    /// captain's own boarding angle for a member.
    fn land_tail(&mut self, r: usize, angle: crate::movement::Angle) {
        let at = self.units[r].pos;
        // **`come_out`'s own tail, for a computer player's unit** (item
        // 1164, `docs/TRANSPORT.md` §6.4): past `set_new_location` at
        // `6186c1`, `6187c8` tests `leaders.list[who] & 4`
        // (`is_human`) and skips on it; a plane, and the four
        // `0x32..=0x35` citizen types (`6187ff`..`618811`), skip too.
        // Everyone else takes `path.length = 0`, `close_orders(0)`,
        // `clear_partial_path` and `update_action` at `618813`..`618836`
        // — step 1's four again, **from the spot**. Path and list are
        // already empty, so what it writes is `orders_x/y`: run420's AI
        // merchant `1/33` prints its landing point (38952, 24840) on
        // 6735, where the human scout of run249 keeps its boarding
        // point. `update_action`'s `+0x58 = +0x50` is the boarding
        // heading still — the dump's `dest_angle` 292814848 against
        // `angle` 165478400 on the same block says `set_angle` comes
        // after — which is step 1's value, kept below.
        if !self.nation[usize::from(self.units[r].owner)].human
            && !(0x32..=0x35).contains(&self.units[r].type_index)
        {
            self.units[r].orders_pos = at;
        }
        // `come_out@6191f4`: with a **unit** for a host the passenger is
        // turned to the host's own `angle` (`+0x50`) in `set_angle`'s
        // snapping form, guys included — which is what puts the scout's
        // dog on the track offset the original prints. run57 block 3979
        // reads `angle -13303808` on the scout, the barge's own heading
        // at the frame it ejects.
        //
        // `Unit::set_angle@00605400` writes `+0x50` and the guys and
        // not `+0x58`, so `dest_angle` stays what step 1's
        // `update_action` seeded it with, the heading the passenger
        // boarded on: run249's `0/6` prints 1073741824 on 1160.
        let des_angle = self.units[r].movement.des_angle;
        self.units[r].movement.set_facing(angle);
        self.units[r].movement.des_angle = des_angle;
        // `set_new_location`'s `param_3` reaches `Guy::set_new_location(0,
        // pos, 1)`, which seats the crew **on** its track offset rather
        // than letting it walk there from wherever it boarded. Without
        // it the scout's dog spends the next hundred frames chasing the
        // sea, and a walking guy takes no idle roll — which is the
        // second of the two `Unit::set_anim` draws the original spends
        // when the scout arrives.
        self.seat_guys(r);
        self.come_out_join_army(r);
    }

    /// **Step 4's squad arm** (item 1223, `docs/TRANSPORT.md` §6.4,
    /// `docs/GOLDEN.md` §52): a passenger whose type's `uber_size` is over 1
    /// takes the boat's orders through a group, not a list move
    /// (`Object::eject_contents@0064cd20`, the `+0x308 != 1` arm):
    ///
    /// 1. `Unit::reset_move_orders@005fd080` on the **boat** — every move
    ///    order's `orig_x`/`orig_y` (`+0x44`/`+0x48`) set to its `x`/`y`, so
    ///    the replay below walks to the point the boat was sailing for;
    /// 2. unless the passenger is AI-driven (`unit_masks & 0x40000`) **and**
    ///    in an army: a stack group of the boat, `Group::set_up_insert` (the
    ///    leader's — the boat's — action-flagged orders copied aside),
    ///    `Group::kill(boat)`, `Group::add(passenger)` — its captain and
    ///    chain — `Groups::push_group(who, g, 1)`, and `Group::finish_insert`
    ///    on the slot, which re-issues each copy as a group action at
    ///    `QUEUE_LAST`.
    ///
    /// The path is not handed over on this arm; the replay plans afresh.
    fn disembark_squad(&mut self, boat: usize, r: usize) {
        for o in &mut self.units[boat].orders {
            if let crate::orders::Body::Move(m) = &mut o.body {
                match m.group.as_mut() {
                    Some(gm) => gm.orig = m.dest,
                    None => m.orig = Some(m.dest),
                }
            }
        }
        let owner = self.units[r].owner;
        if self.ai_driven(owner) && self.army_of(r).is_some() {
            return;
        }
        let mut g = crate::group::Group::stack(owner);
        self.group_add(&mut g, boat);
        let insert = self.group_set_up_insert(&g);
        g.list.retain(|&m| m != boat);
        self.group_add(&mut g, r);
        self.push_group(&mut g, true);
        self.group_finish_insert(&g, insert);
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

    /// A barge type, so `cast_transport` has something to build.
    fn barge(sim: &mut Sim) -> usize {
        let mut t = UnitType {
            hits: 50,
            moves: 25,
            ..UnitType::default()
        };
        t.combat.domain = Domain::Sea;
        t.combat.block_radius = 48;
        let b = sim.add_unit_type(t);
        sim.unit_types[b].tree = Some(ty::TRANSPORTBARGE);
        sim.unit_types[b].type_index = ty::TRANSPORTBARGE as i32;
        b
    }

    /// §9.3 and `num_waterhalf`: the shore column is the land side of the
    /// waterline, and the sea column is the water side.
    #[test]
    fn coast_here_names_the_shore_and_waterhalf_counts_the_water() {
        let f = fix();
        let w = &f.sim.world;
        // Cell (7, 3) is land, flagged `HALFLAND`, with the sea next door.
        assert_ne!(w.coast_here(f.land, f.sea, Cell::new(7, 3)), 0);
        // Read from the sea side the answer is the same pair, and non-zero.
        assert_ne!(w.coast_here(f.land, f.sea, Cell::new(8, 3)), 0);
        // A cell in neither, and a cell with no shore next to it.
        assert_eq!(w.coast_here(f.land, f.sea, Cell::new(3, 3)), 0);
        // `num_waterhalf` is 0 off a `HALFLAND` cell whatever its tiles
        // say, and counts the sixteen otherwise. The fixture's shore cells
        // are dry land flagged coastal, so the count is 0 — which is what
        // `think_civilian_transport` wants.
        assert_eq!(w.num_waterhalf(Cell::new(8, 3)), 0, "not HALFLAND");
        assert_eq!(w.num_waterhalf(Cell::new(7, 3)), 0, "no water tile in it");
    }

    /// §6.1 and §6.2 end to end: a land unit with the bit that steps onto
    /// an ocean tile is not moved — it queues the transport spell, and the
    /// next frame's `do_cast` converts it.
    #[test]
    fn a_step_into_the_sea_becomes_a_boat_with_the_walker_inside() {
        let mut f = fix();
        let b = barge(&mut f.sim);
        let u = unit(&mut f.sim, 1, f.citizen, tile_pos(30, 14));
        f.sim.init_guys(u, Some(f.citizen));
        f.sim.units[u].auto_transport = true;
        // A queued move, so the boat has something to inherit, and a path
        // whose top is the water it is about to step onto.
        f.sim.add_move_order(
            u,
            tile_pos(40, 14),
            crate::orders::MoveKind::MoveTo,
            crate::orders::QueuePos::New,
            false,
        );
        let top = crate::orders::PathData {
            to: tile_pos(33, 14),
            tolerance: 0,
            flags: crate::orders::path_flag::TRANSPORT,
        };
        f.sim.units[u].path.push(top);

        // The step itself: refused, and a cast is in front of the move.
        let before = f.sim.rng.seed;
        assert!(
            !f.sim.set_new_location(u, tile_pos(33, 14), false),
            "a land unit does not walk onto the sea"
        );
        assert_eq!(f.sim.units[u].pos, tile_pos(30, 14), "and it does not move");
        assert_eq!(f.sim.rng.seed, before, "the conversion spends no draw");
        assert!(matches!(
            f.sim.units[u].orders.front().map(|o| o.body),
            Some(crate::orders::Body::Cast(c)) if c.spell == crate::orders::spell::TRANSPORT
        ));
        assert_eq!(
            f.sim.units[u].orders.len(),
            2,
            "the move is still behind it"
        );

        // The cast: one `set_anim` draw a figure, then the boat's own.
        let units_before = f.sim.units.len();
        f.sim.work(u, 1);
        assert_eq!(f.sim.units.len(), units_before + 1, "a boat");
        let boat = units_before;
        assert_eq!(f.sim.units[boat].ty, Some(b));
        assert_eq!(f.sim.units[boat].owner, 1);
        assert!(
            f.sim.world.tile_mask(f.sim.units[boat].pos.tile()) & tile::SURFACE
                == tile::SURFACE_OCEAN,
            "the boat is born on the water"
        );
        assert!(f.sim.units[boat].auto_transport, "`unit_masks |= 0x800000`");
        // The order list moved and the cast at its head was thrown away.
        assert!(f.sim.units[u].orders.is_empty());
        assert_eq!(f.sim.units[boat].orders.len(), 1);
        assert!(f.sim.units[boat].orders[0].is_move());
        // The path moved whole, and the top's embark flag is gone: the
        // boat is on the sea side already.
        assert!(f.sim.units[u].path.is_empty());
        assert_eq!(
            f.sim.units[boat].path.last().map(|p| p.flags),
            Some(0),
            "the top is re-pushed with `flags & 4` cleared"
        );
        // And the walker is cargo: off the map, its clock stopped.
        assert_eq!(f.sim.units[u].inside_unit, Some(boat));
        assert!(!f.sim.units[u].on_map);
    }

    /// §6.2 line 3 and `docs/VISION.md` §11: the boat is born on the
    /// caster's point, which `add_to_world` lights, and `set_new_location
    /// (boat, spot, 1, 1)` then crosses a half-cell onto the water and
    /// lights the **whole disc at the spot**. A fog cell three half-cells
    /// seaward of the spot is out of the birth disc's reach and inside the
    /// spot's: seen only when the second reveal is thrown.
    #[test]
    fn a_boat_born_on_the_shore_lights_its_disc_on_the_water() {
        let mut f = fix();
        assert!(f.sim.world.set_fog(vec![0; 24 * 16]));
        let b = barge(&mut f.sim);
        // `LOS 6`: a fog radius of 3.
        f.sim.unit_types[b].los = 6;
        // A sea unit sees from its own half-cell (`docs/VISION.md` §3):
        // no forward projection to carry the birth disc onto the water.
        f.sim.unit_types[b].kind.domain = Domain::Sea;
        let u = unit(&mut f.sim, 1, f.citizen, tile_pos(30, 14));
        f.sim.init_guys(u, Some(f.citizen));
        f.sim.units[u].auto_transport = true;
        assert!(!f.sim.set_new_location(u, tile_pos(33, 14), false));
        let born = f.sim.units[u].pos;
        let boat = f.sim.units.len();
        f.sim.work(u, 1);
        assert_eq!(f.sim.units.len(), boat + 1, "a boat");
        let spot = f.sim.units[boat].pos;
        let half = |p: i32| p / 0x180;
        assert_ne!(
            (half(spot.x), half(spot.y)),
            (half(born.x), half(born.y)),
            "the spot is another half-cell"
        );
        let (fx, fy) = (half(spot.x) + 3, half(spot.y));
        assert!(
            (fx - half(born.x)).abs() > 3,
            "the probe is beyond the birth disc"
        );
        assert!(
            f.sim.world.seen2(fx, fy).is_some_and(|v| v & 2 != 0),
            "the spot's disc is lit for its owner"
        );
    }

    /// **The boat in the muster** (item 1228, §16): `Objects::init_unit`'s
    /// `Unit::set_type` counts a boat whose type has population, and
    /// `Unit::close` takes it out again. A caravan's Merchant Fleet, of
    /// `POP` 1, moves `num_units` and `control` by one while it is at sea;
    /// a citizen's Transport Barge, of `POP` 0, never does. run462's block 10178
    /// holds one of each, and the original's `num_units` has the fleet
    /// alone.
    ///
    /// Made to fail by dropping either call: without the birth's the fleet
    /// is never counted, without the close's it stays counted ashore.
    #[test]
    fn a_merchant_fleet_is_counted_at_sea_and_a_barge_never_is() {
        let mut f = fix();
        let b = barge(&mut f.sim);
        f.sim.unit_types[b].price.pop = 0;
        let mut fleet_t = UnitType {
            hits: 50,
            moves: 25,
            ..UnitType::default()
        };
        fleet_t.combat.domain = Domain::Sea;
        fleet_t.combat.block_radius = 48;
        fleet_t.price.pop = 1;
        let fleet = f.sim.add_unit_type(fleet_t);
        f.sim.unit_types[fleet].tree = Some(ty::MERCHANTFLEET);
        f.sim.unit_types[fleet].type_index = ty::MERCHANTFLEET as i32;
        let caravan = f.sim.add_unit_type(UnitType {
            hits: 40,
            ..UnitType::default()
        });
        f.sim.unit_types[caravan].tree = Some(ty::CARAVAN);

        for (walker_t, boat_t, counted) in [(caravan, fleet, 1), (f.citizen, b, 0)] {
            let u = unit(&mut f.sim, 1, walker_t, tile_pos(30, 14));
            f.sim.init_guys(u, Some(walker_t));
            f.sim.units[u].auto_transport = true;
            let control = f.sim.muster[1].control;
            assert!(!f.sim.set_new_location(u, tile_pos(33, 14), false));
            let boat = f.sim.units.len();
            f.sim.work(u, 1);
            assert_eq!(f.sim.units[boat].ty, Some(boat_t), "the boat");
            assert_eq!(
                f.sim.muster[1].by_type[boat_t], counted,
                "`set_type`'s `+1`, for a type with population"
            );
            assert_eq!(f.sim.muster[1].control, control + counted);
            // Ashore again: the boat puts its passenger out and closes.
            assert!(!f.sim.set_new_location(boat, tile_pos(30, 14), false));
            assert!(!f.sim.units[boat].alive());
            assert_eq!(
                f.sim.muster[1].by_type[boat_t], 0,
                "`Unit::close`'s `−1` at `0060f3db`"
            );
            assert_eq!(f.sim.muster[1].control, control);
        }
    }

    /// The other arm of the same test: a boat that steps off the water
    /// puts its passenger out and dies.
    #[test]
    fn a_boat_that_steps_ashore_ejects_and_dies() {
        let mut f = fix();
        let b = barge(&mut f.sim);
        let rider = unit(&mut f.sim, 1, f.citizen, tile_pos(30, 14));
        let boat = unit(&mut f.sim, 1, b, tile_pos(33, 14));
        f.sim.units[boat].auto_transport = true;
        f.sim.board(rider, boat);
        assert!(!f.sim.set_new_location(boat, tile_pos(30, 14), false));
        assert!(!f.sim.units[boat].alive(), "an empty boat on land dies");
        assert_eq!(f.sim.units[rider].inside_unit, None);
        assert!(f.sim.units[rider].on_map);
    }

    /// **A boat that steps ashore in its own work still takes its figures'
    /// `Guy::move`** (item 1191, `docs/TRANSPORT.md` §6.4):
    /// `Unit::process@00610bc0` runs the `+0x188` think and then
    /// `Guy::process` over the guy array whatever the think did, and the
    /// boat's `Object::die(0)` → `Unit::close` leaves that array alone. A
    /// boat standing on its `des` on the walk with `stopped` set pays the
    /// arrival stand on the frame it dies — East Indies' `1/56` on 8519.
    ///
    /// Made to fail by returning from `process_unit` on the dead boat before
    /// `process_movement`: the figure keeps the walk.
    #[test]
    fn a_boat_that_steps_ashore_in_its_own_work_still_pays_its_arrival() {
        let mut f = fix();
        let b = barge(&mut f.sim);
        let rider = unit(&mut f.sim, 1, f.citizen, tile_pos(30, 14));
        // A step short of the shore, on the water side of the line.
        let at = Pos::new(32 * UNITS_PER_TILE + 4, tile_pos(32, 14).y);
        let boat = unit(&mut f.sim, 1, b, at);
        f.sim.units[boat].auto_transport = true;
        f.sim.init_guys(boat, Some(b));
        f.sim.board(rider, boat);
        f.sim.add_move_order(
            boat,
            tile_pos(30, 14),
            crate::orders::MoveKind::MoveTo,
            crate::orders::QueuePos::New,
            false,
        );
        // The state `1/56` stood in on block 8519: the body on its unit,
        // facing its waypoint, at its base speed on average, the figure on
        // the walk with `stopped` set.
        let bearing = crate::movement::Angle(-1_130_299_392);
        let u = &mut f.sim.units[boat];
        u.movement.speed = 25;
        u.movement.turning.type_turn_speed = i32::MAX;
        u.movement.facing = bearing;
        u.movement.heading = bearing;
        u.movement.body.pos = at;
        u.movement.body.avg_speed = 25;
        u.guys[0].anim = crate::anim::WALK;
        u.guys[0].stopped = true;
        f.sim.process_unit(boat, 1, &mut Vec::new());
        assert!(
            !f.sim.units[boat].alive(),
            "the boat stepped ashore and died"
        );
        assert!(f.sim.units[rider].on_map, "its passenger is out");
        assert_eq!(
            f.sim.units[boat].guys[0].anim,
            crate::anim::DEFAULT,
            "and its figure took the arrival stand on the way out"
        );
    }

    /// A barge a step off the shore with `rider` aboard and a move for the
    /// land, standing — its body on its unit, facing its waypoint, its
    /// figure's running average `avg`.
    fn landing(f: &mut Fix, rider: usize, avg: i32) -> (usize, Pos) {
        let b = barge(&mut f.sim);
        let at = Pos::new(32 * UNITS_PER_TILE + 4, tile_pos(32, 14).y);
        let boat = unit(&mut f.sim, 1, b, at);
        f.sim.units[boat].auto_transport = true;
        f.sim.init_guys(boat, Some(b));
        f.sim.board(rider, boat);
        f.sim.add_move_order(
            boat,
            tile_pos(30, 14),
            crate::orders::MoveKind::MoveTo,
            crate::orders::QueuePos::New,
            false,
        );
        let bearing = crate::movement::Angle(-1_130_299_392);
        let u = &mut f.sim.units[boat];
        u.movement.speed = 25;
        u.movement.turning.type_turn_speed = i32::MAX;
        u.movement.facing = bearing;
        u.movement.heading = bearing;
        u.movement.body.pos = at;
        u.movement.body.avg_speed = avg;
        (boat, at)
    }

    /// **A boat that dies stepping ashore repaints its disc on its
    /// sixty-fourth frame** (item 1223, `docs/GOLDEN.md` §52):
    /// `Guy::process@005e0230`'s tail runs on the dead boat's figure, and
    /// with `avg_speed` 0 on `(frame + o) % 64 == 0` it re-marks the cells
    /// `Object::close` has just cleared. Standing in the open sea, the
    /// barge's cell is its own region's, so the mark lands.
    ///
    /// Made to fail by leaving `coll_repaint`'s guard at alive and on the
    /// map: the cell is clear after the death.
    #[test]
    fn a_boat_that_dies_standing_on_its_phase_frame_leaves_its_disc_marked() {
        let mut f = fix();
        let rider = unit(&mut f.sim, 1, f.citizen, tile_pos(30, 14));
        let (boat, at) = landing(&mut f, rider, 0);
        f.sim.coll_add(boat);
        let o = i64::from(f.sim.units[boat].index);
        f.sim.frame = 64 * 100 - o;
        f.sim.process_unit(boat, f.sim.frame, &mut Vec::new());
        assert!(!f.sim.units[boat].alive(), "the boat stepped ashore");
        let c = crate::collide::ucell(at);
        assert!(
            f.sim.coll.get(c.x, c.y),
            "the dead figure's repaint marked its own cell"
        );
    }

    /// And off its phase frame the death leaves the cell clear: the mark is
    /// the repaint's, not a close that forgot to clear it.
    #[test]
    fn a_boat_that_dies_off_its_phase_frame_leaves_its_cell_clear() {
        let mut f = fix();
        let rider = unit(&mut f.sim, 1, f.citizen, tile_pos(30, 14));
        let (boat, at) = landing(&mut f, rider, 0);
        f.sim.coll_add(boat);
        let o = i64::from(f.sim.units[boat].index);
        f.sim.frame = 64 * 100 - o + 1;
        f.sim.process_unit(boat, f.sim.frame, &mut Vec::new());
        assert!(!f.sim.units[boat].alive(), "the boat stepped ashore");
        let c = crate::collide::ucell(at);
        assert!(!f.sim.coll.get(c.x, c.y), "the close cleared the cell");
    }

    /// **A squad's passenger takes the boat's orders through a group**
    /// (item 1223, `docs/TRANSPORT.md` §6.4): `eject_contents`'
    /// `uber_size > 1` arm resets the boat's move orders to their own point,
    /// pushes a group of the passenger (`push_group(who, g, 1)`) and replays
    /// the boat's action orders onto it with `finish_insert` — a group move
    /// to where the boat was sailing, where the one-man arm hands the list
    /// over whole.
    ///
    /// Made to fail by dropping [`Sim::disembark_squad`]'s call: the
    /// passenger comes ashore with no order and in no group.
    #[test]
    fn a_squad_s_passenger_comes_ashore_with_the_boat_s_move_as_a_group() {
        let mut f = fix();
        let mut t = f.sim.unit_types[f.citizen].clone();
        t.combat.uber_size = 3;
        let squad = f.sim.add_unit_type(t);
        let rider = unit(&mut f.sim, 1, squad, tile_pos(30, 14));
        let (boat, _) = landing(&mut f, rider, 25);
        // A player's move carries the action bit (run466's barges print
        // `flags 5`), and `set_up_insert` copies only what does.
        f.sim.units[boat].orders[0].flags |= crate::orders::flag::ACTION;
        let crate::orders::Body::Move(sailing) = f.sim.units[boat].orders[0].body else {
            panic!("the boat's move");
        };
        f.sim.process_unit(boat, 1, &mut Vec::new());
        assert!(!f.sim.units[boat].alive(), "the boat stepped ashore");
        assert!(f.sim.units[rider].on_map, "its passenger is out");
        let front = f.sim.units[rider].orders.front().expect("an order");
        let crate::orders::Body::Move(m) = front.body else {
            panic!("a move, not {:?}", front.body);
        };
        assert_eq!(
            m.group.map_or(m.dest, |g| g.orig),
            sailing.dest,
            "to the boat's point"
        );
        assert!(
            f.sim
                .pushed
                .iter()
                .any(|p| p.who == 1 && p.list == vec![rider]),
            "in a pushed group of its own"
        );
    }

    /// **A squad comes ashore whole on its captain's pass** (item 1291,
    /// `docs/TRANSPORT.md` §17, `docs/GOLDEN.md` §55): a captain's
    /// `come_out` calls itself down `o_down` with each member's host its
    /// captain, and `eject_contents` re-reads the boat's chain after it, so
    /// the squad arm runs once — one group move on each figure. And the
    /// members are turned to the captain's own angle, which its tail sets
    /// to the boat's only after they are out. run496 block 1902: `0/7`,
    /// `0/8` and `0/9` hold one `GroupMoveOrder` each, and `0/8`'s and
    /// `0/9`'s angle is 1084948480, `0/7`'s on 1901.
    ///
    /// Made to fail by running the eject's arm on every rider: the captain
    /// holds three moves, the first member two, and both members face the
    /// boat's way.
    #[test]
    fn a_landed_squad_takes_one_group_move_each_and_its_captain_s_angle() {
        let mut f = fix();
        let mut t = f.sim.unit_types[f.citizen].clone();
        t.combat.uber_size = 3;
        let hoplite = f.sim.add_unit_type(t);
        let squad: Vec<usize> = (0..3)
            .map(|k| unit(&mut f.sim, 1, hoplite, tile_pos(28, 13 + k)))
            .collect();
        for w in squad.windows(2) {
            f.sim.units[w[0]].o_down = Some(w[1]);
            f.sim.units[w[1]].o_up = Some(w[0]);
            f.sim.units[w[1]].captain = false;
        }
        let boarded = crate::movement::Angle(1_084_948_480);
        f.sim.units[squad[0]].movement.set_facing(boarded);
        let (boat, _) = landing(&mut f, squad[0], 25);
        f.sim.units[boat].orders[0].flags |= crate::orders::flag::ACTION;
        let bearing = f.sim.units[boat].movement.heading;
        assert_ne!(bearing, boarded);
        f.sim.process_unit(boat, 1, &mut Vec::new());
        assert!(!f.sim.units[boat].alive(), "the boat stepped ashore");
        for &m in &squad {
            let u = &f.sim.units[m];
            assert!(u.on_map && u.inside_unit.is_none(), "figure {m} is out");
            assert_eq!(u.orders.len(), 1, "figure {m} holds one group move");
        }
        assert_eq!(f.sim.units[squad[0]].movement.heading, bearing);
        for &m in &squad[1..] {
            assert_eq!(
                f.sim.units[m].movement.heading, boarded,
                "member {m} faces its captain's boarding angle"
            );
        }
    }

    /// **A captain's boarding takes its whole squad** (item 1235,
    /// `docs/TRANSPORT.md` §17): `Unit::go_inside@0061a2e0` climbs `o_up`
    /// to the captain, inserts it, and calls itself down `o_down` — and
    /// each figure of a type whose `uber_size` is over 1 has its orders
    /// closed aboard. run466 block 1357: the Hoplites `0/8` and `0/9` are
    /// inside their captain's barge `0/10`, their group move gone.
    ///
    /// Made to fail by boarding the unit alone: the members stay ashore
    /// holding their moves.
    #[test]
    fn a_captain_s_boarding_takes_its_squad_aboard_and_closes_its_orders() {
        let mut f = fix();
        let mut t = f.sim.unit_types[f.citizen].clone();
        t.combat.uber_size = 3;
        let hoplite = f.sim.add_unit_type(t);
        let b = barge(&mut f.sim);
        let squad: Vec<usize> = (0..3)
            .map(|k| unit(&mut f.sim, 1, hoplite, tile_pos(28 + k, 14)))
            .collect();
        for w in squad.windows(2) {
            f.sim.units[w[0]].o_down = Some(w[1]);
            f.sim.units[w[1]].o_up = Some(w[0]);
            f.sim.units[w[1]].captain = false;
        }
        for &m in &squad {
            f.sim.add_move_order(
                m,
                tile_pos(20, 14),
                crate::orders::MoveKind::MoveTo,
                crate::orders::QueuePos::New,
                false,
            );
        }
        let boat = unit(&mut f.sim, 1, b, tile_pos(32, 14));
        // From the last figure: `param_3 == 0` climbs to the head first.
        f.sim.board(squad[2], boat);
        for &m in &squad {
            let u = &f.sim.units[m];
            assert_eq!(u.inside_unit, Some(boat), "figure {m} is aboard");
            assert!(!u.on_map, "figure {m} is off the map");
            assert!(u.orders.is_empty(), "figure {m}'s move is closed aboard");
        }
    }

    /// **The chain stops at a dead figure** (`61a497`'s `flags & 1`): what
    /// hangs below it is not walked, and stays ashore.
    #[test]
    fn a_squad_s_boarding_stops_at_a_dead_figure_down_the_chain() {
        let mut f = fix();
        let mut t = f.sim.unit_types[f.citizen].clone();
        t.combat.uber_size = 3;
        let hoplite = f.sim.add_unit_type(t);
        let b = barge(&mut f.sim);
        let squad: Vec<usize> = (0..3)
            .map(|k| unit(&mut f.sim, 1, hoplite, tile_pos(28 + k, 14)))
            .collect();
        for w in squad.windows(2) {
            f.sim.units[w[0]].o_down = Some(w[1]);
            f.sim.units[w[1]].o_up = Some(w[0]);
            f.sim.units[w[1]].captain = false;
        }
        f.sim.units[squad[1]].health = 0;
        let boat = unit(&mut f.sim, 1, b, tile_pos(32, 14));
        f.sim.board(squad[0], boat);
        assert_eq!(f.sim.units[squad[0]].inside_unit, Some(boat));
        assert_eq!(f.sim.units[squad[1]].inside_unit, None);
        assert_eq!(f.sim.units[squad[2]].inside_unit, None);
    }

    /// **The boat's number is held thirty frames** (`docs/COMBAT.md`
    /// §59.3): `Object::die(boat, 0, −1, 0)` makes no death object, but
    /// `close` ends in `Object::close`'s `hold_frames = 0x1e`, and
    /// `Objects::find_free` will not hand the number out until
    /// `process_all` has taken it to zero. East Indies' barge `1/62` died
    /// on 8195, and the original numbered its owner's next birth, on 8210,
    /// past it; this crate gave that birth the barge's number.
    ///
    /// Made to fail by leaving [`Sim::disembark`]'s boat with no hold.
    #[test]
    fn a_boat_that_puts_its_passenger_ashore_holds_its_number_thirty_frames() {
        let mut f = fix();
        let b = barge(&mut f.sim);
        let rider = unit(&mut f.sim, 1, f.citizen, tile_pos(30, 14));
        let at = Pos::new(32 * UNITS_PER_TILE + 4, tile_pos(32, 14).y);
        let boat = unit(&mut f.sim, 1, b, at);
        f.sim.units[boat].auto_transport = true;
        f.sim.init_guys(boat, Some(b));
        f.sim.board(rider, boat);
        f.sim.add_move_order(
            boat,
            tile_pos(30, 14),
            crate::orders::MoveKind::MoveTo,
            crate::orders::QueuePos::New,
            false,
        );
        // 1191's state: the body on its unit, facing its waypoint.
        let bearing = crate::movement::Angle(-1_130_299_392);
        let u = &mut f.sim.units[boat];
        u.movement.speed = 25;
        u.movement.turning.type_turn_speed = i32::MAX;
        u.movement.facing = bearing;
        u.movement.heading = bearing;
        u.movement.body.pos = at;
        u.movement.body.avg_speed = 25;
        f.sim.process_unit(boat, 1, &mut Vec::new());
        assert!(!f.sim.units[boat].alive(), "the boat stepped ashore");
        assert_eq!(f.sim.units[boat].hold_frames, Sim::CLOSE_HOLD);
        let number = f.sim.units[boat].index;
        for _ in 1..Sim::CLOSE_HOLD {
            f.sim.tick();
            assert_ne!(
                f.sim.find_free(1, crate::UNIT_BASE, crate::BUILD_BASE),
                Some(number),
                "the dead boat's number is held"
            );
        }
        f.sim.tick();
        assert_eq!(f.sim.units[boat].hold_frames, 0);
        assert_eq!(
            f.sim.find_free(1, crate::UNIT_BASE, crate::BUILD_BASE),
            Some(number),
            "and handed out once the hold is spent"
        );
    }

    /// §6.4 whole: the passenger comes out on `come_out`'s **host** ring —
    /// the boat's `block_radius` out to `+ UNIT_DISEMBARK_DISTANCE`, swept
    /// from the boat's own angle — keeps its speed, throws the scout arm's
    /// army coin, and takes the boat's order list and inverted path stack
    /// back with the top's embark flag cleared.
    ///
    /// It fails on the code this replaced at four separate lines, which is
    /// why it is one test: the passenger landed on the boat's own point,
    /// with no orders, at zero speed, and spending no draw.
    #[test]
    fn a_passenger_put_ashore_takes_the_boat_s_ring_and_the_boat_s_orders() {
        let mut f = fix();
        let b = barge(&mut f.sim);
        f.sim.unit_types[f.scout].combat.uber_size = 1;
        f.sim.unit_types[f.scout].combat.block_radius = 48;
        // `is_special` is `is(SCOUT)` folded into `unit_flags2` by the
        // loader, and it is what puts this unit on the coin's `% 2` arm.
        f.sim.unit_types[f.scout].cols.unit_flags2 |= crate::ai_load::uflags2::SCOUT;
        let rider = unit(&mut f.sim, 1, f.scout, tile_pos(30, 14));
        f.sim.units[rider].movement.speed = 34;
        let boat = unit(&mut f.sim, 1, b, tile_pos(33, 14));
        f.sim.units[boat].auto_transport = true;
        // The boat is heading due west, at the land: the sweep's first
        // bearing is that, and the ring starts at the **boat's** 48.
        let west = crate::movement::Angle::WEST;
        f.sim.units[boat].movement.set_facing(west);
        f.sim.add_move_order(
            boat,
            tile_pos(20, 14),
            crate::orders::MoveKind::MoveTo,
            crate::orders::QueuePos::New,
            false,
        );
        f.sim.units[boat].path.push(crate::orders::PathData {
            to: tile_pos(20, 14),
            tolerance: 0,
            flags: 1,
        });
        f.sim.units[boat].path.push(crate::orders::PathData {
            to: tile_pos(30, 14),
            tolerance: 0,
            flags: crate::orders::path_flag::TRANSPORT,
        });
        f.sim.units[rider]
            .movement
            .set_facing(crate::movement::Angle::INITIAL);
        let boarded = f.sim.units[rider].pos;
        f.sim.board(rider, boat);

        let before = f.sim.rng.seed;
        assert!(!f.sim.set_new_location(boat, tile_pos(30, 14), false));

        // The coin: an AI-driven scout takes the `% 2` arm and spends one
        // draw (`docs/ARMY.md` §4.1).
        assert_ne!(f.sim.rng.seed, before, "the scout arm's army coin");

        // The ring: on the boat's own heading, at least the boat's
        // `block_radius` away and no further than that plus the disembark
        // distance — never the boat's own point.
        let (at, spot) = (f.sim.units[boat].pos, f.sim.units[rider].pos);
        assert_ne!(spot, at, "not the boat's own point");
        assert!(
            (spot.y - at.y).abs() <= 48,
            "due west of it, on the boat's heading, within the quarter-tile snap"
        );
        let d = at.x - spot.x;
        assert!(
            (48..=48 + f.sim.tuning.unit_disembark_distance).contains(&d),
            "the ring is the boat's `[48, 624]`, and this is {d}"
        );

        // The speed survives, or the passenger stands there for ever.
        assert_eq!(f.sim.units[rider].movement.speed, 34);
        // And the whole order list and path came back, top flag cleared.
        assert!(f.sim.units[boat].orders.is_empty());
        assert_eq!(f.sim.units[rider].orders.len(), 1);
        assert!(f.sim.units[rider].orders[0].is_move());
        assert_eq!(
            f.sim.units[rider]
                .path
                .iter()
                .map(|p| (p.to, p.flags))
                .collect::<Vec<_>>(),
            vec![(tile_pos(20, 14), 1), (tile_pos(30, 14), 0)],
            "the boat's stack, in order, with the top's `flags & 4` gone"
        );
        assert!(!f.sim.units[boat].alive());
        // `update_action` is step 1's (item 803) and, for a computer
        // player's unit, `come_out`'s tail's again from the spot (item
        // 1164): `orders_x/y` are the landing point, not the boarding one,
        // and `dest_angle` the heading it boarded on while `come_out` snaps
        // its facing west.
        assert_ne!(spot, boarded);
        assert_eq!(f.sim.units[rider].orders_pos, spot);
        assert_eq!(f.sim.units[rider].movement.heading, west);
        assert_eq!(
            f.sim.units[rider].movement.des_angle,
            crate::movement::Angle::INITIAL
        );
    }

    /// A passenger put ashore on `come_out`'s ring, for a human or a
    /// computer player: the rider's `orders_x/y` and its guys' speeds.
    fn ashore(human: bool, avg: i32) -> (Fix, usize, Pos) {
        let mut f = fix();
        f.sim.nation[1].human = human;
        let b = barge(&mut f.sim);
        f.sim.unit_types[f.scout].combat.uber_size = 1;
        f.sim.unit_types[f.scout].combat.block_radius = 48;
        let rider = unit(&mut f.sim, 1, f.scout, tile_pos(30, 14));
        f.sim.units[rider].movement.speed = 34;
        f.sim.units[rider].guys = vec![crate::anim::Guy::fresh(1), crate::anim::Guy::fresh(2)];
        f.sim.art.tracks.insert(2, (-48, -192));
        f.sim.seat_guys(rider);
        f.sim.units[rider].movement.body.avg_speed = avg;
        f.sim.units[rider].guys[1]
            .follow
            .as_mut()
            .unwrap()
            .body
            .avg_speed = avg;
        let boat = unit(&mut f.sim, 1, b, tile_pos(33, 14));
        f.sim.units[boat].auto_transport = true;
        f.sim.units[boat]
            .movement
            .set_facing(crate::movement::Angle::WEST);
        let boarded = f.sim.units[rider].pos;
        f.sim.board(rider, boat);
        assert!(!f.sim.set_new_location(boat, tile_pos(30, 14), false));
        assert_eq!(f.sim.units[rider].inside_unit, None, "ashore");
        (f, rider, boarded)
    }

    /// **`come_out`'s tail, for a computer player's unit** (item 1164,
    /// §6.4): past `set_new_location`, `6187c8` skips a human's unit and
    /// `618813`..`618836` run `update_action` on everyone else's from the
    /// spot. run420's AI merchant `1/33` prints its landing point
    /// (38952, 24840) as `orders_x/y` on 6735; a human's keeps the point it
    /// boarded at, step 1's (run249's `0/6`, item 803).
    #[test]
    fn a_computer_player_s_passenger_is_ordered_where_it_lands_and_a_human_s_where_it_boarded() {
        let (ai, r, boarded) = ashore(false, 0);
        let spot = ai.sim.units[r].pos;
        assert_ne!(spot, boarded);
        assert_eq!(
            ai.sim.units[r].orders_pos, spot,
            "the AI's: the landing point"
        );
        let (me, r, boarded) = ashore(true, 0);
        assert_eq!(
            me.sim.units[r].orders_pos, boarded,
            "a human's: the boarding point"
        );
    }

    /// **A passenger comes ashore at the average it boarded with** (item
    /// 1164, §6.4): `Guy::last_speed`/`avg_speed` (`+0x80`/`+0x84`) are
    /// written by `Guy::move`, `Guy::clear` and `Guy::init_real` alone,
    /// and neither `set_new_location` is one. So the first turn ashore is
    /// [`crate::movement::turn_speed`]'s divided one: run420's `1/33` lands
    /// at 12 and turns a quarter, a third, a half … of its rate, facing its
    /// path on 6745 where a zeroed average faced it on 6743.
    #[test]
    fn a_passenger_comes_ashore_at_the_average_it_boarded_with() {
        let (f, r, _) = ashore(false, 12);
        let u = &f.sim.units[r];
        assert_eq!(u.movement.body.avg_speed, 12, "guy 0's body");
        assert_eq!(
            u.guys[1].follow.map(|g| g.body.avg_speed),
            Some(12),
            "and the tracked crew guy's, seated on its offset again"
        );
        assert_eq!(
            u.guys[1].follow.map(|g| g.body.pos),
            u.guys[1].follow.map(|g| g.des),
            "seated on its offset"
        );
    }

    /// §6.4's way in, and `docs/ORDERS.md` §4.4's region check (item 1143):
    /// a barge that **takes** a final waypoint on land — another tile
    /// region than its own — re-pushes it with `flags | 4` and tolerance 0,
    /// so `find_path`'s `invalid_loc` takes the land tile and the barge
    /// keeps its goal. Without the check the pull-back walks the goal back
    /// to the water and the barge stops a tile short of the shore with its
    /// passenger aboard, East Indies' `1/42` on 6321.
    #[test]
    fn a_barge_taking_a_goal_ashore_flags_it_and_keeps_it() {
        let mut f = fix();
        let b = barge(&mut f.sim);
        f.sim.unit_types[b].kind.domain = Domain::Sea;
        let rider = unit(&mut f.sim, 1, f.citizen, tile_pos(40, 14));
        let boat = unit(&mut f.sim, 1, b, tile_pos(40, 14));
        f.sim.units[boat].auto_transport = true;
        f.sim.init_guys(boat, Some(b));
        f.sim.board(rider, boat);
        let goal = tile_pos(28, 14);
        f.sim.add_move_order(
            boat,
            goal,
            crate::orders::MoveKind::MoveTo,
            crate::orders::QueuePos::New,
            false,
        );
        f.sim.units[boat].orders[0].flags |= crate::orders::flag::PATHED;
        f.sim.units[boat].path.push(crate::orders::PathData {
            to: goal,
            tolerance: 96,
            flags: crate::orders::path_flag::FINAL,
        });
        assert_ne!(
            f.sim.world.tregion_alt(goal.tile()),
            f.sim.world.tregion_alt(f.sim.units[boat].pos.tile()),
            "the goal is in another tile region"
        );
        f.sim.work(boat, 1);
        assert_eq!(
            f.sim.units[boat]
                .path
                .last()
                .map(|p| (p.to, p.tolerance, p.flags)),
            Some((goal, 0, 1 | crate::orders::path_flag::TRANSPORT)),
            "the final leg is flagged, and its point is the land tile still"
        );
        assert_eq!(
            f.sim.current_move(boat).map(|m| m.waypoint),
            Some(goal),
            "and the barge walks at it"
        );
    }

    /// §7: the island search picks a coastal cell of another region that
    /// a sea region coasts with mine, and issues the move.
    #[test]
    fn the_island_search_sends_a_scout_to_another_region_s_shore() {
        let mut f = fix();
        barge(&mut f.sim);
        // A second island on the far side of the sea, flagged as a
        // resource region so `go_here` answers bit 1.
        let far = f
            .sim
            .world
            .fill_region(Terrain::Land, Cell::new(11, 0), Cell::new(11, 7));
        for y in 0..8 {
            let c = Cell::new(11, y);
            let mut d = f.sim.world.cell_data(c);
            d.land = 0;
            d.flags |= 0x100;
            d.region2 = Some(f.sea);
            f.sim.world.set_cell_data(c, d);
            for tx in 44..48 {
                for ty in y * 4..y * 4 + 4 {
                    f.sim
                        .world
                        .set_tile_field(Pos::new(tx, ty), tile::SURFACE, 0);
                }
            }
        }
        f.sim.world.rebuild_coasts();
        f.sim.world.set_region_flags(far, 8);
        let u = unit(&mut f.sim, 1, f.scout, tile_pos(10, 14));
        f.sim.units[u].auto_transport = true;
        f.sim.units[u].type_index = ty::SCOUT as i32;
        f.sim.transport.resize_with(2, LeaderTransport::default);
        f.sim.transport[1].set_all();
        assert!(
            f.sim.think_civilian_transport(u, false),
            "a resource region across the water, and nothing scouted"
        );
        let dest = f.sim.units[u].orders.front().and_then(|o| o.move_dest());
        let cell = dest.expect("a move order").cell();
        assert_eq!(cell.x, 11, "the far island's own column");
        // And once its own region is marked, the same region is still
        // fair game — the mark is on *mine*, not on the candidate.
        f.sim.world.mark_region_scouted(far, 1);
        f.sim.units[u].orders.clear();
        assert!(
            !f.sim.think_civilian_transport(u, false),
            "a region this leader has scouted is not a candidate"
        );
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

    /// **`snap_center`'s dock arm** (item 803, `docs/GOLDEN.md` §28): a
    /// Dock whose centred tile `blocked_site` refuses takes the first tile
    /// of the `move_x/move_y` spiral that it clears, in the spiral's own
    /// order; a clear tile, or one with nothing clear within four, stays;
    /// and nobody's shore clears nothing.
    #[test]
    fn a_dock_asked_off_its_shore_moves_along_the_spiral() {
        let mut f = fix();
        let placed = |f: &Fix, tx: i32| {
            f.sim
                .snap_center_placed(f.dock, tile_pos(tx, 15), Some(0))
                .tile()
        };
        // Unowned ground: every site is refused, so every Dock stays.
        assert_eq!(placed(&f, 32), Pos::new(32, 15));
        for y in 0..8 {
            for x in 0..12 {
                f.sim.world.set_owner(
                    Cell::new(x, y),
                    crate::world::Owner::Player(0),
                    crate::world::Owner::None,
                );
            }
        }
        // Clear where asked.
        assert_eq!(
            f.sim.blocked_site(Some(0), f.dock, tile_pos(33, 15), None),
            crate::place::Blocked::Clear
        );
        assert_eq!(placed(&f, 33), Pos::new(33, 15));
        // Too little water: north-east, index 3, before east, index 4.
        assert_eq!(placed(&f, 32), Pos::new(33, 14));
        // Too much: west and south, index 18, before west and north.
        assert_eq!(placed(&f, 36), Pos::new(34, 16));
        // Four tiles out, the radius-4 ring's index 57.
        assert_eq!(placed(&f, 29), Pos::new(33, 11));
        // Five tiles out: nothing in reach, so it stays.
        assert_eq!(placed(&f, 28), Pos::new(28, 15));
        // The plain snap never moves.
        assert_eq!(
            f.sim.snap_center(f.dock, tile_pos(32, 15)).tile(),
            Pos::new(32, 15)
        );
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
