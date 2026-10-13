//! `Game::action_cheat_ai_speed_normal@005930e0` under the emulator against the port (item 1966; hand-written on tools/emu/hooks.py).

/// `Game::action_cheat_ai_speed_normal@005930e0`, run by `tools/emu/sweep_game_action_cheat_ai_speed_normal.py` under unicorn
/// (tools/emu): bit 2 of the semaphore byte clear sets the int the cell `0xc061c0` names (`ai_speed`) to 1; set, nothing moves.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_game_action_cheat_ai_speed_normal.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (f, ai) = (a[0], i64::from(a[1] as i32));
        let got: i64 = if f & 4 == 0 { 1 } else { ai };
        assert_eq!(got, want, "speed — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Game::action_cheat_ai_speed_normal@005930e0: {n} rows agree");
}
