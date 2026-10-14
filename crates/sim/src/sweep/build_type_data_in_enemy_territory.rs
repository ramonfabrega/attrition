//! `BuildTypeData::in_enemy_territory@00638b40` under the emulator against the port (item 2247; tools/sweep_gen.py, obj with windows).

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

/// `BuildTypeData::in_enemy_territory@00638b40`, run by `tools/emu/sweep_build_type_data_in_enemy_territory.py` under unicorn
/// (tools/emu): 576 random bytes of object, 3 argument(s), the global cell 0x00c061d0 points at 320 random bytes; callee 0x006b42c0 (3 args) stubbed; the word at `c0+0x134` points at 256 random bytes less `0` (placed); stub 0 answered from [0, 0, 0, 1] (90%); 8 rows 0x6eec apart at `0xc061e0`, slices +0x8:0x8c, +0x3f8:0x4, +0x125e:0x100 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_type_data_in_enemy_territory.py") else {
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
                // The footprint `+0x234` by `+0x238` from `(a[0], a[1])`; the
                // world `c[0]`, its cell records `p[24]`; leader `k`'s rows are
                // `p[3k]` (from `+8`), `p[3k + 1]` (`+0x3f8`, its cities) and
                // `p[3k + 2]` (`+0x125e`, the region cities, then forts); the
                // visibility callee's answer is `a[3]`.
                let (xs, ys) = (rd(m, 0x234), rd(m, 0x238));
                let who = a[2];
                let w = who as usize;
                let (cw, ww, wh) = (rd(&c[0], 0), rd(&c[0], 0x18), rd(&c[0], 0x1c));
                let me = rd(&p[3 * w], 0);
                let short =
                    |v: &[u8], off: usize| i32::from(i16::from_le_bytes([v[off], v[off + 1]]));
                let mut r = 0;
                'walk: for x in a[0]..a[0] + xs {
                    for y in a[1]..a[1] + ys {
                        if x < 0 || y < 0 || x >= ww || y >= wh {
                            continue;
                        }
                        let rec = (((y >> 2) * cw + (x >> 2)) * 0x1c) as usize;
                        let o = i32::from(p[24][rec + 0xf] as i8);
                        if o < 0 || o == who {
                            continue;
                        }
                        let to = rd(&p[3 * w], (0x6c + 4 * o) as usize) == 2;
                        let from = rd(&p[3 * o as usize], (0x6c + 4 * me) as usize) == 2;
                        if crate::place::friendly_ground(o, me, to, from) {
                            continue;
                        }
                        let near = crate::place::claim_reaches(
                            rd(&p[3 * w + 1], 0),
                            short(&p[24], rec + 4),
                            |g| {
                                (
                                    short(&p[3 * w + 2], 2 * g),
                                    short(&p[3 * w + 2], 0x80 + 2 * g),
                                )
                            },
                        );
                        if near || a[3] != 0 {
                            r = 1;
                            break 'walk;
                        }
                    }
                }
                i64::from(r)
            }
        };
        assert_eq!(got, want_eax, "in_enemy_territory — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "in_enemy_territory — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "in_enemy_territory — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "in_enemy_territory — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("BuildTypeData::in_enemy_territory@00638b40: {n} rows agree");
}
