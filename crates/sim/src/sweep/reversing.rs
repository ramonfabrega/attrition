//! `reversing@0092cf20` against the emulated original (item 1575's sweep).

use crate::group::reversing;
use crate::movement::Angle;

/// `reversing@0092cf20`, run by `tools/emu/sweep_reversing.py` under
/// unicorn on the install's executable, against [`reversing`]: the edges of
/// both compares and a seeded random set over the whole wrapped-angle range.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_reversing.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines() {
        let (args, want) = super::row(line);
        let d = args[0] as i32;
        assert_eq!(
            i64::from(reversing(Angle(d))),
            want,
            "reversing@0092cf20 parts on row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "the sweep printed {n} rows");
}
