//! `Doober::inc_time@00846770` under the emulator against the port (item 2927; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Doober::inc_time@00846770`, run by `tools/emu/sweep_obj.py` on its row `doober_inc_time` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 512 random bytes of object, 0 argument(s), the word at `m+0x110` points at 64 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("doober_inc_time") else {
        return;
    };
    let obj = super::Obj {
        name: "inc_time",
        full: "Doober::inc_time@00846770",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // Doober::inc_time: each hold fade fades 10 a frame toward 0 (negative) or up to 0xff (positive); a negative one that
            // reaches 0 is the end of its life, popping the trailing zeros if it was the last.
            let _ = (&a, &c);
            let ptr_ok = |i: i32| (i as usize) * 4;
            let mut i: i32 = 0;
            while i < rd(m, 0xcc) {
                let o = ptr_ok(i);
                let v = rd(p[0].as_slice(), o);
                if v < 0 {
                    if v + 10 < 0 {
                        wr(&mut p[0], o, v + 10);
                    } else {
                        wr(&mut p[0], o, 0);
                        let mut n = rd(m, 0xcc).wrapping_sub(1);
                        if i == n && n >= 0 {
                            loop {
                                if rd(&p[0], (n as usize) * 4) != 0 {
                                    break;
                                }
                                wr(m, 0xcc, rd(m, 0xcc).wrapping_sub(1));
                                wr(m, 0xe8, rd(m, 0xe8).wrapping_sub(1));
                                wr(m, 0x104, rd(m, 0x104).wrapping_sub(1));
                                n -= 1;
                                if n < 0 {
                                    break;
                                }
                            }
                        }
                        wr(&mut p[0], o, 0);
                        wr(m, 0x1e0, 1);
                    }
                } else if v > 0 {
                    wr(&mut p[0], o, (v + 10).min(0xff));
                }
                i += 1;
            }
            0
        }
    });
}
