//! `Good::inc_time@0066d850` under the emulator against the port (item 1735).

/// `Good::inc_time@0066d850`, run by `tools/emu/sweep_good_inc_time.py` under unicorn
/// (tools/emu): a bare `ret`: no byte of the object changes, whatever it holds — the port keeps no per-frame good clock (a function of no consumer).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_good_inc_time.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (_, want) = super::row(line);
        assert_eq!(want, 0, "Good::inc_time — row `{line}`");
        n += 1;
    }
    assert!(n >= 50, "only {n} rows");
    eprintln!("Good::inc_time@0066d850: {n} rows agree");
}
