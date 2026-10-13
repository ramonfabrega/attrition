//! `PatrolOrder::get_patrol_order@004843e0` under the emulator against the port (item 1736).

/// `PatrolOrder::get_patrol_order@004843e0`, run by `tools/emu/sweep_patrol_order_get_patrol_order.py` under unicorn
/// (tools/emu): `this - 0x44` (a pointer cast to the enclosing order), against `wrapping_sub(flag_reads::PATROL_ORDER_BASE)`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_patrol_order_get_patrol_order.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let t = i32::try_from(a[0]).expect("a 32-bit address");
        assert_eq!(
            want,
            i64::from(t.wrapping_sub(crate::flag_reads::PATROL_ORDER_BASE)),
            "PatrolOrder::get_patrol_order — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("PatrolOrder::get_patrol_order@004843e0: {n} rows agree");
}
