//! `UnitData::is_hero@0046ce60` under the emulator against the port (item 1694).

use crate::ai_load::{UnitCols, uflags2};

/// `UnitData::is_hero@0046ce60`, run by `tools/emu/sweep_unit_is_hero.py` under unicorn
/// (tools/emu): `unit_flags2` (`type+0x2b8`) `& 0x20` through the unit's type pointer, over every bit edge and a seeded band, against `UnitCols::flag2(uflags2::GENERAL)` — the bit the port names as the same predicate.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_is_hero.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let word = u32::try_from(a[0]).expect("a 32-bit flags word");
        let ours = UnitCols {
            unit_flags2: word,
            ..UnitCols::default()
        }
        .flag2(uflags2::GENERAL);
        assert_eq!(ours, want != 0, "is_hero — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitData::is_hero@0046ce60: {n} rows agree");
}
