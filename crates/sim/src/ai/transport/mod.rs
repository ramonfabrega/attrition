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

use crate::ai::ai_place::{MOVE_X, MOVE_Y};
use crate::cities::build;
use crate::combat::Obj;
use crate::orders::Worker;
use crate::tech::TypeId;
use crate::territory::attrition::Domain;
use crate::world::{Cell, Pos, TILES_PER_CELL, UNITS_PER_TILE, tile};
use crate::{BUILD_BASE, Player, Sim, UNIT_BASE, Unit};

pub use crate::ids::ty;

/// The bias angle `Unit::do_cast` and `SpellType::cast_transport` both hand
/// `find_nearby_spot` — `0x55555555`, a third of a turn, which
/// `docs/ORDERS.md` §10 records as arbitrary rather than a sentinel.
pub(crate) const BOARD_BEARING: crate::movement::Angle = crate::movement::Angle(0x5555_5555);

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
pub const NEEDS_DISEMBARK: i32 = 1;
pub const NEEDS_EMBARK: i32 = 2;

/// `orthog_x[1..=4]`, `orthog_y[1..=4]` — north, east, south, west
/// (`.rdata 0x00add250` / `0x00add210`).
const ORTHOG: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

/// `UnitData::can_ever_transport@0046f290` (§3.3) in the original's order: a land unit answers yes unasked;
/// otherwise the vslot `+0x18` (`slot18`, the folded `return 1`), then the carry of the unit's type, then
/// `is(AIRCRAFTCARRIER)` — each asked only when the one before has not decided, and the answer is no carrier.
/// [`Sim::unit_can_ever_transport`] is this with the sim's reads; item 2112's sweep holds the order and the asks.
pub fn can_ever_transport_by(
    land: bool,
    slot18: impl FnOnce() -> bool,
    carry: impl FnOnce() -> bool,
    is_carrier: impl FnOnce() -> bool,
) -> bool {
    land || (slot18() && carry() && !is_carrier())
}

impl Sim {
    // ------------------------------------------------------------------
    // §2–§3: the predicates
    // ------------------------------------------------------------------

    /// `LeaderData::can_transport`.
    pub fn transport_level(&self, who: Player) -> TransportType {
        self.ai
            .transport
            .get(who as usize)
            .map_or(TransportType::None, LeaderTransport::level)
    }

    /// `ObjectData::is(t, 0)` on a unit's type — the lineage test.
    pub(crate) fn unit_is(&self, u: usize, t: TypeId) -> bool {
        self.units[u]
            .ty
            .and_then(|i| self.tables.unit_types[i].tree)
            .is_some_and(|ut| {
                ut == t
                    || (ut < self.tables.tech_tree.types.len()
                        && self.tables.tech_tree.is(ut, t, false))
            })
    }

    /// The unit type's domain; a typeless unit is what its kind says.
    pub(crate) fn unit_domain_of(&self, u: usize) -> Domain {
        self.units[u].ty.map_or(self.units[u].kind.domain, |t| {
            self.tables.unit_types[t].combat.domain
        })
    }

    /// `UnitData::transport_type@0046f790` (§3.1).
    pub fn unit_transport_type(&self, u: usize) -> TransportType {
        if self.unit_is(u, ty::SCOUT) {
            return TransportType::Scout;
        }
        let unit = &self.units[u];
        let worker = unit
            .ty
            .map_or(Worker::None, |t| self.tables.unit_types[t].worker);
        if worker != Worker::None {
            return TransportType::Civilian;
        }
        let base = unit.ty.and_then(|t| self.tables.unit_types[t].tree);
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
            .is_some_and(|t| self.tables.unit_types[t].cols.unit_flags & 0x10 != 0);
        (unit.auto_transport && !unit.never_transport) || always
    }

    /// `UnitData::can_ever_transport@0046f290` (§3.3): a land unit, or a
    /// carrier (`carry != 0`) that is not an aircraft carrier.
    pub fn unit_can_ever_transport(&self, u: usize) -> bool {
        can_ever_transport_by(
            self.unit_domain_of(u) == Domain::Land,
            || true,
            || {
                self.units[u]
                    .ty
                    .is_some_and(|t| self.tables.unit_types[t].cols.carry != 0)
            },
            || self.unit_is(u, ty::AIRCRAFTCARRIER),
        )
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
            NEEDS_DISEMBARK
        } else {
            NEEDS_EMBARK
        }
    }

    // ------------------------------------------------------------------
    // §4: the level, maintained
    // ------------------------------------------------------------------

    /// `has_preq(TRANSPORT_BONUS)`: the bonus row's one prerequisite — the
    /// tree names it in `roles.transport_preq`; a tree that does not know it
    /// grants it, the way every type outside the tree is ungated.
    fn transport_preq_held(&self, who: Player) -> bool {
        match self.tables.tech_tree.roles.transport_preq {
            Some(t) => {
                self.tables
                    .tech_tree
                    .has_tech(&self.tables.setup, &self.tech[who as usize], t)
            }
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
                    && b.ty
                        .is_some_and(|t| build::is_dock(&self.tables.build_types, t))
            })
            .count() as i32
    }

    /// `Leader::check_transport@006bc5f0` (§4). Run when a dock is finished
    /// or lost and after every technology gained.
    pub fn check_transport(&mut self, who: Player) {
        let w = who as usize;
        if w >= self.ai.transport.len() || self.ai.transport[w].locked {
            return;
        }
        let want = if self.transport_preq_held(who) {
            TransportType::Civilian
        } else {
            TransportType::None
        };
        if want != TransportType::None && self.dock_count(who) != 0 {
            if self.ai.transport[w].level() == want {
                return;
            }
            self.ai.transport[w].set_all();
            let option = self.ai.transport[w].scouts_option;
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
        if self.ai.transport[w].level() != TransportType::None {
            self.ai.transport[w].clear();
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
        let Some(lt) = self.ai.transport.get(who as usize).copied() else {
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
        if self.ai.docks.len() <= w {
            self.ai.docks.resize_with(w + 1, Docks::default);
        }
        let slot = {
            let docks = &mut self.ai.docks[w];
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
            && w < self.ai.ai.len()
        {
            let regs = self.world.region_count();
            let cs = &mut self.ai.ai[w].census;
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
            .tables
            .unit_types
            .iter()
            .position(|t| t.tree == Some(ty::GULLBIRD));
        let mut gull = None;
        if let Some(gt) = gull_ty
            && let Some(index) = self.find_free(9, UNIT_BASE, BUILD_BASE)
        {
            // `Unit::init@00612100`'s snap onto the centre of the 48-unit
            // tile (`crate::gaia::init_snap`), as a wild bird's birth takes
            // it (item 1453).
            let at = Pos::new(
                crate::gaia::init_snap(pos.x - GULL_OFFSET),
                crate::gaia::init_snap(pos.y - GULL_OFFSET),
            );
            let mut unit = Unit::new(9, index, at, self.tables.unit_types[gt].hits);
            unit.kind = self.tables.unit_types[gt].kind;
            unit.ty = Some(gt);
            // `TypeIndex::GULLBIRD`. `Guy::set_anim` names the three gaia
            // bird types by identity in its walk arm (`set_anim:620`), so a
            // gull that carried the default −1 could never throw the wing
            // beat's coin — the pasture's own bug, one type over
            // (`docs/SYNC.md` §3.11).
            unit.type_index = self.tables.unit_types[gt].type_index;
            unit.movement.speed = self.type_speed(9, gt);
            unit.movement.turning = self.turning_for(gt);
            // `Unit::init@00612100:282–309` (`crate::units::stance`).
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
        self.ai.docks[w].slots[slot] = DockSlot {
            building: Some(b),
            reg,
            gull,
        };
        slot
    }

    /// The position of the live dock whose slot holds gull `u` — the
    /// target of the gull's `StrafeOrder` (item 1453).
    pub fn gull_dock(&self, u: usize) -> Option<Pos> {
        self.ai
            .docks
            .iter()
            .flat_map(|d| d.slots.iter())
            .find_map(|slot| {
                let b = slot.building?;
                (slot.gull == Some(u) && self.buildings[b].alive).then_some(self.buildings[b].pos)
            })
    }

    /// `Docks::close_dock` + `Dock::close` (§5.3), before the building is
    /// marked dead: the region's count comes back down, the gull is closed,
    /// and the mark walks down past trailing free slots.
    pub(crate) fn dock_close(&mut self, b: usize) {
        let Some(slot) = self.buildings[b].dock_slot.take() else {
            return;
        };
        let w = self.buildings[b].owner as usize;
        let Some(docks) = self.ai.docks.get_mut(w) else {
            return;
        };
        let Some(rec) = docks.slots.get_mut(slot).copied() else {
            return;
        };
        if rec.building == Some(b)
            && self.buildings[b].alive
            && let Some(r) = rec.reg
            && self.world.terrain(r) == crate::world::Terrain::Land
            && w < self.ai.ai.len()
            && let Some(n) = self.ai.ai[w].census.reg_docks.get_mut(r as usize)
        {
            *n -= 1;
        }
        // `Dock::close` clears `o`, `reg`, `who` and the active bit and
        // leaves `gull_o` as it was (audit B.13) — a dump of a closed slot
        // shows the stale gull.
        self.ai.docks[w].slots[slot] = DockSlot {
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
        let docks = &mut self.ai.docks[w];
        while docks.mark > 0 && !docks.slots[docks.mark - 1].active() {
            docks.mark -= 1;
        }
    }

    // ------------------------------------------------------------------
    // §7: the civilian's island
    // ------------------------------------------------------------------

    /// `ObjectData::is_cargo@00653600`: an on-map unit whose type is a unit
    /// type in `0x32..=0x19d` of the **land** domain. What can ride a boat.
    pub(crate) fn is_cargo(&self, u: usize) -> bool {
        self.units[u].on_map
            && cargo_type(
                self.units[u].type_index,
                self.unit_domain_of(u) == Domain::Land,
            )
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
    /// SEAMS: an exploring unit (bit `0x100` of its mask) is clear on every
    /// unit in every capture, so the three arms that re-enter the scout
    /// think are unreachable and the order is always `MOVE_TO`; the
    /// danger grid has no writer, so its half-size read answers 0 and
    /// every score is its distance.
    ///
    /// The document says the region loop skips the unit's own region. It
    /// does not — `005f4283` gates on `Region.size` alone — and what keeps
    /// a scout from choosing home is the scouted bit its own tail has
    /// just set. Amended in §7.
    pub(crate) fn think_civilian_transport(&mut self, u: usize, colonise: bool) -> bool {
        let mut v = crate::units::think::civ_transport::SimCivTransport::new(self, u);
        crate::units::think::civ_transport::think_civilian_transport_steps(&mut v, colonise) != 0
    }

    /// `Unit::think_carry@005f2ee0`'s dispatch (`docs/AI.md` §195), reached from
    /// `Unit::think@005f6e40`'s tail for a type with a non-zero `CARRY` that
    /// is not strictly an aircraft carrier. `true` ends the think.
    ///
    /// A boat with a **worker aboard** asks [`Sim::think_carry_civilian`]
    /// with 1; one with a unit of `is(0x45)` aboard asks it with 0.
    /// `count_inside` opens on `is_on_map` of the carrier itself.
    ///
    /// A boat with a **captain that attacks** aboard
    /// (`count_inside(COUNT_ATTACK)`) takes the attack-carrier's arm,
    /// [`Sim::think_carry_army`] (`docs/AI.md` §235).
    ///
    /// The arm with nothing of the three aboard is `think_carry_idle`
    /// (`005f2f30`, [`Sim::think_carry_idle`]), the walk to the nearest
    /// coastal city with a dock (item 3995).
    pub(crate) fn think_carry(&mut self, u: usize) -> bool {
        // `count_inside` opens on `is_on_map` of the carrier itself.
        let riders: Vec<usize> = if self.units[u].on_map {
            (0..self.units.len())
                .filter(|&i| self.units[i].alive() && self.units[i].inside_unit == Some(u))
                .collect()
        } else {
            Vec::new()
        };
        let mut v = SimCarry {
            sim: self,
            u,
            riders,
        };
        think_carry_steps(&mut v) != 0
    }

    /// `Unit::think_carry@005f2ee0`'s attack arm (`docs/AI.md` §235): a boat
    /// carrying a soldier sails it to **its army's target**.
    ///
    /// The group is the boat's `inside_down`'s (`+0x28`, then that unit's
    /// `+0x80`); no group answers `false` (`return 0`). The army is the
    /// group's `+8`, read straight off the group as the original does, not
    /// through `get_army`'s membership test. With an army:
    ///
    /// 1. any of the boat's nine cells (its own and its eight neighbours,
    ///    `move_x`/`move_y`) in the army's region (`ArmyData +0x8`) —
    ///    `eject_contents(0, -1, 0, 1)` and `return 1`, or the boat's
    ///    death when that left nothing inside (item 3640, `docs/AI.md`
    ///    §335: run1301's Transport Galleon `1/68` puts the Arquebusiers
    ///    `1/48`–`1/50` ashore on 2163 and is gone);
    /// 2. otherwise a **live** target (`+0x30`, `+0x34`; the object's
    ///    `+8 & 1`) — a `MOVE_TO` to the centre of the target's cell, the
    ///    merchant's inlined `add_move_order(…, QUEUE_NEW, 0)`, snapped to
    ///    48: run879's `1/225` holds (6552, 7320) for the human's `0/2000`
    ///    at (6240, 7008), `off_x/off_y` 408.
    ///
    /// 3. **no army** (item 3995, `docs/GOLDEN.md` §188): the riders are
    ///    ejected — at sea `come_out` refuses and they stay — and an
    ///    emptied boat dies; a boat still carrying sails for **its owner's
    ///    nearest live city** (a quarter of the distance for a city under
    ///    attack), the centre of the city building's cell: run1553's barge
    ///    `1/10` takes (38040, 16536) for who=1's added city on its birth
    ///    frame.
    ///
    /// The steps are [`crate::units::think::carry_mil`]'s, swept against the
    /// original (item 3411). SEAM, answering `false`: the army with no live
    /// target away from its region (its region's centre, or a city of the
    /// region its leader marks `& 4`). The rider chain's head is taken as
    /// the lowest-numbered rider; a boat carries one squad here.
    pub(crate) fn think_carry_army(&mut self, u: usize, head: usize) -> bool {
        let who = self.units[u].owner;
        let Some(s) = self.units[head].group_ptr else {
            return false;
        };
        let army = self.ai.armies.get(who as usize).and_then(|x| {
            x.list
                .iter()
                .position(|a| a.valid && a.group.pool == Some(s))
        });
        // SEAM: an army with no live target walks to its region's centre
        // (`Region +0x7c`) or a city of a region its leader marks `& 4`;
        // this crate carries no region centre, and answers `false` there.
        if let Some(a) = army
            && !self.ai.armies[who as usize].list[a]
                .target
                .is_some_and(|t| self.obj_alive(t))
        {
            let reg = self.ai.armies[who as usize].list[a].reg;
            if !self.carry_ashore(u, reg) {
                return false;
            }
        }
        let cities = (0..self.cities.cities.len())
            .filter(|&c| self.cities.cities[c].alive && self.cities.cities[c].owner == who)
            .collect();
        let mut v = crate::units::think::carry_mil::SimCarryMil {
            sim: self,
            u,
            group: usize::from(s),
            army,
            cities,
        };
        crate::units::think::carry_mil::think_carry_military_steps(&mut v) != 0
    }

    /// Whether any of the boat's nine cells (its own and its eight
    /// neighbours) lies in `reg` — `think_carry_military`'s ashore test.
    fn carry_ashore(&self, u: usize, reg: Option<u16>) -> bool {
        let my_cell = self.units[u].pos.cell();
        let (width, height) = (self.world.width(), self.world.height());
        (0..9).any(|k| {
            let c = Cell::new(my_cell.x + MOVE_X[k], my_cell.y + MOVE_Y[k]);
            c.x >= 0
                && c.y >= 0
                && c.x < width
                && c.y < height
                && self.world.region_of(c).map_or(-1, i32::from) == reg.map_or(-1, i32::from)
        })
    }

    /// `Unit::think_carry_civilian(workers)@005f37c0` (`docs/AI.md` §195): a
    /// boat with its cargo aboard looks for the **shore of a land region**
    /// it can put them on.
    ///
    /// The search is [`Sim::think_civilian_transport`]'s with three
    /// differences: the region need only coast **the boat's own region**
    /// (`Region::is_coast(r, my)`, where that one finds a sea first);
    /// the cell is any `coast_here(r, my)` cell, water-half or not; and
    /// the score is `vector_dist × max(1, danger)`, then — for `workers ==
    /// false` only — `/ 3` when the region is `go_here` bit 1 and `/ 2` for
    /// bit 2. A worker boat takes bit 1 alone and ignores `scouted`; the
    /// other takes any bit and skips a region its leader has scouted.
    ///
    /// **The boat's own region is `get_tregion`'s** (`local_64`), the
    /// coastal-refined [`crate::world::World::tregion_alt`]: a boat on the
    /// ocean tile of a coastal land cell is in the cell's sea `region2`,
    /// not its land region (`docs/AI.md` §274, run970e's `1/77`).
    ///
    /// Then three outcomes:
    /// 1. **no region** — `false` for workers (`think_carry_military` is
    ///    the other's, a SEAM);
    /// 2. **none of the boat's nine cells (itself and its eight
    ///    neighbours) is in the chosen region** — a `MOVE_TO` to the chosen
    ///    cell **plus `MOVE[d]`**, where `d` is **the last `coast_here`
    ///    answer the whole search made**, whichever region it was in
    ///    (`move_x[local_48]` is the unit's local and nothing resets it);
    /// 3. **the boat is on the shore** — the sixteen quarter-cells of its
    ///    own cell are scored against the first neighbour found in the
    ///    region, and the nearest by `vector_dist` is walked to — or, when
    ///    the boat already stands in it, `Group::action_eject_all`
    ///    (SEAM: stated here and not modelled — the boat answers `true`
    ///    and keeps its cargo).
    ///
    /// The `MOVE_TO` is the merchant's inlined `add_move_order(…,
    /// QUEUE_NEW, 0)`: the bearing is taken to the unsnapped centre, the
    /// destination snapped to 48.
    pub(crate) fn think_carry_civilian(&mut self, u: usize, workers: bool) -> bool {
        let mut v = crate::units::think::carry_civ::SimCarryCiv::new(self, u);
        crate::units::think::carry_civ::think_carry_civilian_steps(&mut v, workers)
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
    /// The captain check between 4 and 5 is in as of item 2077: for
    /// `0x28a` alone, a figure that is not a captain and whose
    /// `get_captain`'s current order is `CAST_SPELL` takes `spell_time − 1`
    /// and returns (`005ebfe0`, after the clock). The captain's cast boards
    /// its whole squad ([`Sim::cast_transport`]'s `board`), so it is a
    /// member holding a transport cast of its own, stepped **after** a
    /// captain that took one, that reaches it (East Indies' `1/24` on
    /// 3835, `docs/AI.md` §207).
    ///
    /// The Monarch's `has_general(0, 0x162)` extra `spell_time` step, for a
    /// siege unit's pack or unpack, is **in** as of item 3390
    /// ([`Sim::near_monarch`]).
    ///
    /// SEAMS, all stated: the whole non-spell-type arm, which is
    /// `LeaderData::current_upgrade` + `set_type` and reaches no craft
    /// index at all. `is_rare_collector`'s re-seat is **in** as of item
    /// 210 — step 2's second half.
    pub(crate) fn do_cast(&mut self, u: usize, order: crate::orders::CastOrder) {
        let s = order.spell;
        if self.spell(s).is_some_and(|d| d.targeted()) {
            // The targeted half, `crate::spells::cast` (item 790).
            self.do_cast_targeted(u, order);
            return;
        }
        // The whole untargeted half, `crate::spells::do_cast_view`: the craft the rest
        // acts on, the once-only cost (`pay_cast_costs@00676c40`: a unit
        // caster's `mana_burn` takes the craft's `MANA` — Create Decoys'
        // 1,000 on the General, `docs/GOLDEN.md` §48; the pack, unpack and
        // transport crafts ask none), the pack, unpack, hero and boat
        // animations (`docs/AI.md` §137, §285), the clock, the member's wait
        // on its captain's transport (item 2077, `docs/AI.md` §207: East
        // Indies' `1/24` on 3835) and the cast itself.
        let mut v = crate::spells::do_cast_view::SimDoCast {
            sim: &mut *self,
            u,
            order,
        };
        let _ = crate::spells::do_cast_view::do_cast_clock_steps(&mut v);
    }

    /// `SpellType::cast@00676ce0`'s untargeted arms this crate carries, behind
    /// an `is_castable` that has to answer **3**; the caller has cleared the
    /// clock and clears `paid` and kills the order after (`005ece96`).
    pub(crate) fn do_cast_effect(&mut self, u: usize, s: i32) {
        use crate::orders::spell;
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
            return;
        }
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
        // §99.13), and its Ambush (`cast_ambush`, `docs/AI.md` §261).
        // SEAM: Rally's `cast_rally@00671600` is an empty body.
        if matches!(
            s,
            spell::TO_ARMS
                | spell::CAST_CIVILIAN
                | spell::CREATE_DECOY
                | spell::FORCED_MARCH
                | spell::AMBUSH
                | spell::ENTRENCH
        ) && self.spell_castable(s, u)
        {
            match s {
                spell::TO_ARMS => self.cast_to_arms(u),
                spell::CAST_CIVILIAN => self.cast_civilian(u),
                spell::FORCED_MARCH => self.cast_march(u),
                spell::AMBUSH => self.cast_ambush(u),
                // …and its Entrench (`cast_entrench`, item 3919).
                spell::ENTRENCH => self.cast_entrench(u),
                _ => self.cast_create_decoy(u),
            }
        }
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
    /// interface and no record here. ~~The `set_new_location` at the
    /// tail, which re-seats the unit on its own position~~: built by item
    /// 1608.
    pub(crate) fn cast_unpack(&mut self, u: usize) {
        let tree = self.unit_tree(u);
        let usable = self.units[u].alive() && self.units[u].on_map;
        cast_unpack_steps(&mut SimUnpack { sim: self, u }, usable, tree);
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
    pub(crate) fn transport_type_for(&self, u: usize) -> Option<usize> {
        let base = if self.unit_is(u, ty::CARAVAN) {
            ty::MERCHANTFLEET
        } else {
            ty::TRANSPORTBARGE
        };
        self.upgraded_unit_type(self.units[u].owner, base)
    }

    /// `LeaderData::current_upgrade(TRANSPORTBARGE)` as a unit type: the
    /// leader's barge, or the base barge where the tree has no upgrade.
    pub(crate) fn barge_type(&self, who: Player) -> Option<usize> {
        self.upgraded_unit_type(who, ty::TRANSPORTBARGE)
    }

    /// `LeaderData::current_upgrade(base)` as an index into the unit types.
    fn upgraded_unit_type(&self, who: Player, base: TypeId) -> Option<usize> {
        let who = who as usize;
        // A tree that does not carry the base type — the harness's small
        // fixtures — has no upgrade to offer, and the base is the answer.
        let id = match self.tech.get(who) {
            Some(p) if base < self.tables.tech_tree.types.len() => self
                .tables
                .tech_tree
                .current_upgrade(&self.tables.setup, p, base),
            _ => base,
        };
        self.tables
            .unit_types
            .iter()
            .position(|t| t.tree == Some(id))
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
    /// The `MARINES` arm is built (item 3661): `unit_masks2 & 0x200`,
    /// `update_hits` and `update_speed`, in `SimTransport::marines`.
    /// SEAMS, all stated: the console sound, the selection group's swap and
    /// `replace_hotunit` — none of them is state this crate keeps. ~~The
    /// caravan's slot hand-over~~ is built (item 3370): the fleet takes
    /// [`Unit::caravan`] and [`Sim::disembark`] gives it back.
    pub(crate) fn cast_transport(&mut self, u: usize) {
        // The body is `crate::spells::cast_transport::cast_transport_steps` (item
        // 3330), the original's control flow over a view; the Sim answers it.
        let who = self.units[u].owner;
        let mut view = crate::spells::cast_transport::SimTransport::new(self, who as usize);
        crate::spells::cast_transport::cast_transport_steps(&mut view, u as i32, who as i32);
    }

    /// `LeaderData::current_upgrade(base)` of `who`, as a tree id.
    pub(crate) fn current_upgrade_id(&self, who: usize, base: TypeId) -> TypeId {
        // A tree that does not carry the base type — the harness's small
        // fixtures — has no upgrade to offer, and the base is the answer.
        match self.tech.get(who) {
            Some(p) if base < self.tables.tech_tree.types.len() => self
                .tables
                .tech_tree
                .current_upgrade(&self.tables.setup, p, base),
            _ => base,
        }
    }

    /// `Objects::init_unit(who, ty, at, -1, -1, -1)` for a boat of the unit
    /// type `ty` born at `at`: its index, or none when the list is full.
    pub(crate) fn transport_boat(&mut self, who: Player, ty: usize, at: Pos) -> Option<usize> {
        let index = self.find_free(who, UNIT_BASE, BUILD_BASE)?;
        let mut boat = Unit::new(who, index, at, self.tables.unit_types[ty].hits);
        boat.kind = self.tables.unit_types[ty].kind;
        boat.ty = Some(ty);
        boat.type_index = self.tables.unit_types[ty].type_index;
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
        // `Unit::init@00612100:282–309` (`crate::units::stance`).
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
        Some(b)
    }

    /// `Unit::same_damage(o, who)@005f9400`: the boat comes out of the
    /// conversion as damaged, in 256ths of its own hit points, as the unit
    /// that cast it.
    ///
    /// SEAM: the original walks both squads and spreads the damage figure by
    /// figure, killing whole figures off a squad too hurt to carry it. Every
    /// unit that boards in a capture so far is a single figure at full
    /// health, where the walk and this line agree on zero.
    pub(crate) fn same_damage(&mut self, boat: usize, u: usize) {
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
    pub(crate) fn board(&mut self, u: usize, boat: usize) {
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
                .is_some_and(|t| self.tables.unit_types[t].combat.uber_size > 1)
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
        self.eject_riders(boat, true);
        self.transport_dies(boat);
    }

    /// **`Group::action_eject_all(0, −1, −1, −1)@00710b40` on a one-boat
    /// group** — `think_carry_civilian`'s eject, when the boat stands in the
    /// quarter-cell nearest its shore (item 3521, `docs/AI.md` §303). With
    /// `param_2 < 0` the walk takes each member (the boat alone here):
    /// alive (`+8 & 1`), `num_inside(1) > 0` and not
    /// `can_carry(AIR)@00646c40` — `is(0x1bf, 1)`, `is(0x15f, 1)` or
    /// `is(0x208, 1)` — gives `eject_contents(0, −1, 0, 1)`: the
    /// [`Sim::disembark`] steps with **`param_3 = 0`**, so the passenger
    /// keeps its own empty list rather than the boat's. Then a boat that
    /// `is(TRANSPORTBARGE)` or `is(MERCHANTFLEET)` with nothing left inside
    /// (`num_inside(0) == 0`) dies, `Object::die(boat, 0, −1, 0)`.
    ///
    /// run1153c: who=1's Freighter `1/121` (`TRANSPORTFREIGHTER`, 322) stands
    /// at (37272, 26520) on 3673; on 3674 the Citizen `1/10` is ashore at
    /// (37416, 26280) with its boarding point for `orders_x/y`, and `1/121`
    /// is gone.
    ///
    /// The buildings group's arm, from the console's command, is
    /// [`Sim::action_eject_all`].
    pub(crate) fn boat_eject_all(&mut self, boat: usize) {
        if !self.units[boat].alive() || !self.units[boat].on_map {
            return;
        }
        if !(0..self.units.len()).any(|i| self.units[i].inside_unit == Some(boat)) {
            return;
        }
        let air = self.unit_tree(boat).is_some_and(|ut| {
            [0x1bf, 0x15f, 0x208]
                .into_iter()
                .any(|t| self.tables.tech_tree.is(ut, t, true))
        });
        if air {
            return;
        }
        self.eject_riders(boat, false);
        let barge = self.unit_line_is(boat, ty::TRANSPORTBARGE)
            || self.unit_line_is(boat, ty::MERCHANTFLEET);
        if barge && !(0..self.units.len()).any(|i| self.units[i].inside_unit == Some(boat)) {
            self.transport_dies(boat);
        }
    }

    /// `Object::eject_contents(boat, 0, −1, carry, 1)@0064cd20`'s loop:
    /// steps 1–4 of [`Sim::disembark`], step 4 on `carry` (`param_3`) alone.
    pub(crate) fn eject_riders(&mut self, boat: usize, carry: bool) {
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
            // 3a. **a caravan's slot, back the way it came**
            //    (`eject_contents@0064cd20:225`–`241`, item 3370): when the
            //    boat `is_caravan`, the passenger takes the boat's `+0x86`,
            //    the route's `o` (`+0xa`) names the passenger, and the
            //    boat's goes to −1 — so the boat's death closes no route.
            if self.units[boat].ty.is_some_and(|t| {
                self.tables.unit_types[t]
                    .cols
                    .flag2(crate::ai::ai_load::uflags2::FLAG2_CARAVAN)
            }) {
                let who = self.units[boat].owner as usize;
                let slot = self.units[boat].caravan.take();
                self.units[r].caravan = slot;
                if let Some(van) = slot.and_then(|v| self.economy.caravans[who].slots.get_mut(v)) {
                    van.unit = Some(r);
                }
            }
            // 4. the order list and the path, back the way they came —
            //    `if (param_3 != 0)` (`eject_contents@0064cd20:254`).
            //    **Onto what the tail left, never in its place** (item
            //    3550, `docs/AI.md` §308): each boat order is
            //    `remove_current` off the boat's front and
            //    `LinkListBase::add@0046d5a0`ed to the back of the
            //    passenger's queue, and the boat's path is inverted and
            //    popped onto the passenger's own stack, its top re-pushed
            //    `& ~4`. So an order the tail's `add_to_army` gave —
            //    `go_to_unit`'s `ATTACKTOORDER` — stays current, with its
            //    path under the boat's. run1189 block 2172: the Explorer
            //    `1/0` lands with `ATTACKTOORDER` (42072, 43368) current
            //    over the boat's `EXPLORETOORDER`, a 17-node path whose
            //    top is the boat's (34296, 32760) `f0`.
            if !carry {
                continue;
            }
            if self.units[r]
                .ty
                .is_some_and(|t| self.tables.unit_types[t].combat.uber_size == 1)
            {
                self.hand_over_orders(boat, r);
            } else {
                self.disembark_squad(boat, r);
            }
        }
    }

    /// **`eject_contents`' step 4 on a one-figure passenger**
    /// (`eject_contents@0064cd20:254`–`330`, item 3550): the boat's orders
    /// queued **behind** the passenger's own, and its path pushed **on top
    /// of** the passenger's, the new top `& ~4`. The loop takes the boat's
    /// current order (`head->prev`, `remove_current`) and
    /// `LinkListBase::add@0046d5a0`s it to the passenger's list, which
    /// splices it before `head` — the back of the queue (`docs/ORDERS.md`
    /// §1.4) — so the passenger's current order stays current. The path is
    /// `Stack::invert`ed and popped onto the passenger's stack, which keeps
    /// the boat's own order of nodes above what was there.
    fn hand_over_orders(&mut self, boat: usize, r: usize) {
        let orders = std::mem::take(&mut self.units[boat].orders);
        self.units[r].orders.extend(orders);
        let path = std::mem::take(&mut self.units[boat].path);
        self.units[r].path.extend(path);
        if let Some(top) = self.units[r].path.last_mut() {
            top.flags &= !crate::orders::path_flag::BOARDS_TRANSPORT;
        }
    }

    /// `Object::die(boat, 0, −1, 0)` on a transport, from either of its two
    /// callers: [`Sim::disembark`] ashore, and
    /// [`Sim::boat_dies_into`] at an oil platform. `dtype` 0, so
    /// no death draw and no death object.
    pub(crate) fn transport_dies(&mut self, boat: usize) {
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

    /// **`Unit::do_gather@005ef2a0`'s oil-platform arrival for a transport**
    /// (item 1594, `docs/AI.md` §165; the listing `5efa39`..`5efacf`): a
    /// unit that `ObjectData::can_carry(GROUND)@00646c40` — a unit whose
    /// type's `carry` (`+0x2d4`) is not 0 and which is not `is(0x15f)` —
    /// arriving at the platform it gathers:
    ///
    /// 1. `Unit::same_damage(passenger, boat)` on the head of its
    ///    `inside_down` (`+0x28`, `+0x3e`), when it has one;
    /// 2. `Unit::go_inside(boat, platform, who, 0)` — the boat joins the
    ///    platform's chain;
    /// 3. `kill_current_order(boat, 0)`;
    /// 4. `Object::die(boat, 0, −1, 0)`. Its `Object::close@00647160` finds
    ///    the boat off the map (vslot `0xbc`), so it takes the
    ///    `remove_from_inside` arm and not `kill_contents`:
    ///    `Object::remove_from_inside@006480f0` splices the boat out of the
    ///    chain, the platform's `inside_down` becoming the boat's and the
    ///    passenger's `inside_up` the platform. The passenger is the
    ///    platform's now, and the boat is gone.
    ///
    /// run711 block 1954: the citizen `1/26` `inside_up 2031` (the
    /// `OILPLATFORM` its Freighter `1/37` built), `2031 inside_down 26`, and
    /// no `1/37`; uid 128, born in the gap, holds number 37.
    ///
    /// This is the first step, [`Sim::can_carry_ground`] (`do_gather_steps`
    /// asks the other three through its view): `ObjectData::can_carry(GROUND)@00646c40`, the type's `carry` (`+0x2d4`)
    /// is not 0 and the unit is not `is(0x15f)`.
    pub(crate) fn can_carry_ground(&self, boat: usize) -> bool {
        let carry = self.units[boat]
            .ty
            .is_some_and(|t| self.tables.unit_types[t].cols.carry != 0);
        carry && self.units[boat].type_index != 0x15f
    }

    /// `Unit::same_damage(passenger, boat)` on the head of the boat's
    /// `inside_down`, when it has one (`5efa4a`).
    pub(crate) fn boat_same_damage(&mut self, boat: usize) {
        let head = (0..self.units.len()).find(|&i| self.units[i].inside_unit == Some(boat));
        if let Some(head) = head {
            self.same_damage(head, boat);
        }
    }

    /// `Object::die(boat, 0, −1, 0)` on a boat that is inside the platform
    /// `b`: the splice and the boat's close.
    pub(crate) fn boat_dies_into(&mut self, boat: usize, b: usize) {
        let riders: Vec<usize> = (0..self.units.len())
            .filter(|&i| self.units[i].inside_unit == Some(boat))
            .collect();
        // The splice: each passenger takes the boat's place in the chain.
        let at = self.buildings[b]
            .garrison
            .iter()
            .position(|&g| g == boat)
            .unwrap_or(self.buildings[b].garrison.len());
        self.buildings[b].garrison.retain(|&g| g != boat);
        let mut k = at;
        for &r in &riders {
            self.units[r].inside_unit = None;
            self.units[r].inside = Some(b);
            if self.units[r].o_up.is_none() {
                self.buildings[b].garrison.insert(k, r);
                k += 1;
            }
        }
        self.units[boat].inside = None;
        self.transport_dies(boat);
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
        // Guy 0's retained point: where the passenger stood when it went
        // aboard, which `coll_come_out` paints and moves away from.
        let retained = body.pos;
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
        self.coll_come_out(r, retained);
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
        if !self.cities.nation[usize::from(self.units[r].owner)].human
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
        //
        // **And `set_angle` is the whole of it** (item 2193, `docs/AI.md`
        // §214): a turn of 90° or more (`605409`..`605418`, the same
        // `reversing` window) flips the unit's `unit_masks & 2`, which
        // `set_facing` does, and — when the passenger is its group's
        // leader on landing — `GroupData::facing`. The successor pair's
        // BMP `1/28` lands on 4363 and leads army 1 at once; its turn to
        // the boat's heading is a reversal, so the original's flag is 1
        // from there (run841's `GROUPDATA` reads 1 on 4482) and ours was
        // 0 until the formation on 4602 came out mirrored.
        let des_angle = self.units[r].movement.des_angle;
        let turned = crate::groups::group::reversing(crate::movement::Angle(
            angle.0.wrapping_sub(self.units[r].movement.heading.0),
        ));
        self.units[r].movement.set_facing(angle);
        self.units[r].movement.des_angle = des_angle;
        if turned {
            self.toggle_leader_facing(r);
        }
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
        let mut g = crate::groups::group::Group::stack(owner);
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
        dock_tile_in(
            self.world.width(),
            self.world.height(),
            |c| {
                let d = self.world.cell_data(c);
                (d.flags, d.land)
            },
            |t| self.world.tile_mask(t),
            c,
        )
    }
}

/// [`Sim::is_dock_tile`] over plain data (item 3261's sweep drives it):
/// the world's size in cells, a cell's `(flags, land)` and a tile's mask.
pub(crate) fn dock_tile_in(
    width: i32,
    height: i32,
    cell: impl Fn(Cell) -> (u16, i8),
    mask: impl Fn(Pos) -> u16,
    c: Cell,
) -> bool {
    if c.x < 0 || c.y < 0 || c.x >= width || c.y >= height {
        return false;
    }
    let water = |c: Cell| {
        let (flags, land) = cell(c);
        flags & 0x100 == 0 && (land == 1 || land == 2)
    };
    if !water(c) {
        return false;
    }
    let tw = width * TILES_PER_CELL;
    let th = height * TILES_PER_CELL;
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
        if water(Cell::new(tx / TILES_PER_CELL, ty / TILES_PER_CELL)) {
            // Open water that way: not a shore.
            continue;
        }
        let m = mask(Pos::new(tx, ty));
        if m & tile::OBJECT == tile::OBJECT_MOUNTAIN || m & tile::SURFACE == tile::SURFACE_FOREST {
            continue;
        }
        let blocked = |m: u16| m & tile::PLACED != 0 || m & tile::OBJECT == tile::OBJECT_BUILDING;
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
                if blocked(mask(t)) {
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

/// `ObjectData::is_cargo`'s test on the type of a unit on the map, with
/// `ObjectTypeData::is_cargo` and `TypeData::is_unit_type` the base ones:
/// a unit type id `0x32..=0x19d` whose domain (`+0x218`) is land.
pub(crate) fn cargo_type(type_index: i32, land: bool) -> bool {
    (0x32..=0x19d).contains(&type_index) && land
}

/// What `cast_unpack_steps` asks of the unit it unpacks, one method a call
/// of `SpellType::cast_unpack@006709c0` (item 2400).
pub(crate) trait UnpackSteps {
    /// The unit's position (the object's `+0x10`/`+0x14`), read afresh.
    fn pos(&self) -> (i32, i32);
    /// `UnitData::good_merchant_spot@006068a0` at a tile.
    fn good_merchant_spot(&mut self, tx: i32, ty: i32) -> bool;
    /// `Unit::set_new_location@005f8d20(x, y, 1, 1)`.
    fn set_new_location(&mut self, x: i32, y: i32);
    /// `World::set_blocked_at@006b4900(tile, 1)`.
    fn set_blocked(&mut self, tx: i32, ty: i32);
    /// `recalc_builds = 1` and the leader's `0x2000000`.
    fn merchant_deployed(&mut self);
    /// `unit_masks &= !0x80000`.
    fn clear_packed(&mut self);
    /// Vtable `+0x160`.
    fn update_los(&mut self);
    /// Vtable `+0x174` with `0`.
    fn update_seen(&mut self);
    /// `Unit::update_gpiece@005e2920`.
    fn update_gpiece(&mut self);
}

/// **`SpellType::cast_unpack@006709c0`'s steps** (item 2400): nothing
/// happens unless the object is alive (vslot `+8`) and on the map (vslot
/// `+0xbc`). A merchant (`TypeIndex` `0x3d`, `0x3e` or `0x190`) takes the
/// tile under it (`div_3_table[pos >> 6]` each way); the cast returns at
/// `good_merchant_spot` saying no, before the bit; else it snaps to the
/// tile's corner (`tile * 0xc0`), blocks `(x, y)`, `(x-1, y)`, `(x, y-1)`,
/// `(x-1, y-1)` in that order, and raises the leader's flag. Then, for
/// everything: the bit is cleared, `update_los`, `update_seen(0)`,
/// `update_gpiece`, and `set_new_location` on the (re-read) position.
pub(crate) fn cast_unpack_steps<C: UnpackSteps>(cx: &mut C, usable: bool, tree: Option<TypeId>) {
    if !usable {
        return;
    }
    if tree.is_some_and(|t| crate::ai::ai_census::ty::MERCHANTS.contains(&t)) {
        let (x, y) = cx.pos();
        let t = Pos::new(x, y).tile();
        if !cx.good_merchant_spot(t.x, t.y) {
            return;
        }
        cx.set_new_location(t.x * UNITS_PER_TILE, t.y * UNITS_PER_TILE);
        cx.set_blocked(t.x, t.y);
        cx.set_blocked(t.x - 1, t.y);
        cx.set_blocked(t.x, t.y - 1);
        cx.set_blocked(t.x - 1, t.y - 1);
        cx.merchant_deployed();
    }
    cx.clear_packed();
    cx.update_los();
    cx.update_seen();
    cx.update_gpiece();
    let (x, y) = cx.pos();
    cx.set_new_location(x, y);
}

/// [`cast_unpack_steps`] on a live unit.
struct SimUnpack<'a> {
    sim: &'a mut Sim,
    u: usize,
}

impl UnpackSteps for SimUnpack<'_> {
    fn pos(&self) -> (i32, i32) {
        let p = self.sim.units[self.u].pos;
        (p.x, p.y)
    }
    fn good_merchant_spot(&mut self, tx: i32, ty: i32) -> bool {
        self.sim.good_merchant_spot(self.u, Pos::new(tx, ty))
    }
    fn set_new_location(&mut self, x: i32, y: i32) {
        self.sim.set_new_location(self.u, Pos::new(x, y), true);
    }
    fn set_blocked(&mut self, tx: i32, ty: i32) {
        self.sim.set_blocked_at(Pos::new(tx, ty), true);
    }
    fn merchant_deployed(&mut self) {
        let who = self.sim.units[self.u].owner;
        self.sim.economy_changed(who);
    }
    fn clear_packed(&mut self) {
        self.sim.units[self.u].combat.packed = false;
    }
    fn update_los(&mut self) {
        // The clamp lifts here and nowhere earlier (`docs/VISION.md` §2).
        self.sim.update_los(self.u);
    }
    fn update_seen(&mut self) {
        self.sim.update_seen(self.u, false);
    }
    fn update_gpiece(&mut self) {
        self.sim.update_gpiece(self.u);
    }
}

/// What `Unit::think_carry@005f2ee0` asks of the world: three `count_inside`
/// calls and the three functions it hands the think to.
pub trait CarryView {
    /// `count_inside(COUNT_WORKERS, 0)`.
    fn count_workers(&mut self) -> i32;
    /// `count_inside(COUNT_TYPE, line)`.
    fn count_type(&mut self, line: TypeId) -> i32;
    /// `count_inside(COUNT_ATTACK, 0)`.
    fn count_attack(&mut self) -> i32;
    /// `think_carry_civilian@005f37c0(workers)`.
    fn civilian(&mut self, workers: bool) -> i32;
    /// `think_carry_military@005f3260` (the attack-carrier arm).
    fn military(&mut self) -> i32;
    /// `think_carry_idle@005f2f30`.
    fn idle(&mut self) -> i32;
}

/// `Unit::think_carry@005f2ee0` itself: the 0x42-byte dispatcher. A worker
/// aboard is `think_carry_civilian(1)`, a unit of `is(0x45)` is
/// `think_carry_civilian(0)`, an attacker aboard tail-jumps to
/// `think_carry_military`, and nothing of the three to `think_carry_idle`;
/// each callee's answer is the function's own. Swept against the emulated
/// original in `sweep/unit_think_carry.rs`.
pub fn think_carry_steps<V: CarryView>(v: &mut V) -> i32 {
    if v.count_workers() != 0 {
        return v.civilian(true);
    }
    if v.count_type(0x45) != 0 {
        return v.civilian(false);
    }
    if v.count_attack() != 0 {
        v.military()
    } else {
        v.idle()
    }
}

struct SimCarry<'a> {
    sim: &'a mut Sim,
    u: usize,
    riders: Vec<usize>,
}

impl CarryView for SimCarry<'_> {
    fn count_workers(&mut self) -> i32 {
        i32::from(self.riders.iter().any(|&i| self.sim.is_worker(i)))
    }
    fn count_type(&mut self, line: TypeId) -> i32 {
        i32::from(self.riders.iter().any(|&i| self.sim.unit_line_is(i, line)))
    }
    fn count_attack(&mut self) -> i32 {
        i32::from(
            self.riders
                .iter()
                .any(|&i| self.sim.is_captain(i) && self.sim.attack_of(Obj::Unit(i)) != 0),
        )
    }
    fn civilian(&mut self, workers: bool) -> i32 {
        i32::from(self.sim.think_carry_civilian(self.u, workers))
    }
    fn military(&mut self) -> i32 {
        i32::from(self.sim.think_carry_army(self.u, self.riders[0]))
    }
    /// `think_carry_idle` (`005f2f30`), the walk to the nearest coastal city
    /// with a dock (item 3995, `crate::units::think::carry_idle`).
    fn idle(&mut self) -> i32 {
        i32::from(self.sim.think_carry_idle(self.u))
    }
}

#[cfg(test)]
mod tests;

// The files of `ai/transport/` (item 4012, `docs/STRUCTURE.md` §3).
