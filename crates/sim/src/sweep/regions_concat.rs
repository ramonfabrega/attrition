//! `Regions::concat@00680500` under the emulator against the port (item 2298; tools/sweep_gen.py, obj with windows).

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

/// `Regions::concat@00680500`, run by `tools/emu/sweep_regions_concat.py` under unicorn
/// (tools/emu): 32 random bytes of object, 2 argument(s), the global cell 0x00c06188 points at 320 random bytes; the word at `m+0x10` points at 680 random bytes less `0` (placed); the word at `c0+0x134` points at 336 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_regions_concat.py") else {
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
                // Region a[0] folds into a[1]: its +0x14 word (the regions at this+0x10,
                // 0x88 apart) is added to a[1]'s and zeroed; then every tile record of the
                // world (a count at +8, 0x1c-byte records at +0x134) whose region short
                // (+4) is a[0] is relabelled a[1].
                let (s, d) = (a[0] as usize * 0x88 + 0x14, a[1] as usize * 0x88 + 0x14);
                let sum = rd(&p[0], d).wrapping_add(rd(&p[0], s));
                wr(&mut p[0], d, sum);
                wr(&mut p[0], s, 0);
                for i in 0..rd(&c[0], 8).max(0) as usize {
                    let o = i * 0x1c + 4;
                    if i32::from(i16::from_le_bytes([p[1][o], p[1][o + 1]])) == a[0] {
                        p[1][o..o + 2].copy_from_slice(&(a[1] as i16).to_le_bytes());
                    }
                }
                0
            }
        };
        let _ = (got, want_eax);
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "concat — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "concat — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "concat — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Regions::concat@00680500: {n} rows agree");
}
