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
    assert_eq!(
        relative_value(&horse, &slingers),
        128,
        "light cavalry rides down light foot"
    );
    assert_eq!(relative_value(&slingers, &horse), 0);
    let sum: i32 = [barbarians, bushi, slingers]
        .iter()
        .map(|k| relative_value(&horse, k) - relative_value(k, &horse))
        .sum();
    assert_eq!(
        sum / 3,
        -170,
        "theirs' unit_prod_value, inverted from the offer"
    );
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
