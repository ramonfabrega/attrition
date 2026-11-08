//! `Wall::kill_competing_buildings@0063d3a0` under the emulator against the port (item 3044; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Wall::kill_competing_buildings@0063d3a0`, run by `tools/emu/sweep_obj.py` on its row `wall_kill_competing_buildings` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 0 argument(s), `p` windows in order: p0 the place; the global cell 0x00c06188 points at 32 random bytes; callee 0x00643440 (2 args) stubbed; callee 0x0063d440 (2 args) stubbed; the word at `m+0x18` points at 576 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 0 writes a value in -2..5 through its argument 0, printed after the answers (`o` in a place); stub 0 writes a value in -2..5 through its argument 1, printed after the answers (`o` in a place); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("wall_kill_competing_buildings") else {
        return;
    };
    let obj = super::Obj {
        name: "kill_competing_buildings",
        full: "Wall::kill_competing_buildings@0063d3a0",
        cells: 1,
        calls: true,
        eax: false,
        min_rows: 400,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // `tile_corner` (stub 0) writes the corner `(a[2], a[3])` through its two out pointers; the
        // type record `p[0]` is `+0x234` wide and `+0x238` tall; every tile of it inside the
        // world (`c0+0x18` by `c0+0x1c`) goes to `kill_at_tile` (stub 1), columns outer.
        let (x0, y0) = (a[2], a[3]);
        let (tw, th) = (rd(&p[0], 0x234), rd(&p[0], 0x238));
        let (w, h) = (rd(&c[0], 0x18), rd(&c[0], 0x1c));
        log.push("s0:m+0x0,sp-0x10,sp-0xc".to_string());
        for x in x0..x0 + tw {
            for y in y0..y0 + th {
                if x >= 0 && y >= 0 && x < w && y < h {
                    log.push(format!("s1:m+0x0,{x},{y}"));
                }
            }
        }
        0
    });
}
