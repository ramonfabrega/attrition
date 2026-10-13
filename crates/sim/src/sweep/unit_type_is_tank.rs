//! `UnitTypeData::is_tank@00470450` under the emulator against the port (item 1691).

use crate::ai_load::{UnitCols, uflags};

/// `UnitTypeData::is_tank@00470450`, run by `tools/emu/sweep_unit_type_is_tank.py` under unicorn
/// (tools/emu): `unit_flags` (`type+0x2b4`) `& 0x80000` over every single bit, every all-but-one, all and none, and a seeded random band, against `UnitCols::flag(uflags::TANK)` — the bit `docs/COLLISION.md` §13.3 reads.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_type_is_tank.py") else {
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
        .flag(uflags::TANK);
        assert_eq!(ours, want != 0, "UnitTypeData::is_tank — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitTypeData::is_tank@00470450: {n} rows agree");
}
