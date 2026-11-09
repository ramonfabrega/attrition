//! `Terrain::convert_errant_sandy@0084b8d0` under the emulator against the port (item 1866).

/// `Terrain::convert_errant_sandy@0084b8d0`, run by `tools/emu/sweep_terrain_convert_errant_sandy.py` under unicorn
/// (tools/emu): nothing is written: `0` changed bytes.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_terrain_convert_errant_sandy.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (_, want) = super::row(line);
        assert_eq!(want, 0, "Terrain::convert_errant_sandy — row `{line}`");
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("Terrain::convert_errant_sandy@0084b8d0: {n} rows agree");
}
