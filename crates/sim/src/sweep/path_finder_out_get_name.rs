//! `PathFinderOut::get_name@0047a920` under the emulator against the port (item 2583; tools/sweep_gen.py, mix).

/// `PathFinderOut::get_name@0047a920`, run by `tools/emu/sweep_path_finder_out_get_name.py` under unicorn
/// (tools/emu) with every callee and the walker's callbacks hooked: 1 step(s) in order, `ecx` and arguments held to the spec.
#[test]
#[allow(clippy::vec_init_then_push)]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_path_finder_out_get_name.py") else {
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
        let eax: i64 = eax.trim().parse().expect("an integer");
        let (this, walker, g, gate) = (a[0], a[1], a[2], a[3]);
        let _ = eax;
        let stale = gate; // the slot-4 callee's leftover `ecx` when a step names it
        let _ = (this, g, gate, stale);
        let mut calls: Vec<Vec<i64>> = Vec::new();
        calls.push(vec![walker, 11818056]);
        let want: String = calls
            .iter()
            .map(|c| c.iter().map(i64::to_string).collect::<Vec<_>>().join(","))
            .collect::<Vec<_>>()
            .join(";");
        assert_eq!(seen.trim(), want, "get_name — calls, row `{line}`");
        assert_eq!(
            eax, walker,
            "get_name — the result is the out-pointer, row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("PathFinderOut::get_name@0047a920: {n} rows agree");
}
