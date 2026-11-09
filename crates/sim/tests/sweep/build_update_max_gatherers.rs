//! `Build::update_max_gatherers@00623310` under the emulator against the port (item 2336; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Build::update_max_gatherers@00623310`, run by `tools/emu/sweep_obj.py` on its row `build_update_max_gatherers` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 144 random bytes of object, 0 argument(s), callee 0x00643440 (2 args) stubbed; callee 0x0063c430 (4 args) stubbed; the word at `m+0x18` points at 16 random bytes less `0` (placed); stub 1 answered from [0, 1, 2, 3, 255, 256, 4660] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 writes a value in -5..300 through its argument 0, printed after the answers (`o` in a place); stub 0 writes a value in -5..300 through its argument 1, printed after the answers (`o` in a place); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("build_update_max_gatherers") else {
        return;
    };
    let obj = super::Obj {
        name: "update_max_gatherers",
        full: "Build::update_max_gatherers@00623310",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // `tile_corner` (stub 0) writes the corner's two tile coordinates into two locals
            // (`o`); the type's (+0x18) `max_gatherers` (stub 1) is asked (the short at +0xa, the
            // byte at +9, the two coordinates), and its low byte is kept at +0x80.
            log.push("m+0x0,sp-0x10,sp-0xc".into());
            let short = i16::from_le_bytes([m[0xa], m[0xb]]);
            log.push(format!("p0+0x0,{},{},{},{}", short, m[9], a[2], a[3]));
            m[0x80] = a[1] as u8;
            0
        }
    });
}
