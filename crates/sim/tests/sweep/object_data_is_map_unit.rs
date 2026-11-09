//! `ObjectData::is_map_unit@0046ffa0` under the emulator against the port (item 1824).

/// `ObjectData::is_map_unit@0046ffa0`, run by `tools/emu/sweep_object_data_is_map_unit.py` under unicorn
/// (tools/emu): `__thiscall`, `ecx` = the object, two virtual calls — slot `+8`, then (if non-zero) slot `+0xbc` — laid as stubs (`mov eax, imm; ret`) answering each row's two words: 1 only when both are non-zero, else 0.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_data_is_map_unit.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let got = i64::from(a[0] != 0 && a[1] != 0);
        assert_eq!(got, want, "ObjectData::is_map_unit — row `{line}`");
        n += 1;
    }
    assert!(n >= 64, "only {n} rows");
    eprintln!("ObjectData::is_map_unit@0046ffa0: {n} rows agree");
}
