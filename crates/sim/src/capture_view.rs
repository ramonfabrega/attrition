//! **`Cities::capture_city@00733380`'s hand-over head** (item 3362): from the
//! entry to the `City::find_buildings` call, as a pure walk over a
//! [`CaptureView`] — the swept steps read and write the world only through
//! it, so the sweep's port is a view over the emulated original's windows and
//! the crate's is [`SimCapture`] (`docs/CITIES.md` §7.3).
//!
//! The control flow is the original's, call for call: every leaf is one
//! virtual call (or one direct callee, or one inline read) of the listing,
//! asked in the order the listing asks it — which is what the sweep holds.
//!
//! Not swept, and not claimed: the failed-swap arm (the swap answered a
//! negative index, or the new building has no city: the original's local
//! `+0x72` is -1 and the function reads `cities[A][-1]` after it, then
//! walks the old city's members), the allied circle sweep after
//! `find_buildings`, the plunder, the close lists and everything after.

use crate::build::{self, Ident, flags};
use crate::combat::Obj;
use crate::{Player, Sim};

/// `has_tribe_bonus(0x17)`: the Persians.
pub const TRIBE_PERSIANS: i32 = 0x17;
/// `has_tribe_bonus(0x13)`: the Lakota.
pub const TRIBE_LAKOTA: i32 = 0x13;
/// `is(0x1a1, 0)`: a Farm by lineage.
pub const IS_FARM: i32 = 0x1a1;
/// `is(0x1a7, 0)`: a Granary by lineage.
pub const IS_GRANARY: i32 = 0x1a7;
/// `ACHIEVEEVENT_CITY_CAPTURED` / `ACHIEVEEVENT_CITY_LOST`.
pub const ACHIEVE_CAPTURED: i32 = 6;
pub const ACHIEVE_LOST: i32 = 7;
/// What a member another civilian building of the city costs the plunder.
pub const MEMBER_BASE: i32 = 25;

/// What `Cities::capture_city` asks of the world before the plunder. `B`
/// names a building (the short in its owner's objects array).
pub trait CaptureView {
    type B: Copy;

    /// `leaders[o].dip[a].agree == 1` (`[o row + 0x692c + 0x5c * a]`).
    fn diplo_agrees(&mut self, a: Player, o: Player) -> bool;
    /// `Leader::action_clear_all@006d15e0(a)` on leader `o`.
    fn action_clear_all(&mut self, o: Player, a: Player);
    /// The city record's `flags & 0x10`: a capital.
    fn city_is_capital(&mut self, o: Player, c: usize) -> bool;
    /// `LeaderData::has_tribe_bonus@006e1370(id)` on `who`.
    fn has_tribe_bonus(&mut self, who: Player, id: i32) -> bool;
    /// `LeaderData::find_capital@006eb930(&_, &_, c, o)` on leader `o`.
    fn find_capital(&mut self, o: Player, c: usize);
    /// `leaders[o] +0x828 += 1`.
    fn count_lost(&mut self, o: Player);
    /// `leaders[a] +0x824 += 1`.
    fn count_captured(&mut self, a: Player);
    /// The achievements arm (`game->player[0].who == a`, … then
    /// `IStatsAndAchievements::Get()` and its slot `+8`).
    fn achievement_ping(&mut self, a: Player);
    /// `leader_flags |= 0x2000000`.
    fn mark_economy(&mut self, who: Player);
    /// `LeaderData::get_radius@006db790` of the city's building type, the
    /// answer kept for the allied circle sweep after this head.
    fn city_radius_probe(&mut self, o: Player, c: usize);
    /// The city record's building (`+8`).
    fn city_building(&mut self, o: Player, c: usize) -> Self::B;
    /// The old city building's slot `+0xb0`, then its slot `+0x94(a)`:
    /// `Build::swap_team`. `None` when it answers a negative index.
    fn swap_head(&mut self, o: Player, b: Self::B, a: Player) -> Option<Self::B>;
    /// A member's data (slot `+0xac`), then its slot `+0x94(a)`.
    fn swap_member(&mut self, o: Player, b: Self::B, a: Player) -> Option<Self::B>;
    /// The new building `n` of `a`'s data (slot `+0xac`), then its slot
    /// `+0x1a8(1, 1, 0)`: `Build::activate`.
    fn activate(&mut self, a: Player, n: Self::B);
    /// `n`'s data (slot `+0xac`) `+0x72`: the city it belongs to.
    fn city_of(&mut self, a: Player, n: Self::B) -> Option<usize>;
    /// `CityData::get_level@00739340` of `a`'s city `nc`.
    fn level(&mut self, a: Player, nc: usize) -> i32;
    /// `constants->city_plunder_per_level`.
    fn plunder_per_level(&mut self) -> i32;
    /// `Achieve::add_event@007af660(kind, who, &city.name)`, handed the
    /// stale `ecx` the caller has (`c`).
    fn achieve(&mut self, kind: i32, who: Player, o: Player, c: usize);
    /// `City::capture@00736c40(new, old, nc, a, n)`.
    fn city_capture(&mut self, nc: usize, o: Player, c: usize, a: Player, n: Self::B);
    /// `Armies::update_city@006f2d70(old, o, n, a, _)`.
    fn update_city(&mut self, old: Self::B, o: Player, n: Self::B, a: Player);
    /// The old building's data (slot `+0xac`) `+0x74`: the first member.
    fn first_member(&mut self, o: Player, b: Self::B) -> Option<Self::B>;
    /// A member's data (slot `+0xac`) `+0x74`: the next one.
    fn next_member(&mut self, o: Player, m: Self::B) -> Option<Self::B>;
    /// A member's data (slot `+0xac`), its type's `flags & 0x2000`: military,
    /// a city, a dock — exempt from conversion.
    fn not_civilian(&mut self, o: Player, m: Self::B) -> bool;
    /// `is(id, 0)` on a member's data (slot `+0xac`, then the `Build`'s type
    /// slot `+0x60` or the data's own `+0xb8`).
    fn is_ident(&mut self, o: Player, m: Self::B, id: i32) -> bool;
    /// A member's data (slot `+0xac`), then its `is_active` (the `Build`'s
    /// `flags & 4`, or slot `+0x4c`).
    fn is_active(&mut self, o: Player, m: Self::B) -> bool;
    /// `City::find_buildings@007384c0` on `a`'s new city.
    fn find_buildings(&mut self, a: Player, nc: usize);
}

/// What the head leaves for the rest of the function.
pub struct Head<B> {
    /// The new city, or `None` when the swap failed (the failed-swap arm is
    /// the caller's).
    pub newcity: Option<usize>,
    /// `local_28`: the plunder base so far.
    pub base: i32,
    /// `local_74`: the buildings to close, the old city building first.
    pub to_close: Vec<B>,
    /// `local_98`: the new buildings the original `mask_me`s at the end
    /// (SEAM: the port has no mask pass to hand them to).
    #[allow(dead_code)]
    pub masks: Vec<B>,
}

/// Entry to the `City::find_buildings` call of `Cities::capture_city`.
pub fn capture_city_steps<V: CaptureView>(v: &mut V, a: Player, c: usize, o: Player) -> Head<V::B> {
    if v.diplo_agrees(a, o) {
        v.action_clear_all(o, a);
    }
    let capital = v.city_is_capital(o, c);
    if v.has_tribe_bonus(o, TRIBE_PERSIANS) && capital {
        v.find_capital(o, c);
    }
    v.count_lost(o);
    v.count_captured(a);
    v.achievement_ping(a);
    v.mark_economy(a);
    v.mark_economy(o);
    v.city_radius_probe(o, c);
    let old = v.city_building(o, c);
    let mut to_close = vec![old];
    let mut masks = Vec::new();
    let mut base = 0;
    let mut newcity = None;
    let mut n_new = None;
    if let Some(n) = v.swap_head(o, old, a) {
        v.activate(a, n);
        masks.push(n);
        if let Some(nc) = v.city_of(a, n) {
            let level = v.level(a, nc);
            base = v.plunder_per_level().wrapping_mul(level.wrapping_sub(1));
            newcity = Some(nc);
            n_new = Some(n);
        }
    }
    v.achieve(ACHIEVE_CAPTURED, a, o, c);
    v.achieve(ACHIEVE_LOST, o, o, c);
    let (Some(nc), Some(n)) = (newcity, n_new) else {
        return Head {
            newcity: None,
            base: 0,
            to_close,
            masks,
        };
    };
    v.city_capture(nc, o, c, a, n);
    v.update_city(old, o, n, a);
    let mut m = v.first_member(o, old);
    while let Some(x) = m {
        if !v.not_civilian(o, x) {
            to_close.push(x);
            let farmish = v.is_ident(o, x, IS_FARM) || v.is_ident(o, x, IS_GRANARY);
            if !(farmish && v.has_tribe_bonus(a, TRIBE_LAKOTA)) && v.is_active(o, x) {
                if let Some(nm) = v.swap_member(o, x, a) {
                    v.activate(a, nm);
                    masks.push(nm);
                    base = base.wrapping_add(MEMBER_BASE);
                }
            }
        }
        m = v.next_member(o, x);
    }
    v.find_buildings(a, nc);
    Head {
        newcity,
        base,
        to_close,
        masks,
    }
}

/// The crate's view: the live [`Sim`], the old city's member list as it
/// stood at the entry.
pub struct SimCapture<'a> {
    pub sim: &'a mut Sim,
    pub members: Vec<usize>,
}

fn ident_of(id: i32) -> Option<Ident> {
    match id {
        IS_FARM => Some(Ident::Farm),
        IS_GRANARY => Some(Ident::Granary),
        _ => None,
    }
}

impl CaptureView for SimCapture<'_> {
    type B = usize;

    /// SEAM: `dip[t].agree` is never 1 in the port (`diplomacy.rs`).
    fn diplo_agrees(&mut self, _a: Player, _o: Player) -> bool {
        false
    }
    fn action_clear_all(&mut self, _o: Player, _a: Player) {}
    fn city_is_capital(&mut self, _o: Player, c: usize) -> bool {
        self.sim.cities[c].capital
    }
    /// SEAM: only the Lakota bonus is a nation flag here; the Persians' is
    /// asked only to feed `find_capital`, which this crate does not model.
    fn has_tribe_bonus(&mut self, who: Player, id: i32) -> bool {
        id == TRIBE_LAKOTA && self.sim.nation[who as usize].lakota
    }
    /// SEAM: the Persian second-capital flag is read after the plunder.
    fn find_capital(&mut self, _o: Player, _c: usize) {}
    fn count_lost(&mut self, o: Player) {
        self.sim.city_tally[o as usize].lost += 1;
    }
    fn count_captured(&mut self, a: Player) {
        self.sim.city_tally[a as usize].captured += 1;
    }
    /// The console and the achievements are the renderer's.
    fn achievement_ping(&mut self, _a: Player) {}
    fn mark_economy(&mut self, who: Player) {
        self.sim.economy_changed(who);
    }
    /// SEAM: the allied circle sweep is not modelled.
    fn city_radius_probe(&mut self, _o: Player, _c: usize) {}
    fn city_building(&mut self, _o: Player, c: usize) -> usize {
        self.sim.cities[c].building
    }
    fn swap_head(&mut self, _o: Player, b: usize, a: Player) -> Option<usize> {
        self.sim.swap_team(b, a)
    }
    fn swap_member(&mut self, _o: Player, b: usize, a: Player) -> Option<usize> {
        self.sim.swap_team(b, a)
    }
    fn activate(&mut self, _a: Player, n: usize) {
        self.sim.activate(n, true, false);
    }
    fn city_of(&mut self, _a: Player, n: usize) -> Option<usize> {
        self.sim.buildings[n].city
    }
    fn level(&mut self, _a: Player, nc: usize) -> i32 {
        self.sim.city_level_of(nc)
    }
    fn plunder_per_level(&mut self) -> i32 {
        self.sim.tuning.city_plunder_per_level
    }
    fn achieve(&mut self, _kind: i32, _who: Player, _o: Player, _c: usize) {}
    fn city_capture(&mut self, nc: usize, _o: Player, c: usize, a: Player, n: usize) {
        self.sim.city_capture_record(nc, c, a, n);
    }
    fn update_city(&mut self, old: usize, _o: Player, n: usize, a: Player) {
        self.sim
            .armies_update_city(Obj::Building(old), Obj::Building(n), a);
    }
    fn first_member(&mut self, _o: Player, _b: usize) -> Option<usize> {
        self.members.first().copied()
    }
    fn next_member(&mut self, _o: Player, m: usize) -> Option<usize> {
        let i = self.members.iter().position(|&x| x == m)?;
        self.members.get(i + 1).copied()
    }
    fn not_civilian(&mut self, _o: Player, m: usize) -> bool {
        // A member with no type is skipped, as the unconverted.
        self.sim.buildings[m]
            .ty
            .is_none_or(|t| self.sim.build_types[t].has(flags::NOT_CIVILIAN))
    }
    fn is_ident(&mut self, _o: Player, m: usize, id: i32) -> bool {
        match (self.sim.buildings[m].ty, ident_of(id)) {
            (Some(t), Some(i)) => build::is(&self.sim.build_types, t, i),
            _ => false,
        }
    }
    fn is_active(&mut self, _o: Player, m: usize) -> bool {
        self.sim.buildings[m].active
    }
    fn find_buildings(&mut self, _a: Player, nc: usize) {
        self.sim.find_buildings(nc);
    }
}
