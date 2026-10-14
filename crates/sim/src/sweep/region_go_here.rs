//! `Region::go_here@006810f0` under the emulator against the port (item 2287; tools/sweep_gen.py, obj with windows).

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

/// `Region::go_here@006810f0`, run by `tools/emu/sweep_region_go_here.py` under unicorn
/// (tools/emu): 32 random bytes of object, 1 argument(s), 8 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0x8:0x4, +0x74:0x20, +0xa68:0x10, +0xfde:0x10, +0x125e:0x10 (`p` windows after the pointed buffers, before the places); 8 rows 0x6eec apart at `0xc061e0`, slices +0x74:0x20 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_region_go_here.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // The region `+8` flags, `+0x10` its index, `+0x14` its size;
            // leader i's slices `p[6i..6i+6]` — byte 0, `+8` (its team),
            // `+0x74` the stances, `+0xa68` the strategy cells, `+0xfde`
            // the peasants, `+0x125e` the cities — and `leadersc`'s stances
            // `p[48 + i]`. `a[0]` is `who`.
            let who = a[0] as usize;
            let r = rd(m, 0x10) as usize;
            let short =
                |w: &[u8], k: usize| i32::from(u16::from_le_bytes([w[2 * k], w[2 * k + 1]]));
            let team = rd(&p[6 * who + 1], 0);
            i64::from(crate::army::go_here_bits(
                m[8] & 8 != 0,
                rd(m, 0x14),
                who,
                short(&p[6 * who + 5], r),
                short(&p[6 * who + 4], r),
                p[6 * who + 3][2 * r] & 4 != 0,
                8,
                |i| p[6 * i][0] & 2 != 0,
                |i| short(&p[6 * i + 5], r),
                |i| {
                    i as i32 != team
                        && (rd(&p[6 * who + 2], 4 * i) == 0
                            || rd(&p[48 + i], 4 * team as usize) == 0)
                },
            ))
        };
        assert_eq!(got, want_eax, "go_here — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "go_here — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "go_here — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "go_here — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Region::go_here@006810f0: {n} rows agree");
}
