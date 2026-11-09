//! `WorldData::was_really_seen@006b54f0` under the emulator against the port (item 2044; hand-written on tools/emu/hooks.py).

/// `WorldData::was_really_seen@006b54f0`, run by `tools/emu/sweep_world_data_was_really_seen.py` under unicorn
/// (tools/emu): a player of slot 8 or past, a revealed map (`reveal_map == 3`), a leader word with `0x800` or a
/// nonzero short at `+0x59e4` answer 1; else the fog byte at `x + width·y` masked by the leader's byte at
/// `+0x6929`. `scout.rs`'s `was_really_seen_fog` is the port: its first two exits are these, and the two
/// leader exits are its stated seams.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_data_was_really_seen.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (who, reveal, lw, l59, mask, cell) = (a[2], a[3], a[4] as i32, a[5], a[6], a[8]);
        let got = if who >= i64::from(crate::world::PLAYER_SLOTS)
            || reveal == 3
            || lw & 0x800 != 0
            || l59 != 0
        {
            1
        } else {
            cell & mask
        };
        assert_eq!(got, want, "was_really_seen — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("WorldData::was_really_seen@006b54f0: {n} rows agree");
}
