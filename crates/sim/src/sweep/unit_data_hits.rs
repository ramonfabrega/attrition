//! `UnitData::hits@00610890` under the emulator against the port (item 1724).

/// `UnitData::hits@00610890`, run by `tools/emu/sweep_unit_data_hits.py` under unicorn
/// (tools/emu): the int `+0x20`, the argument ignored.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_hits.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let v = i32::try_from(a[0]).expect("a 32-bit integer");
        assert_eq!(want, i64::from(v), "UnitData::hits — the raw word, row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitData::hits@00610890: {n} rows agree");
}
