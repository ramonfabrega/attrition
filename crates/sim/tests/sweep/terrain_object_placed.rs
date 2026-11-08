//! `Terrain::object_placed@00850c40` under the emulator against the port (item 3064; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Terrain::object_placed@00850c40`, run by `tools/emu/sweep_obj.py` on its row `terrain_object_placed` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 21504 random bytes of object, 5 argument(s), `p` windows in order: p0..p1 the place(s), in flag order; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00c061d0 points at 4 random bytes; the global cell 0x00c06218 points at 27328 random bytes; the global cell 0x00e885f0 points at 576 random bytes; callee 0x0084fa50 (4 args) stubbed; callee 0x008b13d0 (1 args) stubbed; the word at `c0+0x134` points at 1024 random bytes less `0` (placed); the word at `c2+0x6ab0` points at 176 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("terrain_object_placed") else {
        return;
    };
    let obj = super::Obj {
        name: "object_placed",
        full: "Terrain::object_placed@00850c40",
        cells: 4,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // `a[0]`, `a[1]` the corner, `a[2]`, `a[3]` the extent, `a[4]` the flag. The world `c[0]` holds its width
        // and height (`+0`, `+4`) and the tile table `p[0]` (0x1c a tile, a byte at `+2`); a tile whose byte is zero
        // calls stub 0 (`spot_update_land`) with the tile, 0 and the tileset's byte `c[3][0x228]`; every tile
        // calls stub 1 (`TerrainVis::invalidate_wcoord`, `this + 0x533c`) with the word of `p[1]` at
        // `worldc_width * y + x`.
        let tokens = &mut *log;
        if a[4] != 0 {
            let (w, h) = (rd(&c[0], 0), rd(&c[0], 4));
            let cw = rd(&c[1], 0);
            let weather = c[3][0x228];
            let xe = ((a[0] + a[2]) >> 2) + 1;
            let ye = ((a[1] + a[3]) >> 2) + 1;
            let mut y = a[1] >> 2;
            while y < ye {
                let mut x = a[0] >> 2;
                while x < xe {
                    if x > -1 && y > -1 && x < w && y < h {
                        if p[0][(w * y + x) as usize * 0x1c + 2] == 0 {
                            tokens.push(format!("s0:m+0x0,{x},{y},0,{weather}"));
                        }
                        let v = rd(&p[1], (cw * y + x) as usize * 4);
                        tokens.push(format!("s1:m+0x533c,{v}"));
                    }
                    x += 1;
                }
                y += 1;
            }
        }
        if tokens.is_empty() {
            tokens.push("-".to_string());
        }
        0
    });
}
