//! `SpellType::cast_jam_radar@006714f0` under the emulator against the port (item 1889).

/// `SpellType::cast_jam_radar@006714f0`, run by `tools/emu/sweep_spell_type_cast_jam_radar.py` under unicorn
/// (tools/emu): nothing is written: `0` changed bytes.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_spell_type_cast_jam_radar.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (_, want) = super::row(line);
        assert_eq!(want, 0, "SpellType::cast_jam_radar — row `{line}`");
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("SpellType::cast_jam_radar@006714f0: {n} rows agree");
}
