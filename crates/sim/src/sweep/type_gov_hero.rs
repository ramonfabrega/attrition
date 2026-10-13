//! `TypeData::is_gov_hero_type@00470680` under the emulator against the port (item 1704).

use crate::type_index::is_gov_hero_type;

/// `TypeData::is_gov_hero_type@00470680`, run by `tools/emu/sweep_type_gov_hero.py` under unicorn
/// (tools/emu): `0x15f < t < 0x166` on the type record's `type`, over every index -2..=0x33f and the edges, against `type_index::is_gov_hero_type` — the bounds `is_gov_hero_type`'s readers (`type_index`'s header) stand on.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_type_gov_hero.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let t = i32::try_from(a[0]).expect("an int");
        assert_eq!(
            is_gov_hero_type(t),
            want != 0,
            "is_gov_hero_type — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 800, "only {n} rows");
    eprintln!("TypeData::is_gov_hero_type@00470680: {n} rows agree");
}
