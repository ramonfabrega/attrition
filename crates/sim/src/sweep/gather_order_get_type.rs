//! `GatherOrder::get_type@00487310` under the emulator against the port (item 1721).

use crate::orders::{Order, *};

/// `GatherOrder::get_type@00487310`, run by `tools/emu/sweep_gather_order_get_type.py` under unicorn
/// (tools/emu): the constant `GATHER` through `eax`, whatever `this` points at, against `Order::index` of a fresh `Body::Gather` order — the port's own `OrderIndex` for the class.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_gather_order_get_type.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let order = Order {
            flags: 0,
            body: Body::Gather(GatherOrder {
                building: 0,
                tile: None,
                wait: 0,
                goto_build: false,
                dist_mod: 0,
                been_there: false,
            }),
        };
        assert_eq!(
            i64::from(order.index()),
            want,
            "GatherOrder::get_type — row `{line}`"
        );
        let _ = a;
        n += 1;
    }
    assert!(n >= 4, "only {n} rows");
    eprintln!("GatherOrder::get_type@00487310: {n} rows agree");
}
