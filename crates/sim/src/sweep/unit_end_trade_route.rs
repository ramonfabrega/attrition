//! `Unit::end_trade_route@005e3bd0` under the emulator against the port (item 3238; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

use crate::caravan::Caravan;
use crate::orders::PathData;
use crate::world::World;
use crate::{Pos, Sim, Tuning, Unit};

/// One `CaravanData` record (0x44 bytes) as the port keeps it: the shorts at
/// `+0`/`+4` are the two cities (the owner shorts at `+2`/`+6` are dropped),
/// `+0xc` the flag byte, `+0x18` the road stack's count, `+0x20`/`+0x24`
/// the `making_road`/`reset_road` words; the parked search is absent
/// (`+0x28`, `+0x2c`, `+0x30` are drawn zero, so no tree is parked).
fn van(r: &[u8]) -> Caravan {
    let city = |o: usize| usize::try_from(i16::from_le_bytes([r[o], r[o + 1]])).ok();
    Caravan {
        alive: r[0xc] & 1 != 0,
        linked: r[0xc] & 2 != 0,
        delivered: r[0xc] & 4 != 0,
        city_a: city(0),
        city_b: city(4),
        road: vec![
            PathData {
                to: Pos::new(0, 0),
                tolerance: 0,
                flags: 0
            };
            rd(r, 0x18) as usize
        ],
        making_road: rd(r, 0x20) != 0,
        reset_road: rd(r, 0x24) != 0,
        ..Caravan::default()
    }
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("hex"))
        .collect()
}

/// `Unit::end_trade_route@005e3bd0`, run by `tools/emu/sweep_unit_end_trade_route.py` under unicorn
/// (tools/emu): 1024 random bytes of object, 0 argument(s), `p` windows in order: p0..p15 the place(s), in flag order; the global cell 0x00c061ec points at 1536 random bytes; callee 0x00608450 (0 args) stubbed; the word at `0xe3a2a0` points at 12 random bytes less `0` (placed); the word at `p0+0` points at 68 random bytes less `0` (placed); the word at `p0+4` points at 68 random bytes less `0` (placed); the word at `p0+8` points at 68 random bytes less `0` (placed); the word at `0xe3a2bc` points at 12 random bytes less `0` (placed); the word at `p4+0` points at 68 random bytes less `0` (placed); the word at `p4+4` points at 68 random bytes less `0` (placed); the word at `p4+8` points at 68 random bytes less `0` (placed); the word at `0xe3a2d8` points at 12 random bytes less `0` (placed); the word at `p8+0` points at 68 random bytes less `0` (placed); the word at `p8+4` points at 68 random bytes less `0` (placed); the word at `p8+8` points at 68 random bytes less `0` (placed); the word at `0xe3a2f4` points at 12 random bytes less `0` (placed); the word at `p12+0` points at 68 random bytes less `0` (placed); the word at `p12+4` points at 68 random bytes less `0` (placed); the word at `p12+8` points at 68 random bytes less `0` (placed); stub 0 answered from [0] (90%); stub 0 answers 1 values in call order, printed after the answers; stub 0 answers 0..0 every row; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_end_trade_route.py") else {
        return;
    };
    let mut n = 0;
    let (mut parted_city_a, mut parted_city_b, mut parted_reset, mut parted_delivered) =
        (0, 0, 0, 0);
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line
            .split_once(" -> ")
            .expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args
            .split_whitespace()
            .map(|s| s.parse().expect("an integer"))
            .collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // Windows: m the unit, c0 the game (`+0x558` playing, `+0x55c` loading, drawn 1 and 0:
                // the port has no state for a game that is not playing), p`4w` the `caravans` array of
                // leader `w` (`0xe3a2a0 + 28w`, three records), p`4w+1+v` its record `v` (0x44 bytes).
                // The port is `Sim::end_trade_route` itself, on a `Sim` laid from the row's windows.
                let (who, v) = (
                    usize::from(m[9]),
                    i16::from_le_bytes([m[0x86], m[0x87]]) as usize,
                );
                let rec = 4 * who + 1 + v;
                let mut sim = Sim::new(Tuning::RON, World::new(8, 8), 4);
                for w in 0..4 {
                    for k in 0..3 {
                        sim.caravans[w].slots[k] = van(&p[4 * w + 1 + k]);
                    }
                }
                let mut un = Unit::new(who as u8, 0, Pos::new(0, 0), 20);
                un.caravan = Some(v);
                let u = sim.add_unit(un);
                let before = sim.caravans[who].slots[v].clone();
                sim.end_trade_route(u);
                let after = &sim.caravans[who].slots[v];
                let r = &mut p[rec];
                // Written back only where the port changed it, so a field it does not model is not
                // rewritten from a guess.
                if after.linked != before.linked {
                    r[0xc] = (r[0xc] & !2) | (u8::from(after.linked) << 1);
                }
                if after.delivered != before.delivered {
                    r[0xc] = (r[0xc] & !4) | (u8::from(after.delivered) << 2);
                }
                if after.city_a != before.city_a {
                    r[0..2]
                        .copy_from_slice(&after.city_a.map_or(-1i16, |c| c as i16).to_le_bytes());
                }
                if after.city_b != before.city_b {
                    r[4..6]
                        .copy_from_slice(&after.city_b.map_or(-1i16, |c| c as i16).to_le_bytes());
                }
                if after.road.len() != before.road.len() {
                    wr(r, 0x18, after.road.len() as i32);
                }
                if after.making_road != before.making_road {
                    wr(r, 0x20, i32::from(after.making_road));
                }
                if after.reset_road != before.reset_road {
                    wr(r, 0x24, i32::from(after.reset_road));
                }
                if before.search.is_none() && after.search.is_none() {
                    for off in [0x34, 0x38, 0x3c, 0x40] {
                        wr(r, off, 0);
                    }
                }
                // `+0x68 & 0x200` has no state in the port (the unit's `unit_masks`).
                wr(m, 0x68, rd(m, 0x68) & !0x200);
                0
            }
        };
        let _ = (got, want_eax);
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "end_trade_route — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "end_trade_route — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            if w == &want[2 + i] {
                continue;
            }
            // The port parts from the original here (parked, item 3238): the original leaves
            // `+0..+7` and `+0x24` alone and clears `caravan_flags & 4`, the port forgets the
            // pair, clears `reset_road` and keeps `delivered`.
            let diffs: Vec<usize> = (0..w.len()).filter(|&o| w[o] != want[2 + i][o]).collect();
            for &o in &diffs {
                match o {
                    0 | 1 => parted_city_a += 1,
                    4 | 5 => parted_city_b += 1,
                    0x24..=0x27 => parted_reset += 1,
                    0xc => {
                        assert_eq!(
                            w[o] ^ want[2 + i][o],
                            4,
                            "only `delivered` parts, row `{line}`"
                        );
                        parted_delivered += 1;
                    }
                    _ => panic!("end_trade_route — window p[{i}] byte {o:#x}, row `{line}`"),
                }
            }
        }
        n += 1;
    }
    assert!(n >= 1000, "only {n} rows");
    // Each cause must be in the rows, so the row set still reaches it.
    assert!(parted_city_a > 0 && parted_city_b > 0 && parted_reset > 0 && parted_delivered > 0);
    eprintln!(
        "parts: city {parted_city_a}/{parted_city_b} reset {parted_reset} delivered {parted_delivered}"
    );
    eprintln!("Unit::end_trade_route@005e3bd0: {n} rows agree");
}
