//! `UnitData::support_cost@0060fde0` under the emulator against the port (item 1724).

/// `UnitData::support_cost@0060fde0`, run by `tools/emu/sweep_unit_data_support_cost.py` under unicorn
/// (tools/emu): the constant 0.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_support_cost.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        assert_eq!(a.len(), 1);
        assert_eq!(want, 0, "UnitData::support_cost — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitData::support_cost@0060fde0: {n} rows agree");
}
