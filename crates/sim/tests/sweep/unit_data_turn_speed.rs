//! `UnitData::turn_speed@0060a600` under the emulator against the port (item 1976; hand-written on tools/emu/hooks.py).

/// `UnitData::turn_speed@0060a600`, run by `tools/emu/sweep_unit_data_turn_speed.py` under unicorn
/// (tools/emu): `(word >> 8) * turn`, times the pack bonus when mask `0x80000` is set, all 32-bit wrapping.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_turn_speed.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line.split_once(" -> ").expect("`<args…> -> <result>`");
        let a: Vec<i64> = lhs
            .split_whitespace()
            .map(|s| s.parse().expect("an integer"))
            .collect();
        let rhs = rhs.trim();
        let base = ((a[0] as u32 >> 8) as i32).wrapping_mul(a[1] as i32);
        let got = if (a[3] as u32) & 0x80000 != 0 {
            (a[2] as i32).wrapping_mul(base)
        } else {
            base
        };
        assert_eq!(
            i64::from(got),
            rhs.parse::<i64>().unwrap(),
            "turn_speed — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 300, "only {n} rows");
    eprintln!("UnitData::turn_speed@0060a600: {n} rows agree");
}
