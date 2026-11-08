//! `PathFinder::add_to_openlist@00687aa0` under the emulator against the port (item 2644; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `PathFinder::add_to_openlist@00687aa0`, run by `tools/emu/sweep_obj.py` on its row `path_finder_add_to_openlist` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 80 random bytes of object, 1 argument(s), argument 0 names 32 random bytes; callee 0x004796f0 (2 args) stubbed; callee 0x00479920 (2 args) stubbed; the word at `m+0x40` points at 32 random bytes less `0` (placed); the word at `m+0x44` points at 32 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("path_finder_add_to_openlist") else {
        return;
    };
    let obj = super::Obj {
        name: "add_to_openlist",
        full: "PathFinder::add_to_openlist@00687aa0",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            log.push(format!("p1+0x0,p0+0x0,{}", rd(&p[0], 0x10)));
            log.push(format!("p2+0x0,{},{}", rd(&p[1], 0x10), rd(&p[0], 0x18)));
            0
        }
    });
}
