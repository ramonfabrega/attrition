//! `cosx@0092d0c0` against this crate's [`crate::movement::cos_component`].

use crate::movement::{Angle, cos_component};
use std::panic::catch_unwind;

/// The inputs where the port and the original part: `(angle, distance, ours,
/// the original's)`, `ours` `None` where the port panics.
///
/// The first is `(0x40000000, i32::MIN)`: `cosx` adds a quarter turn, which
/// lands on a negative angle, so the distance is negated, and `-i32::MIN` is
/// an overflow panic in this crate (`sin_component`, `movement.rs`; the
/// release profile keeps `overflow-checks`) where the original's `neg` wraps
/// and goes on to answer. Every parting row is that one cause: distance
/// `i32::MIN` with a quarter-turned angle that is negative. No game distance
/// is that large — recorded, not fixed (the sweep does not change the port).
const PARTS: &[(i32, i32, Option<i32>, i32)] = &[
    (0x4000_0000, i32::MIN, None, 0),
    (0x4000_0001, i32::MIN, None, 0),
    (0x5555_5555, i32::MIN, None, -32768),
    (0x7fff_ffff, i32::MIN, None, 0),
    (i32::MIN, i32::MIN, None, 0),
    (-0x7fff_ffff, i32::MIN, None, 0),
    (-0x4000_0001, i32::MIN, None, 0),
    (-0x4100_0000, i32::MIN, None, -32768),
    (0x7fc0_0000, i32::MIN, None, -32768),
    (0x6000_0000, i32::MIN, None, 0),
    (-0x6000_0000, i32::MIN, None, -32768),
    (0x7fff_ff80, i32::MIN, None, -32768),
];

/// `cosx@0092d0c0` run under the emulator by `tools/emu/sweep_cosx.py` (after
/// `trig_init@00a46980` has built the original's own `sine_table`), against
/// [`cos_component`], row by row: `<angle> <distance> -> <result>`. Every
/// row agrees except those of [`PARTS`], which are asserted as parting.
#[test]
fn the_emulated_cosx_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_cosx.py") else {
        return;
    };
    let (mut n, mut parted) = (0, 0);
    for line in rows.lines() {
        let (args, want) = super::row(line);
        let (angle, dist) = (args[0] as i32, args[1] as i32);
        let ours = catch_unwind(|| cos_component(Angle(angle), dist)).ok();
        n += 1;
        match PARTS.iter().find(|p| p.0 == angle && p.1 == dist) {
            Some(&(_, _, theirs, orig)) => {
                assert_eq!(i64::from(orig), want, "PARTS row `{line}`: the original's");
                assert_eq!(ours, theirs, "PARTS row `{line}`: ours");
                assert_ne!(ours, Some(orig), "PARTS row `{line}` no longer parts");
                parted += 1;
            }
            None => assert_eq!(
                ours.map(i64::from),
                Some(want),
                "row `{line}`: ours vs the original's {want}"
            ),
        }
    }
    assert!(n >= 200, "only {n} rows");
    assert_eq!(parted, PARTS.len(), "every PARTS row is in the sweep");
}
