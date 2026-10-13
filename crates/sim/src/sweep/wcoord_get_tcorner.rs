//! `WCoord::get_tcorner@0046f080` under the emulator against the port (item 1814).

/// `WCoord::get_tcorner@0046f080`, run by `tools/emu/sweep_wcoord_get_tcorner.py` under unicorn
/// (tools/emu): `__thiscall`, `ecx` = the `WCoord`, an out pointer on the stack: writes `value << 2` and returns the pointer; the printed result is the word written.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wcoord_get_tcorner.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let v = i32::try_from(a[0]).expect("int");
        let got = i64::from(v.wrapping_shl(2));
        assert_eq!(got, want, "get_tcorner — row `{line}`");
        n += 1;
    }
    assert!(n >= 150, "only {n} rows");
    eprintln!("WCoord::get_tcorner@0046f080: {n} rows agree");
}
