//! `NukeOut::num_current_nukes@0092b9a0` under the emulator against the port (item 2253; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `NukeOut::num_current_nukes@0092b9a0`, run by `tools/emu/sweep_obj.py` on its row `nuke_out_num_current_nukes` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 8 random bytes of object, 0 argument(s), the global cell 0x00c061e8 points at 1364 random bytes; 152 random bytes at 0x00c0a7fc; the word at `c1+0xc` points at 64 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("nuke_out_num_current_nukes") else {
        return;
    };
    let obj = super::Obj {
        name: "num_current_nukes",
        full: "NukeOut::num_current_nukes@0092b9a0",
        cells: 2,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let (n, fr, th) = (rd(&c[1], 0), rd(&c[0], 0x550), rd(&c[1], 0x94));
            let mut k = 0i64;
            for i in 0..n.max(0) as usize {
                if fr.wrapping_sub(rd(&p[0], 4 * i)) < th {
                    k += 1;
                }
            }
            k
        }
    });
}
