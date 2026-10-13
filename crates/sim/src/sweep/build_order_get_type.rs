//! `BuildOrder::get_type@00482b50` under the emulator against the port (item 1708).

use crate::orders::{Order, *};
use crate::world::Pos;

/// `BuildOrder::get_type@00482b50`, run by `tools/emu/sweep_build_order_get_type.py` under unicorn
/// (tools/emu): the constant `BUILD_AT` through `eax`, whatever `this` points at, against `Order::index` of a fresh `Body::Build` order — the port's own `OrderIndex` for the class.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_order_get_type.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let order = Order {
            flags: 0,
            body: Body::Build(0),
        };
        assert_eq!(
            i64::from(order.index()),
            want,
            "BuildOrder::get_type — row `{line}`"
        );
        let _ = a;
        n += 1;
    }
    assert!(n >= 4, "only {n} rows");
    eprintln!("BuildOrder::get_type@00482b50: {n} rows agree");
}
