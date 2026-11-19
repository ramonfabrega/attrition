use super::{Founding, anti_attrition, founding_name, name_step};

/// The table of `docs/LEADERS.md` §6.2 over the shipped data, bit for
/// bit: the tier down the side, the Mongol bonus and titanium across
/// (neither / one / both), and Liberty or the disabled flag at 0. The
/// two cells one ulp above the nearest single are the order's mark.
#[test]
fn the_nine_patterns_are_the_listing_s_bits() {
    let t = crate::core::tuning::Tuning::RON;
    let rows: [[u32; 3]; 4] = [
        [0x4380_0000, 0x4400_0000, 0x4480_0000],
        [0x43aa_aaab, 0x442a_aaac, 0x44aa_aaac],
        [0x4400_0000, 0x4480_0000, 0x4500_0000],
        [0x4480_0000, 0x4500_0000, 0x4580_0000],
    ];
    for (tier, row) in rows.iter().enumerate() {
        assert_eq!(
            anti_attrition(&t, false, tier, [false; 3]),
            row[0],
            "{tier}"
        );
        assert_eq!(
            anti_attrition(&t, false, tier, [false, true, false]),
            row[1]
        );
        assert_eq!(
            anti_attrition(&t, false, tier, [false, false, true]),
            row[1]
        );
        assert_eq!(anti_attrition(&t, false, tier, [false, true, true]), row[2]);
        assert_eq!(anti_attrition(&t, false, tier, [true, true, true]), 0);
        assert_eq!(anti_attrition(&t, true, tier, [false; 3]), 0);
    }
}

/// The counter's two stores: two slots of one tribe and one of another
/// (`docs/LEADERS.md` §1.2).
#[test]
fn a_tribe_mate_takes_the_step_and_the_owner_takes_one() {
    let tribes = [4, 4, 11];
    let mut names = [0; 3];
    // Distinct tribes, each naming its first city: the owner steps.
    name_step(&mut names, &tribes, 0, None);
    name_step(&mut names, &tribes, 2, None);
    assert_eq!(names, [1, 0, 1]);
    // Slot 1 shares slot 0's tribe: slot 0 wins the step (2), and slot
    // 1, which had named nothing, takes 1.
    name_step(&mut names, &tribes, 1, None);
    assert_eq!(names, [2, 1, 1]);
    // Slot 0 leads the tribe, so slot 1's next city steps slot 0.
    name_step(&mut names, &tribes, 1, None);
    assert_eq!(names, [3, 1, 1]);
    // A tie keeps the lowest slot.
    let mut tied = [2, 2, 0];
    name_step(&mut tied, &tribes, 1, None);
    assert_eq!(tied, [3, 2, 0]);
}

/// **The bleed reads the record** (item 4159's third residue, parked
/// 4162b): `calc_anti_attrition` hands its single to the player's
/// attrition state, which `UnitData::get_attrition` divides by the
/// claimant's strength. Until then the state kept `Resistance::BASE`
/// whatever the leader held, and a disabled leader (anti_att 0.0) still
/// bled.
#[test]
fn the_anti_attrition_single_reaches_the_bleed() {
    use crate::territory::attrition::Resistance;
    let mut sim = crate::Sim::new(crate::core::tuning::Tuning::RON, crate::World::new(8, 8), 2);
    sim.territory.players[1].take_disabled = true;
    for who in 0..2 {
        sim.calc_anti_attrition(who);
    }
    assert_eq!(sim.leader_anti_att(0), super::ANTI_ATT_BASE);
    assert_eq!(
        sim.territory.players[0].resistance,
        Resistance::Single(super::ANTI_ATT_BASE)
    );
    assert_eq!(sim.territory.players[1].resistance, Resistance::Single(0));
    assert!(!sim.territory.players[0].foraging_1);
}

/// **`generate_name`'s three arms** (`City::generate_name@00735c90`,
/// `docs/LEADERS.md` §1.2), each worked by hand from the listing.
#[test]
fn a_founding_name_takes_the_arm_its_counts_name() {
    // The first city of slot 1, the second of its tribe: the slots below
    // it of its tribe, plus one — the list's second name — and the
    // second store.
    let tribes = [4, 4, 9];
    let mut names = [0, 0, 0];
    assert_eq!(
        founding_name(&mut names, &tribes, 0, 29, 7236),
        Some(Founding::Listed(0))
    );
    assert_eq!(names, [1, 0, 0]);
    assert_eq!(
        founding_name(&mut names, &tribes, 1, 29, 7236),
        Some(Founding::Listed(1)),
        "slot 0 holds the tribe's largest count, so it steps; slot 1 had none"
    );
    assert_eq!(names, [2, 1, 0]);
    // Slot 0's next: n = 3, prev 2, the Galois arm. m = 2, M = 28,
    // x = (4 + 7236) mod 28 + 1 = 17; 17 → 176 → 88 → 44 → 22 (kept, the
    // second) → 11 (kept, the third, n): index 11 + 2 = 13.
    assert_eq!(
        founding_name(&mut names, &tribes, 0, 29, 7236),
        Some(Founding::Listed(12))
    );
    assert_eq!(names, [3, 1, 0]);
    // A count reaching the list's length names by number and skips the
    // second store, where the step alone would make it.
    let mut names = [2, 0, 0];
    assert_eq!(
        founding_name(&mut names, &tribes, 1, 3, 7236),
        Some(Founding::Numbered(4))
    );
    assert_eq!(names, [3, 0, 0]);
    let mut stepped = [2, 0, 0];
    name_step(&mut stepped, &tribes, 1, None);
    assert_eq!(stepped, [3, 1, 0]);
}
