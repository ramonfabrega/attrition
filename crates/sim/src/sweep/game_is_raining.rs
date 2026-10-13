//! `Game::is_raining@00594480` under the emulator against the port (item 1735).

/// `Game::is_raining@00594480`, run by `tools/emu/sweep_game_is_raining.py` under unicorn
/// (tools/emu): returns 0 whatever the record holds — the original keeps no rain — against `flag_reads::GAME_IS_RAINING`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_game_is_raining.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (_, want) = super::row(line);
        assert_eq!(
            want,
            i64::from(crate::flag_reads::GAME_IS_RAINING),
            "Game::is_raining — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 60, "only {n} rows");
    eprintln!("Game::is_raining@00594480: {n} rows agree");
}
