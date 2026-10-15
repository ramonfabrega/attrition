//! `Player::walk_data@006ee2d0` under the emulator against the port (item 2562; tools/sweep_gen.py, mix).

/// `Player::walk_data@006ee2d0`, run by `tools/emu/sweep_player_walk_data.py` under unicorn
/// (tools/emu) with every callee and the walker's callbacks hooked: 4 step(s) in order, `ecx` and arguments held to the spec.
#[test]
#[allow(clippy::vec_init_then_push)]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_player_walk_data.py") else {
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
        let stale = gate; // the slot-4 callee's leftover `ecx` when a step names it
        let _ = (this, g, gate, stale);
        let mut calls: Vec<Vec<i64>> = Vec::new();
        calls.push(vec![walker, g + 0x17a48]);
        calls.push(vec![walker, this + 0x30, this + 0x32]);
        if gate != 0 {
            calls.push(vec![walker, this, this + 0x39]);
        }
        if gate != 0 {
            calls.push(vec![this + 0x40, walker]);
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
    eprintln!("Player::walk_data@006ee2d0: {n} rows agree");
}
