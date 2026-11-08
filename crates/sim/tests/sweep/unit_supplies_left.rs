//! `UnitData::supplies_left@00609a20` under the emulator against the port (item 1708).

/// `UnitData::supplies_left@00609a20`, run by `tools/emu/sweep_unit_supplies_left.py` under unicorn
/// (tools/emu): the constant 1 through `eax`, whatever `this` points at, against `flag_reads::SUPPLIES_LEFT`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_supplies_left.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let _ = a;
        assert_eq!(
            crate::flag_reads::SUPPLIES_LEFT,
            want,
            "supplies_left — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 4, "only {n} rows");
    eprintln!("UnitData::supplies_left@00609a20: {n} rows agree");
}
