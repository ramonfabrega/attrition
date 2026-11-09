//! `PathFinder::find_road_restore@00685950` under the emulator against the port (item 2513; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `PathFinder::find_road_restore@00685950`, run by `tools/emu/sweep_obj.py` on its row `path_finder_find_road_restore` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 1024 random bytes of object, 7 argument(s), 4 random bytes at 0x00e85ec4; callee 0x00688a40 (7 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 prints the 4 bytes at the cell 0x00e85ec4 when it is called, hex, after the record (`/` a call); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("path_finder_find_road_restore") else {
        return;
    };
    let obj = super::Obj {
        name: "find_road_restore",
        full: "PathFinder::find_road_restore@00685950",
        cells: 1,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            log.push(format!(
                "m+0x0,{},{},{},{},{},{},{}",
                a[0], a[1], a[2], a[3], a[4], a[5], a[6]
            ));
            log.push("01000000".into());
            wr(&mut c[0], 0, 0);
            i64::from(a[7])
        }
    });
}
