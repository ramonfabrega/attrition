//! `Map::land_dist@0069d970` under the emulator against the port (item 3008; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Map::land_dist@0069d970`, run by `tools/emu/sweep_obj.py` on its row `map_land_dist` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 8 random bytes of object, 3 argument(s), `p` windows in order: p0 the place; the global cell 0x00c06188 points at 1024 random bytes; 260 random bytes at 0x00cbe330; 1024 random bytes at 0x00cb7e90; 1024 random bytes at 0x00cbb0e0; the word at `c0+0x134` points at 4096 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 600 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("map_land_dist") else {
        return;
    };
    let obj = super::Obj {
        name: "land_dist",
        full: "Map::land_dist@0069d970",
        cells: 4,
        calls: false,
        eax: true,
        min_rows: 600,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        'r: {
            let (ww, wh) = (rd(&c[0], 0), rd(&c[0], 4));
            let (x, y) = (a[0], a[1]);
            let rad = |i: i32| rd(&c[1], 4 * i as usize);
            let flags = |o: usize| u16::from_le_bytes([p[0][o], p[0][o + 1]]);
            let land = |o: usize| matches!(p[0][o + 2], 1 | 2);
            let s = ((ww * y + x) * 0x1c) as usize;
            if flags(s) & 0x100 != 0 || !land(s) {
                break 'r 0;
            }
            for r in 1..=0x40 {
                for i in rad(r - 1)..rad(r) {
                    let tx = c[2][i as usize] as i8 as i32 + x;
                    let ty = c[3][i as usize] as i8 as i32 + y;
                    if tx < 0 || ty < 0 || ww <= tx || wh <= ty {
                        if a[2] != 0 {
                            break 'r i64::from(r);
                        }
                    } else {
                        let o = ((ww * ty + tx) * 0x1c) as usize;
                        if flags(o) & 0x100 != 0 || !land(o) {
                            break 'r i64::from(r);
                        }
                    }
                }
            }
            0x41
        }
    });
}
