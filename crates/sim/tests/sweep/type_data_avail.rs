//! `TypeData::avail@004770d0` under the emulator against the port (item 1793; tools/sweep_gen.py, field).

/// `TypeData::avail@004770d0`, run by `tools/emu/sweep_type_data_avail.py` under unicorn
/// (tools/emu): the signed-32 edges and two hundred random words at the field, against `i64::from(v != -2)`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_type_data_avail.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let v = i32::try_from(a[0]).expect("a 32-bit integer");
        let got: i64 = i64::from(v != -2);
        assert_eq!(got, want, "avail — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("TypeData::avail@004770d0: {n} rows agree");
}
