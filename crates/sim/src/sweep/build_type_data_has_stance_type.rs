//! `BuildTypeData::has_stance_type@00639770` under the emulator against the port (item 1966; hand-written on tools/emu/hooks.py).

/// `BuildTypeData::has_stance_type@00639770`, run by `tools/emu/sweep_build_type_data_has_stance_type.py` under unicorn
/// (tools/emu): the slot's answer drawn so half the rows are equal to the argument.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_type_data_has_stance_type.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let got = i64::from(a[0] as i32 == a[1] as i32);
        assert_eq!(got, want, "has_stance_type — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("BuildTypeData::has_stance_type@00639770: {n} rows agree");
}
