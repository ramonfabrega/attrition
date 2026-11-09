use super::*;

fn at(tolerance: i32, flags: u8) -> PathData {
    PathData {
        to: Pos::new(0, 0),
        tolerance,
        flags,
    }
}

/// The unwind's test, both ends of the range: a tolerance-0 point is
/// kept, `0x60` is unwound, and `0x61` and the world grid's `0x180`
/// are kept.
#[test]
fn a_tolerance_0_waypoint_is_not_loose_and_0x60_is() {
    assert!(!is_loose(&at(0, 0)), "an exact point is kept");
    assert!(is_loose(&at(1, 0)));
    assert!(is_loose(&at(0x60, 0)), "half a tile is loose");
    assert!(!is_loose(&at(0x61, 0)));
    assert!(!is_loose(&at(0x180, 0)));
    assert!(!is_loose(&at(0x60, path_flag::FINAL)), "a final is kept");
    assert!(!is_loose(&at(0x60, 0x20)));
    assert!(!is_loose(&at(-1, 0)), "unsigned: a negative is kept");
}
