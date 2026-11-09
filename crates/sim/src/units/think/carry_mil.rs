//! `Unit::think_carry_military@005f3260` as steps over a view (item 3411,
//! `docs/AI.md` §235): a boat carrying a soldier sails it to **its army's
//! target**, or — with no army — to the **nearest city** of its owner.
//! [`think_carry_military_steps`] is the original's control flow in its
//! order; [`CarryMilView`] is what it reads and the callees it makes, which
//! `sweep/unit_think_carry_military.rs` answers from the emulated
//! original's windows. Wired by item 3995 (`docs/GOLDEN.md` §188):
//! [`crate::Sim::think_carry_army`] runs these steps over [`SimCarryMil`],
//! except for an army with no live target away from its region, whose
//! region-centre arm this crate does not carry (a SEAM there).
//!
//! What the steps say, in order:
//!
//! 1. **Group.** The unit at `units[this+0x3e][this+0x28]` (the boat's
//!    first passenger) names a group by its `+0x80` short; negative ends
//!    the think with `0`. The group's `+8` is an army index.
//! 2. **No army (`< 0`).** `eject_contents(0, -1, 0, 1)`; if the boat's
//!    `+0x28` short is then negative the boat dies (the base `Object::die`
//!    when its vtable slot `+0x158` is `Unit::die`'s address, else the
//!    slot) and the answer is `1`. Otherwise the owner's cities are
//!    scanned (`flags & 1`), nearest by `vector_dist` (a quarter of it
//!    when `flags & 2`), the strict `<` keeping the first and `-1`
//!    meaning none yet; the winner's region is the goal region and its
//!    `+8` short the target object, in the owner's own object table. No
//!    city answers `0`.
//! 3. **An army.** Its `+0x30` is the target index, `+8` the goal region
//!    and `+0x34` the table the index is read in.
//! 4. **Ashore.** The nine cells of the boat (`move_x`/`move_y`, nine
//!    steps, itself first) inside the world, any in the goal region:
//!    eject again and answer `1`, dying under the same test.
//! 5. **Where to.** A target index `>= 0` in a table `>= 0` whose object
//!    has `flags & 1` is that object's cell (`div_3_table[pos >> 8]`,
//!    positions stored xor `0x63637`). A negative table with a target
//!    index reads **the last cell the shore loop looked at** (the ninth
//!    step: the compiled code leaves that pair in `ecx`/`eax` and jumps
//!    to the common tail). Anything else is the goal region's centre cell
//!    (`Region +0x7c` points at an x/y pair), replaced when the owner's
//!    leader marks the region `& 4` by the cell of its **last** city in
//!    the region (`flags & 1`, region equal), the first with `flags & 2`
//!    ending the scan.
//! 6. **The order.** The cell's middle `cell × 0x300 + 0x180`, the bearing
//!    taken to it from the boat's position, `MOVE_TO` to the middle
//!    snapped to 48 (`div_3_table[v >> 4] × 0x30 + 0x18`), queued new;
//!    answer `1`.
//!
//! Confidence: the arms above are the compiled code's, read from the
//! listing rather than the decompile where the decompile hid a register
//! (the shore loop's leftover cell, and `edx` at the bearing, which is the
//! boat's x on every path); a sweep against the emulated original holds
//! them.

use crate::Sim;
use crate::combat::Obj;
use crate::movement::{Angle, find_angle};
use crate::world::{Cell, Pos, vector_dist};

/// One city of a player's list, as the scans read it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct City {
    /// `+4`, a short: bit 0 live, bit 2 a quarter-weight (`/ 4`) city.
    pub flags: u16,
    /// `+8`, a short, sign-extended: the city's object index.
    pub obj: i32,
    /// `+0xa`, a short, sign-extended: the region it stands in.
    pub region: i32,
    /// `+0xc`.
    pub x: i32,
    /// `+0x10`.
    pub y: i32,
}

/// What the think reads of the boat, the tables and the world, and the
/// callees it makes.
pub trait CarryMilView {
    /// The group number of the boat's first passenger
    /// (`units[this+0x3e][this+0x28]`'s `+0x80` short), sign-extended.
    fn cargo_group(&mut self) -> i32;
    /// A group's army index (`groups[g]+8`).
    fn group_army(&mut self, group: i32) -> i32;
    /// An army's `(target index +0x30, goal region +8, target table +0x34)`.
    fn army(&mut self, army: i32) -> (i32, i32, i32);
    /// The boat's `+0x28` short, sign-extended.
    fn cargo_slot(&mut self) -> i32;
    /// `Object::eject_contents(this, 0, -1, 0, 1)`.
    fn eject(&mut self);
    /// Whether the boat's vtable slot `+0x158` is `Unit::die`'s address.
    fn die_is_base(&mut self) -> bool;
    /// `Object::die(this, 0, -1, 0.0)`, entered directly.
    fn die_base(&mut self);
    /// The vtable slot `+0x158`, called with the same three arguments.
    fn die_virtual(&mut self);
    /// The byte at `this+9`, the owner.
    fn owner(&self) -> i32;
    /// The boat's position (`+0x10`, `+0x14`, the stored words xor `0x63637`).
    fn pos(&self) -> Pos;
    /// The player's city count (`leaders.list[who] +0x408`).
    fn city_count(&mut self, who: i32) -> i32;
    /// The `i`th city of the player's list.
    fn city(&mut self, who: i32, i: i32) -> City;
    /// `div_3_table[i]`.
    fn div3(&mut self, i: i32) -> i32;
    /// `move_x[k]`, `move_y[k]`.
    fn step(&self, k: i32) -> (i32, i32);
    /// The world's width and height (`world +0`, `+4`).
    fn world_size(&mut self) -> (i32, i32);
    /// The region short of a world cell (`cells[w × y + x] +4`), in bounds.
    fn cell_region(&mut self, c: Cell) -> i32;
    /// The object `objects[who][index]`: `(flags & 1, +0x10 word, +0x14
    /// word)`, the position words as stored (xor `0x63637`).
    fn object(&mut self, who: i32, index: i32) -> (bool, i32, i32);
    /// A region's centre pair (`regions[r] +0x7c` points at it).
    fn region_centre(&mut self, region: i32) -> (i32, i32);
    /// Whether the player's leader marks the region `& 4`.
    fn region_marked(&mut self, who: i32, region: i32) -> bool;
    /// The order's birth up to its `MOVE_TO`: the unit's `0x4000000` bit
    /// cleared, `+0xc0` zeroed, `close_orders(0)`, `clear_partial_path`,
    /// `update_action`; then the order built, queued and `update_action`
    /// again. `to` is the unsnapped middle of the cell.
    fn order_move_to(&mut self, to: Pos, angle: Angle);
}

/// The boat's end: the base die when the slot is `Unit::die`, else the slot.
fn die<V: CarryMilView>(v: &mut V) {
    if v.die_is_base() {
        v.die_base();
    } else {
        v.die_virtual();
    }
}

/// The original's control flow.
pub fn think_carry_military_steps<V: CarryMilView>(v: &mut V) -> i32 {
    let group = v.cargo_group();
    if group < 0 {
        return 0;
    }
    let army = v.group_army(group);
    let pos = v.pos();
    let (target, region, table);
    if army < 0 {
        v.eject();
        if v.cargo_slot() < 0 {
            die(v);
            return 1;
        }
        let who = v.owner();
        let n = v.city_count(who);
        if n < 1 {
            return 0;
        }
        let (mut best, mut best_i, mut best_region) = (-1, -1, -1);
        for i in 0..n {
            let c = v.city(who, i);
            if c.flags & 1 == 0 {
                continue;
            }
            let mut d = vector_dist(pos.x.wrapping_sub(c.x), pos.y.wrapping_sub(c.y));
            if c.flags & 2 != 0 {
                d = (d + ((d >> 31) & 3)) >> 2;
            }
            if d < best || best < 0 {
                best = d;
                best_i = i;
                best_region = c.region;
            }
        }
        if best < 0 {
            return 0;
        }
        target = v.city(who, best_i).obj;
        region = best_region;
        table = who;
    } else {
        (target, region, table) = v.army(army);
    }
    let xdiv = v.div3(pos.x >> 8);
    let ydiv = v.div3(pos.y >> 8);
    let (w, h) = v.world_size();
    let mut last = (0, 0);
    for k in 0..9 {
        let (sx, sy) = v.step(k);
        let (cx, cy) = (sx.wrapping_add(xdiv), sy.wrapping_add(ydiv));
        last = (cx, cy);
        if cx >= 0 && cy >= 0 && cx < w && cy < h && v.cell_region(Cell::new(cx, cy)) == region {
            v.eject();
            if v.cargo_slot() < 0 {
                die(v);
            }
            return 1;
        }
    }
    let mut at = None;
    if target >= 0 {
        if table < 0 {
            at = Some(last);
        } else {
            let (live, ox, oy) = v.object(table, target);
            if live {
                at = Some((v.div3((ox ^ 0x63637) >> 8), v.div3((oy ^ 0x63637) >> 8)));
            }
        }
    }
    let (cx, cy) = match at {
        Some(c) => c,
        None => {
            let mut c = v.region_centre(region);
            let who = v.owner();
            if v.region_marked(who, region) {
                let n = v.city_count(who);
                for i in 0..n.max(0) {
                    let city = v.city(who, i);
                    if city.flags & 1 != 0 && city.region == region {
                        c = (v.div3(city.x >> 8), v.div3(city.y >> 8));
                        if city.flags & 2 != 0 {
                            break;
                        }
                    }
                }
            }
            c
        }
    };
    let to = Pos::new(
        cx.wrapping_mul(0x300).wrapping_add(0x180),
        cy.wrapping_mul(0x300).wrapping_add(0x180),
    );
    let angle = find_angle(to.x.wrapping_sub(pos.x), to.y.wrapping_sub(pos.y));
    v.order_move_to(to, angle);
    1
}

/// The live view: boat `u`, its head rider's group `group` (a pool slot) and
/// the army that group answers to, if any (`a.group.pool`, read through the
/// crate's army list rather than the slot's stale `+8`). The city list is
/// the owner's live cities in slot order; a city's `obj` is its index here,
/// and [`CarryMilView::object`] reads the city building through it.
pub struct SimCarryMil<'a> {
    pub sim: &'a mut Sim,
    pub u: usize,
    pub group: usize,
    pub army: Option<usize>,
    pub cities: Vec<usize>,
}

impl SimCarryMil<'_> {
    fn who(&self) -> usize {
        self.sim.units[self.u].owner as usize
    }
}

impl CarryMilView for SimCarryMil<'_> {
    fn cargo_group(&mut self) -> i32 {
        self.group as i32
    }
    fn group_army(&mut self, _group: i32) -> i32 {
        self.army.map_or(-1, |a| a as i32)
    }
    fn army(&mut self, army: i32) -> (i32, i32, i32) {
        let a = &self.sim.ai.armies[self.who()].list[army as usize];
        // The target is read back through `object`, which takes the army's
        // own `Obj`; the table is any non-negative word.
        (
            i32::from(a.target.is_some()) - 1,
            a.reg.map_or(-1, i32::from),
            0,
        )
    }
    fn cargo_slot(&mut self) -> i32 {
        let u = self.u;
        if self.sim.units.iter().any(|r| r.alive() && r.inside_unit == Some(u)) {
            0
        } else {
            -1
        }
    }
    fn eject(&mut self) {
        self.sim.eject_riders(self.u, false);
    }
    fn die_is_base(&mut self) -> bool {
        true
    }
    fn die_base(&mut self) {
        self.sim.transport_dies(self.u);
    }
    fn die_virtual(&mut self) {
        self.sim.transport_dies(self.u);
    }
    fn owner(&self) -> i32 {
        self.who() as i32
    }
    fn pos(&self) -> Pos {
        self.sim.units[self.u].pos
    }
    fn city_count(&mut self, _who: i32) -> i32 {
        self.cities.len() as i32
    }
    fn city(&mut self, _who: i32, i: i32) -> City {
        let c = &self.sim.cities.cities[self.cities[i as usize]];
        City {
            flags: u16::from(c.alive) | (u16::from(c.no_heal) << 1),
            obj: i,
            region: c.reg.map_or(-1, i32::from),
            x: c.pos.x,
            y: c.pos.y,
        }
    }
    fn div3(&mut self, i: i32) -> i32 {
        i.div_euclid(3)
    }
    fn step(&self, k: i32) -> (i32, i32) {
        use crate::ai::ai_place::{MOVE_X, MOVE_Y};
        (MOVE_X[k as usize], MOVE_Y[k as usize])
    }
    fn world_size(&mut self) -> (i32, i32) {
        (self.sim.world.width(), self.sim.world.height())
    }
    fn cell_region(&mut self, c: Cell) -> i32 {
        self.sim.world.region_of(c).map_or(-1, i32::from)
    }
    fn object(&mut self, table: i32, index: i32) -> (bool, i32, i32) {
        let target = if self.army.is_some() {
            self.army
                .and_then(|a| self.sim.ai.armies[self.who()].list[a].target)
        } else {
            let _ = table;
            Some(Obj::Building(
                self.sim.cities.cities[self.cities[index as usize]].building,
            ))
        };
        match target {
            Some(t) if self.sim.obj_alive(t) => {
                let p = self.sim.pos_of(t);
                (true, p.x ^ 0x63637, p.y ^ 0x63637)
            }
            _ => (false, 0, 0),
        }
    }
    fn region_centre(&mut self, _region: i32) -> (i32, i32) {
        unreachable!("think_carry_army answers false before the region-centre arm")
    }
    fn region_marked(&mut self, _who: i32, _region: i32) -> bool {
        false
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
