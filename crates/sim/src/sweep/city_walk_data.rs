//! `City::walk_data@00489220` under the emulator against the port (item 2688; tools/sweep_gen.py, mix).

/// `City::walk_data@00489220`, run by `tools/emu/sweep_city_walk_data.py` under unicorn
/// (tools/emu) with every callee and the walker's callbacks hooked: 5 step(s) in order, `ecx` and arguments held to the spec.
#[test]
#[allow(clippy::vec_init_then_push)]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_city_walk_data.py") else {
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
        let stale = a[4]; // the slot-4 callee's leftover `ecx` when a step names it
        let _ = (this, g, gate, stale);
        let bit = a[5];
        let _ = bit;
        let mut calls: Vec<Vec<i64>> = Vec::new();
        calls.push(vec![walker, this + 0x4, this + 0x6]);
        if bit != 0 {
            calls.push(vec![walker, this + 0x6, this + 0x72]);
        }
        if bit != 0 && gate == 0 {
            calls.push(vec![this + 0x90, walker]);
        }
        if bit != 0 && gate == 0 {
            calls.push(vec![this + 0xa4, walker]);
        }
        if bit != 0 {
            calls.push(vec![this + 0x74, walker, stale]);
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
    eprintln!("City::walk_data@00489220: {n} rows agree");
}
