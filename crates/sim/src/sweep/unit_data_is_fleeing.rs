//! `UnitData::is_fleeing@0046efa0` under the emulator against the port (item 2090; hand-written on tools/emu/hooks.py).

use crate::orders::{Body, MoveKind, Order, QueuePos};

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("hex"))
        .collect()
}

/// `UnitData::is_fleeing@0046efa0`, run by `tools/emu/sweep_unit_data_is_fleeing.py` under unicorn (tools/emu): the order
/// list's head `+0xdc` null, or a sentinel whose first node's data (`+8`) is null or an order whose `get_type` slot (`+0x10`) is answered per row.
/// The port is [`crate::Sim::is_fleeing`] on a unit holding no order, or one order of the row's type (`FLEE_TO` for a
/// `get_type` of 4, `THINK` otherwise); the iterator reset the original writes on the way — the node at `+0xd4`,
/// its data at `+0xcc`, its byte `+0xc` at `+0xd0` — is modelled here and every byte held.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_is_fleeing.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want, tail) = super::tail_row(line);
        let (head, data, ty) = (a[0] != 0, a[1] != 0, a[2]);
        let (node, data_at) = (a[3] as u32, a[4] as u32);
        let mut m = unhex(tail[0]);
        let nodebytes = unhex(tail[2]);
        let (mut sim, u) = super::sim_with_unit(crate::UnitType::default());
        let mut calls = "";
        if head {
            m[0xd4..0xd8].copy_from_slice(&node.to_le_bytes());
            m[0xcc..0xd0].copy_from_slice(&(if data { data_at } else { 0 }).to_le_bytes());
            m[0xd0] = nodebytes[0xc];
            if data {
                if ty == 4 {
                    let to = sim.units[u].pos;
                    sim.add_move_order(u, to, MoveKind::FleeTo, QueuePos::First, false);
                } else {
                    sim.units[u].orders.push_back(Order {
                        flags: 0,
                        body: Body::Think,
                    });
                }
                calls = "data";
            }
        }
        let got = sim.is_fleeing(u);
        assert_eq!(m, unhex(tail[1]), "is_fleeing — bytes, row `{line}`");
        assert_eq!(tail[3], calls, "is_fleeing — the calls, row `{line}`");
        assert_eq!(i64::from(got), want, "is_fleeing — eax, row `{line}`");
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("UnitData::is_fleeing@0046efa0: {n} rows agree");
}
