//! `Window::on_focus@0041dbe0` under the emulator against the port (item 1736; tools/sweep_gen.py, const).

/// `Window::on_focus@0041dbe0`, run by `tools/emu/sweep_window_on_focus.py` under unicorn
/// (tools/emu): answers a constant whatever `this` is, held to `1`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_window_on_focus.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let _this = a[0];
        let got: i64 = 1;
        assert_eq!(got, want, "on_focus — row `{line}`");
        n += 1;
    }
    assert!(n >= 5, "only {n} rows");
    eprintln!("Window::on_focus@0041dbe0: {n} rows agree");
}
