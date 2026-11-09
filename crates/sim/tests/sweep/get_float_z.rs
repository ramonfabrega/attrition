//! `get_float_z@008942d0` under the emulator against the port (item 2807; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `get_float_z@008942d0`, run by `tools/emu/sweep_obj.py` on its row `get_float_z` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 8 random bytes of object, 0 argument(s), the global cell 0x00c06218 points at 16 random bytes; callee 0x00866560 (4 args) stubbed; stub 0 answers the address of 8 random bytes, a `p` window after the places; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; xmm 0 an integer in -4000..4000, loaded as that over four (after the other numbers on the row); xmm 1 an integer in -4000..4000, loaded as that over four (after the other numbers on the row); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("get_float_z") else {
        return;
    };
    let obj = super::Obj {
        name: "get_float_z",
        full: "get_float_z@008942d0",
        cells: 1,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // find_data_z (stub 0, handed the terrain cell's object, a local, and the two floats truncated to
            // ints, then 0) answers a pointer; the int it points at, as a float, is the result's bits.
            log.push(format!("c0+0x0,sp-0xc,{},{},0", a[1] / 4, a[2] / 4));
            i64::from((rd(&p[0], 0) as f32).to_bits() as i32)
        }
    });
}
