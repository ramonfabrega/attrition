//! `UnitData::order_type@00616e80` under the emulator against the port (item 2061; hand-written on
//! tools/emu/hooks.py and tools/emu/orderhead.py).

/// `UnitData::order_type@00616e80`, run by `tools/emu/sweep_unit_data_order_type.py` under unicorn (tools/emu):
/// an empty list (`this+0xdc` null) is `NONE` (`orders::index`, what `orders.rs`'s `order_type` answers for no
/// current order), unasked, and its cursor untouched; a list's cursor is rewound to the head — `+0xd4` the
/// first link, `+0xcc` its order, `+0xd0` its byte, and no other byte of the unit moves — and the head order's
/// kind (its slot `+0x10`, entered by a `jmp` on the order's own `ecx`) is the answer.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_order_type.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, eax, tail) = super::tail_row(line);
        let (has, kind) = (a[0] != 0, a[1]);
        let want = if has {
            (kind, ["kind:1", "nnn"])
        } else {
            (i64::from(crate::orders::index::NONE), ["", "ooo"])
        };
        assert_eq!(
            (eax, tail.as_slice()),
            (want.0, &want.1[..]),
            "order_type — row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitData::order_type@00616e80: {n} rows agree");
}
