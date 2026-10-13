//! `ObjectData::is_active_build@0046fb40` under the emulator against the port (item 1834).

/// `ObjectData::is_active_build@0046fb40`, run by `tools/emu/sweep_object_data_is_active_build.py` under unicorn
/// (tools/emu): `__thiscall`, `ecx` = the object: the flag dword at `+8` (`f`), virtual slot `+0x20` (`b`) and slot `+0x4c` (`c`): 1 only when `f & 1`, `b != 0` and `c != 0`. Rows are the 96-point grid of the three.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_data_is_active_build.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let got: i64 = i64::from(a[0] & 1 != 0 && a[1] != 0 && a[2] != 0);
        assert_eq!(got, want, "ObjectData::is_active_build — row `{line}`");
        n += 1;
    }
    assert!(n >= 96, "only {n} rows");
    eprintln!("ObjectData::is_active_build@0046fb40: {n} rows agree");
}
