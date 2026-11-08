//! `UnitData::is_rare_collector@0046fae0` under the emulator against the port (item 1834).

/// `UnitData::is_rare_collector@0046fae0`, run by `tools/emu/sweep_unit_data_is_rare_collector.py` under unicorn
/// (tools/emu): `__thiscall`, `ecx` = the unit, `+0x18` its type record, whose index at `+4` is tested against 0x3d, 0x3e and 400, then (slot `+0x60`, laid as the original's `TypeData::is@004771c0`) against 0x13d: 1 when it is one of the four. The rows are the four, their neighbours, the edges and a hundred random indices.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_is_rare_collector.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let got: i64 = i64::from(matches!(a[0], 0x3d | 0x3e | 400 | 0x13d));
        assert_eq!(got, want, "UnitData::is_rare_collector — row `{line}`");
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("UnitData::is_rare_collector@0046fae0: {n} rows agree");
}
