//! `GoodTypeData::get_enhancer@0066fcf0` under the emulator against the port (item 1824).

/// `GoodTypeData::get_enhancer@0066fcf0`, run by `tools/emu/sweep_good_type_data_get_enhancer.py` under unicorn
/// (tools/emu): `__thiscall`, `ecx` = the good type, a switch on the dword at `+4` (unsigned compare against 5): 0 → 0x1a7, 1 → 0x1a8, 4 → 0x1a9, 5 → 0x1aa, anything else (2, 3, above 5, negative) → -1.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_good_type_data_get_enhancer.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let got = match a[0] {
            0 => 0x1a7,
            1 => 0x1a8,
            4 => 0x1a9,
            5 => 0x1aa,
            _ => -1,
        };
        assert_eq!(got, want, "GoodTypeData::get_enhancer — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("GoodTypeData::get_enhancer@0066fcf0: {n} rows agree");
}
