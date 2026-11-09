//! `TypeData::is_final_type@00470660` under the emulator against the port (item 1704).

use crate::type_index::is_final_type;

/// `TypeData::is_final_type@00470660`, run by `tools/emu/sweep_type_final.py` under unicorn
/// (tools/emu): `0x242 < t < 0x247` on the type record's `type`, over every index -2..=0x33f and the edges, against `type_index::is_final_type` — the bounds `is_final_type`'s readers (`type_index`'s header) stand on.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_type_final.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let t = i32::try_from(a[0]).expect("an int");
        assert_eq!(is_final_type(t), want != 0, "is_final_type — row `{line}`");
        n += 1;
    }
    assert!(n >= 800, "only {n} rows");
    eprintln!("TypeData::is_final_type@00470660: {n} rows agree");
}
