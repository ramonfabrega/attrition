//! `BuildQueue::set_queue@006309f0` under the emulator against the port (item 3008; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `BuildQueue::set_queue@006309f0`, run by `tools/emu/sweep_obj.py` on its row `build_queue_set_queue` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 8 random bytes of object, 4 argument(s), `p` windows in order: p0 argument 2, p1 the place; argument 2 names 24 random bytes (null a third of the rows); the word at `m+4` points at 128 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("build_queue_set_queue") else {
        return;
    };
    let obj = super::Obj {
        name: "set_queue",
        full: "BuildQueue::set_queue@006309f0",
        cells: 0,
        calls: false,
        eax: false,
        min_rows: 400,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let (idx, s, q) = (a[0], a[1] as i16, a[3]);
            if idx < rd(m, 0) {
                let rec = (idx * 0x14) as usize;
                let put16 = |w: &mut Vec<u8>, off: usize, v: i16| {
                    w[off..off + 2].copy_from_slice(&v.to_le_bytes())
                };
                put16(&mut p[1], rec + 4, s);
                if q == 0 {
                    p[1][rec..rec + 4].copy_from_slice(&0i32.to_le_bytes());
                }
                if a[2] != 0 {
                    let mut n = 0usize;
                    let mut done = false;
                    for k in 0..6usize {
                        let v = rd(&p[0], 4 * k);
                        if v != 0 {
                            put16(&mut p[1], rec + 6 + 2 * n, k as i16);
                            put16(&mut p[1], rec + 0xc + 2 * n, v as i16);
                            n += 1;
                            if n > 2 {
                                done = true;
                                break;
                            }
                        }
                    }
                    if !done {
                        for k in n..3 {
                            put16(&mut p[1], rec + 6 + 2 * k, -1);
                            put16(&mut p[1], rec + 0xc + 2 * k, 0);
                        }
                    }
                }
            }
            0
        }
    });
}
