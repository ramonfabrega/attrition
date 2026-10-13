//! `TypeData::is_gaia_type@004707a0` under the emulator against the port (item 1705).

use crate::type_index::is_gaia_type;

/// `TypeData::is_gaia_type@004707a0`, run by `tools/emu/sweep_type_gaia.py` under unicorn
/// (tools/emu): `0x191 < t < 0x19e` on the type record's `type`, over every index -2..=0x33f and the edges, against `type_index::is_gaia_type` (0x192..=0x19d).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_type_gaia.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let t = i32::try_from(a[0]).expect("an int");
        assert_eq!(is_gaia_type(t), want != 0, "is_gaia_type — row `{line}`");
        n += 1;
    }
    assert!(n >= 800, "only {n} rows");
    eprintln!("TypeData::is_gaia_type@004707a0: {n} rows agree");
}
