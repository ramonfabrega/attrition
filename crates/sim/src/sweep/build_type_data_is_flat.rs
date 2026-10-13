//! `BuildTypeData::is_flat@00472b20` under the emulator against the port (item 1793; tools/sweep_gen.py, field).

/// `BuildTypeData::is_flat@00472b20`, run by `tools/emu/sweep_build_type_data_is_flat.py` under unicorn
/// (tools/emu): the signed-32 edges and two hundred random words at the field, against `i64::from(v & 0x1000_0000)`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_type_data_is_flat.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let v = i32::try_from(a[0]).expect("a 32-bit integer");
        let got: i64 = i64::from(v & 0x1000_0000);
        assert_eq!(got, want, "is_flat — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("BuildTypeData::is_flat@00472b20: {n} rows agree");
}
