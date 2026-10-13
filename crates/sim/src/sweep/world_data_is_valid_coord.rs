//! `WorldData::is_valid@0043f360` under the emulator against the port (item 1834).

/// `WorldData::is_valid@0043f360`, run by `tools/emu/sweep_world_data_is_valid_coord.py` under unicorn
/// (tools/emu): `__thiscall`, `ecx` = the world, two stack pointers to `Coord` words: 1 when `0 <= x < tile_xs * 0xc0` and `0 <= y < tile_ys * 0xc0`, the products wrapping in 32 bits and compared signed. Rows `<tile_xs> <tile_ys> <x> <y>`: a 400-point grid, a hundred random and the wrapping tile counts.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_data_is_valid_coord.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let got: i64 = i64::from(a[2] >= 0 && a[3] >= 0 && a[2] < (a[0] as i32).wrapping_mul(0xc0) as i64 && a[3] < (a[1] as i32).wrapping_mul(0xc0) as i64);
        assert_eq!(got, want, "WorldData::is_valid — row `{line}`");
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("WorldData::is_valid@0043f360: {n} rows agree");
}
