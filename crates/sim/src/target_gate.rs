//! `Army::find_target@006f69b0`, **the gate chain** — from the entry to the
//! first score draw (item 3362, `sweep/army_find_target.rs`).
//!
//! The function is 1219 lines of decompile over the leaders table; this
//! module is the part of it the emulator can be held to without the rest:
//! the head (the stop bit, the weak test, `find_capital`, the two sea
//! averages), the leader gate (diplomacy, the difficulty clauses, the
//! stamp, the aggressive-army test and its coin, the team-style list) and
//! the city gate (`valid`, the coast and ocean test of a navy, the founder
//! test of the two easiest difficulties). Everything the sweep sees is a
//! read of a raw field of the original's record or a call of one of its
//! eight callees, in the original's order; [`TargetGateView`] names each.
//!
//! What the sim does not keep is answered the way the sim has always
//! answered it (`army::seams`): no stop bit, the semaphore bytes zero (so
//! the difficulty is the game's), `+0x230` set, `+0x250` clear and no
//! tribute period (so an allied leader passes on `defense_mod` alone), the
//! second wonder timer zero. The sweep draws every one of them.

/// The call ended at the draw the sweep stops at (the coin, or the score).
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Stopped;

type G<T> = Result<T, Stopped>;

/// `ecx` at the one `find_aggressive_army` call the original makes with a
/// stale register: the stamp sum (`fa + 0x1c20`), or the army itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Ecx {
    Army,
    Num(i32),
}

/// The original's reads and calls, by the record they name. `a`/`b` are
/// leader indices, `k` a city's index in `cities[a]`. Reads take `&self`,
/// calls `&mut self` (the sweep logs a call).
pub(crate) trait TargetGateView {
    // ---- `this`: the army ----
    /// `+0x2`, the slot.
    fn slot(&self) -> i32;
    /// `+0x8`, the region; `-1` for none.
    fn region(&self) -> i32;
    /// `+0x14`.
    fn captains(&self) -> i32;
    /// `+0x18`.
    fn standard(&self) -> i32;
    /// `+0x24`.
    fn navy(&self) -> bool;
    /// `+0x94`.
    fn who(&self) -> usize;

    // ---- `leaders[a]` ----
    /// `+0`: 1 active, 2 seated, 4 human, 8 stop.
    fn lflags(&self, a: usize) -> i32;
    /// `+0x8`, the leader's own index.
    fn lid(&self, a: usize) -> i32;
    /// `+0x50`, `multi_diff`.
    fn lmulti(&self, a: usize) -> i32;
    /// `+0x74 + 4b`, `leaders[a].diplos[b]`: 0 war, 2 allied.
    fn ld(&self, a: usize, b: usize) -> i32;
    /// The same word through `GameAccessConst::leadersc`: `leadersc[a].diplos[b]`.
    fn lc(&self, a: usize, b: i32) -> i32;
    /// `leadersc[a].flags` (`& 3 == 3` is seated and live: the start list).
    fn lcflags(&self, a: i32) -> i32;
    /// `+0x230 + 4b`.
    fn x230(&self, a: usize, b: usize) -> i32;
    /// `+0x250 + 4b`.
    fn x250(&self, a: usize, b: usize) -> i32;
    /// `+0x408`, the city count.
    #[allow(dead_code)] // the sweep's loop reads it; `Sim::find_target` walks `cities_of`
    fn ncities(&self, a: usize) -> i32;
    /// `+0x444` (`popwin_timer`) and `+0x44c` (`wonderwin_timer`).
    fn w444(&self, a: usize) -> i32;
    fn w44c(&self, a: usize) -> i32;
    /// `+0x6dec` (`raid`), `+0x6e20` (`early_army`).
    fn raid(&self, a: usize) -> i32;
    fn early(&self, a: usize) -> i32;
    /// `+0x7a8`, `defense_mod`.
    fn defmod(&self, a: usize) -> i32;
    /// `+0x944` (the census combat) and `+0x94c` (the sea average's term).
    fn combat(&self, a: usize) -> i32;
    fn sea(&self, a: usize) -> i32;
    /// `+0xa40`, `frame_attacked`, and `+0xa44`, `attacked_by`.
    fn frame_attacked(&self, a: usize) -> i32;
    fn attacked_by(&self, a: usize) -> i32;
    /// `*(+0x6eb8) + 0xe8 ^ 0x63187`, the Military library level.
    fn military(&self, a: usize) -> i32;

    // ---- the game ----
    /// `gamec +0x820..+0x822`, the semaphore bytes; `+0x2b`, the difficulty.
    fn semaphore(&self) -> [u8; 3];
    fn game_difficulty(&self) -> i32;
    /// `game +0x24`, `team_style`; `game +0x550`, the frame.
    fn team_style(&self) -> i32;
    fn frame(&self) -> i32;
    /// `gamec +0x65c + 4k`, `start_list`; `gamec +0x67c + 4id`, `start_index`.
    fn start_list(&self, k: i32) -> i32;
    fn start_index(&self, id: i32) -> i32;

    // ---- `cities[a][k]` ----
    /// `+0x4`, bit 0 valid.
    fn city_flags(&self, a: usize, k: usize) -> i32;
    /// `+0xa`, the region short.
    fn city_reg(&self, a: usize, k: usize) -> i32;
    /// `+0x60`, the founder byte.
    fn city_founder(&self, a: usize, k: usize) -> i32;
    /// `+0x62`, `ocean`.
    fn city_ocean(&self, a: usize, k: usize) -> i32;
    /// The capital's cell: `div_3_table[pos >> 8]` on each axis.
    fn capital_cell(&self, owner: i32, idx: i32) -> (i32, i32);

    // ---- the callees ----
    /// `Army::count@006f9120(kind, arg)`.
    fn count(&mut self, kind: i32, arg: i32) -> i32;
    /// `LeaderData::type_avail@006e33a0` of the army's owner.
    fn type_avail(&mut self, ty: i32, strict: i32) -> i32;
    /// `LeaderData::find_capital@006eb930(&idx, &owner, -1, -1)`.
    fn find_capital(&mut self) -> (i32, i32);
    /// `LeaderData::is_tribute_period@006d5b90` of the army's owner, handed `b`.
    fn tribute(&mut self, b: usize) -> bool;
    /// `Armies::find_aggressive_army@006f2e10(who)`.
    fn find_aggressive(&mut self, ecx: Ecx) -> i32;
    /// `LeaderData::get_diff@006ec000` of the army's owner.
    fn get_diff(&mut self) -> i32;
    /// `Region::is_coast@00680f90`: the army's region, a city's.
    fn is_coast(&mut self, reg: i32, city_reg: i32) -> bool;
    /// `Random::get(0, 0xffff)@00a39d70`.
    fn draw(&mut self) -> G<i32>;
}

/// The difficulty the original reads at every site, of the army's owner:
/// `multi_diff` under the multiplayer semaphore, the game's otherwise.
pub(crate) fn diff_of<V: TargetGateView>(v: &V, a: usize) -> i32 {
    let s = v.semaphore();
    if s[0] & 4 != 0 {
        return v.lmulti(a);
    }
    if s[1] & 0x10 != 0 && s[2] & 2 == 0 {
        let m = v.lmulti(a);
        if m >= 0 {
            return m;
        }
    }
    v.game_difficulty()
}

/// What the head leaves for the rest of the function.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Head {
    /// `leader_flags & 8`: the function returned.
    pub stop: bool,
    /// `local_54`: not weak.
    pub strong: bool,
    /// The capital's cell, when the owner has one.
    pub cap: Option<(i32, i32)>,
    /// The two sea averages (`local_4c`, `local_48`).
    pub ours: i32,
    pub theirs: i32,
}

/// Entry to the first leader: the supply count, the stop bit, the weak
/// test, `find_capital`, the sea averages.
pub(crate) fn target_head<V: TargetGateView>(v: &mut V) -> Head {
    let who = v.who();
    let supply = v.count(0x13, 0x3f);
    if v.lflags(who) & 8 != 0 {
        return Head {
            stop: true,
            strong: false,
            cap: None,
            ours: 0,
            theirs: 0,
        };
    }
    let mut strong = true;
    if !v.navy() {
        let siege = v.count(4, 0);
        let cata = v.count(0x13, 0xe3);
        let hoplites = v.count(0x13, 0x84);
        strong = false;
        if hoplites + 2 * siege + cata >= 4 {
            // Military 3, a wagon the leader can have, none of its own.
            if !(v.military(who) >= 3 && v.type_avail(0x3f, 1) != 0 && supply == 0) {
                strong = true;
            }
        }
    }
    let (idx, owner) = v.find_capital();
    let cap = if idx < 0 || owner != who as i32 {
        None
    } else {
        Some(v.capital_cell(owner, idx))
    };
    let (mut os, mut on, mut ts, mut tn) = (0i32, 0i32, 0i32, 0i32);
    for x in 0..8 {
        if v.lflags(x) & 1 == 0 {
            continue;
        }
        let idv = v.lid(x);
        if who as i32 == idv || (v.ld(x, who) == 2 && v.lc(who, idv) == 2) {
            os = os.wrapping_add(v.sea(x));
            on += 1;
        } else if v.ld(x, who) == 0 || v.lc(who, idv) == 0 {
            ts = ts.wrapping_add(v.sea(x));
            tn += 1;
        }
    }
    Head {
        stop: false,
        strong,
        cap,
        ours: if on != 0 { os / on } else { os },
        theirs: if tn != 0 { ts / tn } else { ts },
    }
}

/// The leader `i`'s gate (`6f6bc9..6f7065`): `None` skips the leader,
/// `Some(about_to_win)` goes on to its cities (`local_34`, set only on the
/// war arm).
pub(crate) fn target_leader_gate<V: TargetGateView>(v: &mut V, i: usize) -> G<Option<bool>> {
    let who = v.who();
    if v.lflags(i) & 2 == 0 {
        return Ok(None);
    }
    let mut about = false;
    if i != who {
        let wid = v.lid(who);
        let ii = i as i32;
        // "Not at war" (the first arm), or the allied arm's own checks.
        let peace = ii == wid || (v.ld(who, i) != 0 && v.lc(i, wid) != 0);
        if !peace {
            if v.defmod(who) > 0x100 {
                return Ok(None);
            }
            about = v.w44c(who) != 0 || v.w444(who) != 0;
        } else {
            if ii != wid && (v.ld(who, i) != 2 || v.lc(i, wid) != 2) {
                return Ok(None);
            }
            if v.x250(who, i) != 0 || v.x230(who, i) == 0 || v.tribute(i) || v.defmod(who) < 0x100
            {
                return Ok(None);
            }
        }
        let frame = v.frame();
        if diff_of(v, who) < 2 {
            if diff_of(v, who) == 0 && v.lflags(i) & 4 == 0 {
                return Ok(None);
            }
            let stamp = v.frame_attacked(i).wrapping_add(0x1c20);
            if stamp > frame {
                return Ok(None);
            }
            let go = {
                let aggr = v.find_aggressive(Ecx::Num(stamp));
                if aggr >= 0 {
                    aggr == v.slot()
                } else {
                    (v.draw()? & i32::MIN.wrapping_add(1)) == 0
                }
            };
            if !go {
                return Ok(None);
            }
        }
        if diff_of(v, who) == 1 {
            if v.captains() >= v.combat(who) / 2 {
                return Ok(None);
            }
            let aggr = v.find_aggressive(Ecx::Army);
            if aggr >= 0 && aggr != v.slot() {
                return Ok(None);
            }
        }
        if diff_of(v, who) >= 2 {
            let pass = ii == wid
                || (v.ld(who, i) == 2 && v.lc(i, wid) == 2)
                || (!(v.military(who) > 2 || v.early(who) == 0) && v.get_diff() != 2)
                || 7 - 2 * v.raid(who) <= v.standard();
            if !pass {
                return Ok(None);
            }
        }
        if diff_of(v, who) == 2 {
            let fa = v.frame_attacked(i);
            if fa.wrapping_add(0x708) > frame {
                return Ok(None);
            }
            if v.team_style() != 3
                && fa.wrapping_add(0xe10) > frame
                && v.attacked_by(i) != who as i32
                && v.attacked_by(who) != ii
            {
                return Ok(None);
            }
        }
    }
    // The team-style-2 list (`6f6fb4`): me, the leader the list names, or
    // a mutual ally.
    if v.team_style() == 2 {
        let wid = v.lid(who);
        let base = v.start_index(wid).wrapping_add(1);
        let mut target = wid;
        for j in 0..8 {
            let t = v.start_list(base.wrapping_add(j) & 7);
            if v.lcflags(t) & 3 == 3 {
                target = t;
                break;
            }
        }
        let ii = i as i32;
        if target != ii && ii != wid && (v.ld(who, i) != 2 || v.lc(i, wid) != 2) {
            return Ok(None);
        }
    }
    Ok(Some(about))
}

/// A city's gate (`6f70a2..6f7185`): `true` is the score draw.
pub(crate) fn target_city_gate<V: TargetGateView>(v: &mut V, i: usize, k: usize) -> bool {
    let who = v.who();
    if v.city_flags(i, k) & 1 == 0 {
        return false;
    }
    if v.navy() {
        let reg = v.region();
        let cr = v.city_reg(i, k);
        if !v.is_coast(reg, cr) || v.city_ocean(i, k) == 0 {
            return false;
        }
    }
    if diff_of(v, who) <= 1 {
        let iid = v.lid(i);
        let allied = who as i32 == iid || (v.ld(i, who) == 2 && v.lc(who, iid) == 2);
        if allied && (v.city_founder(i, k) as i8 as i16 as u16) != who as u16 {
            return false;
        }
    }
    true
}

/// The sim's answer to every read: `Sim::find_target`'s own state, with the
/// fields it does not keep answered as `army::seams` says.
pub(crate) struct SimGate<'a> {
    pub sim: &'a mut crate::Sim,
    pub who: crate::Player,
    pub slot: usize,
}

impl SimGate<'_> {
    fn army(&self) -> &crate::army::Army {
        &self.sim.armies[self.who as usize].list[self.slot]
    }
    fn live(&self, a: usize) -> bool {
        a < self.sim.players.len() && !self.sim.defeated[a]
    }
    fn city(&self, a: usize, k: usize) -> Option<usize> {
        self.sim.cities_of(a as crate::Player).get(k).copied()
    }
}

impl TargetGateView for SimGate<'_> {
    fn slot(&self) -> i32 {
        self.slot as i32
    }
    fn region(&self) -> i32 {
        self.army().reg.map_or(-1, i32::from)
    }
    fn captains(&self) -> i32 {
        self.army().num_captains
    }
    fn standard(&self) -> i32 {
        self.army().num_standard
    }
    fn navy(&self) -> bool {
        self.army().navy
    }
    fn who(&self) -> usize {
        self.who as usize
    }
    fn lflags(&self, a: usize) -> i32 {
        if !self.live(a) {
            return 0;
        }
        3 | if self.sim.nation[a].human { 4 } else { 0 }
    }
    fn lid(&self, a: usize) -> i32 {
        a as i32
    }
    fn lmulti(&self, _a: usize) -> i32 {
        -1
    }
    fn ld(&self, a: usize, b: usize) -> i32 {
        let (a, b) = (a as crate::Player, b as crate::Player);
        if a == b || self.sim.allied_with(a, b) && !self.sim.at_war_with(a, b) {
            2
        } else if self.sim.at_war_with(a, b) {
            0
        } else {
            1
        }
    }
    fn lc(&self, a: usize, b: i32) -> i32 {
        self.ld(a, b as usize)
    }
    fn lcflags(&self, a: i32) -> i32 {
        self.lflags(a as usize)
    }
    fn x230(&self, _a: usize, _b: usize) -> i32 {
        1
    }
    fn x250(&self, _a: usize, _b: usize) -> i32 {
        0
    }
    fn ncities(&self, a: usize) -> i32 {
        self.sim.cities_of(a as crate::Player).len() as i32
    }
    fn w444(&self, a: usize) -> i32 {
        self.sim.ai[a].popwin_timer
    }
    fn w44c(&self, _a: usize) -> i32 {
        0
    }
    fn raid(&self, a: usize) -> i32 {
        self.sim.ai[a].pers.raid
    }
    fn early(&self, a: usize) -> i32 {
        self.sim.ai[a].pers.early_army
    }
    fn defmod(&self, a: usize) -> i32 {
        self.sim.ai[a].defense_mod
    }
    fn combat(&self, a: usize) -> i32 {
        self.sim.ai[a].census.combat
    }
    fn sea(&self, a: usize) -> i32 {
        self.sim.ai[a].census.sea_combat
    }
    fn frame_attacked(&self, a: usize) -> i32 {
        self.sim.ai[a].frame_attacked as i32
    }
    fn attacked_by(&self, a: usize) -> i32 {
        self.sim.ai[a].attacked_by
    }
    fn military(&self, a: usize) -> i32 {
        self.sim.tech[a].military_level() as i32
    }
    fn semaphore(&self) -> [u8; 3] {
        [0; 3]
    }
    fn game_difficulty(&self) -> i32 {
        self.sim.ai_difficulty()
    }
    fn team_style(&self) -> i32 {
        self.sim.lobby.team_style
    }
    fn frame(&self) -> i32 {
        self.sim.frame as i32
    }
    fn start_list(&self, k: i32) -> i32 {
        self.sim.lobby.start_list[(k & 7) as usize] as i32
    }
    fn start_index(&self, id: i32) -> i32 {
        self.sim
            .lobby
            .start_list
            .iter()
            .position(|&l| l == id as usize)
            .unwrap_or(0) as i32
    }
    fn city_flags(&self, _a: usize, _k: usize) -> i32 {
        1
    }
    fn city_reg(&self, a: usize, k: usize) -> i32 {
        self.city(a, k)
            .and_then(|c| self.sim.cities[c].reg)
            .map_or(-1, i32::from)
    }
    fn city_founder(&self, a: usize, k: usize) -> i32 {
        self.city(a, k)
            .map_or(-1, |c| i32::from(self.sim.cities[c].founder))
    }
    fn city_ocean(&self, a: usize, k: usize) -> i32 {
        self.city(a, k).map_or(0, |c| {
            self.sim.ai[a]
                .city_ai
                .get(c)
                .map_or(0, |r| i32::from(r.ocean))
        })
    }
    fn capital_cell(&self, owner: i32, idx: i32) -> (i32, i32) {
        let c = self.sim.cities_of(owner as crate::Player)[idx as usize];
        let cell = self.sim.cities[c].pos.cell();
        (cell.x, cell.y)
    }
    fn count(&mut self, kind: i32, arg: i32) -> i32 {
        let (w, s) = (self.who, self.slot);
        match (kind, arg) {
            (0x13, 0x3f) => self.sim.army_count_supply(w, s),
            (4, 0) => self.sim.army_count_siege(w, s),
            (0x13, 0xe3) => self.sim.army_count_cataphracts(w, s),
            (0x13, 0x84) => self.sim.army_count_hoplites(w, s),
            _ => 0,
        }
    }
    fn type_avail(&mut self, ty: i32, _strict: i32) -> i32 {
        self.sim.type_avail(self.who, ty as crate::tech::TypeId)
    }
    fn find_capital(&mut self) -> (i32, i32) {
        let w = self.who;
        let idx = self
            .sim
            .cities_of(w)
            .into_iter()
            .position(|c| self.sim.cities[c].capital);
        (idx.map_or(-1, |i| i as i32), i32::from(w))
    }
    fn tribute(&mut self, _b: usize) -> bool {
        false
    }
    fn find_aggressive(&mut self, _ecx: Ecx) -> i32 {
        self.sim
            .find_aggressive_army(self.who)
            .map_or(-1, |k| k as i32)
    }
    fn get_diff(&mut self) -> i32 {
        self.sim.ai_difficulty()
    }
    fn is_coast(&mut self, reg: i32, city_reg: i32) -> bool {
        reg >= 0 && city_reg >= 0 && self.sim.world.is_coast(reg as u16, city_reg as u16)
    }
    fn draw(&mut self) -> G<i32> {
        self.sim.mark(crate::army::SITE_FIND_TARGET_COIN);
        Ok(self.sim.rng.get(0, 0xffff))
    }
}
