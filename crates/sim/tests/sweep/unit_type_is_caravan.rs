//! `UnitTypeData::is_caravan@00470420` under the emulator against the port (item 1694).

use crate::ai_load::{UnitCols, uflags2};

/// `UnitTypeData::is_caravan@00470420`, run by `tools/emu/sweep_unit_type_is_caravan.py` under unicorn
/// (tools/emu): `unit_flags2` (`type+0x2b8`) `& 0x8`, over every bit edge and a seeded band, against `UnitCols::flag2(uflags2::CARAVAN)` — the bit the port names as the same predicate.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_type_is_caravan.py") else {
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
        .flag2(uflags2::CARAVAN);
        assert_eq!(ours, want != 0, "is_caravan — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitTypeData::is_caravan@00470420: {n} rows agree");
}
