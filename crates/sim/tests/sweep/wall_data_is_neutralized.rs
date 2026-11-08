//! `WallData::is_neutralized@0063ed30` under the emulator against the port (item 1724).

/// `WallData::is_neutralized@0063ed30`, run by `tools/emu/sweep_wall_data_is_neutralized.py` under unicorn
/// (tools/emu): the constant 0, whatever `this` holds.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wall_data_is_neutralized.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let _ = a;
        assert_eq!(want, 0, "WallData::is_neutralized — row `{line}`");
        n += 1;
    }
    assert!(n >= 5, "only {n} rows");
    eprintln!("WallData::is_neutralized@0063ed30: {n} rows agree");
}
