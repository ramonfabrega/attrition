//! `OrderList::get_data_type@0046ed70` under the emulator against the port (item 1856).

/// `OrderList::get_data_type@0046ed70`, run by `tools/emu/sweep_order_list_get_data_type.py` under unicorn
/// (tools/emu): `__thiscall`, one stack argument (`ret 4`): the order pointer; its vtable slot `+0x10` is called with `ecx` = the order and its answer returned. The slot here is `lea eax, [ecx + v]`, so the answer carries `this` and `v`. Row `<order> <v> -> <result>`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_order_list_get_data_type.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let _ = &a;
        let got: i64 = (a[0] as i32).wrapping_add(a[1] as i32) as i64;
        assert_eq!(got, want, "OrderList::get_data_type — row `{line}`");
        n += 1;
    }
    assert!(n >= 100, "only {n} rows");
    eprintln!("OrderList::get_data_type@0046ed70: {n} rows agree");
}
