//! `Build::verify_gather_tiles@00623570` under the emulator against the port (item 2749; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Build::verify_gather_tiles@00623570`, run by `tools/emu/sweep_obj.py` on its row `build_verify_gather_tiles` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 176 random bytes of object, 0 argument(s), 640 random bytes at 0x00e3a390; 640 random bytes at 0x00e4127c; callee 0x0046d4f0 (2 args) stubbed; the word at `0xc06188` points at 512 random bytes less `0` (placed); the word at `p0+0x134` points at 256 random bytes less `0` (placed); the word at `p0+0x138` points at 256 random bytes less `0` (placed); the word at `0xc061e0` points at 90112 random bytes less `0` (placed); the word at `m+0xa8` points at 64 random bytes less `0` (placed); stub 0 lowers `m+0x9c` (4 bytes) by one each call; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("build_verify_gather_tiles") else {
        return;
    };
    let obj = super::Obj {
        name: "verify_gather_tiles",
        full: "Build::verify_gather_tiles@00623570",
        cells: 2,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // Array::remove (stub 0, hooked) lowers the count at this+0x9c by one a call (--stub-write dec)
            // and moves nothing, so the same record is read again. Windows: p[0] the world, p[1] its owner
            // grid, p[2] its u16 flags, p[3] leadersc, p[4] the records; c[0], c[1] the leaders of who 0, 1.
            let who = usize::from(m[9]);
            let mut i = 0usize;
            while i32::try_from(i).expect("an index") < rd(m, 0x9c) {
                let tx = rd(&p[4], 8 * i);
                let ty = rd(&p[4], 8 * i + 4);
                let at = usize::try_from(((ty >> 2) * rd(&p[0], 0) + (tx >> 2)) * 0x1c + 0xf)
                    .expect("a cell");
                let owner = i32::from(p[1][at] as i8);
                let mut removed = false;
                if owner >= 0 {
                    let mine = rd(&c[who], 8);
                    let lead = |w: &[u8], off: i32| rd(w, usize::try_from(off).expect("an offset"));
                    if owner != mine
                        && (lead(&c[who], 0x74 + owner * 4) != 2
                            || lead(&p[3], 0x74 + (owner * 0x1bbb + mine) * 4) != 2)
                    {
                        let f = usize::try_from((rd(&p[0], 0x18) * ty + tx) * 2).expect("a flag");
                        let v = u16::from_le_bytes([p[2][f], p[2][f + 1]]) & 0xefff;
                        p[2][f..f + 2].copy_from_slice(&v.to_le_bytes());
                        let n = rd(m, 0x9c);
                        wr(m, 0x9c, n - 1);
                        removed = true;
                    }
                }
                if !removed {
                    i += 1;
                }
            }
            0
        }
    });
}
