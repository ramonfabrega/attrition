//! `World::set_land@006b2b50` under the emulator against the port (item 2844; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `World::set_land@006b2b50`, run by `tools/emu/sweep_obj.py` on its row `world_set_land` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 6 argument(s), the global cell 0x00c06188 points at 312 random bytes; argument 0 names 4 random bytes; argument 1 names 4 random bytes; the word at `c0+0x134` points at 448 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("world_set_land") else {
        return;
    };
    let obj = super::Obj {
        name: "set_land",
        full: "World::set_land@006b2b50",
        cells: 1,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        // The tile at (`p[0]`, `p[1]`) of the world's grid (`c[0]` its width, `p[2]` the 0x1c-byte
        // tiles): the bytes at +3 (`a[3]`) and +2 (`a[2]`) set when they are not negative; and, when
        // `a[4]` is not negative, the word at +0 has its 0x3c bits cleared (and 0x500 too when `a[5]` is
        // set), `a[4]` or'd in, and 0x400 more when `a[4]` is 4 (item 2844).
        let t = (rd(&p[1], 0)
            .wrapping_mul(rd(&c[0], 0))
            .wrapping_add(rd(&p[0], 0)) as usize)
            * 0x1c;
        if a[3] >= 0 {
            p[2][t + 3] = a[3] as u8;
        }
        if a[2] >= 0 {
            p[2][t + 2] = a[2] as u8;
        }
        if a[4] >= 0 {
            let mut w = u16::from_le_bytes([p[2][t], p[2][t + 1]]) & 0xffc3;
            if a[5] != 0 {
                w &= 0xfaff;
            }
            w |= a[4] as u16;
            if a[4] == 4 {
                w |= 0x400;
            }
            p[2][t..t + 2].copy_from_slice(&w.to_le_bytes());
        }
        0
    });
}
