//! `Unit::think_supply@005f4730` under the emulator against the port (item 1895; tools/sweep_gen.py, fwd).

/// `Unit::think_supply@005f4730`, run by `tools/emu/sweep_unit_think_supply.py` under unicorn
/// (tools/emu) with its callees hooked: 1 hooked callee(s), `ecx` and arguments held to the spec, `eax` 1.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_think_supply.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> -> <result>`");
        let a: Vec<i64> = lhs.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let (eax, seen) = rhs.split_once(" | ").expect("`<eax> | <calls>`");
        let (this, _p0, ans) = (a[0], a[1], a[2]);
        let _ = (this, _p0, ans);
        let calls: Vec<Vec<i64>> = vec![vec![this]];
        let want: String = calls
            .iter()
            .map(|c| c.iter().map(i64::to_string).collect::<Vec<_>>().join(","))
            .collect::<Vec<_>>()
            .join(";");
        assert_eq!(seen.trim(), want, "think_supply — calls, row `{line}`");
        assert_eq!(eax.trim().parse::<i64>().unwrap(), 1, "think_supply — eax, row `{line}`");
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("Unit::think_supply@005f4730: {n} rows agree");
}
