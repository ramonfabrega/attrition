//! `UnitData::decoy_time@00609d70` under the emulator against the port (item 2072; hand-written on
//! tools/emu/hooks.py).

use crate::cast::decoy_time_left;

/// `UnitData::decoy_time@00609d70`, run by `tools/emu/sweep_unit_data_decoy_time.py` under unicorn (tools/emu):
/// the owner's general upgrade (a hooked callee, asked once of the owner's leader record), `DECOY_TIME` and the
/// unit's `mana_burn` over the ranges a game holds and their edges, against [`decoy_time_left`] — the port's
/// decoy close in `Unit::process`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_decoy_time.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        assert_eq!(
            (a[4], a[5]),
            (1, 1),
            "decoy_time — the leader asked once, row `{line}`"
        );
        let g = i32::try_from(a[1]).expect("an upgrade");
        let dt = i32::try_from(a[2]).expect("a constant");
        let burn = i16::try_from(a[3]).expect("a short");
        assert_eq!(
            i64::from(decoy_time_left(g, dt, burn)),
            want,
            "decoy_time — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 300, "only {n} rows");
    eprintln!("UnitData::decoy_time@00609d70: {n} rows agree");
}
