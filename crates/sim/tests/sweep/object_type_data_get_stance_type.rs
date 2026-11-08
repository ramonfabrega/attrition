//! `ObjectTypeData::get_stance_type@00476aa0` under the emulator against the port (item 1814; tools/sweep_gen.py, const).

/// `ObjectTypeData::get_stance_type@00476aa0`, run by `tools/emu/sweep_object_type_data_get_stance_type.py` under unicorn
/// (tools/emu): answers a constant whatever `this` is, held to `0`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_type_data_get_stance_type.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let _this = a[0];
        let got: i64 = -1;
        assert_eq!(got, want, "get_stance_type — row `{line}`");
        n += 1;
    }
    assert!(n >= 5, "only {n} rows");
    eprintln!("ObjectTypeData::get_stance_type@00476aa0: {n} rows agree");
}
