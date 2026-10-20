//! `Unit::think_civilian_transport@005f40d0` as steps over a view (item 3376,
//! `docs/TRANSPORT.md` §7): the AI's "there is nothing left for me here, and
//! there is an island". [`think_civilian_transport_steps`] is the original's
//! control flow in its order; [`CivTransportView`] is what it reads and the
//! callees it makes, which [`SimCivTransport`] answers from the crate's
//! [`Sim`] and `sweep/unit_think_civilian_transport.rs` from the emulated
//! original's windows. The method's account is on
//! [`Sim::think_civilian_transport`].
//!
//! Regions are named by **slot**, the original's index into
//! `GameAccess::regions`: land `1..=63` and sea `0x41..=0x7e`, each walked in
//! ascending order, which decides a tie (the strict `<` keeps the first). The
//! crate numbers its regions densely by terrain; [`SimCivTransport`] maps its
//! land regions onto slots `1..` and its sea regions onto `0x41..` in
//! ascending order, a stated stand-in for the original's numbering that also
//! drops any region past the sixty-third of a kind, as the original's table
//! has no slot for it.

use crate::Sim;
use crate::movement::Angle;
use crate::world::{Cell, Pos, Terrain, vector_dist};

/// The land slots the scan walks: `local < 0x2200` at `0x88` a record.
pub const LAND_SLOTS: i32 = 63;
/// The first and last sea slot (`0x2288` to `0x4378` at `0x88` a record).
pub const SEA_FIRST: i32 = 0x41;
pub const SEA_LAST: i32 = 0x7e;

/// What `think_civilian_transport` reads of the unit, the leader, the region
/// table and the world, and the callees it makes.
pub trait CivTransportView {
    /// The owner byte (`this+9`).
    fn owner(&self) -> i32;
    /// The index short (`this+0xa`), sign-extended.
    fn index(&self) -> i32;
    /// `GameAccess::game->frame`.
    fn frame(&self) -> i32;
    /// The unit's position (`+0x10`, `+0x14`, the stored words xor `0x63637`).
    fn pos(&self) -> Pos;
    /// The unit type's domain (`+0x218` of the record at `this+0x18`).
    fn domain(&self) -> i32;
    /// The byte at `+0x2b4` of the record at `this+0x18` (bit `0x10`: a type
    /// that always transports).
    fn type_flags(&self) -> i32;
    /// The owner's leader flags word (`leaders.list[owner]+0`).
    fn leader_flags(&self) -> i32;
    /// `UnitData::transport_type` of the owner's record for this unit.
    fn transport_type(&mut self) -> i32;
    /// The owner's record for this unit: `+0x68`.
    fn rec_flags68(&self) -> i32;
    /// The same record's `+0x6c`.
    fn rec_flags6c(&self) -> i32;
    /// The byte at `+0x2b4` of the record at that record's `+0x18`.
    fn rec_type_flags(&self) -> i32;
    /// The unit's own virtual slot `+0x110`.
    fn can_transport(&mut self) -> bool;
    /// Bit `0x100` of `this+0x68`, an exploring unit.
    fn exploring(&self) -> bool;
    /// `Unit::think_scout(this, 1)`.
    fn think_scout(&mut self) -> i32;
    /// `WorldData::get_tregion` of the unit's tile, as a slot (`-1` for none).
    fn my_region(&mut self) -> i32;
    /// `leaders.list[owner].xport_peasants` (`+0x9c0`).
    fn xport_peasants(&self) -> i32;
    /// `leaders.list[owner].city_num` (`+0x3f8`).
    fn city_num(&self) -> i32;
    /// The owner byte of the world cell at `(cx, cy)`, sign-extended.
    fn cell_owner(&self, cx: i32, cy: i32) -> i32;
    /// `leaders.list[owner].reg_cities[slot]`.
    fn reg_cities(&self, slot: i32) -> i32;
    /// `Region.size` of a slot (`+0x14`).
    fn size(&self, slot: i32) -> i32;
    /// `Region::go_here(slot, owner)`.
    fn go_here(&mut self, slot: i32) -> i32;
    /// `Region::is_coast(sea, other)`.
    fn is_coast(&mut self, sea: i32, other: i32) -> bool;
    /// The owner's bit of `Region.scouted`.
    fn scouted(&self, slot: i32) -> bool;
    /// `leaders.list[owner].reg_xport_peasants[slot]`.
    fn reg_xport(&self, slot: i32) -> i32;
    /// `Region.coords[i]`.
    fn coord(&mut self, slot: i32, i: i32) -> Cell;
    /// `Region::coast_here(land, sea, x, y)`.
    fn coast_here(&mut self, land: i32, sea: i32, c: Cell) -> i32;
    /// `WorldData::num_waterhalf` at a cell, called with `ecx` still the
    /// land slot's record.
    fn num_waterhalf(&mut self, land: i32, c: Cell) -> i32;
    /// The owner's danger figure at the half-cell holding `c`.
    fn danger(&self, c: Cell) -> i32;
    /// `Group::clear(-1)` of the stack group.
    fn group_clear(&mut self);
    /// `Group::add(index, owner, 0, 0)`.
    fn group_add(&mut self);
    /// `Groups::push_group(owner, group, 1)`: the slot it answers.
    fn group_push(&mut self) -> i32;
    /// `Group::action_move_near` of the pushed group to `(x, y)`, with
    /// `QUEUE_NEW`; `kind` 3 is `EXPLORE_TO`, 1 `MOVE_TO`.
    fn group_move_near(&mut self, handle: i32, x: i32, y: i32, kind: i32);
    /// The short at `+4` of the world cell at `c`: its region.
    fn cell_region(&self, c: Cell) -> i32;
    /// `leaders.list[owner].xport_peasants += 1`.
    fn bump_xport(&mut self);
    /// `leaders.list[owner].reg_xport_peasants[slot] += 1`.
    fn bump_reg_xport(&mut self, slot: i32);
}

/// The original's control flow; `eax` as the original answers it.
pub fn think_civilian_transport_steps<V: CivTransportView>(v: &mut V, colonise: bool) -> i32 {
    let domain = v.domain();
    if domain == 1 && v.type_flags() & 0x10 == 0 {
        return 0;
    }
    if domain != 2 {
        let f = v.leader_flags();
        let level = if f & 0x100 != 0 {
            3
        } else if f & 0x200 != 0 {
            2
        } else {
            (f >> 10) & 1
        };
        let tt = v.transport_type();
        // The record's own test and the unit's virtual `+0x110`: the second
        // is asked only when the first does not already refuse.
        let refused = level < tt || {
            let not_cargo = ((v.rec_flags68() & 0x80_0000) == 0 || (v.rec_flags6c() & 0x2000) != 0)
                && (v.rec_type_flags() & 0x10) == 0;
            not_cargo || !v.can_transport()
        };
        if refused {
            return if v.exploring() { v.think_scout() } else { 0 };
        }
    }
    let pos = v.pos();
    let (cx, cy) = (pos.x.div_euclid(0x300), pos.y.div_euclid(0x300));
    let my = v.my_region();
    let who = v.owner();
    if colonise
        && (v.xport_peasants() >= v.city_num()
            || v.cell_owner(cx, cy) != who
            || v.reg_cities(my) == 0)
    {
        return 0;
    }
    let mut best = 99_999_999;
    let mut best_slot = -1;
    let mut best_cell = Cell::new(0, 0);
    for slot in 1..=LAND_SLOTS {
        let size = v.size(slot);
        if size == 0 {
            continue;
        }
        let g = v.go_here(slot);
        let accept = if g & 1 != 0 {
            if colonise {
                v.reg_xport(slot) == 0
            } else {
                !v.scouted(slot)
            }
        } else if colonise || v.go_here(slot) & 2 == 0 {
            false
        } else {
            !v.scouted(slot)
        };
        if !accept {
            continue;
        }
        let mut sea = SEA_FIRST;
        while sea <= SEA_LAST {
            if v.size(sea) != 0 && v.is_coast(sea, my) && v.is_coast(sea, slot) {
                break;
            }
            sea += 1;
        }
        if sea == SEA_LAST + 1 {
            continue;
        }
        let stride = if size / 0x32 < 0x10 {
            0x10
        } else {
            size / 0x32
        };
        let mut phase = v.index().wrapping_add(v.frame());
        let mut found = false;
        for _ in 0..stride {
            let start = phase % stride;
            if start < v.size(slot) {
                let mut i = start;
                loop {
                    let c = v.coord(slot, i);
                    if v.coast_here(slot, sea, c) != 0 && v.num_waterhalf(slot, c) == 0 {
                        found = true;
                        let danger = v.danger(c).max(1);
                        let dist = vector_dist((c.x - cx).abs(), (c.y - cy).abs());
                        let score = dist.wrapping_mul(danger);
                        if score >= 0 && score < best {
                            best = score;
                            best_slot = slot;
                            best_cell = c;
                        }
                    }
                    i = i.wrapping_add(stride);
                    if i >= v.size(slot) {
                        break;
                    }
                }
                if found {
                    break;
                }
            }
            phase = phase.wrapping_add(1);
        }
    }
    if best_slot < 0 {
        return if v.exploring() { v.think_scout() } else { 0 };
    }
    v.group_clear();
    v.group_add();
    let handle = v.group_push();
    let kind = if v.exploring() { 3 } else { 1 };
    v.group_move_near(
        handle,
        best_cell.x * 0x300 + 0x180,
        best_cell.y * 0x300 + 0x180,
        kind,
    );
    if colonise {
        v.bump_xport();
        let r = v.cell_region(best_cell);
        v.bump_reg_xport(r);
    }
    1
}

/// [`CivTransportView`] over the crate's [`Sim`].
pub struct SimCivTransport<'a> {
    pub sim: &'a mut Sim,
    pub u: usize,
    lands: Vec<u16>,
    seas: Vec<u16>,
    sizes: Vec<i32>,
    /// A land slot's coordinate list, read from the grid once.
    coords: std::collections::BTreeMap<i32, Vec<Cell>>,
    group: Option<crate::group::Group>,
    pushed: bool,
}

impl<'a> SimCivTransport<'a> {
    pub fn new(sim: &'a mut Sim, u: usize) -> Self {
        let band = |t: Terrain| -> Vec<u16> {
            sim.world
                .regions()
                .filter(|&(_, k)| k == t)
                .map(|(r, _)| r)
                .take(LAND_SLOTS as usize)
                .collect()
        };
        let lands = band(Terrain::Land);
        let mut seas = band(Terrain::Sea);
        seas.truncate((SEA_LAST - SEA_FIRST + 1) as usize);
        // `Region.size` for every region at once: the scan asks for most of
        // them and each answer is a sweep of the grid.
        let sizes = (0..sim.world.region_count() as u16)
            .map(|r| sim.world.region_size(r))
            .collect();
        SimCivTransport {
            sim,
            u,
            lands,
            seas,
            sizes,
            coords: std::collections::BTreeMap::new(),
            group: None,
            pushed: false,
        }
    }

    /// The region a slot names, `None` outside both bands or past the
    /// regions the crate has.
    fn region(&self, slot: i32) -> Option<u16> {
        if (1..=LAND_SLOTS).contains(&slot) {
            self.lands.get((slot - 1) as usize).copied()
        } else if (SEA_FIRST..=SEA_LAST).contains(&slot) {
            self.seas.get((slot - SEA_FIRST) as usize).copied()
        } else {
            None
        }
    }

    /// The slot a region sits in, `-1` for one with none.
    fn slot_of(&self, r: u16) -> i32 {
        if let Some(i) = self.lands.iter().position(|&l| l == r) {
            i as i32 + 1
        } else if let Some(i) = self.seas.iter().position(|&l| l == r) {
            i as i32 + SEA_FIRST
        } else {
            -1
        }
    }
}

impl CivTransportView for SimCivTransport<'_> {
    fn owner(&self) -> i32 {
        i32::from(self.sim.units[self.u].owner)
    }
    fn index(&self) -> i32 {
        i32::from(self.sim.units[self.u].index)
    }
    fn frame(&self) -> i32 {
        i32::try_from(self.sim.frame).unwrap_or(i32::MAX)
    }
    fn pos(&self) -> Pos {
        self.sim.units[self.u].pos
    }
    fn domain(&self) -> i32 {
        match self.sim.unit_domain_of(self.u) {
            crate::attrition::Domain::Land => 0,
            crate::attrition::Domain::Sea => 1,
            crate::attrition::Domain::Air => 2,
        }
    }
    fn type_flags(&self) -> i32 {
        self.rec_type_flags()
    }
    fn leader_flags(&self) -> i32 {
        use crate::transport::TransportType as T;
        match self.sim.transport_level(self.sim.units[self.u].owner) {
            T::Civilian => 0x100,
            T::Military => 0x200,
            T::Scout => 0x400,
            T::None => 0,
        }
    }
    fn transport_type(&mut self) -> i32 {
        i32::from(self.sim.unit_transport_type(self.u) as u8)
    }
    /// SEAM: the record's two flag words are not carried; the crate's own
    /// cargo test (an on-map land unit type of the unit range) stands in for
    /// the bit the original reads here.
    fn rec_flags68(&self) -> i32 {
        if self.sim.is_cargo(self.u) {
            0x80_0000
        } else {
            0
        }
    }
    fn rec_flags6c(&self) -> i32 {
        0
    }
    fn rec_type_flags(&self) -> i32 {
        self.sim.units[self.u]
            .ty
            .map_or(0, |t| self.sim.unit_types[t].cols.unit_flags as i32)
    }
    fn can_transport(&mut self) -> bool {
        self.sim.unit_can_transport(self.u)
    }
    /// SEAM: `this+0x68 & 0x100` is clear on every unit in every capture.
    fn exploring(&self) -> bool {
        false
    }
    /// SEAM: the exploring arms are unreachable with that bit clear.
    fn think_scout(&mut self) -> i32 {
        0
    }
    fn my_region(&mut self) -> i32 {
        self.sim
            .world
            .tregion_alt(self.sim.units[self.u].pos.tile())
            .map_or(-1, |r| self.slot_of(r))
    }
    fn xport_peasants(&self) -> i32 {
        let w = self.sim.units[self.u].owner as usize;
        self.sim.ai[w].census.xport_peasants
    }
    fn city_num(&self) -> i32 {
        self.sim.city_num(self.sim.units[self.u].owner)
    }
    fn cell_owner(&self, cx: i32, cy: i32) -> i32 {
        use crate::world::Owner;
        match self.sim.world.owner(Cell::new(cx, cy)) {
            Owner::Player(p) => i32::from(p),
            Owner::Ambiguous => -2,
            Owner::None => -1,
        }
    }
    fn reg_cities(&self, slot: i32) -> i32 {
        self.region(slot)
            .map_or(0, |r| self.sim.reg_cities(self.sim.units[self.u].owner, r))
    }
    fn size(&self, slot: i32) -> i32 {
        self.region(slot)
            .map_or(0, |r| self.sizes.get(r as usize).copied().unwrap_or(0))
    }
    fn go_here(&mut self, slot: i32) -> i32 {
        let r = self.region(slot).expect("a sized slot");
        let who = self.sim.units[self.u].owner;
        self.sim.go_here(r, who)
    }
    fn is_coast(&mut self, sea: i32, other: i32) -> bool {
        match (self.region(sea), self.region(other)) {
            (Some(s), Some(o)) => self.sim.world.is_coast(s, o),
            _ => false,
        }
    }
    fn scouted(&self, slot: i32) -> bool {
        self.region(slot).is_some_and(|r| {
            self.sim
                .world
                .region_scouted(r, self.sim.units[self.u].owner)
        })
    }
    fn reg_xport(&self, slot: i32) -> i32 {
        let w = self.sim.units[self.u].owner as usize;
        self.region(slot).map_or(1, |r| {
            self.sim.ai[w]
                .census
                .reg_xport_peasants
                .get(r as usize)
                .map_or(1, |&n| n)
        })
    }
    fn coord(&mut self, slot: i32, i: i32) -> Cell {
        let r = self.region(slot).expect("a sized slot");
        let world = &self.sim.world;
        let list = self
            .coords
            .entry(slot)
            .or_insert_with(|| world.region_coords_strided(r, 0, 1));
        usize::try_from(i)
            .ok()
            .and_then(|i| list.get(i).copied())
            .unwrap_or(Cell::new(0, 0))
    }
    fn coast_here(&mut self, land: i32, sea: i32, c: Cell) -> i32 {
        match (self.region(land), self.region(sea)) {
            (Some(r), Some(s)) => self.sim.world.coast_here(r, s, c),
            _ => 0,
        }
    }
    fn num_waterhalf(&mut self, _land: i32, c: Cell) -> i32 {
        self.sim.world.num_waterhalf(c)
    }
    fn danger(&self, c: Cell) -> i32 {
        self.sim.world.danger_half(self.sim.units[self.u].owner, c)
    }
    fn group_clear(&mut self) {
        self.group = Some(crate::group::Group::stack(self.sim.units[self.u].owner));
    }
    fn group_add(&mut self) {
        if let Some(g) = self.group.as_mut() {
            self.sim.group_add(g, self.u);
        }
    }
    fn group_push(&mut self) -> i32 {
        if let Some(g) = self.group.as_mut() {
            self.pushed = self.sim.push_group(g, true);
        }
        0
    }
    /// SEAM: the original sends the group wherever the push put it without
    /// asking whether it did; a refusal here sends nothing.
    fn group_move_near(&mut self, _handle: i32, x: i32, y: i32, kind: i32) {
        let Some(g) = self.group.as_ref().filter(|_| self.pushed) else {
            return;
        };
        let kind = if kind == 3 {
            crate::orders::MoveKind::ExploreTo
        } else {
            crate::orders::MoveKind::MoveTo
        };
        self.sim.group_action_move_to(
            g,
            Pos::new(x, y),
            crate::orders::QueuePos::New,
            false,
            Angle(0),
            kind,
            false,
        );
    }
    fn cell_region(&self, c: Cell) -> i32 {
        self.sim.world.region_of(c).map_or(-1, |r| self.slot_of(r))
    }
    fn bump_xport(&mut self) {
        let w = self.sim.units[self.u].owner as usize;
        self.sim.ai[w].census.xport_peasants += 1;
    }
    fn bump_reg_xport(&mut self, slot: i32) {
        let w = self.sim.units[self.u].owner as usize;
        if let Some(r) = self.region(slot)
            && let Some(n) = self.sim.ai[w].census.reg_xport_peasants.get_mut(r as usize)
        {
            *n += 1;
        }
    }
}
