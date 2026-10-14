//! `BuildData::num_gatherers@00630450` under the emulator against the port (item 2276; tools/sweep_gen.py, obj with windows).

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

/// `BuildData::num_gatherers@00630450`, run by `tools/emu/sweep_build_data_num_gatherers.py` under unicorn
/// (tools/emu): 116 random bytes of object, 2 argument(s), vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; callee 0x0064dda0 (2 args) stubbed; callee 0x00608880 (3 args) stubbed; the word at `m+0x18` points at 4 random bytes less `0` (placed); the word at `0xc0aec0+2*0x1c` points at 24 random bytes less `0` (placed); the word at `p1+0` points at 148 random bytes less `0` (placed); the word at `p1+4` points at 148 random bytes less `0` (placed); the word at `p1+8` points at 148 random bytes less `0` (placed); the word at `p1+12` points at 148 random bytes less `0` (placed); the word at `p1+16` points at 148 random bytes less `0` (placed); the word at `p1+20` points at 148 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1, 2, 3] (90%); stub 3 answered from [0, 1, 1] (90%); stub 0 answers 2 values in call order, printed after the answers; stub 1 answers 2 values in call order, printed after the answers; stub 2 answers 2 values in call order, printed after the answers; stub 3 answers 8 values in call order, printed after the answers; figures p2..p7 link at +0x92 (2 bytes) above their own index or to -1; the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_data_num_gatherers.py") else {
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
                // The building's chain head `+0x70`, its player `+9` (held at
                // 2), the units table `p[1]` of six units `p[2..8]`, each's
                // next `+0x92` and decoy bit `+0x68 & 1`. `a[20]` says whether
                // `this`'s slot `+0xb8` was `ObjectData::is` (the type's
                // `+0x60` asked, `a[8..10]`) or its own stub (`a[6..8]`);
                // `count_inside` answers `a[10..12]`, `is_gathering_at`
                // `a[12..20]`, each in call order.
                let skip = a[1] != 0;
                let (mut is_n, mut ci, mut ga) = (0usize, 0usize, 0usize);
                let mut is_ = || {
                    is_n += 1;
                    a[if a[20] != 0 { 8 } else { 6 } + (is_n - 1).min(1)] != 0
                };
                let mut n = 0;
                for _ in 0..2 {
                    if is_() {
                        n = i32::wrapping_add(n, a[10 + ci.min(1)]);
                        ci += 1;
                    }
                }
                let mut chain = Vec::new();
                let mut at = i16::from_le_bytes([m[0x70], m[0x71]]);
                while at >= 0 {
                    chain.push(at as usize);
                    at = i16::from_le_bytes([p[2 + at as usize][0x92], p[2 + at as usize][0x93]]);
                }
                n = n.wrapping_add(crate::orders::chain_gatherers(
                    chain,
                    skip,
                    |u| p[2 + u][0x68] & 1 != 0,
                    |_| {
                        ga += 1;
                        a[12 + (ga - 1).min(7)] != 0
                    },
                ));
                i64::from(n)
            }
        };
        assert_eq!(got, want_eax, "num_gatherers — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "num_gatherers — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "num_gatherers — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "num_gatherers — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("BuildData::num_gatherers@00630450: {n} rows agree");
}
