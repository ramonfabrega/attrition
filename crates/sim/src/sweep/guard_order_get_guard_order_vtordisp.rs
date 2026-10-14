//! `GuardOrder::get_guard_order_vtordisp@00486924` under the emulator against the port (item 2374; tools/sweep_gen.py, fwd).

/// `GuardOrder::get_guard_order_vtordisp@00486924`, run by `tools/emu/sweep_guard_order_get_guard_order_vtordisp.py` under unicorn
/// (tools/emu) with its callees hooked: 1 hooked callee(s), `ecx` and arguments held to the spec, `eax` last. `this` is laid in the heap with a drawn displacement at `this-4` (the row's sixth figure), and the callee's `ecx` is `this` less it.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_guard_order_get_guard_order_vtordisp.py") else {
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
        let (this, _p0, ans, _p1, _p2) = (a[0], a[1], a[2], a[3], a[4]);
        let _ = (this, _p0, ans, _p1, _p2);
        let disp = a[5];
        let calls: Vec<Vec<i64>> = vec![vec![i64::from((this as u32).wrapping_sub(disp as u32))]];
        let want: String = calls
            .iter()
            .map(|c| c.iter().map(i64::to_string).collect::<Vec<_>>().join(","))
            .collect::<Vec<_>>()
            .join(";");
        assert_eq!(
            seen.trim(),
            want,
            "get_guard_order_vtordisp — calls, row `{line}`"
        );
        let want_eax = ans as i32;
        assert_eq!(
            eax.trim().parse::<i64>().unwrap(),
            i64::from(want_eax),
            "get_guard_order_vtordisp — eax, row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("GuardOrder::get_guard_order_vtordisp@00486924: {n} rows agree");
}
