//! `Guy::set_pivot_angle@005d8fc0` under the emulator against the port (item 2241; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Guy::set_pivot_angle@005d8fc0`, run by `tools/emu/sweep_obj.py` on its row `guy_set_pivot_angle` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 160 random bytes of object, 2 argument(s), no windows; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("guy_set_pivot_angle") else {
        return;
    };
    let obj = super::Obj {
        name: "set_pivot_angle",
        full: "Guy::set_pivot_angle@005d8fc0",
        cells: 0,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let mut d = (rd(m, 0x20) as u32).wrapping_neg();
            wr(m, 0x30, 0);
            if d > 0x8000_0000 {
                d = !d;
            }
            let w = u16::from_le_bytes([m[0x98], m[0x99]]) | 1;
            m[0x98..0x9a].copy_from_slice(&w.to_le_bytes());
            if d < 0xaaa_aaaa {
                let w = u16::from_le_bytes([m[0x96], m[0x97]]) | 1;
                m[0x96..0x98].copy_from_slice(&w.to_le_bytes());
            }
            0
        }
    });
}
