//! `Player::is_over@0043e220` under the emulator against the port (item 1735).

/// `Player::is_over@0043e220`, run by `tools/emu/sweep_player_is_over.py` under unicorn
/// (tools/emu): the 16-bit word at `+0x30`: bit 0x100 or any of 0xd0 set is 1, else 0 (the high half of the planted dword a decoy), against `flag_reads::player_is_over`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_player_is_over.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let w = i32::try_from(a[0]).expect("a 32-bit integer");
        assert_eq!(
            want,
            i64::from(crate::flag_reads::player_is_over(w as u16)),
            "Player::is_over — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Player::is_over@0043e220: {n} rows agree");
}
