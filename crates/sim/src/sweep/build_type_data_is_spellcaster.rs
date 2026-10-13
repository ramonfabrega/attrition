//! `BuildTypeData::is_spellcaster@00639840` under the emulator against the port (item 2083; hand-written on
//! tools/emu/hooks.py and tools/emu/buildtypes.py).

use crate::build::{BuildType, is_spellcaster};

/// The row's ten records as the port's [`BuildType`]s: a non-negative `FROM` is the parent, the flag word is
/// `flags` (the original's `+0x2c0`).
fn types_of(a: &[i64]) -> Vec<BuildType> {
    (0..10)
        .map(|i| BuildType {
            from: usize::try_from(a[2 + i]).ok(),
            flags: a[12 + i] as u32,
            ..BuildType::default()
        })
        .collect()
}

/// `BuildTypeData::is_spellcaster@00639840`, run by `tools/emu/sweep_build_type_data_is_spellcaster.py` under
/// unicorn (tools/emu): the `buildtypes` forest, the walk from the record's own table entry up its `FROM` chain,
/// `0x20000000` on any record of it, against [`is_spellcaster`]; the unread stack argument random.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_type_data_is_spellcaster.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let types = types_of(&a);
        let got = i64::from(is_spellcaster(&types, a[0] as usize));
        assert_eq!(got, want, "is_spellcaster — row `{line}`");
        n += 1;
    }
    assert!(n >= 300, "only {n} rows");
    eprintln!("BuildTypeData::is_spellcaster@00639840: {n} rows agree");
}
