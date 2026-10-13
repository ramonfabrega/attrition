//! `AirAttackGroundOrder::get_air_attack_ground_order@00487840` under the emulator against the port (item 1856).

/// `AirAttackGroundOrder::get_air_attack_ground_order@00487840`, run by `tools/emu/sweep_air_attack_ground_order_get_air_attack_ground_order.py` under unicorn
/// (tools/emu): `__thiscall`, no arguments: `lea eax, [ecx - 0x40]` — the embedded base object's address, `this - 0x40`. Row `<this> -> <result>`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) =
        super::emu_rows("sweep_air_attack_ground_order_get_air_attack_ground_order.py")
    else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let _ = &a;
        let got: i64 = (a[0] as i32).wrapping_sub(0x40) as i64;
        assert_eq!(
            got, want,
            "AirAttackGroundOrder::get_air_attack_ground_order — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("AirAttackGroundOrder::get_air_attack_ground_order@00487840: {n} rows agree");
}
