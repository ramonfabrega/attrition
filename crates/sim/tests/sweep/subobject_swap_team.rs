//! `SubObject::swap_team@0046f370` under the emulator against the port (item 1723).

/// `SubObject::swap_team@0046f370`, run by `tools/emu/sweep_subobject_swap_team.py` under unicorn
/// (tools/emu): the constant -1, `this` and its argument never read.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_subobject_swap_team.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let _ = a;
        assert_eq!(want, -1, "SubObject::swap_team — row `{line}`");
        n += 1;
    }
    assert!(n >= 5, "only {n} rows");
    eprintln!("SubObject::swap_team@0046f370: {n} rows agree");
}
