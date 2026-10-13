//! `WorldData::is_edge@0047a970` under the emulator against the port (item 1840).

/// `WorldData::is_edge@0047a970`, run by `tools/emu/sweep_world_data_is_edge.py` under unicorn
/// (tools/emu): `__thiscall`, two stack pointers to `WCoord` words (x, then y); the `GameAccess::world` cell's `xs` (`+0`) and `ys` (`+4`): 1 when `x == 0`, `y == 0`, `x == xs - 1` or `y == ys - 1`. Rows `<xs> <ys> <x> <y>`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_data_is_edge.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let got: i64 = i64::from(a[2] == 0 || a[3] == 0 || a[2] == a[0] - 1 || a[3] == a[1] - 1);
        assert_eq!(got, want, "WorldData::is_edge — row `{line}`");
        n += 1;
    }
    assert!(n >= 390, "only {n} rows");
    eprintln!("WorldData::is_edge@0047a970: {n} rows agree");
}
