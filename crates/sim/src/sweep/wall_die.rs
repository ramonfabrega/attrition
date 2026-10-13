//! `Wall::die@0063f6c0` under the emulator against the port (item 1911; tools/sweep_gen.py, fwd).

/// `Wall::die@0063f6c0`, run by `tools/emu/sweep_wall_die.py` under unicorn
/// (tools/emu) with its callees hooked: 1 hooked callee(s), `ecx` and arguments held to the spec, `eax` none.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wall_die.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> -> <result>`");
        let a: Vec<i64> = lhs.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let (eax, seen) = rhs.split_once(" | ").expect("`<eax> | <calls>`");
        let (this, _p0, ans, _p1, _p2) = (a[0], a[1], a[2], a[3], a[4]);
        let _ = (this, _p0, ans, _p1, _p2);
        let calls: Vec<Vec<i64>> = vec![vec![this, _p0, _p1, _p2]];
        let want: String = calls
            .iter()
            .map(|c| c.iter().map(i64::to_string).collect::<Vec<_>>().join(","))
            .collect::<Vec<_>>()
            .join(";");
        assert_eq!(seen.trim(), want, "die — calls, row `{line}`");
        let _ = eax;
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("Wall::die@0063f6c0: {n} rows agree");
}
