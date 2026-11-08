//! `WCoord::traverse_x@0046eef0` under the emulator against the port (item 1971; hand-written on tools/emu/hooks.py).

/// `WCoord::traverse_x@0046eef0`, run by `tools/emu/sweep_wcoord_traverse_x.py` under unicorn
/// (tools/emu): `value * 4 + p % 4` stored through the out-pointer, wrapping at 32 bits.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wcoord_traverse_x.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (v, p) = (a[0] as i32, a[1] as i32);
        let got = i64::from(v.wrapping_mul(4).wrapping_add(p % 4));
        assert_eq!(got, want, "traverse_x — row `{line}`");
        n += 1;
    }
    assert!(n >= 300, "only {n} rows");
    eprintln!("WCoord::traverse_x@0046eef0: {n} rows agree");
}
