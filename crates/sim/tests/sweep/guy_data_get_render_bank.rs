//! `GuyData::get_render_bank@005de530` under the emulator against the port (item 2537; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `GuyData::get_render_bank@005de530`, run by `tools/emu/sweep_obj.py` on its row `guy_data_get_render_bank` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 128 random bytes of object, 0 argument(s), the global cell 0x00c061e8 points at 2304 random bytes; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("guy_data_get_render_bank") else {
        return;
    };
    let obj = super::Obj {
        name: "get_render_bank",
        full: "GuyData::get_render_bank@005de530",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let (a_, b_) = (
                f32::from_bits(rd(m, 0x44) as u32),
                f32::from_bits(rd(m, 0x48) as u32),
            );
            if a_ == b_ {
                i64::from(a_.to_bits() as i32)
            } else {
                let al = f32::from_bits(rd(&c[0], 0x854) as u32);
                i64::from(((1.0f32 - al) * b_ + a_ * al).to_bits() as i32)
            }
        }
    });
}
