//! `flanking@0092cfe0` under the emulator against `combat::flanking`.

/// `flanking@0092cfe0`, run by `tools/emu/sweep_flanking.py` under unicorn:
/// the bias `e` in `ecx`, answers 0, 1 or 2. Every row is asserted against
/// `crate::combat::flanking`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_flanking.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (args, want) = super::row(line);
        let e = u32::try_from(args[0]).expect("a 32-bit bias");
        let ours = i64::from(crate::combat::flanking(e));
        assert_eq!(ours, want, "flanking({e:#x}) — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("flanking@0092cfe0: {n} rows agree");
}
