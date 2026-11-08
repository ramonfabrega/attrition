//! `UnitData::is_casting@0060a530` under the emulator against the port (item 2038; hand-written on tools/emu/hooks.py).

/// `UnitData::is_casting@0060a530`, run by `tools/emu/sweep_unit_data_is_casting.py` under unicorn
/// (tools/emu): the unit's own record's word at `+0x68` answers for two spells only — bit `0x8000` for `0x27c`,
/// bit `0x2000` for `0x27b` — and every other spell is no. `cast.rs`'s `is_casting` is the port's question, asked
/// of the head order rather than a flag word.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_is_casting.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (spell, flags) = (a[2], a[3] as i32 as u32);
        let bit = match spell {
            0x27c => 0x8000,
            0x27b => 0x2000,
            _ => 0,
        };
        assert_eq!(
            i64::from(flags & bit != 0),
            want,
            "is_casting — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitData::is_casting@0060a530: {n} rows agree");
}
