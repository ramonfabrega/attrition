//! `Type::set_graph_name@00663b60` under the emulator against the port (item 1895; tools/sweep_gen.py, fwd).

/// `Type::set_graph_name@00663b60`, run by `tools/emu/sweep_type_set_graph_name.py` under unicorn
/// (tools/emu) with its callees hooked: 1 hooked callee(s), `ecx` and arguments held to the spec, `eax` last.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_type_set_graph_name.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line
            .split_once(" -> ")
            .expect("a row is `<args…> -> <result>`");
        let a: Vec<i64> = lhs
            .split_whitespace()
            .map(|s| s.parse().expect("an integer"))
            .collect();
        let (eax, seen) = rhs.split_once(" | ").expect("`<eax> | <calls>`");
        let (this, _p0, ans) = (a[0], a[1], a[2]);
        let _ = (this, _p0, ans);
        let calls: Vec<Vec<i64>> = vec![vec![this + 0x88, _p0]];
        let want: String = calls
            .iter()
            .map(|c| c.iter().map(i64::to_string).collect::<Vec<_>>().join(","))
            .collect::<Vec<_>>()
            .join(";");
        assert_eq!(seen.trim(), want, "set_graph_name — calls, row `{line}`");
        let want_eax = ans as i32;
        assert_eq!(
            eax.trim().parse::<i64>().unwrap(),
            i64::from(want_eax),
            "set_graph_name — eax, row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("Type::set_graph_name@00663b60: {n} rows agree");
}
