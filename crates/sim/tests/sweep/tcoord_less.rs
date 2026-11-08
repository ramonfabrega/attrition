//! `TCoord::operator<@00472b30` under the emulator against the port (item 1708).

/// `TCoord::operator<` (the export's `tcoord::operator@00472b30`, a name the census's citation pattern cannot spell with its `<`), run by `tools/emu/sweep_tcoord_less.py` under unicorn
/// (tools/emu): `this->value < other->value`, a signed 32-bit compare, over the word's edges squared, equal pairs and a seeded band, against `flag_reads::tcoord_less`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_tcoord_less.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (x, y) = (
            i32::try_from(a[0]).expect("a word"),
            i32::try_from(a[1]).expect("a word"),
        );
        assert_eq!(
            i64::from(crate::flag_reads::tcoord_less(x, y)),
            want,
            "TCoord::operator< — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("TCoord::operator<@00472b30: {n} rows agree");
}
