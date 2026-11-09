//! **`Army::find_besieged_city@006f58b0`'s steps** (item 4031's cut from
//! `army.rs`, `docs/ARMY.md` §10), as a walk over a [`BesiegedView`]:
//! [`crate::Sim::find_besieged_city`] is this walk over [`BesiegedSim`], and
//! the sweep's port is this walk over the emulated original's windows.

use crate::ai::army::{Army, dist};
use crate::combat::Obj;
use crate::movement::{Angle, find_angle};
use crate::territory::attrition::Domain;
use crate::world::{Pos, UNITS_PER_CELL};
use crate::{Player, Sim};

/// What `Army::find_besieged_city@006f58b0` reads and writes, one query a
/// read of the original: the sweep (`sweep/army_find_besieged_city.rs`)
/// answers them from the emulated original's windows and [`BesiegedSim`]
/// from the simulation. Leaders and cities are the original's indices — the
/// leader `j`, and the `k`th city of that leader's list — and an army is
/// the `s`th of a leader's sixteen.
pub(crate) trait BesiegedView {
    /// The leader loop's bound (the original's is the table's eight).
    fn leaders(&self) -> i32;
    /// The calling army's leader (the short at `+0x94`).
    fn who(&self) -> i32;
    /// The calling army's own slot (the short at `+2`).
    fn slot(&self) -> i32;
    fn navy(&self) -> bool;
    fn reg(&self) -> i32;
    fn pos(&self) -> Pos;
    /// `leaders.list[j].flags & 1`.
    fn leader_active(&mut self, j: i32) -> bool;
    /// `j` is the caller's own id or both diplomacy rows hold 2.
    fn friendly(&mut self, j: i32) -> bool;
    /// The caller's leader's city count: the loop bound for **every**
    /// ally's list (`docs/ARMY.md` §18).
    fn city_bound(&self) -> i32;
    /// The city's flag word: bit 0 active, bit 1 attacked.
    fn city_flags(&mut self, j: i32, k: i32) -> i32;
    fn city_reg(&self, j: i32, k: i32) -> i32;
    fn is_coast(&mut self, reg: i32, city_reg: i32) -> bool;
    /// The type id of the city's building (`0x19f` town, `0x1a0`
    /// metropolis, `0x213` forbidden city).
    fn city_type(&mut self, j: i32, k: i32) -> i32;
    /// `find_unit(city point, SEARCH_ENEMY, who, 0x1200, FILTER_DOMAIN 1)`
    /// found one.
    fn enemy_ship_near(&mut self, j: i32, k: i32) -> bool;
    fn army_valid(&self, j: i32, s: i32) -> bool;
    fn army_in_city(&self, j: i32, s: i32, k: i32) -> bool;
    fn army_standard(&self, j: i32, s: i32) -> i32;
    fn city_pos(&self, j: i32, k: i32) -> Pos;
    fn world_cells(&self) -> (i32, i32);
    fn find_muster_spot(&mut self, j: i32, k: i32);
    /// The muster cell, as the army holds it now.
    fn muster_cell(&self) -> (i32, i32);
    /// `muster = cell(pos)` (the original reads its `div_3_table`).
    fn set_muster_from_pos(&mut self);
    fn set_muster_angle(&mut self, a: Angle);
    fn find_angle(&mut self, dx: i32, dy: i32) -> Angle;
    fn set_pos(&mut self, p: Pos);
    /// `target = (the city's building, j)`.
    fn set_target(&mut self, j: i32, k: i32);
}

/// `Army::find_besieged_city@006f58b0` (§10) over a [`BesiegedView`]: the
/// attacked city of mine or an ally's, in my region, that scores best —
/// arithmetic wrapping where the original's 32-bit registers do.
pub(crate) fn find_besieged_city_steps(v: &mut impl BesiegedView) -> bool {
    let (me, slot, navy, reg) = (v.who(), v.slot(), v.navy(), v.reg());
    let mut best = -99_999_999;
    let mut found: Option<(i32, i32)> = None;
    for j in 0..v.leaders() {
        if !v.leader_active(j) || !v.friendly(j) {
            continue;
        }
        for k in 0..v.city_bound() {
            let flags = v.city_flags(j, k);
            if flags & 1 == 0 {
                continue;
            }
            let creg = v.city_reg(j, k);
            let here = if navy {
                v.is_coast(reg, creg)
            } else {
                creg == reg
            };
            if !here || flags & 2 == 0 {
                continue;
            }
            let mut val = 1000
                * match v.city_type(j, k) {
                    0x19f => 2,
                    0x1a0 | 0x213 => 3,
                    _ => 1,
                };
            if navy && !v.enemy_ship_near(j, k) {
                val /= 4;
            }
            if j == me {
                val *= 10;
            }
            for s in 0..16 {
                if v.army_valid(j, s)
                    && (s != slot || j != me)
                    && v.army_in_city(j, s, k)
                    && 4 < v.army_standard(j, s)
                {
                    val /= 4;
                }
            }
            val /= dist(v.pos(), v.city_pos(j, k)) / UNITS_PER_CELL + 1;
            if best <= val {
                best = val;
                found = Some((j, k));
            }
        }
    }
    let Some((j, k)) = found else { return false };
    if navy {
        v.find_muster_spot(j, k);
        let (mx, my) = v.muster_cell();
        v.set_pos(Pos::new(
            mx.wrapping_mul(UNITS_PER_CELL)
                .wrapping_add(UNITS_PER_CELL / 2),
            my.wrapping_mul(UNITS_PER_CELL)
                .wrapping_add(UNITS_PER_CELL / 2),
        ));
        return true;
    }
    let c = v.city_pos(j, k);
    let (mx, my) = v.muster_cell();
    let a = v.find_angle(
        c.x.wrapping_sub(mx.wrapping_mul(UNITS_PER_CELL))
            .wrapping_sub(UNITS_PER_CELL / 2),
        c.y.wrapping_sub(my.wrapping_mul(UNITS_PER_CELL))
            .wrapping_sub(UNITS_PER_CELL / 2),
    );
    v.set_muster_angle(a);
    v.set_muster_from_pos();
    let (w, h) = v.world_cells();
    // Clamped as the original does, the lower bound first and the upper after
    // (a world of no width would leave -1).
    let mut x = c.x;
    let mut y = c.y.wrapping_add(UNITS_PER_CELL);
    if x < 0 {
        x = 0;
    }
    if y < 0 {
        y = 0;
    }
    let (xmax, ymax) = (
        w.wrapping_mul(UNITS_PER_CELL),
        h.wrapping_mul(UNITS_PER_CELL),
    );
    if xmax <= x {
        x = xmax.wrapping_sub(1);
    }
    if ymax <= y {
        y = ymax.wrapping_sub(1);
    }
    v.set_pos(Pos::new(x, y));
    v.set_target(j, k);
    true
}

/// [`BesiegedView`] over the simulation. A leader's `k`th city is the `k`th
/// of the cities it owns, dead ones included (the original's list keeps a
/// city after it falls, its flag word cleared).
pub(crate) struct BesiegedSim<'a> {
    pub(crate) sim: &'a mut Sim,
    pub(crate) who: Player,
    pub(crate) slot: usize,
}

impl BesiegedSim<'_> {
    fn army(&self) -> &Army {
        &self.sim.ai.armies[self.who as usize].list[self.slot]
    }
    fn nth_city(&self, j: i32, k: i32) -> Option<usize> {
        self.sim
            .cities
            .cities
            .iter()
            .enumerate()
            .filter(|(_, c)| c.owner == j as Player)
            .map(|(i, _)| i)
            .nth(usize::try_from(k).ok()?)
    }
}

impl BesiegedView for BesiegedSim<'_> {
    fn leaders(&self) -> i32 {
        self.sim.territory.players.len() as i32
    }
    fn who(&self) -> i32 {
        i32::from(self.who)
    }
    fn slot(&self) -> i32 {
        self.slot as i32
    }
    fn navy(&self) -> bool {
        self.army().navy
    }
    fn reg(&self) -> i32 {
        self.army().reg.map_or(-1, i32::from)
    }
    fn pos(&self) -> Pos {
        self.army().pos
    }
    fn leader_active(&mut self, j: i32) -> bool {
        !self.sim.leaders.defeated[j as usize]
    }
    fn friendly(&mut self, j: i32) -> bool {
        self.sim.is_ally(self.who, j as Player)
    }
    fn city_bound(&self) -> i32 {
        self.sim
            .cities
            .cities
            .iter()
            .filter(|c| c.owner == self.who)
            .count() as i32
    }
    fn city_flags(&mut self, j: i32, k: i32) -> i32 {
        self.nth_city(j, k).map_or(0, |c| {
            let cd = &self.sim.cities.cities[c];
            i32::from(cd.alive) | i32::from(cd.no_heal) << 1
        })
    }
    fn city_reg(&self, j: i32, k: i32) -> i32 {
        self.nth_city(j, k)
            .and_then(|c| self.sim.cities.cities[c].reg)
            .map_or(-1, i32::from)
    }
    fn is_coast(&mut self, reg: i32, city_reg: i32) -> bool {
        match (u16::try_from(reg), u16::try_from(city_reg)) {
            (Ok(r), Ok(cr)) => self.sim.world.is_coast(r, cr),
            _ => false,
        }
    }
    fn city_type(&mut self, j: i32, k: i32) -> i32 {
        match self.nth_city(j, k).map_or(1, |c| self.sim.city_level_of(c)) {
            2 => 0x19f,
            3 => 0x1a0,
            _ => 0,
        }
    }
    fn enemy_ship_near(&mut self, j: i32, k: i32) -> bool {
        let Some(c) = self.nth_city(j, k) else {
            return false;
        };
        let at = self.sim.cities.cities[c].pos;
        self.sim.units.iter().any(|u| {
            u.alive()
                && u.on_map
                && !u.is_gaia()
                && u.kind.domain == Domain::Sea
                && self.sim.is_enemy(self.who, u.owner)
                && dist(u.pos, at) <= 0x1200
        })
    }
    fn army_valid(&self, j: i32, s: i32) -> bool {
        self.sim.ai.armies[j as usize].list[s as usize].valid
    }
    fn army_in_city(&self, j: i32, s: i32, k: i32) -> bool {
        let city = self.sim.ai.armies[j as usize].list[s as usize].city;
        city.is_some() && city == self.nth_city(j, k)
    }
    fn army_standard(&self, j: i32, s: i32) -> i32 {
        self.sim.ai.armies[j as usize].list[s as usize].num_standard
    }
    fn city_pos(&self, j: i32, k: i32) -> Pos {
        self.sim.cities.cities[self.nth_city(j, k).expect("a found city")].pos
    }
    fn world_cells(&self) -> (i32, i32) {
        (self.sim.world.width(), self.sim.world.height())
    }
    fn find_muster_spot(&mut self, j: i32, k: i32) {
        let c = self.nth_city(j, k).expect("a found city");
        let target = Obj::Building(self.sim.cities.cities[c].building);
        self.sim
            .find_muster_spot(self.who, self.slot, target, false);
    }
    fn muster_cell(&self) -> (i32, i32) {
        let m = self.army().muster;
        (m.x, m.y)
    }
    fn set_muster_from_pos(&mut self) {
        let a = &mut self.sim.ai.armies[self.who as usize].list[self.slot];
        a.muster = a.pos.cell();
    }
    fn set_muster_angle(&mut self, angle: Angle) {
        self.sim.ai.armies[self.who as usize].list[self.slot].muster_angle = angle;
    }
    fn find_angle(&mut self, dx: i32, dy: i32) -> Angle {
        find_angle(dx, dy)
    }
    fn set_pos(&mut self, p: Pos) {
        self.sim.ai.armies[self.who as usize].list[self.slot].pos = p;
    }
    fn set_target(&mut self, j: i32, k: i32) {
        let c = self.nth_city(j, k).expect("a found city");
        let b = self.sim.cities.cities[c].building;
        self.sim.ai.armies[self.who as usize].list[self.slot].target = Some(Obj::Building(b));
    }
}
