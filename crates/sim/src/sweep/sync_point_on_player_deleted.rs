//! `SyncPoint::on_player_deleted@0093a140` under the emulator against the port (item 1889).

/// `SyncPoint::on_player_deleted@0093a140`, run by `tools/emu/sweep_sync_point_on_player_deleted.py` under unicorn
/// (tools/emu): nothing is written: `0` changed bytes.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_sync_point_on_player_deleted.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (_, want) = super::row(line);
        assert_eq!(want, 0, "SyncPoint::on_player_deleted — row `{line}`");
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("SyncPoint::on_player_deleted@0093a140: {n} rows agree");
}
