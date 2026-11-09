//! `RiverDoober::add_for_render@00885b80` under the emulator against the port (item 2807; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `RiverDoober::add_for_render@00885b80`, run by `tools/emu/sweep_obj.py` on its row `river_doober_add_for_render` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 0 argument(s), the global cell 0x00e335a0 points at 512 random bytes; callee 0x008871e0 (0 args) stubbed; the word at `c0+0x13c` points at 16 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("river_doober_add_for_render") else {
        return;
    };
    let obj = super::Obj {
        name: "add_for_render",
        full: "RiverDoober::add_for_render@00885b80",
        cells: 1,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // The render scalability section's byte at +6 (the pointer at +0x13c of the graphics system, the
            // global cell) gates the call to RiverDoobers::add_for_render (stub 0), handed this.
            if p[0][6] != 0 {
                log.push("m+0x0".to_string());
            } else {
                log.push("-".to_string());
            }
            0
        }
    });
}
