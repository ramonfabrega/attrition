//! `GroupMoveOrder::walk_data_vtordisp@0048562c` under the emulator against the port (item 2374; tools/sweep_gen.py, fwd).

/// `GroupMoveOrder::walk_data_vtordisp@0048562c`, run by `tools/emu/sweep_group_move_order_walk_data_vtordisp.py` under unicorn
/// (tools/emu) with its callees hooked: 1 hooked callee(s), `ecx` and arguments held to the spec, `eax` none. `this` is laid in the heap with a drawn displacement at `this-4` (the row's sixth figure), and the callee's `ecx` is `this` less it.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_move_order_walk_data_vtordisp.py") else {
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
        let calls: Vec<Vec<i64>> = vec![vec![
            i64::from((this as u32).wrapping_sub(disp as u32)),
            _p0,
        ]];
        let want: String = calls
            .iter()
            .map(|c| c.iter().map(i64::to_string).collect::<Vec<_>>().join(","))
            .collect::<Vec<_>>()
            .join(";");
        assert_eq!(
            seen.trim(),
            want,
            "walk_data_vtordisp — calls, row `{line}`"
        );
        let _ = eax;
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("GroupMoveOrder::walk_data_vtordisp@0048562c: {n} rows agree");
}
