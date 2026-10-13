//! `Game::is_no_rush@005946c0` under the emulator against the port (item 1787).

/// `Game::is_no_rush@005946c0`, run by `tools/emu/sweep_game_is_no_rush.py` under unicorn
/// (tools/emu): the byte at `+0x32` of the game over all 256 values, against `flag_reads::game_is_no_rush`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_game_is_no_rush.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        assert_eq!(
            want,
            i64::from(crate::flag_reads::game_is_no_rush(
                u8::try_from(a[0]).expect("a byte")
            )),
            "Game::is_no_rush — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 256, "only {n} rows");
    eprintln!("Game::is_no_rush@005946c0: {n} rows agree");
}
