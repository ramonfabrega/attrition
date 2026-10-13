//! `UnitData::can_gather@00608850` under the emulator against the port (item 1856).

/// `UnitData::can_gather@00608850`, run by `tools/emu/sweep_unit_data_can_gather.py` under unicorn
/// (tools/emu): `__thiscall`, no arguments: `[[this+0x18]+4]` in `0x32..=0x35` gives 1, else 0. Row `<kind> -> <result>`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_can_gather.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let _ = &a;
        let got: i64 = i64::from((0x32..=0x35).contains(&a[0]));
        assert_eq!(got, want, "UnitData::can_gather — row `{line}`");
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("UnitData::can_gather@00608850: {n} rows agree");
}
