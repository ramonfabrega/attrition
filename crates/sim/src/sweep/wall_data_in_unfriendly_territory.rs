//! `WallData::in_unfriendly_territory@0063eca0` under the emulator against the port (item 2247; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `WallData::in_unfriendly_territory@0063eca0`, run by `tools/emu/sweep_obj.py` on its row `wall_data_in_unfriendly_territory` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 0 argument(s), the global cell 0x00cae5fc points at 1024 random bytes; the global cell 0x00c061d0 points at 320 random bytes; the word at `c1+0x134` points at 28 random bytes less `(i32(c[1],0)*i32(c[0],4*((i32(m,0x14)^0x63637)>>8))+i32(c[0],4*((i32(m,0x10)^0x63637)>>8)))*0x1c` (placed); 8 rows 0x6eec apart at `0xc061e0`, slices +0x8:0x8c (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("wall_data_in_unfriendly_territory") else {
        return;
    };
    let obj = super::Obj {
        name: "in_unfriendly_territory",
        full: "WallData::in_unfriendly_territory@0063eca0",
        cells: 2,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // `who` is the wall's byte `+9`; the cell under it is the placed
            // record (`p[8]`), its owner byte at `+0xf`; the leaders' rows are
            // `p[0..8]`, each from `+8` (`me`) through the diplos at `+0x74`.
            let who = m[9] as usize;
            let o = i32::from(p[8][0xf] as i8);
            if o < 0 {
                0
            } else {
                let me = rd(&p[who], 0);
                let to = rd(&p[who], 0x6c + 4 * o as usize) == 2;
                let from = rd(&p[o as usize], 0x6c + 4 * me as usize) == 2;
                i64::from(!crate::cities::place::friendly_ground(o, me, to, from))
            }
        }
    });
}
