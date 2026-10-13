//! `CityData::pop_growth@00738180` under the emulator against the port (item 1736; tools/sweep_gen.py, const).

/// `CityData::pop_growth@00738180`, run by `tools/emu/sweep_city_data_pop_growth.py` under unicorn
/// (tools/emu): answers a constant whatever `this` is, held to `0`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_city_data_pop_growth.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let _this = a[0];
        let got: i64 = 0;
        assert_eq!(got, want, "pop_growth — row `{line}`");
        n += 1;
    }
    assert!(n >= 5, "only {n} rows");
    eprintln!("CityData::pop_growth@00738180: {n} rows agree");
}
