//! `Unit::kill_current_path@005e31d0` under the emulator against the port (item 2038; hand-written on tools/emu/hooks.py).

use crate::Unit;
use crate::orders::PathData;
use crate::world::Pos;

/// `Unit::kill_current_path@005e31d0`, run by `tools/emu/sweep_unit_kill_current_path.py` under unicorn
/// (tools/emu): a path of length zero or below is left alone and nothing is called; otherwise waypoints are
/// popped through the first final one (bit 0 of the entry's flag byte) or to the bottom, and the one callee —
/// the partial path's clear, hooked — runs once on `this`. The port pops a [`Unit`]'s own stack with
/// `discard_current_path_segment`, which `orders.rs`'s `kill_current_path` calls under the same guard.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_kill_current_path.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (length, calls, ecx_this, flags) = (a[0], a[1], a[2], &a[3..]);
        assert_eq!(ecx_this, 1, "the callee's ecx is this — row `{line}`");
        let mut unit = Unit::new(0, 0, Pos::new(0, 0), 1);
        unit.path
            .extend(flags[..length.max(0) as usize].iter().map(|&f| PathData {
                to: Pos::new(0, 0),
                tolerance: 0,
                flags: f as u8,
            }));
        let got = if unit.path.is_empty() {
            length
        } else {
            unit.discard_current_path_segment();
            unit.path.len() as i64
        };
        assert_eq!(
            calls,
            i64::from(length > 0),
            "the clear runs on a nonempty path — row `{line}`"
        );
        assert_eq!(got, want, "kill_current_path — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Unit::kill_current_path@005e31d0: {n} rows agree");
}
