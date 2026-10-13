//! `TurnControl::is_cannon_time@00956ac0` under the emulator against the port (item 1793).

/// `TurnControl::is_cannon_time@00956ac0`, run by `tools/emu/sweep_turn_control_is_cannon_time.py` under unicorn
/// (tools/emu): the global cell `0x00e863f4` over the signed edges and random words (the stack argument varied, unread), against "non-negative".
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_turn_control_is_cannon_time.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let g = i32::try_from(a[0]).expect("a 32-bit integer");
        assert_eq!(want, i64::from(g >= 0), "TurnControl::is_cannon_time — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("TurnControl::is_cannon_time@00956ac0: {n} rows agree");
}
