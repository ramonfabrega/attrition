//! `Borders::check_border_blocked@00882370` under the emulator against the port (item 3008; tools/sweep_gen.py, obj with windows).

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

/// `Borders::check_border_blocked@00882370`, run by `tools/emu/sweep_borders_check_border_blocked.py` under unicorn
/// (tools/emu): 8 random bytes of object, 6 argument(s), `p` windows in order: p0..p1 pointed argument(s), p2 the place; the global cell 0x00c0621c points at 1024 random bytes; argument 4 names 4 random bytes; argument 5 names 4 random bytes; the word at `c0+0x2c` points at 1024 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_borders_check_border_blocked.py") else {
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
            let w = rd(&c[0], 0x224);
            let i1 = ((w.wrapping_mul(a[1]).wrapping_add(a[0])).wrapping_mul(8)) as usize;
            let i2 = ((w.wrapping_mul(a[3]).wrapping_add(a[2])).wrapping_mul(8)) as usize;
            let t = p[2].clone();
            let (n1, n2) = (t[i1 + 1], t[i2 + 1]);
            let dir = |dx: i32, dy: i32| -> u8 {
                1u8 << ((dx
                    .abs()
                    .wrapping_sub(dy.abs().wrapping_mul(2))
                    .wrapping_sub(dy)
                    .wrapping_sub(dx))
                    & 3)
            };
            let blocked_free = (n1 & 0xf) != 0
                && (n2 & 0xf) != 0
                && (n1 & dir(a[2] - a[0], a[3] - a[1])) == 0
                && (n2 & dir(a[0] - a[2], a[1] - a[3])) == 0;
            if !blocked_free {
                let both = t[i1 + 2] & t[i2 + 2];
                for k in [0usize, 1] {
                    let v = rd(&p[k], 0);
                    if v > -1 && (((1u32 << (v as u32 & 0x1f)) as u8) & both) != 0 {
                        wr(&mut p[k], 0, -1);
                    }
                }
            }
            0
        };
        assert_eq!(got, want_eax, "check_border_blocked — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "check_border_blocked — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "check_border_blocked — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "check_border_blocked — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Borders::check_border_blocked@00882370: {n} rows agree");
}
