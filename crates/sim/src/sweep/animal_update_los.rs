//! `Animal::update_los@005d7d50` under the emulator against the port (item 1724).

/// `Animal::update_los@005d7d50`, run by `tools/emu/sweep_animal_update_los.py` under unicorn
/// (tools/emu): zeroes the byte `field_0x3c` (a char, not a dword), touches its neighbours not, returns 0.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_animal_update_los.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let expect = if a[0] == 0x3c || a[0] == -1 { 0 } else { 0xa5 };
        assert_eq!(want, expect, "Animal::update_los — row `{line}`");
        n += 1;
    }
    assert!(n >= 6, "only {n} rows");
    eprintln!("Animal::update_los@005d7d50: {n} rows agree");
}
