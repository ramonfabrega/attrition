//! `GoodData::get_gpiece@0066e5b0` under the emulator against the port (item 1824).

/// `GoodData::get_gpiece@0066e5b0`, run by `tools/emu/sweep_good_data_get_gpiece.py` under unicorn
/// (tools/emu): `__thiscall`, `ecx` = the good, `+0x18` its type record, whose dword at `+4` plus the `first_good_piece` (`+0xd58`) of the structure the global cell `0xc06214` points to: a wrapping 32-bit sum.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_good_data_get_gpiece.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let got = i64::from((a[0] as i32).wrapping_add(a[1] as i32));
        assert_eq!(got, want, "GoodData::get_gpiece — row `{line}`");
        n += 1;
    }
    assert!(n >= 60, "only {n} rows");
    eprintln!("GoodData::get_gpiece@0066e5b0: {n} rows agree");
}
