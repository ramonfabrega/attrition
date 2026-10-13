//! `AirAttackGroundOrder::get_air_order@004835b0` under the emulator against the port (item 1782).

/// `AirAttackGroundOrder::get_air_order@004835b0`, run by `tools/emu/sweep_airattackgroundorder_get_air_order.py` under unicorn
/// (tools/emu): `this - 0x2c`, against `wrapping_sub(flag_reads::AIR_ATTACK_GROUND_ORDER_BASE)`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_airattackgroundorder_get_air_order.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let t = i32::try_from(a[0]).expect("a 32-bit address");
        assert_eq!(
            want,
            i64::from(t.wrapping_sub(crate::flag_reads::AIR_ATTACK_GROUND_ORDER_BASE)),
            "AirAttackGroundOrder::get_air_order — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("AirAttackGroundOrder::get_air_order@004835b0: {n} rows agree");
}
