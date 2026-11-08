//! `UnitTypeData::get_circle_radius@0061db10` under the emulator against the port (item 1724).

/// `UnitTypeData::get_circle_radius@0061db10`, run by `tools/emu/sweep_unit_type_data_get_circle_radius.py` under unicorn
/// (tools/emu): the identity on its argument.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_type_data_get_circle_radius.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        assert_eq!(want, a[0], "UnitTypeData::get_circle_radius — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitTypeData::get_circle_radius@0061db10: {n} rows agree");
}
