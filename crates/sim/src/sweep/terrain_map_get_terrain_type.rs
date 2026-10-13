//! `TerrainMap::get_terrain_type@009b5ec0` under the emulator against the port (item 1856).

/// `TerrainMap::get_terrain_type@009b5ec0`, run by `tools/emu/sweep_terrain_map_get_terrain_type.py` under unicorn
/// (tools/emu): `__thiscall`, two stack arguments (`ret 8`): `x`, `y`; the byte at `[this+0x9c8] + [[0xc06188]] * y + x` (`GameAccess::world`'s first word is the map width). `eax` keeps `x`'s upper 24 bits (callers read `al`), so the row prints `al`. Row `<width> <x> <y> <byte> -> <al>`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_terrain_map_get_terrain_type.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let _ = &a;
        let got: i64 = a[3];
        assert_eq!(got, want, "TerrainMap::get_terrain_type — row `{line}`");
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("TerrainMap::get_terrain_type@009b5ec0: {n} rows agree");
}
