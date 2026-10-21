//! **`HeroData::get_radius@00739e50`** (item 3396): a hero's aura radius, as a
//! walk over a [`HeroRadiusView`] that hands the numbers to the crate's own
//! [`crate::supply::general_radius`].
//!
//! The port of the arithmetic is `general_radius`; this module is the original's
//! *order of asking* — `LeaderData::get_general_upgrade`, then `is(0x168, 0)`,
//! `is(0x170, 0)`, `is(0x176, 0)`, `LeaderData::has_wonder(0x211)`, then the
//! military patriot set `is(0x160/0x162/0x164, 1)` (stopping at the first
//! yes) and the economic set `is(0x161/0x163/0x165, 1)` likewise — and the seven
//! constants it reads from `GameAccessConst::constantsc`. [`crate::Sim::hero_radius`]
//! asks the same questions of the unit's type directly and reads Terra Cotta
//! as absent; this walk is standalone (nothing in `Sim` calls it).

use crate::supply::{General, Patriot, general_radius};
use crate::tuning::Tuning;

/// The type words `is` is asked about.
pub const IS_PARMENIO: i32 = 0x168;
pub const IS_WELLINGTON: i32 = 0x170;
pub const IS_KUTOSOV: i32 = 0x176;
pub const IS_MIL: [i32; 3] = [0x160, 0x162, 0x164];
pub const IS_ECON: [i32; 3] = [0x161, 0x163, 0x165];
/// The wonder `has_wonder` is asked about: the Terra Cotta Army.
pub const WONDER_TERRA_COTTA: i32 = 0x211;

/// The seven constants of `constantsc` the function reads.
#[derive(Clone, Copy, Debug)]
pub struct HeroConstants {
    pub general_radius: i32,
    pub parmenio_radius_adjust: i32,
    pub wellington_radius: i32,
    pub kutosov_radius: i32,
    pub terra_cotta_range: i32,
    pub mil_patriot_radius_bonus: i32,
    pub econ_patriot_radius_bonus: i32,
}

/// The state the function reads and the calls it makes.
pub trait HeroRadiusView {
    /// `LeaderData::get_general_upgrade@006e0830` of the hero's leader.
    fn general_upgrade(&mut self) -> i32;
    /// The hero's `ObjectData::is(ty, mode)` (slot `0xb8`, or the type's `0x60`).
    fn is(&mut self, ty: i32, mode: i32) -> bool;
    /// `LeaderData::has_wonder@006ebc10(ty)` of the hero's leader.
    fn has_wonder(&mut self, ty: i32) -> bool;
    /// `GameAccessConst::constantsc`'s seven fields.
    fn constants(&mut self) -> HeroConstants;
}

/// The original's body over a view.
pub fn hero_radius_steps<V: HeroRadiusView>(v: &mut V) -> i32 {
    let upgrades = v.general_upgrade();
    let parmenio = v.is(IS_PARMENIO, 0);
    let wellington = v.is(IS_WELLINGTON, 0);
    let kutosov = v.is(IS_KUTOSOV, 0);
    let terra_cotta = v.has_wonder(WONDER_TERRA_COTTA);
    let mil = v.is(IS_MIL[0], 1) || v.is(IS_MIL[1], 1) || v.is(IS_MIL[2], 1);
    let econ = v.is(IS_ECON[0], 1) || v.is(IS_ECON[1], 1) || v.is(IS_ECON[2], 1);
    let patriot = match (mil, econ) {
        (true, true) => Some(Patriot::Both),
        (true, false) => Some(Patriot::Military),
        (false, true) => Some(Patriot::Economic),
        (false, false) => None,
    };
    let k = v.constants();
    let t = Tuning {
        general_radius: k.general_radius,
        parmenio_radius_adjust: k.parmenio_radius_adjust,
        wellington_radius: k.wellington_radius,
        kutosov_radius: k.kutosov_radius,
        terra_cotta_range: k.terra_cotta_range,
        mil_patriot_radius_bonus: k.mil_patriot_radius_bonus,
        econ_patriot_radius_bonus: k.econ_patriot_radius_bonus,
        ..Tuning::RON
    };
    general_radius(
        &t,
        &General {
            upgrades,
            parmenio,
            wellington,
            kutosov,
            terra_cotta,
            patriot,
        },
    )
}
