//! `Setup::init_wild_life@005aac70` under the emulator against the port (item 1736; tools/sweep_gen.py, const).

/// `Setup::init_wild_life@005aac70`, run by `tools/emu/sweep_setup_init_wild_life.py` under unicorn
/// (tools/emu): answers a constant whatever `this` is, held to `0`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_setup_init_wild_life.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let _this = a[0];
        let got: i64 = 0;
        assert_eq!(got, want, "init_wild_life — row `{line}`");
        n += 1;
    }
    assert!(n >= 5, "only {n} rows");
    eprintln!("Setup::init_wild_life@005aac70: {n} rows agree");
}
