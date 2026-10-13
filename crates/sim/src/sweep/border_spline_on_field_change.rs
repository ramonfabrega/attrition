//! `BorderSpline::on_field_change@004a1ca0` under the emulator against the port (item 1866).

/// `BorderSpline::on_field_change@004a1ca0`, run by `tools/emu/sweep_border_spline_on_field_change.py` under unicorn
/// (tools/emu): `field_change` becomes 1 and the result is 0: `0 * 256 + 1`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_border_spline_on_field_change.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (_, want) = super::row(line);
        assert_eq!(want, 1, "BorderSpline::on_field_change — row `{line}`");
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("BorderSpline::on_field_change@004a1ca0: {n} rows agree");
}
