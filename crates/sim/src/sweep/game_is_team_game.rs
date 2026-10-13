//! `Game::is_team_game@00593f70` under the emulator against the port (item 1766).

/// `Game::is_team_game@00593f70`, run by `tools/emu/sweep_game_is_team_game.py` under unicorn
/// (tools/emu): the byte at `+0x820` over all 256 values, against `flag_reads::game_is_team_game`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_game_is_team_game.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        assert_eq!(
            want != 0,
            crate::flag_reads::game_is_team_game(u8::try_from(a[0]).expect("a byte")),
            "Game::is_team_game — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 256, "only {n} rows");
    eprintln!("Game::is_team_game@00593f70: {n} rows agree");
}
