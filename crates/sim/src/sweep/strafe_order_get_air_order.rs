//! `StrafeOrder::get_air_order@0047ff60` under the emulator against the port (item 1736).

/// `StrafeOrder::get_air_order@0047ff60`, run by `tools/emu/sweep_strafe_order_get_air_order.py` under unicorn
/// (tools/emu): `this - 0x28` (a pointer cast to the enclosing order), against `wrapping_sub(flag_reads::STRAFE_AIR_ORDER_BASE)`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_strafe_order_get_air_order.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let t = i32::try_from(a[0]).expect("a 32-bit address");
        assert_eq!(
            want,
            i64::from(t.wrapping_sub(crate::flag_reads::STRAFE_AIR_ORDER_BASE)),
            "StrafeOrder::get_air_order — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("StrafeOrder::get_air_order@0047ff60: {n} rows agree");
}
