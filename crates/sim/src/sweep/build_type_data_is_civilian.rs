//! `BuildTypeData::is_civilian@00639930` under the emulator against the port (item 1819).

/// `BuildTypeData::is_civilian@00639930`, run by `tools/emu/sweep_build_type_data_is_civilian.py` under unicorn
/// (tools/emu): `__thiscall`, `ecx` = the type; `build_flags` at +0x2c0: 0 when bit 0x2000 is set, else bit 4 of the complement.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_type_data_is_civilian.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let f = u32::from_ne_bytes(i32::try_from(a[0]).expect("int").to_ne_bytes());
        let got = if f & 0x2000 != 0 {
            0
        } else {
            i64::from((!f >> 4) & 1)
        };
        assert_eq!(got, want, "BuildTypeData::is_civilian — row `{line}`");
        n += 1;
    }
    assert!(n >= 600, "only {n} rows");
    eprintln!("BuildTypeData::is_civilian@00639930: {n} rows agree");
}
