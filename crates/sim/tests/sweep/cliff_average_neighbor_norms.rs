//! `Cliff::average_neighbor_norms@008a4060` under the emulator against the port (item 1889).

/// `Cliff::average_neighbor_norms@008a4060`, run by `tools/emu/sweep_cliff_average_neighbor_norms.py` under unicorn
/// (tools/emu): nothing is written: `0` changed bytes.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_cliff_average_neighbor_norms.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (_, want) = super::row(line);
        assert_eq!(want, 0, "Cliff::average_neighbor_norms — row `{line}`");
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("Cliff::average_neighbor_norms@008a4060: {n} rows agree");
}
