//! `TypeData::is_wonder_type@00470780` under the emulator against the port (item 1705).

use crate::type_index::is_wonder_type;

/// `TypeData::is_wonder_type@00470780`, run by `tools/emu/sweep_type_wonder.py` under unicorn
/// (tools/emu): `0x20d < t < 0x21f` on the type record's `type`, over every index -2..=0x33f and the edges, against `type_index::is_wonder_type` (0x20e..=0x21e).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_type_wonder.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let t = i32::try_from(a[0]).expect("an int");
        assert_eq!(
            is_wonder_type(t),
            want != 0,
            "is_wonder_type — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 800, "only {n} rows");
    eprintln!("TypeData::is_wonder_type@00470780: {n} rows agree");
}
