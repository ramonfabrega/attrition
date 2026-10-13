//! `ObjectData::support@0046cef0` under the emulator against the port (item 1840).

/// `ObjectData::support@0046cef0`, run by `tools/emu/sweep_object_data_support.py` under unicorn
/// (tools/emu): `__thiscall`, `ecx` = the object, one stack int: the dword at `*(this+0x18) + 0x268 + 4*i`. Rows `<i> <val> <decoy>`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_data_support.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let got: i64 = a[1];
        assert_eq!(got, want, "ObjectData::support — row `{line}`");
        n += 1;
    }
    assert!(n >= 300, "only {n} rows");
    eprintln!("ObjectData::support@0046cef0: {n} rows agree");
}
