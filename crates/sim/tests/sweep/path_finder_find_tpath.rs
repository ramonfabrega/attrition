//! `PathFinder::find_tpath@00688e60` under the emulator against the port (item 2495; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `PathFinder::find_tpath@00688e60`, run by `tools/emu/sweep_obj.py` on its row `path_finder_find_tpath` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 1024 random bytes of object, 3 argument(s), the global cell 0x00c0618c points at 144 random bytes; callee 0x006897d0 (5 args) stubbed; the word at `c0+0x14+0x1c*a[1]` points at 16 random bytes less `0` (placed); the word at `p0+4*a[2]` points at 24 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("path_finder_find_tpath") else {
        return;
    };
    let obj = super::Obj {
        name: "find_tpath",
        full: "PathFinder::find_tpath@00688e60",
        cells: 1,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            let x = rd(&p[1], 0x10) ^ 0x63637;
            let y = rd(&p[1], 0x14) ^ 0x63637;
            log.push(format!("{},{},{},{},{},{}", y, a[0], x, y, a[1], a[2]));
            i64::from(a[3])
        }
    });
}
