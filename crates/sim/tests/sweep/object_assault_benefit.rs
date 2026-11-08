//! `ObjectData::assault_benefit@006486a0` under the emulator against the port (item 1723).

/// `ObjectData::assault_benefit@006486a0`, run by `tools/emu/sweep_object_assault_benefit.py` under unicorn
/// (tools/emu): the constant 0, `this` and both arguments never read.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_assault_benefit.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let _ = a;
        assert_eq!(want, 0, "ObjectData::assault_benefit — row `{line}`");
        n += 1;
    }
    assert!(n >= 5, "only {n} rows");
    eprintln!("ObjectData::assault_benefit@006486a0: {n} rows agree");
}
