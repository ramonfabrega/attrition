//! `UnitTypeData::get_relative_value@0061d740` under the emulator against the port (item 1735).

/// `UnitTypeData::get_relative_value@0061d740`, run by `tools/emu/sweep_unit_type_data_get_relative_value.py` under unicorn
/// (tools/emu): the signed short at `+0x2b0 + i*2` sign-extended (decoys in every neighbouring slot), against `flag_reads::relative_value`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_type_data_get_relative_value.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (i, v) = (a[0], i16::try_from(a[1]).expect("a short"));
        assert!((0..100).contains(&i));
        assert_eq!(
            want,
            i64::from(crate::flag_reads::relative_value(v)),
            "UnitTypeData::get_relative_value — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitTypeData::get_relative_value@0061d740: {n} rows agree");
}
