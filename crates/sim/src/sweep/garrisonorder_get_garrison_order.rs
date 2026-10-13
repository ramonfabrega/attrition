//! `GarrisonOrder::get_garrison_order@00482dd0` under the emulator against the port (item 1782).

/// `GarrisonOrder::get_garrison_order@00482dd0`, run by `tools/emu/sweep_garrisonorder_get_garrison_order.py` under unicorn
/// (tools/emu): `this - 0x1c`, against `wrapping_sub(flag_reads::GARRISON_ORDER_BASE)`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_garrisonorder_get_garrison_order.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let t = i32::try_from(a[0]).expect("a 32-bit address");
        assert_eq!(
            want,
            i64::from(t.wrapping_sub(crate::flag_reads::GARRISON_ORDER_BASE)),
            "GarrisonOrder::get_garrison_order — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("GarrisonOrder::get_garrison_order@00482dd0: {n} rows agree");
}
