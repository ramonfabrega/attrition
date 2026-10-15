//! `Dock::walk_data@00740c40` under the emulator against the port (item 2588; tools/sweep_gen.py, mix).

/// `Dock::walk_data@00740c40`, run by `tools/emu/sweep_dock_walk_data.py` under unicorn
/// (tools/emu) with every callee and the walker's callbacks hooked: 3 step(s) in order, `ecx` and arguments held to the spec.
#[test]
#[allow(clippy::vec_init_then_push)]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_dock_walk_data.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> -> <result>`");
        let a: Vec<i64> = lhs.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let (eax, seen) = rhs.split_once(" | ").expect("`<eax> | <calls>`");
        let eax: i64 = eax.trim().parse().expect("an integer");
        let (this, walker, g, gate) = (a[0], a[1], a[2], a[3]);
        let w = a[4];
        let (seen, tail) = seen.rsplit_once(" | ").expect("`<calls> | <after> <before>`");
        let (after, before) = tail.split_once(' ').expect("`<after> <before>`");
        let rd = |h: &str| (0..h.len() / 2).rev().fold(0i64, |acc, k| acc << 8 | i64::from_str_radix(&h[2 * k..2 * k + 2], 16).expect("hex"));
        let (after, before) = (rd(after), rd(before));
        let _ = eax;
        let stale = g; // the slot-4 callee's leftover `ecx` when a step names it
        let _ = (this, g, gate, stale);
        let mut calls: Vec<Vec<i64>> = Vec::new();
        calls.push(vec![walker, this, this + 0x6]);
        calls.push(vec![walker, this + 0x8, this + 0xa]);
        if gate == 0 { calls.push(vec![walker, this + 0x6, this + 0x8]); }
        let want: String = calls
            .iter()
            .map(|c| c.iter().map(i64::to_string).collect::<Vec<_>>().join(","))
            .collect::<Vec<_>>()
            .join(";");
        assert_eq!(seen.trim(), want, "walk_data — calls, row `{line}`");
        let want_after = if gate == 0 && w != 0 { 0xffff } else { before };
        assert_eq!(after, want_after, "walk_data — the store, row `{line}`");
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("Dock::walk_data@00740c40: {n} rows agree");
}
