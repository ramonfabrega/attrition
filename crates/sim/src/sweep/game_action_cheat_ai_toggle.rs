//! `Game::action_cheat_ai_toggle@005930c0` under the emulator against the port (item 1966; hand-written on tools/emu/hooks.py).

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("hex"))
        .collect()
}

/// `Game::action_cheat_ai_toggle@005930c0`, run by `tools/emu/sweep_game_action_cheat_ai_toggle.py` under unicorn
/// (tools/emu): bit 2 of the semaphore byte clear toggles the int the cell `0xc061c4` names (`ai_off`) between 0 and 1; set, nothing moves.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_game_action_cheat_ai_toggle.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (f, ai) = (a[0], i64::from(a[1] as i32));
        let got: i64 = if f & 4 == 0 { i64::from(ai == 0) } else { ai };
        assert_eq!(got, want, "toggle — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Game::action_cheat_ai_toggle@005930c0: {n} rows agree");
}
