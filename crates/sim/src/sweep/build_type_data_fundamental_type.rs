//! `BuildTypeData::fundamental_type@00639950` under the emulator against the port (item 2072; hand-written on
//! tools/emu/hooks.py and tools/emu/buildtypes.py).

use crate::build::{BuildType, fundamental};

/// The row's ten records as the port's [`BuildType`]s: a non-negative `FROM` is the parent, the flag word is
/// `flags` (the original's `+0x2c0`).
fn types_of(a: &[i64]) -> Vec<BuildType> {
    (0..10)
        .map(|i| BuildType {
            from: usize::try_from(a[4 + i]).ok(),
            flags: a[14 + i] as u32,
            ..BuildType::default()
        })
        .collect()
}

/// `BuildTypeData::fundamental_type@00639950`, run by `tools/emu/sweep_build_type_data_fundamental_type.py` under unicorn (tools/emu): a forest of ten type records in the `buildtypes`
/// table, a parent always earlier; the original walks `+0x3c` to the root and answers its `+0x4` (its own
/// index), against [`fundamental`].
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_type_data_fundamental_type.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        assert_eq!(a[3], 1, "fundamental_type — the slot calls, row `{line}`");
        let types = types_of(&a);
        let (mode, s, r) = (a[0], a[1] as usize, a[2] as usize);
        let _ = (mode, r);
        let got = fundamental(&types, s) as i64;
        assert_eq!(got, want, "fundamental — row `{line}`");
        n += 1;
    }
    assert!(n >= 300, "only {n} rows");
    eprintln!("BuildTypeData::fundamental_type@00639950: {n} rows agree");
}
