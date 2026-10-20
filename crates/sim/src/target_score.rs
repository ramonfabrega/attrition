//! `Army::find_target@006f69b0`, **the score and the tail** (item 3362, the
//! second slice, `sweep/army_find_target_run.rs`): from the draw to the
//! return. [`crate::target_gate`] is the first slice, entry to the draw.
//!
//! The score is one city's `v`, a chain of multipliers and divisions the
//! original writes in one block of `ebx`; the tail picks the best, hands
//! the army its target, marks the target's leader attacked, sends the
//! navy, moves the army's point and asks for the muster spot. Every read
//! and every callee is a [`TargetScoreView`] method named by the record it
//! reads, in the original's order, so the sweep can hold the order of the
//! calls as well as the result.
//!
//! **Not swept, and not modelled** (`army::seams`): the forts pass
//! (`6f7f01..6f81ed`: the sweep keeps the army's group count at zero, so
//! the original's loop over them never runs), the probe's body (the sweep
//! answers its first call `-1`, which returns), `leaders[who] +0x7e8`, the
//! two wonder arms and `has_preq(0x2b9)` (the sim answers all three off).

use crate::target_gate::{SimGate, TargetGateView, diff_of};
use crate::world::vector_dist;

/// What the head and the gates leave for the score (`target_gate::Head`
/// and the leader gate's `about_to_win`).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Ctx {
    /// The army is not weak.
    pub strong: bool,
    /// The capital's cell.
    pub cap: Option<(i32, i32)>,
    /// The two sea averages.
    pub ours: i32,
    pub theirs: i32,
    /// `count(NON_DECOY_TYPE, SUPPLYWAGON)`.
    pub supply: i32,
    /// 0 or 1: the second pass.
    pub pass: usize,
    /// The leader gate's `about_to_win` (`local_34`).
    pub about: bool,
}

/// The score's reads and calls, beyond the gate chain's.
pub(crate) trait TargetScoreView: TargetGateView {
    // ---- reads ----
    /// `leaders[a] +0x7e8`, `+0x18`.
    fn f7e8(&self, a: usize) -> i32;
    fn f18(&self, a: usize) -> i32;
    /// `(leaders[a].flags >> 8) & 7`, the transport level.
    fn transport(&self, a: usize) -> i32;
    /// `*(*0xc06188)`, the world's width.
    fn world_width(&self) -> i32;
    /// `cities[a][k] +8`, the id the objects table is indexed by.
    fn city_id(&self, a: usize, k: usize) -> i32;
    /// `cities[a][k] +4` as a word: `2` attacked, `0x10` capital, `0x2000` no muster spot.
    fn city_bits(&self, a: usize, k: usize) -> i32;
    /// `+0x65`, `+0x68`: the bordering and was-capital masks; `+0x14`, the attack stamp.
    fn city_bordering(&self, a: usize, k: usize) -> i32;
    fn city_was_capital(&self, a: usize, k: usize) -> i32;
    fn city_stamp(&self, a: usize, k: usize) -> i32;
    /// The city's cell, `div_3_table[pos >> 8]` on each axis.
    fn city_cell(&self, a: usize, k: usize) -> (i32, i32);
    /// `armies[who].list[j]`: `*short != 0` and `+4 & 1 == 0`, and its target pair
    /// (`+0x30`, `+0x34`); `None` for an army the loop skips.
    fn army_open(&self, j: usize) -> Option<(i32, i32)>;

    // ---- calls ----
    /// `objects[type][id]->vtable[0x3c]()` — the `BuildData` — and its
    /// `+0x18 -> +4`, the kind (`0x19f` town, `0x1a0`/`0x213` metropolis).
    fn city_kind(&mut self, a: usize, k: usize) -> i32;
    /// The same slot, then `BuildData::is_unassimilated@0062d470`.
    fn unassimilated(&mut self, a: usize, k: usize) -> bool;
    /// `CityData::get_level@00739340`.
    fn city_level(&mut self, a: usize, k: usize) -> i32;
    /// `CityData::num_wonders@007382b0(0)`.
    fn num_wonders(&mut self, a: usize, k: usize) -> i32;
    /// `Game::wonder_winning@005948a0`.
    fn wonder_winning(&mut self) -> i32;
    /// `LeaderData::has_preq@006db810(0x2b9)` of the army's owner.
    fn has_preq(&mut self) -> bool;
    /// The building object's `vtable[0xac]()` and its `+0x24`: the damage.
    fn building_damage(&mut self, a: usize, k: usize) -> i32;
    /// The building's `vtable[0xac]()`, then that record's `vtable[0x11c](0)`: the hits.
    fn building_hits(&mut self, a: usize, k: usize) -> i32;
}

fn div(a: i32, b: i32) -> i32 {
    a.wrapping_div(b)
}

/// The pieces of the war test the score repeats.
fn at_war<V: TargetGateView>(v: &V, who: usize, i: usize) -> bool {
    let wid = v.lid(who);
    !(v.ld(who, i) != 0 && v.lc(i, wid) != 0)
}

fn allied<V: TargetGateView>(v: &V, who: usize, i: usize) -> bool {
    let wid = v.lid(who);
    v.ld(who, i) == 2 && v.lc(i, wid) == 2
}

/// The kind's factor, `TOWN → 2, METROPOLIS / FORBIDDENCITY → 3, else 1`.
fn kind_size(kind: i32) -> i32 {
    match kind {
        0x19f => 2,
        0x1a0 | 0x213 => 3,
        _ => 1,
    }
}

/// One city's score (`6f7185..6f7ea9`), `None` for a city the original
/// skips outright. `draw` is the `Random::get(0, 0xffff)` the caller has
/// just spent.
pub(crate) fn target_score<V: TargetScoreView>(
    v: &mut V,
    c: &Ctx,
    i: usize,
    k: usize,
    draw: i32,
) -> Option<i32> {
    let who = v.who();
    let wid = v.lid(who);
    let ii = i as i32;
    let mut s = (draw % 200).wrapping_add(900);

    // The capital's distance, against an enemy.
    if let Some((cx, cy)) = c.cap
        && ii != wid
        && at_war(v, who, i)
    {
        if !c.strong {
            if (v.city_founder(i, k) as i8 as i16 as u16) != who as u16 {
                return None;
            }
            if !v.unassimilated(i, k) {
                return None;
            }
        }
        let (px, py) = v.city_cell(i, k);
        let dy = (py - cy).abs();
        let dx = (px - cx).abs();
        let d = vector_dist(dx, dy);
        s = s.wrapping_add(div(d.wrapping_mul(-50), v.world_width()));
    }
    // `leaders[who] +0x7e8`: an enemy's city scores four times.
    if v.f7e8(who) != 0 && ii != wid && at_war(v, who, i) {
        s = s.wrapping_shl(2);
    }
    // A leader about to win by wonders.
    if v.w44c(i) != 0 && v.num_wonders(i, k) != 0 && v.wonder_winning() == ii {
        s = s.wrapping_mul(10);
    }

    let bits = v.city_bits(i, k);
    let navy = v.navy();
    let captains = v.captains();
    if ii == wid || !at_war(v, who, i) {
        // Me, or at peace (6f75f4): only an ally is scored.
        if ii != wid && !allied(v, who, i) {
            // 6f73c7
        } else if bits & 2 == 0 {
            // An untroubled city of mine or an ally's.
            let mut done = false;
            if captains >= 20 {
                let siege = v.count(4, 0);
                if siege >= 2 {
                    let lvl = v.city_level(i, k);
                    let t = (4 - lvl).wrapping_mul(s);
                    s = (t + ((t >> 31) & 3)) >> 2;
                    done = true;
                }
            }
            if !done {
                let mut skip = false;
                if captains >= 10 {
                    skip = v.count(4, 0) != 0;
                }
                if !skip {
                    s = s.wrapping_mul(kind_size(v.city_kind(i, k)));
                }
            }
            if bits & 0x2000 != 0 {
                s = div(s, 20);
            }
        } else if v.team_style() == 2 {
            s = s.wrapping_mul(5);
        }
    } else {
        // At war (6f735a).
        let ts = v.team_style();
        if ts == 2 || ts == 3 {
            s = s.wrapping_mul(5);
        }
        if navy {
            s = s.wrapping_mul(10);
        }
        let raid = v.raid(who);
        if raid == -1 {
            s = s.wrapping_mul(kind_size(v.city_kind(i, k)));
        } else {
            #[derive(PartialEq)]
            enum Mult {
                Level,
                Keep,
                Default,
            }
            let m = if raid == 1 {
                if captains >= 20
                    && v.count(4, 0) >= 3
                    && (v.f18(i).wrapping_shl(2)) / 3 < v.f18(who)
                {
                    Mult::Level
                } else {
                    Mult::Default
                }
            } else {
                let mut m = None;
                if captains >= 20 && v.count(4, 0) >= 2 {
                    m = Some(Mult::Level);
                }
                match m {
                    Some(m) => m,
                    None => {
                        if captains >= 10 && v.count(4, 0) != 0 {
                            Mult::Keep
                        } else {
                            Mult::Default
                        }
                    }
                }
            };
            match m {
                Mult::Level => s = s.wrapping_mul(v.city_level(i, k)),
                Mult::Keep => {}
                Mult::Default => {
                    s = s.wrapping_mul(4 - kind_size(v.city_kind(i, k)));
                }
            }
        }
    }

    // Another region, on land.
    if v.city_reg(i, k) != v.region() && !navy {
        s = div(s, 2);
        if v.transport(who) == 0 {
            return None;
        }
        let (ours, theirs) = (c.ours, c.theirs);
        if ours < theirs || ours < 5 {
            s = div(s, 2);
            if ours < theirs - 2 || ours == 0 {
                s = div(s, 5);
            }
        }
    }

    // My own city, or an ally's, that nobody attacks: a third.
    let diff = diff_of(v, who);
    if (c.pass == 0 || diff >= 3) && (ii == wid || allied(v, who, i)) && bits & 2 == 0 {
        s = div(s, 3);
    }
    if (ii == wid || allied(v, who, i)) && bits & 2 != 0 {
        if ii == who as i32 {
            s = s.wrapping_mul(10);
        }
        if bits & 0x10 != 0 {
            s = s.wrapping_mul(10);
        }
    } else if ii != wid && at_war(v, who, i) && bits & 0x10 != 0 {
        if v.has_preq() {
            s = s.wrapping_shl(2);
        } else if v.raid(who) < 0 {
            s = div(s.wrapping_mul(3), 2);
        }
    }

    // The city's own `bordering` mask.
    if v.city_bordering(i, k) & (1u32.wrapping_shl(who as u32) as i32) != 0 {
        let base = (if v.raid(who) <= 0 { 4i32 } else { 2 }).wrapping_mul(s);
        s = base;
        if ii != wid && at_war(v, who, i) && diff >= 3 {
            let frame = v.frame();
            if v.frame_attacked(i).wrapping_add(0x2328) < frame {
                s = base.wrapping_mul(3);
            }
            if v.city_stamp(i, k).wrapping_add(0x2328) < frame {
                s = s.wrapping_mul(3);
            }
        }
    }

    // An ally's city is worth three quarters, half again if untroubled.
    if ii != who as i32 && (ii == wid || allied(v, who, i)) {
        let t = s.wrapping_mul(3);
        s = (t + ((t >> 31) & 3)) >> 2;
        if bits & 2 == 0 {
            s = div(s, 2);
        }
    }
    if c.about {
        s = s.wrapping_mul(100);
    }

    // Damage: mine twice, another's five quarters (three halves, bordering).
    if ii == who as i32 {
        if v.building_damage(i, k) != 0 {
            s = s.wrapping_add(s);
        }
    } else if v.building_damage(i, k) != 0 {
        if v.city_bordering(i, k) & (1u32.wrapping_shl(who as u32) as i32) == 0 {
            let t = s.wrapping_mul(5);
            s = (t + ((t >> 31) & 3)) >> 2;
        } else {
            s = div(s.wrapping_mul(3), 2);
        }
    }

    if v.city_was_capital(i, k) & (1u32.wrapping_shl(who as u32) as i32) != 0 {
        s = 1_000_000;
    } else if ii != wid && at_war(v, who, i) && !navy {
        // A past-Military-2 attacker on land, against a city it could take whole.
        if v.military(who) > 2 && diff >= 3 && !v.unassimilated(i, k) {
            let siege = v.count(4, 0);
            if siege == 0 && v.military(who) >= 4 {
                s = div(s, 10);
            }
            if c.supply < 2 {
                if v.military(who) > 4 {
                    s = div(s, 10);
                }
                s = div(s, 10);
                if c.supply < 1 {
                    s = div(s, 10);
                }
            }
        }
        if v.lflags(i) & 4 != 0 {
            match diff {
                4 => {
                    let t = s.wrapping_mul(5);
                    s = (t + ((t >> 31) & 3)) >> 2;
                }
                5 => s = div(s.wrapping_shl(2), 3),
                _ => {}
            }
        }
    }

    // The other armies already after the city.
    let cid = v.city_id(i, k);
    for j in 0..16 {
        let Some((tc, tl)) = v.army_open(j) else {
            continue;
        };
        if tc != cid || tl != ii {
            continue;
        }
        if j as i32 == v.slot() {
            s = s.wrapping_add(s);
            if (ii != wid && at_war(v, who, i)) || bits & 2 != 0 {
                s = s.wrapping_shl(2);
            }
            if ii == who as i32 && v.unassimilated(i, k) {
                s = s.wrapping_shl(3);
            }
            if ii == who as i32 {
                let max = v.building_hits(i, k);
                let dmg = v.building_damage(i, k);
                if dmg.wrapping_add(dmg) > max {
                    s = s.wrapping_shl(3);
                }
            }
        } else if bits & 2 != 0 {
            if ii == wid || allied(v, who, i) || diff >= 3 || navy {
                s = s.wrapping_shl(2);
            } else {
                s = div(s, 4);
            }
        } else {
            s = div(s, 4);
        }
    }
    Some(if s < 0 { 1_000_000 } else { s })
}

/// What the tail needs of the army and the world, beyond the score's.
pub(crate) trait TargetTailView: TargetScoreView {
    /// `army +0x30`/`+0x34` equal the pair.
    fn target_is(&self, city: i32, leader: usize) -> bool;
    /// The cell region of the army's own point, and of the target building's.
    fn army_cell_region(&self) -> i32;
    fn target_cell_region(&self, leader: usize, city: i32) -> i32;
    fn set_hurry0(&mut self);
    fn set_rally(&mut self, v: i32);
    fn set_target(&mut self, city: i32, leader: usize);
    fn mark_attacked(&mut self, leader: usize);
    /// `Armies::send_navy@006f2c90(who, city, leader, region_a, region_b)`.
    fn send_navy(&mut self, city: i32, leader: usize, ra: i32, rb: i32);
    /// The army's point on the target building, a cell down, clamped to the world.
    fn set_point(&mut self, leader: usize, city: i32);
    /// `Army::find_muster_spot@006f5cc0(city, leader, 0)`.
    fn muster_spot(&mut self, city: i32, leader: usize) -> bool;
    /// `Army::close_army@006f8ea0`.
    fn close_army(&mut self);
    /// `muster_angle += 0x80000000`.
    fn flip_angle(&mut self);
    /// The easiest AI against a human: `0x6f9df0(0)` and the probe after it;
    /// `true` when the function ends there.
    fn probe(&mut self, city: i32, leader: usize) -> bool;
}

/// The tail (`6f8206..6f8727`) for the best `(leader, city)`.
pub(crate) fn target_tail<V: TargetTailView>(v: &mut V, best_leader: usize, best_city: i32) {
    let who = v.who();
    if best_leader != who {
        v.set_hurry0();
    }
    if diff_of(v, who) == 0 && v.lflags(best_leader) & 4 != 0 && v.probe(best_city, best_leader) {
        return;
    }
    if !v.target_is(best_city, best_leader) {
        v.set_rally(0x1200);
    }
    v.set_target(best_city, best_leader);
    let bid = v.lid(best_leader);
    if who as i32 != bid && (v.ld(best_leader, who) == 0 || v.lc(who, bid) == 0) {
        v.mark_attacked(best_leader);
        if !v.navy() {
            let ra = v.army_cell_region();
            let rb = v.target_cell_region(best_leader, best_city);
            if ra != rb {
                v.send_navy(best_city, best_leader, ra, rb);
            }
        }
    }
    v.set_point(best_leader, best_city);
    if v.muster_spot(best_city, best_leader) {
        v.flip_angle();
    } else {
        v.close_army();
    }
}

impl SimGate<'_> {
    fn c(&self, a: usize, k: usize) -> usize {
        self.sim.cities_of(a as crate::Player)[k]
    }
}

fn opt_region(r: i32) -> Option<u16> {
    (r >= 0).then_some(r as u16)
}

impl TargetScoreView for SimGate<'_> {
    fn f7e8(&self, _a: usize) -> i32 {
        0
    }
    fn f18(&self, a: usize) -> i32 {
        // The `score x 4/3` clause is a seam (the module doc of `army`): the
        // comparison holds, so the level counts with the army alone.
        i32::from(a == self.who as usize)
    }
    fn transport(&self, a: usize) -> i32 {
        self.sim.transport_level(a as crate::Player) as i32
    }
    fn world_width(&self) -> i32 {
        self.sim.world.width()
    }
    fn city_id(&self, a: usize, k: usize) -> i32 {
        self.c(a, k) as i32
    }
    fn city_bits(&self, a: usize, k: usize) -> i32 {
        let cd = &self.sim.cities[self.c(a, k)];
        1 | if cd.no_heal { 2 } else { 0 }
            | if cd.capital { 0x10 } else { 0 }
            | if cd.no_muster { 0x2000 } else { 0 }
    }
    fn city_bordering(&self, a: usize, k: usize) -> i32 {
        let c = self.c(a, k);
        let owner = self.sim.cities[c].owner as usize;
        self.sim
            .ai
            .get(owner)
            .and_then(|r| r.city_ai.get(c))
            .map_or(0, |ca| ca.bordering)
    }
    fn city_was_capital(&self, a: usize, k: usize) -> i32 {
        self.sim.cities[self.c(a, k)].was_capital as i32
    }
    fn city_stamp(&self, a: usize, k: usize) -> i32 {
        self.sim.cities[self.c(a, k)].attack_stamp as i32
    }
    fn city_cell(&self, a: usize, k: usize) -> (i32, i32) {
        let cell = self.sim.cities[self.c(a, k)].pos.cell();
        (cell.x, cell.y)
    }
    fn army_open(&self, j: usize) -> Option<(i32, i32)> {
        let b = &self.sim.armies[self.who as usize].list[j];
        if !b.valid || b.status & crate::army::status::MUSTERING != 0 {
            return None;
        }
        let none = Some((-1, -1));
        let Some(crate::combat::Obj::Building(bld)) = b.target else {
            return none;
        };
        self.sim
            .cities
            .iter()
            .position(|cd| cd.building == bld)
            .map_or(none, |c| {
                Some((c as i32, i32::from(self.sim.cities[c].owner)))
            })
    }
    fn city_kind(&mut self, a: usize, k: usize) -> i32 {
        match self.sim.city_level_of(self.c(a, k)) {
            2 => 0x19f,
            3 => 0x1a0,
            _ => 0x100,
        }
    }
    fn unassimilated(&mut self, a: usize, k: usize) -> bool {
        self.sim.cities[self.c(a, k)].unassimilated
    }
    fn city_level(&mut self, a: usize, k: usize) -> i32 {
        self.sim.city_level_of(self.c(a, k))
    }
    fn num_wonders(&mut self, _a: usize, _k: usize) -> i32 {
        0
    }
    fn wonder_winning(&mut self) -> i32 {
        -1
    }
    fn has_preq(&mut self) -> bool {
        false
    }
    fn building_damage(&mut self, a: usize, k: usize) -> i32 {
        let bd = &self.sim.buildings[self.sim.cities[self.c(a, k)].building];
        bd.hits - bd.health
    }
    fn building_hits(&mut self, a: usize, k: usize) -> i32 {
        self.sim.buildings[self.sim.cities[self.c(a, k)].building].hits
    }
}

impl TargetTailView for SimGate<'_> {
    fn target_is(&self, city: i32, _leader: usize) -> bool {
        let t = crate::combat::Obj::Building(self.sim.cities[city as usize].building);
        self.sim.armies[self.who as usize].list[self.slot].target == Some(t)
    }
    fn army_cell_region(&self) -> i32 {
        let a = &self.sim.armies[self.who as usize].list[self.slot];
        self.sim.world.region_of(a.pos.cell()).map_or(-1, i32::from)
    }
    fn target_cell_region(&self, _leader: usize, city: i32) -> i32 {
        let t = crate::combat::Obj::Building(self.sim.cities[city as usize].building);
        self.sim
            .world
            .region_of(self.sim.pos_of(t).cell())
            .map_or(-1, i32::from)
    }
    fn set_hurry0(&mut self) {
        self.sim.armies[self.who as usize].list[self.slot].hurry = 0;
    }
    fn set_rally(&mut self, v: i32) {
        self.sim.armies[self.who as usize].list[self.slot].rally_dist = v;
    }
    fn set_target(&mut self, city: i32, _leader: usize) {
        let t = crate::combat::Obj::Building(self.sim.cities[city as usize].building);
        self.sim.armies[self.who as usize].list[self.slot].target = Some(t);
    }
    fn mark_attacked(&mut self, leader: usize) {
        let frame = self.sim.frame;
        self.sim.ai[leader].frame_attacked = frame;
        self.sim.ai[leader].attacked_by = i32::from(self.who);
    }
    fn send_navy(&mut self, city: i32, _leader: usize, ra: i32, rb: i32) {
        let t = crate::combat::Obj::Building(self.sim.cities[city as usize].building);
        self.sim
            .send_navy(self.who, t, opt_region(ra), opt_region(rb));
    }
    fn set_point(&mut self, _leader: usize, city: i32) {
        let cpos = self.sim.cities[city as usize].pos;
        let p = self.sim.restrict(crate::world::Pos::new(
            cpos.x,
            cpos.y + crate::world::UNITS_PER_CELL,
        ));
        self.sim.armies[self.who as usize].list[self.slot].pos = p;
    }
    fn muster_spot(&mut self, city: i32, _leader: usize) -> bool {
        let t = crate::combat::Obj::Building(self.sim.cities[city as usize].building);
        self.sim.find_muster_spot(self.who, self.slot, t, false)
    }
    fn close_army(&mut self) {
        self.sim.close_army(self.who, self.slot);
    }
    fn flip_angle(&mut self) {
        let a = &mut self.sim.armies[self.who as usize].list[self.slot];
        a.muster_angle = crate::movement::Angle(a.muster_angle.0.wrapping_add(i32::MIN));
    }
    fn probe(&mut self, city: i32, leader: usize) -> bool {
        let frame = self.sim.frame;
        self.sim.find_target_probe(
            self.who,
            self.slot,
            city as usize,
            leader as crate::Player,
            frame,
        );
        true
    }
}
