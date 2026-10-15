//! `BuildQueue::walk_data@006305f0` under the emulator against the port (item 2698; tools/sweep_gen.py, mix).

/// `BuildQueue::walk_data@006305f0`, run by `tools/emu/sweep_build_queue_walk_data.py` under unicorn
/// (tools/emu) with every callee and the walker's callbacks hooked: 5 step(s) in order, `ecx` and arguments held to the spec.
#[test]
#[allow(clippy::vec_init_then_push)]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_queue_walk_data.py") else {
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
        let count = a[4];
        let _ = count;
        let elems = &a[5..];
        assert_eq!(
            elems.len() as i64,
            count.max(0),
            "walk_data — one element a counted record, row `{line}`"
        );
        let _ = eax;
        let stale = g; // the slot-4 callee's leftover `ecx` when a step names it
        let _ = (this, g, gate, stale);
        let mut calls: Vec<Vec<i64>> = Vec::new();
        calls.push(vec![walker, g + 0x1770]);
        calls.push(vec![walker, this, this + 0x4]);
        if gate != 0 && count == 0 {
            calls.push(vec![this]);
        }
        if gate != 0 && count != 0 {
            calls.push(vec![this, count]);
        }
        for &e in elems {
            calls.push(vec![walker, e, e + 0x12]);
        }
        let want: String = calls
            .iter()
            .map(|c| c.iter().map(i64::to_string).collect::<Vec<_>>().join(","))
            .collect::<Vec<_>>()
            .join(";");
        assert_eq!(seen.trim(), want, "walk_data — calls, row `{line}`");
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("BuildQueue::walk_data@006305f0: {n} rows agree");
}
