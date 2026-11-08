//! `Game::is_never_war@00594660` under the emulator against the port (item 1787).

/// `Game::is_never_war@00594660`, run by `tools/emu/sweep_game_is_never_war.py` under unicorn
/// (tools/emu): the byte at `+0x32` of the game over all 256 values, against `flag_reads::game_is_never_war`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_game_is_never_war.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        assert_eq!(
            want,
            i64::from(crate::flag_reads::game_is_never_war(
                u8::try_from(a[0]).expect("a byte")
            )),
            "Game::is_never_war — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 256, "only {n} rows");
    eprintln!("Game::is_never_war@00594660: {n} rows agree");
}
