//! `ThinkOrder::get_type@004845c0` under the emulator against the port (item 1708).

use crate::orders::{Order, *};
use crate::world::Pos;

/// `ThinkOrder::get_type@004845c0`, run by `tools/emu/sweep_think_order_get_type.py` under unicorn
/// (tools/emu): the constant `THINK` through `eax`, whatever `this` points at, against `Order::index` of a fresh `Body::Think` order — the port's own `OrderIndex` for the class.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_think_order_get_type.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let order = Order {
            flags: 0,
            body: Body::Think,
        };
        assert_eq!(
            i64::from(order.index()),
            want,
            "ThinkOrder::get_type — row `{line}`"
        );
        let _ = a;
        n += 1;
    }
    assert!(n >= 4, "only {n} rows");
    eprintln!("ThinkOrder::get_type@004845c0: {n} rows agree");
}
