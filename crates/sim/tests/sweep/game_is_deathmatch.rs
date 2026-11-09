//! `Game::is_deathmatch@00594630` under the emulator against the port (item 1787).

/// `Game::is_deathmatch@00594630`, run by `tools/emu/sweep_game_is_deathmatch.py` under unicorn
/// (tools/emu): the byte at `+0x2d` of the game over all 256 values, against `flag_reads::game_is_deathmatch`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_game_is_deathmatch.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        assert_eq!(
            want,
            i64::from(crate::flag_reads::game_is_deathmatch(
                u8::try_from(a[0]).expect("a byte")
            )),
            "Game::is_deathmatch — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 256, "only {n} rows");
    eprintln!("Game::is_deathmatch@00594630: {n} rows agree");
}
