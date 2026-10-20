//! **`Build::activate@00623e20`**, by slices (item 3364; `docs/CITIES.md` §4).
//!
//! The function is two thousand lines: the head, up to its call to
//! `Wall::activate`, is [`activate_head_steps`] over an [`ActivateHeadView`]
//! that names every access it makes in the order it makes them. [`SimHead`]
//! answers from this crate's state; the sweep
//! (`crates/sim/src/sweep/build_activate_head.rs`) answers from the emulated
//! original's windows and ends the call where `Wall::activate` is first
//! entered (`--stop-stub`).
//!
//! **Not established.** `Build::new_library` has no counterpart here (the
//! crate reads a library's queue from the first library where it is used,
//! `docs/CITIES.md` §4); the per-leader `last_build_built` tables the head
//! writes (`leaders +0x59f8`, `ScenarioData::last_build_built_o`) are read by
//! nothing in this crate; `Wall::mask_me(1)` on the converted city is the
//! city arm's own `mask_city`, later.

use crate::build::Ident;
use crate::{Player, Sim};

/// What `Build::activate`'s head reads and does.
pub trait ActivateHeadView {
    /// The building's `is(id, 0)` (`0x1b3` a Library, `0x213` the Forbidden
    /// City).
    fn is_type(&mut self, id: i32) -> bool;
    /// `Build::new_library`.
    fn new_library(&mut self);
    /// `+0x7a = 0`: the `recharging` count a site collected while built.
    fn clear_recharging(&mut self);
    /// The building's `flags & 0x20`: a city.
    fn city_flag(&mut self) -> bool;
    /// The leader's `+0x3f8` (its city count) moved by `d`.
    fn cities_add(&mut self, d: i32);
    /// `LeaderData::has_tribe_bonus(bonus)` of the owner.
    fn tribe_bonus(&mut self, bonus: i32) -> bool;
    /// `GameAccess::constants->chinese_large_cities != 0`.
    fn chinese_large_cities(&mut self) -> bool;
    /// The building's slot `+0x84(id, 0)`: its type becomes `id`.
    fn set_type(&mut self, id: i32);
    /// `Wall::mask_me(1, REGEN_NONE)`.
    fn mask_me(&mut self);
    /// `leader_flags |= 0x8000000`.
    fn flag_wall_stats(&mut self);
    /// The two `last_build_built` writes: `leaders[who] +0x59f8[type idx]`
    /// and `ScenarioData::last_build_built_o[who]`, the building's `+0xa`.
    fn record_last_built(&mut self);
    /// `Wall::activate(captured, announce, 0)`: where the slice ends.
    fn wall_activate(&mut self, captured: i32, announce: i32);
}

/// `Build::activate@00623e20`, from its entry to its call of
/// `Wall::activate` (lines 121–159).
pub fn activate_head_steps<V: ActivateHeadView>(
    v: &mut V,
    captured: i32,
    announce: i32,
    counted: i32,
) {
    if v.is_type(0x1b3) {
        v.new_library();
    }
    v.clear_recharging();
    if v.city_flag() {
        v.cities_add(1);
        // Chinese cities are founded as Large Cities (a captured one keeps
        // its level), unless it is the Forbidden City.
        if v.tribe_bonus(0xe) && captured == 0 && v.chinese_large_cities() && !v.is_type(0x213) {
            v.set_type(0x19f);
            v.mask_me();
            v.flag_wall_stats();
        }
        v.cities_add(-1);
    }
    if captured == 0 && counted != 0 {
        v.record_last_built();
    }
    v.wall_activate(captured, announce);
}

/// [`ActivateHeadView`] over a [`Sim`]. The head's `Wall::activate` is the
/// rest of [`Sim::activate`], so [`ActivateHeadView::wall_activate`] does
/// nothing here.
pub(crate) struct SimHead<'a> {
    pub(crate) sim: &'a mut Sim,
    pub(crate) b: usize,
    pub(crate) who: Player,
}

impl ActivateHeadView for SimHead<'_> {
    fn is_type(&mut self, id: i32) -> bool {
        let ident = match id {
            0x1b3 => Ident::Library,
            0x213 => Ident::ForbiddenCity,
            _ => unreachable!("the head asks for 0x1b3 and 0x213 only"),
        };
        self.sim.building_is(self.b, ident)
    }
    /// SEAM: see the module's "not established".
    fn new_library(&mut self) {}
    /// `Sim::activate` clears it with the rest of the economy's reset
    /// ahead of the head (the order is unobserved: nothing between reads it).
    fn clear_recharging(&mut self) {
        self.sim.buildings[self.b].recharging = 0;
    }
    fn city_flag(&mut self) -> bool {
        self.sim.building_is_city(self.b)
    }
    /// SEAM: the count is bumped and dropped again around the conversion;
    /// nothing between reads it here (`mask_me` is the city arm's).
    fn cities_add(&mut self, _d: i32) {}
    fn tribe_bonus(&mut self, bonus: i32) -> bool {
        debug_assert_eq!(bonus, 0xe, "the head asks the Chinese bonus only");
        self.sim.nation[self.who as usize].chinese
    }
    fn chinese_large_cities(&mut self) -> bool {
        self.sim.tuning.chinese_large_cities != 0
    }
    fn set_type(&mut self, id: i32) {
        debug_assert_eq!(id, 0x19f, "the head converts to a Town");
        if let Some(town) = self
            .sim
            .build_types
            .iter()
            .position(|t| t.ident == Ident::Town)
        {
            let b = self.b;
            self.sim.buildings[b].ty = Some(town);
            self.sim.buildings[b].combat =
                Some(self.sim.build_types[town].combat.unwrap_or_default());
        }
    }
    /// SEAM: the city arm's `mask_city` masks the converted centre.
    fn mask_me(&mut self) {}
    fn flag_wall_stats(&mut self) {
        self.sim.wall_stats_dirty[self.who as usize] = true;
    }
    /// SEAM: nothing here reads the tables.
    fn record_last_built(&mut self) {}
    fn wall_activate(&mut self, _captured: i32, _announce: i32) {}
}
