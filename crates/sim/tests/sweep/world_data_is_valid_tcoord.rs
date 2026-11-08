//! `WorldData::is_valid@0046f760` under the emulator against the port (item 1834).

/// `WorldData::is_valid@0046f760`, run by `tools/emu/sweep_world_data_is_valid_tcoord.py` under unicorn
/// (tools/emu): `__thiscall`, `ecx` = the world, two stack pointers to `TCoord` words: 1 when `0 <= x < tile_xs` (`+0x18`) and `0 <= y < tile_ys` (`+0x1c`). Rows `<tile_xs> <tile_ys> <x> <y>`: a 400-point grid and a hundred random.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_data_is_valid_tcoord.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let got: i64 = i64::from(a[2] >= 0 && a[3] >= 0 && a[2] < a[0] && a[3] < a[1]);
        assert_eq!(got, want, "WorldData::is_valid — row `{line}`");
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("WorldData::is_valid@0046f760: {n} rows agree");
}
