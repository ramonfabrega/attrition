//! `Game::teams_locked@00594880` under the emulator against the port (item 1971; hand-written on tools/emu/hooks.py).

/// `Game::teams_locked@00594880`, run by `tools/emu/sweep_game_teams_locked.py` under unicorn
/// (tools/emu): the team-style byte of the game record: 0, 8 and 0xb unlocked, every other value locked.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_game_teams_locked.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let got = i64::from(!matches!(a[0], 0 | 8 | 0xb));
        assert_eq!(got, want, "teams_locked — row `{line}`");
        n += 1;
    }
    assert!(n >= 500, "only {n} rows");
    eprintln!("Game::teams_locked@00594880: {n} rows agree");
}
