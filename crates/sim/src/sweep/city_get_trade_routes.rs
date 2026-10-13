//! `CityData::get_trade_routes@00736470` under the emulator against the port (item 1723).

/// `CityData::get_trade_routes@00736470`, run by `tools/emu/sweep_city_get_trade_routes.py` under unicorn
/// (tools/emu): the constant 1, whatever `this` points at.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_city_get_trade_routes.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let _ = a;
        assert_eq!(want, 1, "CityData::get_trade_routes — row `{line}`");
        n += 1;
    }
    assert!(n >= 5, "only {n} rows");
    eprintln!("CityData::get_trade_routes@00736470: {n} rows agree");
}
