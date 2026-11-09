//! Armies — the AI's military state machine (`docs/ARMY.md`).
//!
//! Sixteen slots per leader ([`Armies`]), each an [`Army`] record that runs
//! every 256 frames on a slot-and-owner phase: it recounts, disbands or
//! merges, walks its muster point halfway to its target, and dispatches
//! the state its `status` bits name. The document's section numbers are
//! cited on every function that carries one.
//!
//! What is modelled is the **record and the decisions**: the pool (§2), the
//! counts from the units (§3.3), who joins (§4), the cadence (§5), the
//! tick (§6), mustering and its release (§7), the target validation and
//! retarget (§9), defending and the besieged-city search (§10), the
//! engagement predicates over the units' orders (§11), `find_target`'s
//! scoring over cities (§12), the muster spot — the chase, the ring
//! search over the loaded map's cells, and the mark (§13) — transporting
//! (`docs/TRANSPORT.md` §8.2) and the `Armies` queries (§15). What is
//! **not** is the formation layer's own seams (`docs/GROUPS.md` §12);
//! every `Group::action_*` this module reaches is issued, and
//! `add_to_army`'s walk to the army's first member (§4.3) with it. The
//! rest of the stand-ins are in [`seams`].

use crate::ai_load::{role, uflags2};
use crate::attrition::Domain;
use crate::build::{self, Ident};
use crate::combat::{Obj, Stance};
use crate::movement::{self, Angle, find_angle};
use crate::orders::{MoveKind, QueuePos};
use crate::world::{Cell, Pos, Terrain, UNITS_PER_CELL, vector_dist};
#[cfg(test)]
use crate::world::{Owner, cell};
use crate::{Player, Sim};

/// Where this module knowingly stands in for the original, in one list.
///
/// | seam | stands in for | what it costs |
/// | --- | --- | --- |
/// | `find_target`'s forts | §12's second scan | never a fort target |
/// | `pop_issues`, `wonderwin_timer`, `score`, `num_wonders`, `GLOBAL_GOVERNMENT_BONUS`, `weak[]`/`strong[]`, the tribute period | leader and city fields the sim does not keep | the multipliers they gate are ×1; a leader at peace is never a target |
/// | `is(SUPPLYWAGON)` | the lineage test behind `num_standard` and the caps | `unit_flags2 & 0x40` without `0x20` — the supply-or-hero bit less the generals; a government patriot `FROM General` (The Despot) carries both and is not counted, as the lineage would not count it. The *joiner's* arm in the caps is `is_supply` alone (`army_admits`, §213 of `docs/AI.md`) |
/// | `leader_flags & 8`, `leader_flags2 & 8` | the two stop bits (§18) | never set |
/// | `Region::flags & 8` | `go_here`'s resource-region bit | bit 1 never set; `do_transporting` does not read it |
pub mod seams {}

pub const SLOTS: usize = 16;

/// `TypeIndex` `0x119`, `AAGUN`: `Army::normalize` takes it off the
/// standard line.
const AAGUN: crate::tech::TypeId = 0x119;
/// `TypeIndex` `0x3f`, `SUPPLYWAGON`: `find_target`'s weak test asks
/// `type_avail` of it.
#[cfg(test)]
const SUPPLYWAGON: crate::tech::TypeId = 0x3f;
/// `TypeIndex` `0x36`, the General — the lineage `Army::process` counts
/// before its generals' turn; a government patriot `FROM General` is one.
const GENERAL: crate::tech::TypeId = 0x36;
/// `TypeIndex` `0x45`, the Scout — the lineage `Army::process` counts
/// before its scouts' turn.
const SCOUT: crate::tech::TypeId = 0x45;

/// `Unit::come_out+0x25ca` — the scout arm's coin, `% 2`.
/// `find_target`'s two draw sites, under the original's own offsets
/// (§12). `+0x410` is the per-leader coin at `6f6dbb` — taken only when
/// `find_aggressive_army` answers nothing — and `+0x7df` the per-candidate
/// `% 200 + 900` at `6f718a`. Both are in the trace's sequence and neither
/// was named until item 317; the draws themselves were always made.
pub const SITE_FIND_TARGET_COIN: &str = "Army::find_target+0x410";
pub const SITE_FIND_TARGET_SCORE: &str = "Army::find_target+0x7df";

pub const SITE_COME_OUT: &str = "Unit::come_out+0x25ca";
/// `Unit::come_out+0x25b0` — the naval-scout arm's coin, `% 3`.
pub const SITE_COME_OUT_BARK: &str = "Unit::come_out+0x25b0";

/// The three `TypeIndex` values [`Sim::come_out_join_army`] compares
/// against **exactly** — `MERCHANT`, `MERCHANTDUTCH`, `FURTRAPPER`. The
/// same three `Unit::think_attack` and the census exclude.
const MERCHANTS: [crate::tech::TypeId; 3] = [0x3d, 0x3e, 0x190];
/// `BARK` — the naval scout, and the lineage test `is(0x143, 0)`.
pub(crate) const BARK: crate::tech::TypeId = 0x143;
/// `SPY` — the lineage test `is(0x3a, 0)`.
const SPY: crate::tech::TypeId = 0x3a;

/// `ArmyData::status` bits (§1).
pub mod status {
    pub const MUSTERING: i32 = 0x01;
    pub const MARCHING: i32 = 0x02;
    pub const MARCHED: i32 = 0x04;
    pub const NO_TARGET: i32 = 0x08;
    pub const FORMING: i32 = 0x10;
    pub const DEFENDING: i32 = 0x20;
    pub const TRANSPORTING: i32 = 0x40;
    pub const HURRY: i32 = 0x80;
}

/// One `ArmyData` (§1). Regions and cities are the sim's indices; the
/// target is a sim object; positions are coordinate units and the muster
/// point a cell, as in the original.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Army {
    pub valid: bool,
    pub army: i16,
    pub who: Player,
    pub status: i32,
    pub reg: Option<u16>,
    pub role: i32,
    pub num_units: i32,
    pub num_captains: i32,
    pub num_standard: i32,
    pub num_decoys: i32,
    pub city: Option<usize>,
    pub navy: bool,
    pub human_frame: i32,
    pub hurry: i32,
    pub target: Option<Obj>,
    pub pos: Pos,
    pub angle: Angle,
    pub rally_dist: i32,
    pub muster: Cell,
    pub muster_angle: Angle,
    /// The one group's members, in join order (§3.2). `num_groups` is 1
    /// while this is non-empty.
    pub units: Vec<usize>,
    /// The persistent half of that group's `GroupData` (`docs/GROUPS.md`
    /// §1). The army's `list[16]` of group ids collapses to this one.
    pub group: crate::group::GroupState,
}

impl Army {
    fn empty(slot: usize, who: Player) -> Army {
        Army {
            valid: false,
            army: slot as i16,
            who,
            status: 0,
            reg: None,
            role: 0,
            num_units: 0,
            num_captains: 0,
            num_standard: 0,
            num_decoys: 0,
            city: None,
            navy: false,
            human_frame: 0,
            hurry: 0,
            target: None,
            pos: Pos::new(-1, -1),
            angle: Angle(0),
            rally_dist: 0,
            muster: Cell { x: -1, y: -1 },
            muster_angle: Angle(0),
            units: Vec::new(),
            group: crate::group::GroupState::default(),
        }
    }

    /// The dump's `num_groups`.
    pub fn num_groups(&self) -> i32 {
        i32::from(!self.units.is_empty())
    }

    /// The `s` of §5's `(frame + s) % 256`.
    pub fn phase(&self) -> i64 {
        (i64::from(self.army) + i64::from(self.who) * 2) * 2
    }

    /// Whether this army ticks on `frame` (§5): the 256-frame gate.
    pub fn ticks_on(&self, frame: i64) -> bool {
        (frame + self.phase()) % 256 == 0
    }
}

/// A leader's sixteen slots (§2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Armies {
    pub list: Vec<Army>,
}

impl Armies {
    pub fn new(who: Player) -> Armies {
        Armies {
            list: (0..SLOTS).map(|s| Army::empty(s, who)).collect(),
        }
    }

    /// The valid slots, in order.
    pub fn valid(&self) -> impl Iterator<Item = (usize, &Army)> {
        self.list.iter().enumerate().filter(|(_, a)| a.valid)
    }
}

/// `Armies::find_aggressive_army@006f2e10`'s test of one of the sixteen
/// slots: a live army (`+0` nonzero) of two or more captains (`+0x14`), not
/// mustering (`+4 & 1`), whose cell's owner byte is not `who`. Item 2247's
/// sweep holds it.
pub(crate) fn aggressive(valid: bool, captains: i32, status: i32, owner: i32, who: i32) -> bool {
    valid && captains >= 2 && status & status::MUSTERING == 0 && owner != who
}

/// The centre of a cell in coordinate units — `cell × 0x300 + 0x180`.
pub const fn cell_centre(c: Cell) -> Pos {
    Pos::new(
        c.x * UNITS_PER_CELL + UNITS_PER_CELL / 2,
        c.y * UNITS_PER_CELL + UNITS_PER_CELL / 2,
    )
}

/// `project(p, angle, d)` (§8): a polar step along `angle` —
/// `x + sin(angle)·d`, `y − cos(angle)·d` (`docs/MOVEMENT.md`).
pub fn step_along(p: Pos, angle: Angle, d: i32) -> Pos {
    Pos::new(
        p.x + movement::sin_component(angle, d),
        p.y - movement::cos_component(angle, d),
    )
}

/// `Army::march_to_target`'s formation origin (§9, §23): the army's point
/// one cell **behind** it, `muster_angle − 0x80000000`.
pub fn march_origin(pos: Pos, muster_angle: Angle) -> Pos {
    step_along(
        pos,
        Angle(muster_angle.0.wrapping_sub(i32::MIN)),
        UNITS_PER_CELL,
    )
}

/// `|dx|, |dy|` through `vector_dist`.
const fn dist(a: Pos, b: Pos) -> i32 {
    vector_dist((a.x - b.x).abs(), (a.y - b.y).abs())
}

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
struct BesiegedSim<'a> {
    sim: &'a mut Sim,
    who: Player,
    slot: usize,
}

impl BesiegedSim<'_> {
    fn army(&self) -> &Army {
        &self.sim.armies[self.who as usize].list[self.slot]
    }
    fn nth_city(&self, j: i32, k: i32) -> Option<usize> {
        self.sim
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
        self.sim.players.len() as i32
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
        !self.sim.defeated[j as usize]
    }
    fn friendly(&mut self, j: i32) -> bool {
        self.sim.is_ally(self.who, j as Player)
    }
    fn city_bound(&self) -> i32 {
        self.sim
            .cities
            .iter()
            .filter(|c| c.owner == self.who)
            .count() as i32
    }
    fn city_flags(&mut self, j: i32, k: i32) -> i32 {
        self.nth_city(j, k).map_or(0, |c| {
            let cd = &self.sim.cities[c];
            i32::from(cd.alive) | i32::from(cd.no_heal) << 1
        })
    }
    fn city_reg(&self, j: i32, k: i32) -> i32 {
        self.nth_city(j, k)
            .and_then(|c| self.sim.cities[c].reg)
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
        let at = self.sim.cities[c].pos;
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
        self.sim.armies[j as usize].list[s as usize].valid
    }
    fn army_in_city(&self, j: i32, s: i32, k: i32) -> bool {
        let city = self.sim.armies[j as usize].list[s as usize].city;
        city.is_some() && city == self.nth_city(j, k)
    }
    fn army_standard(&self, j: i32, s: i32) -> i32 {
        self.sim.armies[j as usize].list[s as usize].num_standard
    }
    fn city_pos(&self, j: i32, k: i32) -> Pos {
        self.sim.cities[self.nth_city(j, k).expect("a found city")].pos
    }
    fn world_cells(&self) -> (i32, i32) {
        (self.sim.world.width(), self.sim.world.height())
    }
    fn find_muster_spot(&mut self, j: i32, k: i32) {
        let c = self.nth_city(j, k).expect("a found city");
        let target = Obj::Building(self.sim.cities[c].building);
        self.sim
            .find_muster_spot(self.who, self.slot, target, false);
    }
    fn muster_cell(&self) -> (i32, i32) {
        let m = self.army().muster;
        (m.x, m.y)
    }
    fn set_muster_from_pos(&mut self) {
        let a = &mut self.sim.armies[self.who as usize].list[self.slot];
        a.muster = a.pos.cell();
    }
    fn set_muster_angle(&mut self, angle: Angle) {
        self.sim.armies[self.who as usize].list[self.slot].muster_angle = angle;
    }
    fn find_angle(&mut self, dx: i32, dy: i32) -> Angle {
        find_angle(dx, dy)
    }
    fn set_pos(&mut self, p: Pos) {
        self.sim.armies[self.who as usize].list[self.slot].pos = p;
    }
    fn set_target(&mut self, j: i32, k: i32) {
        let c = self.nth_city(j, k).expect("a found city");
        let b = self.sim.cities[c].building;
        self.sim.armies[self.who as usize].list[self.slot].target = Some(Obj::Building(b));
    }
}

impl Sim {
    // ---- the pool (§2) ----

    /// `Armies::init_army(who, city)`: the first free slot, else the one
    /// with the fewest units (ties to the later slot), re-initialised.
    /// Never fails (audit B.59).
    pub fn init_army(&mut self, who: Player, city: Option<usize>) -> usize {
        let w = who as usize;
        let mut best = 0;
        let mut fewest = 9999;
        let mut slot = None;
        for s in 0..SLOTS {
            let a = &self.armies[w].list[s];
            if !a.valid {
                slot = Some(s);
                break;
            }
            if a.num_units <= fewest {
                fewest = a.num_units;
                best = s;
            }
        }
        let slot = slot.unwrap_or(best);
        self.army_init(who, slot, city);
        slot
    }

    /// `Army::init`: the muster point one cell south of the city (§2).
    fn army_init(&mut self, who: Player, slot: usize, city: Option<usize>) {
        // `Army::init@006f9000` writes only `ArmyData`'s own fields (`list[]`
        // to −1, `num_groups` to 0) and touches no group: when all sixteen
        // slots are valid and the one with fewest units is taken over, the
        // old army's group stays in the pool with its members still naming
        // it (twenty-fourth pass, group 4; A5 #10). The crate's pool record
        // carries no `army` back-pointer, so the seat is the orphan's, the
        // same one `close_army` leaves — without its halt.
        if self.armies[who as usize].list[slot].valid {
            let old = &mut self.armies[who as usize].list[slot];
            let list = std::mem::take(&mut old.units);
            let state = std::mem::take(&mut old.group);
            if state.pool.is_some() && list.iter().any(|&u| self.units[u].alive()) {
                self.seat_orphan(who, list, state);
            }
        }
        let mut a = Army::empty(slot, who);
        a.valid = true;
        a.status = status::MUSTERING;
        a.city = city;
        if let Some(c) = city {
            let cd = &self.cities[c];
            a.reg = cd.reg;
            a.pos = Pos::new(cd.pos.x, cd.pos.y + UNITS_PER_CELL);
            a.muster = a.pos.cell();
        }
        self.armies[who as usize].list[slot] = a;
    }

    /// `Army::close@006f8ea0`: `Group::action_halt(g, 0)` on the one group
    /// — the units stop where they are (`docs/GROUPS.md` §7) — then the
    /// slot is freed.
    ///
    /// **The group is not freed** (`docs/ARMY.md` §22). `close` writes the
    /// group's `army` (`+8`) to −1 and halts it, and nothing else: the
    /// record, its pool index and its list stay in the pool, and every
    /// member's `+0x80` still names it. So the group moves to a
    /// [`crate::group::Pushed`] seat on the same index, orphaned, and the
    /// next `get_open_slot` passes it by while a member still points at
    /// it. East Indies' army 1 closed on 18682 from `find_target`, and the
    /// original's group 71 still lists `1/69` and `1/72` on 18933; the
    /// wagon's new army took another slot, and the escort's stale pointers
    /// at 71 are what its collisions read on 18938.
    pub fn close_army(&mut self, who: Player, slot: usize) {
        if !self.armies[who as usize].list[slot].valid {
            return;
        }
        let g = self.army_group(who, slot);
        self.group_action_halt(&g, 0);
        let a = &mut self.armies[who as usize].list[slot];
        a.valid = false;
        a.status = 0;
        a.human_frame = 0;
        let list = std::mem::take(&mut a.units);
        let state = std::mem::take(&mut a.group);
        if state.pool.is_some() && list.iter().any(|&u| self.units[u].alive()) {
            self.seat_orphan(who, list, state);
        }
    }

    // ---- membership and the counts (§3) ----

    /// `UnitData::is_captain@0046ceb0`: **`o_up < 0`** and nothing else —
    /// the head of its own uber squad ([`Unit::captain`]). It is not a
    /// test of where the unit is: `Group::get_num_cap@007145c0` gates on
    /// the object being *active* and calls this through vslot `0xe8`, and
    /// the garrison chain the record prints beside it (`inside_up`,
    /// `ObjectData`) is a different chain from this one (`o_up`,
    /// `UnitData` — `docs/ORACLE.md`, run79).
    ///
    /// It read `on_map && inside.is_none()` until item 261, which made
    /// every figure of a three-figure squad a captain: Great Lakes'
    /// army held six units and counted six, where the original counts
    /// **two** — one per squad — and `release_mustering`'s `n < 5`
    /// therefore kept it mustering (`docs/ARMY.md` §3.3, §7).
    pub(crate) fn is_captain(&self, u: usize) -> bool {
        self.units[u].captain
    }

    /// `is(SUPPLYWAGON)` as the sim can test it (see [`seams`]).
    fn is_supply_wagon(&self, u: usize) -> bool {
        self.units[u].ty.is_some_and(|t| {
            let c = self.unit_types[t].cols;
            c.flag2(uflags2::SUPPLY_OR_HERO) && !c.flag2(uflags2::GENERAL)
        })
    }

    fn is_general(&self, u: usize) -> bool {
        self.units[u]
            .ty
            .is_some_and(|t| self.unit_types[t].cols.flag2(uflags2::GENERAL))
    }

    /// Shrink an army's list to `keep` (a subsequence of it) **and shift the
    /// group record's per-member arrays with it**: `Group::normalize` and
    /// `Group::kill` drop a member's `off`, `curr` and `angles` entries
    /// together with it, so the slot a siege unit was copied into does not
    /// stay behind at an index another member now holds (item 1493: army
    /// 65's `off[31]`, `[61]`, `[62]` on run653 block 14357).
    pub(crate) fn army_keep_units(&mut self, w: usize, slot: usize, keep: Vec<usize>) {
        let a = &mut self.armies[w].list[slot];
        for i in (0..a.units.len()).rev() {
            if keep.contains(&a.units[i]) {
                continue;
            }
            if i < a.group.off.len() {
                a.group.off.remove(i);
            }
            if i < a.group.curr.len() {
                a.group.curr.remove(i);
            }
            if i < a.group.angles.len() {
                a.group.angles.remove(i);
            }
        }
        a.units = keep;
    }

    /// `Army::normalize` (§3.3): drop the dead, recount, and the standard
    /// line — captains less casters, supply wagons, decoys and AA guns.
    pub fn army_normalize(&mut self, who: Player, slot: usize) {
        let w = who as usize;
        // `Group::normalize` on the group first (§3.3): the dead, and a
        // member whose `+0x80` no longer names the group (`docs/GROUPS.md`
        // §23) — `1/64` once the sort of 11512 has cleared it.
        let pool = self.armies[w].list[slot].group.pool;
        let units: Vec<usize> = self.armies[w].list[slot]
            .units
            .iter()
            .copied()
            .filter(|&u| {
                u < self.units.len()
                    && self.units[u].alive()
                    && (pool.is_none() || self.units[u].group_ptr == pool)
            })
            .collect();
        // `Army::normalize@006f9b50`'s counts, each a `GroupData::count`
        // over the live members: `num_decoys` the decoy **captains**
        // (`COUNT_DECOYS`), and the standard line the captains less the
        // casters (`COUNT_CASTERS`, a decoy excluded), the supply wagons
        // (`COUNT_NON_DECOY_TYPE 0x3f`), the decoys, and the AA guns
        // (`COUNT_NON_DECOY_TYPE 0x119`). run346's army 4 holds six decoy
        // squads from 11637 (item 1302).
        let mut captains = 0;
        let mut casters = 0;
        let mut supply = 0;
        let mut decoys = 0;
        let mut aa = 0;
        for &u in &units {
            let captain = self.is_captain(u);
            captains += i32::from(captain);
            if self.units[u].decoy {
                decoys += i32::from(captain);
                continue;
            }
            if self.units[u]
                .ty
                .is_some_and(|t| self.unit_types[t].cols.flag2(uflags2::CASTER))
            {
                casters += 1;
            }
            if self.is_supply_wagon(u) {
                supply += 1;
            }
            if self.unit_line_is(u, AAGUN) {
                aa += 1;
            }
        }
        self.army_keep_units(w, slot, units);
        // `Group::normalize`'s tail (`00711540`, after `find_role`):
        // `speed = new_speed = UnitData::speed(find_leader)`, or 0 — the
        // cap goes back to the leader's own speed on every army normalize,
        // whatever its members reported (`docs/GROUPS.md` §18.1). run679's
        // army `1/0` takes its periodic normalize on 1434, `1434 − 30 + 4 ≡
        // 0 (mod 128)`, and its leader `1/28` lifts the cap from the slow
        // squad's 25 to its own 47 (`docs/AI.md` §161).
        self.seat_set_speed(crate::group::Seat::Army(who, slot));
        let a = &mut self.armies[w].list[slot];
        a.role = 0;
        a.num_units = a.units.len() as i32;
        a.num_captains = captains;
        a.num_decoys = decoys;
        a.num_standard = captains - casters - supply - decoys - aa;
    }

    /// `Army::add_unit(o)@006f9f40` (§3.2): into the one group, if not
    /// already there, and then `Unit::set_group(unit, group, 0)`.
    ///
    /// **An army with no live group pushes one**: the unit is `Group::add`ed
    /// to a fresh group on the stack — `id −1`, so nothing normalizes and
    /// the whole squad comes in, `captain, o_down, …` — and
    /// `push_group(who, it, 1)` gives it a pool slot like any other
    /// (`docs/GROUPS.md` §19).
    ///
    /// **Otherwise the squad joins through the seated group's own
    /// `Group::add`**, and that is not the same thing (`docs/GROUPS.md`
    /// §23). `add(o, who, 0, 0)` climbs to the captain and walks down
    /// `o_down`, but each step opens with `get_num`, which **normalizes a
    /// group of fewer than four** — and a figure whose `+0x80` still names
    /// the group it came out in is dropped by the step that adds the next.
    /// Great Lakes' `1/62`–`1/64` join army 2's `{1/60, 1/61}` on 11424
    /// from their `come_out` group 69, and the list that results is
    /// `{1/60, 1/61, 1/64}`: `set_group` then points all three at 66. The
    /// counts move as the original's do, `num_units` by the list's change
    /// and `num_captains` by one.
    pub fn army_add_unit(&mut self, who: Player, slot: usize, u: usize) {
        if !self.units[u].alive() {
            return;
        }
        let w = who as usize;
        // `Army::member(o, who)@006f8de0` — the **list**, not the pointer.
        if self.armies[w].list[slot].units.contains(&u) {
            return;
        }
        // **The push arm is an army with no group** (`Army::add_unit
        // @006f9f40`: `+0x96 < 1 || list[0] < 0`). An army loses its group
        // only through `Army::normalize@006f9b50`'s `remove_group` on an
        // empty one, and `remove_group@006f8b50` asks the record to name
        // the army. So a group whose members **died** — `Group::normalize`
        // prunes them and writes no link — leaves the army, and the next
        // join pushes (run146's `1/9..1/11` on 771 to 66, their army's 64
        // dead since 735; item 518). A group emptied by `Group::kill`
        // (`clear(-1)`, the link −1) stays listed, and the join goes
        // through the record's own `Group::add` — `get_num`'s normalize at
        // every step of the squad walk prunes the figure added the step
        // before while its `+0x80` still names another group, and only the
        // walk's last figure is listed. Chapter 128's `1/6..1/8` rejoin
        // army 0 on 642 as `[8]` (run1555's `GROUPDATA 64`), not `[6 7 8]`
        // (item 4000, `docs/GOLDEN.md` §190).
        let live = self.armies[w].list[slot]
            .units
            .iter()
            .any(|&m| self.units[m].alive());
        let group = &self.armies[w].list[slot].group;
        let pool = if group.pool.is_none() || (!live && !group.unlinked) {
            // The stack group's `Group::add`: `captain, o_down, …`.
            let mut g = crate::group::Group::stack(who);
            self.group_add(&mut g, u);
            let chain = g.list;
            let last = self.last_group[w];
            let equal = !chain.is_empty() && self.pool_members(who, last) == chain;
            let s = self.pool_slot_for(who, &chain);
            // `push_group`'s `copy_group` into the slot's own record
            // (`docs/GROUPS.md` §30): the army's group **is** the pool
            // record, so it keeps the previous occupant's `order_num`,
            // `facing`, `form`, `form_num`, `new_speed` and `march`, as a
            // pushed group does. An equal group is not copied at all.
            let prev = self.pool_record(who, s).1;
            // The walk's kill half: the group each figure's `+0x80` names
            // loses the squad, and is cleared if that empties it.
            for a in self.kill_from_named(who, &chain, s) {
                if a != slot {
                    self.army_normalize(who, a);
                }
            }
            self.armies[w].list[slot].group = if equal {
                crate::group::GroupState {
                    pool: Some(s),
                    unlinked: false,
                    ..prev
                }
            } else {
                crate::group::GroupState::copied(prev, s, self.frame, chain.len())
            };
            for f in chain {
                let captain = self.is_captain(f);
                let a = &mut self.armies[w].list[slot];
                if a.units.contains(&f) {
                    continue;
                }
                a.units.push(f);
                a.num_units += 1;
                a.num_captains += i32::from(captain);
            }
            s
        } else {
            let seat = crate::group::Seat::Army(who, slot);
            let before = self.armies[w].list[slot].units.len() as i32;
            self.seat_add(seat, u, false, false);
            let a = &mut self.armies[w].list[slot];
            a.num_units += a.units.len() as i32 - before;
            a.num_captains += 1;
            a.group.pool.expect("a live group has a slot")
        };
        self.set_group(u, Some(pool));
    }

    /// `ArmyData::get_unit(k)@006f9df0`: the `k`-th member of the army's
    /// groups, walked in `list` order and skipping building groups. The
    /// simulation's army is one group (§3.2), so this is its `units[k]`.
    pub fn army_get_unit(&self, who: Player, slot: usize, k: usize) -> Option<usize> {
        let a = &self.armies[who as usize].list[slot];
        if k >= a.num_units as usize {
            return None;
        }
        a.units.get(k).copied()
    }

    /// `Object::get_army`: the army holding this unit, if any.
    ///
    /// `Object::get_army@00649d70`: the group the unit's `+0x80` names must
    /// **list** it, and that group must be an army's. A unit its army's
    /// list holds but whose pointer names nothing — Great Lakes' `1/64`
    /// after 11512 — is in no army (`docs/GROUPS.md` §23).
    pub fn army_of(&self, u: usize) -> Option<usize> {
        let s = self.units[u].group_ptr?;
        let who = self.units[u].owner as usize;
        self.armies
            .get(who)?
            .valid()
            .find(|(_, a)| a.group.pool == Some(s) && a.units.contains(&u))
            .map(|(s, _)| s)
    }

    fn army_count(&self, who: Player, slot: usize, f: impl Fn(&Sim, usize) -> bool) -> i32 {
        self.armies[who as usize].list[slot]
            .units
            .iter()
            .filter(|&&u| self.units[u].alive() && f(self, u))
            .count() as i32
    }

    /// `Army::count(COUNT_ATTACK)`: captains whose type has an attack.
    fn army_count_attack(&self, who: Player, slot: usize) -> i32 {
        self.army_count(who, slot, |s, u| {
            s.is_captain(u) && s.attack_of(Obj::Unit(u)) != 0
        })
    }

    /// `Army::count(COUNT_SIEGE)`: a siege type, and **not** a decoy under
    /// computer control — `GroupData::count@00711720`'s
    /// `unit_masks & 0x40001 != 0x40001` (twenty-fourth pass, group 1; the
    /// decoy bit is `1`, the computer-control bit `0x40000`,
    /// [`Sim::ai_driven`]).
    pub(crate) fn army_count_siege(&self, who: Player, slot: usize) -> i32 {
        self.army_count(who, slot, |s, u| {
            s.units[u].ty.is_some_and(|t| s.unit_types[t].combat.siege)
                && !(s.units[u].decoy && s.unit_ai_bit(s.units[u].owner))
        })
    }

    /// `count(NON_DECOY_TYPE, SUPPLYWAGON)`.
    pub(crate) fn army_count_supply(&self, who: Player, slot: usize) -> i32 {
        self.army_count(who, slot, Sim::is_supply_wagon)
    }

    fn army_count_generals(&self, who: Player, slot: usize) -> i32 {
        self.army_count(who, slot, Sim::is_general)
    }

    /// `COUNT_NON_DECOY_TYPE, CATAPHRACT`: the complete lineage (§25),
    /// including upgrades and unique grafts; dead units and decoys do not count.
    pub(crate) fn army_count_cataphracts(&self, who: Player, slot: usize) -> i32 {
        self.army_count(who, slot, |s, u| {
            !s.units[u].decoy
                && s.units[u]
                    .ty
                    .and_then(|t| s.unit_types[t].tree)
                    .is_some_and(|t| {
                        s.tech_tree
                            .roles
                            .cataphract
                            .is_some_and(|c| s.tech_tree.is(t, c, false))
                    })
        })
    }

    /// `COUNT_NON_DECOY_TYPE, HOPLITES`: a decoy is skipped, as the cavalry
    /// term beside it skips one (twenty-fourth pass, group 1).
    pub(crate) fn army_count_hoplites(&self, who: Player, slot: usize) -> i32 {
        self.army_count(who, slot, |s, u| {
            !s.units[u].decoy
                && s.units[u]
                    .ty
                    .is_some_and(|t| s.unit_types[t].cols.is(role::HOPLITE))
        })
    }

    // ---- who joins (§4) ----

    /// `Unit::add_to_army`: pick an army, or seed one at the nearest
    /// friendly city, and join it. Returns the slot.
    pub fn add_to_army(&mut self, u: usize) -> Option<usize> {
        if let Some(s) = self.army_of(u) {
            return Some(s);
        }
        let who = self.units[u].owner;
        let pos = self.units[u].pos;
        let sea = self.units[u]
            .ty
            .is_some_and(|t| self.unit_types[t].combat.domain == Domain::Sea);
        let damaged = self.units[u].health < self.units[u].max_health;
        let slot = if sea {
            self.find_army(who, pos, -1, None).map(|(s, _)| s)
        } else if damaged {
            self.find_army(who, pos, 0x2400, Some(u)).map(|(s, _)| s)
        } else {
            self.find_local_army(who, pos, Some(u)).map(|(s, _)| s)
        };
        let slot = match slot {
            Some(s) => {
                self.army_normalize(who, s);
                s
            }
            None => {
                if sea {
                    return None;
                }
                let city = self.nearest_friendly_city(who, pos)?;
                self.init_army(who, Some(city))
            }
        };
        // **The army's group is a pool index, and the record on it may be
        // another group's** (`docs/GOLDEN.md` §77): an army whose own seat lists no live
        // member, and whose index a later push took, reads that pushed
        // group as its own — the count, `get_unit(0)` and the join. A
        // joiner the record already lists is `Army::member` there and
        // the original's `add_unit` does nothing; this crate keeps the
        // arm below for it, which seats the record on the army and
        // agrees with run460 (golden chapter forty-two) through its end.
        if let Some(p) = self.army_pool_record(who, slot)
            && !self.pushed[p].list.contains(&u)
        {
            let s = self.armies[who as usize].list[slot].group.pool;
            let first = self.pushed[p]
                .list
                .iter()
                .copied()
                .find(|&m| self.units[m].alive() && self.units[m].group_ptr == s);
            if let Some(first) = first {
                self.go_to_unit(u, first);
            }
            self.seat_add(crate::group::Seat::Pushed(p), u, false, false);
            self.set_group(u, s);
            return Some(slot);
        }
        // **The walk to the army** (§4.3), between the pick and the join,
        // and only on the arm that *found* an army: the `init_army` arm
        // jumps straight to the add (`005f7891`). `ArmyData::get_unit(0)`
        // is the army's first member — the unit every later joiner is sent
        // to stand beside.
        if self.armies[who as usize].list[slot].num_units != 0
            && let Some(first) = self.army_get_unit(who, slot, 0)
            && self.units[first].alive()
        {
            self.go_to_unit(u, first);
        }
        self.army_add_unit(who, slot, u);
        Some(slot)
    }

    /// **The pool record an army's index holds when it is not the army's
    /// own** (`docs/GOLDEN.md` §77). `Army` keeps its groups as pool **indices**
    /// (`list[16]`, §3.1), and nothing tells it when `push_group` kills its
    /// members out of the group (§3.4), when the pool frees the emptied
    /// record, or when a later push takes the index: `remove_group` refuses
    /// a group whose `army` no longer names it, and the push writes −1
    /// there. So `normalize` (§3.3), `get_unit(0)` and `add_unit`'s
    /// `Group::add` on `list[0]` all read whatever group stands on the
    /// index. This crate keeps the army's record on the army; the answer
    /// is the [`crate::group::Pushed`] entry of the same player on the
    /// army's index with a live member, when the army's own seat lists
    /// none.
    fn army_pool_record(&self, who: Player, slot: usize) -> Option<usize> {
        let a = &self.armies[who as usize].list[slot];
        let s = a.group.pool?;
        let mine = |m: usize| self.units[m].alive() && self.units[m].group_ptr == Some(s);
        if a.units.iter().any(|&m| mine(m)) {
            return None;
        }
        self.pushed.iter().position(|p| {
            p.who == who && p.state.pool == Some(s) && p.list.iter().any(|&m| mine(m))
        })
    }

    /// `Unit::go_to_unit(o, who)@005f78c0` — mark the squad, then walk it
    /// to another unit if it is far enough away (§4.3).
    ///
    /// The mask walk runs from the **captain** and down the `o_down`
    /// chain, and it runs whatever the distance says; the move is
    /// `vector_dist(target − me) > 0x480` alone. The bit it sets is
    /// `unit_masks & 4` (`5f7933`), which §6.6 step 6 of
    /// `docs/GROUPS.md` reads as "no `GroupMoveOrder`" — a seam here,
    /// because [`Sim::go_to`]'s group has no army and this crate's own
    /// gate refuses one for that reason already.
    fn go_to_unit(&mut self, u: usize, target: usize) {
        // The mark is **unconditional** and it is the whole squad's: the
        // loop at `5f78ec` starts at this unit's captain and walks
        // `o_down`, setting `unit_masks |= 4` on each, and it runs above
        // the distance test — so a squad already close enough not to walk
        // is marked all the same. [`sim::Unit::in_danger`].
        for f in self.squad_of(self.captain_of(u)) {
            self.units[f].in_danger = true;
        }
        let to = self.units[target].pos;
        let d = vector_dist(to.x - self.units[u].pos.x, to.y - self.units[u].pos.y);
        if d > 0x480 {
            self.go_to(u, to, MoveKind::AttackTo, 0, 0x300);
        }
    }

    /// `Unit::go_to(x, y, orders, min, max)@005f7a50` — a spot near a
    /// point, and the unit's own squad walked to it as a group (§4.3).
    ///
    /// `find_nearby_spot` is asked with the caller's radii, **step 0** (so
    /// the rings are an eighth of the span apart) and the bias angle
    /// `0x55555555`, a third of a turn; a refusal issues nothing at all.
    /// The group is built on the stack from this unit — `Group::add` takes
    /// the whole squad — pushed with `force = 1`, and moved with
    /// `QUEUE_NEW`, `set_angle 0`, `angle 0`, `action 0`, `form/width −1`.
    fn go_to(&mut self, u: usize, to: Pos, kind: MoveKind, min: i32, max: i32) {
        let Some(spot) = self.find_nearby_spot_squad(u, to, min, max, 0, Angle(0x5555_5555)) else {
            return;
        };
        let mut g = crate::group::Group::stack(self.units[u].owner);
        self.group_add(&mut g, u);
        if !self.push_group(&mut g, true) {
            return;
        }
        self.group_action_move_to(&g, spot, QueuePos::New, false, Angle(0), kind, false);
    }

    /// `ObjectsData::find_city(SEARCH_FRIENDLY, flag 0x200, FILTER_ALL)`:
    /// the nearest live city of `who` in the point's region.
    fn nearest_friendly_city(&self, who: Player, pos: Pos) -> Option<usize> {
        let reg = self.world.tregion(pos.tile());
        let mut best: Option<(i32, usize)> = None;
        for c in self.cities_of(who) {
            if self.cities[c].reg != reg {
                continue;
            }
            let d = dist(pos, self.cities[c].pos);
            if best.is_none_or(|(bd, _)| d < bd) {
                best = Some((d, c));
            }
        }
        best.map(|(_, c)| c)
    }

    /// The idle-think hook (§4): an AI-owned unit at the end of
    /// `Unit::think` joins an army **only if it is a supply wagon or a
    /// hero**.
    ///
    /// The tail at `005f7615` is three tests and no more
    /// (`docs/SCOUT.md` §2 transcribes the same listing):
    ///
    /// ```text
    /// if (!is_supply(this) && !is_hero(this)) {
    ///     if ((type->role & 0x10) == 0 && !is(SPY, 0)) return;
    ///     if (get_army() < 0) think_scout(this, 0);
    ///     return;
    /// }
    /// add_to_army(this);
    /// ```
    ///
    /// `is_supply` is `unit_flags2 & 0x40`
    /// ([`uflags2::SUPPLY_OR_HERO`]) and `is_hero` is `& 0x20`
    /// ([`uflags2::GENERAL`]); **everything else returns without an
    /// army**. `think_attack@005f5a80:155`'s own `add_to_army` is a
    /// different site, behind that function's city search, and is not
    /// modelled — no capture reaches it (§17).
    ///
    /// This used to also join "an attacker that is not a scout or a
    /// caravan", which is `docs/ARMY.md` §4's prose for `think_attack`
    /// grafted onto the wrong site. It cost run10's AI two citizens: `1/6`
    /// joined army 0 on frame 99 and `1/7` on 205, and the army's tick at
    /// **252** — leader 1's army 0 is `frame ≡ 252 (mod 256)`, §5 — sent
    /// both on a siege attack while the original had them chopping wood.
    /// The original never calls `add_to_army` at all in that game
    /// (run33's coverage: `Unit::add_to_army@005f7740` and
    /// `Army::add_unit@006f9f40` are on the never-entered list over all
    /// 1,850 frames), and every citizen's dumped `group` is −1 from the
    /// frame it appears.
    pub(crate) fn think_join_army(&mut self, u: usize) {
        let unit = &self.units[u];
        let w = unit.owner as usize;
        if self.nation.get(w).is_none_or(|n| n.human) || self.defeated[w] {
            return;
        }
        let Some(t) = unit.ty else { return };
        let cols = self.unit_types[t].cols;
        if cols.flag2(uflags2::SUPPLY_OR_HERO) || cols.flag2(uflags2::GENERAL) {
            self.add_to_army(u);
        }
    }

    /// **`think_attack`'s tail** (`think_attack@005f5a80:190`–`239`,
    /// listing `5f5e75`–`5f5fd8`; item 3369, `docs/ARMY.md` §4.3,
    /// `docs/AI.md` §285): an AI-driven unit that found no melee target,
    /// joined no army and is **not standing on its own territory** walks to
    /// its best city of its own in its own land region.
    ///
    /// ```text
    /// if !manual && !home && army < 0:
    ///     region = get_tregion(tile(unit)); best = 99999999; idx = -1
    ///     for i < leader.num_cities:                  # +0x408, its own list
    ///         c = cities[who][i]
    ///         if c.flags & 1 and region(cell(c)) == region:
    ///             w = max(1, danger[who][half-cell(c)])  # world+0x13c
    ///             d = vector_dist(c − unit) / w
    ///             if c.flags & 2: d /= 2              # CITY_UNDER_ATTACK
    ///             if d < best: best = d, idx = i      # strict, first wins
    ///     if idx >= 0: go_to_city(idx, ATTACK_TO)
    /// ```
    ///
    /// `manual` is the head's (not AI-driven, or a leader without
    /// `leader_flags & 2`, or `leader_flags2 & 8`), read as this crate
    /// reads it everywhere, `!ai_driven || defeated`; `home` is the
    /// own-territory arm [`Self::think_attack_join_army`] tests too.
    pub(crate) fn think_attack_tail(&mut self, u: usize) {
        if let Some(c) = self.think_attack_city(u) {
            self.go_to_city(u, c);
        }
    }

    /// The tail's choice, apart from its walk: the city
    /// [`Self::think_attack_tail`] sends `u` to, or none.
    fn think_attack_city(&self, u: usize) -> Option<usize> {
        let who = self.units[u].owner;
        if !self.ai_driven(who) || self.defeated[who as usize] || self.think_attack_home(u) {
            return None;
        }
        let p = self.units[u].pos;
        let reg = self.world.tregion(p.tile());
        let mut best = 99_999_999;
        let mut idx = None;
        for c in self.cities_of(who) {
            let city = &self.cities[c];
            if city.reg != reg {
                continue;
            }
            let w = self.world.danger_at(who, city.pos).max(1);
            let mut d = vector_dist((p.x - city.pos.x).abs(), (p.y - city.pos.y).abs()) / w;
            if city.no_heal {
                d /= 2;
            }
            if d < best {
                best = d;
                idx = Some(c);
            }
        }
        idx
    }

    /// `Unit::go_to_city(idx, who, orders)@005f79c0` with `orders = 2`,
    /// `ATTACK_TO`: the city's **building** (`objects[who][city +8]`), and
    /// a walk only when it is farther than `0x480` — `go_to(x, y, orders,
    /// 0x240, 0)` (listing `5f7a27`–`5f7a35`; `go_to_unit` pushes `0,
    /// 0x300` in the same two slots).
    fn go_to_city(&mut self, u: usize, c: usize) {
        let to = self.buildings[self.cities[c].building].pos;
        let at = self.units[u].pos;
        if vector_dist(to.x - at.x, to.y - at.y) > 0x480 {
            self.go_to(u, to, MoveKind::AttackTo, 0x240, 0);
        }
    }

    /// `think_attack`'s own-territory test: the tile is inside a city's
    /// radius (`tile & 0x100`) and the cell's owner is the unit's.
    fn think_attack_home(&self, u: usize) -> bool {
        let who = self.units[u].owner;
        let p = self.units[u].pos;
        self.world.tile_mask(p.tile()) & crate::world::tile::CITY_RADIUS != 0
            && self.world.owner_at(p) == crate::world::Owner::Player(who)
    }

    /// **`think_attack` for a Dutch merchant** (item 3369): the lineage's
    /// leg of `Unit::think`'s auto-attack disjunction reaches the same
    /// function as a soldier, and of it a merchant can take only two
    /// things — its packed arm is skipped for `is(0x3e, 1)` and its army
    /// join for the three merchant ids (`5f5d53`–`5f5d94`). So: the melee
    /// search with radius −1, as the soldier's, and when that finds
    /// nothing, the tail's walk to a city ([`Self::think_attack_tail`]).
    /// `true` is the function's `return 1`, a target found.
    ///
    /// SEAM: the head's `get_combat_stance() > 4` refusal and the
    /// game-semaphore radius `0x2d00` are not read, as the soldier's arm
    /// does not read them.
    pub(crate) fn think_attack_merchant(&mut self, u: usize) -> bool {
        let unit = &self.units[u];
        if unit.combat.stance != Stance::HoldFire
            && unit.combat.target.is_none()
            && let Some(t) = self.find_melee_target(u, -1)
        {
            self.nearby_add(u, t);
            if matches!(t, Obj::Unit(_))
                && let Some(seat) = self.seat_of(u)
                && self.seat_list(seat).contains(&u)
            {
                self.group_target_opportunity(seat, t, u, true);
            }
            return true;
        }
        self.think_attack_tail(u);
        false
    }

    /// `Unit::think_attack@005f5a80`'s head — the **sixth** caller of
    /// [`Sim::add_to_army`], and the one the AI's **first soldier** takes
    /// (`docs/ARMY.md` §4.2).
    ///
    /// It sits before the function's target search, so a military unit
    /// that reaches `Unit::think`'s auto-attack arm joins an army whether
    /// or not it finds something to shoot. The listing is
    /// `llvm-objdump 0x5f5a80..0x5f5db0`, and the gate is five tests:
    ///
    /// ```text
    /// manual = !(unit_masks & 0x40000)                 # not AI-driven
    /// if !manual and !(leader_flags & 2):   manual = 1 # not in play
    /// if leader_flags2 & 8:                 manual = 1
    /// if manual: return                                # 5f5c4d
    /// range = 0
    /// if tile(x, y) & 0x100 and cell(x, y).who == who: # my own city's radius
    ///     find_city(x, y, SEARCH_FRIENDLY, who, 0x200, FILTER_ALL)   # dropped
    ///     if damage != 0 and ((obj_masks & 0x1020) or healing != 0):
    ///         range = -1                               # 5f5d10 — stay and heal
    /// if role & 0x10:            range = -1            # 5f5d49 — a scout
    /// if type_index in {0x3d, 0x3e, 0x190} or is_caravan(): return
    /// if range >= 0: army = add_to_army(this)          # 5f5d8c
    /// ```
    ///
    /// `0x1020` is `MOUNTED | FOOT` (`docs/COMBAT.md` §3), `+0x24` is
    /// `ObjectData::damage` and `+0x38` its `healing`; `role & 0x10` is
    /// the scout bit the think tail reads too. So **a damaged foot or
    /// mounted unit standing inside one of its own cities' radius stays
    /// there** and everything else joins.
    ///
    /// Three things in the head are seams: `leader_flags2 & 8`, which no
    /// capture sets; `ObjectData::healing`, which this crate does not
    /// carry (it only widens the stay-and-heal arm, and only for a
    /// damaged unit that is neither foot nor mounted); and the
    /// `find_city` whose answer the original **discards** — `local_10` is
    /// written 1 before the call, and the only reader of the call would
    /// be that variable. The tail's `go_to_city` walk, which that
    /// variable and the army slot gate, is [`Self::think_attack_tail`]
    /// (item 3369).
    pub(crate) fn think_attack_join_army(&mut self, u: usize) -> Option<usize> {
        use crate::ai_load::{role, uflags2};
        use crate::combat::mask;
        let who = self.units[u].owner;
        if !self.ai_driven(who) || self.defeated[who as usize] {
            return None;
        }
        let t = self.units[u].ty?;
        // **Which units enter `think_attack` at all** — `Unit::think`'s
        // step 3, at `5f7152`: `type.attack != 0` (`+0x1e8`, the base
        // column, not the runtime stat) **and** `role & 0x10000`, the
        // military bit. That second half is the whole difference between
        // a soldier and an armed citizen, and without it this crate
        // conscripted run53's woodcutters on frame 307. The other arm —
        // `is(0x3e, 1) && !is_packing_or_unpacking()`, the merchant
        // lineage's — is a seam; a merchant is filtered again below.
        if self.unit_types[t].combat.attack == 0 || !self.unit_types[t].cols.is(role::MILITARY) {
            return None;
        }
        // The own-territory arm: inside a city's radius (`tile & 0x100`)
        // and on a cell my own territory holds.
        let home = self.think_attack_home(u);
        if home
            && self.units[u].health < self.units[u].max_health
            && self.unit_types[t].combat.obj_masks & (mask::FOOT | mask::MOUNTED) != 0
        {
            return None;
        }
        if self.unit_types[t].cols.is(role::SCOUT) {
            return None;
        }
        if self.is_merchant(u) || self.unit_types[t].cols.flag2(uflags2::CARAVAN) {
            return None;
        }
        self.add_to_army(u)
    }

    /// `Unit::come_out@00617c10`'s tail (`0061a0c5`..`0061a1fb`) — the coin
    /// a unit throws as it steps out of whatever was carrying it, and the
    /// **fifth** of §4's callers.
    ///
    /// It is the last thing the function does, after the exit spot is taken,
    /// and the listing is five gates and two arms
    /// (`llvm-objdump 0x61a0a0..0x61a230`; the decompile merges the two
    /// `is` calls into one unnamed slot and the type arguments are only in
    /// the listing):
    ///
    /// ```text
    /// options->rebuild = 1
    /// if !(unit_masks & 0x40000):                      return   # not AI-driven
    /// if is_caravan():                                 return   # unit_flags2 & 8
    /// if type_index in {MERCHANT 0x3d, MERCHANTDUTCH 0x3e, FURTRAPPER 0x190}: return
    /// if !is_special() and !is(BARK, 0):                         # 0x2b8 & 0x10, 0x143
    ///     join = is(SPY, 0) != 0                                 # 0x3a — no draw
    /// else:
    ///     r    = Random::get(game_random, 0, 0xffff) % (is(BARK) ? 3 : 2)
    ///     join = (game->frame & r) != 0
    /// if join: add_to_army(this)
    /// ```
    ///
    /// `is_special` is `is(SCOUT)` by the loader
    /// (`docs/DATALAYER.md`, `unit_flags2`), so the two coin arms are **the
    /// scout line and the naval-scout line** and nothing else: a citizen, a
    /// soldier or a boat leaves a building spending nothing, which is why
    /// the site is rare enough to have gone unnoticed for 3,977 frames.
    /// The `% 2` arm is written `& 0x80000001` with a sign fixup the draw's
    /// own range makes dead.
    ///
    /// **Both arms are diff-backed on the same capture.** run54's 24,000
    /// frames reach the site eleven times: nine `+0x25ca` under
    /// `Object::eject_contents < Unit::set_new_location` — a passenger put
    /// ashore — and two `+0x25b0` under `Build::train < Build::finished`,
    /// which is the AI's Bark being trained on frames 10323 and 10465. The
    /// two addresses are the two arms, and that is what names them.
    pub(crate) fn come_out_join_army(&mut self, u: usize) {
        if !self.ai_driven(self.units[u].owner) {
            return;
        }
        let Some(t) = self.units[u].ty else { return };
        if self.unit_types[t].cols.flag2(uflags2::CARAVAN) {
            return;
        }
        if self.unit_tree(u).is_some_and(|ti| MERCHANTS.contains(&ti)) {
            return;
        }
        let bark = self.unit_line_is(u, BARK);
        let join = if !self.unit_types[t].cols.flag2(uflags2::SCOUT) && !bark {
            self.unit_line_is(u, SPY)
        } else {
            self.mark(if bark {
                SITE_COME_OUT_BARK
            } else {
                SITE_COME_OUT
            });
            let r = i64::from(self.rnd(if bark { 3 } else { 2 }));
            self.frame & r != 0
        };
        if join {
            self.add_to_army(u);
        }
    }

    // ---- the frame hook and the cadence (§5) ----

    /// `Armies::process_all`: every in-use, non-human leader's valid
    /// armies, the hurry bit consumed into the bypass.
    pub fn armies_process_all(&mut self) {
        for w in 0..self.players.len() {
            if self.nation[w].human {
                continue;
            }
            let who = w as Player;
            for s in 0..SLOTS {
                let a = &mut self.armies[w].list[s];
                if !a.valid {
                    continue;
                }
                let bypass = a.status & status::HURRY != 0;
                a.status &= !status::HURRY;
                self.army_process(who, s, bypass);
            }
        }
    }

    /// `Army::process@006f93d0` (§5–§6): the walk of
    /// [`crate::army_process_view::army_process`] over this army's slot.
    pub fn army_process(&mut self, who: Player, slot: usize, bypass: bool) {
        crate::army_process_view::army_process(&mut SimProcess { s: self, who, slot }, bypass);
    }

    /// `Army::use_generals@006f4c30`: every member of the army's groups,
    /// in list order, that is active and a hero (`unit_flags2 & 0x20`) is
    /// offered [`Sim::think_spellcaster`]; the first that casts ends the
    /// turn.
    fn use_generals(&mut self, who: Player, slot: usize) {
        let members = self.armies[who as usize].list[slot].units.clone();
        generals_walk(
            self,
            members,
            |s, u| s.units[u].alive(),
            |s, u| s.is_hero_unit(u),
            |s, u| s.think_spellcaster(u),
        );
    }

    /// `Army::use_spies@006f4af0`: as [`Self::use_generals`], with the
    /// member's `is(SPY)` (vslot `0xb8`) for the hero test. A Spy's
    /// [`Sim::think_spellcaster`] draws nothing (its cloak is a seam).
    fn use_spies(&mut self, who: Player, slot: usize) {
        let members = self.armies[who as usize].list[slot].units.clone();
        generals_walk(
            self,
            members,
            |s, u| s.units[u].alive(),
            |s, u| s.unit_line_is(u, SPY),
            |s, u| s.think_spellcaster(u),
        );
    }

    /// `Army::use_scouts@006f49a0`: as [`Self::use_generals`], with the
    /// member's `is_special` (vslot `0xd4`, `UnitData::is_special`'s
    /// `TypeData +0x2b8 & 0x10`) for the hero test — the Scout's turn,
    /// whose special arm throws the coin at `+0x413`.
    fn use_scouts(&mut self, who: Player, slot: usize) {
        let members = self.armies[who as usize].list[slot].units.clone();
        generals_walk(
            self,
            members,
            |s, u| s.units[u].alive(),
            |s, u| s.unit_is_special(u),
            |s, u| s.think_spellcaster(u),
        );
    }

    // ---- mustering (§7) ----

    /// `strategy[reg]` of the leader's census, 0 off the table.
    fn strategy_of(&self, who: Player, reg: Option<u16>) -> i32 {
        reg.and_then(|r| {
            self.ai[who as usize]
                .census
                .strategy
                .get(r as usize)
                .copied()
        })
        .unwrap_or(0)
    }

    /// `Army::do_mustering` — `docs/TRANSPORT.md` §8.1's table. Public
    /// because `rondata::diff` replays it on a dumped block
    /// (`docs/ARMY.md` §17).
    pub fn do_mustering(&mut self, who: Player, slot: usize) {
        let w = who as usize;
        let released = self.release_mustering(who, slot);
        let (city, reg, navy, captains) = {
            let a = &self.armies[w].list[slot];
            (a.city, a.reg, a.navy, a.num_captains)
        };
        let s = self.strategy_of(who, reg);
        let expand = s & 8 != 0;
        let weak = s & 4 != 0;
        let level = self.transport_level(who) as i32 >= 2;
        let new_status = if !released {
            if let Some(c) = city
                && self.cities[c].alive
            {
                let b = self.cities[c].building;
                if self.find_muster_spot(who, slot, Obj::Building(b), true) {
                    self.armies[w].list[slot].status |= status::FORMING;
                    return;
                }
            }
            if expand && captains > 7 && level {
                status::TRANSPORTING
            } else {
                status::MARCHING
            }
        } else if !navy {
            if weak && self.ai_difficulty() < 3 {
                status::DEFENDING
            } else if !expand || !level {
                status::MARCHING
            } else {
                status::TRANSPORTING
            }
        } else {
            status::MARCHING
        };
        let a = &mut self.armies[w].list[slot];
        a.status = new_status;
        a.city = None;
        a.pos = cell_centre(a.muster);
        a.angle = a.muster_angle;
    }

    /// `Army::release_mustering` (§7): true means go.
    pub fn release_mustering(&self, who: Player, slot: usize) -> bool {
        let w = who as usize;
        let a = &self.armies[w].list[slot];
        let Some(c) = a.city else { return true };
        if self.cities[c].owner != who || !self.cities[c].alive {
            return true;
        }
        // **The pre-war gate** (`006f87c0`, after the city test: `rush_rules
        // && !Game::war_allowed()` returns 0). The peace pair's army 0 holds
        // its five standard at the muster through the five minutes' peace;
        // released, it spent `find_target`'s two scores on 2040 that the
        // original never spent (item 2388, `docs/AI.md` §221).
        if self.lobby.rush_rules != 0 && !self.war_allowed() {
            return false;
        }
        let n = a.num_standard;
        if n < 2 {
            return false;
        }
        for i in 0..self.players.len() {
            let ip = i as Player;
            if self.defeated[i] || !self.is_ally(who, ip) {
                continue;
            }
            for c in self.cities_of(ip) {
                if self.cities[c].no_heal && (i == w || n > 4) {
                    return true;
                }
            }
        }
        if n < self.city_num(who) || n < 5 {
            return false;
        }
        if a.navy {
            return true;
        }
        let p = self.muster[w].cap;
        if p * 7 / 8 <= self.ai[w].effective_pop || p / 8 <= n {
            return true;
        }
        let mustering = self.num_armies(who, status::MUSTERING, a.reg).max(1);
        if p / (2 * mustering) <= n {
            return true;
        }
        let diff = self.ai_difficulty();
        if (diff == 0 && n > 6) || (diff == 1 && n > 10) || (diff == 2 && n > 12) {
            return true;
        }
        // **The Military library level, not the age** (`docs/ARMY.md`
        // §7, item 657): `data_encrypted->epoch[0] ^ 0x63187`
        // (`6f8997`, `6f89c3`). `ages` is `+0xdc`, under its own key
        // `0x62766`. Great Lakes' army 2 on sim-frame 12024 reads age 1
        // and Military 2: six standard stay mustering (`n >= 16`), and it
        // is the army `1/68` joins on 12057.
        let age = self.tech[w].military_level() as i32;
        let pers = &self.ai[w].pers;
        if pers.rush == -1 {
            return n >= 33;
        }
        if pers.rush == 1 {
            if age < 2 {
                return true;
            }
            if age < 3 {
                return n >= 16;
            }
            if age < 5 {
                return n >= 21;
            }
        } else if pers.early_army != 0 {
            if age < 2 {
                return true;
            }
            if age < 3 {
                return n >= 16;
            }
        }
        n >= 26
    }

    // ---- marching (§9) ----

    /// `Army::do_marching`: validate the target, retarget, then
    /// `march_to_target`. The steps are [`do_marching_steps`]'s (item 3238's
    /// sweep, `Army::do_marching@006f3df0`); this crate's view of the army
    /// and its target is [`MarchSim`].
    ///
    /// **No target is the retarget branch, not the march.** The
    /// original opens on `iVar4 = field_0x30; if (iVar4 < 0) goto
    /// LAB_006f3fb2` (`006f3df0`+0x1d), so an army that reaches
    /// `do_marching` holding nothing goes straight to `find_target` —
    /// and it reaches it holding nothing on the very tick
    /// `do_mustering` releases it, because `Army::process` re-reads
    /// `status` between its dispatch `if`s and `status = 2` leaves the
    /// target at `-1` (§9, §11). Modelling only the "target present but
    /// stale" arm cost Great Lakes' word 7930: the original spends
    /// `find_target`'s coin once and its per-candidate score twice
    /// there and this crate spent nothing (item 317).
    fn do_marching(&mut self, who: Player, slot: usize) {
        let t = self.armies[who as usize].list[slot].target;
        do_marching_steps(&mut MarchSim {
            sim: self,
            who,
            slot,
            t,
        });
    }

    /// `Army::march_to_target` (§9): not engaged → forming and marching;
    /// engaged, and only on the first tick of the fight, the orders. The
    /// steps are [`march_to_target_steps`]'s (item 3259's sweep), over this
    /// crate's [`MarchToSim`].
    fn march_to_target(&mut self, who: Player, slot: usize) {
        march_to_target_steps(&mut MarchToSim {
            sim: self,
            who,
            slot,
        });
    }

    // ---- forming (§8) ----

    /// `Army::do_forming`: the muster cell's centre, projected a tile per
    /// group, and one order per group. The steps are [`do_forming_steps`]'s
    /// (item 3250's sweep), over this crate's [`FormSim`].
    fn do_forming(&mut self, who: Player, slot: usize) {
        do_forming_steps(&mut FormSim {
            sim: self,
            who,
            slot,
        });
    }

    /// §8.2's stance rule, shared by `do_forming`, `march_to_target` and
    /// `engagement`: an army marching on an enemy before the Gunpowder age
    /// with no siege of its own raids.
    fn army_stance_rule(&mut self, who: Player, slot: usize) {
        let w = who as usize;
        let (target, hurry) = {
            let a = &self.armies[w].list[slot];
            (a.target, a.hurry)
        };
        let Some(t) = target else { return };
        let tw = self.owner_of(t);
        if tw != who
            && self.is_enemy(who, tw)
            && hurry == 0
            && (self.tech[w].military_level() as i32) < 4
            && self.army_count_siege(who, slot) == 0
        {
            self.army_set_stance(who, slot, Stance::Raid);
        }
    }

    /// `Army::set_stance(s)` (§6): `Group::action_stance` on the one group,
    /// and only when its stance panel is the combat one.
    /// The steps are [`set_stance_steps`]'s (item 2341's sweep); the army's
    /// group slots collapse to the one group (`num_groups` 1 while it has
    /// members), whose `num` is its member count.
    fn army_set_stance(&mut self, who: Player, slot: usize, s: Stance) {
        set_stance_steps(
            self,
            |sim| i16::from(!sim.armies[who as usize].list[slot].units.is_empty()),
            |_, _| 0,
            |sim, _| sim.armies[who as usize].list[slot].units.len() as i32,
            |sim, _| {
                let g = sim.army_group(who, slot);
                sim.group_stance_type(&g) == crate::group::StanceType::Combat
            },
            |sim, _, s| {
                let g = sim.army_group(who, slot);
                sim.group_action_stance(&g, s);
            },
            crate::group::stance_option(s),
        );
    }

    // ---- the helpers (§14) ----

    /// `Army::send_here(x, y, order)`: the point and the muster cell move,
    /// and every group walks there.
    pub fn send_here(&mut self, who: Player, slot: usize, p: Pos, kind: MoveKind) {
        let w = who as usize;
        let p = self.restrict(p);
        let here = self.armies[w].list[slot].pos;
        if p.x != here.x && p.y != here.y {
            self.armies[w].list[slot].muster_angle = find_angle(p.x - here.x, p.y - here.y);
        }
        {
            let a = &mut self.armies[w].list[slot];
            a.pos = p;
            a.muster = p.cell();
        }
        let ang = self.armies[w].list[slot].muster_angle;
        let g = self.army_group(who, slot);
        self.group_action_move_to(&g, p, QueuePos::New, true, ang, kind, true);
    }

    /// `Army::charge(o, who)`: a siege unit under attack drags its army
    /// onto the attacker.
    pub fn army_charge(&mut self, who: Player, slot: usize, target: Obj) {
        if !self.armies[who as usize].list[slot].valid {
            return;
        }
        let p = self.pos_of(target);
        let g = self.army_group(who, slot);
        self.group_action_move_to(
            &g,
            p,
            QueuePos::First,
            false,
            Angle(0),
            MoveKind::AttackTo,
            true,
        );
        self.armies[who as usize].list[slot].target = Some(target);
    }

    // ---- defending (§10) ----

    fn do_defending(&mut self, who: Player, slot: usize) {
        let w = who as usize;
        if self.army_count_attack(who, slot) < 5 {
            self.close_army(who, slot);
            return;
        }
        if self.army_is_engaged(who, slot) {
            return;
        }
        self.armies[w].list[slot].status &= !status::MARCHED;
        let (st, target) = {
            let a = &self.armies[w].list[slot];
            (a.status, a.target)
        };
        if st & status::MARCHING != 0
            && let Some(t) = target
            && self.active(t)
        {
            if let Obj::Building(b) = t
                && let Some(c) = self.buildings[b].city
                && self.cities[c].alive
                && self.cities[c].no_heal
            {
                return;
            }
            let a = &mut self.armies[w].list[slot];
            a.target = None;
            a.muster = a.pos.cell();
            a.status = (a.status & !(status::DEFENDING | status::MARCHING)) | status::FORMING;
            return;
        }
        if self.find_besieged_city(who, slot) {
            self.armies[w].list[slot].status |= status::MARCHING;
            return;
        }
        let pos = self.armies[w].list[slot].pos;
        let city = self.nearest_friendly_city(who, pos);
        {
            let a = &mut self.armies[w].list[slot];
            a.target = None;
            a.city = city;
        }
        if let Some(c) = city {
            self.armies[w].list[slot].status &= !(status::DEFENDING | status::MARCHING);
            let cpos = self.cities[c].pos;
            let p = self.restrict(Pos::new(cpos.x, cpos.y + UNITS_PER_CELL));
            self.armies[w].list[slot].pos = p;
            let b = self.cities[c].building;
            if self.find_muster_spot(who, slot, Obj::Building(b), true) {
                self.armies[w].list[slot].status |= status::FORMING;
            } else {
                self.close_army(who, slot);
            }
            return;
        }
        let a = &mut self.armies[w].list[slot];
        a.muster = a.pos.cell();
        a.status = (a.status & !(status::DEFENDING | status::MARCHING)) | status::FORMING;
    }

    /// `WorldData::restrict`: clamp a position into the world.
    pub(crate) fn restrict(&self, p: Pos) -> Pos {
        Pos::new(
            p.x.clamp(0, self.world.width() * UNITS_PER_CELL - 1),
            p.y.clamp(0, self.world.height() * UNITS_PER_CELL - 1),
        )
    }

    /// `Army::find_besieged_city` (§10): the attacked city of mine or an
    /// ally's, in my region, that scores best —
    /// [`find_besieged_city_steps`] over [`BesiegedSim`].
    fn find_besieged_city(&mut self, who: Player, slot: usize) -> bool {
        find_besieged_city_steps(&mut BesiegedSim {
            sim: self,
            who,
            slot,
        })
    }

    // ---- engagement (§11) ----

    fn army_centre_of_gravity(&self, who: Player, slot: usize) -> Cell {
        let a = &self.armies[who as usize].list[slot];
        let mut n = 0;
        let mut sx = 0;
        let mut sy = 0;
        for &u in &a.units {
            if self.units[u].alive() && self.units[u].on_map {
                let c = self.units[u].pos.cell();
                sx += c.x;
                sy += c.y;
                n += 1;
            }
        }
        if n == 0 {
            a.pos.cell()
        } else {
            Cell {
                x: sx / n,
                y: sy / n,
            }
        }
    }

    /// `Army::is_engaged`: more than a quarter of the units fighting near
    /// the army's point and its centre of gravity.
    pub fn army_is_engaged(&mut self, who: Player, slot: usize) -> bool {
        self.army_normalize(who, slot);
        let w = who as usize;
        let (units, pos, n) = {
            let a = &self.armies[w].list[slot];
            (a.units.clone(), a.pos, a.num_units)
        };
        if n <= 0 {
            return false;
        }
        let mut cog: Option<Pos> = None;
        let mut count = 0;
        for u in units {
            let unit = &self.units[u];
            if !unit.alive() || !unit.on_map {
                continue;
            }
            // `get_action()`, not the front order: the test at `6f5207`
            // is on `UnitData::get_action`'s result, so a unit walking a
            // transit leg **in front of** its attack still counts as
            // attacking. run29's block 15100 is exactly that shape —
            // every member of army 0's group holds `[MOVEORDER(pathed),
            // ATTACKORDER]` — and with the front order tested no unit
            // qualified and `engagement` did nothing.
            let attacking = self
                .action_of(u)
                .and_then(|i| self.units[u].orders.get(i))
                .is_some_and(|o| matches!(o.body, crate::orders::Body::Attack(_)));
            let unit = &self.units[u];
            if !attacking || dist(unit.pos, pos) >= 0xc00 {
                continue;
            }
            let c = *cog.get_or_insert_with(|| cell_centre(self.army_centre_of_gravity(who, slot)));
            if dist(self.units[u].pos, c) <= 0xc00 {
                count += 1;
            }
        }
        count != 0 && count > n / 4
    }

    /// `Army::is_moving@006f5470`: more than a sixth of the units either
    /// standing outside the region or walking near the centre of gravity
    /// (the walk is [`crate::army_is_moving_view::army_is_moving`]).
    pub fn army_is_moving(&self, who: Player, slot: usize) -> bool {
        crate::army_is_moving_view::army_is_moving(&mut SimIsMoving { s: self, who, slot })
    }

    /// `Army::engagement`'s seed (§11): which member's attack the army
    /// adopts as its target of the moment.
    ///
    /// Read from the listing at `6f5160`, because the decompiler's locals
    /// do not show the shape: `%edi` (`ox`) and `%ebx` (`whom`) are
    /// rewritten by **every** unit that gets as far as the two distance
    /// tests, and the `is_map_unit` call at `6f5362` gates only the
    /// **break** (`6f5369 jne 6f5383`). So the seed is the *first*
    /// qualifying unit whose target is a map unit — and, when there is
    /// none, the *last* qualifying unit's target whatever it is, which
    /// the tail at `6f5383` accepts on `ox >= 0 && whom >= 0` alone. The
    /// walk is `ArmyData::get_unit`'s: the groups' member lists in order.
    ///
    /// Returns the member that seeded it and the target it named.
    pub fn army_engagement_seed(&mut self, who: Player, slot: usize) -> Option<(usize, Obj)> {
        let w = who as usize;
        let (units, pos) = {
            let a = &self.armies[w].list[slot];
            (a.units.clone(), a.pos)
        };
        let cog = cell_centre(self.army_centre_of_gravity(who, slot));
        let mut last = None;
        for u in units {
            if !self.units[u].alive() || !self.units[u].on_map {
                continue;
            }
            // `get_action()`, the intent under the transit legs (`6f51fa`).
            if !self
                .action_of(u)
                .and_then(|i| self.units[u].orders.get(i))
                .is_some_and(|o| matches!(o.body, crate::orders::Body::Attack(_)))
            {
                continue;
            }
            if dist(self.units[u].pos, pos) >= 0xc00 || dist(self.units[u].pos, cog) > 0xc00 {
                continue;
            }
            // `update_action().get_target_order()` — called for its
            // side effects too, as the original does (twice, once per
            // half of the pair).
            self.update_action(u);
            let Some(t) = self.units[u].combat.target else {
                // `6f5345`/`6f5349`: a negative `ox`/`whom` goes to the
                // next iteration without restoring the spills, so a
                // qualifying unit whose target has gone erases an earlier
                // fallback rather than leaving it standing.
                last = None;
                continue;
            };
            last = Some((u, t));
            // `ObjectData::is_map_unit`: a building target does not stop
            // the walk, it is only kept in case nothing better turns up.
            if let Obj::Unit(t) = t
                && self.units[t].alive()
                && self.units[t].on_map
            {
                break;
            }
        }
        last
    }

    /// `Army::engagement` (§11): the seed's target becomes the army's
    /// target of the moment, and every group with a valid member is
    /// pointed at it.
    pub fn engagement(&mut self, who: Player, slot: usize) {
        let Some((_, t)) = self.army_engagement_seed(who, slot) else {
            return;
        };
        self.army_stance_rule(who, slot);
        let g = self.army_group(who, slot);
        if self.group_num_valid(&g) == 0 {
            return;
        }
        self.group_action_attack(&g, t, false, QueuePos::New, 0);
    }

    // ---- the target (§12) ----

    /// `LeaderData::get_target@006da000`: the first leader after `who` in
    /// [`Lobby::start_list`] whose `leader_flags & 3 == 3`, else `who`.
    ///
    /// **SEAM**: `flags & 3 == 3` is "seated and not defeated", this
    /// crate's one proxy for both bits, and the list is an input the lobby
    /// carries — nothing here shuffles it, and the harness does not yet read
    /// the dump's `start_list` lines into it. Read only under `team_style ==
    /// 2`, a lobby no capture on disk has (scan: `grep -h '^  TEAM_STYLE'
    /// gamelog*.txt | sort | uniq -c`, 313 of style 1 and none of 2 over the
    /// 316 dumps).
    #[cfg(test)]
    pub(crate) fn start_list_target(&self, who: Player) -> usize {
        let list = &self.lobby.start_list;
        let at = list.iter().position(|&l| l == who as usize).unwrap_or(0);
        for j in 0..8 {
            let l = list[(at + 1 + j) & 7];
            if l < self.players.len() && !self.defeated[l] {
                return l;
            }
        }
        who as usize
    }

    /// `Army::find_target` over cities. The forts pass and the fields the
    /// sim lacks are seams; see the module doc.
    pub fn find_target(&mut self, who: Player, slot: usize) {
        use crate::target_gate::{SimGate, target_city_gate, target_head, target_leader_gate};
        use crate::target_score::{Ctx, target_score, target_tail};
        // `epoch[0] ^ 0x63187`, the Military library level, is read at every
        // site below (§12); the forts pass's `ages ^ 0x62766` is a seam.
        // The head (`target_gate::target_head`, swept against the original
        // in `sweep/army_find_target.rs`): the weak test — a land army is
        // weak with fewer than four of hoplites, siege twice and
        // cataphracts, **or**, past Military 2, with no wagon of its own
        // while the leader can have one (French East Indies' army 0 is that
        // on 14332..14844 and scores no enemy city; army 6 carries a wagon
        // on 15088, item 1487) — the capital, and the two sea averages
        // (audit B.20: the listing's base is the leader plus 8, so `+0x944`
        // there is `+0x94c`).
        let head = target_head(&mut SimGate {
            sim: &mut *self,
            who,
            slot,
        });
        if head.stop {
            return;
        }

        let mut best: Option<(i32, usize, Player)> = None;
        for pass in 0..2 {
            for i in 0..8 {
                // The leader gate (`target_gate::target_leader_gate`):
                // seated, the diplomacy arms (`defense_mod` is the switch,
                // audit B.22), the difficulty clauses (listing
                // `6f6ce1`-`6f6fb4`; at `diff < 2` the stamp, the
                // aggressive-army test and the coin come first and a
                // leader that passes them **goes on** — run26), and the
                // `team_style == 2` list (`6f6fb4..6f705f`; group 4, A5
                // #30). `weak`/`strong`/the tribute period are seams.
                let about = match target_leader_gate(
                    &mut SimGate {
                        sim: &mut *self,
                        who,
                        slot,
                    },
                    i,
                ) {
                    Ok(Some(about)) => about,
                    _ => continue,
                };
                let ip = i as Player;
                for (k, c) in self.cities_of(ip).into_iter().enumerate() {
                    // The city gate (`target_gate::target_city_gate`): a
                    // navy's coast and `ocean` test (`6f70b5..6f70e0`;
                    // twenty-fourth pass, group 4: the count the owner's
                    // census wrote, so a leader whose census never ran has
                    // none), and the test before the draw (B.25): on the
                    // two easiest difficulties a city of mine or an ally's
                    // is scored only if I founded it; an enemy's always is.
                    // (Run26 caught the inverse — the human's capital
                    // dropped and one draw short.)
                    if !target_city_gate(
                        &mut SimGate {
                            sim: &mut *self,
                            who,
                            slot,
                        },
                        i,
                        k,
                    ) {
                        continue;
                    }
                    // The score (`target_score::target_score`, swept against
                    // the original in `sweep/army_find_target_run.rs`).
                    self.mark(SITE_FIND_TARGET_SCORE);
                    let draw = self.rng.get(0, 0xffff);
                    let ctx = Ctx {
                        strong: head.strong,
                        cap: head.cap,
                        ours: head.ours,
                        theirs: head.theirs,
                        supply: head.supply,
                        pass,
                        about,
                    };
                    let Some(v) = target_score(
                        &mut SimGate {
                            sim: &mut *self,
                            who,
                            slot,
                        },
                        &ctx,
                        i,
                        k,
                        draw,
                    ) else {
                        continue;
                    };
                    if best.is_none_or(|(bv, _, _)| bv <= v) {
                        best = Some((v, c, ip));
                    }
                }
            }
            if best.is_some() {
                break;
            }
        }
        let Some((_, c, tw)) = best else { return };
        // The tail (`target_score::target_tail`): the hurry, the probe of the
        // easiest AI against a human (`docs/ARMY.md` §12, `docs/COMBAT.md`
        // §17; the whole of Great Lakes' frame 8186, where the long word
        // parts), the target and its rally, the target's leader marked
        // attacked, the escort (`find_target@006f69b0:1061`-`1087`,
        // `docs/TRANSPORT.md` §8.3), the point and the muster spot.
        target_tail(
            &mut SimGate {
                sim: &mut *self,
                who,
                slot,
            },
            tw as usize,
            c as i32,
        );
    }

    /// `Armies::send_navy(who, o, target_who, reg_a, reg_b)@006f2c90`
    /// (`docs/TRANSPORT.md` §8.3): every valid navy of an AI leader with
    /// more than two captains, in a sea region that coasts both the land
    /// army's region and the target's, takes the same target and is
    /// `Army::process(0)`ed at once — the gated call, which does nothing
    /// off its own frames but count `human_frame` down.
    pub(crate) fn send_navy(
        &mut self,
        who: Player,
        target: Obj,
        reg_a: Option<u16>,
        reg_b: Option<u16>,
    ) {
        let w = who as usize;
        // `leader_flags & 5 == 1`: active and not human. `leader_flags2 &
        // 0xa`, the stop bits, are never set here (the module doc).
        if self.nation[w].human || self.defeated[w] {
            return;
        }
        let (Some(a), Some(b)) = (reg_a, reg_b) else {
            return;
        };
        for k in 0..SLOTS {
            let n = &self.armies[w].list[k];
            if !n.valid || !n.navy || n.num_captains <= 2 {
                continue;
            }
            // `reg > 0x3f`: a sea region, by the original's numbering —
            // here the region's terrain, since the sim numbers its own.
            let Some(sea) = n.reg.filter(|&r| self.world.terrain(r) == Terrain::Sea) else {
                continue;
            };
            if self.world.is_coast(sea, a) && self.world.is_coast(sea, b) {
                self.armies[w].list[k].target = Some(target);
                self.army_process(who, k, false);
            }
        }
    }

    /// **The probe** — `Army::find_target@006f69b0:1120`-`1199`, the arm
    /// the easiest difficulty takes against a human leader instead of
    /// making the city an army target (`docs/ARMY.md` §12, "The probe";
    /// `docs/COMBAT.md` §17.1).
    ///
    /// `ArmyData::get_unit(army, 0)` and `get_unit(army, count − 1)` go
    /// into a stack-local group. **Two adds become six orders**, because
    /// `Group::add`'s subordinate recursion brings each captain's
    /// followers (`docs/GROUPS.md` §4.1) and Great Lakes' army group 64
    /// is five squads of three. Then, when the target's leader has no
    /// soldiers at all, the group is pointed at one of that leader's
    /// **farms** — `find_building(..., FILTER_TYPE, FARM)` — with
    /// `action_stance(5)`, a **mandatory** `action_attack`, and a
    /// trailing `action_move_to` back to the pair's own leader; otherwise
    /// a single `ATTACK_TO` at the city.
    pub(crate) fn find_target_probe(
        &mut self,
        who: Player,
        slot: usize,
        c: usize,
        tw: Player,
        frame: i64,
    ) {
        let w = who as usize;
        let units = self.armies[w].list[slot].units.clone();
        let (Some(&first), Some(&last)) = (units.first(), units.last()) else {
            return;
        };
        let mut g = crate::group::Group {
            who,
            army: None,
            pushed: None,
            list: Vec::new(),
        };
        self.group_add(&mut g, first);
        self.group_add(&mut g, last);
        // `Groups::push_group(who, who, &g, 1)` — forced, so a group of
        // one still takes a slot.
        if !self.push_group(&mut g, true) {
            return;
        }
        let target = Obj::Building(self.cities[c].building);
        let tpos = self.pos_of(target);
        let farm = if self.ai[tw as usize].census.combat == 0 {
            self.find_probe_farm(tpos, tw)
        } else {
            None
        };
        if let Some(b) = farm {
            self.group_action_stance(&g, 5);
            self.group_action_attack(&g, Obj::Building(b), true, QueuePos::New, 0);
            // The trailing move goes back to the **pair's own leader**
            // (`:1152`, read back at `:1191`), not to `get_unit(0)`.
            if let Some(leader) = self.group_find_leader(&g) {
                let back = self.units[leader].pos;
                self.group_action_move_to(
                    &g,
                    back,
                    QueuePos::Last,
                    false,
                    Angle(0),
                    MoveKind::MoveTo,
                    false,
                );
            }
        } else {
            self.group_action_move_to(
                &g,
                tpos,
                QueuePos::New,
                false,
                Angle(0),
                MoveKind::AttackTo,
                false,
            );
        }
        self.group_normalize(&mut g);
        self.ai[tw as usize].frame_attacked = frame;
        self.ai[tw as usize].attacked_by = i32::from(who);
    }

    /// `ObjectsData::find_building(x, y, SEARCH_FRIENDLY, who, −1,
    /// 0x200, FILTER_TYPE, FARM, 0)@0065d260` — the probe's one call.
    ///
    /// **The `0x200` is the flag word, not a radius.** The radius is the
    /// argument before it and it is **−1**, so `65d288`'s
    /// `param_5 < 0 || 9 < (param_5 + 0x2ff)/0x300` sends the search down
    /// the *exhaustive* arm rather than the `circle_x`/`circle_y` grid:
    /// every object of every leader `valid_search` admits, with no
    /// distance bound at all. `0x200` is the flag the head reads as
    /// `param_6 & 0x200` — the **region** filter, which pins the answer
    /// to the tile region the asking point stands in.
    ///
    /// The distance is measured in **tiles**, not position units
    /// (`65d2f4`: `div_3_table[x >> 6]` on both operands before the
    /// octagonal approximation), and the winner test is `<=` on both the
    /// running best and the bound, so a tie goes to the **last**
    /// candidate in walk order rather than the first.
    fn find_probe_farm(&self, at: Pos, who: Player) -> Option<usize> {
        let reg = self.world.tregion(at.tile());
        let a = at.tile();
        let mut best: Option<(i32, usize)> = None;
        for (b, bd) in self.buildings.iter().enumerate() {
            if bd.owner != who || !bd.alive || self.building_ident(b) != Ident::Farm {
                continue;
            }
            let p = bd.pos.tile();
            if self.world.tregion(p) != reg {
                continue;
            }
            let d = vector_dist(p.x - a.x, p.y - a.y);
            if best.is_none_or(|(bd, _)| d <= bd) {
                best = Some((d, b));
            }
        }
        best.map(|(_, b)| b)
    }

    // ---- the muster spot (§13) ----

    /// `LandData::move_rate` — `lands[class].+0x100`, what a neighbour
    /// cell adds to a candidate's score. `Lands::init@0067e730` writes
    /// `0x100` to every one of the nine lands' `move_rate` (and
    /// `combat_bonus`), and nothing else in the export writes either, so
    /// the score is a constant: 256 per admissible neighbour, and the
    /// ring's walk order breaks the ties.
    pub(crate) const LAND_MOVE_RATE: i32 = 0x100;

    /// `Army::find_muster_spot(o, who, flag)` (§13): the chase over the
    /// target city's damaged buildings; then the ring search — the cells
    /// `circle_x/y[circle_radius[inner]..circle_radius[outer])` around the
    /// army's point, each admitted by its neighbourhood's regions, land
    /// classes and owners and scored by [`Self::LAND_MOVE_RATE`], the best
    /// by strict `>` in walk order; then the original's fallback — the
    /// target city's cell one row down (up on the last row) — for a land
    /// army at an enemy's or a damaged city centre, and the `no_muster`
    /// mark at an untroubled city centre of one's own.
    ///
    /// `flag` widens the neighbourhood the search walks for bounds and
    /// foreign owners from the 3 × 3 to the 7 × 7; `do_mustering`,
    /// `do_defending` and `do_transporting` pass it, `process`,
    /// `find_target` and `find_besieged_city` do not.
    ///
    /// The return is the original's `local_c` — **the last in-bounds
    /// candidate's validity**, or 1 after the early stop — not whether a
    /// spot was found (listing `6f6824`: `mov eax, [ebp-8]`); a found
    /// best whose ring ends on an inadmissible cell returns 0 with the
    /// muster set. Kept as it is, because `do_mustering` and
    /// `do_defending` act on that 0.
    pub fn find_muster_spot(&mut self, who: Player, slot: usize, target: Obj, flag: bool) -> bool {
        crate::muster::find_muster_spot_steps(&mut crate::muster::SimMuster::new(
            self, who, slot, target, flag,
        ))
    }

    /// `find_unit(SEARCH_ENEMY, FILTER_COMBAT)`: the nearest enemy unit of
    /// `who` with an attack within `range` of `p`.
    ///
    /// The search's leader loop stops at eight (`world::PLAYER_SLOTS`), so
    /// gaia's animals are not in its space at all — and asking the diplomacy
    /// question about them is what panicked the first fuzzed seed.
    pub(crate) fn nearest_enemy_attacker(&self, who: Player, p: Pos, range: i32) -> Option<usize> {
        let mut best: Option<(i32, usize)> = None;
        for (i, u) in self.units.iter().enumerate() {
            if !u.alive() || !u.on_map || u.is_gaia() || !self.is_enemy(who, u.owner) {
                continue;
            }
            if self.attack_of(Obj::Unit(i)) == 0 {
                continue;
            }
            let d = dist(u.pos, p);
            if d <= range && best.is_none_or(|(bd, _)| d < bd) {
                best = Some((d, i));
            }
        }
        best.map(|(_, i)| i)
    }

    // ---- transporting (docs/TRANSPORT.md §8.2) ----

    /// `Region::go_here(r, who)` (`docs/TRANSPORT.md` §9.4) — all three
    /// bits since 2026-09-01, when `Region.flags` started arriving from the
    /// dump's `REGIONS` block: bit 1 is "free to settle", and it is what
    /// `think_civilian_transport` (§7) needs to find an island at all.
    ///
    /// **Every other leader's count is `LeaderData +0x125e`**, which the
    /// original's census keeps for a human too (`docs/AI.md` §23.1) and
    /// this crate's does not yet run, so it is read through
    /// [`Sim::leader_reg_cities`]'s recount. With the census's zero the
    /// human's home island answered neither bit, and East Indies' scout,
    /// out of land on 11549, sailed for the wrong one (§9.4, item 642).
    pub fn go_here(&self, r: u16, who: Player) -> i32 {
        let w = who as usize;
        let census = &self.ai[w].census;
        let mine = census.reg_cities.get(r as usize).copied().unwrap_or(0);
        let peasants = census.reg_peasants.get(r as usize).copied().unwrap_or(0);
        go_here_bits(
            self.world.region_flags(r) & 8 != 0,
            self.world.region_size(r),
            w,
            mine,
            peasants,
            self.strategy_of(who, Some(r)) & 4 != 0,
            self.players.len(),
            |i| !self.defeated[i],
            |i| self.leader_reg_cities(i as Player, r),
            |i| self.is_enemy(who, i as Player),
        )
    }

    /// `Army::do_transporting`: a one-shot retarget to the best land region
    /// the census wants; hands the army to marching. The steps are
    /// [`do_transporting_steps`]' (item 3259's sweep), over this crate's
    /// [`TransSim`].
    fn do_transporting(&mut self, who: Player, slot: usize) {
        do_transporting_steps(&mut TransSim {
            sim: self,
            who,
            slot,
        });
    }

    // ---- the Armies queries (§15) ----

    /// The supply-wagon and hero caps of `find_army` / `find_local_army`.
    ///
    /// **The arm is the joiner's `is_supply`, `unit_flags2 & 0x40` alone**
    /// (`find_local_army@006f32e0`, `find_army@006f34f0`: vslot `0xcc`, then
    /// `is_hero`'s `& 0x20` only when it is clear), and the count is the
    /// army's wagons. A government patriot `FROM General` carries both bits
    /// and is capped by the wagons: run820's Despot `1/191` refuses army 0's
    /// three wagons and joins army 5's two, group 89 (`docs/AI.md` §213).
    fn army_admits(
        &self,
        who: Player,
        slot: usize,
        unit: Option<usize>,
        supply_cap: i32,
        hero_cap: i32,
    ) -> bool {
        let Some(u) = unit else { return true };
        let supply = self.units[u]
            .ty
            .is_some_and(|t| self.unit_types[t].cols.flag2(uflags2::SUPPLY_OR_HERO));
        if supply {
            return self.army_count_supply(who, slot) < supply_cap;
        }
        if self.is_general(u) {
            return self.army_count_generals(who, slot) < hero_cap;
        }
        true
    }

    /// `Armies::find_army`: the nearest valid army in the point's region,
    /// within `max_dist` if given, with the supply and hero caps when a
    /// unit is given. Returns the slot and the distance (`find_dist`).
    pub fn find_army(
        &self,
        who: Player,
        p: Pos,
        max_dist: i32,
        unit: Option<usize>,
    ) -> Option<(usize, i32)> {
        let w = who as usize;
        // `WorldData::get_tregion`, with the coastal `region2` refinement
        // (`find_army@006f34f0`'s first call): a ship on the ocean tile of
        // a coastal cell is in the sea, not the land the cell belongs to.
        let reg = self.world.tregion_alt(p.tile());
        let mut best: Option<(usize, i32)> = None;
        for (s, a) in self.armies[w].valid() {
            if a.reg != reg {
                continue;
            }
            let d = dist(p, a.pos);
            if best.is_some_and(|(_, bd)| bd < d) || (max_dist >= 0 && max_dist < d) {
                continue;
            }
            if !self.army_admits(who, s, unit, 3, 2) {
                continue;
            }
            best = Some((s, d));
        }
        best
    }

    /// `Armies::find_local_army`: in the region, not marching on someone
    /// else, mustering or at difficulty > 2; an empty army sorts last.
    pub fn find_local_army(
        &self,
        who: Player,
        p: Pos,
        unit: Option<usize>,
    ) -> Option<(usize, i32)> {
        let w = who as usize;
        let reg = self.world.tregion(p.tile());
        let diff = self.ai_difficulty();
        let mut best: Option<(usize, i32)> = None;
        for (s, a) in self.armies[w].valid() {
            if a.reg != reg {
                continue;
            }
            if let Some(t) = a.target
                && self.owner_of(t) != who
            {
                continue;
            }
            if !(diff > 2 || a.status & status::MUSTERING != 0) {
                continue;
            }
            let d = if a.num_units == 0 {
                90_000_000
            } else {
                dist(p, a.pos)
            };
            if best.is_some_and(|(_, bd)| d > bd) {
                continue;
            }
            if !self.army_admits(who, s, unit, 3, 2) {
                continue;
            }
            best = Some((s, d));
        }
        best
    }

    /// `Armies::find_useful_army`: the ping's finder — distance per captain.
    pub fn find_useful_army(&self, who: Player, p: Pos) -> Option<usize> {
        let w = who as usize;
        let reg = self.world.tregion(p.tile());
        let level = self.transport_level(who) as i32 != 0;
        let mut best: Option<(usize, i32)> = None;
        for (s, a) in self.armies[w].valid() {
            if a.num_captains == 0 || !((!a.navy && level) || a.reg == reg) {
                continue;
            }
            let mut d = dist(p, a.pos);
            if a.reg != reg {
                d *= 3;
            }
            let score = d / a.num_captains;
            if best.is_none_or(|(_, bs)| score <= bs) {
                best = Some((s, score));
            }
        }
        best.map(|(s, _)| s)
    }

    /// `Armies::find_aggressive_army`: the first army of two or more
    /// captains, not mustering, standing outside its owner's territory.
    pub fn find_aggressive_army(&self, who: Player) -> Option<usize> {
        self.armies[who as usize]
            .valid()
            .find(|(_, a)| {
                aggressive(
                    a.valid,
                    a.num_captains,
                    a.status,
                    crate::place::owner_byte(self.world.owner(a.pos.cell())),
                    i32::from(who),
                )
            })
            .map(|(s, _)| s)
    }

    /// `Armies::num_armies(who, mask, reg)`.
    pub fn num_armies(&self, who: Player, mask: i32, reg: Option<u16>) -> i32 {
        self.armies[who as usize]
            .valid()
            .filter(|(_, a)| a.status & mask != 0 && (reg.is_none() || a.reg == reg))
            .count() as i32
    }

    /// `Armies::update_city`: a captured city's targeters follow it; the
    /// new owner's own armies are hurried.
    pub fn armies_update_city(&mut self, old: Obj, new: Obj, new_who: Player) {
        for w in 0..self.players.len() {
            if self.nation[w].human {
                continue;
            }
            for s in 0..SLOTS {
                let a = &mut self.armies[w].list[s];
                if a.valid && a.target == Some(old) {
                    a.target = Some(new);
                    if new_who as usize == w {
                        a.status |= status::HURRY;
                    }
                }
            }
        }
    }

    /// `Armies::emergency(who)`: every army drops its target and ticks at
    /// once. From `do_damage` on an AI leader's object.
    pub fn armies_emergency(&mut self, who: Player) {
        let w = who as usize;
        if self.nation[w].human || self.defeated[w] {
            return;
        }
        for s in 0..SLOTS {
            if !self.armies[w].list[s].valid {
                continue;
            }
            self.armies[w].list[s].target = None;
            self.army_process(who, s, true);
        }
    }

    /// `Armies::diplo_change(who)`: every army ticks at once.
    pub fn armies_diplo_change(&mut self, who: Player) {
        let w = who as usize;
        if self.nation[w].human || self.defeated[w] {
            return;
        }
        for s in 0..SLOTS {
            if self.armies[w].list[s].valid {
                self.army_process(who, s, true);
            }
        }
    }

    /// `Armies::leader_defeated(who)`: `Army::stop` on every army — which
    /// is `Group::action_halt(g, 0)` on each of its groups (§14,
    /// `docs/GROUPS.md` §7); the slots stay valid.
    pub fn armies_leader_defeated(&mut self, who: Player) {
        let w = who as usize;
        for s in 0..SLOTS {
            if !self.armies[w].list[s].valid {
                continue;
            }
            let g = self.army_group(who, s);
            self.group_action_halt(&g, 0);
        }
    }

    /// `City::close`'s army pass: armies mustering at the city close,
    /// armies targeting its building drop the target.
    pub fn armies_city_closed(&mut self, c: usize, building: usize) {
        let owner = self.cities[c].owner as usize;
        for w in 0..self.players.len() {
            for s in 0..SLOTS {
                let a = &self.armies[w].list[s];
                if !a.valid {
                    continue;
                }
                if w == owner && a.city == Some(c) && a.status & status::MUSTERING != 0 {
                    self.close_army(w as Player, s);
                } else if a.target == Some(Obj::Building(building)) {
                    self.armies[w].list[s].target = None;
                }
            }
        }
    }

    /// The census's step 16 (`docs/AI.md` §2.3): for each region holding
    /// a city of mine with fewer than two mustering-or-defending armies,
    /// seed one at the best-scoring city there without a mustering army.
    pub(crate) fn census_seed_army(&mut self, who: Player) {
        let w = who as usize;
        let cities = self.cities_of(who);
        let mut regions: Vec<Option<u16>> = Vec::new();
        for &c in &cities {
            let r = self.cities[c].reg;
            if !regions.contains(&r) {
                regions.push(r);
            }
        }
        // `plan_strategy` visits region ids in increasing order. City
        // creation order changes free-slot allocation and each army's
        // processing phase (docs/ARMY.md §24).
        regions.sort_unstable();
        for reg in regions {
            let here = self.armies[w]
                .valid()
                .filter(|(_, a)| {
                    a.reg == reg && a.status & (status::MUSTERING | status::DEFENDING) != 0
                })
                .count();
            if here >= 2 {
                continue;
            }
            let mut best: Option<(i32, usize)> = None;
            for &k in &cities {
                if self.cities[k].reg != reg {
                    continue;
                }
                if self.armies[w]
                    .valid()
                    .any(|(_, a)| a.city == Some(k) && a.status & status::MUSTERING != 0)
                {
                    continue;
                }
                let mut trainers = 0;
                let mut docks = 0;
                for m in self.city_chain(k) {
                    let b = &self.buildings[m];
                    if !b.alive || !b.active {
                        continue;
                    }
                    let Some(t) = b.ty else { continue };
                    let types = &self.build_types;
                    if build::is(types, t, Ident::Barracks)
                        || build::is(types, t, Ident::SiegeFactory)
                        || build::is(types, t, Ident::Stable)
                    {
                        trainers += 1;
                    }
                    if build::is(types, t, Ident::Dock) {
                        docks += 1;
                    }
                }
                let mut v = self.city_level_of(k) * 20 * (trainers * 4 + 1 + docks);
                if self.cities[k].no_heal {
                    v /= 3;
                }
                v *= match self.find_army(who, self.cities[k].pos, -1, None) {
                    None => 12,
                    Some((_, d)) => d / 4 + 6,
                };
                if best.is_none_or(|(bv, _)| bv <= v) {
                    best = Some((v, k));
                }
            }
            if let Some((_, k)) = best {
                self.init_army(who, Some(k));
            }
        }
    }
}

/// **`Region::go_here@006810f0`'s arithmetic** (item 2287; `docs/TRANSPORT.md`
/// §9.4). Bit 1: a region whose `flags & 8` is set, where `who` has neither
/// a city (`LeaderData +0x125e`) nor a peasant (`+0xfde`), and no **other**
/// active leader (`byte 0 & 2`) holds densely — `size < cities × 200`, the
/// first such leader ends the walk. Then, for every active leader that is
/// neither `who` nor its team and is an enemy, with a city there: bit 2
/// when `who` has none there or its strategy cell (`+0xa68`) carries `4`,
/// else bit 4.
#[allow(clippy::too_many_arguments)]
pub(crate) fn go_here_bits(
    settle: bool,
    size: i32,
    who: usize,
    mine: i32,
    mine_peasants: i32,
    weak: bool,
    leaders: usize,
    active: impl Fn(usize) -> bool,
    cities: impl Fn(usize) -> i32,
    enemy: impl Fn(usize) -> bool,
) -> i32 {
    let mut g = 0;
    if settle && mine == 0 && mine_peasants == 0 {
        let dense = (0..leaders).any(|i| {
            i != who && active(i) && {
                let theirs = cities(i);
                theirs != 0 && size < theirs * 200
            }
        });
        if !dense {
            g |= 1;
        }
    }
    for i in 0..leaders {
        if active(i) && i != who && enemy(i) && cities(i) != 0 {
            g |= if mine == 0 || weak { 2 } else { 4 };
        }
    }
    g
}

/// **`Army::use_generals@006f4c30`'s walk** (item 2287): the members of the
/// army's groups in order (`+0x54`, `+0x96` of them; each group's shorts
/// at `+0x8cc`, a negative id or member skipped), each in use (vslot
/// `+8`) and, asked only then, a hero (`is_hero`, `type +0x2b8 & 0x20`)
/// offered `think_spellcaster`; the first that casts ends the walk.
pub(crate) fn generals_walk<C>(
    cx: &mut C,
    members: impl IntoIterator<Item = usize>,
    mut in_use: impl FnMut(&mut C, usize) -> bool,
    mut hero: impl FnMut(&mut C, usize) -> bool,
    mut cast: impl FnMut(&mut C, usize) -> bool,
) {
    for u in members {
        if in_use(cx, u) && hero(cx, u) && cast(cx, u) {
            return;
        }
    }
}

/// **`Army::set_stance@006f8750`'s steps** (item 2341): for each of the
/// army's group slots (`+0x54`, the count the short `+0x96`, re-read every
/// pass), a slot `>= 0` names a group (`groups` `+0x10`, stride `0x9d4`);
/// one whose `num` (`+0xc`) is non-zero and whose
/// `GroupData::get_stance_type@0070d370` is `STANCE_COMBAT` (0) is handed
/// `Group::action_stance@0070d440(s)`, in slot order. The callees are the
/// caller's: `count`, `slot_at(i)`, `num(g)`, `is_combat(g)` and
/// `action(g, s)`.
pub(crate) fn set_stance_steps<C>(
    cx: &mut C,
    count: impl Fn(&C) -> i16,
    slot_at: impl Fn(&C, usize) -> i32,
    num: impl Fn(&C, i32) -> i32,
    mut is_combat: impl FnMut(&mut C, i32) -> bool,
    mut action: impl FnMut(&mut C, i32, i32),
    s: i32,
) {
    let mut i = 0usize;
    while (i as i32) < i32::from(count(cx)) {
        let g = slot_at(cx, i);
        if g >= 0 && num(cx, g) != 0 && is_combat(cx, g) {
            action(cx, g, s);
        }
        i += 1;
    }
}

/// What `Army::do_forming@006f43c0` asks of the army, its groups and the
/// world (item 3250): the swept steps [`do_forming_steps`] read the world
/// only through this, so the sweep's port is a view over the emulated
/// original's windows and the crate's is [`FormSim`]. A group is the id the
/// army's slot `i` (`+0x54 + 4i`) names; this crate has the one.
pub(crate) trait FormView {
    /// `Army::is_engaged@006f56d0`.
    fn engaged(&mut self) -> bool;
    /// The short at `+0x96`: the army's group slots.
    fn group_count(&self) -> i32;
    /// The muster cell (`+0x48`, `+0x4c`).
    fn muster(&self) -> Cell;
    /// `+0x50`.
    fn muster_angle(&self) -> Angle;
    /// The group id in slot `i` (`+0x54 + 4i`); negative is empty.
    fn group_id(&self, i: i32) -> i32;
    /// The group record's `+0xc` is nonzero (it has units).
    fn group_populated(&mut self, g: i32) -> bool;
    /// The group record's `+8` = the army's id (the short at `+2`).
    fn group_claim(&mut self, g: i32);
    /// `+0x2c != 0`.
    fn hurry(&self) -> bool;
    /// `+0x34 >= 0`: the army holds a target's owner.
    fn has_target_who(&self) -> bool;
    /// `+0x34 != leaders.list[me].+8`, the stance rule's own test.
    fn target_who_differs(&self) -> bool;
    /// The stance rule's war test on the target's owner.
    fn target_enemy(&mut self) -> bool;
    /// The Military age's rule: `(age ^ 0x63187) < 4`.
    fn young(&mut self) -> bool;
    /// `Army::count(COUNT_SIEGE, 0)@006f9120`.
    fn siege_count(&mut self) -> i32;
    /// `Army::set_stance@006f8750`.
    fn set_stance(&mut self, s: Stance);
    /// `ObjectsData::find_building(.., SEARCH_ENEMY, me, 0x300, 0x200,
    /// FILTER_COMBAT) >= 0` at `p`.
    fn enemy_combat_building_near(&mut self, p: Pos) -> bool;
    /// `+0x34 == me` (the short at `+0x94`), the friend test's own.
    fn target_is_me(&self) -> bool;
    /// `LeaderData::is_ally@006edb50(+0x34)`.
    fn target_ally(&mut self) -> bool;
    /// `+0x30 >= 0`: the army holds a target object.
    fn has_target_obj(&self) -> bool;
    /// The target's city record `flags` (`+4`), or `None` for a negative
    /// city number.
    fn target_city_flags(&mut self) -> Option<i32>;
    /// `project@0092cf40(p, angle, d)`: the point `d` along `angle`.
    fn project(&mut self, p: Pos, angle: Angle, d: i32) -> Pos;
    /// `sin_table@00a46a00(folded, d)`: the sine component of the folded
    /// angle (`ecx`) and signed distance (`edx`).
    fn sin_table(&mut self, folded: i32, d: i32) -> i32;
    /// `Group::action_move_to@0070fba0`.
    #[allow(clippy::too_many_arguments)]
    fn form_move_to(
        &mut self,
        g: i32,
        to: Pos,
        queue: QueuePos,
        set_angle: bool,
        angle: Angle,
        kind: MoveKind,
        action: bool,
    );
    /// `Group::action_siege_attack_to@0070d830`.
    fn siege_attack_to(&mut self, g: i32, to: Pos, angle: Angle);
}

/// **`Army::do_forming@006f43c0`'s steps** (item 3250): an engaged army
/// does nothing (returns `false`). Otherwise the muster cell's centre is
/// projected `num_groups x 0xc0` along the muster angle, and every
/// populated group is claimed, gets the stance rule (a hostile target before
/// Gunpowder with no hurry and no siege: raid), and one order at the
/// current point: the siege order when an enemy combat building stands
/// within a cell and the army has siege, or when neither hurry nor an
/// attacked friendly city asks for the move; the attack-move otherwise
/// (stance Aggressive first). The point then steps half a cell back along
/// `muster_angle - 0x80000000` for the next group.
pub(crate) fn do_forming_steps<V: FormView>(v: &mut V) -> bool {
    if v.engaged() {
        return false;
    }
    let n = v.group_count();
    let ang = v.muster_angle();
    let mut at = v.project(cell_centre(v.muster()), ang, n * 0xc0);
    for i in 0..n {
        let g = v.group_id(i);
        if g < 0 || !v.group_populated(g) {
            continue;
        }
        v.group_claim(g);
        if v.has_target_who()
            && v.target_who_differs()
            && v.target_enemy()
            && !v.hurry()
            && v.young()
            && v.siege_count() == 0
        {
            v.set_stance(Stance::Raid);
        }
        let siege = v.enemy_combat_building_near(at) && v.siege_count() != 0;
        let mv = !siege
            && (v.hurry()
                || ((v.target_is_me() || v.target_ally())
                    && v.has_target_obj()
                    && v.target_city_flags().is_some_and(|f| f & 3 == 3)));
        if mv {
            v.set_stance(Stance::Aggressive);
            v.form_move_to(g, at, QueuePos::New, true, ang, MoveKind::AttackTo, true);
        } else {
            v.siege_attack_to(g, at, ang);
        }
        // The half-cell step back along `ang - 0x80000000`: the inline fold
        // (a negative angle negates the distance and drops its sign bit; the
        // second quarter mirrors) ahead of `sin_table`, once for x and once,
        // a quarter turn on, for y.
        let back = ang.0.wrapping_add(i32::MIN);
        let fold = |a: i32| -> (i32, i32) {
            let (mut a, mut d) = (a, 0x180);
            if a < 0 {
                d = -0x180;
                a &= 0x7fff_ffff;
            }
            (
                if a & 0x4000_0000 != 0 {
                    0x7fff_ffff - a
                } else {
                    a
                },
                d,
            )
        };
        let (e, d) = fold(back);
        let dx = v.sin_table(e, d);
        let (e, d) = fold(back.wrapping_add(0x4000_0000));
        let dy = v.sin_table(e, d);
        at = Pos::new(at.x.wrapping_add(dx), at.y.wrapping_sub(dy));
    }
    true
}

/// This crate's [`FormView`]: the army `slot` of `who`, whose one group
/// carries every unit. Where the crate models less than the original, the
/// answer is the crate's rule: the group's id is the slot, the hostile and
/// ally tests are on the target object's owner, and a target building's
/// city is the friend test's record.
struct FormSim<'a> {
    sim: &'a mut Sim,
    who: Player,
    slot: usize,
}

impl FormSim<'_> {
    fn tw(&self) -> Option<Player> {
        self.sim.armies[self.who as usize].list[self.slot]
            .target
            .map(|t| self.sim.owner_of(t))
    }
}

impl FormView for FormSim<'_> {
    fn engaged(&mut self) -> bool {
        self.sim.army_is_engaged(self.who, self.slot)
    }
    fn group_count(&self) -> i32 {
        self.sim.armies[self.who as usize].list[self.slot].num_groups()
    }
    fn muster(&self) -> Cell {
        self.sim.armies[self.who as usize].list[self.slot].muster
    }
    fn muster_angle(&self) -> Angle {
        self.sim.armies[self.who as usize].list[self.slot].muster_angle
    }
    fn group_id(&self, i: i32) -> i32 {
        i
    }
    fn group_populated(&mut self, _: i32) -> bool {
        true
    }
    fn group_claim(&mut self, _: i32) {}
    fn hurry(&self) -> bool {
        self.sim.armies[self.who as usize].list[self.slot].hurry != 0
    }
    fn has_target_who(&self) -> bool {
        self.tw().is_some()
    }
    fn target_who_differs(&self) -> bool {
        self.tw().is_some_and(|tw| tw != self.who)
    }
    fn target_enemy(&mut self) -> bool {
        self.tw().is_some_and(|tw| self.sim.is_enemy(self.who, tw))
    }
    fn young(&mut self) -> bool {
        (self.sim.tech[self.who as usize].military_level() as i32) < 4
    }
    fn siege_count(&mut self) -> i32 {
        self.sim.army_count_siege(self.who, self.slot)
    }
    fn set_stance(&mut self, s: Stance) {
        self.sim.army_set_stance(self.who, self.slot, s);
    }
    fn enemy_combat_building_near(&mut self, p: Pos) -> bool {
        let who = self.who;
        self.sim.buildings.iter().any(|b| {
            b.alive
                && b.active
                && self.sim.is_enemy(who, b.owner)
                && b.ty.is_some_and(|t| self.sim.build_types[t].attack != 0)
                && dist(b.pos, p) <= UNITS_PER_CELL
        })
    }
    fn target_is_me(&self) -> bool {
        self.tw() == Some(self.who)
    }
    fn target_ally(&mut self) -> bool {
        self.tw().is_some_and(|tw| self.sim.is_ally(self.who, tw))
    }
    fn has_target_obj(&self) -> bool {
        self.sim.armies[self.who as usize].list[self.slot]
            .target
            .is_some()
    }
    fn target_city_flags(&mut self) -> Option<i32> {
        let t = self.sim.armies[self.who as usize].list[self.slot].target?;
        match t {
            Obj::Building(b) => self.sim.buildings[b].city.map(|c| {
                let cd = &self.sim.cities[c];
                i32::from(cd.alive) | i32::from(cd.no_heal) << 1
            }),
            _ => None,
        }
    }
    fn project(&mut self, p: Pos, angle: Angle, d: i32) -> Pos {
        step_along(p, angle, d)
    }
    fn sin_table(&mut self, folded: i32, d: i32) -> i32 {
        movement::sin_component(Angle(folded), d)
    }
    fn form_move_to(
        &mut self,
        _: i32,
        to: Pos,
        queue: QueuePos,
        set_angle: bool,
        angle: Angle,
        kind: MoveKind,
        action: bool,
    ) {
        let g = self.sim.army_group(self.who, self.slot);
        self.sim
            .group_action_move_to(&g, to, queue, set_angle, angle, kind, action);
    }
    fn siege_attack_to(&mut self, _: i32, to: Pos, angle: Angle) {
        let g = self.sim.army_group(self.who, self.slot);
        self.sim.group_action_siege_attack_to(&g, to, angle);
    }
}

/// What `Army::march_to_target@006f4d80` asks of the army, its groups and
/// the world (item 3259): the swept steps [`march_to_target_steps`] read the
/// world only through this, so the sweep's port is a view over the emulated
/// original's windows and the crate's is [`MarchToSim`]. A group is the id
/// the army's slot `i` (`+0x54 + 4i`) names; this crate has the one.
pub(crate) trait MarchToView {
    /// `+0x30 >= 0`: the army holds a target object.
    fn has_target_obj(&self) -> bool;
    /// `Army::is_engaged@006f56d0`.
    fn engaged(&mut self) -> bool;
    /// `+4`.
    fn status(&self) -> i32;
    fn set_status(&mut self, s: i32);
    /// `+0x50`.
    fn muster_angle(&self) -> Angle;
    /// The army's point (`+0x38`, `+0x3c`).
    fn pos(&self) -> Pos;
    /// `sin_table@00a46a00(folded, d)`.
    fn sin_table(&mut self, folded: i32, d: i32) -> i32;
    /// The target object's point (`+0x10`, `+0x14`, each `^ 0x63637`).
    fn target_pos(&self) -> Pos;
    /// `find_angle@0092d130(dx, dy)`.
    fn find_angle(&mut self, dx: i32, dy: i32) -> Angle;
    /// `LeaderData::is_enemy@006ebaa0(target_who)`.
    fn target_enemy(&mut self) -> bool;
    /// `+0x2c != 0`.
    fn hurry(&self) -> bool;
    /// The Military age's rule: `(age ^ 0x63187) < 4`.
    fn young(&mut self) -> bool;
    /// `Army::count(COUNT_SIEGE, 0)@006f9120`.
    fn siege_count(&mut self) -> i32;
    /// `Army::set_stance@006f8750`.
    fn set_stance(&mut self, s: Stance);
    /// `ObjectsData::find_building(.., SEARCH_ENEMY, me, 0x300, 0x200,
    /// FILTER_COMBAT) >= 0` at `p`.
    fn enemy_combat_building_near(&mut self, p: Pos) -> bool;
    /// `+0x34 == me` (the short at `+0x94`).
    fn target_is_me(&self) -> bool;
    /// `LeaderData::is_ally@006edb50(+0x34)`.
    fn target_ally(&mut self) -> bool;
    /// The target's city record `flags` (`+4`), or `None` for a negative
    /// city number.
    fn target_city_flags(&mut self) -> Option<i32>;
    /// The short at `+0x96`: the army's group slots.
    fn group_count(&self) -> i32;
    /// The group id in slot `i` (`+0x54 + 4i`); negative is empty.
    fn group_id(&self, i: i32) -> i32;
    /// The group record's `+0xc` is nonzero (it has units).
    fn group_populated(&mut self, g: i32) -> bool;
    /// The group record's `+8` = the army's id (the short at `+2`).
    fn group_claim(&mut self, g: i32);
    /// The target object's `flags & 0x20`: a city centre.
    fn target_city_centre(&mut self) -> bool;
    /// `vslot 0x11c(0)` of the target object: its hits.
    fn target_hits(&mut self) -> i32;
    /// The target object's `+0x24`: its damage.
    fn target_damage(&mut self) -> i32;
    /// `Group::action_move_to@0070fba0`.
    #[allow(clippy::too_many_arguments)]
    fn march_move_to(
        &mut self,
        g: i32,
        to: Pos,
        queue: QueuePos,
        set_angle: bool,
        angle: Angle,
        kind: MoveKind,
        action: bool,
    );
    /// `Group::action_siege_attack_to@0070d830`.
    fn siege_attack_to(&mut self, g: i32, to: Pos, angle: Angle);
}

/// **`Army::march_to_target@006f4d80`'s steps** (item 3259): no target does
/// nothing. An army that is not engaged is forming and marching
/// (`status = (status & ~4) | 0x12`). An engaged one that has already
/// ordered (`status & 4`) does nothing. Otherwise the formation origin is
/// the army's point stepped `0x300` along `muster_angle - 0x80000000`
/// (inline fold ahead of `sin_table`, `x + sin`, `y - sin` a quarter turn
/// on); the facing is `find_angle(origin - target)`; the stance rule (an
/// enemy owner, no hurry, before Gunpowder, no siege: raid); and the
/// move-or-siege choice at the origin (an enemy combat building within a
/// cell with siege forces the siege order, else `hurry` or an attacked
/// friendly city picks the move). Every populated group is claimed and
/// ordered: a city-centre target with nine tenths of its hits lost is walked
/// to directly, else the move or siege order at the origin, which then steps
/// `0x180` back for the next group. `status |= 4`. The move order carries no
/// stance change (`do_forming`'s does).
pub(crate) fn march_to_target_steps<V: MarchToView>(v: &mut V) {
    if !v.has_target_obj() {
        return;
    }
    if !v.engaged() {
        let s = v.status();
        v.set_status((s & !4) | 0x12);
        return;
    }
    if v.status() & 4 != 0 {
        return;
    }
    // The fold ahead of `sin_table`: a negative angle negates the distance
    // and drops its sign bit; the second quarter mirrors.
    let fold = |a: i32, d0: i32| -> (i32, i32) {
        let (mut a, mut d) = (a, d0);
        if a < 0 {
            d = -d0;
            a &= 0x7fff_ffff;
        }
        (
            if a & 0x4000_0000 != 0 {
                0x7fff_ffff - a
            } else {
                a
            },
            d,
        )
    };
    let ang = v.muster_angle();
    let back = ang.0.wrapping_add(i32::MIN);
    let here = v.pos();
    let (e, d) = fold(back, 0x300);
    let dx = v.sin_table(e, d);
    let (e, d) = fold(back.wrapping_add(0x4000_0000), 0x300);
    let dy = v.sin_table(e, d);
    let mut at = Pos::new(here.x.wrapping_add(dx), here.y.wrapping_sub(dy));
    let tp = v.target_pos();
    let angle = v.find_angle(at.x.wrapping_sub(tp.x), at.y.wrapping_sub(tp.y));
    if v.target_enemy() && !v.hurry() && v.young() && v.siege_count() == 0 {
        v.set_stance(Stance::Raid);
    }
    let siege = v.enemy_combat_building_near(at) && v.siege_count() != 0;
    let mv = !siege
        && (v.hurry()
            || ((v.target_is_me() || v.target_ally())
                && v.target_city_flags().is_some_and(|f| f & 3 == 3)));
    let n = v.group_count();
    for i in 0..n {
        let g = v.group_id(i);
        if g < 0 || !v.group_populated(g) {
            continue;
        }
        v.group_claim(g);
        if v.target_city_centre() && {
            let h = v.target_hits().wrapping_mul(9) / 10;
            v.target_damage() >= h
        } {
            v.march_move_to(g, tp, QueuePos::New, true, angle, MoveKind::AttackTo, true);
        } else if mv {
            v.march_move_to(g, at, QueuePos::New, true, angle, MoveKind::AttackTo, true);
        } else {
            v.siege_attack_to(g, at, angle);
        }
        let (e, d) = fold(back, 0x180);
        let dx = v.sin_table(e, d);
        let (e, d) = fold(back.wrapping_add(0x4000_0000), 0x180);
        let dy = v.sin_table(e, d);
        at = Pos::new(at.x.wrapping_add(dx), at.y.wrapping_sub(dy));
    }
    let s = v.status();
    v.set_status(s | 4);
}

/// This crate's [`MarchToView`]: the army `slot` of `who`, whose one group
/// carries every unit. Where the crate models less than the original, the
/// answer is the crate's rule: the group's id is the slot, a target building
/// that is its city's centre is the city-centre object, its damage is
/// `hits - health`, and the hostile and ally tests are on the target
/// object's owner.
struct MarchToSim<'a> {
    sim: &'a mut Sim,
    who: Player,
    slot: usize,
}

impl MarchToSim<'_> {
    fn army(&self) -> &Army {
        &self.sim.armies[self.who as usize].list[self.slot]
    }
    fn tw(&self) -> Option<Player> {
        self.army().target.map(|t| self.sim.owner_of(t))
    }
}

impl MarchToView for MarchToSim<'_> {
    fn has_target_obj(&self) -> bool {
        self.army().target.is_some()
    }
    fn engaged(&mut self) -> bool {
        self.sim.army_is_engaged(self.who, self.slot)
    }
    fn status(&self) -> i32 {
        self.army().status
    }
    fn set_status(&mut self, s: i32) {
        self.sim.armies[self.who as usize].list[self.slot].status = s;
    }
    fn muster_angle(&self) -> Angle {
        self.army().muster_angle
    }
    fn pos(&self) -> Pos {
        self.army().pos
    }
    fn sin_table(&mut self, folded: i32, d: i32) -> i32 {
        movement::sin_component(Angle(folded), d)
    }
    fn target_pos(&self) -> Pos {
        self.army()
            .target
            .map_or(Pos::new(0, 0), |t| self.sim.pos_of(t))
    }
    fn find_angle(&mut self, dx: i32, dy: i32) -> Angle {
        let angle = find_angle(dx, dy);
        self.sim.armies[self.who as usize].list[self.slot].angle = angle;
        angle
    }
    fn target_enemy(&mut self) -> bool {
        self.tw()
            .is_some_and(|tw| tw != self.who && self.sim.is_enemy(self.who, tw))
    }
    fn hurry(&self) -> bool {
        self.army().hurry != 0
    }
    fn young(&mut self) -> bool {
        (self.sim.tech[self.who as usize].military_level() as i32) < 4
    }
    fn siege_count(&mut self) -> i32 {
        self.sim.army_count_siege(self.who, self.slot)
    }
    fn set_stance(&mut self, s: Stance) {
        self.sim.army_set_stance(self.who, self.slot, s);
    }
    fn enemy_combat_building_near(&mut self, p: Pos) -> bool {
        let who = self.who;
        self.sim.buildings.iter().any(|b| {
            b.alive
                && b.active
                && self.sim.is_enemy(who, b.owner)
                && b.ty.is_some_and(|t| self.sim.build_types[t].attack != 0)
                && dist(b.pos, p) <= UNITS_PER_CELL
        })
    }
    fn target_is_me(&self) -> bool {
        self.tw() == Some(self.who)
    }
    fn target_ally(&mut self) -> bool {
        self.tw().is_some_and(|tw| self.sim.is_ally(self.who, tw))
    }
    fn target_city_flags(&mut self) -> Option<i32> {
        match self.army().target? {
            Obj::Building(b) => self.sim.buildings[b].city.map(|c| {
                let cd = &self.sim.cities[c];
                i32::from(cd.alive) | i32::from(cd.no_heal) << 1
            }),
            Obj::Unit(_) => None,
        }
    }
    fn group_count(&self) -> i32 {
        self.army().num_groups()
    }
    fn group_id(&self, i: i32) -> i32 {
        i
    }
    fn group_populated(&mut self, _: i32) -> bool {
        true
    }
    fn group_claim(&mut self, _: i32) {}
    fn target_city_centre(&mut self) -> bool {
        match self.army().target {
            Some(Obj::Building(b)) => {
                let bd = &self.sim.buildings[b];
                bd.city.is_some_and(|c| self.sim.cities[c].building == b)
            }
            _ => false,
        }
    }
    fn target_hits(&mut self) -> i32 {
        match self.army().target {
            Some(Obj::Building(b)) => self.sim.buildings[b].hits,
            _ => 0,
        }
    }
    fn target_damage(&mut self) -> i32 {
        match self.army().target {
            Some(Obj::Building(b)) => self.sim.buildings[b].hits - self.sim.buildings[b].health,
            _ => 0,
        }
    }
    fn march_move_to(
        &mut self,
        _: i32,
        to: Pos,
        queue: QueuePos,
        set_angle: bool,
        angle: Angle,
        kind: MoveKind,
        action: bool,
    ) {
        let g = self.sim.army_group(self.who, self.slot);
        self.sim
            .group_action_move_to(&g, to, queue, set_angle, angle, kind, action);
    }
    fn siege_attack_to(&mut self, _: i32, to: Pos, angle: Angle) {
        let g = self.sim.army_group(self.who, self.slot);
        self.sim.group_action_siege_attack_to(&g, to, angle);
    }
}

/// What `Army::do_marching@006f3df0` asks of the army and its target (item
/// 3238): the swept steps [`do_marching_steps`] read the world only through
/// this, so the sweep's port is a view over the emulated original's windows
/// and the crate's is [`MarchSim`].
pub(crate) trait MarchView {
    /// `Army::count(COUNT_ATTACK, 0)@006f9120`.
    fn attack_count(&mut self) -> i32;
    /// `Army::close@006f8ea0`.
    fn close(&mut self);
    /// `field_0x30 >= 0`: the army holds a target.
    fn has_target(&self) -> bool;
    /// `field_0x30 = field_0x34 = -1`.
    fn clear_target(&mut self);
    fn status(&self) -> i32;
    fn set_status(&mut self, s: i32);
    /// The target's data record `flags & 1` (`vslot 0xac`, `+8`).
    fn target_valid(&mut self) -> bool;
    /// The target's data record `flags & 0x20`: a city centre.
    fn target_city_centre(&mut self) -> bool;
    /// `LeaderData::is_enemy@006ebaa0(target_who)`.
    fn target_enemy(&mut self) -> bool;
    /// `Build::vftable ? build_masks & 0x20 : vslot 0x188()`.
    fn target_under_attack(&mut self) -> bool;
    /// The city record's `flags` (`+4`) of the target's `+0x72` city, or
    /// `None` for a negative city number.
    fn target_city_flags(&mut self) -> Option<i32>;
    /// `field_0x24 != 0`.
    fn navy(&self) -> bool;
    /// `CityData::is_unassimilated@00738030`.
    fn city_unassimilated(&mut self) -> bool;
    /// `vslot 0x11c(0)` of the target's data: its hits.
    fn target_hits(&mut self) -> i32;
    /// The target's data `+0x24`: its damage.
    fn target_damage(&mut self) -> i32;
    /// `LeaderData::is_ally@006edb50`.
    fn ally(&mut self) -> bool;
    /// `leaders.list[me].+0x250[target_who] != 0`.
    fn weak(&mut self) -> bool;
    /// `leaders.list[me].+0x230[target_who] != 0`.
    fn strong(&mut self) -> bool;
    /// `LeaderData::is_tribute_period@006d5b90`.
    fn tribute(&mut self) -> bool;
    /// `Army::find_target@006f69b0`; `false` when the army is gone after it
    /// (this crate's `find_target` can close it; the original's does not).
    fn find_target(&mut self) -> bool;
    /// `Army::do_forming@006f43c0`.
    fn do_forming(&mut self);
    /// `Army::march_to_target@006f4d80`.
    fn march_to_target(&mut self);
}

/// **`Army::do_marching@006f3df0`'s steps** (item 3238): nothing that can
/// attack closes the army; a target that is missing, or fails the
/// validation (a non-enemy that is not "under attack", a city that is
/// gone, a navy's non-enemy city that is not attacked, a damaged
/// assimilated one the army may not bleed), is cleared and `find_target`
/// retried (a defending army is parked at `0x30` instead; a hit forms
/// the army, a miss sets `status = 8`); otherwise a marching army
/// marches. A target whose data record is not valid is *kept* (§9).
pub(crate) fn do_marching_steps<V: MarchView>(v: &mut V) {
    if v.attack_count() < 1 {
        v.close();
        return;
    }
    let mut retarget = !v.has_target();
    if !retarget && v.target_valid() {
        if !v.target_city_centre() {
            if !v.target_enemy() && !v.target_under_attack() {
                retarget = true;
            }
        } else if let Some(flags) = v.target_city_flags() {
            if flags & 1 == 0 || (v.navy() && !v.target_enemy() && flags & 2 == 0) {
                retarget = true;
            } else if !v.target_enemy() && flags & 2 == 0 && !v.city_unassimilated() {
                let h3 = v.target_hits().wrapping_mul(3);
                if v.target_damage() < h3.wrapping_add((h3 >> 31) & 3) >> 2
                    && (v.ally() || v.weak() || !v.strong() || v.tribute())
                {
                    retarget = true;
                }
            }
        }
    }
    if retarget {
        v.clear_target();
        if v.status() & status::DEFENDING != 0 {
            v.set_status(status::DEFENDING | status::FORMING);
            return;
        }
        if !v.find_target() {
            return;
        }
        if v.has_target() {
            let s = v.status();
            v.set_status(s | status::FORMING);
            v.do_forming();
            return;
        }
        v.set_status(status::NO_TARGET);
        return;
    }
    if v.status() & status::MARCHING != 0 {
        v.march_to_target();
    }
}

/// This crate's [`MarchView`]: the army `slot` of `who`, and the target it
/// held when `do_marching` began. Where the crate models less than the
/// original, the answer is the crate's rule: no "under attack" slot (a
/// non-enemy unit is always dropped), a building is a city centre (with no
/// city when it has none), and the weak/strong/tribute tests are seams
/// (`weak` clear, `strong` set, no tribute: the ally test decides).
struct MarchSim<'a> {
    sim: &'a mut Sim,
    who: Player,
    slot: usize,
    t: Option<Obj>,
}

impl MarchSim<'_> {
    fn tw(&self) -> Player {
        self.sim.owner_of(self.t.expect("a target"))
    }
    fn city(&self) -> Option<usize> {
        match self.t {
            Some(Obj::Building(b)) => self.sim.buildings[b].city,
            _ => None,
        }
    }
}

impl MarchView for MarchSim<'_> {
    fn attack_count(&mut self) -> i32 {
        self.sim.army_count_attack(self.who, self.slot)
    }
    fn close(&mut self) {
        self.sim.close_army(self.who, self.slot);
    }
    fn has_target(&self) -> bool {
        self.sim.armies[self.who as usize].list[self.slot]
            .target
            .is_some()
    }
    fn clear_target(&mut self) {
        self.sim.armies[self.who as usize].list[self.slot].target = None;
    }
    fn status(&self) -> i32 {
        self.sim.armies[self.who as usize].list[self.slot].status
    }
    fn set_status(&mut self, s: i32) {
        self.sim.armies[self.who as usize].list[self.slot].status = s;
    }
    fn target_valid(&mut self) -> bool {
        self.t.is_some_and(|t| self.sim.active(t))
    }
    fn target_city_centre(&mut self) -> bool {
        matches!(self.t, Some(Obj::Building(_)))
    }
    fn target_enemy(&mut self) -> bool {
        self.sim.is_enemy(self.who, self.tw())
    }
    fn target_under_attack(&mut self) -> bool {
        false
    }
    fn target_city_flags(&mut self) -> Option<i32> {
        self.city().map(|c| {
            let cd = &self.sim.cities[c];
            i32::from(cd.alive) | i32::from(cd.no_heal) << 1
        })
    }
    fn navy(&self) -> bool {
        self.sim.armies[self.who as usize].list[self.slot].navy
    }
    fn city_unassimilated(&mut self) -> bool {
        self.city()
            .is_some_and(|c| self.sim.cities[c].unassimilated)
    }
    fn target_hits(&mut self) -> i32 {
        match self.t {
            Some(Obj::Building(b)) => self.sim.buildings[b].hits,
            _ => 0,
        }
    }
    fn target_damage(&mut self) -> i32 {
        match self.t {
            Some(Obj::Building(b)) => self.sim.buildings[b].hits - self.sim.buildings[b].health,
            _ => 0,
        }
    }
    fn ally(&mut self) -> bool {
        self.sim.is_ally(self.who, self.tw())
    }
    fn weak(&mut self) -> bool {
        false
    }
    fn strong(&mut self) -> bool {
        true
    }
    fn tribute(&mut self) -> bool {
        false
    }
    fn find_target(&mut self) -> bool {
        self.sim.find_target(self.who, self.slot);
        self.sim.armies[self.who as usize].list[self.slot].valid
    }
    fn do_forming(&mut self) {
        self.sim.do_forming(self.who, self.slot);
    }
    fn march_to_target(&mut self) {
        self.sim.march_to_target(self.who, self.slot);
    }
}

/// What `Army::do_transporting@006f4690` asks of the army and the world
/// (item 3259): the swept steps [`do_transporting_steps`] read the world
/// only through this, so the sweep's port is a view over the emulated
/// original's windows and the crate's is [`TransSim`].
pub(crate) trait TransView {
    /// The region the army's cell lies in is the army's `reg` (`field_0x8`).
    fn at_home(&mut self) -> bool;
    /// The army's `reg`.
    fn my_region(&self) -> Option<u16>;
    /// The regions the loop visits, in order, `my` left out: the original's
    /// `1..64` with `my` skipped.
    fn regions(&mut self, my: u16) -> Vec<u16>;
    fn region_size(&mut self, r: u16) -> i32;
    /// The leader's census strategy word for `r`.
    fn strategy(&mut self, r: u16) -> i32;
    /// `Region::go_here@006810f0(who)`.
    fn go_here(&mut self, r: u16) -> i32;
    /// `vector_dist` of the two regions' first cells.
    fn distance(&mut self, my: u16, r: u16) -> i32;
    /// The leader's cities, in the original's `0..count` order.
    fn cities(&mut self) -> Vec<usize>;
    /// City `c` is alive (`flags & 1`), in region `r`, and `flags & 2`.
    fn city_in(&mut self, c: usize, r: u16) -> bool;
    /// `CityData::get_level@00739340`.
    fn city_level(&mut self, c: usize) -> i32;
    fn set_region(&mut self, r: Option<u16>);
    /// `Army::find_target@006f69b0`; `false` when the army is gone after it
    /// (this crate's `find_target` can close it; the original's does not).
    fn find_target(&mut self) -> bool;
    fn has_target(&self) -> bool;
    /// `reg` back to the region the army's cell lies in.
    fn region_home(&mut self);
    /// The target becomes city `c`'s centre building (the original also
    /// writes `field_0x34 = who`, `field_0x20 = c`).
    fn target_city(&mut self, c: usize);
    /// `Army::find_muster_spot(target, who, 1)`.
    fn find_muster_spot(&mut self, c: usize) -> bool;
    /// A muster spot that was not found leaves the muster at the city
    /// centre's cell, one row down (up on the last row).
    fn muster_fallback(&mut self, c: usize);
    fn set_marching(&mut self);
}

/// **`Army::do_transporting@006f4690`'s steps** (item 3259): from the
/// army's own region, the best other region the census or `go_here` wants
/// by `size * (3 | 2 | 1) / (distance / 4 + 1)`, times the level of each of
/// the leader's no-heal cities there; the army is retargeted at it once and
/// handed to marching. Returns the original's 1 when it acted.
pub(crate) fn do_transporting_steps<V: TransView>(v: &mut V) -> bool {
    if !v.at_home() {
        return false;
    }
    let Some(my) = v.my_region() else {
        return false;
    };
    let (mut best_v, mut best_r, mut best_city) = (-1i32, 0u16, None::<usize>);
    for r in v.regions(my) {
        let size = v.region_size(r);
        if size <= 0 {
            continue;
        }
        let s = v.strategy(r);
        let g = v.go_here(r);
        if !(s & 2 != 0 || g & 2 != 0 || g & 4 != 0) {
            continue;
        }
        let mut val = if g & 4 != 0 {
            size.wrapping_mul(3)
        } else if s & 2 != 0 {
            size.wrapping_mul(2)
        } else {
            size
        };
        let d = v.distance(my, r);
        val /= ((d + ((d >> 31) & 3)) >> 2) + 1;
        let mut city = None;
        for c in v.cities() {
            if v.city_in(c, r) {
                val = val.wrapping_mul(v.city_level(c));
                city = Some(c);
            }
        }
        if best_v < val {
            best_v = val;
            best_r = r;
            best_city = city;
        }
    }
    if best_v <= 0 {
        return false;
    }
    v.set_region(Some(best_r));
    match best_city {
        None => {
            if !v.find_target() {
                return true;
            }
            if !v.has_target() {
                v.region_home();
                return true;
            }
        }
        Some(c) => {
            v.target_city(c);
            if !v.find_muster_spot(c) {
                v.muster_fallback(c);
            }
        }
    }
    v.set_marching();
    true
}

/// This crate's [`TransView`]: the army `slot` of `who`. **Where the crate
/// models less than the original**: the regions visited are the land ones
/// (the original has no terrain test and stops at 63), a missing first cell
/// is a zero distance, and `muster_fallback` is a no-op (SEAM: the crate's
/// `find_muster_spot` already leaves its own muster; the original overwrites
/// it with the city centre's cell, one row down, when it answers 0).
struct TransSim<'a> {
    sim: &'a mut Sim,
    who: Player,
    slot: usize,
}

impl TransSim<'_> {
    fn army(&mut self) -> &mut Army {
        &mut self.sim.armies[self.who as usize].list[self.slot]
    }
}

impl TransView for TransSim<'_> {
    fn at_home(&mut self) -> bool {
        let a = &self.sim.armies[self.who as usize].list[self.slot];
        self.sim.world.region_of(a.pos.cell()) == a.reg
    }
    fn my_region(&self) -> Option<u16> {
        self.sim.armies[self.who as usize].list[self.slot].reg
    }
    fn regions(&mut self, my: u16) -> Vec<u16> {
        self.sim
            .world
            .regions()
            .filter(|&(r, t)| t == Terrain::Land && r != my)
            .map(|(r, _)| r)
            .collect()
    }
    fn region_size(&mut self, r: u16) -> i32 {
        self.sim.world.region_size(r)
    }
    fn strategy(&mut self, r: u16) -> i32 {
        self.sim.strategy_of(self.who, Some(r))
    }
    fn go_here(&mut self, r: u16) -> i32 {
        self.sim.go_here(r, self.who)
    }
    fn distance(&mut self, my: u16, r: u16) -> i32 {
        match (
            self.sim.world.cells_in(my).next(),
            self.sim.world.cells_in(r).next(),
        ) {
            (Some(a), Some(b)) => vector_dist((a.x - b.x).abs(), (a.y - b.y).abs()),
            _ => 0,
        }
    }
    fn cities(&mut self) -> Vec<usize> {
        self.sim.cities_of(self.who)
    }
    fn city_in(&mut self, c: usize, r: u16) -> bool {
        self.sim.cities[c].reg == Some(r) && self.sim.cities[c].no_heal
    }
    fn city_level(&mut self, c: usize) -> i32 {
        self.sim.city_level_of(c)
    }
    fn set_region(&mut self, r: Option<u16>) {
        self.army().reg = r;
    }
    fn find_target(&mut self) -> bool {
        self.sim.find_target(self.who, self.slot);
        self.sim.armies[self.who as usize].list[self.slot].valid
    }
    fn has_target(&self) -> bool {
        self.sim.armies[self.who as usize].list[self.slot]
            .target
            .is_some()
    }
    fn region_home(&mut self) {
        let here = {
            let a = &self.sim.armies[self.who as usize].list[self.slot];
            self.sim.world.region_of(a.pos.cell())
        };
        self.army().reg = here;
    }
    fn target_city(&mut self, c: usize) {
        let target = Obj::Building(self.sim.cities[c].building);
        self.army().target = Some(target);
    }
    fn find_muster_spot(&mut self, c: usize) -> bool {
        let target = Obj::Building(self.sim.cities[c].building);
        self.sim.find_muster_spot(self.who, self.slot, target, true)
    }
    fn muster_fallback(&mut self, _c: usize) {}
    fn set_marching(&mut self) {
        self.army().status = status::MARCHING;
    }
}

/// [`crate::army_process_view::ArmyProcessView`] over one army of the sim.
struct SimProcess<'a> {
    s: &'a mut Sim,
    who: Player,
    slot: usize,
}

impl SimProcess<'_> {
    fn a(&self) -> &Army {
        &self.s.armies[self.who as usize].list[self.slot]
    }
    fn am(&mut self) -> &mut Army {
        &mut self.s.armies[self.who as usize].list[self.slot]
    }
}

impl crate::army_process_view::ArmyProcessView for SimProcess<'_> {
    fn human_frame(&self) -> i32 {
        self.a().human_frame
    }
    fn set_human_frame(&mut self, v: i32) {
        self.am().human_frame = v;
    }
    fn frame(&self) -> i32 {
        self.s.frame as i32
    }
    fn phase(&self) -> i32 {
        self.a().phase() as i32
    }
    fn defeated(&self) -> bool {
        self.s.defeated[self.who as usize]
    }
    fn num_captains(&self) -> i32 {
        self.a().num_captains
    }
    fn num_standard(&self) -> i32 {
        self.a().num_standard
    }
    fn num_decoys(&self) -> i32 {
        self.a().num_decoys
    }
    fn valid(&self) -> bool {
        self.a().valid
    }
    fn status(&self) -> i32 {
        self.a().status
    }
    fn set_status(&mut self, s: i32) {
        self.am().status = s;
    }
    fn navy(&self) -> bool {
        self.a().navy
    }
    fn reg(&self) -> i32 {
        self.a().reg.map_or(-1, i32::from)
    }
    fn ping_pos(&self) -> Pos {
        self.a().pos
    }
    fn muster(&self) -> (i32, i32) {
        let m = self.a().muster;
        (m.x, m.y)
    }
    fn set_muster(&mut self, x: i32, y: i32) {
        self.am().muster = Cell { x, y };
    }
    fn has_target(&self) -> bool {
        self.a().target.is_some()
    }
    fn target_owner(&self) -> i32 {
        self.a()
            .target
            .map_or(-1, |t| i32::from(self.s.owner_of(t)))
    }
    fn target_cell(&mut self) -> (i32, i32) {
        let t = self.a().target.expect("a target");
        let c = self.s.pos_of(t).cell();
        (c.x, c.y)
    }
    fn city_count(&self) -> i32 {
        self.s.cities_of(self.who).len() as i32
    }
    fn set_scan(&mut self, _i: i32) {}
    fn city_alive(&mut self, i: i32) -> bool {
        let c = self.s.cities_of(self.who)[i as usize];
        self.s.cities[c].alive
    }
    fn city_pos(&mut self, i: i32) -> Pos {
        let c = self.s.cities_of(self.who)[i as usize];
        self.s.cities[c].pos
    }
    fn peer(&mut self, k: usize) -> crate::army_process_view::Peer {
        let b = &self.s.armies[self.who as usize].list[k];
        crate::army_process_view::Peer {
            valid: b.valid,
            navy: b.navy,
            reg: b.reg.map_or(-1, i32::from),
            num_captains: b.num_captains,
            num_standard: b.num_standard,
            num_decoys: b.num_decoys,
        }
    }
    fn normalize(&mut self) {
        self.s.army_normalize(self.who, self.slot);
    }
    fn count_type(&mut self, ty: i32) -> i32 {
        use crate::army_process_view::{COUNT_GENERAL, COUNT_SPY};
        let line = match ty {
            COUNT_GENERAL => GENERAL,
            COUNT_SPY => SPY,
            _ => SCOUT,
        };
        self.s.army_count(self.who, self.slot, |s, u| {
            !s.units[u].decoy && s.unit_line_is(u, line)
        })
    }
    fn use_generals(&mut self) {
        self.s.use_generals(self.who, self.slot);
    }
    fn use_spies(&mut self) {
        self.s.use_spies(self.who, self.slot);
    }
    fn use_scouts(&mut self) {
        self.s.use_scouts(self.who, self.slot);
    }
    fn send_here(&mut self, p: Pos, kind: i32) {
        let kind = if kind == crate::orders::index::ATTACK_TO as i32 {
            MoveKind::AttackTo
        } else {
            MoveKind::MoveTo
        };
        self.s.send_here(self.who, self.slot, p, kind);
    }
    fn close(&mut self) {
        self.s.close_army(self.who, self.slot);
    }
    fn merge_into(&mut self, k: usize) {
        let units: Vec<usize> = self.a().units.iter().rev().copied().collect();
        for u in units {
            self.s.army_add_unit(self.who, k, u);
        }
    }
    fn is_moving(&mut self) -> bool {
        self.s.army_is_moving(self.who, self.slot)
    }
    fn is_engaged(&mut self) -> bool {
        // A closed slot (`valid` cleared by a callee) has no units to engage.
        self.a().valid && self.s.army_is_engaged(self.who, self.slot)
    }
    fn is_enemy(&mut self, owner: i32) -> bool {
        self.s.is_enemy(self.who, owner as Player)
    }
    fn find_muster_spot(&mut self) {
        let t = self.a().target.expect("a target");
        self.s.find_muster_spot(self.who, self.slot, t, false);
    }
    fn set_stance(&mut self, code: i32) {
        if !self.a().valid {
            return;
        }
        let st = match code {
            0 => Stance::Aggressive,
            1 => Stance::Defensive,
            _ => Stance::Raid,
        };
        self.s.army_set_stance(self.who, self.slot, st);
    }
    fn do_mustering(&mut self) {
        self.s.do_mustering(self.who, self.slot);
    }
    fn do_defending(&mut self) {
        self.s.do_defending(self.who, self.slot);
    }
    fn do_marching(&mut self) {
        self.s.do_marching(self.who, self.slot);
    }
    fn do_forming(&mut self, _ran: bool) {
        self.s.do_forming(self.who, self.slot);
    }
    fn engagement(&mut self) {
        self.s.engagement(self.who, self.slot);
    }
    fn do_transporting(&mut self) {
        self.s.do_transporting(self.who, self.slot);
    }
}

/// [`crate::army_is_moving_view::ArmyIsMovingView`] over one army of the sim.
struct SimIsMoving<'a> {
    s: &'a Sim,
    who: Player,
    slot: usize,
}

impl crate::army_is_moving_view::ArmyIsMovingView for SimIsMoving<'_> {
    fn count(&self) -> i32 {
        self.s.armies[self.who as usize].list[self.slot].num_units
    }
    fn unit(&mut self, i: i32) -> i32 {
        let a = &self.s.armies[self.who as usize].list[self.slot];
        a.units.get(i as usize).map_or(-1, |&u| u as i32)
    }
    fn is_active(&mut self, u: i32) -> bool {
        self.s.units[u as usize].alive()
    }
    fn is_on_map(&mut self, u: i32) -> bool {
        self.s.units[u as usize].on_map
    }
    fn head_is_move(&mut self, u: i32) -> Option<bool> {
        self.s.units[u as usize]
            .orders
            .front()
            .map(crate::orders::Order::is_move)
    }
    fn outside_region(&mut self, u: i32) -> bool {
        let reg = self.s.armies[self.who as usize].list[self.slot].reg;
        self.s.world.region_of(self.s.units[u as usize].pos.cell()) != reg
    }
    fn centre_of_gravity(&mut self) -> Pos {
        cell_centre(self.s.army_centre_of_gravity(self.who, self.slot))
    }
    fn dist_to(&mut self, u: i32, c: Pos) -> i32 {
        dist(self.s.units[u as usize].pos, c)
    }
}

#[cfg(test)]
mod tests;

// The files of `ai/army/` (item 4012, `docs/STRUCTURE.md` §3).
pub mod army_do_defending_view;
pub mod army_is_moving_view;
pub mod army_normalize_view;
pub mod army_process_view;
pub mod muster;
pub mod target_gate;
pub mod target_score;
