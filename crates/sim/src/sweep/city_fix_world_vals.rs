//! `City::fix_world_vals@00735aa0` under the emulator against the port (item 2217; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `City::fix_world_vals@00735aa0`, run by `tools/emu/sweep_obj.py` on its row `city_fix_world_vals` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 96 random bytes of object, 0 argument(s), the global cell 0x00cae5fc points at 1024 random bytes; the global cell 0x00c06188 points at 320 random bytes; 224 random bytes at 0x00c0ab84; 260 random bytes at 0x00cbe330; 32 random bytes at 0x00cb7e90; 32 random bytes at 0x00cbb0e0; callee 0x006db790 (1 args) stubbed; the word at `c2+0` points at 4 random bytes less `0` (placed); the word at `p0+0` points at 28 random bytes less `0` (placed); the word at `p1+0x18` points at 8 random bytes less `0` (placed); the word at `c1+0x134` points at 1008 random bytes less `0` (placed); stub 0 answers -6..80 every row; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("city_fix_world_vals") else {
        return;
    };
    let obj = super::Obj {
        name: "fix_world_vals",
        full: "City::fix_world_vals@00735aa0",
        cells: 6,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // A world of `c[1]`'s width and height in cells, `WData.val` (`+0xc`) from p[3].
            let (xs, ys) = (rd(&c[1], 0), rd(&c[1], 4));
            let mut w = crate::World::new(xs, ys);
            let cell = |i: i32| crate::world::Cell::new(i % xs, i / xs);
            for i in 0..xs * ys {
                let mut d = w.cell_data(cell(i));
                d.val = p[3][i as usize * 0x1c + 0xc];
                w.set_cell_data(cell(i), d);
            }
            let centre = crate::world::Cell::new(
                rd(&c[0], 4 * (rd(m, 0xc) >> 8) as usize),
                rd(&c[0], 4 * (rd(m, 0x10) >> 8) as usize),
            );
            let radius: Vec<usize> = (0..0x41).map(|i| rd(&c[3], 4 * i) as usize).collect();
            let x: Vec<i32> = c[4].iter().map(|&b| i32::from(b as i8)).collect();
            let y: Vec<i32> = c[5].iter().map(|&b| i32::from(b as i8)).collect();
            crate::city::wear_site_vals(&mut w, centre, a[0], &radius, &x, &y);
            for i in 0..xs * ys {
                p[3][i as usize * 0x1c + 0xc] = w.cell_data(cell(i)).val;
            }
            0
        }
    });
}
