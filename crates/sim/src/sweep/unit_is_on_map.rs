//! `UnitData::is_on_map@0046ce30` under the emulator against the port (item 1694).

use crate::UnitType;

/// `UnitData::is_on_map@0046ce30`, run by `tools/emu/sweep_unit_is_on_map.py` under unicorn
/// (tools/emu): `inside_up >> 15`, `inside_up` the signed short at `unit+0x82`, over the short's edges and a seeded band. The port keeps the flag as `Unit::on_map`, a bool, where the original keeps the short's sign bit; the row sets the port's bool from the sign and reads it back, so what is proved is the sign-bit test and that the answer is 0 or 1 — the port's `on_map == (inside_up < 0)`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_is_on_map.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (mut s, u) = super::sim_with_unit(UnitType::default());
        s.units[u].on_map = a[0] < 0;
        assert_eq!(
            i64::from(s.units[u].on_map),
            want,
            "is_on_map — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitData::is_on_map@0046ce30: {n} rows agree");
}
