//! `RepairOrder::get_type@004827a0` under the emulator against the port (item 1708).

use crate::orders::{Order, *};
use crate::world::Pos;

/// `RepairOrder::get_type@004827a0`, run by `tools/emu/sweep_repair_order_get_type.py` under unicorn
/// (tools/emu): the constant `REPAIR` through `eax`, whatever `this` points at, against `Order::index` of a fresh `Body::Repair` order — the port's own `OrderIndex` for the class.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_repair_order_get_type.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let order = Order {
            flags: 0,
            body: Body::Repair(0),
        };
        assert_eq!(
            i64::from(order.index()),
            want,
            "RepairOrder::get_type — row `{line}`"
        );
        let _ = a;
        n += 1;
    }
    assert!(n >= 4, "only {n} rows");
    eprintln!("RepairOrder::get_type@004827a0: {n} rows agree");
}
