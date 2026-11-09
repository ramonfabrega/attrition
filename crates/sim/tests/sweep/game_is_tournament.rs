//! `Game::is_tournament@0043ec90` under the emulator against the port (item 1819).

/// `Game::is_tournament@0043ec90`, run by `tools/emu/sweep_game_is_tournament.py` under unicorn
/// (tools/emu): `__thiscall`, `ecx` = the game; true when the byte at +0x820 has 0x4 and the byte at +0x20 has 0x80 — every byte against the other's clear and set cases.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_game_is_tournament.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let got = i64::from(a[0] & 4 != 0 && a[1] & 0x80 != 0);
        assert_eq!(got, want, "Game::is_tournament — row `{line}`");
        n += 1;
    }
    assert!(n >= 1000, "only {n} rows");
    eprintln!("Game::is_tournament@0043ec90: {n} rows agree");
}
