//! `World::compute_all_territory@006b5700` under the emulator against the port (item 3200; tools/sweep_gen.py, obj with windows).

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

/// `World::compute_all_territory@006b5700`, run by `tools/emu/sweep_world_compute_all_territory.py` under unicorn
/// (tools/emu): 64 random bytes of object, 0 argument(s), `p` windows in order: p0..p23 sparse table 0 (8 rows × 3 slice(s), row-major), p24..p28 the place(s), in flag order; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00c061bc points at 64 random bytes; the global cell 0x00c061b8 points at 64 random bytes; 4 random bytes at 0x00e8a794; callee 0x006b0bb0 (2 args) stubbed; the word at `c2+0x10` points at 17408 random bytes less `0` (placed); the word at `c0+0x134` points at 512 random bytes less `0` (placed); the word at `p24+0x227c` points at 64 random bytes less `0` (placed); the word at `p24+0x2304` points at 64 random bytes less `0` (placed); the word at `p24+17396` points at 64 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 writes 3 (2 bytes) at `p2+0x0` each call; stub 0 writes 65520 (2 bytes) at `p2+0x2` each call; stub 0 writes 7 (2 bytes) at `p2+0x4` each call; stub 0 writes 11 (2 bytes) at `p2+0x6` each call; stub 0 writes 13 (2 bytes) at `p2+0x8` each call; stub 0 writes 17 (2 bytes) at `p2+0xa` each call; stub 0 writes 19 (2 bytes) at `p2+0xc` each call; stub 0 writes 23 (2 bytes) at `p2+0xe` each call; stub 0 writes 29 (2 bytes) at `p2+0x3e` each call; stub 0 writes 31 (2 bytes) at `p2+0x7e` each call; stub 0 writes 100 (2 bytes) at `p5+0x0` each call; stub 0 writes 200 (2 bytes) at `p5+0x2` each call; stub 0 writes 300 (2 bytes) at `p5+0x4` each call; stub 0 writes 400 (2 bytes) at `p5+0x6` each call; stub 0 writes 500 (2 bytes) at `p5+0x8` each call; stub 0 writes 600 (2 bytes) at `p5+0xa` each call; stub 0 writes 700 (2 bytes) at `p5+0xc` each call; stub 0 writes 800 (2 bytes) at `p5+0xe` each call; stub 0 writes 900 (2 bytes) at `p5+0x3e` each call; stub 0 writes 1000 (2 bytes) at `p5+0x7e` each call; stub 0 writes 65535 (2 bytes) at `p8+0x0` each call; stub 0 writes 65535 (2 bytes) at `p8+0x2` each call; stub 0 writes 65535 (2 bytes) at `p8+0x4` each call; stub 0 writes 65535 (2 bytes) at `p8+0x6` each call; stub 0 writes 65535 (2 bytes) at `p8+0x8` each call; stub 0 writes 65535 (2 bytes) at `p8+0xa` each call; stub 0 writes 65535 (2 bytes) at `p8+0xc` each call; stub 0 writes 65535 (2 bytes) at `p8+0xe` each call; stub 0 writes 65535 (2 bytes) at `p8+0x3e` each call; stub 0 writes 65535 (2 bytes) at `p8+0x7e` each call; 8 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0x9d8:0x8, +0x145e:0x80 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 300 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_compute_all_territory.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().collect();
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
        let mut p: Vec<Vec<u8>> = wins.split_off(4);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // m is unused; c0 the world (`+0` width, `+0x134` the tile table `p[25]`), c1 the game daemon
            // (`+0x24` borders, -1 while it runs, then 0), c2 the regions (`+0x10` the array `p[24]`), c3 the
            // fog flags; p[3k], p[3k+1], p[3k+2] leader k's flag word, `[sum, max]` and 128 bytes of territory
            // shorts; p[26], p[27] regions 64 and 65's coordinate lists. Recorded: `compute_reg_territory`.
            let rdw = |w: &[u8], o: usize| {
                i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
            };
            let wrw =
                |w: &mut [u8], o: usize, v: i32| w[o..o + 4].copy_from_slice(&v.to_le_bytes());
            let mut calls: Vec<String> = Vec::new();
            for k in 0..8 {
                if p[3 * k][0] & 1 != 0 {
                    p[3 * k + 2].fill(0);
                }
            }
            wrw(&mut c[1], 0x24, -1);
            for i in 0..64 {
                if rdw(&p[24], 0x88 * i + 0x14) != 0 {
                    wrw(&mut p[24], 0x88 * i + 0x2c, 0);
                    calls.push(format!("c0+0x0,{i},1"));
                    // The stub's writes (`--stub-write`): leaders 0..2's territory shorts, every call.
                    const OFFS: [usize; 10] = [0, 2, 4, 6, 8, 10, 12, 14, 62, 126];
                    const VALS: [[u16; 10]; 3] = [
                        [3, 65520, 7, 11, 13, 17, 19, 23, 29, 31],
                        [100, 200, 300, 400, 500, 600, 700, 800, 900, 1000],
                        [65535; 10],
                    ];
                    for (li, vals) in VALS.iter().enumerate() {
                        for (o, v) in OFFS.iter().zip(vals) {
                            p[3 * li + 2][*o..*o + 2].copy_from_slice(&v.to_le_bytes());
                        }
                    }
                }
            }
            c[3][0] |= 2;
            for r in 64..128 {
                // Regions 66..126 are held at no coordinates; 64, 65 and 127 have lists.
                let n = rdw(&p[24], 0x88 * r + 0x14);
                for j in 0..n.max(0) as usize {
                    let l = match r {
                        64 => &p[26],
                        65 => &p[27],
                        _ => &p[28],
                    };
                    let (x, y) = (rdw(l, 8 * j), rdw(l, 8 * j + 4));
                    let at = (rdw(&c[0], 0).wrapping_mul(y).wrapping_add(x) as usize) * 0x1c;
                    p[25][at + 0xf] = 0xff;
                    p[25][at + 0x10] = 0xff;
                }
            }
            for k in 0..8 {
                if p[3 * k][0] & 1 != 0 {
                    let mut sum = 0i32;
                    for i in 0..64 {
                        if rdw(&p[24], 0x88 * i + 0x14) != 0 {
                            sum += i32::from(u16::from_le_bytes([
                                p[3 * k + 2][2 * i],
                                p[3 * k + 2][2 * i + 1],
                            ]));
                        }
                    }
                    let old = rdw(&p[3 * k + 1], 4);
                    wrw(&mut p[3 * k + 1], 0, sum);
                    wrw(&mut p[3 * k + 1], 4, old.max(sum));
                }
            }
            wrw(&mut c[1], 0x24, 0);
            log.push(if calls.is_empty() {
                "-".to_string()
            } else {
                calls.join("/")
            });
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "compute_all_territory — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "compute_all_territory — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "compute_all_territory — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[5 + i],
                "compute_all_territory — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 300, "only {n} rows");
    eprintln!("World::compute_all_territory@006b5700: {n} rows agree");
}
