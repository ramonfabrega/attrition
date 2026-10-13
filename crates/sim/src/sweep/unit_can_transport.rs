//! `UnitData::can_transport@0046f960` under the emulator against the port (item 1701).

use crate::UnitType;
use crate::ai_load::UnitCols;

/// `UnitData::can_transport@0046f960`, run by `tools/emu/sweep_unit_can_transport.py` under unicorn
/// (tools/emu): `(unit_masks & 0x800000 && !(unit_masks2 & 0x2000)) || type byte +0x2b4 & 0x10`, over the cross of the bit edges and a seeded band, against `Sim::unit_can_transport` — the port's `auto_transport`, `never_transport` and `unit_flags & 0x10`, set from the row's words.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_can_transport.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (masks, masks2, tflags) = (a[0], a[1], a[2]);
        let (mut s, u) = super::sim_with_unit(UnitType {
            cols: UnitCols {
                unit_flags: u32::try_from(tflags).expect("a byte"),
                ..UnitCols::default()
            },
            ..UnitType::default()
        });
        s.units[u].auto_transport = masks & 0x80_0000 != 0;
        s.units[u].never_transport = masks2 & 0x2000 != 0;
        assert_eq!(
            s.unit_can_transport(u),
            want != 0,
            "can_transport — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitData::can_transport@0046f960: {n} rows agree");
}
