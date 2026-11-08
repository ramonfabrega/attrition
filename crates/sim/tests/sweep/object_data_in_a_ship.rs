//! `ObjectData::in_a_ship@006440c0` under the emulator against the port (item 2325; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `ObjectData::in_a_ship@006440c0`, run by `tools/emu/sweep_obj.py` on its row `object_data_in_a_ship` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 64 random bytes of object, 0 argument(s), callee 0x00651a80 (1 args) stubbed; vtable slot +0x18 (0 args) stubbed; the word at `0xc0ab84+o[0]*0x1c` points at 4 random bytes less `max(0,s[0])*4` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p1+0x18` points at 544 random bytes less `0` (placed); stub 1 answered from [0, 1] (90%); the stubbed vtable laid at `p1+0`; stub 0 answers -2..6 every row; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 writes a value in 0..7 through its argument 0, printed after the answers (`o` in a place); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("object_data_in_a_ship") else {
        return;
    };
    let obj = super::Obj {
        name: "in_a_ship",
        full: "ObjectData::in_a_ship@006440c0",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // `get_inside` (stub 0) answers the index and writes the owner (`o`, a[2]); a
            // negative index answers 0. Else objects[owner][index]'s slot +0x18 (stub 1): zero
            // answers 0, else whether its type record (+0x18) holds 1 at +0x218.
            if a[0] < 0 {
                log.push("-".into());
                0
            } else {
                log.push("p1+0x0".into());
                if a[1] == 0 {
                    0
                } else {
                    i64::from(rd(&p[2], 0x218) == 1)
                }
            }
        }
    });
}
