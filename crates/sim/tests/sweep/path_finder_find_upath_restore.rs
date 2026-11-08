//! `PathFinder::find_upath_restore@00688f40` under the emulator against the port (item 2513; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `PathFinder::find_upath_restore@00688f40`, run by `tools/emu/sweep_obj.py` on its row `path_finder_find_upath_restore` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 1024 random bytes of object, 3 argument(s), the global cell 0x00c061bc points at 16 random bytes; the global cell 0x00c0618c points at 144 random bytes; 8 random bytes at 0x00e85ec0; callee 0x00682f30 (6 args) stubbed; the word at `c1+0x14+0x1c*a[1]` points at 16 random bytes less `0` (placed); the word at `p0+4*a[2]` points at 24 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 prints the 4 bytes at the cell 0x00e85ec4 when it is called, hex, after the record (`/` a call); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("path_finder_find_upath_restore") else {
        return;
    };
    let obj = super::Obj {
        name: "find_upath_restore",
        full: "PathFinder::find_upath_restore@00688f40",
        cells: 3,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            let x = rd(&p[1], 0x10) ^ 0x63637;
            let y = rd(&p[1], 0x14) ^ 0x63637;
            let sq = rd(&c[0], 4 * a[1] as usize)
                .wrapping_mul(rd(&c[0], 4 * a[1] as usize))
                .max(1);
            wr(&mut c[2], 0, 300 / sq);
            log.push(format!("{},{},{},{},{},{},0", y, a[0], x, y, a[1], a[2]));
            log.push("01000000".into());
            wr(&mut c[2], 4, 0);
            i64::from(a[3])
        }
    });
}
