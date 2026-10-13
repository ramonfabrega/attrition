//! `Game::is_scenario@0043ec70` under the emulator against the port (item 1814).

/// `Game::is_scenario@0043ec70`, run by `tools/emu/sweep_game_is_scenario.py` under unicorn
/// (tools/emu): `__thiscall`, `ecx` = the game; two flag bytes at +0x821 and +0x822 — true when byte 0 has 0x10 or byte 1 has 0x02, over every byte against the other's clear and set cases.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_game_is_scenario.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let got = i64::from(a[0] & 0x10 != 0 || a[1] & 0x02 != 0);
        assert_eq!(got, want, "Game::is_scenario — row `{line}`");
        n += 1;
    }
    assert!(n >= 1000, "only {n} rows");
    eprintln!("Game::is_scenario@0043ec70: {n} rows agree");
}
