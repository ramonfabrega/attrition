//! `UnitData::is_flying_high@0060a310` under the emulator against the port (item 2090; hand-written on tools/emu/hooks.py).

use crate::air::flying_high_by;

/// `UnitData::is_flying_high@0060a310`, run by `tools/emu/sweep_unit_data_is_flying_high.py` under unicorn
/// (tools/emu): the type record's three fixed-wing tests (domain `+0x218` 2, `+0x2b4 & 0x20` and `+0x1e4 &
/// 0x8000000` clear — `Sim::is_fixed_wing`'s, read off the bytes here), then on the map (the unit's slot `+0xbc`
/// the image's own `is_on_map`, bit 15 of `+0x82`, or a stub asked on the unit), then `is_flying_low` asked on
/// the unit. The port's [`flying_high_by`] is handed an `on_map` and a `low` that answer the row and record their
/// asks, so the order and the short circuit are held to the original's calls, and the answer to `eax`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_is_flying_high.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want, tail) = super::tail_row(line);
        let (base, domain, b2b4, w1e4, w82) = (a[0] != 0, a[1], a[2], a[3], a[4]);
        let (slot, low) = (a[5], a[6]);
        let fixed_wing = domain == 2 && b2b4 & 0x20 == 0 && w1e4 & 0x0800_0000 == 0;
        let calls = std::cell::RefCell::new(Vec::new());
        let got = flying_high_by(
            fixed_wing,
            || {
                if base {
                    w82 >> 15 != 0
                } else {
                    calls.borrow_mut().push("is_on_map,unit");
                    slot != 0
                }
            },
            || {
                calls.borrow_mut().push("is_flying_low,unit");
                low != 0
            },
        );
        assert_eq!(
            tail[0],
            calls.borrow().join(";"),
            "is_flying_high — the asks, row `{line}`"
        );
        assert_eq!(i64::from(got), want, "is_flying_high — eax, row `{line}`");
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("UnitData::is_flying_high@0060a310: {n} rows agree");
}
