//! `ResourceDivvyPool::get_late@0068a570` under the emulator against the port (item 2852; tools/sweep_gen.py, obj with windows).

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

/// `ResourceDivvyPool::get_late@0068a570`, run by `tools/emu/sweep_resource_divvy_pool_get_late.py` under unicorn
/// (tools/emu): 120 random bytes of object, 0 argument(s), callee 0x00a39d70 (2 args) stubbed; the word at `m+0x30` points at 64 random bytes less `0` (placed); the word at `m+0x44` points at 256 random bytes less `0` (placed); stub 0 answers 12 values in call order, printed after the answers; stub 0 answers 0..65535 every row; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_resource_divvy_pool_get_late.py") else {
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
            // The pool's drawn pick: while no good is taken, `Random::get(0, 0xffff) % count` (index 0 without a call when
    // `count <= 1`, the stub's answers `a[1..]` in call order) names a bit of the mask `p[0]`; a clear bit is set and
    // the good `p[1][idx]` taken (a loop runs on while that is -1); then the mask is cleared through its `size`
    // (`m+0x2c`) bytes when every bit below `count` (`m+0x38`) is set (item 2852).
            let count = rd(m, 0x38);
            let size = rd(m, 0x2c);
            let mut next = 1;
            let mut got = -1;
            while got == -1 {
                let idx = if count - 1 > 0 {
                    next += 1;
                    a[next - 1] % count
                } else {
                    0
                };
                let (byte, bit) = ((idx >> 3) as usize, 1u8 << (idx & 7));
                if p[0][byte] & bit == 0 {
                    p[0][byte] |= bit;
                    got = rd(&p[1], 4 * idx as usize);
                }
            }
            let all_taken = (0..count).all(|i| p[0][(i >> 3) as usize] & (1 << (i & 7)) != 0);
            if all_taken {
                for b in p[0].iter_mut().take(size.max(0) as usize) {
                    *b = 0;
                }
            }
            i64::from(got)
        };
        assert_eq!(got, want_eax, "get_late — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_late — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_late — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_late — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("ResourceDivvyPool::get_late@0068a570: {n} rows agree");
}
