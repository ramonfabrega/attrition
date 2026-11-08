//! `AnimalData::valid_target_const@005d8120` under the emulator against the port (item 1998; hand-written on tools/emu/hooks.py).

/// `AnimalData::valid_target_const@005d8120`, run by `tools/emu/sweep_animal_data_valid_target_const.py` under unicorn
/// (tools/emu): 0 when `o` or `who` is negative, else bit 0 of the byte at `+8` of `objects.list[who][o]` — the
/// listing's reading; the decompile prints `return 0` alone. No port carries the animal's override (`fight.rs`'s
/// `valid_target_const` is `ObjectData`'s), so the row is held to the listing's arithmetic.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_animal_data_valid_target_const.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (o, who, b) = (a[0], a[1], a[2]);
        let got = if o < 0 || who < 0 { 0 } else { b & 1 };
        assert_eq!(got, want, "valid_target_const — row `{line}`");
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("AnimalData::valid_target_const@005d8120: {n} rows agree");
}
