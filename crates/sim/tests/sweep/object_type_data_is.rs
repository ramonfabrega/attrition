//! `ObjectTypeData::is@0065f7d0` under the emulator against the port (item 2200; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `ObjectTypeData::is@0065f7d0`, run by `tools/emu/sweep_obj.py` on its row `object_type_data_is` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 684 random bytes of object, 2 argument(s), vtable slot +0xf4 (2 args) stubbed; the word at `m+0x2a8` points at 64 random bytes less `0` (placed); the word at `m+0x28c` points at 64 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("object_type_data_is") else {
        return;
    };
    let obj = super::Obj {
        name: "is",
        full: "ObjectTypeData::is@0065f7d0",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let x = a[0];
            if rd(m, 4) == x {
                1
            } else if a[1] != 0 {
                let n = rd(m, 0x29c);
                if n == 0 {
                    i64::from(a[2])
                } else {
                    i64::from((0..n.max(0) as usize).any(|k| {
                        i32::from(u16::from_le_bytes([p[0][2 * k], p[0][2 * k + 1]])) == x
                    }))
                }
            } else if x < 0 {
                0
            } else {
                let n = rd(m, 0x280);
                if n == 0 {
                    i64::from(a[2])
                } else {
                    i64::from((0..n.max(0) as usize).any(|k| {
                        i32::from(u16::from_le_bytes([p[1][2 * k], p[1][2 * k + 1]])) == x
                    }))
                }
            }
        }
    });
}
