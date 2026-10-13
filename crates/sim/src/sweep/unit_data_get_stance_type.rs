//! `UnitData::get_stance_type@0046cee0` under the emulator against the port (item 1993; tools/sweep_gen.py, vcall).

/// `UnitData::get_stance_type@0046cee0`, run by `tools/emu/sweep_unit_data_get_stance_type.py` under unicorn
/// (tools/emu): the slot's answer over the signed-32 edges and two hundred random words, against `v as i64`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_get_stance_type.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let v = i32::try_from(a[0]).expect("a 32-bit integer");
        let got: i64 = v as i64;
        assert_eq!(got, want, "get_stance_type — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitData::get_stance_type@0046cee0: {n} rows agree");
}
