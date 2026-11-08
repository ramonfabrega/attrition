//! `Unit::set_in_danger@005fcfb0` under the emulator against the port (item 2276; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Unit::set_in_danger@005fcfb0`, run by `tools/emu/sweep_obj.py` on its row `unit_set_in_danger` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 248 random bytes of object, 1 argument(s), the word at `0xc0aec0+2*0x1c` points at 24 random bytes less `0` (placed); the word at `p0+0` points at 248 random bytes less `0` (placed); the word at `p0+4` points at 248 random bytes less `0` (placed); the word at `p0+8` points at 248 random bytes less `0` (placed); the word at `p0+12` points at 248 random bytes less `0` (placed); the word at `p0+16` points at 248 random bytes less `0` (placed); the word at `p0+20` points at 248 random bytes less `0` (placed); the word at `p1+0xf4` points at 4 random bytes less `0` (placed); the word at `p7+0` points at 4 random bytes less `0x98` (placed, null a third of the rows); the word at `p2+0xf4` points at 4 random bytes less `0` (placed); the word at `p9+0` points at 4 random bytes less `0x98` (placed, null a third of the rows); the word at `p3+0xf4` points at 4 random bytes less `0` (placed); the word at `p11+0` points at 4 random bytes less `0x98` (placed, null a third of the rows); the word at `p4+0xf4` points at 4 random bytes less `0` (placed); the word at `p13+0` points at 4 random bytes less `0x98` (placed, null a third of the rows); the word at `p5+0xf4` points at 4 random bytes less `0` (placed); the word at `p15+0` points at 4 random bytes less `0x98` (placed, null a third of the rows); the word at `p6+0xf4` points at 4 random bytes less `0` (placed); the word at `p17+0` points at 4 random bytes less `0x98` (placed, null a third of the rows); the word at `m+0xf4` points at 4 random bytes less `0` (placed); the word at `p19+0` points at 4 random bytes less `0x98` (placed, null a third of the rows); figures p1..p6 link at +0x8e (2 bytes) below their own index or to -1, -32768; figures p1..p6 link at +0x90 (2 bytes) above their own index or to -1; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_set_in_danger") else {
        return;
    };
    let obj = super::Obj {
        name: "set_in_danger",
        full: "Unit::set_in_danger@005fcfb0",
        cells: 0,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
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
                let short =
                    |i: usize, off: usize| i16::from_le_bytes([fig(i)[off], fig(i)[off + 1]]);
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
                let fig: &mut [u8] = if k == N {
                    &mut m[..]
                } else {
                    &mut p[1 + k][..]
                };
                fig[0x68] |= 4;
                let guys = fig[0xb5] as i8;
                if guys > 0 && rd(&p[7 + 2 * k], 0) != 0 {
                    p[8 + 2 * k][2] |= 0x20;
                }
            }
            0
        }
    });
}
