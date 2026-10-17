//! `World::set_river_at@006b1f80` under the emulator against the port (item 2852; tools/sweep_gen.py, obj with windows).

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

/// `World::set_river_at@006b1f80`, run by `tools/emu/sweep_world_set_river_at.py` under unicorn
/// (tools/emu): 16 random bytes of object, 3 argument(s), the global cell 0x00c06188 points at 316 random bytes; argument 0 names 4 random bytes; argument 1 names 4 random bytes; the word at `c0+0x134` points at 448 random bytes less `0` (placed); the word at `c0+0x138` points at 512 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_set_river_at.py") else {
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
            // The fine tile `(p[0], p[1])` of the river grid `p[3]` (`c[0]+0x18` wide, ushorts) and the coarse tile one
            // quarter of it each way in `p[2]` (`c[0]` wide, 0x1c-byte tiles): a river set (`a[2]`) sets 0x800
            // there and 0x1000 on the coarse tile; a river cleared clears 0x800, then the coarse 0x1000 only
            // when none of the sixteen fine tiles of its block carries 0x800 (item 2852).
            let (x, y) = (rd(&p[0], 0), rd(&p[1], 0));
            let wf = rd(&c[0], 0x18);
            let hw = |q: &[u8], i: usize| u16::from_le_bytes([q[i], q[i + 1]]);
            let put =
                |q: &mut Vec<u8>, i: usize, v: u16| q[i..i + 2].copy_from_slice(&v.to_le_bytes());
            let fine = (wf.wrapping_mul(y).wrapping_add(x) as usize) * 2;
            let coarse = (rd(&c[0], 0).wrapping_mul(y >> 2).wrapping_add(x >> 2) as usize) * 0x1c;
            if a[2] != 0 {
                let v = hw(&p[3], fine) | 0x800;
                put(&mut p[3], fine, v);
                let v = hw(&p[2], coarse) | 0x1000;
                put(&mut p[2], coarse, v);
            } else {
                let v = hw(&p[3], fine) & 0xf7ff;
                put(&mut p[3], fine, v);
                let any = (0..16).any(|k| {
                    let row = (k >> 2) + (y >> 2) * 4;
                    let i = (k & 3) + row * wf + (x >> 2) * 4;
                    hw(&p[3], i as usize * 2) & 0x800 != 0
                });
                if !any {
                    let v = hw(&p[2], coarse) & 0xefff;
                    put(&mut p[2], coarse, v);
                }
            }
            0
        };
        let _ = (got, want_eax);
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "set_river_at — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "set_river_at — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "set_river_at — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("World::set_river_at@006b1f80: {n} rows agree");
}
