//! `Unit::do_spec_unit_anim@005e5be0` under the emulator against the port (item 1889).

/// `Unit::do_spec_unit_anim@005e5be0`, run by `tools/emu/sweep_unit_do_spec_unit_anim.py` under unicorn
/// (tools/emu): nothing is written: `0` changed bytes.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_do_spec_unit_anim.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (_, want) = super::row(line);
        assert_eq!(want, 0, "Unit::do_spec_unit_anim — row `{line}`");
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("Unit::do_spec_unit_anim@005e5be0: {n} rows agree");
}
