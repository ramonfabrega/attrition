//! `Player::is_active@0047e0e0` under the emulator against the port (item 1787).

/// `Player::is_active@0047e0e0`, run by `tools/emu/sweep_player_is_active.py` under unicorn
/// (tools/emu): the 16-bit word at `+0x30` over every low byte under six high bytes, against `flag_reads::player_is_active`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_player_is_active.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        assert_eq!(
            want,
            i64::from(crate::flag_reads::player_is_active(
                u16::try_from(a[0]).expect("a word")
            )),
            "Player::is_active — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 1536, "only {n} rows");
    eprintln!("Player::is_active@0047e0e0: {n} rows agree");
}
