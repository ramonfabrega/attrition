//! `Map::is_near_ocean@0068b0a0` under the emulator against the port (item 3008; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Map::is_near_ocean@0068b0a0`, run by `tools/emu/sweep_obj.py` on its row `map_is_near_ocean` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 8 random bytes of object, 4 argument(s), `p` windows in order: p0..p1 pointed argument(s), p2 the place; the global cell 0x00c06188 points at 1024 random bytes; 260 random bytes at 0x00cbe330; 1024 random bytes at 0x00cb7e90; 1024 random bytes at 0x00cbb0e0; argument 0 names 4 random bytes; argument 1 names 4 random bytes; the word at `c0+0x134` points at 4096 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("map_is_near_ocean") else {
        return;
    };
    let obj = super::Obj {
        name: "is_near_ocean",
        full: "Map::is_near_ocean@0068b0a0",
        cells: 4,
        calls: false,
        eax: true,
        min_rows: 1500,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        'r: {
            let (ww, wh) = (rd(&c[0], 0), rd(&c[0], 4));
            let (x, y) = (rd(&p[0], 0), rd(&p[1], 0));
            let rad = |i: i32| rd(&c[1], 4 * i as usize);
            let land = |i: i32| -> bool {
                let tx = c[2][i as usize] as i8 as i32 + x;
                let ty = c[3][i as usize] as i8 as i32 + y;
                tx > -1
                    && ty > -1
                    && tx < ww
                    && ty < wh
                    && p[2][((ww * ty + tx) * 0x1c + 2) as usize] != 0
            };
            if x < 0 || y < 0 || ww <= x || wh <= y {
                break 'r 0;
            }
            let (p3, p4) = (a[2], a[3]);
            if p4 != 0 && !(p4 - 1 < 0x41 && p4 + 1 < 0x41 && (rad(p4 - 1)..rad(p4 + 1)).any(&land))
            {
                break 'r 0;
            }
            if p3 != 0 && (0x40 < p3 || (rad(0)..rad(p3)).any(&land)) {
                break 'r 0;
            }
            1
        }
    });
}
