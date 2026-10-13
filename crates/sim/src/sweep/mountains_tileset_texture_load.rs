//! `Mountains::tileset_texture_load@0089d3a0` under the emulator against the port (item 1866).

/// `Mountains::tileset_texture_load@0089d3a0`, run by `tools/emu/sweep_mountains_tileset_texture_load.py` under unicorn
/// (tools/emu): `LIBERR_OK`, `0`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_mountains_tileset_texture_load.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (_, want) = super::row(line);
        assert_eq!(want, 0, "Mountains::tileset_texture_load — row `{line}`");
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("Mountains::tileset_texture_load@0089d3a0: {n} rows agree");
}
