//! `Special::walk_data@00740030` under the emulator against the port (item 1921; tools/sweep_gen.py, mix).

/// `Special::walk_data@00740030`, run by `tools/emu/sweep_special_walk_data.py` under unicorn
/// (tools/emu) with every callee and the walker's callbacks hooked: 2 step(s) in order, `ecx` and arguments held to the spec.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_special_walk_data.py") else {
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
        let (_eax, seen) = rhs.split_once(" | ").expect("`<eax> | <calls>`");
        let (this, walker, g, gate) = (a[0], a[1], a[2], a[3]);
        let _ = (this, g, gate);
        let mut calls: Vec<Vec<i64>> = Vec::new();
        calls.push(vec![this + 0x4, walker, this]);
        calls.push(vec![walker, this + 0x24, this + 0x2a]);
        let want: String = calls
            .iter()
            .map(|c| c.iter().map(i64::to_string).collect::<Vec<_>>().join(","))
            .collect::<Vec<_>>()
            .join(";");
        assert_eq!(seen.trim(), want, "walk_data — calls, row `{line}`");
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("Special::walk_data@00740030: {n} rows agree");
}
