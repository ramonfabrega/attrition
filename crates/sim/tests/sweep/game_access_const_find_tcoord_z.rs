//! `GameAccessConst::find_tcoord_z@005834b0` under the emulator against the port (item 2346; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `GameAccessConst::find_tcoord_z@005834b0`, run by `tools/emu/sweep_obj.py` on its row `game_access_const_find_tcoord_z` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 2 argument(s), the global cell 0x00c06218 points at 16 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; callee 0x008544a0 (4 args) stubbed; `__cdecl`: the caller pops the arguments; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `ecx` an integer in -2147483648..2147483647, not the object, the row's last; `edx` an integer in -2147483648..2147483647, after `ecx` on the row; `ecx` and `edx` held to their draw every row, no wild word; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("game_access_const_find_tcoord_z") else {
        return;
    };
    let obj = super::Obj {
        name: "find_tcoord_z",
        full: "GameAccessConst::find_tcoord_z@005834b0",
        cells: 2,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // The terrain (cell 0xc06218) asked for the height into the caller's `Coord` (`ecx`, the
            // hidden return pointer) at the tile the two coordinates name — each `>> 6` through
            // `div_3_table` (cell 0xcae5fc) — with `edx` passed through; `ecx` answered.
            let (x, y) = (
                rd(&c[1], (a[0] >> 6) as usize * 4),
                rd(&c[1], (a[1] >> 6) as usize * 4),
            );
            log.push(format!("c0+0x0,{},{x},{y},{}", a[3], a[4]));
            i64::from(a[3])
        }
    });
}
