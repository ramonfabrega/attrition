//! `ObjectData::is_active_wallbuild@0046eea0` under the emulator against the port (item 1840).

/// `ObjectData::is_active_wallbuild@0046eea0`, run by `tools/emu/sweep_object_data_is_active_wallbuild.py` under unicorn
/// (tools/emu): `__thiscall`, `ecx` = the object: virtual slots `+0xc` (`b`) and `+0x4c` (`c`): 1 only when both are non-zero. Rows are the grid of the two.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_data_is_active_wallbuild.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let got: i64 = i64::from(a[0] != 0 && a[1] != 0);
        assert_eq!(got, want, "ObjectData::is_active_wallbuild — row `{line}`");
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("ObjectData::is_active_wallbuild@0046eea0: {n} rows agree");
}
