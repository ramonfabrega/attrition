//! `Guy::turn_angles@005d98c0` under the emulator against the port (item 2241; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Guy::turn_angles@005d98c0`, run by `tools/emu/sweep_obj.py` on its row `guy_turn_angles` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 4 argument(s), argument 1 names 4 random bytes; callee 0x005de340 (1 args) stubbed; stub 0 answered from [0, 1, 16777216, 35791392, 67108864, 268435456, 1073741824, 2147483647, -1] (90%); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("guy_turn_angles") else {
        return;
    };
    let obj = super::Obj {
        name: "turn_angles",
        full: "Guy::turn_angles@005d98c0",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let cur = rd(m, 0x18) as u32;
            let d = (a[0] as u32).wrapping_sub(cur);
            let e = if d > 0x8000_0000 { !d } else { d };
            if e < 0x222_2220 {
                wr(&mut p[0], 0, a[0]);
                0
            } else {
                let mut s = a[4] as u32;
                if a[3] != 0 {
                    s >>= 1;
                }
                if e <= s {
                    wr(&mut p[0], 0, a[0]);
                    0
                } else {
                    let to = if d <= 0x8000_0000 {
                        cur.wrapping_add(s)
                    } else {
                        cur.wrapping_sub(s)
                    };
                    wr(&mut p[0], 0, to as i32);
                    i64::from((e - s) as i32)
                }
            }
        }
    });
}
