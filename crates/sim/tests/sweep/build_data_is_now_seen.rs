//! `BuildData::is_now_seen@0062e130` under the emulator against the port (item 2241; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `BuildData::is_now_seen@0062e130`, run by `tools/emu/sweep_obj.py` on its row `build_data_is_now_seen` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 72 random bytes of object, 1 argument(s), vtable slot +0xdc (1 args) stubbed; callee 0x00642ca0 (1 args) stubbed; slot +0xdc laid as 0x00653750 or its stub, a flag after the answers; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("build_data_is_now_seen") else {
        return;
    };
    let obj = super::Obj {
        name: "is_now_seen",
        full: "BuildData::is_now_seen@0062e130",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let w = a[0];
            let bit = 1i32.wrapping_shl(w as u32);
            let wall = i64::from(a[2]);
            let z = i32::from(m[0x3a] as i8);
            if bit & i32::from(m[0x40] as i8) != 0 {
                1
            } else if a[3] == 1 {
                if w == i32::from(m[9]) || z == 0 {
                    wall
                } else if w < 0 || bit & z != 0 {
                    1
                } else {
                    wall
                }
            } else if a[1] != 0 {
                1
            } else {
                wall
            }
        }
    });
}
