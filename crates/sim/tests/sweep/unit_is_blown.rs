//! `UnitData::is_blown@0060a6e0` under the emulator against the port (item 1705).

use crate::flag_reads::unit_is_blown;

/// `UnitData::is_blown@0060a6e0`, run by `tools/emu/sweep_unit_is_blown.py` under unicorn
/// (tools/emu): `unit_masks` (`+0x68`) `& 0x1000`, over every bit edge and a seeded band, against `flag_reads::unit_is_blown` — what is proved is the bit's identity (0x1000), that the other bits and the sign do not leak, and that the original returns the masked value itself.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_is_blown.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let word = u32::try_from(a[0]).expect("a u32");
        assert_eq!(
            unit_is_blown(word),
            want != 0,
            "unit_is_blown — row `{line}`"
        );
        assert_eq!(
            want,
            i64::from(word & 0x1000),
            "unit_is_blown — returns the masked word, row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitData::is_blown@0060a6e0: {n} rows agree");
}
