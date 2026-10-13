//! `TargetOrder::get_target_order@0047ff40` under the emulator against the port (item 1735).

/// `TargetOrder::get_target_order@0047ff40`, run by `tools/emu/sweep_target_order_get_target_order.py` under unicorn
/// (tools/emu): `this - 0x18` (a pointer cast to the enclosing order), against `wrapping_sub(flag_reads::TARGET_ORDER_BASE)`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_target_order_get_target_order.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let t = i32::try_from(a[0]).expect("a 32-bit address");
        assert_eq!(
            want,
            i64::from(t.wrapping_sub(crate::flag_reads::TARGET_ORDER_BASE)),
            "TargetOrder::get_target_order — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("TargetOrder::get_target_order@0047ff40: {n} rows agree");
}
