//! `BuildTypeData::is_tower@00472a30` under the emulator against the port (item 1824).

/// `BuildTypeData::is_tower@00472a30`, run by `tools/emu/sweep_build_type_data_is_tower.py` under unicorn
/// (tools/emu): `__thiscall`, `ecx` = the type record, whose vtable slot `+0x60` is laid as the original's `TypeData::is@004771c0` (so the answer is what the dispatch does with its constant): answers `type(+4) == 0x1b7`. The rows are the constant, its neighbours and the other four constants, and two hundred random words.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_type_data_is_tower.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let got = i64::from(a[0] == 0x1b7);
        assert_eq!(got, want, "BuildTypeData::is_tower — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("BuildTypeData::is_tower@00472a30: {n} rows agree");
}
