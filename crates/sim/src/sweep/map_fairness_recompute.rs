//! `MapFairness::recompute@0068a100` under the emulator against the port (item 2857; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// The f32 bits' `a < b` on integers: ordered compare, a NaN on either side false, the two zeros equal.
fn f32_bits_lt(a: u32, b: u32) -> bool {
    let nan = |x: u32| (x & 0x7FFF_FFFF) > 0x7F80_0000;
    if nan(a) || nan(b) {
        return false;
    }
    // sign-magnitude to a monotone key; -0 and +0 both map to 0x8000_0000
    let key = |x: u32| if x & 0x8000_0000 != 0 { 0x8000_0000 - (x & 0x7FFF_FFFF) } else { 0x8000_0000 + x };
    key(a) < key(b)
}

/// `MapFairness::recompute@0068a100`: over the first `num_players` team slots, the lowest and the highest
/// fairness (strict compares, the lowest bounded by 1e12f = 0x5368d4a5, the highest by +0.0), recorded as the player.
fn recompute(m: &mut [u8]) {
    const LIMIT: u32 = 0x5368_d4a5;
    let n = rd(m, 0x74);
    let (mut lowest, mut highest) = (LIMIT, 0u32);
    for i in 0..n.max(0) as usize {
        let t = rd(m, 0x54 + 4 * i) as usize;
        let f = rd(m, 0x4 + 4 * t) as u32;
        if f32_bits_lt(f, lowest) {
            wr(m, 0x44, t as i32);
            lowest = f;
        }
        if f32_bits_lt(highest, f) {
            wr(m, 0x48, t as i32);
            highest = f;
        }
    }
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("hex"))
        .collect()
}

/// `MapFairness::recompute@0068a100`, run by `tools/emu/sweep_map_fairness_recompute.py` under unicorn
/// (tools/emu): 120 random bytes of object, 0 argument(s), no windows; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_map_fairness_recompute.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            recompute(m);
            want_eax
        };
        assert_eq!(got, want_eax, "recompute — eax (void), row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "recompute — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "recompute — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "recompute — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("MapFairness::recompute@0068a100: {n} rows agree");
}
