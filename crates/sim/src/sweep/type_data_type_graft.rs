//! `TypeData::type_graft@00472a50` under the emulator against the port (item 1866).

/// `TypeData::type_graft@00472a50`, run by `tools/emu/sweep_type_data_type_graft.py` under unicorn
/// (tools/emu): the record's own `type`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_type_data_type_graft.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        assert_eq!(want, a[0], "TypeData::type_graft — row `{line}`");
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("TypeData::type_graft@00472a50: {n} rows agree");
}
