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
use crate::world::{Cell, MOVE_49, Owner, Pos, Terrain, UNITS_PER_CELL, cell, vector_dist};
use crate::{Player, Sim};

/// Where this module knowingly stands in for the original, in one list.
///
/// | seam | stands in for | what it costs |
/// | --- | --- | --- |
/// | `find_target`'s forts | §12's second scan | never a fort target |
/// | `pop_issues`, `wonderwin_timer`, `popwin_timer`, `score`, `num_wonders`, `GLOBAL_GOVERNMENT_BONUS`, `weak[]`/`strong[]`, the tribute period | leader and city fields the sim does not keep | the multipliers they gate are ×1; a leader at peace is never a target |
/// | `type_avail(SUPPLYWAGON)`, `is(CATAPHRACT)` | two of §12's strength-gate terms | a wagon-less army is not weak for it; cataphracts count 0 |
/// | `is(SUPPLYWAGON)` | the lineage test behind `num_standard` and the caps | `unit_flags2 & 0x40` without `0x20` — the supply-or-hero bit less the generals, which also admits the government patriots |
/// | `Game::war_allowed` under rush rules | §7's pre-war gate | always allowed |
/// | `leader_flags & 8`, `leader_flags2 & 8` | the two stop bits (§18) | never set |
/// | `Region::flags & 8` | `go_here`'s resource-region bit | bit 1 never set; `do_transporting` does not read it |
/// | `use_generals` / `use_spies` / `use_scouts` | the 128-frame spellcaster turn (§5) | no spells |
pub mod seams {}

pub const SLOTS: usize = 16;

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
const BARK: crate::tech::TypeId = 0x143;
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

    /// `Army::close`: `Group::action_halt(g, 0)` on the one group — the
    /// units stop where they are (`docs/GROUPS.md` §7) — then the slot is
    /// freed.
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
        a.units.clear();
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
    ///
    /// **A squad joins whole.** `add_unit` is `Group::add(o, who, 0, 0)`
    /// followed by `Unit::set_group(unit, group, 0)`, and both walk the
    /// figure chain: `Group::add` replaces a non-captain by its captain and
    /// then recurses down `o_down` (`docs/GROUPS.md` §4.1), and
    /// `set_group@00605220` writes `+0x80` down the same chain. So the
    /// group's member list holds `captain, o_down, …` and every figure's
    /// `Object::get_army` answers at once — which is what keeps a squad's
    /// second and third figures out of [`Sim::add_to_army`] on the frame
    /// their captain joins (§4.3).
    pub fn army_add_unit(&mut self, who: Player, slot: usize, u: usize) {
        if !self.units[u].alive() {
            return;
        }
        let cap = self.captain_of(u);
        // `captain, o_down, …` is the order `Group::add`'s recursion
        // leaves; [`Sim::squad_of`] answers in object order, which is the
        // same for a squad born together and not guaranteed to be.
        let mut chain = self.squad_of(cap);
        chain.sort_by_key(|&f| (f != cap, f));
        for f in chain {
            let captain = self.is_captain(f);
            let a = &mut self.armies[who as usize].list[slot];
            if a.units.contains(&f) {
                continue;
            }
            a.units.push(f);
            a.num_units += 1;
            a.num_captains += i32::from(captain);
        }
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
        if !self.push_group(&g, true) {
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
    /// variable and the army slot gate, is not modelled.
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
        let p = self.units[u].pos;
        // The own-territory arm: inside a city's radius (`tile & 0x100`)
        // and on a cell my own territory holds.
        let home = self.world.tile_mask(p.tile()) & crate::world::tile::CITY_RADIUS != 0
            && self.world.owner_at(p) == crate::world::Owner::Player(who);
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
        // 1. Disband: the survivors walk home, then the slot is freed
        //    either way.
        {
            let a = &self.armies[w].list[slot];
            if a.num_standard < 1 && a.status & status::MUSTERING == 0 {
                if let Some(c) = self
                    .cities_of(who)
                    .into_iter()
                    .find(|&c| self.cities[c].alive)
                {
                    let p = self.cities[c].pos;
                    self.send_here(who, slot, p, MoveKind::MoveTo);
                }
                self.close_army(who, slot);
                return;
            }
        }
        // 2. A human's ping: the AI stands aside while the countdown runs.
        if self.armies[w].list[slot].human_frame != 0 {
            let p = self.armies[w].list[slot].pos;
            self.send_here(who, slot, p, MoveKind::AttackTo);
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
                    self.find_muster_spot(who, slot, t, false);
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
        // 6. Dispatch, each `if` independent (§6).
        if self.armies[w].list[slot].status & status::MUSTERING != 0 {
            self.army_set_stance(who, slot, Stance::Defensive);
            self.do_mustering(who, slot);
        }
        if self.armies[w].list[slot].valid
            && self.armies[w].list[slot].status & status::DEFENDING != 0
        {
            self.army_set_stance(who, slot, Stance::Defensive);
            self.do_defending(who, slot);
        }
        if self.armies[w].list[slot].valid
            && self.armies[w].list[slot].status & status::MARCHING != 0
        {
            self.army_set_stance(who, slot, Stance::Aggressive);
            self.do_marching(who, slot);
        }
        if self.armies[w].list[slot].valid {
            if self.armies[w].list[slot].status & status::FORMING != 0 {
                self.do_forming(who, slot);
            } else if self.army_is_engaged(who, slot) {
                self.engagement(who, slot);
            }
        }
        if self.armies[w].list[slot].valid
            && self.armies[w].list[slot].status & status::TRANSPORTING != 0
        {
            self.do_transporting(who, slot);
        }
        if self.armies[w].list[slot].valid && self.armies[w].list[slot].navy {
            self.army_set_stance(who, slot, Stance::Raid);
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
        // **No target is the retarget branch, not the march.** The
        // original opens on `iVar4 = field_0x30; if (iVar4 < 0) goto
        // LAB_006f3fb2` (`006f3df0`+0x1d), so an army that reaches
        // `do_marching` holding nothing goes straight to `find_target` —
        // and it reaches it holding nothing on the very tick
        // `do_mustering` releases it, because `Army::process` re-reads
        // `status` between its dispatch `if`s and `status = 2` leaves the
        // target at `-1` (§9, §11). Modelling only the "target present but
        // stale" arm cost Great Lakes' word 7930: the original spends
        // `find_target`'s coin once and its per-candidate score twice
        // there and this crate spent nothing (item 317).
        let mut retarget = self.armies[w].list[slot].target.is_none();
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

    /// `Army::march_to_target` (§9): not engaged → forming and marching;
    /// engaged, and only on the first tick of the fight, the orders.
    fn march_to_target(&mut self, who: Player, slot: usize) {
        let w = who as usize;
        if self.armies[w].list[slot].target.is_none() {
            return;
        }
        if !self.army_is_engaged(who, slot) {
            let a = &mut self.armies[w].list[slot];
            a.status = (a.status & !status::MARCHED) | status::FORMING | status::MARCHING;
            return;
        }
        if self.armies[w].list[slot].status & status::MARCHED != 0 {
            return;
        }
        // The formation origin: the army's point stepped one cell along the
        // muster angle, and the facing from the target back to it.
        let (pos, ang, target, hurry) = {
            let a = &self.armies[w].list[slot];
            (a.pos, a.muster_angle, a.target, a.hurry)
        };
        let origin = step_along(pos, ang, UNITS_PER_CELL);
        let Some(t) = target else { return };
        let tp = self.pos_of(t);
        let angle = find_angle(origin.x - tp.x, origin.y - tp.y);
        self.armies[w].list[slot].angle = angle;
        self.army_stance_rule(who, slot);
        let siege = self.army_siege_here(who, slot, origin);
        let g = self.army_group(who, slot);
        // A **city-centre** target that has lost nine tenths of its hits is
        // walked to directly (`damage < hits × 9/10` false, `6f5007`);
        // anything else takes the move-or-siege choice at the origin.
        let nearly_dead = match t {
            Obj::Building(b) => {
                let bd = &self.buildings[b];
                let centre = bd.city.is_some_and(|c| self.cities[c].building == b);
                centre && bd.hits - bd.health >= bd.hits * 9 / 10
            }
            Obj::Unit(_) => false,
        };
        if nearly_dead {
            self.group_action_move_to(&g, tp, QueuePos::New, true, angle, MoveKind::AttackTo, true);
        } else if !siege && (hurry != 0 || self.army_target_is_a_friend_under_attack(who, slot)) {
            self.army_set_stance(who, slot, Stance::Aggressive);
            self.group_action_move_to(
                &g,
                origin,
                QueuePos::New,
                true,
                angle,
                MoveKind::AttackTo,
                true,
            );
        } else {
            self.group_action_siege_attack_to(&g, origin, angle);
        }
        self.armies[w].list[slot].status |= status::MARCHED;
    }

    // ---- forming (§8) ----

    /// `Army::do_forming`: the muster cell's centre, projected a tile per
    /// group, and one order per group.
    fn do_forming(&mut self, who: Player, slot: usize) {
        if self.army_is_engaged(who, slot) {
            return;
        }
        let w = who as usize;
        let (muster, ang, hurry) = {
            let a = &self.armies[w].list[slot];
            (a.muster, a.muster_angle, a.hurry)
        };
        let groups = self.armies[w].list[slot].num_groups();
        let origin = step_along(cell_centre(muster), ang, groups * 0xc0);
        self.army_stance_rule(who, slot);
        let siege = self.army_siege_here(who, slot, origin);
        let friendly_attacked = self.army_target_is_a_friend_under_attack(who, slot);
        let g = self.army_group(who, slot);
        if !siege && (hurry != 0 || friendly_attacked) {
            self.army_set_stance(who, slot, Stance::Aggressive);
            self.group_action_move_to(
                &g,
                origin,
                QueuePos::New,
                true,
                ang,
                MoveKind::AttackTo,
                true,
            );
        } else {
            self.group_action_siege_attack_to(&g, origin, ang);
        }
    }

    /// §8.4's second half of the move-or-siege choice, shared with
    /// `march_to_target` (`6f4559`, `6f4f55`): the target is **mine or an
    /// ally's** — `is_ally`, so a leader merely at peace does not count —
    /// and the building's city is active and attacked.
    fn army_target_is_a_friend_under_attack(&self, who: Player, slot: usize) -> bool {
        let Some(t) = self.armies[who as usize].list[slot].target else {
            return false;
        };
        let tw = self.owner_of(t);
        if tw != who && !self.is_ally(who, tw) {
            return false;
        }
        matches!(t, Obj::Building(b) if self.buildings[b]
            .city
            .is_some_and(|c| self.cities[c].alive && self.cities[c].no_heal))
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
            && self.tech[w].ages < 4
            && self.army_count_siege(who, slot) == 0
        {
            self.army_set_stance(who, slot, Stance::Raid);
        }
    }

    /// §8.3's siege test: an enemy combat building within a cell of the
    /// formation's origin, and I have siege.
    fn army_siege_here(&self, who: Player, slot: usize, origin: Pos) -> bool {
        if self.army_count_siege(who, slot) == 0 {
            return false;
        }
        self.buildings.iter().any(|b| {
            b.alive
                && b.active
                && self.is_enemy(who, b.owner)
                && b.ty.is_some_and(|t| self.build_types[t].attack != 0)
                && dist(b.pos, origin) <= UNITS_PER_CELL
        })
    }

    /// `Army::set_stance(s)` (§6): `Group::action_stance` on the one group,
    /// and only when its stance panel is the combat one.
    fn army_set_stance(&mut self, who: Player, slot: usize, s: Stance) {
        let g = self.army_group(who, slot);
        if self.group_stance_type(&g) != crate::group::StanceType::Combat {
            return;
        }
        self.group_action_stance(&g, crate::group::stance_option(s));
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
            self.find_muster_spot(who, slot, target, false);
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

        // The two averages of `sea_combat` (audit B.20: the listing's base
        // is the leader plus 8, so `+0x944` there is `+0x94c`).
        let (mut ours, mut theirs) = ((0, 0), (0, 0));
        for i in 0..self.players.len() {
            if self.defeated[i] {
                continue;
            }
            let ip = i as Player;
            if self.is_ally(who, ip) {
                ours = (ours.0 + self.ai[i].census.sea_combat, ours.1 + 1);
            } else if self.is_enemy(who, ip) {
                theirs = (theirs.0 + self.ai[i].census.sea_combat, theirs.1 + 1);
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
                if i != w {
                    // `defense_mod` is the switch (audit B.22): above 0x100
                    // only allies are considered, below it only enemies.
                    // `weak`/`strong`/the tribute period are seams.
                    let dm = self.ai[w].defense_mod;
                    if !enemy && (!allied || dm < 0x100) {
                        continue;
                    }
                    if enemy && dm > 0x100 {
                        continue;
                    }
                    // The difficulty gate (listing `6f6ce1`–`6f6fb4`): at
                    // `diff < 2` the stamp, the aggressive-army test and
                    // the coin come first, and a leader that passes them
                    // **goes on** to the tests every difficulty applies —
                    // the `diff == 1` clause is reached only that way
                    // (run26: the decompiler's `else` had hidden the
                    // `goto` and left it dead).
                    if diff < 2 {
                        if !(diff != 0 || self.nation[i].human) {
                            continue;
                        }
                        if self.ai[i].frame_attacked + 0x1c20 > frame {
                            continue;
                        }
                        let go = match self.find_aggressive_army(who) {
                            Some(k) => k == slot,
                            None => {
                                self.mark(SITE_FIND_TARGET_COIN);
                                self.rng.get(0, 0xffff) & 1 == 0
                            }
                        };
                        if !go {
                            continue;
                        }
                    }
                    if diff == 1
                        && (num_captains >= my_combat / 2
                            || self.find_aggressive_army(who).is_some_and(|k| k != slot))
                    {
                        continue;
                    }
                    if !(diff < 2
                        || allied
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
                    // The test before the draw (B.25): on the two easiest
                    // difficulties a city of mine or an ally's is scored
                    // only if I founded it; an enemy's always is. (Run26
                    // caught the inverse — the human's capital dropped and
                    // one draw short.)
                    if diff <= 1 && allied && cd.founder != who {
                        continue;
                    }
                    self.mark(SITE_FIND_TARGET_SCORE);
                    let mut v = self.rng.get(0, 0xffff) % 200 + 900;
                    if let Some(cap) = capital
                        && enemy
                    {
                        if weak_army && !(cd.founder == who && cd.unassimilated) {
                            continue;
                        }
                        let cc = cd.pos.cell();
                        let d = vector_dist((cc.x - cap.x).abs(), (cc.y - cap.y).abs());
                        v += d * -50 / self.world.width();
                    }
                    let attacked = cd.no_heal;
                    let lvl = self.city_level_of(c);
                    if !enemy {
                        // Not at war: me, or an ally — a neutral never
                        // reaches the scoring. `diplos` is 2 on its
                        // diagonal, so `me` passes the allied test the
                        // same way an ally does (`LeaderData +0x8` is
                        // `who`, not an ally slot — `types.txt`): my own
                        // untroubled city takes the size factor and the
                        // mark's division.
                        if allied {
                            if !attacked {
                                if num_captains < 20 || siege < 2 {
                                    if num_captains < 10 || siege == 0 {
                                        v *= size_factor(lvl);
                                    }
                                } else {
                                    v = v * (4 - lvl) / 4;
                                }
                                // §13's "no muster spot" mark.
                                if cd.no_muster {
                                    v /= 20;
                                }
                            } else if team_style == 2 {
                                v *= 5;
                            }
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
            // SEAM, and **not** a quiet one: `docs/ARMY.md` §12's probe
            // (`find_target@006f69b0:1120`-`1199`) is the whole of Great
            // Lakes' frame 8186, where the long word parts. The comment
            // that used to stand here said "nothing changes"; it is the
            // frame's forty-eight `Unit::find_attack_pos` draws, the six
            // mandatory attack orders on the farm `0/2004`, the stance 5
            // and the pushed group. `docs/COMBAT.md` §17 specifies it and
            // `run53_s_8186_is_find_target_s_probe_and_its_ring_walks`
            // pins what the original does on the frame.
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
        if self.find_muster_spot(who, slot, target, false) {
            let a = &mut self.armies[w].list[slot];
            a.muster_angle = Angle(a.muster_angle.0.wrapping_add(i32::MIN));
        } else {
            self.close_army(who, slot);
        }
    }

    // ---- the muster spot (§13) ----

    /// `LandData::move_rate` — `lands[class].+0x100`, what a neighbour
    /// cell adds to a candidate's score. `Lands::init@0067e730` writes
    /// `0x100` to every one of the nine lands' `move_rate` (and
    /// `combat_bonus`), and nothing else in the export writes either, so
    /// the score is a constant: 256 per admissible neighbour, and the
    /// ring's walk order breaks the ties.
    const LAND_MOVE_RATE: i32 = 0x100;

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
        let w = who as usize;
        self.armies[w].list[slot].hurry = 0;
        let tw = self.owner_of(target);
        let (apos, navy) = {
            let a = &self.armies[w].list[slot];
            (a.pos, a.navy)
        };
        let ac = apos.cell();
        let tc = self.pos_of(target).cell();
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
                                    && !u.is_gaia()
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
                        a.muster_angle = find_angle(mc.x - tc.x, mc.y - tc.y);
                        a.hurry = 1;
                        return true;
                    }
                    if muster.is_none() {
                        muster = Some(bpos.cell());
                    }
                }
            }
            // No enemy at the chain: a provisional muster — the damaged
            // building's cell, else the target's, one row down — and on
            // into the ring search, which overrides it with a best.
            let m = below(muster.unwrap_or(tc), height);
            let a = &mut self.armies[w].list[slot];
            a.muster = m;
            a.muster_angle = find_angle(m.x - tc.x, m.y - tc.y);
        }
        // The ring search. `OBJECT_CITY` is the city centre itself, not a
        // member.
        let city = match target {
            Obj::Building(b) => self.buildings[b]
                .city
                .filter(|&c| self.cities[c].building == b)
                .map(|c| (b, c)),
            Obj::Unit(_) => None,
        };
        let siege = self.army_count_siege(who, slot);
        let inner = match city {
            Some((b, c)) if !navy => {
                if self.building_unassimilated(b) {
                    4
                } else {
                    let mut r = self.radius_of(c) / 4 + 1;
                    if siege == 0 && who != tw {
                        r /= 2;
                    }
                    r
                }
            }
            _ => 5,
        };
        let outer = (inner + (3 * i32::from(navy) + 1) * 2).min(0x40);
        let walk = if flag { 0x31 } else { 9 };
        let scored = if navy { 0x19 } else { 9 };
        let ally = self.is_ally(who, tw);
        let near = if ally { 4 } else { 2 };
        // A land army with any transport level may muster in any land
        // region (`leader_flags & 0x700`, `region < 0x41`).
        let any_land = !navy && self.transport_level(who) as i32 != 0;
        let reg = self.armies[w].list[slot].reg;
        let others: Vec<Cell> = self.armies[w]
            .list
            .iter()
            .enumerate()
            .filter(|(j, a)| *j != slot && a.valid)
            .map(|(_, a)| a.muster)
            .collect();
        let circle = crate::ai_place::circle();
        let mut best: Option<(i32, usize, Cell)> = None;
        // The original's `local_c`: the last in-bounds candidate's verdict.
        let mut last_valid = false;
        for i in circle.radius[inner as usize]..circle.radius[outer as usize] {
            let cand = Cell::new(ac.x + circle.x[i], ac.y + circle.y[i]);
            if !self.world.contains(cand) {
                continue;
            }
            // Not too near another army of mine — the first that is ends
            // the candidate's walk before it scores, so it cannot win.
            let too_near = others
                .iter()
                .any(|m| vector_dist((cand.x - m.x).abs(), (cand.y - m.y).abs()) < near);
            if !too_near {
                let mut score = 0;
                let mut out = false;
                for (j, &(dx, dy)) in MOVE_49[..walk].iter().enumerate() {
                    let n = Cell::new(cand.x + dx, cand.y + dy);
                    if !self.world.contains(n) {
                        out = true;
                        break;
                    }
                    if !navy {
                        // The neighbour's own neighbour in the same
                        // direction: another leader's cell — not mine,
                        // not an ally's, not the target owner's — is out.
                        // A `−2` owner is "another leader" to the test.
                        let nn = Cell::new(n.x + dx, n.y + dy);
                        if self.world.contains(nn) {
                            let foreign = match self.world.owner(nn) {
                                Owner::None => false,
                                Owner::Ambiguous => true,
                                Owner::Player(p) => p != who && !self.is_ally(who, p) && p != tw,
                            };
                            if foreign {
                                out = true;
                                break;
                            }
                        }
                    }
                    if j >= scored {
                        continue;
                    }
                    let nreg = self.world.region_of(n);
                    let in_region = nreg == reg
                        || (any_land
                            && nreg.is_some_and(|r| self.world.terrain(r) == Terrain::Land));
                    if !in_region {
                        out = true;
                        break;
                    }
                    let f = self.world.cell_data(n).flags;
                    let water = self.world.is_ocean(n);
                    // The same five tests `WorldData::get_land` runs, which
                    // is what `World::land_class` is.
                    let class = self.world.land_class(n);
                    let admissible = if navy {
                        water
                    } else {
                        !water && class != 5 && class != 4 && class != 3
                    };
                    if !admissible {
                        out = true;
                        break;
                    }
                    if f & cell::BUILDING == 0 {
                        score += Self::LAND_MOVE_RATE;
                    } else if navy && j > 8 {
                        score += 10;
                    }
                }
                last_valid = !out;
                if !out && score > best.map_or(0, |(s, _, _)| s) {
                    best = Some((score, i, cand));
                }
            }
            // The early stop: more than 0x28 entries past the best.
            if let Some((_, bi, _)) = best
                && i - bi > 0x28
            {
                last_valid = true;
                break;
            }
        }
        if let Some((_, _, m)) = best {
            let a = &mut self.armies[w].list[slot];
            a.muster = m;
            // From the army's point here, not the target's (listing
            // `6f67aa`; audit B.46).
            a.muster_angle = find_angle(m.x - ac.x, m.y - ac.y);
            if let Some((_, c)) = city
                && who == tw
                && self.cities[c].alive
            {
                self.cities[c].no_muster = false;
            }
            return last_valid;
        }
        // No cell. A land army at a city centre that is not mine or an
        // ally's, or that is damaged: the city's cell, one row down.
        if let Some((b, c)) = city
            && !navy
        {
            let bd = &self.buildings[b];
            let damaged = bd.health < bd.hits;
            if !ally || damaged {
                let m = below(self.cities[c].pos.cell(), height);
                let a = &mut self.armies[w].list[slot];
                a.muster = m;
                a.muster_angle = find_angle(m.x - tc.x, m.y - tc.y);
                return true;
            }
        }
        // An active, untroubled city centre of my own: the mark §12
        // divides by 20, and the caller sees the failure.
        if let Some((_, c)) = city
            && who == tw
            && self.cities[c].alive
            && !self.cities[c].no_heal
        {
            self.cities[c].no_muster = true;
        }
        last_valid
    }

    /// `find_unit(SEARCH_ENEMY, FILTER_COMBAT)`: the nearest enemy unit of
    /// `who` with an attack within `range` of `p`.
    ///
    /// The search's leader loop stops at eight (`world::PLAYER_SLOTS`), so
    /// gaia's animals are not in its space at all — and asking the diplomacy
    /// question about them is what panicked the first fuzzed seed.
    fn nearest_enemy_attacker(&self, who: Player, p: Pos, range: i32) -> Option<usize> {
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
        // Bit 1: a resource region nobody of mine is in, that no *other*
        // active leader holds densely — `size < cities × 200`.
        if self.world.region_flags(r) & 8 != 0
            && mine == 0
            && self.ai[w]
                .census
                .reg_peasants
                .get(r as usize)
                .copied()
                .unwrap_or(0)
                == 0
        {
            let size = self.world.region_size(r);
            let dense = (0..self.players.len()).any(|i| {
                i != w && !self.defeated[i] && {
                    let theirs = self.ai[i]
                        .census
                        .reg_cities
                        .get(r as usize)
                        .copied()
                        .unwrap_or(0);
                    theirs != 0 && size < theirs * 200
                }
            });
            if !dense {
                g |= 1;
            }
        }
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
                self.find_muster_spot(who, slot, target, true);
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
                a.num_captains >= 2
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
    /// §4.1's coin, all four outcomes: a citizen leaving a building spends
    /// nothing, a **scout** spends one draw on the `% 2` arm, a **Bark**
    /// one on the `% 3` arm, and a **spy** joins with no draw at all.
    ///
    /// The whole point of the predicate is that the site is *rare*: it is
    /// reached eleven times in run54's 24,000 frames, and the citizen row
    /// here is why. A version that threw the coin for every AI unit passes
    /// the scout and Bark rows and fails the other two.
    #[test]
    fn come_out_s_army_coin_is_thrown_by_the_two_scout_lineages_alone() {
        use crate::ai_load::uflags2;

        let mut sim = crate::Sim::new(crate::Tuning::RON, crate::world::World::new(8, 8), 2);
        sim.world.fill_region(
            crate::world::Terrain::Land,
            crate::world::Cell::new(0, 0),
            crate::world::Cell::new(7, 7),
        );
        sim.nation[1].human = false;

        let make = |sim: &mut crate::Sim, flags2: u32, tree: Option<crate::tech::TypeId>| {
            let t = sim.add_unit_type(crate::UnitType {
                hits: 40,
                ..crate::UnitType::default()
            });
            sim.unit_types[t].cols.unit_flags2 = flags2;
            sim.unit_types[t].tree = tree;
            let index = sim
                .find_free(1, crate::UNIT_BASE, crate::BUILD_BASE)
                .unwrap();
            let mut u = crate::Unit::new(1, index, crate::world::Pos::new(384, 384), 40);
            u.ty = Some(t);
            sim.add_unit(u)
        };

        // A plain citizen: neither `is_special` nor `is(BARK)`, and no
        // draw — this is the row that keeps the site rare.
        let citizen = make(&mut sim, 0, None);
        let before = sim.rng.seed;
        sim.come_out_join_army(citizen);
        assert_eq!(sim.rng.seed, before, "a citizen throws no coin");
        assert!(sim.army_of(citizen).is_none());

        // A scout: `is_special` is `is(SCOUT)` folded into `unit_flags2`.
        let scout = make(&mut sim, uflags2::SCOUT, None);
        let before = sim.rng.seed;
        sim.come_out_join_army(scout);
        assert_ne!(sim.rng.seed, before, "the scout arm's `% 2`");

        // A Bark: the naval-scout lineage, the `% 3` arm, and no
        // `unit_flags2` bit of its own.
        let bark = make(&mut sim, 0, Some(0x143));
        let before = sim.rng.seed;
        sim.come_out_join_army(bark);
        assert_ne!(sim.rng.seed, before, "the Bark arm's `% 3`");

        // A spy: the draw-free arm, and the only one that always joins.
        let spy = make(&mut sim, 0, Some(0x3a));
        let before = sim.rng.seed;
        sim.come_out_join_army(spy);
        assert_eq!(sim.rng.seed, before, "the `is(SPY)` arm spends nothing");
        // It reaches `add_to_army` unconditionally; whether a slot comes
        // back is that function's own business, and this bare world has no
        // city to seed one at (§4).
        assert!(sim.army_of(spy).is_none(), "no city, so no army to seed");

        // The human's own units never reach any of it.
        let index = sim
            .find_free(0, crate::UNIT_BASE, crate::BUILD_BASE)
            .unwrap();
        let t = sim.units[scout].ty.unwrap();
        let mut h = crate::Unit::new(0, index, crate::world::Pos::new(384, 384), 40);
        h.ty = Some(t);
        let h = sim.add_unit(h);
        let before = sim.rng.seed;
        sim.come_out_join_army(h);
        assert_eq!(sim.rng.seed, before, "`unit_masks & 0x40000` gates it");
    }

    use super::*;

    fn sim_with_city() -> (Sim, usize) {
        sim_with_city_slots(2)
    }

    fn sim_with_city_slots(slots: usize) -> (Sim, usize) {
        let mut sim = Sim::new(
            crate::tuning::Tuning::RON,
            crate::world::World::new(60, 60),
            slots,
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
            no_muster: false,
            was_capital: 0,
            capture_stamp: 0,
            assimilation_timer: 0,
            attack_stamp: 0,
            capture_strength: 0,
            pop: 1,
            has_citizen: false,
            source: None,
            trade_val: 0,
            traded_with: [0; 8],
        });
        (sim, c)
    }

    // ---- the order-issuing half (§8, §9, §11, §14; docs/GROUPS.md) ----

    fn soldier_type(sim: &mut Sim) -> usize {
        sim.add_unit_type(crate::UnitType {
            hits: 100,
            combat: crate::combat::Profile {
                attack: 15,
                uber_size: 1,
                ..crate::combat::Profile::default()
            },
            ..crate::UnitType::default()
        })
    }

    fn put(sim: &mut Sim, who: Player, ty: usize, p: Pos) -> usize {
        let index = i16::try_from(sim.units.len()).unwrap();
        let hits = sim.unit_types[ty].hits;
        let mut u = crate::Unit::new(who, index, p, hits);
        u.ty = Some(ty);
        u.on_map = true;
        u.kind = sim.unit_types[ty].kind;
        u.movement.speed = 25;
        u.movement.turning = crate::movement::Turning {
            type_turn_speed: crate::movement::degrees_to_angle(45).0,
            packed: false,
            instant_from_stop: true,
            wide_limit: false,
        };
        sim.add_unit(u)
    }

    #[test]
    fn do_forming_walks_the_army_to_the_projected_muster_origin() {
        let (mut sim, c) = sim_with_city();
        let t = soldier_type(&mut sim);
        let slot = sim.init_army(1, Some(c));
        let a = put(&mut sim, 1, t, Pos::new(0x1000, 0x1000));
        let b = put(&mut sim, 1, t, Pos::new(0x1200, 0x1000));
        sim.army_add_unit(1, slot, a);
        sim.army_add_unit(1, slot, b);
        sim.armies[1].list[slot].muster = Cell { x: 20, y: 20 };
        sim.armies[1].list[slot].muster_angle = Angle(0);
        sim.do_forming(1, slot);
        // One group, so the origin is the muster cell's centre projected
        // one tile north (§8: `num_groups × 0xc0` along the muster angle,
        // which is 0 — due north, `y − 0xc0`).
        let want = step_along(cell_centre(Cell { x: 20, y: 20 }), Angle(0), 0xc0);
        for u in [a, b] {
            let o = *sim.current_order(u).expect("an order from do_forming");
            // Two of them in a land formation, so the order the original
            // hands each is a `GroupAttackToOrder` — `GROUP_ATTACK_TO`,
            // not the plain `ATTACK_TO` (item 237; §1.2).
            assert_eq!(o.index(), crate::orders::index::GROUP_ATTACK_TO);
            let crate::orders::Body::Move(m) = o.body else {
                panic!("a move");
            };
            // The destination is the origin snapped to its 48-unit centre
            // (`docs/ORDERS.md` §4.3).
            assert_eq!(m.dest.x.div_euclid(0x30), want.x.div_euclid(0x30));
            assert_eq!(m.dest.y.div_euclid(0x30), want.y.div_euclid(0x30));
        }
    }

    #[test]
    fn a_forming_army_actually_moves_its_units_over_the_ticks() {
        let (mut sim, c) = sim_with_city();
        let t = soldier_type(&mut sim);
        let slot = sim.init_army(1, Some(c));
        let a = put(&mut sim, 1, t, Pos::new(0x1000, 0x1000));
        sim.army_add_unit(1, slot, a);
        sim.armies[1].list[slot].muster = Cell { x: 30, y: 30 };
        let before = sim.units[a].pos;
        sim.do_forming(1, slot);
        // The order step and the body follow, as `Objects::process_all`
        // runs them; the rest of the tick needs a tech tree this fixture
        // has no use for.
        for f in 0..64 {
            sim.work(a, f);
            sim.process_movement(a);
        }
        assert_ne!(
            sim.units[a].pos, before,
            "the order the group issued is stepped by the order system"
        );
        let d = |p: Pos| {
            vector_dist(
                (p.x - cell_centre(Cell { x: 30, y: 30 }).x).abs(),
                (p.y - cell_centre(Cell { x: 30, y: 30 }).y).abs(),
            )
        };
        assert!(
            d(sim.units[a].pos) < d(before),
            "and it walks toward the muster spot"
        );
    }

    #[test]
    fn engagement_points_the_whole_army_at_the_first_fighter_s_target() {
        let (mut sim, c) = sim_with_city();
        let t = soldier_type(&mut sim);
        let slot = sim.init_army(1, Some(c));
        let p = Pos::new(0x5000, 0x5000);
        sim.armies[1].list[slot].pos = p;
        // Three of mine at the army's point; one is already fighting, and
        // `is_engaged` wants more than a quarter (§11).
        let mine: Vec<usize> = (0..3)
            .map(|k| put(&mut sim, 1, t, Pos::new(p.x + k * 0x40, p.y)))
            .collect();
        for &u in &mine {
            sim.army_add_unit(1, slot, u);
        }
        let foe = put(&mut sim, 0, t, Pos::new(p.x + 0x100, p.y));
        sim.add_attack_order(mine[0], Obj::Unit(foe), QueuePos::New, true, true);
        assert!(
            sim.army_is_engaged(1, slot),
            "one of three is over the quarter"
        );
        sim.engagement(1, slot);
        for &u in &mine {
            assert_eq!(
                sim.units[u].combat.target,
                Some(Obj::Unit(foe)),
                "every member takes the engaged unit's target"
            );
        }
    }

    /// An armed enemy building at the army's point: `active`, so
    /// `group_action_attack` will order against it. The earlier form of
    /// the test below left it unarmed, and its "the army ignores a
    /// building target" passed only because nobody could be ordered at
    /// an inactive target — the opposite of §11's fallback rule.
    fn armed_wall(sim: &mut Sim, p: Pos) -> usize {
        let wall = sim.add_building(0, p, 1);
        sim.buildings[wall].combat = Some(crate::combat::Profile::default());
        sim.buildings[wall].hits = 100;
        sim.buildings[wall].health = 50;
        wall
    }

    #[test]
    fn engagement_falls_back_to_the_last_qualifier_s_building_target() {
        let (mut sim, c) = sim_with_city();
        let t = soldier_type(&mut sim);
        let slot = sim.init_army(1, Some(c));
        let p = Pos::new(0x5000, 0x5000);
        sim.armies[1].list[slot].pos = p;
        let a = put(&mut sim, 1, t, p);
        let b = put(&mut sim, 1, t, Pos::new(p.x + 0x40, p.y));
        sim.army_add_unit(1, slot, a);
        sim.army_add_unit(1, slot, b);
        let wall = armed_wall(&mut sim, Pos::new(p.x + 0x100, p.y));
        sim.add_attack_order(a, Obj::Building(wall), QueuePos::New, true, true);
        sim.engagement(1, slot);
        assert_eq!(
            sim.units[b].combat.target,
            Some(Obj::Building(wall)),
            "`is_map_unit` gates only the break: with no map-unit target the \
             last qualifying unit's building is the army's (§11)"
        );
    }

    #[test]
    fn engagement_yields_nothing_when_the_last_qualifier_s_target_is_gone() {
        let (mut sim, c) = sim_with_city();
        let t = soldier_type(&mut sim);
        let slot = sim.init_army(1, Some(c));
        let p = Pos::new(0x5000, 0x5000);
        sim.armies[1].list[slot].pos = p;
        let a = put(&mut sim, 1, t, p);
        let b = put(&mut sim, 1, t, Pos::new(p.x + 0x40, p.y));
        let idle = put(&mut sim, 1, t, Pos::new(p.x + 0x80, p.y));
        for &u in &[a, b, idle] {
            sim.army_add_unit(1, slot, u);
        }
        let wall = armed_wall(&mut sim, Pos::new(p.x + 0x100, p.y));
        sim.add_attack_order(a, Obj::Building(wall), QueuePos::New, true, true);
        // `b` qualifies — it carries an attack action — but its target
        // order has gone negative, as an attack whose target died reads.
        sim.add_attack_order(b, Obj::Building(wall), QueuePos::New, true, true);
        sim.units[b].combat.target = None;
        sim.engagement(1, slot);
        assert_eq!(
            sim.units[idle].combat.target, None,
            "`6f5345`: the last qualifier's −1 overwrites the earlier wall, \
             and the tail at `6f5383` gives nothing"
        );
    }

    #[test]
    fn close_halts_the_units_it_held() {
        let (mut sim, c) = sim_with_city();
        let t = soldier_type(&mut sim);
        let slot = sim.init_army(1, Some(c));
        let a = put(&mut sim, 1, t, Pos::new(0x1000, 0x1000));
        sim.army_add_unit(1, slot, a);
        sim.add_move_order(
            a,
            Pos::new(0x4000, 0x4000),
            MoveKind::MoveTo,
            QueuePos::New,
            true,
        );
        sim.close_army(1, slot);
        assert!(
            sim.current_order(a).is_none(),
            "`Army::close` is `Group::action_halt(g, 0)` per group"
        );
    }

    #[test]
    fn send_here_moves_the_point_the_muster_cell_and_the_units() {
        let (mut sim, c) = sim_with_city();
        let t = soldier_type(&mut sim);
        let slot = sim.init_army(1, Some(c));
        let a = put(&mut sim, 1, t, Pos::new(0x1000, 0x1000));
        sim.army_add_unit(1, slot, a);
        let to = Pos::new(0x6000, 0x7000);
        sim.send_here(1, slot, to, MoveKind::MoveTo);
        let army = &sim.armies[1].list[slot];
        assert_eq!(army.pos, to);
        assert_eq!(army.muster, to.cell());
        assert_eq!(sim.order_type(a), crate::orders::index::MOVE_TO);
    }

    /// **`Unit::think`'s tail joins supply wagons and heroes, and nothing
    /// else** (§4) — the rule item 68 was.
    ///
    /// The listing at `005f7615` is `if (!is_supply && !is_hero) { … return
    /// } add_to_army(this)`, so a fighting unit reaching the *tail* does
    /// **not** join: `think_attack`'s own `add_to_army` is a different
    /// site, behind that function's city search, and `think_attack` is not
    /// reached by a unit that has a job. The implementation used to join
    /// "an attacker that is not a scout or a caravan" here, which
    /// conscripted run10's woodcutters.
    #[test]
    fn only_a_supply_wagon_or_a_hero_joins_an_army_from_the_think_tail() {
        let (mut sim, _c) = sim_with_city();
        // A plain fighting type — an attack, no lineage bit.
        let soldier = soldier_type(&mut sim);
        let a = put(&mut sim, 1, soldier, Pos::new(0x1000, 0x1000));
        sim.think_join_army(a);
        assert_eq!(
            sim.army_of(a),
            None,
            "the tail is not `think_attack`: an attacker does not join here"
        );

        // The same type with the supply lineage bit — `unit_flags2 & 0x40`.
        let wagon = sim.add_unit_type(crate::UnitType {
            hits: 100,
            cols: crate::ai_load::UnitCols {
                unit_flags2: uflags2::SUPPLY_OR_HERO,
                ..crate::ai_load::UnitCols::default()
            },
            ..crate::UnitType::default()
        });
        let b = put(&mut sim, 1, wagon, Pos::new(0x1000, 0x1000));
        sim.think_join_army(b);
        assert!(
            sim.army_of(b).is_some(),
            "`is_supply` is the first of the tail's two ways in"
        );

        // And the hero bit — `unit_flags2 & 0x20`.
        let hero = sim.add_unit_type(crate::UnitType {
            hits: 100,
            cols: crate::ai_load::UnitCols {
                unit_flags2: uflags2::GENERAL,
                ..crate::ai_load::UnitCols::default()
            },
            ..crate::UnitType::default()
        });
        let h = put(&mut sim, 1, hero, Pos::new(0x1000, 0x1000));
        sim.think_join_army(h);
        assert!(sim.army_of(h).is_some(), "`is_hero` is the second");
    }

    /// **`think_attack`'s head is the one an AI soldier takes** (§4.2) —
    /// the site the think tail's test above says is a different one.
    ///
    /// Three gates, each made to fail: a type with an attack but no
    /// `role & 0x10000` is an armed citizen and does not join (that
    /// omission was run53's frame 307); a military type does; and a
    /// **damaged foot** unit standing inside its own city's radius stays
    /// where it is.
    #[test]
    fn think_attack_s_head_joins_a_military_unit_and_leaves_an_armed_citizen() {
        use crate::ai_load::role;
        let (mut sim, _c) = sim_with_city();
        // An attack, and no military bit: `determine_roles`' civilian arm.
        let armed_citizen = soldier_type(&mut sim);
        let a = put(&mut sim, 1, armed_citizen, Pos::new(0x1000, 0x1000));
        assert_eq!(
            sim.think_attack_join_army(a),
            None,
            "`Unit::think`'s step 3 needs `role & 0x10000` as well as the attack"
        );

        let soldier = sim.add_unit_type(crate::UnitType {
            hits: 100,
            combat: crate::combat::Profile {
                attack: 15,
                obj_masks: crate::combat::mask::FOOT,
                uber_size: 1,
                ..crate::combat::Profile::default()
            },
            cols: crate::ai_load::UnitCols {
                role: role::MILITARY,
                ..crate::ai_load::UnitCols::default()
            },
            ..crate::UnitType::default()
        });
        let b = put(&mut sim, 1, soldier, Pos::new(0x1000, 0x1000));
        assert!(
            sim.think_attack_join_army(b).is_some(),
            "a military type with an attack joins on its first idle think"
        );

        // The stay-and-heal arm: damaged, foot, inside my own city's
        // radius. Both halves of the territory test have to hold, so the
        // tile mask and the cell's owner are written together.
        let c = put(&mut sim, 1, soldier, Pos::new(0x1000, 0x1000));
        sim.units[c].health -= 1;
        let p = sim.units[c].pos;
        sim.world.set_owner(
            p.cell(),
            crate::world::Owner::Player(1),
            crate::world::Owner::None,
        );
        sim.world
            .set_tile_mask(p.tile(), crate::world::tile::CITY_RADIUS);
        assert_eq!(
            sim.think_attack_join_army(c),
            None,
            "a damaged foot unit at home stays there (`5f5d10`)"
        );
        // And the same unit one tile outside the radius does join.
        sim.world.set_tile_mask(p.tile(), 0);
        assert!(
            sim.think_attack_join_army(c).is_some(),
            "the stay-and-heal arm is the city radius', not the damage's alone"
        );
    }

    #[test]
    fn charge_drags_the_army_onto_the_attacker() {
        let (mut sim, c) = sim_with_city();
        let t = soldier_type(&mut sim);
        let slot = sim.init_army(1, Some(c));
        let a = put(&mut sim, 1, t, Pos::new(0x1000, 0x1000));
        sim.army_add_unit(1, slot, a);
        let foe = put(&mut sim, 0, t, Pos::new(0x2000, 0x2000));
        sim.army_charge(1, slot, Obj::Unit(foe));
        assert_eq!(sim.armies[1].list[slot].target, Some(Obj::Unit(foe)));
        assert_eq!(sim.order_type(a), crate::orders::index::ATTACK_TO);
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

    // ---- the ring search (§13) ----

    use crate::world::CellData;

    /// [`sim_with_city`] with the city building wired to its city, as
    /// `activate` leaves it, and one army mustering at the city: its
    /// point is one cell south of the city's, cell (16, 17). A level-1
    /// city's radius is 20 tiles, so `inner = 6` and `outer = 8`: the
    /// search walks rings 7 and 8 — 40 and 52 entries — around (16, 17),
    /// and the first entry of ring 7 is (−7, −3), the cell (9, 14).
    fn sim_with_army() -> (Sim, usize, usize, usize) {
        let (mut sim, c) = sim_with_city();
        let b = sim.cities[c].building;
        sim.buildings[b].city = Some(c);
        let s = sim.init_army(1, Some(c));
        assert_eq!(sim.armies[1].list[s].pos.cell(), Cell::new(16, 17));
        assert_eq!(sim.radius_of(c) / 4 + 1, 6);
        (sim, c, b, s)
    }

    fn flag_cell(sim: &mut Sim, x: i32, y: i32, flags: u16) {
        let c = Cell::new(x, y);
        let mut d = sim.world.cell_data(c);
        d.flags = flags;
        sim.world.set_cell_data(c, d);
    }

    fn flag_all(sim: &mut Sim, flags: u16) {
        for y in 0..sim.world.height() {
            for x in 0..sim.world.width() {
                flag_cell(sim, x, y, flags);
            }
        }
    }

    #[test]
    fn the_ring_search_takes_the_first_cell_of_the_ring_past_inner_in_walk_order() {
        let (mut sim, c, b, s) = sim_with_army();
        assert!(sim.find_muster_spot(1, s, Obj::Building(b), true));
        let a = &sim.armies[1].list[s];
        assert_eq!(a.muster, Cell::new(9, 14), "ring 7's first entry, (−7, −3)");
        assert_eq!(
            a.muster_angle,
            find_angle(-7, -3),
            "the angle is from the army's point"
        );
        assert!(!sim.cities[c].no_muster);
        // The same cell whichever neighbourhood the flag walks: nothing is
        // out of bounds or foreign.
        assert!(sim.find_muster_spot(1, s, Obj::Building(b), false));
        assert_eq!(sim.armies[1].list[s].muster, Cell::new(9, 14));
    }

    #[test]
    fn a_forest_coast_or_mountain_cell_in_the_3x3_puts_a_candidate_out() {
        let (mut sim, _, b, s) = sim_with_army();
        // Ring 7 opens (−7, −3), (−7, −2), (−7, −1) … (−7, 3), then (−6,
        // −4), (−6, 4), … — cells (9, 14) to (9, 20), then (10, 13), (10, 21).
        // A forest on (9, 14) is in the 3 × 3 of (9, 15) too.
        flag_cell(&mut sim, 9, 14, cell::FOREST);
        assert!(sim.find_muster_spot(1, s, Obj::Building(b), true));
        assert_eq!(sim.armies[1].list[s].muster, Cell::new(9, 16));
        flag_cell(&mut sim, 10, 16, cell::COAST);
        assert!(sim.find_muster_spot(1, s, Obj::Building(b), true));
        assert_eq!(sim.armies[1].list[s].muster, Cell::new(9, 18));
        flag_cell(&mut sim, 8, 17, cell::MOUNTAIN);
        assert!(sim.find_muster_spot(1, s, Obj::Building(b), true));
        assert_eq!(sim.armies[1].list[s].muster, Cell::new(9, 19));
        // A cell two away is walked for bounds and owners only, not class:
        // the 7 × 7 walk does not read it.
        flag_cell(&mut sim, 11, 19, cell::FOREST);
        assert!(sim.find_muster_spot(1, s, Obj::Building(b), true));
        assert_eq!(sim.armies[1].list[s].muster, Cell::new(9, 19));
        // Rock and road are admissible.
        flag_cell(&mut sim, 9, 19, cell::ROCK | cell::ROAD);
        assert!(sim.find_muster_spot(1, s, Obj::Building(b), true));
        assert_eq!(sim.armies[1].list[s].muster, Cell::new(9, 19));
        // A building's cell in the 3 × 3 scores nothing: (9, 19) and (9,
        // 20) score eight neighbours, (10, 13) is out on the forest, and
        // (10, 21) is the first to score nine.
        flag_cell(&mut sim, 9, 19, cell::BUILDING);
        assert!(sim.find_muster_spot(1, s, Obj::Building(b), true));
        assert_eq!(sim.armies[1].list[s].muster, Cell::new(10, 21));
    }

    #[test]
    fn another_leader_s_cell_two_steps_out_puts_a_candidate_out_and_the_flag_widens_the_walk() {
        let (mut sim, _, b, s) = sim_with_army();
        // The double of (9, 14)'s north-west neighbour.
        sim.world
            .set_owner(Cell::new(7, 12), Owner::Player(0), Owner::None);
        assert!(sim.find_muster_spot(1, s, Obj::Building(b), false));
        assert_eq!(sim.armies[1].list[s].muster, Cell::new(9, 15));
        assert!(sim.find_muster_spot(1, s, Obj::Building(b), true));
        assert_eq!(sim.armies[1].list[s].muster, Cell::new(9, 15));
        sim.world
            .set_owner(Cell::new(7, 12), Owner::None, Owner::None);
        // The double of (9, 14)'s (3, 0) entry — six cells east — is walked
        // only with the flag.
        sim.world
            .set_owner(Cell::new(15, 14), Owner::Player(0), Owner::None);
        assert!(sim.find_muster_spot(1, s, Obj::Building(b), false));
        assert_eq!(sim.armies[1].list[s].muster, Cell::new(9, 14));
        assert!(sim.find_muster_spot(1, s, Obj::Building(b), true));
        assert_eq!(sim.armies[1].list[s].muster, Cell::new(9, 15));
        // My own cell, or an ally's, is not foreign.
        sim.world
            .set_owner(Cell::new(15, 14), Owner::Player(1), Owner::None);
        assert!(sim.find_muster_spot(1, s, Obj::Building(b), true));
        assert_eq!(sim.armies[1].list[s].muster, Cell::new(9, 14));
    }

    #[test]
    fn a_candidate_within_four_cells_of_another_army_of_mine_is_dropped() {
        let (mut sim, c, b, s) = sim_with_army();
        let s2 = sim.init_army(1, Some(c));
        sim.armies[1].list[s2].muster = Cell::new(9, 14);
        assert!(sim.find_muster_spot(1, s, Obj::Building(b), true));
        assert_eq!(
            sim.armies[1].list[s].muster,
            Cell::new(9, 18),
            "(9, 15), (9, 16) and (9, 17) are within four; (9, 18) is at four"
        );
        // An invalid slot does not count.
        sim.close_army(1, s2);
        assert!(sim.find_muster_spot(1, s, Obj::Building(b), true));
        assert_eq!(sim.armies[1].list[s].muster, Cell::new(9, 14));
    }

    /// The return is the last in-bounds candidate's verdict, not whether a
    /// best was found (listing `6f6824`). Ring 7 and ring 8 hold 92 entries
    /// and the early stop fires 0x28 past the best, so the best has to sit
    /// near the end of ring 8 for the two to differ: with everything forest
    /// but one 3 × 3 patch, the patch is the best, and the return says
    /// whether the *last* entry, (8, 3), was admissible.
    #[test]
    fn the_return_is_the_last_candidate_s_verdict_not_the_best_s() {
        let (mut sim, c, b, s) = sim_with_army();
        flag_all(&mut sim, cell::FOREST);
        for y in 19..=21 {
            for x in 23..=25 {
                flag_cell(&mut sim, x, y, 0);
            }
        }
        assert!(sim.find_muster_spot(1, s, Obj::Building(b), true));
        assert_eq!(
            sim.armies[1].list[s].muster,
            Cell::new(24, 20),
            "(8, 3), the last entry"
        );
        assert!(!sim.cities[c].no_muster);

        let (mut sim, c, b, s) = sim_with_army();
        flag_all(&mut sim, cell::FOREST);
        for y in 18..=20 {
            for x in 23..=25 {
                flag_cell(&mut sim, x, y, 0);
            }
        }
        assert!(
            !sim.find_muster_spot(1, s, Obj::Building(b), true),
            "the last entry is out, so the search reports failure"
        );
        assert_eq!(
            sim.armies[1].list[s].muster,
            Cell::new(24, 19),
            "with the best, (8, 2), set as the muster all the same"
        );
        assert!(!sim.cities[c].no_muster, "the mark is the no-best path's");
    }

    #[test]
    fn no_cell_at_an_untroubled_city_of_my_own_sets_the_mark_and_a_find_clears_it() {
        let (mut sim, c, b, s) = sim_with_army();
        flag_all(&mut sim, cell::FOREST);
        let before = sim.armies[1].list[s].muster;
        assert!(!sim.find_muster_spot(1, s, Obj::Building(b), true));
        assert!(sim.cities[c].no_muster);
        assert_eq!(
            sim.armies[1].list[s].muster, before,
            "the muster is left alone"
        );
        // Under attack, no mark.
        sim.cities[c].no_muster = false;
        sim.cities[c].no_heal = true;
        assert!(!sim.find_muster_spot(1, s, Obj::Building(b), true));
        assert!(!sim.cities[c].no_muster);
        sim.cities[c].no_heal = false;
        sim.cities[c].no_muster = true;
        flag_all(&mut sim, 0);
        assert!(sim.find_muster_spot(1, s, Obj::Building(b), true));
        assert!(!sim.cities[c].no_muster);
    }

    #[test]
    fn no_cell_at_an_enemy_city_falls_back_to_the_city_s_cell_one_row_down() {
        let (mut sim, c, b, s) = sim_with_army();
        sim.cities[c].owner = 0;
        sim.buildings[b].owner = 0;
        flag_all(&mut sim, cell::FOREST);
        // Not mine: `inner` halves to 3 without siege, and the rings are
        // 4 and 5 — all forest.
        assert!(sim.find_muster_spot(1, s, Obj::Building(b), true));
        assert_eq!(sim.armies[1].list[s].muster, Cell::new(16, 17));
        assert_eq!(sim.armies[1].list[s].muster_angle, find_angle(0, 1));
        assert!(!sim.cities[c].no_muster);
    }

    #[test]
    fn a_navy_musters_on_water_of_its_region_scoring_the_5x5() {
        let (mut sim, c, b, s) = sim_with_army();
        sim.cities[c].owner = 0;
        sim.buildings[b].owner = 0;
        let sea = sim.world.add_region(Terrain::Sea);
        for y in 0..60 {
            for x in 0..60 {
                let k = Cell::new(x, y);
                sim.world.set_region(k, sea);
                sim.world.set_cell_data(
                    k,
                    CellData {
                        land: 2,
                        ..CellData::default()
                    },
                );
            }
        }
        {
            let a = &mut sim.armies[1].list[s];
            a.navy = true;
            a.reg = Some(sea);
        }
        // `inner = 5`, `outer = 13`: ring 6 opens (−6, −3) … (−6, 3), the
        // cells (10, 14) to (10, 20).
        assert!(sim.find_muster_spot(1, s, Obj::Building(b), false));
        assert_eq!(sim.armies[1].list[s].muster, Cell::new(10, 14));
        // A land cell puts the candidate out — a coastal cell (`HALFLAND`)
        // is land to `is_ocean`. Without the flag only the 3 × 3 is
        // walked, so (11, 15) puts out (10, 14) to (10, 16).
        flag_cell(&mut sim, 11, 15, cell::HALFLAND);
        assert!(sim.find_muster_spot(1, s, Obj::Building(b), false));
        assert_eq!(sim.armies[1].list[s].muster, Cell::new(10, 17));
        // With it the 5 × 5 is, and (10, 17) is out as well.
        assert!(sim.find_muster_spot(1, s, Obj::Building(b), true));
        assert_eq!(sim.armies[1].list[s].muster, Cell::new(10, 18));
        // Another army's spacing is two cells for an enemy target.
        let s2 = sim.init_army(1, Some(c));
        sim.armies[1].list[s2].muster = Cell::new(10, 17);
        assert!(sim.find_muster_spot(1, s, Obj::Building(b), false));
        assert_eq!(sim.armies[1].list[s].muster, Cell::new(10, 19));
    }

    // ---- gaia is outside the object searches (`world::PLAYER_SLOTS`) ----

    /// A damaged friendly building with an animal standing beside it. The
    /// muster search asks `find_unit(SEARCH_ENEMY)` about everything on the
    /// map, and the diplomacy table a two-player lobby builds is two wide —
    /// which is how the first fuzzed seed (424242) took `Sim::is_enemy`
    /// down with `who 8`. The accessors are total now, so it merely
    /// answers.
    #[test]
    fn the_muster_search_survives_a_gaia_unit_with_a_two_wide_table() {
        let (mut sim, c) = sim_with_city();
        let t = soldier_type(&mut sim);
        assert_eq!(sim.at_war.len(), 2, "the lobby's table, not a wide one");
        let b = sim.cities[c].building;
        sim.buildings[b].hits = 100;
        sim.buildings[b].health = 50;
        let slot = sim.init_army(1, Some(c));
        let sheep = put(
            &mut sim,
            crate::world::PLAYER_SLOTS,
            t,
            Pos::new(0x3100, 0x3000),
        );
        assert!(sim.units[sheep].is_gaia());
        // No panic, and the animal is not an attacker to hurry towards.
        sim.find_muster_spot(1, slot, Obj::Building(b), false);
        assert_eq!(sim.armies[1].list[slot].hurry, 0);
    }

    /// And it is the leader bound that refuses it, not the table's length:
    /// widen the table past the eight the original has, declare war on
    /// gaia, and the animal is still not the muster point.
    #[test]
    fn a_gaia_unit_is_never_the_muster_point_even_at_war() {
        let slots = usize::from(crate::world::PLAYER_SLOTS) + 2;
        let (mut sim, c) = sim_with_city_slots(slots);
        let t = soldier_type(&mut sim);
        let b = sim.cities[c].building;
        sim.buildings[b].hits = 100;
        sim.buildings[b].health = 50;
        let slot = sim.init_army(1, Some(c));
        let g = usize::from(crate::world::PLAYER_SLOTS);
        sim.at_war[1][g] = true;
        sim.at_war[g][1] = true;
        let sheep = put(
            &mut sim,
            crate::world::PLAYER_SLOTS,
            t,
            Pos::new(0x3100, 0x3000),
        );
        assert!(
            sim.is_enemy(1, crate::world::PLAYER_SLOTS),
            "the table says enemy"
        );
        let sheep_cell = sim.units[sheep].pos.cell();
        sim.find_muster_spot(1, slot, Obj::Building(b), false);
        assert_eq!(sim.armies[1].list[slot].hurry, 0);
        assert_ne!(sim.armies[1].list[slot].muster, sheep_cell);
        // A player's soldier in the same spot *is* the muster point, so the
        // search is otherwise working.
        let foe = put(&mut sim, 0, t, Pos::new(0x3100, 0x3000));
        sim.at_war[1][0] = true;
        sim.at_war[0][1] = true;
        assert!(sim.find_muster_spot(1, slot, Obj::Building(b), false));
        assert_eq!(sim.armies[1].list[slot].hurry, 1);
        assert_eq!(sim.armies[1].list[slot].muster, sim.units[foe].pos.cell());
    }
}
