//! `Army::find_muster_spot@006f5cc0` as steps over a view (item 3348,
//! `docs/ARMY.md` §13): the cell beside a target city an army forms at.
//! [`find_muster_spot_steps`] is the original's control flow in its order;
//! [`MusterView`] is what it reads and the callees it makes, which
//! [`SimMuster`] answers from the crate's [`Sim`] and
//! `sweep/army_find_muster_spot.rs` from the emulated original's windows.
//! The method's account is on [`Sim::find_muster_spot`].
//!
//! Every method of the view is **one reach into the original**: the object
//! methods are each a fresh `objects[tw][i]->vtable[0xac]()` and then one read
//! on what it answered, which is how the original asks (it never keeps the
//! record between questions), so the sweep's per-call windows are answered in
//! the same order and the recorded calls hold the order itself.
//!
//! The target and the chain of its city's buildings are **handles**: the
//! original's are indices into the target owner's object row (`param_1`, then
//! the type record's `+0x74` link), the crate's are positions in
//! [`SimMuster`]'s `elems`.

use crate::army::Army;
use crate::combat::Obj;
use crate::movement::Angle;
use crate::world::{Cell, MOVE_49, Owner, Pos, vector_dist};
use crate::{Player, Sim};

/// What `find_muster_spot` reads of the army, the leaders, the objects, the
/// world and the other armies, and the callees it makes.
pub trait MusterView {
    /// `this+0x94`, the army's leader.
    fn who(&self) -> i32;
    /// `param_2`, the target's owner (its row of the object table).
    fn tw(&self) -> i32;
    /// `param_1`, the target's handle.
    fn first(&self) -> i32;
    /// `param_3 != 0`: the 7 x 7 neighbourhood.
    fn wide(&self) -> bool;
    /// `this+0x24 != 0`.
    fn navy(&self) -> bool;
    /// `this+0x2`, the army's slot.
    fn slot(&self) -> i32;
    /// `this+0x8`, the army's region.
    fn reg(&self) -> i32;
    /// `this+0x2c = h`.
    fn set_hurry(&mut self, h: i32);
    /// `(this+0x48, this+0x4c)`.
    fn muster(&self) -> (i32, i32);
    /// Writes `(this+0x48, this+0x4c)`.
    fn set_muster(&mut self, x: i32, y: i32);
    /// The army's point as a cell: `div_3_table[this+0x38 >> 8]`, `[this+0x3c >> 8]`.
    fn army_cell(&self) -> (i32, i32);
    /// `find_angle@0092d130(dx, dy)`.
    fn find_angle(&mut self, dx: i32, dy: i32) -> i32;
    /// `this+0x50 = a`.
    fn set_angle(&mut self, a: i32);
    /// Whether leader `b` is `a`'s friend: `b == leaders[a].+8`, or each
    /// side's diplomacy toward the other is 2 (`LeaderData::is_ally`).
    fn friend(&mut self, a: i32, b: i32) -> bool;
    /// `leaders[who].flags & 0x700 != 0`: it can transport.
    fn transports(&self) -> bool;
    /// The world's width and height in cells (`worldc +0`, `+4`).
    fn world_cells(&self) -> (i32, i32);

    /// `objects[tw][i]->vtable[0xac]()->+0x24 != 0`.
    fn elem_damaged(&mut self, i: i32) -> bool;
    /// `objects[tw][i]->vtable[0xac]()->+0x60 & 0x20`.
    fn elem_alive(&mut self, i: i32) -> bool;
    /// `objects[tw][i]->vtable[0xac]()->+0x74`: the next handle, negative for none.
    fn elem_next(&mut self, i: i32) -> i32;
    /// `objects[tw][i]`'s point as a cell (`div_3_table[(+0x10 ^ 0x63637) >> 8]`, `+0x14`).
    fn elem_cell(&self, i: i32) -> (i32, i32);
    /// `find_unit(point of i, SEARCH_ENEMY, who, 0x900, 0x200, FILTER_COMBAT, ...)`.
    fn find_combat(&mut self, i: i32) -> i32;
    /// `find_unit(point of i, SEARCH_ENEMY, who, 0x1200, 0x200, FILTER_TARGET, i, tw, ...)`.
    fn find_target(&mut self, i: i32) -> i32;
    /// `find_building(point of i, SEARCH_ENEMY, who, 0x1200, 0x200, FILTER_COMBAT, 0, 0)`.
    fn find_building(&mut self, i: i32) -> i32;
    /// `count(this, COUNT_SIEGE, 0)`.
    fn siege(&mut self) -> i32;
    /// `objects.field_0x200 >= 0`: the row a search result lies in.
    fn found_row_ok(&self) -> bool;
    /// A search result's point as a cell.
    fn found_cell(&self, f: i32) -> (i32, i32);

    /// `objects[tw][param_1]->vtable[0xac]()->+0x8 & 0x20`: the target's type is a city's.
    fn target_type_city(&mut self) -> bool;
    /// `BuildData::is_unassimilated` of the city's building (`vtable[0xac]()->+0x72`
    /// names the city, its record `+0x5e`/`+0x8` the building, `vtable[0x3c]()` its data).
    fn city_unassimilated(&mut self) -> bool;
    /// `CityData::get_radius` of the city that call named.
    fn city_radius(&mut self) -> i32;
    /// `objects[tw][param_1]->+0x8 & 0x20`: the target is a city.
    fn target_is_city(&self) -> bool;
    /// `objects[tw][param_1]->vtable[0xac]()->+0x72`: the target's city, negative for none.
    fn target_city(&mut self) -> i32;
    /// `cities[tw][c]->+0x4`.
    fn city_flags(&self, c: i32) -> i32;
    /// Writes `cities[tw][c]->+0x4`.
    fn set_city_flags(&mut self, c: i32, f: i32);
    /// `div_3_table[cities[tw][c]->+0xc >> 8]`, `[+0x10 >> 8]`: the city's cell, for the fallback.
    fn city_cell(&self, c: i32) -> (i32, i32);

    /// `armies[who].count`.
    fn armies(&self) -> i32;
    /// `armies[who].list[j]->valid != 0`.
    fn army_valid(&self, j: i32) -> bool;
    /// `armies[who].list[j]->+0x48`, `+0x4c`.
    fn army_muster(&self, j: i32) -> (i32, i32);

    /// `WData.flags` of the cell, as a `u16`.
    fn cell_flags(&self, x: i32, y: i32) -> i32;
    /// `WData.land` of the cell, a signed byte.
    fn cell_land(&self, x: i32, y: i32) -> i32;
    /// `WData.region` of the cell, a signed short.
    fn cell_region(&self, x: i32, y: i32) -> i32;
    /// `WData.who` of the cell, a signed byte: `-1` for none.
    fn cell_owner(&self, x: i32, y: i32) -> i32;
    /// `lands[class].move_rate`.
    fn move_rate(&self, class: i32) -> i32;
}

/// The original's control flow; the return is its `eax` (the last in-bounds
/// candidate's verdict, or 1), as a bool.
pub fn find_muster_spot_steps<V: MusterView>(v: &mut V) -> bool {
    let (who, tw, first, navy) = (v.who(), v.tw(), v.first(), v.navy());
    v.set_hurry(0);
    let mut local_c = false;
    if v.friend(who, tw) && !navy {
        v.set_muster(-1, -1);
        if first >= 0 {
            let mut i = first;
            loop {
                if v.elem_damaged(i) && v.elem_alive(i) {
                    let mut f = v.find_combat(i);
                    if f < 0 {
                        f = v.find_target(i);
                        if f < 0 && v.siege() != 0 {
                            f = v.find_building(i);
                        }
                    }
                    if f >= 0 && v.found_row_ok() {
                        let (fx, fy) = v.found_cell(f);
                        v.set_muster(fx, fy);
                        let (tx, ty) = v.elem_cell(first);
                        let a = v.find_angle(fx - tx, fy - ty);
                        v.set_angle(a);
                        v.set_hurry(1);
                        return true;
                    }
                    let (mx, my) = v.muster();
                    if mx < 0 || my < 0 {
                        let (cx, cy) = v.elem_cell(i);
                        v.set_muster(cx, cy);
                    }
                }
                i = v.elem_next(i);
                if i < 0 {
                    break;
                }
            }
        }
        let (mx, my) = v.muster();
        if mx < 0 || my < 0 {
            let (tx, ty) = v.elem_cell(first);
            v.set_muster(tx, ty);
        }
        let (_, h) = v.world_cells();
        let (mx, my) = v.muster();
        v.set_muster(mx, if my < h - 1 { my + 1 } else { my - 1 });
        let (mx, my) = v.muster();
        let (tx, ty) = v.elem_cell(first);
        let a = v.find_angle(mx - tx, my - ty);
        v.set_angle(a);
    }

    let (ax, ay) = v.army_cell();
    let mut inner = if !v.target_type_city() || navy {
        5
    } else if v.city_unassimilated() {
        4
    } else {
        let r = v.city_radius();
        let mut inner = ((r + ((r >> 31) & 3)) >> 2) + 1;
        if v.siege() == 0 && who != tw {
            inner /= 2;
        }
        inner
    };
    let walk = if v.wide() { 0x31 } else { 9 };
    let scored = if navy { 0x19 } else { 9 };
    let outer = (inner + (3 * i32::from(navy) + 1) * 2).min(0x40);
    // The tables hold 0x41 rings; a ring below the first is none.
    inner = inner.max(0);
    let circle = crate::ai_place::circle();
    let (lo, hi) = (circle.radius[inner as usize], circle.radius[outer as usize]);
    let (w, h) = v.world_cells();
    let slot = v.slot();
    let reg = v.reg();
    let any_land = !navy && v.transports();

    // The best candidate so far: its score, its ring index, its cell.
    let mut best_score = 0;
    let mut best_idx = 999_999;
    let mut best: Option<(i32, i32)> = None;
    let mut i = lo;
    while i < hi {
        let (cx, cy) = (ax + circle.x[i], ay + circle.y[i]);
        if cx >= 0 && cy >= 0 && cx < w && cy < h {
            // Not too near another army of mine — the first that is ends the
            // candidate's walk before it scores, so it cannot win, and
            // leaves the last verdict as it was.
            let mut too_near = false;
            for j in 0..v.armies() {
                if j != slot && v.army_valid(j) {
                    let near = if v.friend(tw, who) { 4 } else { 2 };
                    let (mx, my) = v.army_muster(j);
                    if vector_dist((cx - mx).abs(), (cy - my).abs()) <= near {
                        too_near = true;
                        break;
                    }
                }
            }
            if !too_near {
                let mut score = 0;
                let mut out = false;
                for (j, &(dx, dy)) in MOVE_49[..walk].iter().enumerate() {
                    let (nx, ny) = (cx + dx, cy + dy);
                    if nx < 0 || ny < 0 || nx >= w || ny >= h {
                        out = true;
                        break;
                    }
                    if !navy {
                        // The neighbour's own neighbour in the same
                        // direction: another leader's cell — not mine, not
                        // a friend's, not the target owner's — is out.
                        let (mx, my) = (nx + dx, ny + dy);
                        if mx >= 0 && my >= 0 && mx < w && my < h {
                            let o = v.cell_owner(mx, my);
                            if o != -1 && !v.friend(who, o) && o != tw {
                                out = true;
                                break;
                            }
                        }
                    }
                    if j >= scored {
                        continue;
                    }
                    let nreg = v.cell_region(nx, ny);
                    if !(nreg == reg || (any_land && nreg < 0x41)) {
                        out = true;
                        break;
                    }
                    let f = v.cell_flags(nx, ny);
                    let land = v.cell_land(nx, ny);
                    // `WorldData::is_ocean`: not half land, and land 1 or 2.
                    let water = f & 0x100 == 0 && (land == 1 || land == 2);
                    // The five tests `WorldData::get_land` runs.
                    let class = if f & 4 != 0 {
                        3
                    } else if f & 0x20 != 0 {
                        4
                    } else if f & 0x50 != 0 {
                        5
                    } else if f & 8 != 0 {
                        ((f & 0x800) | 0x3000) >> 11
                    } else if water && f & 0x800 != 0 {
                        7
                    } else {
                        land
                    };
                    let admissible = if navy {
                        water
                    } else {
                        !water && class != 5 && class != 4 && class != 3
                    };
                    if !admissible {
                        out = true;
                        break;
                    }
                    if f & 0x4000 == 0 {
                        score += v.move_rate(class);
                    } else if navy && j > 8 {
                        score += 10;
                    }
                }
                local_c = !out;
                if !out && best_score < score {
                    best_score = score;
                    best = Some((cx, cy));
                    best_idx = i as i32;
                }
            }
            // The early stop: more than 0x28 entries past the best.
            if i as i32 - best_idx > 0x28 {
                local_c = true;
                break;
            }
        }
        i += 1;
    }
    if let Some((bx, by)) = best {
        v.set_muster(bx, by);
        let a = v.find_angle(bx - ax, by - ay);
        v.set_angle(a);
        if v.target_is_city() && who == tw {
            let c = v.target_city();
            if c >= 0 {
                let f = v.city_flags(c);
                if f & 1 != 0 {
                    v.set_city_flags(c, f & !0x2000);
                }
            }
        }
        return local_c;
    }

    // No cell. A land army at a city centre that is not a friend's, or whose
    // type is damaged: the city's cell, one row down.
    if !navy && v.target_is_city() && (!v.friend(who, tw) || v.elem_damaged(first)) {
        let c = v.target_city();
        let (cx, cy) = v.city_cell(c);
        v.set_muster(cx, cy);
        let (_, h) = v.world_cells();
        let (mx, my) = v.muster();
        v.set_muster(mx, if my < h - 1 { my + 1 } else { my - 1 });
        let (mx, my) = v.muster();
        let a = v.find_angle(mx - ax, my - ay);
        v.set_angle(a);
        return true;
    }
    // An active, untroubled city centre of my own: the mark §12 divides by
    // 20, and the caller sees the failure.
    if v.target_is_city() && who == tw {
        let c = v.target_city();
        if c >= 0 {
            let f = v.city_flags(c);
            if f & 3 == 1 {
                v.set_city_flags(c, f | 0x2000);
            }
        }
    }
    local_c
}

/// What one handle of the chain is, as the crate holds it.
struct Elem {
    /// The building, for a building target's chain.
    b: Option<usize>,
    pos: Pos,
    damaged: bool,
    alive: bool,
}

/// [`MusterView`] on the crate's state: the army `slot` of `who`, the target
/// `target` and its city's chain.
pub struct SimMuster<'a> {
    sim: &'a mut Sim,
    who: Player,
    slot: usize,
    tw: Player,
    wide: bool,
    /// The target, then the buildings after it in its city's chain.
    elems: Vec<Elem>,
    /// The target's city, when it is the city's centre.
    city: Option<(usize, usize)>,
    /// What the searches found, as cells.
    found: Vec<Cell>,
}

impl<'a> SimMuster<'a> {
    pub fn new(sim: &'a mut Sim, who: Player, slot: usize, target: Obj, wide: bool) -> Self {
        let tw = sim.owner_of(target);
        let elems = match target {
            Obj::Building(b) => {
                let chain: Vec<usize> = match sim.buildings[b].city {
                    Some(c) => {
                        let ch = sim.city_chain(c);
                        ch.iter()
                            .position(|&x| x == b)
                            .map_or_else(|| vec![b], |at| ch[at..].to_vec())
                    }
                    None => vec![b],
                };
                chain
                    .into_iter()
                    .map(|m| {
                        let bd = &sim.buildings[m];
                        Elem {
                            b: Some(m),
                            pos: bd.pos,
                            damaged: bd.health < bd.hits,
                            alive: bd.alive,
                        }
                    })
                    .collect()
            }
            Obj::Unit(_) => vec![Elem {
                b: None,
                pos: sim.pos_of(target),
                damaged: false,
                alive: false,
            }],
        };
        let city = match target {
            Obj::Building(b) => sim.buildings[b]
                .city
                .filter(|&c| sim.cities[c].building == b)
                .map(|c| (b, c)),
            Obj::Unit(_) => None,
        };
        SimMuster {
            sim,
            who,
            slot,
            tw,
            wide,
            elems,
            city,
            found: Vec::new(),
        }
    }

    fn army(&self) -> &Army {
        &self.sim.armies[self.who as usize].list[self.slot]
    }

    fn army_mut(&mut self) -> &mut Army {
        &mut self.sim.armies[self.who as usize].list[self.slot]
    }

    fn found_at(&mut self, p: Cell) -> i32 {
        self.found.push(p);
        (self.found.len() - 1) as i32
    }
}

impl MusterView for SimMuster<'_> {
    fn who(&self) -> i32 {
        i32::from(self.who)
    }
    fn tw(&self) -> i32 {
        i32::from(self.tw)
    }
    fn first(&self) -> i32 {
        0
    }
    fn wide(&self) -> bool {
        self.wide
    }
    fn navy(&self) -> bool {
        self.army().navy
    }
    fn slot(&self) -> i32 {
        self.slot as i32
    }
    fn reg(&self) -> i32 {
        self.army().reg.map_or(-1, i32::from)
    }
    fn set_hurry(&mut self, h: i32) {
        self.army_mut().hurry = h;
    }
    fn muster(&self) -> (i32, i32) {
        let m = self.army().muster;
        (m.x, m.y)
    }
    fn set_muster(&mut self, x: i32, y: i32) {
        self.army_mut().muster = Cell { x, y };
    }
    fn army_cell(&self) -> (i32, i32) {
        let c = self.army().pos.cell();
        (c.x, c.y)
    }
    fn find_angle(&mut self, dx: i32, dy: i32) -> i32 {
        crate::movement::find_angle(dx, dy).0
    }
    fn set_angle(&mut self, a: i32) {
        self.army_mut().muster_angle = Angle(a);
    }
    fn friend(&mut self, a: i32, b: i32) -> bool {
        // Total: a leader outside the table is nobody's friend.
        let n = self.sim.players.len() as i32;
        (0..n).contains(&a) && (0..n).contains(&b) && self.sim.is_ally(a as Player, b as Player)
    }
    fn transports(&self) -> bool {
        self.sim.transport_level(self.who) as i32 != 0
    }
    fn world_cells(&self) -> (i32, i32) {
        (self.sim.world.width(), self.sim.world.height())
    }

    fn elem_damaged(&mut self, i: i32) -> bool {
        self.elems[i as usize].damaged
    }
    fn elem_alive(&mut self, i: i32) -> bool {
        self.elems[i as usize].alive
    }
    fn elem_next(&mut self, i: i32) -> i32 {
        if (i as usize) + 1 < self.elems.len() {
            i + 1
        } else {
            -1
        }
    }
    fn elem_cell(&self, i: i32) -> (i32, i32) {
        let c = self.elems[i as usize].pos.cell();
        (c.x, c.y)
    }
    fn find_combat(&mut self, i: i32) -> i32 {
        let p = self.elems[i as usize].pos;
        match self.sim.nearest_enemy_attacker(self.who, p, 0x900) {
            Some(u) => {
                let c = self.sim.units[u].pos.cell();
                self.found_at(c)
            }
            None => -1,
        }
    }
    fn find_target(&mut self, i: i32) -> i32 {
        let e = &self.elems[i as usize];
        let (who, p) = (self.who, e.pos);
        let Some(b) = e.b else {
            return -1;
        };
        let found = self.sim.units.iter().position(|u| {
            u.alive()
                && u.on_map
                && !u.is_gaia()
                && self.sim.is_enemy(who, u.owner)
                && u.combat.target == Some(Obj::Building(b))
                && vector_dist((u.pos.x - p.x).abs(), (u.pos.y - p.y).abs()) <= 0x1200
        });
        match found {
            Some(u) => {
                let c = self.sim.units[u].pos.cell();
                self.found_at(c)
            }
            None => -1,
        }
    }
    fn find_building(&mut self, _i: i32) -> i32 {
        // SEAM: the siege army's enemy-building search is not modelled.
        -1
    }
    fn siege(&mut self) -> i32 {
        self.sim.army_count_siege(self.who, self.slot)
    }
    fn found_row_ok(&self) -> bool {
        true
    }
    fn found_cell(&self, f: i32) -> (i32, i32) {
        let c = self.found[f as usize];
        (c.x, c.y)
    }

    fn target_type_city(&mut self) -> bool {
        self.city.is_some()
    }
    fn city_unassimilated(&mut self) -> bool {
        self.city
            .is_some_and(|(b, _)| self.sim.building_unassimilated(b))
    }
    fn city_radius(&mut self) -> i32 {
        self.city.map_or(0, |(_, c)| self.sim.radius_of(c))
    }
    fn target_is_city(&self) -> bool {
        self.city.is_some()
    }
    fn target_city(&mut self) -> i32 {
        self.city.map_or(-1, |(_, c)| c as i32)
    }
    fn city_flags(&self, c: i32) -> i32 {
        let city = &self.sim.cities[c as usize];
        i32::from(city.alive) | i32::from(city.no_heal) << 1 | i32::from(city.no_muster) << 13
    }
    fn set_city_flags(&mut self, c: i32, f: i32) {
        self.sim.cities[c as usize].no_muster = f & 0x2000 != 0;
    }
    fn city_cell(&self, c: i32) -> (i32, i32) {
        let p = self.sim.cities[c as usize].pos.cell();
        (p.x, p.y)
    }

    fn armies(&self) -> i32 {
        self.sim.armies[self.who as usize].list.len() as i32
    }
    fn army_valid(&self, j: i32) -> bool {
        self.sim.armies[self.who as usize].list[j as usize].valid
    }
    fn army_muster(&self, j: i32) -> (i32, i32) {
        let m = self.sim.armies[self.who as usize].list[j as usize].muster;
        (m.x, m.y)
    }

    fn cell_flags(&self, x: i32, y: i32) -> i32 {
        i32::from(self.sim.world.cell_data(Cell { x, y }).flags)
    }
    fn cell_land(&self, x: i32, y: i32) -> i32 {
        i32::from(self.sim.world.cell_data(Cell { x, y }).land)
    }
    fn cell_region(&self, x: i32, y: i32) -> i32 {
        self.sim
            .world
            .region_of(Cell { x, y })
            .map_or(-1, i32::from)
    }
    fn cell_owner(&self, x: i32, y: i32) -> i32 {
        match self.sim.world.owner(Cell { x, y }) {
            Owner::None => -1,
            Owner::Ambiguous => -2,
            Owner::Player(p) => i32::from(p),
        }
    }
    fn move_rate(&self, _class: i32) -> i32 {
        Sim::LAND_MOVE_RATE
    }
}
