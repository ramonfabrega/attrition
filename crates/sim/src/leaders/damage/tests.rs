use super::{ATTRITION_WINDOW, Damage, LEAVES, STAMP_WINDOW, past};

/// Every window here is strict: exactly 300 (or 900) frames since the
/// stamp does not re-arm it, and a stamp of 0 opens only past frame 300
/// (`docs/LEADERS.md` §4.2, §5.2).
#[test]
fn the_windows_are_strict() {
    assert!(!past(300, 0, STAMP_WINDOW));
    assert!(past(301, 0, STAMP_WINDOW));
    assert!(!past(1200, 300, ATTRITION_WINDOW));
    assert!(past(1201, 300, ATTRITION_WINDOW));
}

/// The record starts clear, and every leaf it names is answered once.
#[test]
fn the_record_starts_clear() {
    let d = Damage::default();
    assert_eq!(d.attack_stamp, [0; 8]);
    assert_eq!((d.units_killed, d.units_lost), (0, 0));
    let mut names = LEAVES.to_vec();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), LEAVES.len(), "no leaf twice");
}
