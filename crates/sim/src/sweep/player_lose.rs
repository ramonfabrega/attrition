//! `Player::lose@006ee250` under the emulator against the port (item 2020; hand-written on tools/emu/hooks.py).

/// `Player::lose@006ee250`, run by `tools/emu/sweep_player_lose.py` under unicorn (tools/emu): flag 0x80 is set,
/// the player's game is ended (hooked, so not backed here), and unless flag 0x40 is set a sound is played on the
/// player's own channel (the byte at `+0x36`, hooked too): 0x7f, you lose, when that channel is the console's,
/// else 0x66, someone else loses. No other byte of the player moves.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_player_lose.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, _) = super::row(line);
        let (flags, play, console) = (a[0], a[1], a[2]);
        assert_eq!(a[3], flags | 0x80, "the flags — row `{line}`");
        assert_eq!(a[4], 1, "the game is ended once, on this — row `{line}`");
        let sound = (flags & 0x40 == 0).then_some(if play == console { 0x7f } else { 0x66 });
        let got = match sound {
            Some(s) => (1, play, s),
            None => (0, 0, 0),
        };
        assert_eq!((a[5], a[6], a[7]), got, "the sound — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Player::lose@006ee250: {n} rows agree");
}
