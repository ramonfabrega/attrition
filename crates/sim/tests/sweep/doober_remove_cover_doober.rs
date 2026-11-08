//! `Doober::remove_cover_doober@00846a30` under the emulator against the port (item 3008; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Doober::remove_cover_doober@00846a30`, run by `tools/emu/sweep_obj.py` on its row `doober_remove_cover_doober` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 512 random bytes of object, 5 argument(s), `p` windows in order: p0..p5 the place(s), in flag order; the word at `m+0x148` points at 64 random bytes less `0` (placed); the word at `m+0x164` points at 128 random bytes less `0` (placed); the word at `m+0x180` points at 64 random bytes less `0` (placed); the word at `m+0x19c` points at 64 random bytes less `0` (placed); the word at `m+0x1b8` points at 64 random bytes less `0` (placed); the word at `m+0x1d4` points at 64 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("doober_remove_cover_doober") else {
        return;
    };
    let obj = super::Obj {
        name: "remove_cover_doober",
        full: "Doober::remove_cover_doober@00846a30",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 3000,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let sx = |w: &[u8], off: usize| i32::from(i16::from_le_bytes([w[off], w[off + 1]]));
            let mut i: i32 = 0;
            let mut n = rd(m, 0x13c);
            if n > 0 {
                loop {
                    let u = i as usize;
                    if a[0] == rd(&p[2], 4 * u)
                        && a[1] == sx(&p[1], 8 * u)
                        && a[2] == sx(&p[1], 8 * u + 4)
                        && a[3] == rd(&p[3], 4 * u)
                        && a[4] == rd(&p[4], 4 * u)
                    {
                        let last = |m: &[u8], off: usize| (rd(m, off) - 1) as usize;
                        let l0 = last(m, 0x13c);
                        let v = rd(&p[0], 4 * l0);
                        wr(&mut p[0], 4 * u, v);
                        let l1 = last(m, 0x158);
                        let (v0, v1) = (rd(&p[1], 8 * l1), rd(&p[1], 8 * l1 + 4));
                        wr(&mut p[1], 8 * u, v0);
                        wr(&mut p[1], 8 * u + 4, v1);
                        for (k, off) in [(2usize, 0x174usize), (3, 0x190), (4, 0x1ac), (5, 0x1c8)] {
                            let l = last(m, off);
                            let v = rd(&p[k], 4 * l);
                            wr(&mut p[k], 4 * u, v);
                        }
                        for off in [0x13cusize, 0x158, 0x174, 0x190, 0x1ac, 0x1c8] {
                            let v = rd(m, off) - 1;
                            wr(m, off, v);
                        }
                        i -= 1;
                        wr(m, 0x1e0, 1);
                    }
                    n = rd(m, 0x13c);
                    i += 1;
                    if i >= n {
                        break;
                    }
                }
            }
            0
        }
    });
}
