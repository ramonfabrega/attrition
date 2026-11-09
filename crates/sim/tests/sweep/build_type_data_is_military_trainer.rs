//! `BuildTypeData::is_military_trainer@0063bcf0` under the emulator against the port (item 2072; hand-written on
//! tools/emu/hooks.py and tools/emu/buildtypes.py).

use crate::build::{BuildType, is_military_trainer};

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

/// `BuildTypeData::is_military_trainer@0063bcf0`, run by `tools/emu/sweep_build_type_data_is_military_trainer.py` under unicorn (tools/emu): the `buildtypes` forest, the record's slot `+0xe4`
/// either the base (the original's own root walk, recursing through each parent's slot) or a stub answering an
/// index; the root's `+0x2c0 & 0x40000000` against [`is_military_trainer`] on the port's `FROM` chain.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_type_data_is_military_trainer.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        assert_eq!(
            a[3], 1,
            "is_military_trainer — the slot calls, row `{line}`"
        );
        let types = types_of(&a);
        let (mode, s, r) = (a[0], a[1] as usize, a[2] as usize);
        let _ = (mode, r);
        // Mode 0 is a subclass's own `basic_type` answering `r`: the port has no override, so the arm is the
        // table entry's flag; mode 1 is the port's root walk.
        let has = if mode == 0 {
            types[r].flags & 0x4000_0000 != 0
        } else {
            is_military_trainer(&types, s)
        };
        let got = if has {
            i64::from(0x4000_0000_u32 as i32)
        } else {
            0
        };
        assert_eq!(got, want, "is_military_trainer — row `{line}`");
        n += 1;
    }
    assert!(n >= 300, "only {n} rows");
    eprintln!("BuildTypeData::is_military_trainer@0063bcf0: {n} rows agree");
}
