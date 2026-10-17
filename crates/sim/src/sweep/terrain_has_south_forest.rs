//! `Terrain::has_south_forest@00850060` under the emulator against the port (item 2877; tools/sweep_gen.py, obj with windows).

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

/// `Terrain::has_south_forest@00850060`, run by `tools/emu/sweep_terrain_has_south_forest.py` under unicorn
/// (tools/emu): 16 random bytes of object, 2 argument(s), the global cell 0x00c06188 points at 316 random bytes; argument 0 names 4 random bytes; argument 1 names 4 random bytes; the word at `c0+0x134` points at 1024 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_terrain_has_south_forest.py") else {
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
                let (x, y) = (rd(&p[0], 0), rd(&p[1], 0));
                let (w, h) = (rd(&c[0], 0), rd(&c[0], 4));
                let flag = |cx: i32, cy: i32| {
                    cx >= 0
                        && cy >= 0
                        && cx < w
                        && cy < h
                        && p[2][((w.wrapping_mul(cy).wrapping_add(cx)) as usize).wrapping_mul(0x1c)]
                            & 0x20
                            != 0
                };
                i64::from(flag(x + 1, y + 1) || flag(x, y + 1) || flag(x + 1, y))
            }
        };
        assert_eq!(got, want_eax, "has_south_forest — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "has_south_forest — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "has_south_forest — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "has_south_forest — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Terrain::has_south_forest@00850060: {n} rows agree");
}
