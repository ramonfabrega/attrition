use super::{bins_of, step_bin};

/// `track_queued`'s step 5: a negative step is skipped only at exactly 0,
/// so a counter from `0 < m < n` taken down by `n` goes below zero and a
/// `-1` keeps it falling (`docs/LEADERS.md` §14.2's corrected floor).
#[test]
fn a_bin_skips_a_negative_step_only_at_zero() {
    let mut c = 0;
    step_bin(&mut c, -1);
    assert_eq!(c, 0, "skipped at zero");
    step_bin(&mut c, 2);
    assert_eq!(c, 2);
    step_bin(&mut c, -3);
    assert_eq!(c, -1, "no clamp");
    step_bin(&mut c, -1);
    assert_eq!(c, -2, "a negative counter keeps falling");
}

/// `track_queued`'s step 4, by equality on the trainer: the Barracks and
/// the Stable step combat too; the Auto Plant `0x1ad`, the `FACTORY`
/// `0x1af` and the other docks fall through to the air test.
#[test]
fn the_bins_are_the_trainer_s_by_equality() {
    let b = |t: i64, air: bool| bins_of(Some(t), air);
    assert_eq!(b(0x1ab, false), [true, false, false, true, false, false]);
    assert_eq!(b(0x1ac, true), [false, true, false, true, false, false]);
    assert_eq!(b(0x1ae, false), [false, false, true, false, false, false]);
    assert_eq!(b(0x1b0, false), [false, false, false, false, true, false]);
    for t in [0x1ad, 0x1af, 0x1b1, 0x1b2] {
        assert_eq!(b(t, false), [false; 6], "{t:#x}");
        assert_eq!(b(t, true), [false, false, false, false, false, true]);
    }
    assert_eq!(bins_of(None, true)[5], true);
}
