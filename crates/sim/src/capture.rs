//! **`Build::check_capture@006276a0`** — the capture test, step for step
//! (item 3356; `docs/CITIES.md` §7.2).
//!
//! The function is one tall conditional: a city building at zero asks
//! whether the attackers in its radius outweigh the defenders, and if they
//! do, hands the city over. [`check_capture_steps`] is that, over a
//! [`CaptureView`] that names every access it makes, in the order it makes
//! them: [`SimCapture`] answers from this crate's state, and the sweep
//! (`crates/sim/src/sweep/build_check_capture.rs`) answers from the
//! emulated original's windows.
//!
//! **The walk is the original's.** The tally does not scan the object lists:
//! it walks `circle_x`/`circle_y` out to `circle_radius[ring]` round the
//! building's cell and follows each cell's chain (`WorldData::get_down`,
//! then `+0x2c`/`+0x2e` of each object) — so an object is counted only when
//! its *cell* lies in the octagon, whatever its distance (`docs/CITIES.md`
//! §7.2). [`SimCapture`] builds the chains for the objects near the building
//! once a call; the crate keeps no chain for buildings.
//!
//! **Not established / SEAM.** The crate has no counterpart of a building's
//! `flags & 0x20`, so [`SimCapture::centre`] answers yes for every city and
//! the village arm (`swap_team`, the console's sound and message, the
//! hand-over notice) is never entered here; its methods are inert.

use std::collections::HashMap;

use crate::combat::Obj;
use crate::world::{Cell, Pos, vector_dist};
use crate::{Player, Sim};

/// Everything `Build::check_capture` reads or does, one method an access,
/// in the order the function makes them (a method that is a stubbed call
/// in the sweep logs it there). `H` names a chain element: the sweep's
/// `(list, index)` pair, the `Sim`'s candidate number.
pub trait CaptureView {
    type H: Copy + PartialEq;
    /// `BuildData::check_capture_eligible@0062d1d0`.
    fn eligible(&mut self) -> bool;
    /// The building's `+0x28` short: `inside_down`, negative when empty.
    fn inside_down(&mut self) -> i32;
    /// `objects[who][o]`'s vslot `+0xac` record, `build_masks |= 0x4000`.
    fn eject_pending(&mut self);
    /// The byte at `+0x9`.
    fn owner(&mut self) -> i32;
    /// The building itself, as a chain element would name it.
    fn building(&mut self) -> Self::H;
    /// `flags & 0x20`: a city centre, not a village.
    fn centre(&mut self) -> bool;
    /// The building's `x`/`y` (`+0x10`/`+0x14` ^ `0x63637`).
    fn building_pos(&mut self) -> (i32, i32);
    /// `div_3_table[i]`.
    fn div3(&mut self, i: i32) -> i32;
    /// `leaders[who]`'s `+0x8`.
    fn leader_id(&mut self, who: i32) -> i32;
    /// `leaders[who]`'s flags word: bit 0 in use, bit 1 alive.
    fn leader_flags(&mut self, who: i32) -> i32;
    /// `leaders[who]`'s `+0x74 + 4 * other`.
    fn relation(&mut self, who: i32, other: i32) -> i32;
    /// `leadersc[who]`'s `+0x74 + 4 * leader`.
    fn relation_const(&mut self, who: i32, leader: i32) -> i32;
    /// `LeaderData::is_ally@006edb50` on `leaders[who]`.
    fn is_ally(&mut self, who: i32, other: i32) -> bool;
    /// `LeaderData::is_enemy@006ebaa0` on `leaders[who]`.
    fn is_enemy(&mut self, who: i32, other: i32) -> bool;
    /// `objects[who][idx]`.
    fn handle(&mut self, who: i32, idx: i32) -> Self::H;
    /// The object's type's `+0x218`: 0 land, 1 and 2 not.
    fn kind(&mut self, h: Self::H) -> i32;
    /// Vslot `+0x18`: a unit.
    fn is_unit(&mut self, h: Self::H) -> bool;
    /// `units[who][idx]`'s `unit_masks & 1`: a decoy.
    fn decoy(&mut self, h: Self::H) -> bool;
    /// Vslot `+0x120`.
    fn can_capture(&mut self, h: Self::H) -> bool;
    /// The type's `unit_flags & 0x8000000`: a missile.
    fn missile(&mut self, h: Self::H) -> bool;
    /// The object's `flags & 1`.
    fn in_use(&mut self, h: Self::H) -> bool;
    /// The list the element lives in: its owner.
    fn player(&mut self, h: Self::H) -> i32;
    /// `Search::valid_filter@0067dbb0` on the element, filter 8.
    fn valid_filter(&mut self, h: Self::H) -> bool;
    /// The element's `x`/`y`.
    fn pos(&mut self, h: Self::H) -> (i32, i32);
    /// Vslot `+0xec`: what it counts for in a capture.
    fn value(&mut self, h: Self::H) -> i32;
    /// Vslot `+0x20`: a building.
    fn is_building(&mut self, h: Self::H) -> bool;
    /// A defending building's extra weight: vslot `+0xac` record's type's
    /// vslot `+0xfc` (a fort), and `num_inside(1)` of the record.
    fn garrison_weight(&mut self, h: Self::H) -> i32;
    /// The building's `visible` bit for `who` raised, and `update_local_seen`
    /// asked, when the bit was not already up.
    fn reveal(&mut self, h: Self::H, who: i32);
    /// The city's `+0x20` stamp.
    fn city_stamp(&mut self) -> i32;
    /// The city's `+0x28` strength.
    fn city_strength(&mut self) -> i32;
    /// The building's `+0x72` short: its city's number in its owner's list.
    fn city_index(&mut self) -> i32;
    /// The city's `+0x60` byte.
    fn city_founder(&mut self) -> i32;
    /// The city's `+0x5f` byte.
    fn city_race(&mut self) -> i32;
    /// `game->frame`.
    fn frame(&mut self) -> i32;
    /// `constants->city_capture_radius`.
    fn city_capture_radius(&mut self) -> i32;
    /// `constants->unit_respond_range`.
    fn unit_respond_range(&mut self) -> i32;
    /// `WorldData::valid(&x, &y)`.
    fn valid_tile(&mut self, x: i32, y: i32) -> bool;
    /// `WorldData::get_down`: its answer, and the head of the cell's chain
    /// it wrote (`None` for a negative index).
    fn get_down(&mut self, x: i32, y: i32) -> (bool, Option<Self::H>);
    /// The land id at the tile (`+0x134`, `+4`, a short).
    fn tile_region(&mut self, x: i32, y: i32) -> u16;
    /// The element's `+0x2c`/`+0x2e` link, `None` for a negative index.
    fn next(&mut self, h: Self::H) -> Option<Self::H>;
    /// `Cities::capture_city@00733380(taker, city, owner)`; `ecx` is the
    /// register the caller last left (a leaders row, or the city number).
    /// Answers the new city's number.
    fn capture_city(&mut self, taker: i32, city: i32, owner: i32, ecx: i32) -> i32;
    /// `cities[taker][city].capture_strength = v`.
    fn set_city_strength(&mut self, taker: i32, city: i32, v: i32);
    /// The village arm: vslot `+0x94`, `swap_team(who)`; the new object's
    /// number, negative when the swap failed.
    fn swap_team(&mut self, who: i32) -> i32;
    /// A failed swap: `Object::die(0, -1, 0.0)` (`Build`'s own, or the
    /// vtable's `+0x158`).
    fn die(&mut self);
    /// `MiscAccess::console->who`.
    fn console_who(&mut self) -> i32;
    /// `SoundGlobal::play(console, sound)`.
    fn play(&mut self, sound: i32);
    /// The captured notice: the new object's name through the message
    /// window.
    fn announce(&mut self, new: i32, who: i32);
    /// The new object's data record's vslot `+0x1a8(0, 1, 0)`.
    fn activate_new(&mut self, new: i32, who: i32);
    /// The building's own vslot `+0x150(0, -1, 0)`.
    fn close_old(&mut self);
    /// The new object's data record's `mask_me(1, 2)`.
    fn mask_new(&mut self, new: i32, who: i32);
}

/// `Build::check_capture@006276a0(this, idx, who)`: the captor is
/// `objects[who][idx]`. Answers the executable's `eax`: 1 when ownership
/// changed.
pub fn check_capture_steps<V: CaptureView>(v: &mut V, idx: i32, p2: i32) -> i32 {
    if !v.eligible() {
        return 0;
    }
    if v.inside_down() >= 0 {
        v.eject_pending();
        return 0;
    }
    let who = v.owner();
    // The captor is not the owner's own leader, or the two are not allied
    // both ways, or the owner is out; and the captor is not out.
    let lid = v.leader_id(who);
    let apart = p2 != lid && (v.relation(who, p2) != 2 || v.relation_const(p2, lid) != 2);
    if !(apart || v.leader_flags(who) & 2 == 0) {
        return 0;
    }
    if v.leader_flags(p2) & 2 == 0 {
        return 0;
    }
    let captor = v.handle(p2, idx);
    if v.kind(captor) != 0 || !v.is_unit(captor) || v.decoy(captor) || !v.can_capture(captor) {
        return 0;
    }
    if v.missile(captor) {
        return 0;
    }
    let stamp = v.city_stamp();
    let age = v.frame().wrapping_sub(stamp);
    if age < 0x4b {
        return 0;
    }
    let centre = v.centre();
    let radius = if centre {
        v.city_capture_radius()
    } else {
        v.unit_respond_range()
    };
    let lim = radius.wrapping_mul(0xc0);
    let ring = lim.wrapping_add(0x2ff) / 0x300;
    let (bx, by) = v.building_pos();
    let (tx, ty) = (v.div3(bx >> 8), v.div3(by >> 8));
    let region = v.tile_region(tx, ty);
    let def_base = if age < 900 { v.city_strength() } else { 2 };
    let mine = if v.kind(captor) == 0 {
        v.value(captor)
    } else {
        0
    };
    let mut tally = [0i32; 8];
    tally[p2 as usize] = mine;
    tally[who as usize] = def_base;
    let (mut attackers, mut defenders) = (mine, def_base);
    let me = v.building();
    let circle = crate::ai_place::circle();
    let ring_len = usize::try_from(ring)
        .ok()
        .and_then(|r| circle.radius.get(r).copied())
        .unwrap_or(0);
    for i in 0..ring_len {
        let (x, y) = (tx.wrapping_add(circle.x[i]), ty.wrapping_add(circle.y[i]));
        if !v.valid_tile(x, y) {
            continue;
        }
        let (down, head) = v.get_down(x, y);
        if !down || v.tile_region(x, y) != region {
            continue;
        }
        let mut cur = head;
        while let Some(h) = cur {
            cur = v.next(h);
            if h == me || h == captor {
                continue;
            }
            if !v.in_use(h) {
                continue;
            }
            let k = v.kind(h);
            if k == 2 || k == 1 {
                continue;
            }
            if v.is_unit(h) && v.decoy(h) {
                continue;
            }
            if !v.valid_filter(h) {
                continue;
            }
            let (ox, oy) = v.pos(h);
            let d = vector_dist(
                ox.wrapping_sub(bx).wrapping_abs(),
                oy.wrapping_sub(by).wrapping_abs(),
            );
            if d > lim {
                continue;
            }
            let mut val = v.value(h);
            let pl = v.player(h);
            if val != 0 && v.is_building(h) && pl == who {
                val = val.wrapping_add(v.garrison_weight(h));
                v.reveal(h, p2);
            }
            tally[pl as usize] = tally[pl as usize].wrapping_add(val);
            if pl == p2 || (v.is_ally(p2, pl) && v.is_enemy(who, pl)) {
                attackers = attackers.wrapping_add(val);
            } else if pl == who || (v.is_ally(who, pl) && v.is_enemy(p2, pl)) {
                defenders = defenders.wrapping_add(val);
            }
        }
    }
    if defenders >= attackers {
        return 0;
    }
    if centre {
        // The captor: the allied leader with the most in the radius.
        let (mut best, mut best_v) = (p2, 0);
        for k in 0..8 {
            if v.leader_flags(k) & 3 == 3
                && (k == p2 || v.is_ally(k, p2))
                && tally[k as usize] > best_v
            {
                best = k;
                best_v = tally[k as usize];
            }
        }
        let city = v.city_index();
        let founder = v.city_founder();
        let race = v.city_race();
        let mut taker = best;
        let mut ecx = city;
        let mut founded = false;
        if founder != who && founder != taker {
            ecx = 0x00e3_a390_i32.wrapping_add(taker.wrapping_mul(0x6eec));
            if v.is_ally(taker, founder) && v.leader_flags(founder) & 2 != 0 {
                taker = founder;
                founded = true;
            }
        }
        if !founded && race != who && race != taker {
            ecx = 0x00e3_a390_i32.wrapping_add(taker.wrapping_mul(0x6eec));
            if v.is_ally(taker, race) && v.leader_flags(race) & 2 != 0 {
                taker = race;
            }
        }
        let new = v.capture_city(taker, city, who, ecx);
        v.set_city_strength(taker, new, mine);
        return 1;
    }
    // The village arm.
    let new = v.swap_team(p2);
    if new < 0 {
        v.die();
        return 0;
    }
    let console = v.console_who();
    if who == console {
        v.play(0x86);
    }
    let console = v.console_who();
    if p2 == console {
        v.play(0x7c);
    }
    let console = v.console_who();
    if who == console || p2 == console {
        v.announce(new, p2);
    }
    v.activate_new(new, p2);
    v.close_old();
    v.mask_new(new, p2);
    1
}

/// A chain element of [`SimCapture`]: an object near the building, with the
/// link to the next object in its cell.
#[derive(Clone, Copy)]
struct Cand {
    obj: Obj,
    owner: i32,
    pos: Pos,
    next: Option<usize>,
}

/// [`CaptureView`] over the live [`Sim`]: `b` the building, `unit` the
/// captor.
pub(crate) struct SimCapture<'a> {
    s: &'a mut Sim,
    b: usize,
    c: usize,
    cands: Vec<Cand>,
    heads: HashMap<(i32, i32), usize>,
}

impl<'a> SimCapture<'a> {
    pub(crate) fn new(s: &'a mut Sim, b: usize, unit: usize) -> Self {
        let bp = s.buildings[b].pos;
        let c = s.buildings[b].city.unwrap_or(usize::MAX);
        let mut cands = vec![
            Cand {
                obj: Obj::Building(b),
                owner: i32::from(s.buildings[b].owner),
                pos: bp,
                next: None,
            },
            Cand {
                obj: Obj::Unit(unit),
                owner: i32::from(s.units[unit].owner),
                pos: s.units[unit].pos,
                next: None,
            },
        ];
        // The cells the walk can reach: the octagon is inside the square of
        // the ring's radius round the building's cell.
        let radius = if c == usize::MAX {
            0
        } else {
            s.tuning.city_capture_radius
        };
        let reach = (radius.wrapping_mul(0xc0).wrapping_add(0x2ff) / 0x300).max(0);
        let home = bp.cell();
        let near = |p: Pos| {
            let q = p.cell();
            (q.x - home.x).abs() <= reach && (q.y - home.y).abs() <= reach
        };
        let mut chained: Vec<usize> = Vec::new();
        for (i, x) in s.units.iter().enumerate() {
            if !x.alive() || !x.on_map || !near(x.pos) {
                continue;
            }
            if i == unit {
                chained.push(1);
                continue;
            }
            cands.push(Cand {
                obj: Obj::Unit(i),
                owner: i32::from(x.owner),
                pos: x.pos,
                next: None,
            });
            chained.push(cands.len() - 1);
        }
        for (i, x) in s.buildings.iter().enumerate() {
            if !x.alive || !near(x.pos) {
                continue;
            }
            if i == b {
                chained.push(0);
                continue;
            }
            cands.push(Cand {
                obj: Obj::Building(i),
                owner: i32::from(x.owner),
                pos: x.pos,
                next: None,
            });
            chained.push(cands.len() - 1);
        }
        // Each cell's chain, in the order the objects were met.
        let mut heads: HashMap<(i32, i32), usize> = HashMap::new();
        let mut tails: HashMap<(i32, i32), usize> = HashMap::new();
        for h in chained {
            let q = cands[h].pos.cell();
            let key = (q.x, q.y);
            match tails.insert(key, h) {
                Some(t) => cands[t].next = Some(h),
                None => {
                    heads.insert(key, h);
                }
            }
        }
        SimCapture {
            s,
            b,
            c,
            cands,
            heads,
        }
    }
}

impl CaptureView for SimCapture<'_> {
    type H = usize;
    fn eligible(&mut self) -> bool {
        self.s.capture_eligible(self.b)
    }
    fn inside_down(&mut self) -> i32 {
        if self.s.buildings[self.b].garrison.is_empty() {
            -1
        } else {
            0
        }
    }
    fn eject_pending(&mut self) {
        self.s.buildings[self.b].eject_pending = true;
    }
    fn owner(&mut self) -> i32 {
        i32::from(self.s.buildings[self.b].owner)
    }
    fn building(&mut self) -> usize {
        0
    }
    /// SEAM: the record's `0x20` flag has no counterpart here; every city
    /// building is a centre.
    fn centre(&mut self) -> bool {
        true
    }
    fn building_pos(&mut self) -> (i32, i32) {
        let p = self.s.buildings[self.b].pos;
        (p.x, p.y)
    }
    fn div3(&mut self, i: i32) -> i32 {
        i.div_euclid(3)
    }
    fn leader_id(&mut self, who: i32) -> i32 {
        who
    }
    fn leader_flags(&mut self, who: i32) -> i32 {
        match usize::try_from(who) {
            Ok(w) if w < self.s.players.len() => {
                1 | if self.s.defeated.get(w).copied().unwrap_or(true) {
                    0
                } else {
                    2
                }
            }
            _ => 0,
        }
    }
    fn relation(&mut self, who: i32, other: i32) -> i32 {
        if self.s.allied_with(who as Player, other as Player) {
            2
        } else {
            0
        }
    }
    fn relation_const(&mut self, who: i32, leader: i32) -> i32 {
        if self.s.allied_with(who as Player, leader as Player) {
            2
        } else {
            0
        }
    }
    fn is_ally(&mut self, who: i32, other: i32) -> bool {
        match (u8::try_from(who), u8::try_from(other)) {
            (Ok(a), Ok(b)) => self.s.is_ally(a, b),
            _ => false,
        }
    }
    fn is_enemy(&mut self, who: i32, other: i32) -> bool {
        match (u8::try_from(who), u8::try_from(other)) {
            (Ok(a), Ok(b)) => self.s.is_enemy(a, b),
            _ => false,
        }
    }
    fn handle(&mut self, _who: i32, _idx: i32) -> usize {
        1
    }
    fn kind(&mut self, h: usize) -> i32 {
        match self.cands[h].obj {
            Obj::Unit(u) => i32::from(!matches!(
                self.s.units[u].kind.domain,
                crate::attrition::Domain::Land
            )),
            Obj::Building(_) => 0,
        }
    }
    fn is_unit(&mut self, h: usize) -> bool {
        matches!(self.cands[h].obj, Obj::Unit(_))
    }
    fn decoy(&mut self, h: usize) -> bool {
        match self.cands[h].obj {
            Obj::Unit(u) => self.s.units[u].decoy,
            Obj::Building(_) => false,
        }
    }
    fn can_capture(&mut self, h: usize) -> bool {
        self.s.attack_of(self.cands[h].obj) != 0
    }
    fn missile(&mut self, h: usize) -> bool {
        self.s
            .profile(self.cands[h].obj)
            .has(crate::combat::mask::MISSILE)
    }
    fn in_use(&mut self, h: usize) -> bool {
        match self.cands[h].obj {
            Obj::Unit(u) => self.s.units[u].alive() && self.s.units[u].on_map,
            Obj::Building(b) => self.s.buildings[b].alive,
        }
    }
    fn player(&mut self, h: usize) -> i32 {
        self.cands[h].owner
    }
    /// **`Search::valid_filter(…, 8)`: only what is armed counts**
    /// (item 1099, `docs/CITIES.md` §7.2). The tally's loop asks the
    /// filter of every object before its distance, and entry 8 of the
    /// filter's jump table (`0067de47`, read off the PE's own table at
    /// `0067e57c`) answers, for a unit, `type->attack != 0 && (type->role
    /// & 0x10000)` — an armed combat type (`0067debf`–`0067deeb`) — and,
    /// for a building, `type->attack != 0`, not a city (`flags & 0x20`),
    /// finished (vslot `+0x4c`, `flags & 4`, `00472350`) and with hits
    /// left (vslot `+0x114`, `ObjectData::hits_left@006535c0`). A
    /// citizen, a scout, a house or a farm counts for neither side, and
    /// the defender's own buildings weigh 7 only when they can shoot.
    /// run347's game ends on it: the human's city falls on 5930 with
    /// eighteen AI soldiers round it against the human's five unarmed
    /// buildings, which this crate counted 35. Gaia is out of the tally
    /// (`world::PLAYER_SLOTS`): the original's local array has eight
    /// slots and nothing in the loop bounds the leader.
    fn valid_filter(&mut self, h: usize) -> bool {
        match self.cands[h].obj {
            Obj::Unit(u) => {
                let x = &self.s.units[u];
                !x.is_gaia()
                    && x.ty.is_some_and(|t| {
                        self.s.unit_types[t].combat.attack != 0
                            && self.s.unit_types[t].combat.combat_role
                    })
            }
            Obj::Building(b) => {
                let x = &self.s.buildings[b];
                x.ty.is_some_and(|t| self.s.build_types[t].attack != 0)
                    && !self.s.building_is_city(b)
                    && x.active
                    && crate::city::health_level(x.hits_now(), x.damage) <= 5
            }
        }
    }
    fn pos(&mut self, h: usize) -> (i32, i32) {
        (self.cands[h].pos.x, self.cands[h].pos.y)
    }
    fn value(&mut self, h: usize) -> i32 {
        match self.cands[h].obj {
            Obj::Unit(u) => self.s.unit_capture_value(u),
            Obj::Building(_) => 1,
        }
    }
    fn is_building(&mut self, h: usize) -> bool {
        matches!(self.cands[h].obj, Obj::Building(_))
    }
    fn garrison_weight(&mut self, h: usize) -> i32 {
        let Obj::Building(b) = self.cands[h].obj else {
            return 0;
        };
        let x = &self.s.buildings[b];
        let fort =
            x.ty.is_some_and(|t| crate::build::is_fort(&self.s.build_types, t));
        x.garrison.len() as i32 + if fort { 12 } else { 6 }
    }
    fn reveal(&mut self, h: usize, who: i32) {
        let Obj::Building(b) = self.cands[h].obj else {
            return;
        };
        let bit = 1u8.checked_shl(who as u32).unwrap_or(0);
        if self.s.buildings[b].visible & bit == 0 {
            self.s.buildings[b].visible |= bit;
            self.s.update_local_seen_build(b);
        }
    }
    fn city_stamp(&mut self) -> i32 {
        self.s.cities[self.c].capture_stamp as i32
    }
    fn city_strength(&mut self) -> i32 {
        self.s.cities[self.c].capture_strength
    }
    fn city_index(&mut self) -> i32 {
        self.c as i32
    }
    fn city_founder(&mut self) -> i32 {
        i32::from(self.s.cities[self.c].founder)
    }
    fn city_race(&mut self) -> i32 {
        self.s.cities[self.c].race.map_or(-1, i32::from)
    }
    fn frame(&mut self) -> i32 {
        self.s.frame as i32
    }
    fn city_capture_radius(&mut self) -> i32 {
        self.s.tuning.city_capture_radius
    }
    fn unit_respond_range(&mut self) -> i32 {
        self.s.tuning.unit_respond_range
    }
    fn valid_tile(&mut self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.s.world.width() && y < self.s.world.height()
    }
    fn get_down(&mut self, x: i32, y: i32) -> (bool, Option<usize>) {
        let head = self.heads.get(&(x, y)).copied();
        (head.is_some(), head)
    }
    fn tile_region(&mut self, x: i32, y: i32) -> u16 {
        self.s.world.region_of(Cell::new(x, y)).unwrap_or(0xffff)
    }
    fn next(&mut self, h: usize) -> Option<usize> {
        self.cands[h].next
    }
    fn capture_city(&mut self, taker: i32, _city: i32, owner: i32, _ecx: i32) -> i32 {
        self.s
            .capture_city(taker as Player, self.c, owner as Player)
            .map_or(-1, |n| n as i32)
    }
    fn set_city_strength(&mut self, _taker: i32, city: i32, v: i32) {
        if let Ok(n) = usize::try_from(city) {
            self.s.cities[n].capture_strength = v;
        }
    }
    fn swap_team(&mut self, _who: i32) -> i32 {
        -1
    }
    fn die(&mut self) {}
    fn console_who(&mut self) -> i32 {
        -1
    }
    fn play(&mut self, _sound: i32) {}
    fn announce(&mut self, _new: i32, _who: i32) {}
    fn activate_new(&mut self, _new: i32, _who: i32) {}
    fn close_old(&mut self) {}
    fn mask_new(&mut self, _new: i32, _who: i32) {}
}

impl Sim {
    /// `Build::check_capture(B, o, who)` with an enemy unit as the would-be
    /// captor. Returns whether ownership changed. `docs/CITIES.md` §7.2;
    /// [`check_capture_steps`] over [`SimCapture`].
    pub fn check_capture(&mut self, b: usize, unit: usize) -> bool {
        // A dead captor is the callers' to refuse (the original's captor
        // is a live object by construction), and a city building with no
        // record behind it has nothing to capture.
        if !self.units[unit].alive() {
            return false;
        }
        if self.buildings[b].city.is_none() {
            if self.capture_eligible(b) && !self.buildings[b].garrison.is_empty() {
                self.buildings[b].eject_pending = true;
            }
            return false;
        }
        let who = i32::from(self.units[unit].owner);
        let mut v = SimCapture::new(self, b, unit);
        check_capture_steps(&mut v, unit as i32, who) != 0
    }

    /// `UnitData::get_capture_value` for one figure.
    fn unit_capture_value(&self, i: usize) -> i32 {
        let u = &self.units[i];
        crate::city::capture_value(u.kind.siege, u.squad_size, 1)
    }
}
