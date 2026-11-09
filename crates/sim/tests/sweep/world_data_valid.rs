//! `WorldData::valid@00461420` under the emulator against the port (item 1814).

/// `WorldData::valid@00461420`, run by `tools/emu/sweep_world_data_valid.py` under unicorn
/// (tools/emu): `__thiscall`, `ecx` = the world data (`xs` +0, `ys` +4); two `WCoord` pointers on the stack — in range when both are non-negative and under their extent.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_data_valid.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (xs, ys, x, y) = (a[0], a[1], a[2], a[3]);
        let got = (x > -1 && y > -1 && x < xs && y < ys) as i64;
        assert_eq!(got, want, "WorldData::valid — row `{line}`");
        n += 1;
    }
    assert!(n >= 800, "only {n} rows");
    eprintln!("WorldData::valid@00461420: {n} rows agree");
}
