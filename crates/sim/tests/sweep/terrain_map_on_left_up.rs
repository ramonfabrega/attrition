//! `TerrainMap::on_left_up@009b4930` under the emulator against the port (item 1866).

/// `TerrainMap::on_left_up@009b4930`, run by `tools/emu/sweep_terrain_map_on_left_up.py` under unicorn
/// (tools/emu): `drawing` and `stamping` are both cleared, against `0`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_terrain_map_on_left_up.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (_, want) = super::row(line);
        assert_eq!(want, 0, "TerrainMap::on_left_up — row `{line}`");
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("TerrainMap::on_left_up@009b4930: {n} rows agree");
}
