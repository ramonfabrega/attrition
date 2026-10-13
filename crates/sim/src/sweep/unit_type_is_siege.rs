//! `UnitTypeData::is_siege@00470460` under the emulator against the port (item 1694).

use crate::ai_load::{UnitCols, uflags};

/// `UnitTypeData::is_siege@00470460`, run by `tools/emu/sweep_unit_type_is_siege.py` under unicorn
/// (tools/emu): `unit_flags` (`type+0x2b4`) `& 0x20000`, over every bit edge and a seeded band, against `UnitCols::flag(uflags::SIEGE)` — the bit the port names as the same predicate.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_type_is_siege.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let word = u32::try_from(a[0]).expect("a 32-bit flags word");
        let ours = UnitCols {
            unit_flags: word,
            ..UnitCols::default()
        }
        .flag(uflags::SIEGE);
        assert_eq!(ours, want != 0, "is_siege — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitTypeData::is_siege@00470460: {n} rows agree");
}
