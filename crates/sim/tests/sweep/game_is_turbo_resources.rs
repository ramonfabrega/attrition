//! `Game::is_turbo_resources@0043ecb0` under the emulator against the port (item 1819).

/// `Game::is_turbo_resources@0043ecb0`, run by `tools/emu/sweep_game_is_turbo_resources.py` under unicorn
/// (tools/emu): `__thiscall`, `ecx` = the game; true when the flag byte at +0x20 has 0x2 or the rules byte at +0x2a is 9 — every byte against the other's clear and set cases.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_game_is_turbo_resources.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let got = i64::from(a[0] & 2 != 0 || a[1] == 9);
        assert_eq!(got, want, "Game::is_turbo_resources — row `{line}`");
        n += 1;
    }
    assert!(n >= 1000, "only {n} rows");
    eprintln!("Game::is_turbo_resources@0043ecb0: {n} rows agree");
}
