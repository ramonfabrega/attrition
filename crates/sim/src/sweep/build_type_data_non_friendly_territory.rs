//! `BuildTypeData::non_friendly_territory@006389c0` under the emulator against the port (item 2247; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("hex"))
        .collect()
}

/// `BuildTypeData::non_friendly_territory@006389c0`, run by `tools/emu/sweep_build_type_data_non_friendly_territory.py` under unicorn
/// (tools/emu): 576 random bytes of object, 3 argument(s), the global cell 0x00c061d0 points at 320 random bytes; the word at `c0+0x134` points at 256 random bytes less `0` (placed); the word at `c0+0x138` points at 288 random bytes less `0` (placed); 8 rows 0x6eec apart at `0xc061e0`, slices +0x8:0x8c (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_type_data_non_friendly_territory.py") else {
        return;
    };
    let mut n = 0;
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
                use crate::world::tile;
                // The footprint `+0x234` by `+0x238` from `(a[0], a[1])`, the
                // world's `c[0]` (cells wide `+0`, tiles `+0x18` by `+0x1c`), its
                // cell records `p[8]` and tile masks `p[9]`, the leaders `p[0..8]`.
                let (xs, ys) = (rd(m, 0x234), rd(m, 0x238));
                let who = a[2] as usize;
                let (cw, w, h) = (rd(&c[0], 0), rd(&c[0], 0x18), rd(&c[0], 0x1c));
                let me = rd(&p[who], 0);
                let mut r = 0;
                'walk: for x in a[0]..a[0] + xs {
                    for y in a[1]..a[1] + ys {
                        if x < 0 || y < 0 || x >= w || y >= h {
                            continue;
                        }
                        let t = ((w * y + x) * 2) as usize;
                        let mask = u16::from_le_bytes([p[9][t], p[9][t + 1]]);
                        if mask & tile::SURFACE == tile::SURFACE_OCEAN {
                            continue;
                        }
                        let o = i32::from(
                            p[8][(((y >> 2) * cw + (x >> 2)) * 0x1c + 0xf) as usize] as i8,
                        );
                        let (to, from) = if o < 0 {
                            (-1, -1)
                        } else {
                            (
                                rd(&p[who], (0x6c + 4 * o) as usize),
                                rd(&p[o as usize], (0x6c + 4 * me) as usize),
                            )
                        };
                        let rank = crate::place::ground_rank(
                            o,
                            me,
                            (to == 2, from == 2),
                            (to == 0, from == 0),
                        );
                        if rank == 3 {
                            r = 3;
                            break 'walk;
                        }
                        r = r.max(rank);
                    }
                }
                i64::from(r)
            }
        };
        assert_eq!(got, want_eax, "non_friendly_territory — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "non_friendly_territory — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "non_friendly_territory — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "non_friendly_territory — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("BuildTypeData::non_friendly_territory@006389c0: {n} rows agree");
}
