//! `PathFinder::valid_tcoord@00687d60` under the emulator against the port (item 1998; hand-written on tools/emu/hooks.py).

/// `PathFinder::valid_tcoord@00687d60`, run by `tools/emu/sweep_path_finder_valid_tcoord.py` under unicorn
/// (tools/emu): `x` and `y` through `div_3_table` at `c >> 6`, handed to the unit's `invalid_loc` (hooked, so not backed here) on the
/// unit at `this+0x54` with `(tx, ty, 0, 1, 1, 1, 0, this)`; 1 when that answers 0. `path.rs`'s `valid_tcoord` is
/// the port's probe; the row holds the call it makes and the answer's sense.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_path_finder_valid_tcoord.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (tx, ty, ans) = (a[2], a[3], a[4]);
        assert_eq!(a[5], 1, "the callee's ecx is the unit — row `{line}`");
        assert_eq!(
            &a[6..14],
            &[tx, ty, 0, 1, 1, 1, 0, 1],
            "the arguments — row `{line}`"
        );
        let got = i64::from(ans == 0);
        assert_eq!(got, want, "valid_tcoord — row `{line}`");
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("PathFinder::valid_tcoord@00687d60: {n} rows agree");
}
