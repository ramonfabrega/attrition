//! **`ObjectData::get_damage@00644130`** as steps over a view of the facts
//! it asks (item 3356; the sweep `sweep/object_data_get_damage.rs`).
//!
//! The formula is `docs/COMBAT.md` section 6, and [`crate::combat::get_damage_checked`]
//! is its entry for the callers that hold a [`crate::combat::Profile`] and two
//! [`crate::combat::Side`]s. This module is the same twenty-eight steps with
//! the questions each asks pulled out into [`DamageFacts`], in the order the
//! original asks them and with its short circuits: the original reaches its
//! answers through forty virtual calls and a dozen globals, and a lane that
//! lays the emulated original's records under a view answers each question
//! from the record it names. [`Reduced`] is the view a `Profile` and a pair of
//! `Side`s make, which is what every caller of the crate has.
//!
//! Arithmetic is the original's own 32-bit: a product wraps, a quotient
//! truncates, and `/ 256` is the compiler's signed shift.

use crate::attrition::Domain;
use crate::combat::{Modifiers, Profile, Side, mask, role, shr8};
use crate::movement::Angle;
use crate::tuning::Tuning;

/// The rules record's fields the formula reads (`GameAccessConst::constantsc`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rule {
    RedFortAirDefense,
    WellingtonSiegeAttack,
    JapaneseDamage,
    RiverModifier,
    FlankBonus,
    CavalryFlankBonus,
    VehicleFlankBonus,
    RockyModifier,
    OverkillFrames,
    OverkillDamage,
    EntrenchmentModifier,
    AntipaterEntrenchBonus,
    HeightIncrement,
    HeightBonus,
    RussianCossackDamage,
    SuperImmune,
    RecaptureCityModifier,
}

/// What `get_damage` asks of the two objects, in the words the original
/// uses for the source: `attacker` is `this` (the `ObjectData`), `attacker_obj`
/// the `Object` the attacker's player and index name, `target` the `Object`
/// the first two arguments name and `target_unit` its `UnitData`. A method
/// that reads nothing but memory has no order; one that calls a virtual or a
/// callee is asked where the original asks it, so a view that answers each
/// call from a queue sees the original's own order.
pub trait DamageFacts {
    fn rule(&mut self, r: Rule) -> i32;
    /// The combat table entry, `table[attacker_obj type * 0x1ed + target type]`.
    fn pct(&mut self) -> i32;
    /// `target.armor()` (slot `0x124`).
    fn target_armor(&mut self) -> i32;
    /// `attacker.attack()` (slot `0x120`).
    fn attacker_attack(&mut self) -> i32;
    /// The attacker's and the target's type masks (`+0x1e4`).
    fn attacker_mask(&mut self) -> u32;
    fn target_mask(&mut self) -> u32;

    /// Step 1: the attacker's type is `0x1bb`/`0x1bc` or its flags carry `0x20`.
    fn attacker_fort_or_city(&mut self) -> bool;
    /// Step 1: the Gunpowder test on the attacker's leader.
    fn gunpowder(&mut self) -> bool;

    fn attacker_obj_is_build(&mut self) -> bool;
    fn attacker_is_build(&mut self) -> bool;
    fn target_is_unit(&mut self) -> bool;
    fn target_packs(&mut self) -> bool;
    fn target_packed(&mut self) -> bool;
    fn target_attacks(&mut self) -> bool;
    fn target_is_moving(&mut self) -> bool;
    /// `target.type.is_siege()` (slot `0x10c`).
    fn target_type_siege(&mut self) -> bool;
    fn target_is_supply(&mut self) -> bool;
    fn target_is_caravan(&mut self) -> bool;
    fn target_is_build(&mut self) -> bool;
    /// `target.get_build()` is active (`WallData::is_active`).
    fn target_build_active(&mut self) -> bool;
    /// The attacker's type is one of the four citizens.
    fn attacker_citizen(&mut self) -> bool;
    fn attacker_militia(&mut self) -> bool;
    fn tile_owned_by_attacker(&mut self) -> bool;
    fn attacker_v2rocket(&mut self) -> bool;
    fn attacker_is_unit(&mut self) -> bool;
    /// `+0x218` of the attacker's and of the target's type: land 0, sea 1, air 2.
    fn attacker_domain(&mut self) -> i32;
    fn target_domain(&mut self) -> i32;
    fn target_redfort(&mut self) -> bool;
    fn attacker_is_building(&mut self) -> bool;
    /// `BuildData::get_shot` of the attacker's building is 4 (the redoubt).
    fn attacker_shot_redoubt(&mut self) -> bool;
    fn wellington_aura(&mut self) -> bool;
    fn target_factory_made(&mut self) -> bool;
    fn attacker_obj_is_unit(&mut self) -> bool;
    fn attacker_barracks_made(&mut self) -> bool;
    /// `Some(min(ages, military level))` when the attacker's leader has the Japanese bonus.
    fn japanese_level(&mut self) -> Option<i32>;
    fn target_unit_mask_400000(&mut self) -> bool;
    fn target_unit_domain(&mut self) -> i32;
    fn team_untargeted(&mut self) -> bool;
    fn target_uber_size(&mut self) -> i32;
    fn splash_percent(&mut self) -> i32;
    fn target_unit_type_siege(&mut self) -> bool;
    fn target_bark(&mut self) -> bool;
    fn target_unit_z(&mut self) -> i32;
    fn target_decoy(&mut self) -> bool;
    /// `ObjectData::attack` (the direct call, not the slot).
    fn attack_stat(&mut self) -> i32;
    fn target_is_building(&mut self) -> bool;
    fn target_unfriendly(&mut self) -> bool;
    fn target_wall_is_build(&mut self) -> bool;
    fn target_type_attack_zero(&mut self) -> bool;
    /// 0 for none, 1 or 2.
    fn flank_level(&mut self, angle: i32) -> i32;
    fn attacker_is_ranged(&mut self) -> bool;
    fn frame(&mut self) -> i32;
    fn target_damage_frame(&mut self) -> i32;
    fn attacker_captain(&mut self) -> i32;
    fn target_damage_o(&mut self) -> i32;
    fn target_catapult(&mut self) -> bool;
    fn attacker_type_siege(&mut self) -> bool;
    fn target_rocky(&mut self) -> bool;
    fn attacker_z(&mut self) -> i32;
    fn target_z(&mut self) -> i32;
    fn attacker_obj_type_siege(&mut self) -> bool;
    fn target_entrenched(&mut self) -> bool;
    fn attacker_obj_flamethrower(&mut self) -> bool;
    fn trench_facing(&mut self) -> i32;
    fn antipater_entrenched(&mut self) -> bool;
    fn russian(&mut self) -> bool;
    fn attacker_stable_made(&mut self) -> bool;
    fn target_type_supercollider(&mut self) -> bool;
    fn recapturable_city(&mut self) -> bool;
}

/// The signed `/ 2` the compiler writes: the quotient truncates toward zero.
const fn half(v: i32) -> i32 {
    v / 2
}

/// One call of `ObjectData::get_damage`. `angle` is its third argument,
/// `splash` the fourth, `check_overkill` the fifth and `dtype` the sixth
/// (`None` for a null out-pointer); the answer is the net hit in whole hits,
/// before `scale`. `dtype` is left 2, or 3 for an explosive or bombard
/// attacker, or a building whose shot is 4.
pub fn get_damage_steps<F: DamageFacts>(
    f: &mut F,
    angle: i32,
    splash: bool,
    check_overkill: bool,
    mut dtype: Option<&mut i32>,
) -> i32 {
    let pct = f.pct();
    if let Some(d) = dtype.as_deref_mut() {
        *d = 2;
    }
    let mut armor = f.target_armor();
    let attack = f.attacker_attack();
    let mut amask = f.attacker_mask();
    let tmask = f.target_mask();

    // 1. Gunpowder arrows.
    if f.attacker_fort_or_city() && f.gunpowder() {
        amask = (amask & !mask::ARCHERY) | mask::GUN;
    }
    // 2.
    let mut base = attack.wrapping_mul(pct) / 100;
    if amask & mask::MUSKET_INF != 0 {
        armor = armor.wrapping_mul(133) / 100;
    }
    // 4.
    if f.attacker_obj_is_build()
        && tmask & mask::SIEGE != 0
        && f.target_is_unit()
        && f.target_packs()
        && !f.target_packed()
    {
        base /= 3;
    }
    // 5.
    if f.attacker_is_build() && f.target_is_unit() && f.target_attacks() && f.target_is_moving() {
        if tmask & mask::FOOT == 0 {
            if f.target_type_siege() || f.target_is_supply() || f.target_is_caravan() {
                base = half(base);
            }
        } else {
            base = base.wrapping_mul(3) / 4;
        }
    }
    // 6.
    if f.attacker_is_build() && f.target_is_build() && !f.target_build_active() {
        base = base.wrapping_shl(2);
    }
    // 7.
    if f.target_is_build()
        && (f.attacker_citizen() || f.attacker_militia())
        && !f.tile_owned_by_attacker()
    {
        base = half(base);
    }
    // 8.
    if f.attacker_v2rocket() && f.target_is_build() && !f.target_build_active() {
        base = half(base);
    }
    // 9.
    if f.attacker_is_unit()
        && f.attacker_domain() == 2
        && amask & mask::MISSILE == 0
        && f.target_redfort()
    {
        base = (100i32.wrapping_sub(f.rule(Rule::RedFortAirDefense))).wrapping_mul(base) / 100;
    }
    // 10.
    if f.target_is_unit() {
        if f.target_packed() {
            base = base.wrapping_add(base);
        }
        if f.target_packs() && !f.target_packed() {
            armor = armor.wrapping_add(1);
        }
    }
    // 11. The death class.
    if amask & mask::EXPLOSIVE != 0
        && let Some(d) = dtype.as_deref_mut()
    {
        *d = 3;
    }
    if amask & mask::BOMBARD != 0
        && let Some(d) = dtype.as_deref_mut()
    {
        *d = 3;
    }
    if f.attacker_is_building() {
        let redoubt = f.attacker_shot_redoubt();
        if let Some(d) = dtype.as_deref_mut() {
            *d = if redoubt { 3 } else { 2 };
        }
    }
    // 12. Wellington.
    if f.attacker_is_unit() && f.wellington_aura() && f.target_factory_made() {
        base = base.wrapping_add(f.rule(Rule::WellingtonSiegeAttack));
    }
    // 13. Japanese.
    if f.attacker_obj_is_unit()
        && f.attacker_barracks_made()
        && let Some(n) = f.japanese_level()
    {
        let mut j = f.rule(Rule::JapaneseDamage);
        if j < 0 {
            j = j.wrapping_mul(n).wrapping_neg();
        }
        base = j.wrapping_add(100).wrapping_mul(base) / 100;
    }
    // 14. Double damage.
    if (f.attacker_is_build() || (f.attacker_domain() == 0 && f.target_domain() != 0))
        && f.target_is_unit()
        && (f.target_unit_mask_400000() || (f.target_unit_domain() == 1 && f.team_untargeted()))
    {
        base = base.wrapping_add(base);
    }
    // 15. Splash.
    if splash {
        if f.target_is_unit() {
            base /= f.target_uber_size();
        }
        base = f.splash_percent().wrapping_mul(base) / 100;
        if f.target_is_unit() {
            if f.target_unit_type_siege() && f.target_packs() && !f.target_packed() {
                base = base.wrapping_mul(3);
            }
            if f.target_bark() {
                base = base.wrapping_mul(25) / 100;
            }
        }
    }
    // 16. River.
    if f.target_is_unit() && f.target_unit_z() < 0 {
        base = shr8(f.rule(Rule::RiverModifier).wrapping_mul(base));
    }
    // 17. Decoy.
    if f.target_is_unit() && f.target_decoy() {
        armor = 0;
        base = f.attack_stat().max(base).wrapping_mul(1000);
    }
    // 18. Unfriendly territory.
    if f.target_is_building() && f.target_unfriendly() {
        armor = 0;
        if !f.target_wall_is_build() || f.target_type_attack_zero() {
            base = base.wrapping_shl(2);
        }
    }
    // 19. Flanking.
    if f.attacker_is_unit()
        && f.target_is_unit()
        && amask & mask::CIVILIAN == 0
        && tmask & mask::CIVILIAN == 0
        && amask & mask::AIR == 0
        && tmask & mask::AIR == 0
        && (amask & mask::NAVAL) == (tmask & mask::NAVAL)
        && tmask & mask::NAVAL == 0
    {
        let level = f.flank_level(angle);
        if level != 0 {
            // The reduction is keyed on the target's mask (item 601, `docs/COMBAT.md` 53).
            let mut fb = f.rule(Rule::FlankBonus);
            if tmask & mask::VEHICLE != 0 {
                fb = shr8(f.rule(Rule::VehicleFlankBonus).wrapping_mul(fb));
            } else if tmask & mask::MOUNTED != 0 {
                fb = shr8(f.rule(Rule::CavalryFlankBonus).wrapping_mul(fb));
            }
            base = fb.wrapping_mul(level).wrapping_add(100).wrapping_mul(base) / 100;
        }
    }
    // 20. Net damage.
    let mut dmg = (base.wrapping_add(5) / 10).wrapping_sub(armor);
    // 21. Overkill.
    if check_overkill && f.attacker_is_ranged() && f.attacker_is_unit() && f.target_is_unit() {
        let df = f.target_damage_frame();
        if df != 0
            && f.frame().wrapping_sub(df) < f.rule(Rule::OverkillFrames)
            && f.attacker_captain() != f.target_damage_o()
        {
            dmg = shr8(dmg.wrapping_mul(f.rule(Rule::OverkillDamage)));
            if f.target_catapult() && !f.attacker_type_siege() {
                dmg = half(dmg);
            }
        }
    }
    // 22. Rocky.
    if tmask & (mask::LIGHT_INF | mask::MODERN_INF | mask::MUSKET_INF) != 0 && f.target_rocky() {
        dmg = shr8(dmg.wrapping_mul(f.rule(Rule::RockyModifier)));
    }
    // 23. Height.
    let az = f.attacker_z();
    let tz = f.target_z();
    if f.target_domain() != 2 && f.attacker_domain() != 2 && !f.attacker_obj_type_siege() && tz < az
    {
        let num = az
            .wrapping_sub(tz)
            .wrapping_mul(f.rule(Rule::HeightBonus))
            .wrapping_mul(dmg);
        dmg = dmg.wrapping_add(num / f.rule(Rule::HeightIncrement).wrapping_mul(100));
    }
    // 24. Entrenchment.
    if f.target_is_unit() && f.target_entrenched() && !f.attacker_obj_flamethrower() {
        let d = f.trench_facing().wrapping_sub(angle) as u32;
        let c = if d.wrapping_add(0x5555_5556) < 0xaaaa_aaac {
            1 + u32::from(d.wrapping_add(0x2000_0000) > 0x4000_0000)
        } else {
            0
        };
        if splash || c == 0 {
            dmg = shr8(f.rule(Rule::EntrenchmentModifier).wrapping_mul(dmg));
            if f.antipater_entrenched() {
                dmg = shr8(f.rule(Rule::AntipaterEntrenchBonus).wrapping_mul(dmg));
            }
        }
    }
    // 25. Cossacks.
    if f.rule(Rule::RussianCossackDamage) != 0
        && f.russian()
        && f.attacker_is_unit()
        && f.attacker_stable_made()
        && (f.target_is_supply() || f.target_type_siege())
    {
        dmg = f
            .rule(Rule::RussianCossackDamage)
            .wrapping_add(100)
            .wrapping_mul(dmg)
            / 100;
    }
    // 26. At least one.
    if dmg < 1
        && (f.attacker_domain() != 0 || f.target_domain() != 1)
        && (tmask & mask::AIR) == ((amask >> 3) & mask::AIR)
        && !splash
    {
        dmg = 1;
    }
    // 27.
    if f.target_type_supercollider() && f.attacker_domain() == 2 && f.rule(Rule::SuperImmune) != 0 {
        dmg = 0;
    }
    // 28. Recapture.
    if f.target_is_build() && f.target_build_active() && f.recapturable_city() {
        dmg = shr8(f.rule(Rule::RecaptureCityModifier).wrapping_mul(dmg));
    }
    dmg
}

/// The view a [`Profile`] and two [`Side`]s make: every fact the original
/// reads from a virtual or a record is a field the crate already carries.
pub struct Reduced<'a> {
    pub t: &'a Tuning,
    pub a: &'a Profile,
    pub at: Side,
    pub tp: &'a Profile,
    pub tt: Side,
    pub attack: i32,
    pub armor: i32,
    pub pct: i32,
    pub frame: i64,
    pub m: &'a Modifiers,
}

const fn domain(d: Domain) -> i32 {
    match d {
        Domain::Land => 0,
        Domain::Sea => 1,
        Domain::Air => 2,
    }
}

impl DamageFacts for Reduced<'_> {
    fn rule(&mut self, r: Rule) -> i32 {
        let t = self.t;
        match r {
            Rule::RedFortAirDefense => t.red_fort_air_defense,
            Rule::WellingtonSiegeAttack => t.wellington_siege_attack,
            Rule::JapaneseDamage => t.japanese_damage,
            Rule::RiverModifier => t.river_modifier,
            Rule::FlankBonus => t.flank_bonus,
            Rule::CavalryFlankBonus => t.cavalry_flank_bonus,
            Rule::VehicleFlankBonus => t.vehicle_flank_bonus,
            Rule::RockyModifier => t.rocky_modifier,
            Rule::OverkillFrames => t.overkill_frames,
            Rule::OverkillDamage => t.overkill_damage,
            Rule::EntrenchmentModifier => t.entrenchment_modifier,
            Rule::AntipaterEntrenchBonus => t.antipater_entrench_bonus,
            Rule::HeightIncrement => t.height_increment,
            Rule::HeightBonus => t.height_bonus,
            Rule::RussianCossackDamage => t.russian_cossack_damage,
            Rule::SuperImmune => t.super_immune,
            Rule::RecaptureCityModifier => t.recapture_city_modifier,
        }
    }
    fn pct(&mut self) -> i32 {
        self.pct
    }
    fn target_armor(&mut self) -> i32 {
        self.armor
    }
    fn attacker_attack(&mut self) -> i32 {
        self.attack
    }
    fn attacker_mask(&mut self) -> u32 {
        self.a.obj_masks
    }
    fn target_mask(&mut self) -> u32 {
        self.tp.obj_masks
    }
    fn attacker_fort_or_city(&mut self) -> bool {
        self.a.is(role::FORT) || self.a.is(role::CITY)
    }
    fn gunpowder(&mut self) -> bool {
        self.m.gunpowder
    }
    fn attacker_obj_is_build(&mut self) -> bool {
        self.at.build_proper
    }
    fn attacker_is_build(&mut self) -> bool {
        self.at.build_proper
    }
    fn target_is_unit(&mut self) -> bool {
        self.tt.unit
    }
    fn target_packs(&mut self) -> bool {
        self.tp.packs
    }
    fn target_packed(&mut self) -> bool {
        self.tt.packed
    }
    fn target_attacks(&mut self) -> bool {
        self.tt.attacks
    }
    fn target_is_moving(&mut self) -> bool {
        self.tt.moving
    }
    fn target_type_siege(&mut self) -> bool {
        self.tp.siege
    }
    fn target_is_supply(&mut self) -> bool {
        self.tp.is(role::SUPPLY)
    }
    fn target_is_caravan(&mut self) -> bool {
        self.tp.is(role::CARAVAN)
    }
    fn target_is_build(&mut self) -> bool {
        self.tt.build_proper
    }
    fn target_build_active(&mut self) -> bool {
        !self.tt.under_construction
    }
    fn attacker_citizen(&mut self) -> bool {
        self.a.is(role::CITIZEN)
    }
    fn attacker_militia(&mut self) -> bool {
        self.a.is(role::MILITIA)
    }
    fn tile_owned_by_attacker(&mut self) -> bool {
        self.tt.tile_owned_by_attacker
    }
    fn attacker_v2rocket(&mut self) -> bool {
        self.a.is(role::V2ROCKET)
    }
    fn attacker_is_unit(&mut self) -> bool {
        self.at.unit
    }
    fn attacker_domain(&mut self) -> i32 {
        domain(self.a.domain)
    }
    fn target_domain(&mut self) -> i32 {
        domain(self.tp.domain)
    }
    fn target_redfort(&mut self) -> bool {
        self.tp.is(role::REDFORT)
    }
    fn attacker_is_building(&mut self) -> bool {
        self.at.building
    }
    fn attacker_shot_redoubt(&mut self) -> bool {
        // SEAM: `BuildData::get_shot` is not modelled; no caller reads the death class.
        false
    }
    fn wellington_aura(&mut self) -> bool {
        self.m.wellington
    }
    fn target_factory_made(&mut self) -> bool {
        self.tp.is(role::FACTORY_MADE)
    }
    fn attacker_obj_is_unit(&mut self) -> bool {
        self.at.unit
    }
    fn attacker_barracks_made(&mut self) -> bool {
        self.a.is(role::BARRACKS_MADE)
    }
    fn japanese_level(&mut self) -> Option<i32> {
        self.m.japanese_level
    }
    fn target_unit_mask_400000(&mut self) -> bool {
        self.tt.mask_400000
    }
    fn target_unit_domain(&mut self) -> i32 {
        domain(self.tp.domain)
    }
    fn team_untargeted(&mut self) -> bool {
        self.tt.team_untargeted_ship
    }
    fn target_uber_size(&mut self) -> i32 {
        self.tp.uber_size.max(1)
    }
    fn splash_percent(&mut self) -> i32 {
        self.a.splash_percent
    }
    fn target_unit_type_siege(&mut self) -> bool {
        self.tp.siege
    }
    fn target_bark(&mut self) -> bool {
        self.tp.is(role::BARK)
    }
    fn target_unit_z(&mut self) -> i32 {
        self.tt.z
    }
    fn target_decoy(&mut self) -> bool {
        self.tt.decoy
    }
    fn attack_stat(&mut self) -> i32 {
        self.attack
    }
    fn target_is_building(&mut self) -> bool {
        self.tt.building
    }
    fn target_unfriendly(&mut self) -> bool {
        self.tt.in_unfriendly_territory
    }
    fn target_wall_is_build(&mut self) -> bool {
        self.tt.build_proper
    }
    fn target_type_attack_zero(&mut self) -> bool {
        self.tp.attack == 0
    }
    fn flank_level(&mut self, angle: i32) -> i32 {
        crate::combat::flank_level(self.tt.facing, Angle(angle)).unwrap_or(0)
    }
    fn attacker_is_ranged(&mut self) -> bool {
        self.a.is_ranged()
    }
    fn frame(&mut self) -> i32 {
        self.frame as i32
    }
    fn target_damage_frame(&mut self) -> i32 {
        self.tt.damage_frame as i32
    }
    fn attacker_captain(&mut self) -> i32 {
        self.at.captain
    }
    fn target_damage_o(&mut self) -> i32 {
        self.tt.damage_o
    }
    fn target_catapult(&mut self) -> bool {
        self.tp.is(role::CATAPULT)
    }
    fn attacker_type_siege(&mut self) -> bool {
        self.a.siege
    }
    fn target_rocky(&mut self) -> bool {
        self.tt.rocky
    }
    fn attacker_z(&mut self) -> i32 {
        self.at.z
    }
    fn target_z(&mut self) -> i32 {
        self.tt.z
    }
    fn attacker_obj_type_siege(&mut self) -> bool {
        self.a.siege
    }
    fn target_entrenched(&mut self) -> bool {
        self.tt.entrenched
    }
    fn attacker_obj_flamethrower(&mut self) -> bool {
        self.a.is(role::FLAMETHROWER)
    }
    fn trench_facing(&mut self) -> i32 {
        self.tt.trench_facing.0
    }
    fn antipater_entrenched(&mut self) -> bool {
        self.tt.antipater_entrenched
    }
    fn russian(&mut self) -> bool {
        self.m.russian
    }
    fn attacker_stable_made(&mut self) -> bool {
        self.a.is(role::STABLE_MADE)
    }
    fn target_type_supercollider(&mut self) -> bool {
        self.tp.is(role::SUPERCOLLIDER)
    }
    fn recapturable_city(&mut self) -> bool {
        self.tt.recapturable_city
    }
}
