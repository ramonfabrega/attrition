//! `Build::find_gather_tiles@00623350` under the emulator against the port (item 3238; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Build::find_gather_tiles@00623350`, run by `tools/emu/sweep_obj.py` on its row `build_find_gather_tiles` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 176 random bytes of object, 0 argument(s), `p` windows in order: p0..p2 the place(s), in flag order; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00cae5fc points at 4096 random bytes; vtable slot +0x90 (0 args) stubbed; vtable slot +0x94 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; callee 0x0063bdc0 (4 args) stubbed; callee 0x00a39d70 (2 args) stubbed; callee 0x0063c430 (4 args) stubbed; the word at `m+0x18` points at 576 random bytes less `0` (placed); the word at `m+0xa8` points at 128 random bytes less `0` (placed); the word at `c0+0x138` points at 1024 random bytes less `0` (placed); stub 0 answered from [1, 1, 1, 1, 0] (90%); stub 1 answered from [0, 0, 0, 0, 1] (90%); stub 2 answered from [0, 0, 0, 0, 1] (90%); stub 4 answers 30 values in call order, printed after the answers; stub 5 answers 0..127 every row; stub 4 answers 0..65534 every row; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 writes 6 (4 bytes) at `m+0x9c` each call; the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("build_find_gather_tiles") else {
        return;
    };
    let obj = super::Obj {
        name: "find_gather_tiles",
        full: "Build::find_gather_tiles@00623350",
        cells: 2,
        calls: true,
        eax: false,
        min_rows: 1500,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // Windows: m the Build, c0 `GameAccess::world` (stride at +0x18), p0 the type (sizes at
            // +0x234/+0x238), p1 the MiningList's tcoords (x, y words), p2 the world's u16 tile masks.
            // Answers: a[0] `is_gather`, a[1] `is_flat`, a[2] the university test, a[3] the
            // `find_gather_tcoords` stub (it appends the list's tail: it writes the count 6 and the
            // tiles are already in p1), a[4] unused, a[5] `max_gatherers`, a[6..] the shuffle's
            // `Random::get(0, 0xffff)` answers in call order.
            // The crate's own `extend_and_shuffle_gather_list` runs the append and the shuffle
            // (`Sim::find_gather_tiles` calls it); the gate, the `0x1000` marking and `gather_max`
            // are the sweep's, since the crate method needs a whole `Sim` world for `gather_tcoords`.
            use crate::Pos;
            use crate::cities::place::tile_corner_of;
            use crate::economy::gather::extend_and_shuffle_gather_list;
            const K: i32 = 0x63637;
            let mut toks = ["-".to_string(), "-".to_string()];
            if a[0] != 0 && a[1] == 0 && a[2] == 0 {
                let before = rd(m, 0x9c) as usize;
                let tile = |i: usize| Pos::new(rd(&p[1], 8 * i), rd(&p[1], 8 * i + 4));
                let mut list: Vec<Pos> = (0..before).map(tile).collect();
                let found: Vec<Pos> = (before..6).map(tile).collect();
                let (x, y) = (rd(m, 0x10) ^ K, rd(m, 0x14) ^ K);
                let who = i32::from(m[9]);
                toks[0] = format!("p0+0x0,{x},{y},m+0x98,{who}");
                let mut next = 6;
                extend_and_shuffle_gather_list(&mut list, found, |n| {
                    if n <= 1 {
                        return 0;
                    }
                    next += 1;
                    a[next - 1] % n
                });
                let stride = rd(&c[0], 0x18);
                for (i, t) in list.iter().enumerate() {
                    wr(&mut p[1], 8 * i, t.x);
                    wr(&mut p[1], 8 * i + 4, t.y);
                    let o = 2 * (t.y * stride + t.x) as usize;
                    let v = u16::from_le_bytes([p[2][o], p[2][o + 1]]) | 0x1000;
                    p[2][o..o + 2].copy_from_slice(&v.to_le_bytes());
                }
                wr(m, 0x9c, list.len() as i32);
                let (xs, ys) = (rd(&p[0], 0x234), rd(&p[0], 0x238));
                let corner = tile_corner_of(xs, ys, Pos::new(x, y));
                let ty = i32::from(i16::from_le_bytes([m[0xa], m[0xb]]));
                toks[1] = format!("p0+0x0,{ty},{who},{},{}", corner.x, corner.y);
                m[0x80] = a[5] as u8;
            }
            log.extend(toks);
            0
        }
    });
}
