//! `TechType::walk_rules_data@0066d5c0` under the emulator against the port (item 2672; tools/sweep_gen.py, mix).

/// `TechType::walk_rules_data@0066d5c0`, run by `tools/emu/sweep_tech_type_walk_rules_data.py` under unicorn
/// (tools/emu) with every callee and the walker's callbacks hooked: 11 step(s) in order, `ecx` and arguments held to the spec.
#[test]
#[allow(clippy::vec_init_then_push)]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_tech_type_walk_rules_data.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> -> <result>`");
        let a: Vec<i64> = lhs.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let (eax, seen) = rhs.split_once(" | ").expect("`<eax> | <calls>`");
        let eax: i64 = eax.trim().parse().expect("an integer");
        let (this, walker, g, gate) = (a[0], a[1], a[2], a[3]);
        let _ = eax;
        let stale = g; // the slot-4 callee's leftover `ecx` when a step names it
        let _ = (this, g, gate, stale);
        let mut calls: Vec<Vec<i64>> = Vec::new();
        calls.push(vec![walker, this + 0x4, this + 0x5e]);
        if gate == 0 { calls.push(vec![this + 0x74, walker]); }
        calls.push(vec![walker, this + 0x1c8, this + 0x1e3]);
        if gate == 0 { calls.push(vec![this + 0x1e4, walker]); }
        if gate == 0 { calls.push(vec![this + 0x1f8, walker]); }
        if gate == 0 { calls.push(vec![this + 0x20c, walker]); }
        if gate == 0 { calls.push(vec![this + 0x220, walker]); }
        if gate == 0 { calls.push(vec![this + 0x234, walker]); }
        if gate == 0 { calls.push(vec![this + 0x248, walker]); }
        if gate == 0 { calls.push(vec![this + 0x25c, walker]); }
        if gate == 0 { calls.push(vec![this + 0x270, walker]); }
        let want: String = calls
            .iter()
            .map(|c| c.iter().map(i64::to_string).collect::<Vec<_>>().join(","))
            .collect::<Vec<_>>()
            .join(";");
        assert_eq!(seen.trim(), want, "walk_rules_data — calls, row `{line}`");
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("TechType::walk_rules_data@0066d5c0: {n} rows agree");
}
