//! `Build::finished@00470e50` under the emulator against the port (item 1998; hand-written on tools/emu/hooks.py).

/// `Build::finished@00470e50`, run by `tools/emu/sweep_build_finished.py` under unicorn
/// (tools/emu): a forwarder into its own slot `+0x1b0` — `this` and the argument handed on unchanged, the slot's
/// answer returned.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_finished.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        assert_eq!(a[3], 1, "the slot's ecx is this — row `{line}`");
        assert_eq!(a[2], a[1], "the slot is handed the argument — row `{line}`");
        assert_eq!(a[0], want, "finished — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Build::finished@00470e50: {n} rows agree");
}
