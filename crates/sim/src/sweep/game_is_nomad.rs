//! `Game::is_nomad@00594650` under the emulator against the port (item 1787).

/// `Game::is_nomad@00594650`, run by `tools/emu/sweep_game_is_nomad.py` under unicorn
/// (tools/emu): the byte at `+0x2c` of the game over all 256 values, against `flag_reads::game_is_nomad`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_game_is_nomad.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        assert_eq!(
            want,
            i64::from(crate::flag_reads::game_is_nomad(
                u8::try_from(a[0]).expect("a byte")
            )),
            "Game::is_nomad — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 256, "only {n} rows");
    eprintln!("Game::is_nomad@00594650: {n} rows agree");
}
