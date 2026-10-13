//! `TimeSync::test_accept_ping@00954c60` under the emulator against the port (item 1866).

/// `TimeSync::test_accept_ping@00954c60`, run by `tools/emu/sweep_time_sync_test_accept_ping.py` under unicorn
/// (tools/emu): unsigned `param <= cell`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_time_sync_test_accept_ping.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (cell, p) = (a[0] as u32, a[1] as u32);
        assert_eq!(
            want,
            i64::from(p <= cell),
            "TimeSync::test_accept_ping — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("TimeSync::test_accept_ping@00954c60: {n} rows agree");
}
