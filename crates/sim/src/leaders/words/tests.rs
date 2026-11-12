use super::{anti_attrition, name_step};

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
    name_step(&mut names, &tribes, 0);
    name_step(&mut names, &tribes, 2);
    assert_eq!(names, [1, 0, 1]);
    // Slot 1 shares slot 0's tribe: slot 0 wins the step (2), and slot
    // 1, which had named nothing, takes 1.
    name_step(&mut names, &tribes, 1);
    assert_eq!(names, [2, 1, 1]);
    // Slot 0 leads the tribe, so slot 1's next city steps slot 0.
    name_step(&mut names, &tribes, 1);
    assert_eq!(names, [3, 1, 1]);
    // A tie keeps the lowest slot.
    let mut tied = [2, 2, 0];
    name_step(&mut tied, &tribes, 1);
    assert_eq!(tied, [3, 2, 0]);
}
