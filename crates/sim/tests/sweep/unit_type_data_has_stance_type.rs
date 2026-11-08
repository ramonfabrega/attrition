//! `UnitTypeData::has_stance_type@0061d3b0` under the emulator against the port (item 1840).

/// `UnitTypeData::has_stance_type@0061d3b0`, run by `tools/emu/sweep_unit_type_data_has_stance_type.py` under unicorn
/// (tools/emu): `__thiscall`, one stack int `a`; virtual slot `+0x138` (`v`): 1 when `v == a`. Rows `<v> <a>`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_type_data_has_stance_type.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let got: i64 = i64::from(a[0] == a[1]);
        assert_eq!(got, want, "UnitTypeData::has_stance_type — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitTypeData::has_stance_type@0061d3b0: {n} rows agree");
}
