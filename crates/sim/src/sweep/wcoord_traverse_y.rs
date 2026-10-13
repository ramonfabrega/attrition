//! `WCoord::traverse_y@0046eed0` under the emulator against the port (item 1856).

/// `WCoord::traverse_y@0046eed0`, run by `tools/emu/sweep_wcoord_traverse_y.py` under unicorn
/// (tools/emu): `__thiscall`, two stack arguments (`ret 8`): the out pointer, then `y`; stores `((y + ((y >> 31) & 3)) >> 2) + [this] * 4`. Row `<value> <y> -> <stored word>`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wcoord_traverse_y.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let _ = &a;
        let got: i64 = (((a[1] as i32).wrapping_add(((a[1] as i32) >> 31) & 3)) >> 2)
            .wrapping_add((a[0] as i32).wrapping_mul(4)) as i64;
        assert_eq!(got, want, "WCoord::traverse_y — row `{line}`");
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("WCoord::traverse_y@0046eed0: {n} rows agree");
}
