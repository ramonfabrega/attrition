//! `RiverDoobers::add_doobers_for_render@00887700` under the emulator against the port (item 2588; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `RiverDoobers::add_doobers_for_render@00887700`, run by `tools/emu/sweep_obj.py` on its row `river_doobers_add_doobers_for_render` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 1 argument(s), argument 0 names 4 random bytes; `__cdecl`: the caller pops the arguments; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("river_doobers_add_doobers_for_render") else {
        return;
    };
    let obj = super::Obj {
        name: "add_doobers_for_render",
        full: "RiverDoobers::add_doobers_for_render@00887700",
        cells: 0,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        0
    });
}
