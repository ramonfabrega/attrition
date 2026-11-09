//! `MapFairness::point_lowers_highest@0068a1b0` under the emulator against the port (item 1866).

/// `MapFairness::point_lowers_highest@0068a1b0`, run by `tools/emu/sweep_map_fairness_point_lowers_highest.py` under unicorn
/// (tools/emu): `lowest_dist == highest_player`, the two ints at +0x4c and +0x48.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_map_fairness_point_lowers_highest.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        assert_eq!(
            want,
            i64::from(a[0] == a[1]),
            "MapFairness::point_lowers_highest — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("MapFairness::point_lowers_highest@0068a1b0: {n} rows agree");
}
