//! `Terrain::refresh_good_z@00850b60` under the emulator against the port (item 3044; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Terrain::refresh_good_z@00850b60`, run by `tools/emu/sweep_obj.py` on its row `terrain_refresh_good_z` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 1 argument(s), `p` windows in order: p0..p7 the place(s), in flag order; 4 random bytes at 0x00c0a0e4; 4 random bytes at 0x00c0a104; 4 random bytes at 0x00c06218; callee 0x00866560 (4 args) stubbed; the word at `0xc0a0f0` points at 16 random bytes less `0` (placed); the word at `0xc0a110` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `p0+8` points at 32 random bytes less `0` (placed); the word at `p1+0` points at 32 random bytes less `0` (placed); the word at `p1+4` points at 32 random bytes less `0` (placed); the word at `p1+8` points at 32 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 0 writes a value in 0..262143 through its argument 0, printed after the answers (`o` in a place); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("terrain_refresh_good_z") else {
        return;
    };
    let obj = super::Obj {
        name: "refresh_good_z",
        full: "Terrain::refresh_good_z@00850b60",
        cells: 3,
        calls: true,
        eax: false,
        min_rows: 400,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // The goods (`c0` the count, list `p[0]`, record `j` in `p[2+j]`) then the items (`c1`, `p[1]`,
        // `p[5+j]`): each live record (bit 0 of `+8`) is handed to the terrain's `find_data_z` (the
        // cell `c2`; `a[2]` the z it writes through its out pointer) with its position words
        // `+0x10`, `+0x14` unmasked (^ 0x63637), and its `+0xc` becomes the answer masked again.
        let terrain = rd(&c[2], 0);
        for (count, first) in [(rd(&c[0], 0), 2usize), (rd(&c[1], 0), 5)] {
            for j in 0..count.max(0) as usize {
                let r = &mut p[first + j];
                if r[8] & 1 != 0 {
                    let x = rd(r, 0x10) ^ 0x63637;
                    let y = rd(r, 0x14) ^ 0x63637;
                    log.push(format!("s0:{terrain},sp-0x10,{x},{y},1"));
                    wr(r, 0xc, a[2] ^ 0x63637);
                }
            }
        }
        if log.is_empty() {
            log.push("-".to_string());
        }
        0
    });
}
