//! `Unit::air_turn_speed@005ea390` under the emulator against the port (item 2061; hand-written on
//! tools/emu/hooks.py).

/// `Unit::air_turn_speed@005ea390`, run by `tools/emu/sweep_unit_air_turn_speed.py` under unicorn (tools/emu),
/// held to `air.rs`'s `air_turn_speed(base, roll, sign)` on the arm the port models — the second argument 0
/// and the type's `+0x2b4 & 0x20` clear — with `base` the type's `+0x2c4 >> 8` times the constants'
/// `unit_turn_speed`, wrapping, and `roll` the guy's stored bank, a single drawn inside `cvttss2si`'s range
/// (past it the port saturates, `single.rs`).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_air_turn_speed.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, eax) = super::row(line);
        let base = ((a[0] as i32 as u32) >> 8).wrapping_mul(a[1] as i32 as u32);
        let roll = crate::single::Single(a[2] as u32);
        let got = crate::air::air_turn_speed(base, roll, a[3] as i32);
        assert_eq!(eax as i32 as u32, got, "air_turn_speed — row `{line}`");
        n += 1;
    }
    assert!(n >= 300, "only {n} rows");
    eprintln!("Unit::air_turn_speed@005ea390: {n} rows agree");
}
