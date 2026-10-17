//! `World::set_tree_at@006b2060` under the emulator against the port (item 2877; tools/sweep_gen.py, obj with windows).

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

/// `World::set_tree_at@006b2060`, run by `tools/emu/sweep_world_set_tree_at.py` under unicorn
/// (tools/emu): 316 random bytes of object, 3 argument(s), argument 0 names 4 random bytes; argument 1 names 4 random bytes; the word at `m+0x134` points at 448 random bytes less `0` (placed); the word at `m+0x138` points at 512 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_set_tree_at.py") else {
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
            {
                let x = rd(&p[0], 0);
                let y = rd(&p[1], 0);
                let flag = a[2];
                let fw = rd(m, 0x18);
                let cw = rd(m, 0);
                let fi = ((fw.wrapping_mul(y).wrapping_add(x)) as usize).wrapping_mul(2);
                let ci = (((((y >> 2).wrapping_mul(cw)).wrapping_add(x >> 2)) as usize)
                    .wrapping_mul(0x1c))
                .wrapping_add(0x13);
                let fine = u16::from_le_bytes([p[3][fi], p[3][fi + 1]]);
                let bits = fine & 0x30;
                let out = if flag != 0 {
                    if bits != 0x30 {
                        p[2][ci] = p[2][ci].wrapping_sub(1);
                    }
                    fine | 0x38
                } else {
                    let mut v = fine;
                    if bits == 0x30 {
                        v &= 0xffcf;
                        p[2][ci] = p[2][ci].wrapping_add(1);
                    }
                    v & 0xfff7
                };
                p[3][fi..fi + 2].copy_from_slice(&out.to_le_bytes());
                0
            }
        };
        let _ = (got, want_eax);
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "set_tree_at — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "set_tree_at — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "set_tree_at — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("World::set_tree_at@006b2060: {n} rows agree");
}
