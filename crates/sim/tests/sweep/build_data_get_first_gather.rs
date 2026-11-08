//! `BuildData::get_first_gather@0046f140` under the emulator against the port (item 2090; hand-written on tools/emu/hooks.py).

use crate::rally::GatherPoint;
use crate::world::Pos;

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("hex"))
        .collect()
}

/// `BuildData::get_first_gather@0046f140`, run by `tools/emu/sweep_build_data_get_first_gather.py` under unicorn
/// (tools/emu): the gather list's head `+0xcc` null or its count `+0xc8` not positive answers null, else the
/// head's data. The port is [`crate::Sim::first_gather`] on a building holding no point, or one: present exactly
/// when the original answers the data. The iterator reset the original writes on the way — the head at `+0xc4`,
/// its data at `+0xbc`, its byte `+0xc` at `+0xc0` — is modelled here and every byte held.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_data_get_first_gather.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want, tail) = super::tail_row(line);
        let (head, count) = (a[0] != 0, a[1]);
        let (node, data) = (a[2] as u32, a[3]);
        let mut m = unhex(tail[0]);
        let nodebytes = unhex(tail[2]);
        let mut sim = crate::Sim::new(crate::Tuning::RON, crate::World::new(8, 8), 2);
        let b = sim.add_building(0, Pos::new(0x1000, 0x1000), 0);
        if head && count > 0 {
            m[0xc4..0xc8].copy_from_slice(&node.to_le_bytes());
            m[0xbc..0xc0].copy_from_slice(&nodebytes[8..12]);
            m[0xc0] = nodebytes[0xc];
            sim.buildings[b].gather.push(GatherPoint {
                pos: Pos::new(0x1000, 0x1000),
                action: 0,
            });
        }
        let got = sim.first_gather(b).map_or(0, |_| data);
        assert_eq!(m, unhex(tail[1]), "get_first_gather — bytes, row `{line}`");
        assert_eq!(got, want, "get_first_gather — eax, row `{line}`");
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("BuildData::get_first_gather@0046f140: {n} rows agree");
}
