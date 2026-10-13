//! `WallData::is_sea_trade@004722f0` under the emulator against the port (item 1840).

/// `WallData::is_sea_trade@004722f0`, run by `tools/emu/sweep_wall_data_is_sea_trade.py` under unicorn
/// (tools/emu): `__thiscall`: slot `+0xb8` == `ObjectData::is` calls the type's slot `+0x60` with `(0x1b0, 0)`, else slot `+0xb8`. Stubs answer `v + 1000*arg0 + arg1`. Rows `<mode> <sv> <tv>`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wall_data_is_sea_trade.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let got: i64 = {
            let v = if a[0] == 0 { a[2] } else { a[1] };
            v + 1000 * 0x1b0
        };
        assert_eq!(got, want, "WallData::is_sea_trade — row `{line}`");
        n += 1;
    }
    assert!(n >= 10, "only {n} rows");
    eprintln!("WallData::is_sea_trade@004722f0: {n} rows agree");
}
