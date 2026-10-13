//! `AttackOrder::get_type@0047ef80` under the emulator against the port (item 1708).

use crate::orders::{Order, *};

/// `AttackOrder::get_type@0047ef80`, run by `tools/emu/sweep_attack_order_get_type.py` under unicorn
/// (tools/emu): the constant `ATTACK` through `eax`, whatever `this` points at, against `Order::index` of a fresh `Body::Attack` order — the port's own `OrderIndex` for the class.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_attack_order_get_type.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let order = Order {
            flags: 0,
            body: Body::Attack(AttackOrder {
                defensive: false,
                def: None,
                in_range: false,
                ever_in_range: false,
                new_ord: false,
            }),
        };
        assert_eq!(
            i64::from(order.index()),
            want,
            "AttackOrder::get_type — row `{line}`"
        );
        let _ = a;
        n += 1;
    }
    assert!(n >= 4, "only {n} rows");
    eprintln!("AttackOrder::get_type@0047ef80: {n} rows agree");
}
