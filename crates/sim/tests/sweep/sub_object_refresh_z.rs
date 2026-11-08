//! `SubObject::refresh_z@00661ed0` under the emulator against the port (item 2346; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `SubObject::refresh_z@00661ed0`, run by `tools/emu/sweep_obj.py` on its row `sub_object_refresh_z` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 0 argument(s), the global cell 0x00c06218 points at 16 random bytes; callee 0x00866560 (4 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 writes a value in -2147483648..2147483647 through its argument 0, printed after the answers (`o` in a place); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("sub_object_refresh_z") else {
        return;
    };
    let obj = super::Obj {
        name: "refresh_z",
        full: "SubObject::refresh_z@00661ed0",
        cells: 1,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // The terrain (cell 0xc06218) asked for the height into a local at the sub-object's
            // position — `+0x10` and `+0x14`, each kept masked `^ 0x63637` — with 1; the height
            // kept masked at `+0xc`, and 1 answered.
            let (x, y) = (rd(m, 0x10) ^ 0x63637, rd(m, 0x14) ^ 0x63637);
            log.push(format!("c0+0x0,sp-0xc,{x},{y},1"));
            wr(m, 0xc, a[1] ^ 0x63637);
            1
        }
    });
}
