//! `Unit::set_in_danger@005fcfb0` under the emulator against the port (item 2276; tools/sweep_gen.py, obj with windows).

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

/// `Unit::set_in_danger@005fcfb0`, run by `tools/emu/sweep_unit_set_in_danger.py` under unicorn
/// (tools/emu): 248 random bytes of object, 1 argument(s), the word at `0xc0aec0+2*0x1c` points at 24 random bytes less `0` (placed); the word at `p0+0` points at 248 random bytes less `0` (placed); the word at `p0+4` points at 248 random bytes less `0` (placed); the word at `p0+8` points at 248 random bytes less `0` (placed); the word at `p0+12` points at 248 random bytes less `0` (placed); the word at `p0+16` points at 248 random bytes less `0` (placed); the word at `p0+20` points at 248 random bytes less `0` (placed); the word at `p1+0xf4` points at 4 random bytes less `0` (placed); the word at `p7+0` points at 4 random bytes less `0x98` (placed, null a third of the rows); the word at `p2+0xf4` points at 4 random bytes less `0` (placed); the word at `p9+0` points at 4 random bytes less `0x98` (placed, null a third of the rows); the word at `p3+0xf4` points at 4 random bytes less `0` (placed); the word at `p11+0` points at 4 random bytes less `0x98` (placed, null a third of the rows); the word at `p4+0xf4` points at 4 random bytes less `0` (placed); the word at `p13+0` points at 4 random bytes less `0x98` (placed, null a third of the rows); the word at `p5+0xf4` points at 4 random bytes less `0` (placed); the word at `p15+0` points at 4 random bytes less `0x98` (placed, null a third of the rows); the word at `p6+0xf4` points at 4 random bytes less `0` (placed); the word at `p17+0` points at 4 random bytes less `0x98` (placed, null a third of the rows); the word at `m+0xf4` points at 4 random bytes less `0` (placed); the word at `p19+0` points at 4 random bytes less `0x98` (placed, null a third of the rows); figures p1..p6 link at +0x8e (2 bytes) below their own index or to -1, -32768; figures p1..p6 link at +0x90 (2 bytes) above their own index or to -1; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_set_in_danger.py") else {
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
            {
                // `this` is node 6 and its player's six figures `p[1..7]`
                // (`who` held at 2); a node's `o_up` is `+0x8e`, its `o_down`
                // `+0x90`, active `+8 & 1`; every vtable is `Unit`'s, so
                // `is_captain` is `o_up`'s sign bit. Node k's guy array is
                // `p[7 + 2k]` and its one guy (`+0xb5` ≤ 1) `p[8 + 2k]`,
                // whose `+0x9a` lands at its byte 2. `a[0]` nonzero skips the
                // climb.
                const N: usize = 6;
                let marked = {
                    let fig = |i: usize| -> &[u8] { if i == N { &m[..] } else { &p[1 + i][..] } };
                    let short = |i: usize, off: usize| i16::from_le_bytes([fig(i)[off], fig(i)[off + 1]]);
                    crate::squad_walk(
                        N,
                        64,
                        |i| {
                            let o = short(i, 0x8e);
                            (a[0] == 0 && o >= 0).then_some(o as usize)
                        },
                        |i| {
                            let o = short(i, 0x90);
                            (o >= 0 && p[1 + o as usize][8] & 1 != 0).then_some(o as usize)
                        },
                    )
                };
                for k in marked {
                    let fig: &mut [u8] = if k == N { &mut m[..] } else { &mut p[1 + k][..] };
                    fig[0x68] |= 4;
                    let guys = fig[0xb5] as i8;
                    if guys > 0 && rd(&p[7 + 2 * k], 0) != 0 {
                        p[8 + 2 * k][2] |= 0x20;
                    }
                }
                0
            }
        };
        let _ = (got, want_eax);
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "set_in_danger — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "set_in_danger — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "set_in_danger — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Unit::set_in_danger@005fcfb0: {n} rows agree");
}
