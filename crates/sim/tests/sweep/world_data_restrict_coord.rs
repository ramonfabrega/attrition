//! `WorldData::restrict@006b53a0` under the emulator against the port (item 2044; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `WorldData::restrict@006b53a0`, run by `tools/emu/sweep_obj.py` on its row `world_data_restrict_coord` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 8 random bytes of object, 2 argument(s), argument 0 names 4 random bytes; argument 1 names 4 random bytes; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
/// The world's cell size times `0x300` (`world::UNITS_PER_CELL`, wrapping as the original's `lea`/`shl`) clamps
/// each Coord; `army.rs`'s `restrict` is the port.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("world_data_restrict_coord") else {
        return;
    };
    let obj = super::Obj {
        name: "restrict",
        full: "WorldData::restrict@006b53a0",
        cells: 0,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let (w, h) = (rd(m, 0).wrapping_mul(0x300), rd(m, 4).wrapping_mul(0x300));
            let x = rd(&p[0], 0).max(0);
            wr(&mut p[0], 0, if x >= w { w.wrapping_sub(1) } else { x });
            let y = rd(&p[1], 0).max(0);
            wr(&mut p[1], 0, if y >= h { h.wrapping_sub(1) } else { y });
            0
        }
    });
}
