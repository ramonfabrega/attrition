//! `TypeData::is_good_type@00470800` under the emulator against the port (item 1705).

use crate::type_index::is_good_type;

/// `TypeData::is_good_type@00470800`, run by `tools/emu/sweep_type_good.py` under unicorn
/// (tools/emu): `t < 0x32` on the type record's `type`, over every index -2..=0x33f and the edges, against `type_index::is_good_type` (everything below `0x32`, negatives included).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_type_good.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let t = i32::try_from(a[0]).expect("an int");
        assert_eq!(is_good_type(t), want != 0, "is_good_type — row `{line}`");
        n += 1;
    }
    assert!(n >= 800, "only {n} rows");
    eprintln!("TypeData::is_good_type@00470800: {n} rows agree");
}
