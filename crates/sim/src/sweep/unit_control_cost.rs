//! `UnitData::control_cost@00609d40` under the emulator against the port (item 1701).

use crate::UnitType;
use crate::cost::Price;

/// `UnitData::control_cost@00609d40`, run by `tools/emu/sweep_unit_control_cost.py` under unicorn
/// (tools/emu): 0 when the argument is 0 and `unit_masks & 1` (a decoy), else `type+0x2f0` — the `POP` column — over the cross of the decoy bit, the argument and the edges, against `UnitType::price.pop` and `Unit::decoy`. The port has no function of its own (`ai_units` reads `price.pop` inline), so the row asserts the rule from the port's two fields.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_control_cost.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (masks, arg, pop) = (a[0], a[1], a[2]);
        let (mut s, u) = super::sim_with_unit(UnitType {
            price: Price {
                pop: i32::try_from(pop).expect("a 32-bit integer"),
                ..Price::default()
            },
            ..UnitType::default()
        });
        s.units[u].decoy = masks & 1 != 0;
        let ours = if arg == 0 && s.units[u].decoy {
            0
        } else {
            i64::from(s.unit_types[s.units[u].ty.expect("typed")].price.pop)
        };
        assert_eq!(ours, want, "control_cost — row `{line}`");
        n += 1;
    }
    assert!(n >= 300, "only {n} rows");
    eprintln!("UnitData::control_cost@00609d40: {n} rows agree");
}
