//! `UnitData::is_supply@0046ce80` under the emulator against the port (item 1691).

use crate::ai_load::{UnitCols, uflags2};

/// `UnitData::is_supply@0046ce80`, run by `tools/emu/sweep_unit_is_supply.py` under unicorn
/// (tools/emu): `unit_flags2` (`type+0x2b8`) `& 0x40` through the unit's type pointer, over every bit edge and a seeded band, against `UnitCols::flag2(uflags2::SUPPLY_OR_HERO)` — the raw bit `path::army_mode` reads (`Sim::is_supply_unit` adds a general exclusion the original's predicate does not have).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_is_supply.py") else {
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
        .flag2(uflags2::SUPPLY_OR_HERO);
        assert_eq!(ours, want != 0, "UnitData::is_supply — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitData::is_supply@0046ce80: {n} rows agree");
}
