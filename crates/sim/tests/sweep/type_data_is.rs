//! `TypeData::is@004771c0` under the emulator against the port (item 1824).

/// `TypeData::is@004771c0`, run by `tools/emu/sweep_type_data_is.py` under unicorn
/// (tools/emu): `__thiscall`, `ecx` = the type record, two stack arguments (`ret 8`): answers `type(+4) == arg1`; `arg2` is ignored.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_type_data_is.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let got = i64::from(a[0] == a[1]);
        assert_eq!(got, want, "TypeData::is — row `{line}`");
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("TypeData::is@004771c0: {n} rows agree");
}
