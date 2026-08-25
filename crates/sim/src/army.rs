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
//! scoring over cities (§12), the muster-spot chase (§13), transporting
//! (`docs/TRANSPORT.md` §8.2) and the `Armies` queries (§15). What is
//! **not** is every `Group::action_*` the original issues — the sim has
//! no group orders yet — so `do_forming`, `march_to_target`,
//! `engagement`'s attack order, `send_here`, `charge` and `stop` write the
//! record and move nothing; each says so where it stands. The rest of the
//! stand-ins are in [`seams`].

use crate::ai_load::{role, uflags2};
use crate::attrition::Domain;
use crate::build::{self, Ident};
use crate::combat::Obj;
use crate::movement::{Angle, find_angle};
use crate::world::{Cell, Owner, Pos, Terrain, UNITS_PER_CELL, vector_dist};
use crate::{Player, Sim};

/// Where this module knowingly stands in for the original, in one list.
///
/// | seam | stands in for | what it costs |
/// | --- | --- | --- |
/// | no group orders | every `Group::action_*` in §8, §9, §11, §14 | armies decide but never move units; `status & 4` is never set; `add_to_army`'s walk to the army is not issued |
/// | `find_muster_spot`'s ring | §13's `circle_*` walk over the land classes | the muster cell is the target's cell, `y ± 1` — the original's own fallback |
/// | `find_target`'s forts | §12's second scan | never a fort target |
/// | `pop_issues`, `wonderwin_timer`, `popwin_timer`, `score`, `num_wonders`, `GLOBAL_GOVERNMENT_BONUS`, `weak[]`/`strong[]`, the tribute period | leader and city fields the sim does not keep | the multipliers they gate are ×1; a leader at peace is never a target |
/// | `type_avail(SUPPLYWAGON)`, `is(CATAPHRACT)` | two of §12's strength-gate terms | a wagon-less army is not weak for it; cataphracts count 0 |
/// | `is(SUPPLYWAGON)` | the lineage test behind `num_standard` and the caps | `unit_flags2 & 0x40` without `0x20` — the supply-or-hero bit less the generals, which also admits the government patriots |
/// | `Game::war_allowed` under rush rules | §7's pre-war gate | always allowed |
/// | `leader_flags & 8`, `leader_flags2 & 8` | the two stop bits (§18) | never set |
/// | `come_out`'s draw | §4's coin on a unit leaving a building | no unit reaches `add_to_army` that way yet |
/// | `Region::flags & 8` | `go_here`'s resource-region bit | bit 1 never set; `do_transporting` does not read it |
/// | `use_generals` / `use_spies` / `use_scouts` | the 128-frame spellcaster turn (§5) | no spells |
pub mod seams {}

pub const SLOTS: usize = 16;

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
}

impl Army {
    const fn empty(slot: usize, who: Player) -> Army {
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

/// The centre of a cell in coordinate units — `cell × 0x300 + 0x180`.
pub const fn cell_centre(c: Cell) -> Pos {
    Pos::new(
        c.x * UNITS_PER_CELL + UNITS_PER_CELL / 2,
        c.y * UNITS_PER_CELL + UNITS_PER_CELL / 2,
    )
}

/// `|dx|, |dy|` through `vector_dist`.
const fn dist(a: Pos, b: Pos) -> i32 {
    vector_dist((a.x - b.x).abs(), (a.y - b.y).abs())
}

/// `TOWN → 2, METROPOLIS / FORBIDDENCITY → 3, else 1` — the city size
/// factor §10 and §12 use, from [`Sim::city_level_of`] (1 city, 2 town, 3
/// metropolis).
const fn size_factor(level: i32) -> i32 {
    match level {
        2 => 2,
        3 => 3,
        _ => 1,
    }
}

/// `TOWN → 2, METROPOLIS / FORBIDDENCITY → 1, else 3` — a raider's
/// preference for small cities (§12).
const fn raid_factor(level: i32) -> i32 {
    match level {
        2 => 2,
        3 => 1,
        _ => 3,
    }
}

/// The target's cell, one row down — or up on the map's last row — the
/// fallback muster cell of §12 and §13.
fn below(c: Cell, height: i32) -> Cell {
    Cell {
        x: c.x,
        y: if c.y < height - 1 { c.y + 1 } else { c.y - 1 },
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

    /// `Army::close`: the slot freed, the units left where they are. The
    /// original halts every group here; the sim's units carry no army
    /// orders to halt (seam).
    pub fn close_army(&mut self, who: Player, slot: usize) {
        let a = &mut self.armies[who as usize].list[slot];
        if !a.valid {
            return;
        }
        a.valid = false;
        a.status = 0;
        a.human_frame = 0;
        a.units.clear();
    }

    // ---- membership and the counts (§3) ----

    /// `UnitData::is_captain`: `o_up < 0`, a unit inside nothing.
    fn is_captain(&self, u: usize) -> bool {
        self.units[u].on_map && self.units[u].inside.is_none()
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

    /// `Army::normalize` (§3.3): drop the dead, recount, and the standard
    /// line — captains less casters and supply wagons. The sim has no
    /// decoys and no anti-air yet.
    pub fn army_normalize(&mut self, who: Player, slot: usize) {
        let w = who as usize;
        let units: Vec<usize> = self.armies[w].list[slot]
            .units
            .iter()
            .copied()
            .filter(|&u| u < self.units.len() && self.units[u].alive())
            .collect();
        let mut captains = 0;
        let mut casters = 0;
        let mut supply = 0;
        for &u in &units {
            if !self.is_captain(u) {
                continue;
            }
            captains += 1;
            if self.units[u]
                .ty
                .is_some_and(|t| self.unit_types[t].cols.flag2(uflags2::CASTER))
            {
                casters += 1;
            }
            if self.is_supply_wagon(u) {
                supply += 1;
            }
        }
        let a = &mut self.armies[w].list[slot];
        a.units = units;
        a.role = 0;
        a.num_units = a.units.len() as i32;
        a.num_captains = captains;
        a.num_decoys = 0;
        a.num_standard = captains - casters - supply;
    }

    /// `Army::add_unit` (§3.2): into the one group, if not already there.
    pub fn army_add_unit(&mut self, who: Player, slot: usize, u: usize) {
        if !self.units[u].alive() {
            return;
        }
        let a = &mut self.armies[who as usize].list[slot];
        if a.units.contains(&u) {
            return;
        }
        a.units.push(u);
        a.num_units += 1;
        a.num_captains += 1;
    }

    /// `Object::get_army`: the army holding this unit, if any.
    pub fn army_of(&self, u: usize) -> Option<usize> {
        let who = self.units[u].owner as usize;
        self.armies
            .get(who)?
            .valid()
            .find(|(_, a)| a.units.contains(&u))
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

    /// `Army::count(COUNT_SIEGE)`.
    fn army_count_siege(&self, who: Player, slot: usize) -> i32 {
        self.army_count(who, slot, |s, u| {
            s.units[u].ty.is_some_and(|t| s.unit_types[t].combat.siege)
        })
    }

    /// `count(NON_DECOY_TYPE, SUPPLYWAGON)`.
    fn army_count_supply(&self, who: Player, slot: usize) -> i32 {
        self.army_count(who, slot, Sim::is_supply_wagon)
    }

    fn army_count_generals(&self, who: Player, slot: usize) -> i32 {
        self.army_count(who, slot, Sim::is_general)
    }

    fn army_count_hoplites(&self, who: Player, slot: usize) -> i32 {
        self.army_count(who, slot, |s, u| {
            s.units[u]
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
        self.army_add_unit(who, slot, u);
        Some(slot)
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

    /// The idle-think hook (§4): an AI-owned unit with nothing to do joins
    /// an army if it is a supply wagon or hero (`Unit::think`'s tail) or
    /// an attacker that is not a scout or caravan (`think_attack`).
    pub(crate) fn think_join_army(&mut self, u: usize) {
        let unit = &self.units[u];
        let w = unit.owner as usize;
        if self.nation.get(w).is_none_or(|n| n.human) || self.defeated[w] {
            return;
        }
        let Some(t) = unit.ty else { return };
        let cols = self.unit_types[t].cols;
        let supply_or_hero = cols.flag2(uflags2::SUPPLY_OR_HERO) || cols.flag2(uflags2::GENERAL);
        let attacker = self.attack_of(Obj::Unit(u)) != 0
            && !cols.flag2(uflags2::SCOUT)
            && !cols.flag2(uflags2::CARAVAN);
        if supply_or_hero || attacker {
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

    /// `Army::process` (§5–§6).
    pub fn army_process(&mut self, who: Player, slot: usize, bypass: bool) {
        let w = who as usize;
        {
            let a = &mut self.armies[w].list[slot];
            if a.human_frame != 0 {
                a.human_frame -= 1;
            }
        }
        let frame = self.frame;
        if !bypass {
            let s = self.armies[w].list[slot].phase();
            if (frame - 30 + s) % 128 == 0 {
                if self.defeated[w] {
                    return;
                }
                self.army_normalize(who, slot);
                if self.armies[w].list[slot].num_captains == 0 {
                    return;
                }
                // The spellcasters' turn — a seam.
            }
            if (frame + s) % 256 != 0 {
                return;
            }
        }
        if self.defeated[w] {
            return;
        }
        self.army_tick(who, slot);
    }

    // ---- the tick (§6) ----

    fn army_tick(&mut self, who: Player, slot: usize) {
        let w = who as usize;
        self.army_normalize(who, slot);
        // 1. Disband: the survivors are sent home (`send_here`, an order
        //    the sim does not issue); the slot is freed either way.
        {
            let a = &self.armies[w].list[slot];
            if a.num_standard < 1 && a.status & status::MUSTERING == 0 {
                self.close_army(who, slot);
                return;
            }
        }
        // 2. A human's ping: the AI stands aside while the countdown runs.
        if self.armies[w].list[slot].human_frame != 0 {
            return;
        }
        // 3. Merge into a stronger army of the same region.
        {
            let a = &self.armies[w].list[slot];
            if a.num_standard < (a.num_captains - a.num_decoys) / 2 {
                let reg = a.reg;
                let into = (0..SLOTS).find(|&k| {
                    let b = &self.armies[w].list[k];
                    b.valid
                        && !b.navy
                        && b.reg == reg
                        && b.num_standard > 4
                        && (b.num_captains - b.num_decoys) / 2 <= b.num_standard
                });
                if let Some(k) = into {
                    let units: Vec<usize> = self.armies[w].list[slot]
                        .units
                        .iter()
                        .rev()
                        .copied()
                        .collect();
                    for u in units {
                        self.army_add_unit(who, k, u);
                    }
                    self.close_army(who, slot);
                    return;
                }
            }
        }
        // 4. The muster point walks halfway to the target.
        if !self.army_is_moving(who, slot) && !self.army_is_engaged(who, slot) {
            let (target, navy, st0) = {
                let a = &self.armies[w].list[slot];
                (a.target, a.navy, a.status)
            };
            let mut st = st0 & !(status::FORMING | status::MARCHING);
            if let Some(t) = target {
                let tw = self.owner_of(t);
                if !self.is_enemy(who, tw) || navy {
                    self.find_muster_spot(who, slot, t);
                } else {
                    let tc = self.pos_of(t).cell();
                    let a = &mut self.armies[w].list[slot];
                    a.muster = Cell {
                        x: (tc.x + a.muster.x) / 2,
                        y: (tc.y + a.muster.y) / 2,
                    };
                }
                st |= status::FORMING | status::MARCHING;
            }
            self.armies[w].list[slot].status = st;
        }
        // 5. Nothing but the forming and no-target bits → marching.
        {
            let a = &mut self.armies[w].list[slot];
            if a.status & (status::FORMING | status::NO_TARGET) == a.status {
                a.status = status::MARCHING;
            }
        }
        // 6. Dispatch. `do_forming` and `engagement` issue group orders
        //    only (seam), so they are not called.
        if self.armies[w].list[slot].status & status::MUSTERING != 0 {
            self.do_mustering(who, slot);
        }
        if self.armies[w].list[slot].status & status::DEFENDING != 0 {
            self.do_defending(who, slot);
        }
        if self.armies[w].list[slot].status & status::MARCHING != 0 {
            self.do_marching(who, slot);
        }
        if self.armies[w].list[slot].valid
            && self.armies[w].list[slot].status & status::TRANSPORTING != 0
        {
            self.do_transporting(who, slot);
        }
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

    /// `Army::do_mustering` — `docs/TRANSPORT.md` §8.1's table.
    fn do_mustering(&mut self, who: Player, slot: usize) {
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
                if self.find_muster_spot(who, slot, Obj::Building(b)) {
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
        // `rush_rules && !Game::war_allowed()` — always allowed (seam).
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
        let age = self.tech[w].ages;
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
    /// `march_to_target`.
    fn do_marching(&mut self, who: Player, slot: usize) {
        let w = who as usize;
        if self.army_count_attack(who, slot) < 1 {
            self.close_army(who, slot);
            return;
        }
        let mut retarget = false;
        if let Some(t) = self.armies[w].list[slot].target {
            let tw = self.owner_of(t);
            if self.active(t) {
                match t {
                    Obj::Unit(_) => {
                        if !self.is_enemy(who, tw) {
                            retarget = true;
                        }
                    }
                    Obj::Building(b) => {
                        if let Some(c) = self.buildings[b].city {
                            let cd = &self.cities[c];
                            let navy = self.armies[w].list[slot].navy;
                            if !cd.alive || (navy && !self.is_enemy(who, tw) && !cd.no_heal) {
                                retarget = true;
                            } else if !self.is_enemy(who, tw) && !cd.no_heal && !cd.unassimilated {
                                let bd = &self.buildings[b];
                                // `weak`/`strong`/tribute are seams: the
                                // ally test alone decides.
                                if bd.hits - bd.health < bd.hits * 3 / 4 && self.is_ally(who, tw) {
                                    retarget = true;
                                }
                            }
                        }
                    }
                }
            }
        }
        if retarget {
            self.armies[w].list[slot].target = None;
            if self.armies[w].list[slot].status & status::DEFENDING != 0 {
                self.armies[w].list[slot].status = status::DEFENDING | status::FORMING;
                return;
            }
            self.find_target(who, slot);
            if !self.armies[w].list[slot].valid {
                return;
            }
            if self.armies[w].list[slot].target.is_some() {
                self.armies[w].list[slot].status |= status::FORMING;
                return;
            }
            self.armies[w].list[slot].status = status::NO_TARGET;
            return;
        }
        if self.armies[w].list[slot].status & status::MARCHING != 0 {
            self.march_to_target(who, slot);
        }
    }

    /// `Army::march_to_target`: not engaged → forming and marching; engaged
    /// → the attack orders (seam), so `MARCHED` is never set here.
    fn march_to_target(&mut self, who: Player, slot: usize) {
        let w = who as usize;
        if self.armies[w].list[slot].target.is_none() {
            return;
        }
        if !self.army_is_engaged(who, slot) {
            let a = &mut self.armies[w].list[slot];
            a.status = (a.status & !status::MARCHED) | status::FORMING | status::MARCHING;
        }
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
            if self.find_muster_spot(who, slot, Obj::Building(b)) {
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
    fn restrict(&self, p: Pos) -> Pos {
        Pos::new(
            p.x.clamp(0, self.world.width() * UNITS_PER_CELL - 1),
            p.y.clamp(0, self.world.height() * UNITS_PER_CELL - 1),
        )
    }

    /// `Army::find_besieged_city` (§10): the attacked city of mine or an
    /// ally's, in my region, that scores best.
    fn find_besieged_city(&mut self, who: Player, slot: usize) -> bool {
        let w = who as usize;
        let (navy, reg, pos) = {
            let a = &self.armies[w].list[slot];
            (a.navy, a.reg, a.pos)
        };
        let mut best: Option<(i32, usize)> = None;
        for i in 0..self.players.len() {
            let ip = i as Player;
            if self.defeated[i] || !self.is_ally(who, ip) {
                continue;
            }
            for c in self.cities_of(ip) {
                let cd = &self.cities[c];
                let here = if navy {
                    match (reg, cd.reg) {
                        (Some(r), Some(cr)) => self.world.is_coast(r, cr),
                        _ => false,
                    }
                } else {
                    cd.reg == reg
                };
                if !here || !cd.no_heal {
                    continue;
                }
                let mut v = 1000 * size_factor(self.city_level_of(c));
                if i == w {
                    v *= 10;
                }
                for (k, b) in self.armies[i].valid() {
                    if (k != slot || i != w) && b.city == Some(c) && b.num_standard > 4 {
                        v /= 4;
                    }
                }
                v /= dist(pos, cd.pos) / UNITS_PER_CELL + 1;
                if best.is_none_or(|(bv, _)| bv <= v) {
                    best = Some((v, c));
                }
            }
        }
        let Some((_, c)) = best else { return false };
        let target = Obj::Building(self.cities[c].building);
        let cpos = self.cities[c].pos;
        if navy {
            self.find_muster_spot(who, slot, target);
            let a = &mut self.armies[w].list[slot];
            a.pos = cell_centre(a.muster);
            return true;
        }
        let p = self.restrict(Pos::new(cpos.x, cpos.y + UNITS_PER_CELL));
        let a = &mut self.armies[w].list[slot];
        let mc = cell_centre(a.muster);
        a.muster_angle = find_angle(cpos.x - mc.x, cpos.y - mc.y);
        a.muster = a.pos.cell();
        a.pos = p;
        a.target = Some(target);
        true
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
            let attacking = unit
                .orders
                .front()
                .is_some_and(|o| matches!(o.body, crate::orders::Body::Attack(_)));
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

    /// `Army::is_moving`: more than a sixth of the units either standing
    /// outside the region or walking near the centre of gravity.
    pub fn army_is_moving(&self, who: Player, slot: usize) -> bool {
        let w = who as usize;
        let a = &self.armies[w].list[slot];
        let mut cog: Option<Pos> = None;
        let mut count = 0;
        for &u in &a.units {
            let unit = &self.units[u];
            if !unit.alive() || !unit.on_map {
                continue;
            }
            let Some(o) = unit.orders.front() else {
                continue;
            };
            if !o.is_move() {
                if self.world.region_of(unit.pos.cell()) != a.reg {
                    count += 1;
                }
                continue;
            }
            let c = *cog.get_or_insert_with(|| cell_centre(self.army_centre_of_gravity(who, slot)));
            if dist(unit.pos, c) <= 0xf00 {
                count += 1;
            }
        }
        count > a.num_units / 6
    }

    // ---- the target (§12) ----

    /// `Army::find_target` over cities. The forts pass and the fields the
    /// sim lacks are seams; see the module doc.
    pub fn find_target(&mut self, who: Player, slot: usize) {
        let w = who as usize;
        let diff = self.ai_difficulty();
        let age = self.tech[w].ages;
        let team_style = self.lobby.team_style;
        let frame = self.frame;
        let (navy, my_reg, num_captains, num_standard) = {
            let a = &self.armies[w].list[slot];
            (a.navy, a.reg, a.num_captains, a.num_standard)
        };
        let siege = self.army_count_siege(who, slot);
        let supply = self.army_count_supply(who, slot);
        let weak_army = !navy && self.army_count_hoplites(who, slot) + 2 * siege < 4;
        let capital = self
            .cities_of(who)
            .into_iter()
            .find(|&c| self.cities[c].capital)
            .map(|c| self.cities[c].pos.cell());
        let level = self.transport_level(who) as i32;
        let pers = self.ai[w].pers;
        let my_combat = self.ai[w].census.combat;

        // The two averages of `combat`.
        let (mut ours, mut theirs) = ((0, 0), (0, 0));
        for i in 0..self.players.len() {
            if self.defeated[i] {
                continue;
            }
            let ip = i as Player;
            if self.is_ally(who, ip) {
                ours = (ours.0 + self.ai[i].census.combat, ours.1 + 1);
            } else if self.is_enemy(who, ip) {
                theirs = (theirs.0 + self.ai[i].census.combat, theirs.1 + 1);
            }
        }
        let ours = if ours.1 != 0 { ours.0 / ours.1 } else { 0 };
        let theirs = if theirs.1 != 0 {
            theirs.0 / theirs.1
        } else {
            0
        };

        let mut best: Option<(i32, usize, Player)> = None;
        for pass in 0..2 {
            for i in 0..self.players.len() {
                if self.defeated[i] {
                    continue;
                }
                let ip = i as Player;
                let enemy = self.is_enemy(who, ip);
                let allied = self.is_ally(who, ip);
                if i != w && !allied {
                    if !enemy {
                        // At peace: `weak`/`strong`/tribute are seams; a
                        // leader at peace is never a target.
                        continue;
                    }
                    if diff < 2 {
                        if !(diff != 0 || self.nation[i].human) {
                            continue;
                        }
                        if self.ai[i].frame_attacked + 0x1c20 > frame {
                            continue;
                        }
                        let go = match self.find_aggressive_army(who) {
                            Some(k) => k == slot,
                            None => self.rng.get(0, 0xffff) & 1 == 0,
                        };
                        if !go {
                            continue;
                        }
                    } else {
                        if diff == 1
                            && num_captains >= my_combat / 2
                            && self.find_aggressive_army(who).is_some_and(|k| k != slot)
                        {
                            continue;
                        }
                        if !(diff < 2
                            || (age < 3 && pers.early_army != 0 && diff != 2)
                            || num_standard >= 7 - 2 * pers.raid)
                        {
                            continue;
                        }
                        if diff == 2 {
                            let fa = self.ai[i].frame_attacked;
                            if !(fa + 0x708 <= frame
                                && (team_style == 3
                                    || fa + 0xe10 <= frame
                                    || self.ai[i].attacked_by == i32::from(who)
                                    || self.ai[w].attacked_by == i32::from(ip)))
                            {
                                continue;
                            }
                        }
                    }
                }
                for c in self.cities_of(ip) {
                    let cd = self.cities[c].clone();
                    if navy {
                        let coast = match (my_reg, cd.reg) {
                            (Some(r), Some(cr)) => self.world.is_coast(r, cr),
                            _ => false,
                        };
                        if !coast {
                            continue;
                        }
                    }
                    if !(diff > 1 || i == w || allied || cd.founder == who) {
                        continue;
                    }
                    let mut v = self.rng.get(0, 0xffff) % 200 + 900;
                    if let Some(cap) = capital
                        && enemy
                    {
                        if weak_army && !(cd.owner == who && cd.unassimilated) {
                            continue;
                        }
                        let cc = cd.pos.cell();
                        let d = vector_dist((cc.x - cap.x).abs(), (cc.y - cap.y).abs());
                        v += d * -50 / self.world.width();
                    }
                    let attacked = cd.no_heal;
                    let lvl = self.city_level_of(c);
                    if i == w || allied {
                        if allied && i != w && !attacked {
                            if num_captains < 20 || siege < 2 {
                                if num_captains < 10 || siege == 0 {
                                    v *= size_factor(lvl);
                                }
                            } else {
                                v = v * (4 - lvl) / 4;
                            }
                        } else if i != w && team_style == 2 {
                            v *= 5;
                        }
                    } else {
                        if team_style == 2 || team_style == 3 {
                            v *= 5;
                        }
                        if navy {
                            v *= 10;
                        }
                        let k = if pers.raid == -1 {
                            Some(size_factor(lvl))
                        } else if pers.raid == 1 {
                            // The `score × 4/3` clause is a seam: the
                            // level counts only with the army.
                            if num_captains > 19 && siege > 2 {
                                Some(lvl)
                            } else {
                                Some(raid_factor(lvl))
                            }
                        } else if num_captains > 19 && siege > 1 {
                            Some(lvl)
                        } else if num_captains > 9 && siege != 0 {
                            None
                        } else {
                            Some(raid_factor(lvl))
                        };
                        if let Some(k) = k {
                            v *= k;
                        }
                    }
                    if cd.reg != my_reg && !navy {
                        v /= 2;
                        if level == 0 {
                            continue;
                        }
                        if ours < theirs || ours < 5 {
                            v /= 2;
                            if ours < theirs - 2 || ours == 0 {
                                v /= 5;
                            }
                        }
                    }
                    if (pass == 0 || diff > 2) && (i == w || allied) && !attacked {
                        v /= 3;
                    }
                    if (i == w || allied) && attacked {
                        if i == w {
                            v *= 10;
                        }
                        if cd.capital {
                            v *= 10;
                        }
                    } else if enemy && cd.capital && pers.raid < 0 {
                        v = v * 3 / 2;
                    }
                    let bordering = self.ai[w]
                        .city_ai
                        .get(c)
                        .is_some_and(|ca| ca.bordering & (1 << who) != 0);
                    if bordering {
                        v *= if pers.raid < 1 { 4 } else { 2 };
                        if enemy && diff > 2 {
                            if self.ai[i].frame_attacked + 9000 < frame {
                                v *= 3;
                            }
                            if cd.attack_stamp + 9000 < frame {
                                v *= 3;
                            }
                        }
                    }
                    if i != w && allied {
                        v = v * 3 / 4;
                        if !attacked {
                            v /= 2;
                        }
                    }
                    let (hits, health) = {
                        let bd = &self.buildings[cd.building];
                        (bd.hits, bd.health)
                    };
                    let damaged = health < hits;
                    if i == w && damaged {
                        v *= 2;
                    } else if damaged {
                        v = if bordering { v * 3 / 2 } else { v * 5 / 4 };
                    }
                    if cd.was_capital & (1 << who) == 0 {
                        if enemy && !navy {
                            if age > 2 && diff > 2 && !cd.unassimilated {
                                if siege == 0 && age > 3 {
                                    v /= 10;
                                }
                                if supply < 2 {
                                    if age > 4 {
                                        v /= 10;
                                    }
                                    v /= 10;
                                    if supply < 1 {
                                        v /= 10;
                                    }
                                }
                            }
                            if self.nation[i].human {
                                if diff == 4 {
                                    v = v * 5 / 4;
                                } else if diff == 5 {
                                    v = v * 4 / 3;
                                }
                            }
                        }
                    } else {
                        v = 1_000_000;
                    }
                    let target = Obj::Building(cd.building);
                    for (k, b) in self.armies[w].valid() {
                        if b.status & status::MUSTERING != 0 || b.target != Some(target) {
                            continue;
                        }
                        if k == slot {
                            v *= if enemy || attacked { 8 } else { 2 };
                            if i == w && cd.unassimilated {
                                v *= 8;
                            }
                            if i == w && (hits - health) * 2 > hits {
                                v *= 8;
                            }
                        } else if !attacked || (i != w && !allied && diff < 3 && !navy) {
                            // Two arms of the original, the same quarter:
                            // another army has a quiet city, or an enemy's
                            // attacked one at low difficulty on land.
                            v /= 4;
                        } else {
                            v *= 4;
                        }
                    }
                    if v < 0 {
                        v = 1_000_000;
                    }
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
        if tw != who {
            self.armies[w].list[slot].hurry = 0;
        }
        if diff == 0 && self.nation[tw as usize].human {
            // The two-unit probe is a group order (seam); nothing changes.
            return;
        }
        let target = Obj::Building(self.cities[c].building);
        {
            let a = &mut self.armies[w].list[slot];
            if a.target != Some(target) {
                a.rally_dist = 0x1200;
            }
            a.target = Some(target);
        }
        if self.is_enemy(who, tw) {
            self.ai[tw as usize].frame_attacked = frame;
            self.ai[tw as usize].attacked_by = i32::from(who);
            // `send_navy` — `docs/TRANSPORT.md` §8.3, a navy hint (seam).
        }
        let cpos = self.cities[c].pos;
        let p = self.restrict(Pos::new(cpos.x, cpos.y + UNITS_PER_CELL));
        self.armies[w].list[slot].pos = p;
        if self.find_muster_spot(who, slot, target) {
            let a = &mut self.armies[w].list[slot];
            a.muster_angle = Angle(a.muster_angle.0.wrapping_add(i32::MIN));
        } else {
            self.close_army(who, slot);
        }
    }

    // ---- the muster spot (§13) ----

    /// `Army::find_muster_spot(o, who, ·)`: the chase over the target
    /// city's damaged buildings, then the original's own fallback — the
    /// target's cell, one row down (up on the last row). The ring search
    /// is a seam, so this always finds a spot.
    pub fn find_muster_spot(&mut self, who: Player, slot: usize, target: Obj) -> bool {
        let w = who as usize;
        self.armies[w].list[slot].hurry = 0;
        let tw = self.owner_of(target);
        let (apos, navy) = {
            let a = &self.armies[w].list[slot];
            (a.pos, a.navy)
        };
        let ac = apos.cell();
        let height = self.world.height();
        if self.is_ally(who, tw) && !navy {
            let mut muster: Option<Cell> = None;
            if let Obj::Building(b) = target {
                let chain: Vec<usize> = match self.buildings[b].city {
                    Some(c) => {
                        let ch = self.city_chain(c);
                        let at = ch.iter().position(|&x| x == b).unwrap_or(0);
                        ch[at..].to_vec()
                    }
                    None => vec![b],
                };
                for m in chain {
                    let bd = &self.buildings[m];
                    if !bd.alive || bd.health >= bd.hits {
                        continue;
                    }
                    let bpos = bd.pos;
                    let enemy = self.nearest_enemy_attacker(who, bpos, 0x900).or_else(|| {
                        self.units
                            .iter()
                            .enumerate()
                            .find(|(_, u)| {
                                u.alive()
                                    && u.on_map
                                    && self.is_enemy(who, u.owner)
                                    && u.combat.target == Some(Obj::Building(m))
                                    && dist(u.pos, bpos) <= 0x1200
                            })
                            .map(|(i, _)| i)
                    });
                    if let Some(e) = enemy {
                        let mc = self.units[e].pos.cell();
                        let a = &mut self.armies[w].list[slot];
                        a.muster = mc;
                        a.muster_angle = find_angle(mc.x - ac.x, mc.y - ac.y);
                        a.hurry = 1;
                        return true;
                    }
                    if muster.is_none() {
                        muster = Some(bpos.cell());
                    }
                }
            }
            let m = below(muster.unwrap_or_else(|| self.pos_of(target).cell()), height);
            let a = &mut self.armies[w].list[slot];
            a.muster = m;
            a.muster_angle = find_angle(m.x - ac.x, m.y - ac.y);
            return true;
        }
        let m = below(self.pos_of(target).cell(), height);
        let a = &mut self.armies[w].list[slot];
        a.muster = m;
        a.muster_angle = find_angle(m.x - ac.x, m.y - ac.y);
        true
    }

    /// `find_unit(SEARCH_ENEMY, FILTER_COMBAT)`: the nearest enemy unit of
    /// `who` with an attack within `range` of `p`.
    fn nearest_enemy_attacker(&self, who: Player, p: Pos, range: i32) -> Option<usize> {
        let mut best: Option<(i32, usize)> = None;
        for (i, u) in self.units.iter().enumerate() {
            if !u.alive() || !u.on_map || !self.is_enemy(who, u.owner) {
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

    /// `Region::go_here(r, who)` (`docs/TRANSPORT.md` §9.4), bits 2 and 4;
    /// bit 1 is a seam (the resource-region flag).
    pub fn go_here(&self, r: u16, who: Player) -> i32 {
        let w = who as usize;
        let mine = self.ai[w]
            .census
            .reg_cities
            .get(r as usize)
            .copied()
            .unwrap_or(0);
        let weak = self.strategy_of(who, Some(r)) & 4 != 0;
        let mut g = 0;
        for i in 0..self.players.len() {
            if i == w || self.defeated[i] || !self.is_enemy(who, i as Player) {
                continue;
            }
            let theirs = self.ai[i]
                .census
                .reg_cities
                .get(r as usize)
                .copied()
                .unwrap_or(0);
            if theirs != 0 {
                g |= if mine == 0 || weak { 2 } else { 4 };
            }
        }
        g
    }

    /// `Army::do_transporting`: a one-shot retarget to the best land region
    /// the census wants; hands the army to marching.
    fn do_transporting(&mut self, who: Player, slot: usize) {
        let w = who as usize;
        let (pos, reg) = {
            let a = &self.armies[w].list[slot];
            (a.pos, a.reg)
        };
        let here = self.world.region_of(pos.cell());
        if here != reg {
            return;
        }
        let Some(my_reg) = reg else { return };
        let my_first = self.world.cells_in(my_reg).next();
        let mut best: Option<(i32, u16, Option<usize>)> = None;
        let regions: Vec<u16> = self
            .world
            .regions()
            .filter(|&(r, t)| t == Terrain::Land && r != my_reg)
            .map(|(r, _)| r)
            .collect();
        for r in regions {
            let size = self.world.region_size(r);
            if size <= 0 {
                continue;
            }
            let s = self.strategy_of(who, Some(r));
            let g = self.go_here(r, who);
            if !(s & 2 != 0 || g & 2 != 0 || g & 4 != 0) {
                continue;
            }
            let mut v = size
                * if g & 4 != 0 {
                    3
                } else if s & 2 != 0 {
                    2
                } else {
                    1
                };
            let their_first = self.world.cells_in(r).next();
            let d = match (my_first, their_first) {
                (Some(a), Some(b)) => vector_dist((a.x - b.x).abs(), (a.y - b.y).abs()),
                _ => 0,
            };
            v /= d / 4 + 1;
            let mut city = None;
            for c in self.cities_of(who) {
                if self.cities[c].reg == Some(r) && self.cities[c].no_heal {
                    v *= self.city_level_of(c);
                    city = Some(c);
                }
            }
            if best.is_none_or(|(bv, _, _)| bv < v) {
                best = Some((v, r, city));
            }
        }
        let Some((v, r, city)) = best else { return };
        if v <= 0 {
            return;
        }
        self.armies[w].list[slot].reg = Some(r);
        match city {
            None => {
                self.find_target(who, slot);
                if !self.armies[w].list[slot].valid {
                    return;
                }
                if self.armies[w].list[slot].target.is_none() {
                    self.armies[w].list[slot].reg = here;
                    return;
                }
            }
            Some(c) => {
                let target = Obj::Building(self.cities[c].building);
                self.armies[w].list[slot].target = Some(target);
                self.find_muster_spot(who, slot, target);
            }
        }
        self.armies[w].list[slot].status = status::MARCHING;
    }

    // ---- the Armies queries (§15) ----

    /// The supply-wagon and hero caps of `find_army` / `find_local_army`.
    fn army_admits(
        &self,
        who: Player,
        slot: usize,
        unit: Option<usize>,
        supply_cap: i32,
        hero_cap: i32,
    ) -> bool {
        let Some(u) = unit else { return true };
        if self.is_supply_wagon(u) {
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
        let reg = self.world.tregion(p.tile());
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
            if !self.army_admits(who, s, unit, 2, 1) {
                continue;
            }
            best = Some((s, d));
        }
        best
    }

    /// `Armies::find_useful_army`: the ping's finder.
    pub fn find_useful_army(&self, who: Player, p: Pos) -> Option<usize> {
        let w = who as usize;
        let reg = self.world.tregion(p.tile());
        let level = self.transport_level(who) as i32 != 0;
        let mut best: Option<(usize, i32)> = None;
        for (s, a) in self.armies[w].valid() {
            if a.num_units == 0 || !((!a.navy && level) || a.reg == reg) {
                continue;
            }
            let mut d = dist(p, a.pos);
            if a.reg != reg {
                d *= 3;
            }
            let score = d / a.num_units;
            if best.is_none_or(|(_, bs)| score <= bs) {
                best = Some((s, score));
            }
        }
        best.map(|(s, _)| s)
    }

    /// `Armies::find_aggressive_army`: the first army of two or more units,
    /// not mustering, standing outside its owner's territory.
    pub fn find_aggressive_army(&self, who: Player) -> Option<usize> {
        self.armies[who as usize]
            .valid()
            .find(|(_, a)| {
                a.num_units >= 2
                    && a.status & status::MUSTERING == 0
                    && self.world.owner(a.pos.cell()) != Owner::Player(who)
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

    /// `Armies::leader_defeated(who)`: `Army::stop` on every army — the
    /// units' orders are cleared; the slots stay valid.
    pub fn armies_leader_defeated(&mut self, who: Player) {
        let w = who as usize;
        for s in 0..SLOTS {
            if !self.armies[w].list[s].valid {
                continue;
            }
            let units = self.armies[w].list[s].units.clone();
            for u in units {
                if u < self.units.len() {
                    self.units[u].orders.clear();
                    self.units[u].path.clear();
                }
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sim_with_city() -> (Sim, usize) {
        let mut sim = Sim::new(
            crate::tuning::Tuning::RON,
            crate::world::World::new(60, 60),
            2,
        );
        sim.nation[0].human = true;
        sim.nation[1].human = false;
        let pos = Pos::new(0x3000, 0x3000);
        let b = sim.add_building(1, pos, 1);
        let c = sim.cities.len();
        sim.cities.push(crate::city::City {
            alive: true,
            owner: 1,
            race: Some(1),
            founder: 1,
            building: b,
            members: Vec::new(),
            reg: None,
            pos,
            capital: true,
            founding_capital: true,
            was_founding_capital: false,
            unassimilated: false,
            no_heal: false,
            alarm: false,
            was_capital: 0,
            capture_stamp: 0,
            assimilation_timer: 0,
            attack_stamp: 0,
            capture_strength: 0,
            pop: 1,
            has_citizen: false,
            source: None,
        });
        (sim, c)
    }

    #[test]
    fn init_army_takes_the_first_free_slot_then_evicts_the_smallest() {
        let (mut sim, c) = sim_with_city();
        for s in 0..SLOTS {
            assert_eq!(sim.init_army(1, Some(c)), s);
            sim.armies[1].list[s].num_units = (s as i32 % 5) + 1;
        }
        // Slots 0, 5, 10 and 15 hold one unit; `<=` on the scan sends the
        // tie to the later slot.
        assert_eq!(sim.init_army(1, Some(c)), 15);
        sim.armies[1].list[15].num_units = 9;
        assert_eq!(sim.init_army(1, Some(c)), 10);
    }

    #[test]
    fn the_muster_cell_is_one_row_below_the_city_and_the_record_is_run20_s() {
        let (mut sim, c) = sim_with_city();
        sim.cities[c].pos = Pos::new(39264, 40032);
        let s = sim.init_army(1, Some(c));
        let a = &sim.armies[1].list[s];
        assert_eq!((a.pos.x, a.pos.y), (39264, 40800));
        assert_eq!((a.muster.x, a.muster.y), (51, 53));
        assert_eq!(a.status, status::MUSTERING);
        assert_eq!(a.target, None);
        assert_eq!(a.num_groups(), 0);
        assert!(a.valid);
    }

    #[test]
    fn the_cadence_is_slot_and_owner_phased() {
        let (mut sim, c) = sim_with_city();
        let s = sim.init_army(1, Some(c));
        let a = &sim.armies[1].list[s];
        // Leader 1's army 0 ticks at frame ≡ 252 (mod 256) — run21's 252.
        assert!(a.ticks_on(252));
        assert!(!a.ticks_on(253));
        assert!(a.ticks_on(14588));
        // Its army 1 at 250 — run21's `do_transporting` at 14586.
        let s1 = sim.init_army(1, Some(c));
        assert!(sim.armies[1].list[s1].ticks_on(14586));
    }

    #[test]
    fn close_frees_the_slot_and_keeps_the_rest_of_the_record() {
        let (mut sim, c) = sim_with_city();
        let s = sim.init_army(1, Some(c));
        sim.armies[1].list[s].rally_dist = 0x1200;
        sim.close_army(1, s);
        let a = &sim.armies[1].list[s];
        assert!(!a.valid);
        assert_eq!(a.status, 0);
        assert_eq!(
            a.rally_dist, 0x1200,
            "close clears valid, status, human_frame and the groups only"
        );
    }

    #[test]
    fn release_mustering_wants_five_and_the_city_count_before_the_thresholds() {
        let (mut sim, c) = sim_with_city();
        let s = sim.init_army(1, Some(c));
        sim.muster[1].cap = 200;
        sim.armies[1].list[s].num_standard = 1;
        assert!(!sim.release_mustering(1, s));
        sim.armies[1].list[s].num_standard = 4;
        assert!(!sim.release_mustering(1, s), "fewer than five");
        // Five standard: `pop_cap / 8 = 25 > 5`, one mustering army → `200 / 2 = 100 > 5`,
        // difficulty 0 wants more than six, `rush 0`, `early_army 0` → 26.
        sim.armies[1].list[s].num_standard = 5;
        assert!(!sim.release_mustering(1, s));
        sim.armies[1].list[s].num_standard = 7;
        assert!(
            sim.release_mustering(1, s),
            "difficulty 0 releases at seven"
        );
        sim.lobby.difficulty = 3;
        assert!(!sim.release_mustering(1, s));
        sim.armies[1].list[s].num_standard = 26;
        assert!(sim.release_mustering(1, s));
        // My own city under attack releases at two.
        sim.armies[1].list[s].num_standard = 2;
        sim.cities[c].no_heal = true;
        assert!(sim.release_mustering(1, s));
    }
}
