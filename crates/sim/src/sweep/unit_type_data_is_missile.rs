//! `UnitTypeData::is_missile@0061d430` under the emulator against the port (item 2020; hand-written on tools/emu/hooks.py).

use crate::combat::mask;

/// `UnitTypeData::is_missile@0061d430`, run by `tools/emu/sweep_unit_type_data_is_missile.py` under unicorn
/// (tools/emu): with its own slot `+0xf0` the base object-mask test, the mask word at `+0x1e4` against bit 27
/// with no call; with the slot overridden, the slot asked once with that bit and its answer's sense. The port's
/// flag is [`mask::MISSILE`], `airbase.rs`'s missile skip.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_type_data_is_missile.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (mode, word, ans, calls, handed, ecx_this) = (a[0], a[1], a[2], a[3], a[4], a[5]);
        let got = if mode == 1 {
            assert_eq!(calls, 0, "the base slot is never entered — row `{line}`");
            i64::from(word as u32 & mask::MISSILE != 0)
        } else {
            assert_eq!(
                (calls, handed, ecx_this),
                (1, i64::from(mask::MISSILE), 1),
                "the slot is asked once with the flag — row `{line}`"
            );
            i64::from(ans != 0)
        };
        assert_eq!(got, want, "is_missile — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitTypeData::is_missile@0061d430: {n} rows agree");
}
