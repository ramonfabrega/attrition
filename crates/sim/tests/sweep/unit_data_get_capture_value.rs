//! `UnitData::get_capture_value@00610a70` under the emulator against the port (item 2083; hand-written on
//! tools/emu/hooks.py).

use crate::city::capture_value;

/// `UnitData::get_capture_value@00610a70`, run by `tools/emu/sweep_unit_data_get_capture_value.py` under
/// unicorn (tools/emu): the decoy bit, the type's `is_siege` slot answered per row, the type's `+0x308` and the
/// unit's signed `guy_mark` byte over edges and random words, against [`capture_value`] — its one boolean the
/// decoy bit or the slot's answer, `squad` the type's word and `figures` the byte. The slot is asked only when
/// the unit is not a decoy, and once.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_get_capture_value.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (decoy, siege, squad, mark, calls) = (a[0] != 0, a[1] != 0, a[2], a[3], a[4]);
        assert_eq!(
            calls,
            i64::from(!decoy),
            "get_capture_value — the slot calls, row `{line}`"
        );
        let got = capture_value(
            decoy || siege,
            i32::try_from(squad).expect("a 32-bit word"),
            i32::try_from(mark).expect("a byte"),
        );
        assert_eq!(i64::from(got), want, "get_capture_value — row `{line}`");
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("UnitData::get_capture_value@00610a70: {n} rows agree");
}
