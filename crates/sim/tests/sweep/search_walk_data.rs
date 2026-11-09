//! `Search::walk_data@0067e600` under the emulator against the port (item 1889).

/// `Search::walk_data@0067e600`, run by `tools/emu/sweep_search_walk_data.py` under unicorn
/// (tools/emu): nothing is written: `0` changed bytes.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_search_walk_data.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (_, want) = super::row(line);
        assert_eq!(want, 0, "Search::walk_data — row `{line}`");
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("Search::walk_data@0067e600: {n} rows agree");
}
