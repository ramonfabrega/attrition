//! `Unit::set_type@0046ce10` under the emulator against the port (item 1998; hand-written on tools/emu/hooks.py).

/// `Unit::set_type@0046ce10`, run by `tools/emu/sweep_unit_set_type.py` under unicorn
/// (tools/emu): a forwarder that tears its frame down and jumps into its own slot `+0x84` — the slot is handed
/// `this` and both stack arguments unchanged, and its answer is `eax` (a `void`, so only the jump shows it).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_set_type.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        assert_eq!(a[5], 1, "the slot's ecx is this — row `{line}`");
        assert_eq!(
            &a[3..5],
            &a[1..3],
            "the slot is handed both arguments — row `{line}`"
        );
        assert_eq!(a[0], want, "set_type — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Unit::set_type@0046ce10: {n} rows agree");
}
