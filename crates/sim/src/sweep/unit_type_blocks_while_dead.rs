//! `UnitTypeData::blocks_while_dead@00470440` under the emulator against the port (item 1694).

use crate::ai_load::UnitCols;

/// `UnitTypeData::blocks_while_dead@00470440`, run by `tools/emu/sweep_unit_type_blocks_while_dead.py` under unicorn
/// (tools/emu): `unit_flags` (`type+0x2b4`) `& 0x800000`, over every bit edge and a seeded band, against `UnitCols::flag(0x80_0000)` — the bit the port names as the same predicate.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_type_blocks_while_dead.py") else {
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
        .flag(0x80_0000);
        assert_eq!(ours, want != 0, "blocks_while_dead — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitTypeData::blocks_while_dead@00470440: {n} rows agree");
}
