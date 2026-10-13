//! `TerrainMap::on_right_up@009b4820` under the emulator against the port (item 1866).

/// `TerrainMap::on_right_up@009b4820`, run by `tools/emu/sweep_terrain_map_on_right_up.py` under unicorn
/// (tools/emu): this+0xa84 (`drawing`) is zeroed whatever it held, against `0`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_terrain_map_on_right_up.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (_, want) = super::row(line);
        assert_eq!(want, 0, "TerrainMap::on_right_up — row `{line}`");
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("TerrainMap::on_right_up@009b4820: {n} rows agree");
}
