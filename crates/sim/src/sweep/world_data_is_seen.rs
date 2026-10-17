//! `WorldData::is_seen@006b55c0`, `WorldData::is_seen_wcoord@006b2540` and `WorldData::is_really_seen@006b42c0` (no revealed-map exit) under the emulator against the port (item 2837; hand-written on tools/emu/hooks.py).

/// `WorldData::is_seen@006b55c0`, `WorldData::is_seen_wcoord@006b2540` and `WorldData::is_really_seen@006b42c0`, run by `tools/emu/sweep_world_data_is_seen.py` under unicorn (tools/emu). A player of slot 8 or past, a
/// revealed map (`reveal_map == 3`), a leader word with `0x800` or a nonzero short at `+0x59e4` answer 1; else,
/// with `0x2000`, a cell owner that is the leader or an ally both ways (`LeaderData::is_ally`: diplo 2 each
/// side) answers 1; else the fog byte masked by the leader's byte at `+0x6929`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_data_is_seen.py") else {
        return;
    };
    let mut n = 0;
    let mut by_exit = [0usize; 4];
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (who, reveal, lw, nu, mask) = (a[3], a[4], a[5] as i32, a[6], a[7]);
        let (lwho, d_lc, d_cl, owner, cell) = (a[8], a[9], a[10], a[11], a[12]);
        let (got, exit) = if who >= i64::from(crate::world::PLAYER_SLOTS) || (reveal == 3 && a[0] != 2) {
            (1, 0)
        } else if lw & 0x800 != 0 || nu != 0 {
            (1, 0)
        } else if lw & 0x2000 != 0
            && owner >= 0
            && (owner == lwho || (d_lc == 2 && d_cl == 2))
        {
            (1, 1)
        } else {
            (cell & mask, 2)
        };
        by_exit[exit] += 1;
        assert_eq!(got, want, "is_seen (form {}) — row `{line}`", a[0]);
        n += 1;
    }
    assert!(n >= 500, "only {n} rows");
    assert!(by_exit[1] >= 10 && by_exit[2] >= 50, "starved arm {by_exit:?}");
    eprintln!("WorldData::is_seen/is_seen_wcoord/is_really_seen: {n} rows agree {by_exit:?}");
}
