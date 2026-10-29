//! **How much one unit type is worth against another** —
//! `UnitType::fill_relative_values@0061be50`, the table
//! `UnitTypeData::relative_value` (`+0x314`, a short per type from `0x32`)
//! that `Types::init` fills once, at load (item 3793, `docs/GOLDEN.md`
//! §150.5).
//!
//! The computer's unit offer reads it: `Leader::unit_prod_value@006cc580`
//! sums `relative_value[t][k] − relative_value[k][t]` over the counted
//! enemy types ([`crate::Sim`]'s `unit_prod_value`). It is **not** the
//! combat table's `return_modifier`, which this crate read in its place
//! until item 3793: a Light Horse against the Barbarians reads 100 − 166
//! there and 0 − 320 here.
//!
//! A pair of combat types starts at 256 — even — and each matchup the two
//! `obj_masks` words describe scales it by a half, a quarter or a factor
//! of two or three; the stored value is `min(w, 768) − 256`, so it runs
//! from −256 to +512. A type that is not a combat type (`role & 0x10000`)
//! is worth nothing against anything, and nothing is worth anything
//! against it.
//!
//! The listing settles the one arm the decompiler's nesting leaves open:
//! the anti-air halving (`0061c26d`–`0061c289`) goes straight to the clamp,
//! so an Anti-Aircraft Gun's third is taken only on the other paths
//! (`0061c28b`).

use crate::combat::mask;

/// What the rule reads of one type: its `obj_masks` (`+0x1e4`), whether it
/// has no range (`max_range`, `+0x1fc`, zero), whether it is a combat type,
/// and the six lineages it tests by `ObjectTypeData::is(x, 0)` — fixed at
/// load, as the original's table is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Facts {
    pub masks: u32,
    pub melee: bool,
    pub combat_role: bool,
    /// `is(MACHINEGUN)`, `0x7b`.
    pub machinegun: bool,
    /// `is(TRIREME)`, `0x154`.
    pub trireme: bool,
    /// `is(BOMBARDSHIP)`, `0x15a`.
    pub bomb_vessel: bool,
    /// `is(FIRERAFT)`, `0x14e`.
    pub fire_raft: bool,
    /// `is(CAMEL)`, `0xd5`.
    pub camel: bool,
    /// `is(AAGUN)`, `0x119`.
    pub aa_gun: bool,
}

const fn half_again(w: u32) -> u32 {
    w * 3 / 2
}

/// `relative_value[a][b]`: what `a` is worth against `b`, as the original
/// stores it.
pub fn relative_value(a: &Facts, b: &Facts) -> i32 {
    if !a.combat_role || !b.combat_role {
        return 0;
    }
    let (m, n) = (a.masks, b.masks);
    let has = |x: u32, bits: u32| x & bits != 0;
    let mut w: u32 = 0x100;

    // Cavalry against what it rides down.
    if has(m, mask::HEAVY_CAV)
        && a.melee
        && has(n, mask::MUSKET_INF | mask::FOOT_ARCHER | mask::LIGHT_INF | mask::WAR_MACHINE)
    {
        w = 0x180;
    }
    if has(m, mask::LIGHT_CAV)
        && a.melee
        && has(n, mask::LIGHT_INF | mask::HORSE_ARCHER | mask::WAR_MACHINE)
    {
        w = half_again(w);
    }
    // Heavy foot and pikes against horses and the large.
    if has(m, mask::HEAVY_INF) && has(n, mask::LARGE | mask::MOUNTED | mask::WAR_MACHINE) {
        w = half_again(w);
    }
    if has(m, mask::PIKE) && has(n, mask::MOUNTED) {
        w = half_again(w);
    }
    // The machine gun and the infantry it mows down.
    if a.machinegun && has(n, mask::MUSKET_INF | mask::MODERN_INF | mask::LIGHT_INF) {
        w = half_again(w);
    }
    if has(m, mask::MUSKET_INF | mask::MODERN_INF) && b.machinegun {
        w /= 2;
    }
    // Ship against ship.
    if has(m & n, mask::NAVAL) {
        w = half_again(w);
        if a.trireme && b.bomb_vessel {
            w = half_again(w);
        }
        if a.fire_raft && !b.fire_raft {
            w = half_again(w);
        }
        if !a.fire_raft && b.fire_raft {
            w /= 2;
        }
        if a.fire_raft {
            w /= 2;
        }
    }
    // The elephant against machines and horses; a camel scares horses too.
    if has(m, mask::ELEPHANT) && has(n, mask::WAR_MACHINE) {
        w = half_again(w);
    }
    if (has(m, mask::ELEPHANT) || a.camel)
        && n & (mask::MOUNTED | mask::ELEPHANT) == mask::MOUNTED
        && !b.camel
    {
        w = half_again(w);
    }
    // Blades and arrows against armour.
    let a_melee_mask = has(m, mask::MELEE);
    if a_melee_mask && has(n, mask::ARMORED) && !has(m, mask::ARCHERY | mask::GUN) {
        w >>= 3;
    }
    if m & (mask::MELEE | mask::LIGHT_INF | mask::ARCHERY) == mask::ARCHERY
        && has(n, mask::ARMORED)
    {
        w /= 2;
    }
    if has(m, mask::FOOT_ARCHER) {
        if has(n, mask::HEAVY_INF) {
            w = half_again(w);
        }
        if has(n, mask::HORSE_ARCHER) {
            w = half_again(w);
        }
        if !a_melee_mask && has(n, mask::LIGHT_INF) {
            w /= 2;
        }
    }
    if has(m, mask::HORSE_ARCHER) {
        if has(n, mask::HEAVY_INF) {
            w = w * 5 / 4;
        }
        if !a_melee_mask && has(n, mask::LIGHT_INF) {
            w = w * 2 / 3;
        }
    }
    if has(m, mask::GUN) {
        if has(n, mask::HEAVY_INF) {
            w *= 2;
        }
        if has(m, mask::FOOT) && has(n, mask::MOUNTED) {
            w = w * 5 / 4;
        }
    }
    if has(m, mask::ARMORPIERCE) && has(n, mask::ARMORED) {
        w = half_again(w);
    }

    // The air: an anti-air flyer against a flyer takes ×3 first; then
    // anything against a non-missile flyer is halved, and an anti-air type
    // takes back ×3 (×6 on a vehicle) of the half. Only the paths that do
    // not halve reach the Anti-Aircraft Gun's third.
    let a_air_aa = has(m, mask::AIR) && !has(m, mask::MISSILE) && has(m, mask::ANTI_AIR);
    let mut aa_third = true;
    if a_air_aa && has(n, mask::AIR) {
        w *= 3;
    }
    if has(n, mask::AIR) && !has(n, mask::MISSILE) {
        let h = w / 2;
        w = h;
        if has(m, mask::ANTI_AIR) {
            w = if has(m, mask::VEHICLE) { h * 6 } else { h * 3 };
        }
        aa_third = false;
    }
    if aa_third && a.aa_gun {
        w /= 3;
    }

    w.min(0x300) as i32 - 0x100
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit(masks: u32, melee: bool) -> Facts {
        Facts {
            masks,
            melee,
            combat_role: true,
            ..Facts::default()
        }
    }

    /// The second melee's Light Horse against the human's three combat
    /// types on 1380 (item 3793, run1465b's `num_units`): the masks
    /// `rondata`'s loader gives TI 209, 154, 158 and 89.
    #[test]
    fn the_light_horse_reads_minus_170_against_the_nubians() {
        let horse = unit(0x2040_1000, true);
        let barbarians = unit(0x4040_00a0, true);
        let bushi = unit(0x4040_00a0, true);
        let slingers = Facts {
            melee: false,
            ..unit(0x0003_0020, false)
        };
        assert_eq!(relative_value(&horse, &barbarians), 0);
        assert_eq!(relative_value(&barbarians, &horse), 320, "×3/2 ×3/2");
        assert_eq!(relative_value(&horse, &slingers), 128, "light cavalry rides down light foot");
        assert_eq!(relative_value(&slingers, &horse), 0);
        let sum: i32 = [barbarians, bushi, slingers]
            .iter()
            .map(|k| relative_value(&horse, k) - relative_value(k, &horse))
            .sum();
        assert_eq!(sum / 3, -170, "theirs' unit_prod_value, inverted from the offer");
    }

    #[test]
    fn a_type_that_is_not_a_combat_type_is_worth_nothing() {
        let citizen = Facts {
            combat_role: false,
            ..unit(0x0040_8024, true)
        };
        let horse = unit(0x2040_1000, true);
        assert_eq!(relative_value(&horse, &citizen), 0);
        assert_eq!(relative_value(&citizen, &horse), 0);
    }

    #[test]
    fn the_value_is_clamped_at_three_times_and_the_halving_skips_the_aa_third() {
        // A heavy-infantry pike against a large mounted war machine:
        // 256 ×3/2 ×3/2 = 576, below the cap.
        let pike = unit(mask::HEAVY_INF | mask::PIKE, true);
        let beast = unit(mask::LARGE | mask::MOUNTED, true);
        assert_eq!(relative_value(&pike, &beast), 320);
        // An Anti-Aircraft Gun (ground, anti-air) against a plane: 256/2 ×3
        // = 384, and no third; against a tank it takes the third.
        let aa = Facts {
            aa_gun: true,
            ..unit(mask::ANTI_AIR, false)
        };
        let plane = unit(mask::AIR, false);
        let tank = unit(mask::ARMORED, true);
        assert_eq!(relative_value(&aa, &plane), 128);
        assert_eq!(relative_value(&aa, &tank), 85 - 256);
    }
}
