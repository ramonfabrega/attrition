//! `Unit::set_group@00605220` under the emulator against the port (item 2276; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Unit::set_group@00605220`, run by `tools/emu/sweep_obj.py` on its row `unit_set_group` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 152 random bytes of object, 2 argument(s), the word at `0xc0618c` points at 80 random bytes less `0` (placed); the word at `p0+0x4c` points at 24 random bytes less `0` (placed); the word at `p1+0` points at 152 random bytes less `0` (placed); the word at `p1+4` points at 152 random bytes less `0` (placed); the word at `p1+8` points at 152 random bytes less `0` (placed); the word at `p1+12` points at 152 random bytes less `0` (placed); the word at `p1+16` points at 152 random bytes less `0` (placed); the word at `p1+20` points at 152 random bytes less `0` (placed); figures p2..p7 link at +0x8e (2 bytes) below their own index or to -1, -32768; figures p2..p7 link at +0x90 (2 bytes) above their own index or to -1; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_set_group") else {
        return;
    };
    let obj = super::Obj {
        name: "set_group",
        full: "Unit::set_group@00605220",
        cells: 0,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // `this` is node 6 and its player's six figures `p[2..8]`
            // (`who` held at 2, the objects record `p[0]`, its array
            // `p[1]`); a node's `o_up` is `+0x8e`, its `o_down` `+0x90`,
            // active `+8 & 1`; every vtable is `Unit`'s, so `is_captain`
            // is `o_up`'s sign bit and `+0xa8` answers the figure itself.
            // `a[1]` nonzero skips the climb.
            const N: usize = 6;
            let walked = {
                let fig = |i: usize| -> &[u8] { if i == N { &m[..] } else { &p[2 + i][..] } };
                let short =
                    |i: usize, off: usize| i16::from_le_bytes([fig(i)[off], fig(i)[off + 1]]);
                crate::squad_walk(
                    N,
                    64,
                    |i| {
                        let o = short(i, 0x8e);
                        (a[1] == 0 && o >= 0).then_some(o as usize)
                    },
                    |i| {
                        let o = short(i, 0x90);
                        (o >= 0 && p[2 + o as usize][8] & 1 != 0).then_some(o as usize)
                    },
                )
            };
            for k in walked {
                let fig: &mut [u8] = if k == N {
                    &mut m[..]
                } else {
                    &mut p[2 + k][..]
                };
                fig[0x80..0x82].copy_from_slice(&(a[0] as i16).to_le_bytes());
            }
            0
        }
    });
}
