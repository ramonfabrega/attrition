//! `GroupData::get_speed@0070f8a0` under the emulator against the port (item 1723).

/// `GroupData::get_speed@0070f8a0`, run by `tools/emu/sweep_group_get_speed.py` under unicorn
/// (tools/emu): the int `speed` at `this+0x40`, over 32-bit edges and a seeded band, against `group::GroupState::speed` (`+0x40`, `docs/GROUPS.md` §14).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_get_speed.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let v = i32::try_from(a[0]).expect("a 32-bit integer");
        assert_eq!(
            want,
            i64::from(v),
            "GroupData::get_speed — the raw word, row `{line}`"
        );
        let g = crate::group::GroupState {
            speed: v,
            ..Default::default()
        };
        assert_eq!(want, i64::from(g.speed), "GroupState::speed — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("GroupData::get_speed@0070f8a0: {n} rows agree");
}
