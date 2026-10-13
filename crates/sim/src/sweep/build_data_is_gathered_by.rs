//! `BuildData::is_gathered_by@0062f520` under the emulator against the port (item 2038; hand-written on tools/emu/hooks.py).

/// `BuildData::is_gathered_by@0062f520`, run by `tools/emu/sweep_build_data_is_gathered_by.py` under unicorn
/// (tools/emu): the building's gatherer chain, from `gather_down` through each unit's next link to the first
/// negative one, holds the unit or does not. `orders.rs`'s `is_gathered_by` is the port, the chain a list.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_data_is_gathered_by.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (head, unit, next) = (a[1], a[2], &a[3..]);
        let mut chain = Vec::new();
        let mut at = head;
        while at >= 0 {
            chain.push(at);
            at = next[at as usize];
        }
        assert_eq!(
            i64::from(chain.contains(&unit)),
            want,
            "is_gathered_by — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("BuildData::is_gathered_by@0062f520: {n} rows agree");
}
