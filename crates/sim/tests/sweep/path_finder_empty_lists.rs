//! `PathFinder::empty_lists@00689c70` under the emulator against the port (item 2281; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `PathFinder::empty_lists@00689c70`, run by `tools/emu/sweep_obj.py` on its row `path_finder_empty_lists` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 8 random bytes of object, 2 argument(s), argument 0 names 16 random bytes (null a third of the rows); argument 1 names 16 random bytes (null a third of the rows); callee 0x00687c10 (1 args) stubbed; callee 0x00687ba0 (1 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("path_finder_empty_lists") else {
        return;
    };
    let obj = super::Obj {
        name: "empty_lists",
        full: "PathFinder::empty_lists@00689c70",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // Each list's head, non-null in a non-null list, handed to its own
            // `kill_tree` with `this` (item 2281).
            for k in 0..2 {
                log.push(if a[k] != 0 && rd(&p[k], 0xc) != 0 {
                    format!("m+0x0,{}", rd(&p[k], 0xc))
                } else {
                    "-".into()
                });
            }
            0
        }
    });
}
