//! `GroupAttackToOrder::get_group_attack_to_order@004825e0` under the emulator against the port (item 1856).

/// `GroupAttackToOrder::get_group_attack_to_order@004825e0`, run by `tools/emu/sweep_group_attack_to_order_get_group_attack_to_order.py` under unicorn
/// (tools/emu): `__thiscall`, no arguments: `lea eax, [ecx - 0x70]` — the embedded base object's address, `this - 0x70`. Row `<this> -> <result>`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_attack_to_order_get_group_attack_to_order.py")
    else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let _ = &a;
        let got: i64 = (a[0] as i32).wrapping_sub(0x70) as i64;
        assert_eq!(
            got, want,
            "GroupAttackToOrder::get_group_attack_to_order — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("GroupAttackToOrder::get_group_attack_to_order@004825e0: {n} rows agree");
}
