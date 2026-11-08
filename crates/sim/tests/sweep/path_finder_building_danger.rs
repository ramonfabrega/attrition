//! `PathFinder::building_danger@00685900` under the emulator against the port (item 2129; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `PathFinder::building_danger@00685900`, run by `tools/emu/sweep_obj.py` on its row `path_finder_building_danger` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 4 argument(s), the global cell 0x00c06188 points at 320 random bytes; the word at `c0+0x138` points at 4 random bytes less `(i32(c[0],0x18)*a[1]+a[0])*2` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("path_finder_building_danger") else {
        return;
    };
    let obj = super::Obj {
        name: "building_danger",
        full: "PathFinder::building_danger@00685900",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let t = u16::from_le_bytes([p[0][0], p[0][1]]);
            if t & 3 == 3 && (t >> 14) & 1 != 0 {
                4000
            } else {
                0
            }
        }
    });
}
