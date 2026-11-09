//! `Storm::on_field_change@00842050` under the emulator against the port (item 1723).

/// `Storm::on_field_change@00842050`, run by `tools/emu/sweep_storm_on_field_change.py` under unicorn
/// (tools/emu): the constant 0, `this` never read.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_storm_on_field_change.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let _ = a;
        assert_eq!(want, 0, "Storm::on_field_change — row `{line}`");
        n += 1;
    }
    assert!(n >= 5, "only {n} rows");
    eprintln!("Storm::on_field_change@00842050: {n} rows agree");
}
