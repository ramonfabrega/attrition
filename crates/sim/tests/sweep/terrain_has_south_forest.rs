//! `Terrain::has_south_forest@00850060` under the emulator against the port (item 2877; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Terrain::has_south_forest@00850060`, run by `tools/emu/sweep_obj.py` on its row `terrain_has_south_forest` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 2 argument(s), the global cell 0x00c06188 points at 316 random bytes; argument 0 names 4 random bytes; argument 1 names 4 random bytes; the word at `c0+0x134` points at 1024 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("terrain_has_south_forest") else {
        return;
    };
    let obj = super::Obj {
        name: "has_south_forest",
        full: "Terrain::has_south_forest@00850060",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let (x, y) = (rd(&p[0], 0), rd(&p[1], 0));
            let (w, h) = (rd(&c[0], 0), rd(&c[0], 4));
            let flag = |cx: i32, cy: i32| {
                cx >= 0
                    && cy >= 0
                    && cx < w
                    && cy < h
                    && p[2][((w.wrapping_mul(cy).wrapping_add(cx)) as usize).wrapping_mul(0x1c)]
                        & 0x20
                        != 0
            };
            i64::from(flag(x + 1, y + 1) || flag(x, y + 1) || flag(x + 1, y))
        }
    });
}
