//! `Doober::has_doobers@008466d0` under the emulator against the port (item 2783; tools/sweep_gen.py, obj with windows).

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

/// `Doober::has_doobers@008466d0`, run by `tools/emu/sweep_doober_has_doobers.py` under unicorn
/// (tools/emu): 16 random bytes of object, 2 argument(s), 4 random bytes at 0x00eafdfc; the word at `0xeafe08` points at 64 random bytes less `0` (placed); the word at `0xcae5fc` points at 16 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_doober_has_doobers.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `Doober::has_doobers(p1, p2)`: over the `count` (`c[0]`) float pairs of the locations table (`p[0]`)
            // it truncates the x float, takes `>> 8` as an index into `div_3_table` (`p[1]`) and, on a row
            // equal to `p1`, answers 1 if `div_3_table[0]` is nonzero: the original's second test indexes the
            // table by `(int)(float)(y == (float)p2) >> 8`, which is 0 whatever the comparison said, so `p2`
            // and `y` are never decisive.
            let count = rd(&c[0], 0);
            let table = |i: i32| rd(&p[1], 4 * i as usize);
            let mut ans = 0i64;
            for i in 0..count.max(0) as usize {
                let x = f32::from_bits(rd(&p[0], 8 * i) as u32) as i32;
                if table(x >> 8) == a[0] && table(0) != 0 {
                    ans = 1;
                    break;
                }
            }
            ans
        };
        assert_eq!(got, want_eax, "has_doobers — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "has_doobers — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "has_doobers — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "has_doobers — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Doober::has_doobers@008466d0: {n} rows agree");
}
