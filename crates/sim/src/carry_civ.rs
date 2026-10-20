//! `Unit::think_carry_civilian@005f37c0` as steps over a view (item 3346,
//! `docs/AI.md` §195): a loaded boat looks for the shore of a land region it
//! can put its cargo on. [`think_carry_civilian_steps`] is the original's
//! control flow in its order; [`CarryCivView`] is what it reads and the
//! callees it makes, which [`SimCarryCiv`] answers from the crate's [`Sim`]
//! and `sweep/unit_think_carry_civilian.rs` from the emulated original's
//! windows. The method's account is on [`Sim::think_carry_civilian`].
//!
//! Regions are named by **slot**, the original's 1-based index into
//! `GameAccess::regions`: the scan walks slots `1..=63` in order and the
//! ascending order decides ties (the strict `<` keeps the first). The
//! crate numbers its regions densely by terrain; [`SimCarryCiv`] maps its
//! land regions onto slots `1..` in ascending order, which is the same
//! order and only a stated stand-in for the original's numbering.

use crate::Sim;
use crate::movement::{Angle, find_angle};
use crate::world::{Cell, Pos, vector_dist};

/// The slots the scan walks: `local_10 < 0x2200` at `0x88` a record.
pub const SLOTS: i32 = 63;

/// What `think_carry_civilian` reads of the unit, the region table and the
/// world, and the callees it makes.
pub trait CarryCivView {
    /// The owner byte (`this+9`).
    fn owner(&self) -> i32;
    /// The index short (`this+0xa`), sign-extended.
    fn index(&self) -> i32;
    /// `GameAccess::game->frame`.
    fn frame(&self) -> i32;
    /// The unit's position (`+0x10`, `+0x14`, the stored words xor `0x63637`).
    fn pos(&self) -> Pos;
    /// `WorldData::get_tregion` of the unit's tile.
    fn my_region(&mut self) -> i32;
    /// `Region.size` of a slot (`+0x14`).
    fn size(&self, slot: i32) -> i32;
    /// `Region::go_here(slot, owner)`.
    fn go_here(&mut self, slot: i32) -> i32;
    /// `Region::is_coast(slot, my_region)`.
    fn is_coast(&mut self, slot: i32, my: i32) -> bool;
    /// The owner's bit of `Region.scouted`.
    fn scouted(&self, slot: i32) -> bool;
    /// `Region.coords[i]`.
    fn coord(&mut self, slot: i32, i: i32) -> Cell;
    /// `Region::coast_here(slot, my_region, x, y)`.
    fn coast_here(&mut self, slot: i32, my: i32, c: Cell) -> i32;
    /// The owner's danger figure at the half-cell holding `c`.
    fn danger(&self, c: Cell) -> i32;
    /// `move_x[d]`, `move_y[d]` (the engine's table, read past nine too).
    fn step(&self, d: i32) -> (i32, i32);
    /// The slot of the world cell, `None` outside the world.
    fn cell_slot(&self, c: Cell) -> Option<i32>;
    /// `Unit::think_carry_military` (the attack carrier's, a SEAM in the crate).
    fn military(&mut self) -> bool;
    /// The order's birth, up to its `MOVE_TO`: the unit's `0x4000000` bit
    /// cleared, `+0xc0` zeroed, `close_orders(0)`, `clear_partial_path`,
    /// `update_action`; then the order built, queued and `update_action` again.
    fn move_to(&mut self, to: Pos, angle: Angle);
    /// The eject: a one-boat `Group` (`add(index, owner, 0, 0)`) and
    /// `action_eject_all(0, -1, -1, -1)`.
    fn eject_all(&mut self);
}

/// The original's control flow.
pub fn think_carry_civilian_steps<V: CarryCivView>(v: &mut V, workers: bool) -> bool {
    let pos = v.pos();
    let cx = pos.x.div_euclid(0x300);
    let cy = pos.y.div_euclid(0x300);
    let my = v.my_region();
    let mut best = 99_999_999;
    let mut best_slot = -1;
    let mut best_cell = Cell::new(0, 0);
    let mut last_dir = 0;
    for slot in 1..=SLOTS {
        let size = v.size(slot);
        if size == 0 {
            continue;
        }
        let g = v.go_here(slot);
        let g = if workers { g & 1 } else { g };
        if g == 0 || !v.is_coast(slot, my) {
            continue;
        }
        if !workers && v.scouted(slot) {
            continue;
        }
        let stride = (size / 0x32).max(0x10);
        let mut phase = v.index().wrapping_add(v.frame());
        let mut found = false;
        for _ in 0..stride {
            let start = phase % stride;
            if start < v.size(slot) {
                let mut i = start;
                loop {
                    let c = v.coord(slot, i);
                    let d = v.coast_here(slot, my, c);
                    last_dir = d;
                    if d != 0 {
                        found = true;
                        let danger = v.danger(c).max(1);
                        let dist = vector_dist(c.x - cx, c.y - cy);
                        let mut score = dist.wrapping_mul(danger);
                        if !workers {
                            if g & 1 != 0 {
                                score /= 3;
                            } else if g & 2 != 0 {
                                score /= 2;
                            }
                        }
                        if score >= 0 && score < best {
                            best = score;
                            best_slot = slot;
                            best_cell = c;
                        }
                    }
                    i += stride;
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
        return if workers { false } else { v.military() };
    }
    let shore = (0..9).find_map(|k| {
        let (mx, my_) = v.step(k);
        let c = Cell::new(cx + mx, cy + my_);
        (v.cell_slot(c) == Some(best_slot)).then_some(c)
    });
    let Some(shore) = shore else {
        let (mx, my_) = v.step(last_dir);
        let to = Pos::new(
            (best_cell.x + mx) * 0x300 + 0x180,
            (best_cell.y + my_) * 0x300 + 0x180,
        );
        v.move_to(to, find_angle(to.x - pos.x, to.y - pos.y));
        return true;
    };
    // On the shore: the sixteen quarter-cells of the boat's own cell, each
    // scored against the shore cell's middle; the nearest (the first of a
    // tie) is walked to, or ejected at when the boat already stands in it.
    let (qx, qy) = (cx * 4, cy * 4);
    let mut best_q = (qx + 2, qy + 2);
    let mut best_d = 99_999_999;
    for k in 0..16 {
        let (x4, y4) = (qx + (k & 3), qy + (k >> 2));
        let d = vector_dist(x4 - shore.x * 4 - 2, y4 - shore.y * 4 - 2);
        if d < best_d {
            best_d = d;
            best_q = (x4, y4);
        }
    }
    if pos.x.div_euclid(0xc0) == best_q.0 && pos.y.div_euclid(0xc0) == best_q.1 {
        v.eject_all();
        return true;
    }
    let to = Pos::new(best_q.0 * 0xc0 + 0x60, best_q.1 * 0xc0 + 0x60);
    v.move_to(to, find_angle(to.x - pos.x, to.y - pos.y));
    true
}

/// [`CarryCivView`] over the crate's [`Sim`].
pub struct SimCarryCiv<'a> {
    pub sim: &'a mut Sim,
    pub u: usize,
    lands: Vec<u16>,
    my: Option<u16>,
    /// A slot's coordinate list, read from the grid once.
    coords: std::collections::HashMap<i32, Vec<Cell>>,
}

impl<'a> SimCarryCiv<'a> {
    pub fn new(sim: &'a mut Sim, u: usize) -> Self {
        let lands = sim
            .world
            .regions()
            .filter(|&(_, t)| t == crate::world::Terrain::Land)
            .map(|(r, _)| r)
            .collect();
        let my = sim.world.tregion_alt(sim.units[u].pos.tile());
        SimCarryCiv {
            sim,
            u,
            lands,
            my,
            coords: std::collections::HashMap::new(),
        }
    }
    fn land(&self, slot: i32) -> Option<u16> {
        usize::try_from(slot - 1)
            .ok()
            .and_then(|i| self.lands.get(i).copied())
    }
}

impl CarryCivView for SimCarryCiv<'_> {
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
    fn my_region(&mut self) -> i32 {
        self.my.map_or(-1, i32::from)
    }
    fn size(&self, slot: i32) -> i32 {
        self.land(slot).map_or(0, |r| self.sim.world.region_size(r))
    }
    fn go_here(&mut self, slot: i32) -> i32 {
        let r = self.land(slot).expect("a sized slot");
        let who = self.sim.units[self.u].owner;
        self.sim.go_here(r, who)
    }
    fn is_coast(&mut self, slot: i32, my: i32) -> bool {
        match (self.land(slot), u16::try_from(my)) {
            (Some(r), Ok(m)) => self.sim.world.is_coast(r, m),
            _ => false,
        }
    }
    fn scouted(&self, slot: i32) -> bool {
        self.land(slot).is_some_and(|r| {
            self.sim
                .world
                .region_scouted(r, self.sim.units[self.u].owner)
        })
    }
    fn coord(&mut self, slot: i32, i: i32) -> Cell {
        let r = self.land(slot).expect("a sized slot");
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
    fn coast_here(&mut self, slot: i32, my: i32, c: Cell) -> i32 {
        match (self.land(slot), u16::try_from(my)) {
            (Some(r), Ok(m)) => self.sim.world.coast_here(r, m, c),
            _ => 0,
        }
    }
    fn danger(&self, c: Cell) -> i32 {
        self.sim.world.danger_half(self.sim.units[self.u].owner, c)
    }
    fn step(&self, d: i32) -> (i32, i32) {
        let d = usize::try_from(d).unwrap_or(0).min(8);
        (crate::ai_place::MOVE_X[d], crate::ai_place::MOVE_Y[d])
    }
    fn cell_slot(&self, c: Cell) -> Option<i32> {
        let r = self.sim.world.region_of(c)?;
        self.lands
            .iter()
            .position(|&l| l == r)
            .map(|i| i as i32 + 1)
    }
    /// SEAM: `think_carry_military@005f3260` is the attack carrier's and is
    /// not reached from here.
    fn military(&mut self) -> bool {
        false
    }
    fn move_to(&mut self, to: Pos, angle: Angle) {
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
    /// SEAM: `Group::action_eject_all(one-boat group, 0, -1, -1, -1)` is
    /// stated and not modelled; the boat keeps its cargo.
    fn eject_all(&mut self) {}
}
