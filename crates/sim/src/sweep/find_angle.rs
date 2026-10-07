//! `find_angle@0092d130` under the emulator against `movement::find_angle`.

/// `find_angle@0092d130`, run by `tools/emu/sweep_find_angle.py` under
/// unicorn: a register pair (`ecx` = dx, `edx` = dy), no memory read. Every
/// row `<dx> <dy> -> <angle>` is asserted against `crate::movement::find_angle`.
/// The rows stay within |v| <= 131071, where the original's `lo * 0x4000`
/// fits an `i32`; past that the original wraps (and traps at `i32::MIN`) and
/// the port's `lo * 0x4000` would panic in a checked build.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_find_angle.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (args, want) = super::row(line);
        let dx = i32::try_from(args[0]).expect("a 32-bit dx");
        let dy = i32::try_from(args[1]).expect("a 32-bit dy");
        let ours = i64::from(crate::movement::find_angle(dx, dy).0);
        assert_eq!(ours, want, "find_angle({dx}, {dy}) — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("find_angle@0092d130: {n} rows agree");
}
