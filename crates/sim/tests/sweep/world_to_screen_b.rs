//! `MiscAccess::world_to_screen@00583a00` under the emulator against the port (item 2801; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `MiscAccess::world_to_screen@00583a00`, run by `tools/emu/sweep_obj.py` on its row `world_to_screen_b` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 4 argument(s), the global cell 0x00c06218 points at 32 random bytes; the global cell 0x00c06210 points at 320 random bytes; argument 2 names 8 random bytes; callee 0x00866560 (4 args) stubbed; callee 0x00583840 (5 args) stubbed; stub 1 answered from [0, 1, -1, 320, 240] (90%); `__cdecl`: the caller pops the arguments; stub 0 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 0 writes a value in -300..300 through its argument 0, printed after the answers (`o` in a place); stub 1 is `__cdecl`: the caller pops its arguments; `edx` the address of window `p0`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("world_to_screen_b") else {
        return;
    };
    let obj = super::Obj {
        name: "world_to_screen",
        full: "MiscAccess::world_to_screen@00583a00",
        cells: 2,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // `find_data_z` (stub 0, thiscall on the terrain, `o` = a[6] the height it writes through its
            // out-pointer) then `world_to_console` (stub 1, cdecl, `ecx` this, `edx` the word's window,
            // answering a[5]); the word `edx` names gains the signed short at +0x12a of the console
            // (the cell at 0xc06210).
            log.push(format!("c0+0x0,p0+0x0,sp-0x1c,{},{},1", a[0], a[1]));
            log.push(format!(
                "m+0x0,p0+0x0,{},{},{},p0+0x0,{}",
                a[0], a[1], a[6], a[3]
            ));
            let k = i32::from(i16::from_le_bytes([c[1][0x12a], c[1][0x12b]]));
            let w = rd(&p[0], 0).wrapping_add(k);
            wr(&mut p[0], 0, w);
            i64::from(a[5])
        }
    });
}
