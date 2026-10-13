//! `FollowOrder::get_type@00486c20` under the emulator against the port (item 1708).

use crate::orders::{Order, *};

/// `FollowOrder::get_type@00486c20`, run by `tools/emu/sweep_follow_order_get_type.py` under unicorn
/// (tools/emu): the constant `FOLLOW` through `eax`, whatever `this` points at, against `Order::index` of a fresh `Body::Follow` order — the port's own `OrderIndex` for the class.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_follow_order_get_type.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let order = Order {
            flags: 0,
            body: Body::Follow(FollowOrder { target: 0 }),
        };
        assert_eq!(
            i64::from(order.index()),
            want,
            "FollowOrder::get_type — row `{line}`"
        );
        let _ = a;
        n += 1;
    }
    assert!(n >= 4, "only {n} rows");
    eprintln!("FollowOrder::get_type@00486c20: {n} rows agree");
}
