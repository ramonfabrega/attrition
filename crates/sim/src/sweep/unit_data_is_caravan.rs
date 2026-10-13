//! `UnitData::is_caravan@0046ce90` under the emulator against the port (item 1834).

/// `UnitData::is_caravan@0046ce90`, run by `tools/emu/sweep_unit_data_is_caravan.py` under unicorn
/// (tools/emu): `__thiscall`, `ecx` = the unit, a tail jump through slot `+0x130` of its type record's (`+0x18`) vtable, laid as a stub answering the row's word: the function answers that word unchanged. The rows are ten edge words and a hundred random ones.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_is_caravan.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let got: i64 = a[0];
        assert_eq!(got, want, "UnitData::is_caravan — row `{line}`");
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("UnitData::is_caravan@0046ce90: {n} rows agree");
}
