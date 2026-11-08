//! `World::adjust_fog@006b76a0` under the emulator against the port (item 1889).

/// `World::adjust_fog@006b76a0`, run by `tools/emu/sweep_world_adjust_fog.py` under unicorn
/// (tools/emu): nothing is written: `0` changed bytes.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_adjust_fog.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (_, want) = super::row(line);
        assert_eq!(want, 0, "World::adjust_fog — row `{line}`");
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("World::adjust_fog@006b76a0: {n} rows agree");
}
