//! `UnitData::is_special@0046cea0` under the emulator against the port (item 1691).

use crate::UnitType;
use crate::ai_load::UnitCols;

/// `UnitData::is_special@0046cea0`, run by `tools/emu/sweep_unit_is_special.py` under unicorn
/// (tools/emu): `unit_flags2` (`type+0x2b8`) `& 0x10` through the unit's type pointer, over every bit edge and a seeded band, against `Sim::unit_is_special` on a unit whose type carries the word.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_is_special.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let word = u32::try_from(a[0]).expect("a 32-bit flags word");
        let (s, u) = super::sim_with_unit(UnitType {
            cols: UnitCols {
                unit_flags2: word,
                ..UnitCols::default()
            },
            ..UnitType::default()
        });
        assert_eq!(s.unit_is_special(u), want != 0, "is_special — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitData::is_special@0046cea0: {n} rows agree");
}
